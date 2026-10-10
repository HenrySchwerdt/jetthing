use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("failed to execute Git: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git command failed: {0}")]
    Command(String),
    #[error("path is not inside a Git repository: {0}")]
    NotRepository(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub staged: bool,
    pub unstaged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub root: PathBuf,
    pub branch: String,
    pub changes: Vec<FileChange>,
}

impl RepositoryStatus {
    pub fn load(path: &Path) -> Result<Self, GitError> {
        let root_output = run(path, &["rev-parse", "--show-toplevel"])?;
        if !root_output.status.success() {
            return Err(GitError::NotRepository(path.to_owned()));
        }
        let root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
        let branch = current_branch(&root)?;
        let output = checked(
            &root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?;
        let changes = parse_porcelain(&output.stdout);
        Ok(Self {
            root,
            branch,
            changes,
        })
    }

    pub fn stage(&self, path: &Path) -> Result<(), GitError> {
        checked_with_path(&self.root, &["add", "--"], path).map(|_| ())
    }

    pub fn unstage(&self, path: &Path) -> Result<(), GitError> {
        checked_with_path(&self.root, &["restore", "--staged", "--"], path).map(|_| ())
    }

    /// Commits exactly `paths` (repository-relative), including new and deleted files,
    /// leaving other staged changes untouched. Returns the new short commit hash.
    pub fn commit(&self, message: &str, paths: &[PathBuf]) -> Result<String, GitError> {
        if message.trim().is_empty() {
            return Err(GitError::Command("commit message is empty".into()));
        }
        if paths.is_empty() {
            return Err(GitError::Command("no changes selected for commit".into()));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["add", "-A", "--"])
            .args(paths)
            .output()?;
        if !output.status.success() {
            return Err(GitError::Command(
                String::from_utf8_lossy(&output.stderr).trim().into(),
            ));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["commit", "--quiet", "-m", message, "--only", "--"])
            .args(paths)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(GitError::Command(
                format!("{}{}", stderr.trim(), stdout.trim()).trim().into(),
            ));
        }
        let head = checked(&self.root, &["rev-parse", "--short", "HEAD"])?;
        Ok(String::from_utf8_lossy(&head.stdout).trim().into())
    }
}

fn current_branch(root: &Path) -> Result<String, GitError> {
    let symbolic = run(root, &["symbolic-ref", "--short", "-q", "HEAD"])?;
    if symbolic.status.success() {
        return Ok(String::from_utf8_lossy(&symbolic.stdout).trim().into());
    }
    let detached = checked(root, &["rev-parse", "--short", "HEAD"])?;
    Ok(format!(
        "detached@{}",
        String::from_utf8_lossy(&detached.stdout).trim()
    ))
}

fn run(root: &Path, args: &[&str]) -> Result<Output, GitError> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(GitError::from)
}

fn checked(root: &Path, args: &[&str]) -> Result<Output, GitError> {
    let output = run(root, args)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(GitError::Command(
            String::from_utf8_lossy(&output.stderr).trim().into(),
        ))
    }
}

fn checked_with_path(root: &Path, args: &[&str], path: &Path) -> Result<Output, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .arg(path)
        .output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(GitError::Command(
            String::from_utf8_lossy(&output.stderr).trim().into(),
        ))
    }
}

fn parse_porcelain(bytes: &[u8]) -> Vec<FileChange> {
    let records: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect();
    let mut changes = Vec::new();
    let mut index = 0;
    while index < records.len() {
        let record = records[index];
        if record.len() < 4 {
            index += 1;
            continue;
        }
        let x = record[0] as char;
        let y = record[1] as char;
        let path = PathBuf::from(String::from_utf8_lossy(&record[3..]).into_owned());
        let kind = if x == '?' && y == '?' {
            ChangeKind::Untracked
        } else if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
            ChangeKind::Conflicted
        } else if x == 'R' || y == 'R' {
            ChangeKind::Renamed
        } else if x == 'D' || y == 'D' {
            ChangeKind::Deleted
        } else if x == 'A' {
            ChangeKind::Added
        } else {
            ChangeKind::Modified
        };
        changes.push(FileChange {
            path,
            kind,
            staged: x != ' ' && x != '?',
            unstaged: y != ' ' || (x == '?' && y == '?'),
        });
        index += if matches!(x, 'R' | 'C') { 2 } else { 1 };
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(root: &Path, args: &[&str]) {
        assert!(run(root, args).unwrap().status.success());
    }

    #[test]
    fn reads_real_repository_status_and_stages_safely() {
        let root = std::env::temp_dir().join(format!("jetthing-git-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.invalid"]);
        git(&root, &["config", "user.name", "Jetthing Test"]);
        fs::write(root.join("tracked.txt"), "first\n").unwrap();
        git(&root, &["add", "tracked.txt"]);
        git(&root, &["commit", "-qm", "initial"]);
        fs::write(root.join("tracked.txt"), "changed\n").unwrap();
        fs::write(root.join("space name.txt"), "new\n").unwrap();

        let status = RepositoryStatus::load(&root).unwrap();
        assert!(
            status
                .changes
                .iter()
                .any(|change| change.path == Path::new("tracked.txt") && change.unstaged)
        );
        let untracked = status
            .changes
            .iter()
            .find(|change| change.path == Path::new("space name.txt"))
            .unwrap();
        assert_eq!(untracked.kind, ChangeKind::Untracked);
        status.stage(&untracked.path).unwrap();
        let staged = RepositoryStatus::load(&root).unwrap();
        assert!(
            staged
                .changes
                .iter()
                .any(|change| change.path == Path::new("space name.txt") && change.staged)
        );
        staged.unstage(Path::new("space name.txt")).unwrap();
        assert!(
            RepositoryStatus::load(&root)
                .unwrap()
                .changes
                .iter()
                .any(|change| change.path == Path::new("space name.txt") && change.unstaged)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_non_repository() {
        let root = std::env::temp_dir();
        let result = RepositoryStatus::load(&root);
        assert!(matches!(result, Err(GitError::NotRepository(_))));
    }

    #[test]
    fn commits_only_selected_paths_including_new_files() {
        let root = std::env::temp_dir().join(format!("jetthing-git-commit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.invalid"]);
        git(&root, &["config", "user.name", "Jetthing Test"]);
        fs::write(root.join("a.txt"), "a\n").unwrap();
        git(&root, &["add", "a.txt"]);
        git(&root, &["commit", "-qm", "initial"]);
        fs::write(root.join("a.txt"), "changed\n").unwrap();
        fs::write(root.join("new file.txt"), "new\n").unwrap();
        fs::write(root.join("other.txt"), "keep\n").unwrap();
        let status = RepositoryStatus::load(&root).unwrap();
        assert!(status.commit("  ", &[PathBuf::from("a.txt")]).is_err());
        assert!(status.commit("msg", &[]).is_err());
        let hash = status
            .commit(
                "Update a\n\nwith body",
                &[PathBuf::from("a.txt"), PathBuf::from("new file.txt")],
            )
            .unwrap();
        assert!(!hash.is_empty());
        let after = RepositoryStatus::load(&root).unwrap();
        let remaining: Vec<_> = after.changes.iter().map(|c| c.path.clone()).collect();
        assert_eq!(remaining, vec![PathBuf::from("other.txt")]);
        fs::remove_dir_all(root).unwrap();
    }
}
