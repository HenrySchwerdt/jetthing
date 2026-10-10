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
                    Some(".git" | "target" | "node_modules")
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
}
