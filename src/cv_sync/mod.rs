//! Native-only synchronization of the versioned CV source bundle.
//!
//! The synchronizer deliberately remains independent of the Leptos
//! presentation layer. It treats the TeX and PDF as opaque release artifacts,
//! validates their structure and provenance, and commits them as one local
//! bundle.

mod github;
mod manifest;
mod store;
mod synchronizer;

pub use github::GitHubCvSource;
pub use manifest::{
    AssetManifest, CvManifest, RemoteTag, ValidatedBundle, parse_semantic_tag,
    select_highest_semantic_tag, validate_commit_sha, validate_pdf, validate_tex,
};
pub use store::CvBundleStore;
pub use synchronizer::{CvSource, CvSyncError, SyncOutcome, synchronize};

/// Repository that owns the canonical CV artifacts.
pub const UPSTREAM_REPOSITORY: &str = "osdesa/cv";

/// Filename used for the upstream and checked-in TeX source.
pub const TEX_FILENAME: &str = "Hayden-Farrell-CV.tex";

/// Filename used for the upstream and downloadable checked-in PDF.
pub const PDF_FILENAME: &str = "Hayden-Farrell-CV.pdf";

/// Filename used for the checked-in provenance manifest.
pub const MANIFEST_FILENAME: &str = "source-manifest.json";
