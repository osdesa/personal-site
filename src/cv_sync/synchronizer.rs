use std::fmt;

use semver::Version;
use thiserror::Error;

use super::{
    CvBundleStore, PDF_FILENAME, RemoteTag, TEX_FILENAME, ValidatedBundle, parse_semantic_tag,
    select_highest_semantic_tag,
};

/// Remote operations required by the synchronization application service.
///
/// Keeping this boundary small makes remote failures deterministic to test and
/// leaves the GitHub transport replaceable.
pub trait CvSource {
    /// Returns all upstream tags with their resolved commit SHAs.
    fn tags(&self) -> Result<Vec<RemoteTag>, CvSyncError>;

    /// Downloads one artifact from an immutable commit SHA.
    fn download(&self, commit_sha: &str, filename: &str) -> Result<Vec<u8>, CvSyncError>;
}

/// Result of a successful synchronization run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncOutcome {
    /// The local manifest already identifies the highest upstream tag.
    Unchanged { tag: String, commit_sha: String },
    /// A new complete bundle was committed locally.
    Updated { tag: String, commit_sha: String },
}

impl fmt::Display for SyncOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unchanged { tag, commit_sha } => {
                write!(formatter, "CV is already at {tag} ({commit_sha})")
            }
            Self::Updated { tag, commit_sha } => {
                write!(formatter, "updated CV to {tag} ({commit_sha})")
            }
        }
    }
}

/// Errors that abort synchronization before a new bundle can be committed.
#[derive(Debug, Error)]
pub enum CvSyncError {
    /// The upstream repository has no tag that follows semantic versioning.
    #[error("upstream repository has no semantic-version tags")]
    NoSemanticVersionTag,
    /// GitHub or another source could not be queried safely.
    #[error("remote operation failed: {0}")]
    Remote(String),
    /// The downloaded or checked-in bundle failed validation.
    #[error("CV bundle validation failed: {0}")]
    Validation(String),
    /// The checked-in manifest or bundle could not be read.
    #[error("local CV bundle failed: {0}")]
    Local(String),
    /// A tag name was reused for a different commit.
    #[error(
        "upstream tag {tag} moved from {local_sha} to {remote_sha}; refusing mutable provenance"
    )]
    MovedTag {
        /// Reused tag name.
        tag: String,
        /// SHA recorded in the checked-in manifest.
        local_sha: String,
        /// SHA currently returned by GitHub.
        remote_sha: String,
    },
    /// Upstream no longer exposes a version at least as new as the local one.
    #[error(
        "upstream highest version {remote} is older than local version {local}; refusing rollback"
    )]
    VersionRollback {
        /// Version in the checked-in manifest.
        local: Version,
        /// Highest version reported upstream.
        remote: Version,
    },
}

/// Synchronizes the highest upstream semantic version into the local bundle.
///
/// All network and validation work completes before `store` starts its
/// transaction. Any error leaves the previously committed files untouched.
pub fn synchronize(
    source: &impl CvSource,
    store: &CvBundleStore,
) -> Result<SyncOutcome, CvSyncError> {
    let tags = source.tags()?;
    let selected = select_highest_semantic_tag(&tags).ok_or(CvSyncError::NoSemanticVersionTag)?;
    super::validate_commit_sha(&selected.commit_sha).map_err(CvSyncError::Validation)?;

    let local = store.load_validated_manifest()?;
    if let Some(manifest) = local {
        let local_version = parse_semantic_tag(&manifest.tag)
            .expect("validated manifests always contain a semantic tag");
        let remote_version = parse_semantic_tag(&selected.name)
            .expect("selected tags always contain a semantic tag");

        if remote_version < local_version {
            return Err(CvSyncError::VersionRollback {
                local: local_version,
                remote: remote_version,
            });
        }
        if manifest.tag == selected.name {
            if manifest.commit_sha != selected.commit_sha {
                return Err(CvSyncError::MovedTag {
                    tag: selected.name,
                    local_sha: manifest.commit_sha,
                    remote_sha: selected.commit_sha,
                });
            }
            return Ok(SyncOutcome::Unchanged {
                tag: selected.name,
                commit_sha: selected.commit_sha,
            });
        }
    }

    let tex = source.download(&selected.commit_sha, TEX_FILENAME)?;
    let pdf = source.download(&selected.commit_sha, PDF_FILENAME)?;
    let bundle =
        ValidatedBundle::new(selected.clone(), tex, pdf).map_err(CvSyncError::Validation)?;
    store.commit(&bundle)?;

    Ok(SyncOutcome::Updated {
        tag: selected.name,
        commit_sha: selected.commit_sha,
    })
}
