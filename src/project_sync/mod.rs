//! Native-only build-time synchronization of portfolio project data.

mod config;
mod generator;
mod github;
mod store;
mod synchronizer;

pub use config::{ProjectSyncConfig, load_config};
pub use generator::generate_projects_module;
pub use github::GitHubProjectSource;
pub use store::ProjectDataStore;
pub use synchronizer::{
    NormalizedProject, PortfolioMetadata, ProjectSource, ProjectSyncError, RemoteRepository,
    SelectionMethod, SyncOutcome, normalize_projects, parse_portfolio_metadata, synchronize,
};

/// Repository-relative synchronization configuration.
pub const CONFIG_PATH: &str = "portfolio-projects.toml";

/// Repository-relative generated project data.
pub const GENERATED_PROJECTS_PATH: &str = "src/generated_projects.rs";

/// Default artwork used when a repository supplies no featured image.
pub const DEFAULT_PROJECT_IMAGE: &str = "/images/project-default.svg";
