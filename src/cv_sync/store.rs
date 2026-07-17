use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use super::{
    CvManifest, CvSyncError, GENERATED_CV_PATH, MANIFEST_FILENAME, MANIFEST_REPOSITORY_PATH,
    PDF_REPOSITORY_PATH, RemoteTag, TEX_REPOSITORY_PATH, ValidatedBundle, generate_cv_module,
    parse_cv, validate_pdf, validate_tex,
};

const LOCK_FILENAME: &str = ".cv-sync.lock";

/// Filesystem boundary for the checked-in CV source bundle.
#[derive(Clone, Debug)]
pub struct CvBundleStore {
    repository_root: PathBuf,
    directory: PathBuf,
}

impl CvBundleStore {
    /// Creates a store rooted at `<repository_root>/public/cv`.
    #[must_use]
    pub fn new(repository_root: impl AsRef<Path>) -> Self {
        let repository_root = repository_root.as_ref().to_path_buf();
        Self {
            directory: repository_root.join("public").join("cv"),
            repository_root,
        }
    }

    /// Returns the directory containing the imported source artifacts and
    /// manifest. The generated module is stored under `src/`.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Loads and fully validates the checked-in manifest and referenced files.
    ///
    /// A missing manifest means the repository has not been bootstrapped yet.
    /// Any other inconsistency is an error and will block replacement.
    pub fn load_validated_manifest(&self) -> Result<Option<CvManifest>, CvSyncError> {
        let manifest_path = self.directory.join(MANIFEST_FILENAME);
        let manifest_bytes = match fs::read(&manifest_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(CvSyncError::Local(format!(
                    "could not read {}: {error}",
                    manifest_path.display()
                )));
            }
        };
        let manifest: CvManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| CvSyncError::Local(format!("manifest is not valid JSON: {error}")))?;
        manifest.validate_metadata().map_err(CvSyncError::Local)?;

        let tex = self.read_asset(TEX_REPOSITORY_PATH)?;
        let pdf = self.read_asset(PDF_REPOSITORY_PATH)?;
        let generated = self.read_asset(GENERATED_CV_PATH)?;
        validate_tex(&tex).map_err(CvSyncError::Local)?;
        validate_pdf(&pdf).map_err(CvSyncError::Local)?;
        if !manifest.source.matches(&tex) {
            return Err(CvSyncError::Local(format!(
                "{TEX_REPOSITORY_PATH} does not match its manifest digest"
            )));
        }
        if !manifest.pdf.matches(&pdf) {
            return Err(CvSyncError::Local(format!(
                "{PDF_REPOSITORY_PATH} does not match its manifest digest"
            )));
        }
        if !manifest.generated.matches(&generated) {
            return Err(CvSyncError::Local(format!(
                "{GENERATED_CV_PATH} does not match its manifest digest"
            )));
        }
        let source = std::str::from_utf8(&tex)
            .map_err(|_| CvSyncError::Local("TeX source is not UTF-8".to_owned()))?;
        let cv = parse_cv(source)
            .map_err(|error| CvSyncError::Local(format!("LaTeX parse failed at {error}")))?;
        let expected_generated = generate_cv_module(
            &cv,
            &RemoteTag {
                name: manifest.tag.clone(),
                commit_sha: manifest.commit_sha.clone(),
            },
        );
        if generated != expected_generated {
            return Err(CvSyncError::Local(format!(
                "{GENERATED_CV_PATH} was not generated from the manifested TeX and source identity"
            )));
        }
        Ok(Some(manifest))
    }

    /// Commits a complete validated bundle with rollback on any replacement
    /// failure. The manifest is installed last as the transaction marker.
    pub fn commit(&self, bundle: &ValidatedBundle) -> Result<(), CvSyncError> {
        bundle
            .manifest
            .validate_metadata()
            .map_err(CvSyncError::Validation)?;
        validate_tex(&bundle.tex).map_err(CvSyncError::Validation)?;
        validate_pdf(&bundle.pdf).map_err(CvSyncError::Validation)?;
        if !bundle.manifest.source.matches(&bundle.tex)
            || !bundle.manifest.pdf.matches(&bundle.pdf)
            || !bundle.manifest.generated.matches(&bundle.generated)
        {
            return Err(CvSyncError::Validation(
                "bundle bytes do not match the proposed manifest".to_owned(),
            ));
        }
        let source = std::str::from_utf8(&bundle.tex)
            .map_err(|_| CvSyncError::Validation("TeX source is not UTF-8".to_owned()))?;
        let cv = parse_cv(source)
            .map_err(|error| CvSyncError::Validation(format!("LaTeX parse failed at {error}")))?;
        let expected_generated = generate_cv_module(
            &cv,
            &RemoteTag {
                name: bundle.manifest.tag.clone(),
                commit_sha: bundle.manifest.commit_sha.clone(),
            },
        );
        if bundle.generated != expected_generated {
            return Err(CvSyncError::Validation(
                "generated CV module does not match the proposed TeX and source identity"
                    .to_owned(),
            ));
        }

        fs::create_dir_all(&self.directory).map_err(|error| {
            CvSyncError::Local(format!(
                "could not create {}: {error}",
                self.directory.display()
            ))
        })?;
        if let Some(parent) = self.repository_root.join(GENERATED_CV_PATH).parent() {
            fs::create_dir_all(parent).map_err(|error| {
                CvSyncError::Local(format!("could not create {}: {error}", parent.display()))
            })?;
        }
        let _lock = LockGuard::acquire(self.directory.join(LOCK_FILENAME))?;
        let stage = tempfile::Builder::new()
            .prefix(".cv-sync-stage-")
            .tempdir_in(&self.directory)
            .map_err(|error| CvSyncError::Local(format!("could not stage CV bundle: {error}")))?;

        let manifest_bytes = serialize_manifest(&bundle.manifest)?;
        let entries = [
            Entry::new(TEX_REPOSITORY_PATH, "new-source.tex", &bundle.tex),
            Entry::new(PDF_REPOSITORY_PATH, "new-cv.pdf", &bundle.pdf),
            Entry::new(GENERATED_CV_PATH, "new-generated.rs", &bundle.generated),
            Entry::new(
                MANIFEST_REPOSITORY_PATH,
                "new-manifest.json",
                &manifest_bytes,
            ),
        ];
        for entry in &entries {
            write_synced(&stage.path().join(entry.staged_name()), entry.bytes)?;
        }

        self.replace_entries(stage, &entries)
    }

    fn read_asset(&self, repository_path: &str) -> Result<Vec<u8>, CvSyncError> {
        let path = self.repository_root.join(repository_path);
        fs::read(&path).map_err(|error| {
            CvSyncError::Local(format!("could not read {}: {error}", path.display()))
        })
    }

    fn replace_entries(&self, stage: TempDir, entries: &[Entry<'_>]) -> Result<(), CvSyncError> {
        let mut backed_up = Vec::new();
        let mut installed = Vec::new();

        for entry in entries {
            let target = self.repository_root.join(entry.repository_path);
            if target.exists() {
                let backup = stage.path().join(entry.backup_name());
                if let Err(error) = fs::rename(&target, &backup) {
                    return self.rollback_or_preserve(
                        stage,
                        &installed,
                        &backed_up,
                        format!("could not back up {}: {error}", target.display()),
                    );
                }
                backed_up.push((target, backup));
            }
        }

        for entry in entries {
            let staged = stage.path().join(entry.staged_name());
            let target = self.repository_root.join(entry.repository_path);
            if let Err(error) = fs::rename(&staged, &target) {
                return self.rollback_or_preserve(
                    stage,
                    &installed,
                    &backed_up,
                    format!("could not install {}: {error}", target.display()),
                );
            }
            installed.push(target);
        }

        if let Err(error) = sync_directory(&self.directory) {
            return self.rollback_or_preserve(
                stage,
                &installed,
                &backed_up,
                format!("could not finalize CV transaction: {error}"),
            );
        }
        let generated_parent = self
            .repository_root
            .join(GENERATED_CV_PATH)
            .parent()
            .expect("the generated path has a parent")
            .to_path_buf();
        if let Err(error) = sync_directory(&generated_parent) {
            return self.rollback_or_preserve(
                stage,
                &installed,
                &backed_up,
                format!("could not finalize generated CV transaction: {error}"),
            );
        }
        Ok(())
    }

    fn rollback_or_preserve(
        &self,
        stage: TempDir,
        installed: &[PathBuf],
        backed_up: &[(PathBuf, PathBuf)],
        original_error: String,
    ) -> Result<(), CvSyncError> {
        match rollback(installed, backed_up) {
            Ok(()) => Err(CvSyncError::Local(format!(
                "{original_error}; previous bundle restored"
            ))),
            Err(rollback_error) => {
                let recovery_directory = stage.keep();
                Err(CvSyncError::Local(format!(
                    "{original_error}; rollback also failed: {rollback_error}; backups preserved at {}",
                    recovery_directory.display()
                )))
            }
        }
    }
}

struct Entry<'a> {
    repository_path: &'static str,
    staged_name: &'static str,
    bytes: &'a [u8],
}

impl<'a> Entry<'a> {
    fn new(repository_path: &'static str, staged_name: &'static str, bytes: &'a [u8]) -> Self {
        Self {
            repository_path,
            staged_name,
            bytes,
        }
    }

    fn staged_name(&self) -> &'static str {
        self.staged_name
    }

    fn backup_name(&self) -> String {
        format!("old-{}", self.staged_name)
    }
}

struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    fn acquire(path: PathBuf) -> Result<Self, CvSyncError> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                CvSyncError::Local(format!(
                    "could not acquire CV synchronization lock {}: {error}",
                    path.display()
                ))
            })?;
        Ok(Self { path })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn serialize_manifest(manifest: &CvManifest) -> Result<Vec<u8>, CvSyncError> {
    let mut bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| CvSyncError::Local(format!("could not serialize manifest: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), CvSyncError> {
    let mut file = File::create(path).map_err(|error| {
        CvSyncError::Local(format!("could not create {}: {error}", path.display()))
    })?;
    file.write_all(bytes).map_err(|error| {
        CvSyncError::Local(format!("could not write {}: {error}", path.display()))
    })?;
    file.sync_all()
        .map_err(|error| CvSyncError::Local(format!("could not sync {}: {error}", path.display())))
}

fn rollback(installed: &[PathBuf], backed_up: &[(PathBuf, PathBuf)]) -> Result<(), std::io::Error> {
    for path in installed.iter().rev() {
        fs::remove_file(path)?;
    }
    for (target, backup) in backed_up.iter().rev() {
        fs::rename(backup, target)?;
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> Result<(), CvSyncError> {
    File::open(directory)
        .and_then(|file| file.sync_all())
        .map_err(|error| {
            CvSyncError::Local(format!(
                "could not sync directory {}: {error}",
                directory.display()
            ))
        })
}

#[cfg(not(unix))]
fn sync_directory(_directory: &Path) -> Result<(), CvSyncError> {
    Ok(())
}
