use std::fmt::Write as _;

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    GENERATED_CV_PATH, PDF_REPOSITORY_PATH, TEX_REPOSITORY_PATH, UPSTREAM_REPOSITORY,
    generate_cv_module, parse_cv,
};

const MANIFEST_SCHEMA_VERSION: u32 = 2;
const MAX_TEX_BYTES: usize = 2 * 1024 * 1024;
const MAX_PDF_BYTES: usize = 20 * 1024 * 1024;

/// A semantic Git tag and the immutable commit it resolved to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteTag {
    /// Original Git tag name, retained verbatim in the manifest.
    pub name: String,
    /// Commit SHA returned by GitHub for the tag.
    pub commit_sha: String,
}

/// Integrity metadata for one synchronized artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetManifest {
    /// Repository-relative artifact path.
    pub path: String,
    /// Artifact length in bytes.
    pub bytes: u64,
    /// Lowercase SHA-256 digest.
    pub sha256: String,
}

/// Provenance and integrity metadata for a synchronized CV bundle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CvManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Canonical GitHub `owner/repository` identifier.
    pub repository: String,
    /// Highest semantic-version tag selected during synchronization.
    pub tag: String,
    /// Immutable 40-character Git commit SHA resolved from `tag`.
    pub commit_sha: String,
    /// Integrity metadata for the TeX source.
    pub source: AssetManifest,
    /// Integrity metadata for the PDF artifact.
    pub pdf: AssetManifest,
    /// Integrity metadata for the generated Rust data module.
    pub generated: AssetManifest,
}

/// A downloaded bundle that has passed structural and integrity validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedBundle {
    /// Manifest generated from the validated bytes.
    pub manifest: CvManifest,
    /// Validated TeX bytes.
    pub tex: Vec<u8>,
    /// Validated PDF bytes.
    pub pdf: Vec<u8>,
    /// Deterministically generated Rust module bytes.
    pub generated: Vec<u8>,
}

impl CvManifest {
    /// Builds deterministic provenance metadata from validated artifact bytes.
    pub fn from_artifacts(tag: &RemoteTag, tex: &[u8], pdf: &[u8], generated: &[u8]) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            repository: UPSTREAM_REPOSITORY.to_owned(),
            tag: tag.name.clone(),
            commit_sha: tag.commit_sha.clone(),
            source: AssetManifest::new(TEX_REPOSITORY_PATH, tex),
            pdf: AssetManifest::new(PDF_REPOSITORY_PATH, pdf),
            generated: AssetManifest::new(GENERATED_CV_PATH, generated),
        }
    }

    /// Validates schema, provenance, repository paths and digest syntax.
    pub fn validate_metadata(&self) -> Result<(), String> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(format!(
                "unsupported manifest schema version {}",
                self.schema_version
            ));
        }
        if self.repository != UPSTREAM_REPOSITORY {
            return Err(format!("manifest repository must be {UPSTREAM_REPOSITORY}"));
        }
        parse_semantic_tag(&self.tag)
            .ok_or_else(|| format!("manifest tag is not semantic: {}", self.tag))?;
        validate_commit_sha(&self.commit_sha)?;
        self.source.validate(TEX_REPOSITORY_PATH)?;
        self.pdf.validate(PDF_REPOSITORY_PATH)?;
        self.generated.validate(GENERATED_CV_PATH)?;
        Ok(())
    }
}

impl AssetManifest {
    fn new(path: &str, bytes: &[u8]) -> Self {
        Self {
            path: path.to_owned(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(bytes),
        }
    }

    fn validate(&self, expected_path: &str) -> Result<(), String> {
        if self.path != expected_path {
            return Err(format!(
                "manifest path must be {expected_path}, found {}",
                self.path
            ));
        }
        if self.bytes == 0 {
            return Err(format!("{} must not be empty", self.path));
        }
        if !is_lowercase_hex(&self.sha256, 64) {
            return Err(format!(
                "{} SHA-256 must be 64 lowercase hexadecimal characters",
                self.path
            ));
        }
        Ok(())
    }

    pub(crate) fn matches(&self, bytes: &[u8]) -> bool {
        self.bytes == bytes.len() as u64 && self.sha256 == sha256_hex(bytes)
    }
}

impl ValidatedBundle {
    /// Validates downloaded files and creates their deterministic manifest.
    pub fn new(tag: RemoteTag, tex: Vec<u8>, pdf: Vec<u8>) -> Result<Self, String> {
        validate_commit_sha(&tag.commit_sha)?;
        validate_tex(&tex)?;
        validate_pdf(&pdf)?;
        let source = std::str::from_utf8(&tex).map_err(|_| "TeX source is not UTF-8".to_owned())?;
        let cv = parse_cv(source).map_err(|error| format!("LaTeX parse failed at {error}"))?;
        let generated = generate_cv_module(&cv, &tag);
        let manifest = CvManifest::from_artifacts(&tag, &tex, &pdf, &generated);
        Ok(Self {
            manifest,
            tex,
            pdf,
            generated,
        })
    }
}

/// Parses an optional `v`-prefixed semantic-version tag.
#[must_use]
pub fn parse_semantic_tag(tag: &str) -> Option<Version> {
    let candidate = tag
        .strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag);
    Version::parse(candidate).ok()
}

/// Selects the highest semantic-version tag, ignoring non-semantic tags.
///
/// Ties in semantic precedence (for example build metadata variants) are
/// resolved by tag name so results are deterministic regardless of API order.
#[must_use]
pub fn select_highest_semantic_tag(tags: &[RemoteTag]) -> Option<RemoteTag> {
    tags.iter()
        .filter_map(|tag| parse_semantic_tag(&tag.name).map(|version| (version, tag)))
        .max_by(|(left_version, left_tag), (right_version, right_tag)| {
            left_version
                .cmp(right_version)
                .then_with(|| left_tag.name.cmp(&right_tag.name))
        })
        .map(|(_, tag)| tag.clone())
}

/// Validates a full Git SHA-1 returned by the GitHub API.
pub fn validate_commit_sha(sha: &str) -> Result<(), String> {
    if is_lowercase_hex(sha, 40) {
        Ok(())
    } else {
        Err("commit SHA must be 40 lowercase hexadecimal characters".to_owned())
    }
}

/// Performs bounded structural validation of a TeX source artifact.
///
/// The synchronizer does not interpret CV content; these markers only reject
/// empty, truncated and obvious error-response files.
pub fn validate_tex(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("TeX source is empty".to_owned());
    }
    if bytes.len() > MAX_TEX_BYTES {
        return Err(format!("TeX source exceeds {MAX_TEX_BYTES} bytes"));
    }

    let text = std::str::from_utf8(bytes).map_err(|_| "TeX source is not UTF-8".to_owned())?;
    for marker in ["\\documentclass", "\\begin{document}", "\\end{document}"] {
        if !text.contains(marker) {
            return Err(format!("TeX source is missing {marker}"));
        }
    }
    let document_class = text
        .find("\\documentclass")
        .expect("required marker was checked above");
    let document_start = text
        .find("\\begin{document}")
        .expect("required marker was checked above");
    let document_end = text
        .find("\\end{document}")
        .expect("required marker was checked above");
    if !(document_class < document_start && document_start < document_end) {
        return Err("TeX document markers are out of order".to_owned());
    }
    Ok(())
}

/// Parses a bounded PDF and requires at least one page and a valid trailer.
pub fn validate_pdf(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_PDF_BYTES {
        return Err(format!("PDF exceeds {MAX_PDF_BYTES} bytes"));
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err("PDF header is missing".to_owned());
    }
    if bytes
        .rsplit(|byte| byte.is_ascii_whitespace())
        .find(|part| !part.is_empty())
        .is_none_or(|part| part != b"%%EOF")
    {
        return Err("PDF end-of-file marker is missing".to_owned());
    }

    let document = lopdf::Document::load_mem(bytes)
        .map_err(|error| format!("PDF structure is invalid: {error}"))?;
    if document.get_pages().is_empty() {
        return Err("PDF contains no pages".to_owned());
    }
    Ok(())
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn is_lowercase_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
