use std::{fs, path::Path};

use serde::Deserialize;

use super::{CONFIG_PATH, ProjectSyncError};

/// Declarative selection and output limits for project synchronization.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectSyncConfig {
    pub owner: String,
    pub list: String,
    pub topic: String,
    pub metadata_path: String,
    pub limit: usize,
    pub fallback_repositories: Vec<String>,
}

impl ProjectSyncConfig {
    /// Validates values that affect remote requests and generated output.
    pub fn validate(&self) -> Result<(), ProjectSyncError> {
        if self.owner.trim().is_empty() {
            return Err(ProjectSyncError::Configuration(
                "owner must not be empty".to_owned(),
            ));
        }
        if self.topic.trim().is_empty() {
            return Err(ProjectSyncError::Configuration(
                "topic must not be empty".to_owned(),
            ));
        }
        if self.list.trim().is_empty() {
            return Err(ProjectSyncError::Configuration(
                "list must not be empty".to_owned(),
            ));
        }
        if self.metadata_path != ".github/portfolio.toml" {
            return Err(ProjectSyncError::Configuration(
                "metadata_path must be .github/portfolio.toml".to_owned(),
            ));
        }
        if !(1..=4).contains(&self.limit) {
            return Err(ProjectSyncError::Configuration(
                "limit must be between 1 and 4".to_owned(),
            ));
        }
        if self.fallback_repositories.is_empty() {
            return Err(ProjectSyncError::Configuration(
                "at least one fallback repository is required".to_owned(),
            ));
        }
        for repository in &self.fallback_repositories {
            validate_repository_name(repository)?;
        }
        Ok(())
    }
}

/// Loads and validates `portfolio-projects.toml` from a repository root.
pub fn load_config(repository_root: &Path) -> Result<ProjectSyncConfig, ProjectSyncError> {
    let path = repository_root.join(CONFIG_PATH);
    let source = fs::read_to_string(&path).map_err(|error| {
        ProjectSyncError::Local(format!("could not read {}: {error}", path.display()))
    })?;
    let config: ProjectSyncConfig = toml::from_str(&source).map_err(|error| {
        ProjectSyncError::Configuration(format!("{} is invalid: {error}", path.display()))
    })?;
    config.validate()?;
    Ok(config)
}

fn validate_repository_name(repository: &str) -> Result<(), ProjectSyncError> {
    let mut segments = repository.split('/');
    let owner = segments.next().unwrap_or_default();
    let name = segments.next().unwrap_or_default();
    if owner.is_empty()
        || name.is_empty()
        || segments.next().is_some()
        || !repository.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
        })
    {
        return Err(ProjectSyncError::Configuration(format!(
            "invalid fallback repository {repository}"
        )));
    }
    Ok(())
}
