use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::{GENERATED_PROJECTS_PATH, PROJECT_IMAGES_PATH, ProjectSyncError};

const LOCK_FILENAME: &str = ".project-sync.lock";
const PROJECT_IMAGES_BACKUP_FILENAME: &str = ".project-images.backup";
const GENERATED_PROJECTS_BACKUP_FILENAME: &str = ".generated-projects.backup";

/// Atomic filesystem boundary for generated project data.
#[derive(Clone, Debug)]
pub struct ProjectDataStore {
    repository_root: PathBuf,
}

impl ProjectDataStore {
    #[must_use]
    pub fn new(repository_root: impl AsRef<Path>) -> Self {
        Self {
            repository_root: repository_root.as_ref().to_path_buf(),
        }
    }

    /// Atomically installs generated data and its selected project thumbnails.
    pub fn commit_if_changed(
        &self,
        generated: &[u8],
        thumbnails: &[(String, Vec<u8>)],
    ) -> Result<bool, ProjectSyncError> {
        if generated.is_empty() || std::str::from_utf8(generated).is_err() {
            return Err(ProjectSyncError::Validation(
                "generated project module must be non-empty UTF-8".to_owned(),
            ));
        }
        let target = self.repository_root.join(GENERATED_PROJECTS_PATH);
        let parent = target
            .parent()
            .expect("generated project path has a parent");
        fs::create_dir_all(parent).map_err(|error| {
            ProjectSyncError::Local(format!("could not create {}: {error}", parent.display()))
        })?;
        let lock = self.repository_root.join(LOCK_FILENAME);
        let _guard = LockGuard::acquire(lock)?;
        let thumbnails = thumbnail_map(thumbnails)?;
        let generated_changed = file_differs(&target, generated)?;
        let images_directory = self.repository_root.join(PROJECT_IMAGES_PATH);
        let images_changed = thumbnails_differ(&images_directory, &thumbnails)?;
        if !generated_changed && !images_changed {
            return Ok(false);
        }

        let images_parent = images_directory
            .parent()
            .expect("project images path has a parent");
        fs::create_dir_all(images_parent).map_err(|error| {
            ProjectSyncError::Local(format!(
                "could not create {}: {error}",
                images_parent.display()
            ))
        })?;
        let staged_images_root = tempfile::tempdir_in(images_parent).map_err(|error| {
            ProjectSyncError::Local(format!("could not stage project thumbnails: {error}"))
        })?;
        let staged_images = staged_images_root.path().join("projects");
        fs::create_dir(&staged_images).map_err(|error| {
            ProjectSyncError::Local(format!(
                "could not prepare staged thumbnails {}: {error}",
                staged_images.display()
            ))
        })?;
        for (name, bytes) in &thumbnails {
            let path = staged_images.join(name);
            fs::write(&path, bytes).map_err(|error| {
                ProjectSyncError::Local(format!(
                    "could not write staged thumbnail {}: {error}",
                    path.display()
                ))
            })?;
            OpenOptions::new()
                .write(true)
                .open(&path)
                .and_then(|file| file.sync_all())
                .map_err(|error| {
                    ProjectSyncError::Local(format!(
                        "could not sync staged thumbnail {}: {error}",
                        path.display()
                    ))
                })?;
        }

        let staged = if generated_changed {
            let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|error| {
                ProjectSyncError::Local(format!("could not stage project data: {error}"))
            })?;
            staged.write_all(generated).map_err(|error| {
                ProjectSyncError::Local(format!("could not write staged project data: {error}"))
            })?;
            staged.as_file().sync_all().map_err(|error| {
                ProjectSyncError::Local(format!("could not sync staged project data: {error}"))
            })?;
            Some(staged)
        } else {
            None
        };
        let backup = parent.join(GENERATED_PROJECTS_BACKUP_FILENAME);
        if generated_changed && backup.exists() {
            return Err(ProjectSyncError::Local(format!(
                "could not replace project data because backup {} already exists",
                backup.display()
            )));
        }

        let images_backup = images_parent.join(PROJECT_IMAGES_BACKUP_FILENAME);
        if images_changed && images_backup.exists() {
            return Err(ProjectSyncError::Local(format!(
                "could not replace project thumbnails because backup {} already exists",
                images_backup.display()
            )));
        }
        let had_images = images_directory.exists();
        if images_changed && had_images {
            fs::rename(&images_directory, &images_backup).map_err(|error| {
                ProjectSyncError::Local(format!(
                    "could not back up {}: {error}",
                    images_directory.display()
                ))
            })?;
        }
        if images_changed && let Err(error) = fs::rename(&staged_images, &images_directory) {
            if had_images {
                let _ = fs::rename(&images_backup, &images_directory);
            }
            return Err(ProjectSyncError::Local(format!(
                "could not install project thumbnails {}: {error}",
                images_directory.display()
            )));
        }

        if !generated_changed {
            if had_images {
                fs::remove_dir_all(&images_backup).map_err(|error| {
                    ProjectSyncError::Local(format!(
                        "installed project thumbnails but could not remove backup {}: {error}",
                        images_backup.display()
                    ))
                })?;
            }
            return Ok(true);
        }

        let had_target = target.exists();
        if had_target && let Err(error) = fs::rename(&target, &backup) {
            if images_changed {
                restore_project_thumbnails(&images_directory, &images_backup, had_images);
            }
            return Err(ProjectSyncError::Local(format!(
                "could not back up {}: {error}",
                target.display()
            )));
        }
        let staged = staged.expect("generated project data is staged when it changed");
        if let Err(error) = staged.persist(&target) {
            if had_target {
                let _ = fs::rename(&backup, &target);
            }
            if images_changed {
                restore_project_thumbnails(&images_directory, &images_backup, had_images);
            }
            return Err(ProjectSyncError::Local(format!(
                "could not install {}: {}",
                target.display(),
                error.error
            )));
        }
        if had_target {
            fs::remove_file(&backup).map_err(|error| {
                ProjectSyncError::Local(format!(
                    "installed project data but could not remove backup {}: {error}",
                    backup.display()
                ))
            })?;
        }
        if images_changed && had_images {
            fs::remove_dir_all(&images_backup).map_err(|error| {
                ProjectSyncError::Local(format!(
                    "installed project data but could not remove thumbnail backup {}: {error}",
                    images_backup.display()
                ))
            })?;
        }
        Ok(true)
    }
}

fn thumbnail_map(
    thumbnails: &[(String, Vec<u8>)],
) -> Result<BTreeMap<&str, &[u8]>, ProjectSyncError> {
    let mut output = BTreeMap::new();
    for (name, bytes) in thumbnails {
        if !name.ends_with(".png")
            || name.len() <= ".png".len()
            || name.contains(['/', '\\'])
            || bytes.is_empty()
        {
            return Err(ProjectSyncError::Validation(format!(
                "invalid synchronized thumbnail name {name:?}"
            )));
        }
        if output.insert(name.as_str(), bytes.as_slice()).is_some() {
            return Err(ProjectSyncError::Validation(format!(
                "duplicate synchronized thumbnail name {name:?}"
            )));
        }
    }
    Ok(output)
}

fn file_differs(path: &Path, expected: &[u8]) -> Result<bool, ProjectSyncError> {
    match fs::read(path) {
        Ok(current) => Ok(current != expected),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(ProjectSyncError::Local(format!(
            "could not read {}: {error}",
            path.display()
        ))),
    }
}

fn thumbnails_differ(
    directory: &Path,
    expected: &BTreeMap<&str, &[u8]>,
) -> Result<bool, ProjectSyncError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(!expected.is_empty());
        }
        Err(error) => {
            return Err(ProjectSyncError::Local(format!(
                "could not read {}: {error}",
                directory.display()
            )));
        }
    };
    let mut found = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            ProjectSyncError::Local(format!(
                "could not read a thumbnail entry in {}: {error}",
                directory.display()
            ))
        })?;
        if !entry
            .file_type()
            .map_err(|error| {
                ProjectSyncError::Local(format!(
                    "could not inspect thumbnail entry {}: {error}",
                    entry.path().display()
                ))
            })?
            .is_file()
        {
            return Err(ProjectSyncError::Local(format!(
                "project thumbnail directory contains a non-file entry: {}",
                entry.path().display()
            )));
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            ProjectSyncError::Local(format!(
                "project thumbnail filename is not UTF-8: {}",
                entry.path().display()
            ))
        })?;
        found.insert(
            name.to_owned(),
            fs::read(entry.path()).map_err(|error| {
                ProjectSyncError::Local(format!(
                    "could not read thumbnail {}: {error}",
                    entry.path().display()
                ))
            })?,
        );
    }
    Ok(found.len() != expected.len()
        || expected
            .iter()
            .any(|(name, bytes)| found.get(*name).is_none_or(|current| current != *bytes)))
}

fn restore_project_thumbnails(images: &Path, backup: &Path, had_images: bool) {
    if images.exists() {
        let _ = fs::remove_dir_all(images);
    }
    if had_images {
        let _ = fs::rename(backup, images);
    }
}

struct LockGuard {
    path: PathBuf,
    _file: File,
}

impl LockGuard {
    fn acquire(path: PathBuf) -> Result<Self, ProjectSyncError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                ProjectSyncError::Local(format!(
                    "could not acquire project synchronization lock {}: {error}",
                    path.display()
                ))
            })?;
        Ok(Self { path, _file: file })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
