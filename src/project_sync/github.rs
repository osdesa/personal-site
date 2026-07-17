use std::time::Duration;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::json;
use ureq::Agent;

use super::{ProjectSource, ProjectSyncError, RemoteRepository};

const GITHUB_API_VERSION: &str = "2022-11-28";
const REPOSITORIES_PER_PAGE: usize = 100;
const MAX_REPOSITORY_PAGES: usize = 100;
const MAX_METADATA_BYTES: usize = 256 * 1024;
const GRAPHQL_PAGE_SIZE: usize = 100;
const MAX_GRAPHQL_PAGES: usize = 100;

const LISTS_QUERY: &str = r#"
query PortfolioLists($after: String) {
  viewer {
    lists(first: 100, after: $after) {
      nodes {
        id
        name
        items(first: 100) {
          nodes { ... on Repository { nameWithOwner } }
          pageInfo { hasNextPage endCursor }
        }
      }
      pageInfo { hasNextPage endCursor }
    }
  }
}"#;

const LIST_ITEMS_QUERY: &str = r#"
query PortfolioListItems($id: ID!, $after: String) {
  node(id: $id) {
    ... on UserList {
      items(first: 100, after: $after) {
        nodes { ... on Repository { nameWithOwner } }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}"#;

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ListsData {
    viewer: ListsViewer,
}

#[derive(Debug, Deserialize)]
struct ListsViewer {
    lists: UserListConnection,
}

#[derive(Debug, Deserialize)]
struct UserListConnection {
    nodes: Vec<Option<UserList>>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct UserList {
    id: String,
    name: String,
    items: UserListItemsConnection,
}

#[derive(Debug, Deserialize)]
struct UserListItemsConnection {
    nodes: Vec<Option<UserListRepository>>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct UserListRepository {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListItemsData {
    node: Option<ListItemsNode>,
}

#[derive(Debug, Deserialize)]
struct ListItemsNode {
    items: UserListItemsConnection,
}

/// Authenticated GitHub REST adapter for repository metadata and override files.
pub struct GitHubProjectSource {
    agent: Agent,
    authorization: Option<String>,
    api_base_url: String,
    owner: String,
    metadata_path: String,
}

impl GitHubProjectSource {
    /// Creates a production GitHub source.
    pub fn new(
        owner: impl Into<String>,
        metadata_path: impl Into<String>,
        token: Option<&str>,
    ) -> Self {
        Self::with_base_url(owner, metadata_path, "https://api.github.com", token)
    }

    /// Creates a source with a custom API endpoint for deterministic tests.
    pub fn with_base_url(
        owner: impl Into<String>,
        metadata_path: impl Into<String>,
        api_base_url: impl Into<String>,
        token: Option<&str>,
    ) -> Self {
        let agent: Agent = Agent::config_builder()
            .user_agent("personal-site-project-sync/1")
            .accept("application/vnd.github+json")
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        Self {
            agent,
            authorization: token
                .filter(|token| !token.trim().is_empty())
                .map(|token| format!("Bearer {token}")),
            api_base_url: api_base_url.into().trim_end_matches('/').to_owned(),
            owner: owner.into(),
            metadata_path: metadata_path.into(),
        }
    }

    fn request(&self, url: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
        let mut request = self
            .agent
            .get(url)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
        if let Some(authorization) = &self.authorization {
            request = request.header("Authorization", authorization);
        }
        request
    }

    fn read_repository(&self, url: &str) -> Result<RemoteRepository, ProjectSyncError> {
        let mut response = self.request(url).call().map_err(|error| {
            ProjectSyncError::Remote(format!("could not fetch repository metadata: {error}"))
        })?;
        response.body_mut().read_json().map_err(|error| {
            ProjectSyncError::Remote(format!("GitHub repository response was invalid: {error}"))
        })
    }

    fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, ProjectSyncError> {
        let url = format!("{}/graphql", self.api_base_url);
        let mut request = self
            .agent
            .post(&url)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
        if let Some(authorization) = &self.authorization {
            request = request.header("Authorization", authorization);
        }
        let mut response = request
            .send_json(json!({ "query": query, "variables": variables }))
            .map_err(|error| {
                ProjectSyncError::Remote(format!("could not query GitHub user lists: {error}"))
            })?;
        let response: GraphQlResponse<T> = response.body_mut().read_json().map_err(|error| {
            ProjectSyncError::Remote(format!("GitHub GraphQL response was invalid: {error}"))
        })?;
        if !response.errors.is_empty() {
            return Err(ProjectSyncError::Remote(format!(
                "GitHub GraphQL rejected the user-list query: {}",
                response
                    .errors
                    .iter()
                    .map(|error| error.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            )));
        }
        response.data.ok_or_else(|| {
            ProjectSyncError::Remote("GitHub GraphQL returned no user-list data".to_owned())
        })
    }

    fn remaining_list_items(
        &self,
        list_id: &str,
        mut page_info: PageInfo,
    ) -> Result<Vec<String>, ProjectSyncError> {
        let mut repositories = Vec::new();
        for _ in 1..MAX_GRAPHQL_PAGES {
            if !page_info.has_next_page {
                return Ok(repositories);
            }
            let cursor = page_info.end_cursor.ok_or_else(|| {
                ProjectSyncError::Remote("GitHub list pagination omitted its end cursor".to_owned())
            })?;
            let data: ListItemsData =
                self.graphql(LIST_ITEMS_QUERY, json!({ "id": list_id, "after": cursor }))?;
            let node = data.node.ok_or_else(|| {
                ProjectSyncError::Remote("GitHub portfolio list disappeared".to_owned())
            })?;
            repositories.extend(
                node.items
                    .nodes
                    .into_iter()
                    .flatten()
                    .map(|repository| repository.name_with_owner),
            );
            page_info = node.items.page_info;
        }
        Err(ProjectSyncError::Remote(format!(
            "GitHub portfolio list exceeded {} repositories",
            GRAPHQL_PAGE_SIZE * MAX_GRAPHQL_PAGES
        )))
    }
}

impl ProjectSource for GitHubProjectSource {
    fn named_list_repositories(
        &self,
        list_name: &str,
    ) -> Result<Option<Vec<String>>, ProjectSyncError> {
        if self.authorization.is_none() {
            return Ok(None);
        }
        let mut cursor = None;
        for _ in 0..MAX_GRAPHQL_PAGES {
            let data: ListsData = self.graphql(LISTS_QUERY, json!({ "after": cursor }))?;
            let connection = data.viewer.lists;
            if let Some(list) = connection
                .nodes
                .into_iter()
                .flatten()
                .find(|list| list.name.eq_ignore_ascii_case(list_name))
            {
                let mut repositories: Vec<_> = list
                    .items
                    .nodes
                    .into_iter()
                    .flatten()
                    .map(|repository| repository.name_with_owner)
                    .collect();
                repositories.extend(self.remaining_list_items(&list.id, list.items.page_info)?);
                return Ok(Some(repositories));
            }
            if !connection.page_info.has_next_page {
                return Ok(None);
            }
            cursor = connection.page_info.end_cursor;
            if cursor.is_none() {
                return Err(ProjectSyncError::Remote(
                    "GitHub user-list pagination omitted its end cursor".to_owned(),
                ));
            }
        }
        Err(ProjectSyncError::Remote(format!(
            "GitHub user-list listing exceeded {} lists",
            GRAPHQL_PAGE_SIZE * MAX_GRAPHQL_PAGES
        )))
    }

    fn accessible_repositories(&self) -> Result<Vec<RemoteRepository>, ProjectSyncError> {
        let url = if self.authorization.is_some() {
            format!("{}/user/repos", self.api_base_url)
        } else {
            format!("{}/users/{}/repos", self.api_base_url, self.owner)
        };
        let mut repositories = Vec::new();
        for page in 1..=MAX_REPOSITORY_PAGES {
            let mut response = self
                .request(&url)
                .query("visibility", "all")
                .query("affiliation", "owner")
                .query("per_page", REPOSITORIES_PER_PAGE.to_string())
                .query("page", page.to_string())
                .call()
                .map_err(|error| {
                    ProjectSyncError::Remote(format!(
                        "could not list accessible GitHub repositories: {error}"
                    ))
                })?;
            let page_repositories: Vec<RemoteRepository> =
                response.body_mut().read_json().map_err(|error| {
                    ProjectSyncError::Remote(format!(
                        "GitHub repository listing was invalid: {error}"
                    ))
                })?;
            let page_length = page_repositories.len();
            repositories.extend(page_repositories);
            if page_length < REPOSITORIES_PER_PAGE {
                return Ok(repositories);
            }
        }
        Err(ProjectSyncError::Remote(format!(
            "GitHub repository listing exceeded {} entries",
            REPOSITORIES_PER_PAGE * MAX_REPOSITORY_PAGES
        )))
    }

    fn repository(&self, full_name: &str) -> Result<RemoteRepository, ProjectSyncError> {
        self.read_repository(&format!("{}/repos/{full_name}", self.api_base_url))
    }

    fn portfolio_metadata(&self, full_name: &str) -> Result<Option<String>, ProjectSyncError> {
        let url = format!(
            "{}/repos/{full_name}/contents/{}",
            self.api_base_url, self.metadata_path
        );
        let request = self
            .request(&url)
            .header("Accept", "application/vnd.github.raw+json");
        let mut response = match request.call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(404)) => return Ok(None),
            Err(error) => {
                return Err(ProjectSyncError::Remote(format!(
                    "could not fetch {full_name}/{}: {error}",
                    self.metadata_path
                )));
            }
        };
        let bytes = response
            .body_mut()
            .with_config()
            .limit((MAX_METADATA_BYTES + 1) as u64)
            .read_to_vec()
            .map_err(|error| {
                ProjectSyncError::Remote(format!("could not read portfolio metadata: {error}"))
            })?;
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(ProjectSyncError::Validation(format!(
                "{full_name}/{} exceeds {MAX_METADATA_BYTES} bytes",
                self.metadata_path
            )));
        }
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| ProjectSyncError::Validation("portfolio metadata is not UTF-8".to_owned()))
    }
}
