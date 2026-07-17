//! Native-only synchronization of the versioned CV source bundle.
//!
//! The synchronizer deliberately remains independent of the Leptos
//! presentation layer. It validates TeX/PDF structure and provenance, parses
//! the supported CV grammar, and commits a generated static Rust data module
//! in the same local bundle.

mod generator;
mod github;
mod manifest;
mod parser;
mod store;
mod synchronizer;

pub use generator::generate_cv_module;
pub use github::GitHubCvSource;
pub use manifest::{
    AssetManifest, CvManifest, RemoteTag, ValidatedBundle, parse_semantic_tag,
    select_highest_semantic_tag, validate_commit_sha, validate_pdf, validate_tex,
};
pub use parser::{CvParseError, parse_cv};
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

/// Repository-relative path of the generated, statically typed CV module.
pub const GENERATED_CV_PATH: &str = "src/generated_cv.rs";

/// Repository-relative path of the synchronized TeX source.
pub const TEX_REPOSITORY_PATH: &str = "public/cv/Hayden-Farrell-CV.tex";

/// Repository-relative path of the synchronized PDF.
pub const PDF_REPOSITORY_PATH: &str = "public/cv/Hayden-Farrell-CV.pdf";

/// Repository-relative path of the source manifest transaction marker.
pub const MANIFEST_REPOSITORY_PATH: &str = "public/cv/source-manifest.json";
