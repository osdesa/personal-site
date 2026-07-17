use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use personal_site::cv_sync::{
    CvBundleStore, CvSource, CvSyncError, MANIFEST_FILENAME, PDF_FILENAME, RemoteTag, SyncOutcome,
    TEX_FILENAME, ValidatedBundle, synchronize,
};
use tempfile::TempDir;

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PDF: &[u8] = include_bytes!("../public/cv/Hayden-Farrell-CV.pdf");

#[derive(Clone)]
struct FakeSource {
    tags: Result<Vec<RemoteTag>, String>,
    files: HashMap<(&'static str, &'static str), Result<Vec<u8>, String>>,
    downloads: Arc<Mutex<Vec<(String, String)>>>,
}

impl FakeSource {
    fn new(tags: Vec<RemoteTag>) -> Self {
        Self {
            tags: Ok(tags),
            files: HashMap::new(),
            downloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn file(
        mut self,
        sha: &'static str,
        filename: &'static str,
        result: Result<Vec<u8>, String>,
    ) -> Self {
        self.files.insert((sha, filename), result);
        self
    }
}

impl CvSource for FakeSource {
    fn tags(&self) -> Result<Vec<RemoteTag>, CvSyncError> {
        self.tags.clone().map_err(CvSyncError::Remote)
    }

    fn download(&self, commit_sha: &str, filename: &str) -> Result<Vec<u8>, CvSyncError> {
        self.downloads
            .lock()
            .unwrap()
            .push((commit_sha.to_owned(), filename.to_owned()));
        self.files
            .get(&(commit_sha, filename))
            .cloned()
            .unwrap_or_else(|| Err(format!("unexpected download: {commit_sha}/{filename}")))
            .map_err(CvSyncError::Remote)
    }
}

fn tag(name: &str, commit_sha: &str) -> RemoteTag {
    RemoteTag {
        name: name.to_owned(),
        commit_sha: commit_sha.to_owned(),
    }
}

fn tex(label: &str) -> Vec<u8> {
    format!("\\documentclass{{article}}\n\\begin{{document}}\n{label}\n\\end{{document}}\n")
        .into_bytes()
}

fn commit_bundle(root: &Path, version: &str, sha: &str, source: Vec<u8>) {
    let bundle = ValidatedBundle::new(tag(version, sha), source, PDF.to_vec()).unwrap();
    CvBundleStore::new(root).commit(&bundle).unwrap();
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    [TEX_FILENAME, PDF_FILENAME, MANIFEST_FILENAME]
        .into_iter()
        .map(|filename| {
            (
                filename.to_owned(),
                fs::read(root.join("public").join("cv").join(filename)).unwrap(),
            )
        })
        .collect()
}

#[test]
fn update_downloads_from_the_selected_commit_and_commits_one_valid_bundle() {
    let root = TempDir::new().unwrap();
    let source = FakeSource::new(vec![tag("v1.9.0", SHA_A), tag("v1.10.0", SHA_B)])
        .file(SHA_B, TEX_FILENAME, Ok(tex("new")))
        .file(SHA_B, PDF_FILENAME, Ok(PDF.to_vec()));

    let outcome = synchronize(&source, &CvBundleStore::new(root.path())).unwrap();

    assert_eq!(
        outcome,
        SyncOutcome::Updated {
            tag: "v1.10.0".to_owned(),
            commit_sha: SHA_B.to_owned(),
        }
    );
    assert_eq!(
        *source.downloads.lock().unwrap(),
        vec![
            (SHA_B.to_owned(), TEX_FILENAME.to_owned()),
            (SHA_B.to_owned(), PDF_FILENAME.to_owned()),
        ]
    );
    let manifest = CvBundleStore::new(root.path())
        .load_validated_manifest()
        .unwrap()
        .unwrap();
    assert_eq!(manifest.tag, "v1.10.0");
    assert_eq!(manifest.commit_sha, SHA_B);
    assert_eq!(
        fs::read(root.path().join("public/cv/Hayden-Farrell-CV.tex")).unwrap(),
        tex("new")
    );
}

#[test]
fn unchanged_tag_performs_no_downloads_or_writes() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v1.0.0", SHA_A, tex("current"));
    let before = snapshot(root.path());
    let metadata_before: Vec<_> = before
        .iter()
        .map(|(filename, _)| {
            fs::metadata(root.path().join("public/cv").join(filename))
                .unwrap()
                .modified()
                .unwrap()
        })
        .collect();
    let source = FakeSource::new(vec![tag("v1.0.0", SHA_A)]);

    let outcome = synchronize(&source, &CvBundleStore::new(root.path())).unwrap();

    assert!(matches!(outcome, SyncOutcome::Unchanged { .. }));
    assert!(source.downloads.lock().unwrap().is_empty());
    assert_eq!(snapshot(root.path()), before);
    let metadata_after: Vec<_> = before
        .iter()
        .map(|(filename, _)| {
            fs::metadata(root.path().join("public/cv").join(filename))
                .unwrap()
                .modified()
                .unwrap()
        })
        .collect();
    assert_eq!(metadata_after, metadata_before);
}

#[test]
fn failed_second_download_preserves_the_complete_current_version() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v1.0.0", SHA_A, tex("current"));
    let before = snapshot(root.path());
    let source = FakeSource::new(vec![tag("v2.0.0", SHA_B)])
        .file(SHA_B, TEX_FILENAME, Ok(tex("new")))
        .file(SHA_B, PDF_FILENAME, Err("upstream unavailable".to_owned()));

    assert!(synchronize(&source, &CvBundleStore::new(root.path())).is_err());
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn invalid_download_preserves_the_complete_current_version() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v1.0.0", SHA_A, tex("current"));
    let before = snapshot(root.path());
    let source = FakeSource::new(vec![tag("v2.0.0", SHA_B)])
        .file(SHA_B, TEX_FILENAME, Ok(tex("new")))
        .file(
            SHA_B,
            PDF_FILENAME,
            Ok(b"<html>rate limited</html>".to_vec()),
        );

    assert!(matches!(
        synchronize(&source, &CvBundleStore::new(root.path())),
        Err(CvSyncError::Validation(_))
    ));
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn lock_failure_happens_before_replacement_and_preserves_current_version() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v1.0.0", SHA_A, tex("current"));
    let before = snapshot(root.path());
    fs::write(root.path().join("public/cv/.cv-sync.lock"), b"held").unwrap();
    let source = FakeSource::new(vec![tag("v2.0.0", SHA_B)])
        .file(SHA_B, TEX_FILENAME, Ok(tex("new")))
        .file(SHA_B, PDF_FILENAME, Ok(PDF.to_vec()));

    assert!(matches!(
        synchronize(&source, &CvBundleStore::new(root.path())),
        Err(CvSyncError::Local(_))
    ));
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn moved_tag_and_version_rollback_are_rejected_without_downloads() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v2.0.0", SHA_A, tex("current"));
    let before = snapshot(root.path());

    let moved = FakeSource::new(vec![tag("v2.0.0", SHA_B)]);
    assert!(matches!(
        synchronize(&moved, &CvBundleStore::new(root.path())),
        Err(CvSyncError::MovedTag { .. })
    ));
    assert!(moved.downloads.lock().unwrap().is_empty());

    let rollback = FakeSource::new(vec![tag("v1.9.0", SHA_B)]);
    assert!(matches!(
        synchronize(&rollback, &CvBundleStore::new(root.path())),
        Err(CvSyncError::VersionRollback { .. })
    ));
    assert!(rollback.downloads.lock().unwrap().is_empty());
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn corrupted_current_bundle_blocks_even_an_unchanged_tag() {
    let root = TempDir::new().unwrap();
    commit_bundle(root.path(), "v1.0.0", SHA_A, tex("current"));
    fs::write(
        root.path().join("public/cv/Hayden-Farrell-CV.tex"),
        tex("tampered"),
    )
    .unwrap();
    let source = FakeSource::new(vec![tag("v1.0.0", SHA_A)]);

    assert!(matches!(
        synchronize(&source, &CvBundleStore::new(root.path())),
        Err(CvSyncError::Local(_))
    ));
    assert!(source.downloads.lock().unwrap().is_empty());
}

#[test]
fn checked_in_cv_bundle_matches_its_provenance_manifest() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let manifest = CvBundleStore::new(repository_root)
        .load_validated_manifest()
        .unwrap()
        .expect("the repository must contain a synchronized CV manifest");

    assert_eq!(manifest.repository, "osdesa/cv");
    assert_eq!(manifest.source.filename, TEX_FILENAME);
    assert_eq!(manifest.pdf.filename, PDF_FILENAME);
}
