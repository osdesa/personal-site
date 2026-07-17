use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use super::{
    CvManifest, CvSyncError, MANIFEST_FILENAME, PDF_FILENAME, TEX_FILENAME, ValidatedBundle,
    validate_pdf, validate_tex,
};

const LOCK_FILENAME: &str = ".cv-sync.lock";

/// Filesystem boundary for the checked-in CV source bundle.
#[derive(Clone, Debug)]
pub struct CvBundleStore {
    directory: PathBuf,
}

impl CvBundleStore {
    /// Creates a store rooted at `<repository_root>/public/cv`.
    #[must_use]
    pub fn new(repository_root: impl AsRef<Path>) -> Self {
        Self {
            directory: repository_root.as_ref().join("public").join("cv"),
        }
    }

    /// Returns the directory containing the three committed bundle files.
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

        let tex = self.read_asset(TEX_FILENAME)?;
        let pdf = self.read_asset(PDF_FILENAME)?;
        validate_tex(&tex).map_err(CvSyncError::Local)?;
        validate_pdf(&pdf).map_err(CvSyncError::Local)?;
        if !manifest.source.matches(&tex) {
            return Err(CvSyncError::Local(format!(
                "{TEX_FILENAME} does not match its manifest digest"
            )));
        }
        if !manifest.pdf.matches(&pdf) {
            return Err(CvSyncError::Local(format!(
                "{PDF_FILENAME} does not match its manifest digest"
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
        if !bundle.manifest.source.matches(&bundle.tex) || !bundle.manifest.pdf.matches(&bundle.pdf)
        {
            return Err(CvSyncError::Validation(
                "bundle bytes do not match the proposed manifest".to_owned(),
            ));
        }

        fs::create_dir_all(&self.directory).map_err(|error| {
            CvSyncError::Local(format!(
                "could not create {}: {error}",
                self.directory.display()
            ))
        })?;
        let _lock = LockGuard::acquire(self.directory.join(LOCK_FILENAME))?;
        let stage = tempfile::Builder::new()
            .prefix(".cv-sync-stage-")
            .tempdir_in(&self.directory)
            .map_err(|error| CvSyncError::Local(format!("could not stage CV bundle: {error}")))?;

        let manifest_bytes = serialize_manifest(&bundle.manifest)?;
        let entries = [
            Entry::new(TEX_FILENAME, &bundle.tex),
            Entry::new(PDF_FILENAME, &bundle.pdf),
            Entry::new(MANIFEST_FILENAME, &manifest_bytes),
        ];
        for entry in &entries {
            write_synced(&stage.path().join(entry.staged_name()), entry.bytes)?;
        }

        self.replace_entries(stage, &entries)
    }

    fn read_asset(&self, filename: &str) -> Result<Vec<u8>, CvSyncError> {
        let path = self.directory.join(filename);
        fs::read(&path).map_err(|error| {
            CvSyncError::Local(format!("could not read {}: {error}", path.display()))
        })
    }

    fn replace_entries(&self, stage: TempDir, entries: &[Entry<'_>]) -> Result<(), CvSyncError> {
        let mut backed_up = Vec::new();
        let mut installed = Vec::new();

        for entry in entries {
            let target = self.directory.join(entry.filename);
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
            let target = self.directory.join(entry.filename);
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
    filename: &'static str,
    bytes: &'a [u8],
}

impl<'a> Entry<'a> {
    fn new(filename: &'static str, bytes: &'a [u8]) -> Self {
        Self { filename, bytes }
    }

    fn staged_name(&self) -> String {
        format!("new-{}", self.filename)
    }

    fn backup_name(&self) -> String {
        format!("old-{}", self.filename)
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
