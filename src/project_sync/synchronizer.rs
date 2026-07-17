use std::{cmp::Ordering, collections::HashSet};

use serde::Deserialize;
use thiserror::Error;

use super::{DEFAULT_PROJECT_IMAGE, ProjectDataStore, ProjectSyncConfig, generate_projects_module};

/// A repository response containing only fields used by the portfolio.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RemoteRepository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub homepage: Option<String>,
    pub created_at: String,
    pub language: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub archived: bool,
    pub fork: bool,
}

/// Optional `.github/portfolio.toml` values supplied by a selected repository.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PortfolioMetadata {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub date: Option<String>,
    pub status: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub highlights: Option<Vec<String>>,
    pub image: Option<String>,
    pub demo_url: Option<String>,
    pub show_repository: Option<bool>,
    #[serde(default)]
    pub include_archived: bool,
    #[serde(default)]
    pub include_fork: bool,
}

/// Fully validated, owned data used to generate the static project module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedProject {
    pub id: String,
    pub repository: String,
    pub title: String,
    pub summary: String,
    pub private: bool,
    pub created_date: String,
    pub portfolio_date: Option<String>,
    pub status: Option<String>,
    pub technologies: Vec<String>,
    pub highlights: Vec<String>,
    pub image_url: String,
    pub repository_url: Option<String>,
    pub demo_url: Option<String>,
}

/// Selection route used for observable CLI output and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMethod {
    NamedList,
    Topic,
    AllowlistFallback,
}

/// Remote operations needed by the application service.
pub trait ProjectSource {
    fn named_list_repositories(
        &self,
        list_name: &str,
    ) -> Result<Option<Vec<String>>, ProjectSyncError>;
    fn accessible_repositories(&self) -> Result<Vec<RemoteRepository>, ProjectSyncError>;
    fn repository(&self, full_name: &str) -> Result<RemoteRepository, ProjectSyncError>;
    fn portfolio_metadata(&self, full_name: &str) -> Result<Option<String>, ProjectSyncError>;
}

/// Result of a successful synchronization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Unchanged {
        projects: usize,
        selection: SelectionMethod,
    },
    Updated {
        projects: usize,
        selection: SelectionMethod,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectSyncError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("remote error: {0}")]
    Remote(String),
    #[error("metadata validation error: {0}")]
    Validation(String),
    #[error("local data error: {0}")]
    Local(String),
}

/// Parses strict optional portfolio metadata.
pub fn parse_portfolio_metadata(source: &str) -> Result<PortfolioMetadata, ProjectSyncError> {
    let metadata: PortfolioMetadata = toml::from_str(source).map_err(|error| {
        ProjectSyncError::Validation(format!("invalid portfolio TOML: {error}"))
    })?;
    validate_metadata(&metadata)?;
    Ok(metadata)
}

/// Fetches, normalizes, sorts and atomically publishes project data.
pub fn synchronize(
    source: &dyn ProjectSource,
    store: &ProjectDataStore,
    config: &ProjectSyncConfig,
) -> Result<SyncOutcome, ProjectSyncError> {
    config.validate()?;
    let named_repositories = source.named_list_repositories(&config.list)?;
    let (selected, selection) = if let Some(repositories) =
        named_repositories.filter(|repositories| !repositories.is_empty())
    {
        let selected = repositories
            .iter()
            .map(|repository| source.repository(repository))
            .collect::<Result<Vec<_>, _>>()?;
        (selected, SelectionMethod::NamedList)
    } else {
        let accessible = source.accessible_repositories()?;
        let topic_repositories: Vec<_> = accessible
            .into_iter()
            .filter(|repository| {
                repository
                    .full_name
                    .split('/')
                    .next()
                    .is_some_and(|owner| owner.eq_ignore_ascii_case(&config.owner))
            })
            .filter(|repository| {
                repository
                    .topics
                    .iter()
                    .any(|topic| topic.eq_ignore_ascii_case(&config.topic))
            })
            .collect();
        if topic_repositories.is_empty() {
            let selected = config
                .fallback_repositories
                .iter()
                .map(|repository| source.repository(repository))
                .collect::<Result<Vec<_>, _>>()?;
            (selected, SelectionMethod::AllowlistFallback)
        } else {
            (topic_repositories, SelectionMethod::Topic)
        }
    };

    let mut candidates = Vec::with_capacity(selected.len());
    let mut seen = HashSet::new();
    for repository in selected {
        if !seen.insert(repository.full_name.to_ascii_lowercase()) {
            continue;
        }
        let metadata = source
            .portfolio_metadata(&repository.full_name)?
            .map(|contents| parse_portfolio_metadata(&contents))
            .transpose()?;
        candidates.push((repository, metadata.unwrap_or_default()));
    }

    let projects = normalize_projects(candidates, config.limit)?;
    let generated = generate_projects_module(&projects);
    let changed = store.commit_if_changed(&generated)?;
    Ok(if changed {
        SyncOutcome::Updated {
            projects: projects.len(),
            selection,
        }
    } else {
        SyncOutcome::Unchanged {
            projects: projects.len(),
            selection,
        }
    })
}

/// Applies filtering, fallback and deterministic ordering rules.
pub fn normalize_projects(
    candidates: Vec<(RemoteRepository, PortfolioMetadata)>,
    limit: usize,
) -> Result<Vec<NormalizedProject>, ProjectSyncError> {
    let mut projects = Vec::new();
    for (repository, metadata) in candidates {
        validate_metadata(&metadata)?;
        if (repository.archived && !metadata.include_archived)
            || (repository.fork && !metadata.include_fork)
        {
            continue;
        }
        validate_https_url(&repository.html_url, "repository URL")?;
        let created_date = parse_github_date(&repository.created_at)?;
        let portfolio_date = metadata.date.clone();
        if let Some(date) = &portfolio_date {
            validate_date(date, "portfolio date")?;
        }
        let title = non_empty(metadata.title, || format_repository_name(&repository.name));
        let owner = repository
            .full_name
            .split('/')
            .next()
            .unwrap_or("repository owner");
        let summary = non_empty(metadata.summary.or(repository.description), || {
            format!("A software project from {owner}.")
        });
        let mut technologies = metadata.technologies.unwrap_or_else(|| {
            let mut values = repository.topics.clone();
            if let Some(language) = repository.language.clone() {
                values.push(language);
            }
            values
        });
        normalize_list(&mut technologies);
        if technologies.is_empty() {
            technologies.push("Software Engineering".to_owned());
        }
        let mut highlights = metadata.highlights.unwrap_or_default();
        normalize_list(&mut highlights);
        let demo_url = metadata
            .demo_url
            .or(repository.homepage)
            .and_then(non_blank);
        if let Some(url) = &demo_url {
            validate_https_url(url, "demo URL")?;
        }
        let image_url = metadata
            .image
            .and_then(non_blank)
            .unwrap_or_else(|| DEFAULT_PROJECT_IMAGE.to_owned());
        validate_image_url(&image_url)?;
        let show_repository = metadata.show_repository.unwrap_or(!repository.private);
        let id = repository
            .full_name
            .to_ascii_lowercase()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect();

        projects.push(NormalizedProject {
            id,
            repository: repository.full_name,
            title,
            summary,
            private: repository.private,
            created_date,
            portfolio_date,
            status: metadata.status.and_then(non_blank),
            technologies,
            highlights,
            image_url,
            repository_url: show_repository.then_some(repository.html_url),
            demo_url,
        });
    }

    projects.sort_by(compare_projects);
    projects.truncate(limit.min(4));
    Ok(projects)
}

fn compare_projects(left: &NormalizedProject, right: &NormalizedProject) -> Ordering {
    let left_date = left.portfolio_date.as_ref().unwrap_or(&left.created_date);
    let right_date = right.portfolio_date.as_ref().unwrap_or(&right.created_date);
    right_date
        .cmp(left_date)
        .then_with(|| right.created_date.cmp(&left.created_date))
        .then_with(|| {
            left.repository
                .to_ascii_lowercase()
                .cmp(&right.repository.to_ascii_lowercase())
        })
}

fn validate_metadata(metadata: &PortfolioMetadata) -> Result<(), ProjectSyncError> {
    for (label, value) in [
        ("title", metadata.title.as_deref()),
        ("summary", metadata.summary.as_deref()),
        ("status", metadata.status.as_deref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(ProjectSyncError::Validation(format!(
                "{label} must not be blank"
            )));
        }
    }
    if let Some(date) = &metadata.date {
        validate_date(date, "portfolio date")?;
    }
    if let Some(url) = metadata.demo_url.as_deref() {
        validate_https_url(url, "demo URL")?;
    }
    if let Some(url) = metadata.image.as_deref() {
        validate_image_url(url)?;
    }
    Ok(())
}

fn parse_github_date(value: &str) -> Result<String, ProjectSyncError> {
    let date = value.get(..10).ok_or_else(|| {
        ProjectSyncError::Validation(format!("invalid GitHub creation date {value}"))
    })?;
    validate_date(date, "GitHub creation date")?;
    Ok(date.to_owned())
}

fn validate_date(value: &str, label: &str) -> Result<(), ProjectSyncError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return Err(ProjectSyncError::Validation(format!(
            "{label} must use YYYY-MM-DD"
        )));
    }
    let year: u16 = value[..4].parse().unwrap_or_default();
    let month: u8 = value[5..7].parse().unwrap_or_default();
    let day: u8 = value[8..10].parse().unwrap_or_default();
    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    if day == 0 || day > days_in_month {
        return Err(ProjectSyncError::Validation(format!(
            "invalid {label} {value}"
        )));
    }
    Ok(())
}

fn validate_https_url(value: &str, label: &str) -> Result<(), ProjectSyncError> {
    if !value.starts_with("https://") || value.chars().any(char::is_whitespace) {
        return Err(ProjectSyncError::Validation(format!(
            "{label} must be an HTTPS URL"
        )));
    }
    Ok(())
}

fn validate_image_url(value: &str) -> Result<(), ProjectSyncError> {
    if value.starts_with('/') && !value.starts_with("//") && !value.chars().any(char::is_whitespace)
    {
        return Ok(());
    }
    validate_https_url(value, "image URL")
}

fn non_empty(value: Option<String>, fallback: impl FnOnce() -> String) -> String {
    value.and_then(non_blank).unwrap_or_else(fallback)
}

fn non_blank(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn normalize_list(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain_mut(|value| {
        *value = value.trim().to_owned();
        !value.is_empty() && seen.insert(value.to_ascii_lowercase())
    });
}

fn format_repository_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}
