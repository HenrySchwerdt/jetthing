use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("path is outside the project: {0}")]
    OutsideRoot(PathBuf),
    #[error("file is not valid UTF-8: {0}")]
    NonUtf8(PathBuf),
    #[error("already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error("invalid name `{0}`: use a relative path without `..`")]
    InvalidName(String),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub relative_path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
}

impl Project {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let root = path.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(ProjectError::NotDirectory(root));
        }
        Ok(Self { root })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn entries(&self, max: usize) -> Vec<FileEntry> {
        WalkDir::new(&self.root)
            .follow_links(false)
            .sort_by(|a, b| {
                // Project view order: directories first, then case-insensitive names.
                b.file_type()
                    .is_dir()
                    .cmp(&a.file_type().is_dir())
                    .then_with(|| {
                        let a = a.file_name().to_string_lossy().to_lowercase();
                        let b = b.file_name().to_string_lossy().to_lowercase();
                        a.cmp(&b)
                    })
            })
            .into_iter()
            .filter_entry(|e| {
                !matches!(
                    e.file_name().to_str(),
                    Some(".git" | ".jetthing" | "target" | "node_modules")
                )
            })
            .filter_map(Result::ok)
            .skip(1)
            .take(max)
            .filter_map(|e| {
                Some(FileEntry {
                    relative_path: e.path().strip_prefix(&self.root).ok()?.to_owned(),
                    is_dir: e.file_type().is_dir(),
                })
            })
            .collect()
    }
    fn checked(&self, relative: &Path) -> Result<PathBuf, ProjectError> {
        let candidate = self.root.join(relative);
        let canonical = candidate.canonicalize()?;
        if !canonical.starts_with(&self.root) {
            return Err(ProjectError::OutsideRoot(candidate));
        }
        Ok(canonical)
    }
    pub fn read_text(&self, relative: &Path) -> Result<String, ProjectError> {
        let path = self.checked(relative)?;
        String::from_utf8(fs::read(&path)?).map_err(|_| ProjectError::NonUtf8(path))
    }
    /// Validates a project-relative path that may not exist yet.
    fn checked_new(&self, relative: &Path) -> Result<PathBuf, ProjectError> {
        use std::path::Component;
        let invalid = || ProjectError::InvalidName(relative.display().to_string());
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(invalid());
        }
        let candidate = self.root.join(relative);
        // The nearest existing ancestor must resolve inside the root (no symlink escape).
        let mut ancestor = candidate.parent();
        while let Some(dir) = ancestor {
            if dir.exists() {
                if !dir.canonicalize()?.starts_with(&self.root) {
                    return Err(ProjectError::OutsideRoot(candidate));
                }
                break;
            }
            ancestor = dir.parent();
        }
        if candidate.symlink_metadata().is_ok() {
            return Err(ProjectError::AlreadyExists(relative.to_owned()));
        }
        Ok(candidate)
    }

    /// Creates an empty file, including missing parent directories.
    pub fn create_file(&self, relative: &Path) -> Result<(), ProjectError> {
        let path = self.checked_new(relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        Ok(())
    }

    pub fn create_dir(&self, relative: &Path) -> Result<(), ProjectError> {
        let path = self.checked_new(relative)?;
        fs::create_dir_all(path)?;
        Ok(())
    }

    /// Renames a file or directory in place; `new_name` is a single path segment.
    /// Returns the new project-relative path.
    pub fn rename(&self, relative: &Path, new_name: &str) -> Result<PathBuf, ProjectError> {
        if new_name.is_empty()
            || new_name.contains(['/', '\\'])
            || new_name == "."
            || new_name == ".."
        {
            return Err(ProjectError::InvalidName(new_name.to_owned()));
        }
        let from = self.checked(relative)?;
        if from == self.root {
            return Err(ProjectError::InvalidName(relative.display().to_string()));
        }
        let target = relative
            .parent()
            .map(|parent| parent.join(new_name))
            .unwrap_or_else(|| PathBuf::from(new_name));
        let to = self.checked_new(&target)?;
        fs::rename(from, to)?;
        Ok(target)
    }

    /// Deletes a file or a directory tree. The project root itself cannot be deleted.
    pub fn delete(&self, relative: &Path) -> Result<(), ProjectError> {
        let path = self.checked(relative)?;
        if path == self.root {
            return Err(ProjectError::InvalidName(relative.display().to_string()));
        }
        // Delete the link itself, never its target.
        let link = self.root.join(relative);
        if link.symlink_metadata()?.file_type().is_symlink() {
            fs::remove_file(link)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn write_text(&self, relative: &Path, text: &str) -> Result<(), ProjectError> {
        let path = self.checked(relative)?;
        fs::write(path, text).map_err(ProjectError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opens_lists_and_reads_project() {
        let root = std::env::temp_dir().join(format!("jetthing-project-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        let project = Project::open(&root).unwrap();
        assert!(
            project
                .entries(10)
                .iter()
                .any(|e| e.relative_path == Path::new("src/main.rs"))
        );
        assert_eq!(
            project.read_text(Path::new("src/main.rs")).unwrap(),
            "fn main() {}\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn creates_renames_and_deletes_inside_the_root() {
        let root =
            std::env::temp_dir().join(format!("jetthing-project-ops-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let project = Project::open(&root).unwrap();
        project.create_file(Path::new("src/new.rs")).unwrap();
        assert!(root.join("src/new.rs").is_file());
        assert!(matches!(
            project.create_file(Path::new("src/new.rs")),
            Err(ProjectError::AlreadyExists(_))
        ));
        for bad in ["../escape.rs", "/abs.rs", "", "a/../../b"] {
            assert!(
                matches!(
                    project.create_file(Path::new(bad)),
                    Err(ProjectError::InvalidName(_))
                ),
                "{bad}"
            );
        }
        let renamed = project.rename(Path::new("src/new.rs"), "lib.rs").unwrap();
        assert_eq!(renamed, Path::new("src/lib.rs"));
        assert!(project.rename(&renamed, "x/y").is_err());
        project.create_dir(Path::new("empty")).unwrap();
        project.delete(Path::new("src")).unwrap();
        project.delete(Path::new("empty")).unwrap();
        assert!(!root.join("src").exists());
        assert!(project.delete(Path::new("")).is_err());
        assert!(project.delete(Path::new(".")).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
