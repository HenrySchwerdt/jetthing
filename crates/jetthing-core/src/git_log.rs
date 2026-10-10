//! State and commands behind the Git tool window's Log tab: history with a
//! commit graph, branch/text filters, and the selected commit's details.

use crate::{CommandError, IdeState, ToolWindow, spec, string_param};
use jetthing_command::{Command, CommandRegistry};
use jetthing_git::{
    Branches, CommitDetails, CommitInfo, GraphRow, LogQuery, branches, commit_details, graph, log,
};
use serde_json::{Value, json};
use std::path::PathBuf;

/// Commits loaded into the log at once.
const LOG_LIMIT: usize = 2_000;

#[derive(Debug, Clone, Default)]
pub struct GitLogState {
    pub commits: Vec<CommitInfo>,
    /// Graph row per entry of `commits`.
    pub graph: Vec<GraphRow>,
    pub branches: Branches,
    /// Branch filter; `None` shows all branches.
    pub branch: Option<String>,
    /// Only commits touching this repository-relative path (file history).
    pub path: Option<PathBuf>,
    /// Text or hash filter.
    pub filter: String,
    pub selected: Option<String>,
    pub details: Option<CommitDetails>,
    pub error: Option<String>,
    /// Whether the log reflects the current repository state.
    pub loaded: bool,
}

impl GitLogState {
    /// Indices into `commits` that pass the text filter.
    pub fn visible(&self) -> Vec<usize> {
        self.commits
            .iter()
            .enumerate()
            .filter(|(_, commit)| commit.matches(&self.filter))
            .map(|(index, _)| index)
            .collect()
    }
}

impl IdeState {
    /// Reloads history and branches, keeping the selection when possible.
    pub fn refresh_git_log(&mut self) -> Result<(), String> {
        let root = self
            .git
            .as_ref()
            .ok_or("the project is not a Git repository")?
            .root
            .clone();
        let state = &mut self.git_log;
        state.loaded = true;
        let result = branches(&root).and_then(|branches| {
            let commits = log(
                &root,
                &LogQuery {
                    revision: state.branch.clone(),
                    path: state.path.clone(),
                    limit: LOG_LIMIT,
                },
            )?;
            Ok((branches, commits))
        });
        match result {
            Ok((branches, commits)) => {
                state.graph = graph(&commits);
                state.commits = commits;
                state.branches = branches;
                state.error = None;
            }
            Err(error) => {
                state.error = Some(error.to_string());
                return Err(error.to_string());
            }
        }
        let keep = state
            .selected
            .clone()
            .filter(|hash| state.commits.iter().any(|commit| &commit.hash == hash));
        let selected = keep.or_else(|| state.commits.first().map(|commit| commit.hash.clone()));
        match selected {
            Some(hash) => self.select_commit(&hash),
            None => {
                self.git_log.selected = None;
                self.git_log.details = None;
                Ok(())
            }
        }
    }

    /// Marks the log stale; it reloads the next time it is shown.
    pub(crate) fn invalidate_git_log(&mut self) {
        self.git_log.loaded = false;
    }

    pub fn select_commit(&mut self, hash: &str) -> Result<(), String> {
        let root = self
            .git
            .as_ref()
            .ok_or("the project is not a Git repository")?
            .root
            .clone();
        if !self
            .git_log
            .commits
            .iter()
            .any(|commit| commit.hash == hash)
        {
            return Err(format!("commit {hash} is not in the loaded log"));
        }
        let details = commit_details(&root, hash).map_err(|error| error.to_string())?;
        self.git_log.selected = Some(hash.to_owned());
        self.git_log.details = Some(details);
        Ok(())
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    let has_git = |state: &IdeState| state.git.is_some();
    registry
        .register(Command::new(
            spec("git.log.refresh", "Refresh Log", "Git", None, "{}"),
            has_git,
            |state: &mut IdeState, _| {
                state.refresh_git_log().map_err(CommandError::Failed)?;
                Ok(json!({"commits": state.git_log.commits.len()}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("git.log.show", "Show Git Log", "Git", None, "{}"),
            has_git,
            |state: &mut IdeState, _| {
                state.tool_windows.show(ToolWindow::Git);
                if !state.git_log.loaded {
                    state.refresh_git_log().map_err(CommandError::Failed)?;
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.log.selectCommit",
                "Select Commit",
                "Git",
                None,
                r#"{"hash":"string"}"#,
            ),
            has_git,
            |state: &mut IdeState, params| {
                state
                    .select_commit(&string_param(&params, "hash")?)
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.log.filterBranch",
                "Filter Log by Branch",
                "Git",
                None,
                r#"{"branch":"string?"}"#,
            ),
            has_git,
            |state: &mut IdeState, params| {
                state.git_log.branch = params
                    .get("branch")
                    .and_then(Value::as_str)
                    .filter(|branch| !branch.is_empty())
                    .map(Into::into);
                state.refresh_git_log().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.log.filterText",
                "Filter Log by Text or Hash",
                "Git",
                None,
                r#"{"text":"string"}"#,
            ),
            has_git,
            |state: &mut IdeState, params| {
                state.git_log.filter = string_param(&params, "text")?;
                Ok(json!({"visible": state.git_log.visible().len()}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.log.showFileDiff",
                "Show Diff for Commit File",
                "Git",
                None,
                r#"{"hash":"string","path":"string"}"#,
            ),
            has_git,
            |state: &mut IdeState, params| {
                let hash = string_param(&params, "hash")?;
                let path = PathBuf::from(string_param(&params, "path")?);
                state
                    .show_commit_diff(&hash, &path)
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use crate::{IdeState, ToolWindow, built_in_commands};
    use serde_json::{Value, json};
    use std::fs;
    use std::path::Path;

    fn git(root: &Path, args: &[&str]) {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }

    #[test]
    fn loads_filters_and_inspects_history() {
        let root = std::env::temp_dir().join(format!("jetthing-core-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.email", "dev@example.invalid"]);
        git(&root, &["config", "user.name", "Dev"]);
        fs::write(root.join("a.txt"), "one\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-qm", "First"]);
        git(&root, &["checkout", "-qb", "topic"]);
        fs::write(root.join("a.txt"), "two\n").unwrap();
        git(&root, &["commit", "-qam", "Second on topic"]);
        git(&root, &["checkout", "-q", "main"]);

        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("git.log.show", &mut state, Value::Null)
            .unwrap();
        assert!(state.tool_windows.is_visible(ToolWindow::Git));
        assert_eq!(state.git_log.commits.len(), 2);
        assert_eq!(state.git_log.graph.len(), 2);
        // Newest commit is selected with its details.
        let newest = state.git_log.commits[0].hash.clone();
        assert_eq!(state.git_log.selected.as_ref(), Some(&newest));
        assert_eq!(state.git_log.details.as_ref().unwrap().files.len(), 1);

        let result = commands
            .execute("git.log.filterText", &mut state, json!({"text": "topic"}))
            .unwrap();
        assert_eq!(result["visible"], 1);
        commands
            .execute(
                "git.log.filterBranch",
                &mut state,
                json!({"branch": "main"}),
            )
            .unwrap();
        assert_eq!(state.git_log.commits.len(), 1);
        commands
            .execute("git.log.filterBranch", &mut state, json!({}))
            .unwrap();
        assert_eq!(state.git_log.commits.len(), 2);

        commands
            .execute(
                "git.log.showFileDiff",
                &mut state,
                json!({"hash": newest, "path": "a.txt"}),
            )
            .unwrap();
        let tab = state.diff.as_ref().unwrap();
        assert_eq!(tab.diff.old_text, "one\n");
        assert_eq!(tab.diff.new_text, "two\n");
        assert!(tab.new_revision.is_some());
        assert!(
            commands
                .execute("git.log.selectCommit", &mut state, json!({"hash": "ffff"}))
                .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
