use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;

use super::{CvSource, CvSyncError, PDF_FILENAME, RemoteTag, TEX_FILENAME};

const GITHUB_API_VERSION: &str = "2022-11-28";
const TAGS_PER_PAGE: usize = 100;
const MAX_TAG_PAGES: usize = 100;
const MAX_DOWNLOAD_BYTES: usize = 20 * 1024 * 1024;

/// GitHub REST and raw-content adapter for a CV repository.
pub struct GitHubCvSource {
    agent: Agent,
    authorization: Option<String>,
    api_base_url: String,
    raw_base_url: String,
    owner: String,
    repository: String,
}

#[derive(Debug, Deserialize)]
struct ApiTag {
    name: String,
    commit: ApiCommit,
}

#[derive(Debug, Deserialize)]
struct ApiCommit {
    sha: String,
}

impl GitHubCvSource {
    /// Creates a GitHub source using the public API and raw-content service.
    pub fn new(
        owner: impl Into<String>,
        repository: impl Into<String>,
        token: Option<&str>,
    ) -> Result<Self, CvSyncError> {
        Self::with_base_urls(
            owner,
            repository,
            "https://api.github.com",
            "https://raw.githubusercontent.com",
            token,
        )
    }

    /// Creates a source with explicit endpoints.
    ///
    /// This is useful for GitHub Enterprise and deterministic transport tests.
    pub fn with_base_urls(
        owner: impl Into<String>,
        repository: impl Into<String>,
        api_base_url: impl Into<String>,
        raw_base_url: impl Into<String>,
        token: Option<&str>,
    ) -> Result<Self, CvSyncError> {
        let agent: Agent = Agent::config_builder()
            .user_agent("personal-site-cv-sync/1")
            .accept("application/vnd.github+json")
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        let authorization = token
            .filter(|token| !token.trim().is_empty())
            .map(|token| format!("Bearer {token}"));

        Ok(Self {
            agent,
            authorization,
            api_base_url: api_base_url.into().trim_end_matches('/').to_owned(),
            raw_base_url: raw_base_url.into().trim_end_matches('/').to_owned(),
            owner: owner.into(),
            repository: repository.into(),
        })
    }
}

impl CvSource for GitHubCvSource {
    fn tags(&self) -> Result<Vec<RemoteTag>, CvSyncError> {
        let url = format!(
            "{}/repos/{}/{}/tags",
            self.api_base_url, self.owner, self.repository
        );
        let mut tags = Vec::new();

        for page in 1..=MAX_TAG_PAGES {
            let mut request = self
                .agent
                .get(&url)
                .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
                .query("per_page", TAGS_PER_PAGE.to_string())
                .query("page", page.to_string());
            if let Some(authorization) = &self.authorization {
                request = request.header("Authorization", authorization);
            }
            let mut response = request.call().map_err(|error| {
                CvSyncError::Remote(format!("could not list GitHub tags: {error}"))
            })?;
            let api_tags: Vec<ApiTag> = response.body_mut().read_json().map_err(|error| {
                CvSyncError::Remote(format!("GitHub tag response was invalid: {error}"))
            })?;
            let page_length = api_tags.len();

            tags.extend(api_tags.into_iter().map(|tag| RemoteTag {
                name: tag.name,
                commit_sha: tag.commit.sha,
            }));

            if page_length < TAGS_PER_PAGE {
                return Ok(tags);
            }
        }

        Err(CvSyncError::Remote(format!(
            "GitHub tag listing exceeded {} tags",
            TAGS_PER_PAGE * MAX_TAG_PAGES
        )))
    }

    fn download(&self, commit_sha: &str, filename: &str) -> Result<Vec<u8>, CvSyncError> {
        if !matches!(filename, TEX_FILENAME | PDF_FILENAME) {
            return Err(CvSyncError::Remote(format!(
                "refusing unexpected CV filename {filename}"
            )));
        }
        super::validate_commit_sha(commit_sha).map_err(CvSyncError::Validation)?;

        let url = format!(
            "{}/{}/{}/{}/{}",
            self.raw_base_url, self.owner, self.repository, commit_sha, filename
        );
        let mut request = self.agent.get(&url);
        if let Some(authorization) = &self.authorization {
            request = request.header("Authorization", authorization);
        }
        let mut response = request.call().map_err(|error| {
            CvSyncError::Remote(format!("could not download {filename}: {error}"))
        })?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit((MAX_DOWNLOAD_BYTES + 1) as u64)
            .read_to_vec()
            .map_err(|error| CvSyncError::Remote(format!("could not read {filename}: {error}")))?;
        if bytes.len() > MAX_DOWNLOAD_BYTES {
            return Err(CvSyncError::Remote(format!(
                "download of {filename} exceeds {MAX_DOWNLOAD_BYTES} bytes"
            )));
        }
        Ok(bytes)
    }
}
