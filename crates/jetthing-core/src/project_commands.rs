//! Project file operations, commits, and workspace persistence.

use crate::{Anchor, IdeState, ToolWindow, spec, string_array_param, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Per-project UI state, stored like the reference IDE's `.idea/workspace.xml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Workspace {
    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub expanded_dirs: Vec<PathBuf>,
    pub left_tool_window: Option<String>,
    pub bottom_tool_window: Option<String>,
    pub recent_files: Vec<PathBuf>,
}

pub const WORKSPACE_FILE: &str = ".jetthing/workspace.json";

impl IdeState {
    fn refresh_files(&mut self) {
        if let Some(project) = &self.project {
            self.files = project.entries(20_000);
        }
        let _ = self.refresh_git();
    }

    pub fn create_path(&mut self, path: &Path, directory: bool) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        if directory {
            project.create_dir(path)
        } else {
            project.create_file(path)
        }
        .map_err(|error| error.to_string())?;
        // Reveal the new entry in the Project tree.
        for ancestor in path.ancestors().skip(1) {
            if !ancestor.as_os_str().is_empty() {
                self.expanded_dirs.insert(ancestor.to_owned());
            }
        }
        self.refresh_files();
        if !directory {
            self.open_file(path)?;
        }
        self.status = format!("Created {}", path.display());
        Ok(())
    }

    pub fn rename_path(&mut self, path: &Path, new_name: &str) -> Result<PathBuf, String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        let renamed = project
            .rename(path, new_name)
            .map_err(|error| error.to_string())?;
        let remap = |old: &Path| -> Option<PathBuf> {
            old.strip_prefix(path).ok().map(|rest| {
                if rest.as_os_str().is_empty() {
                    renamed.clone()
                } else {
                    renamed.join(rest)
                }
            })
        };
        for document in &mut self.documents {
            if let Some(next) = remap(&document.path) {
                document.path = next;
            }
        }
        self.expanded_dirs = self
            .expanded_dirs
            .iter()
            .map(|dir| remap(dir).unwrap_or_else(|| dir.clone()))
            .collect();
        for recent in &mut self.recent_files {
            if let Some(next) = remap(recent) {
                *recent = next;
            }
        }
        self.refresh_files();
        self.status = format!("Renamed to {}", renamed.display());
        Ok(renamed)
    }

    pub fn delete_path(&mut self, path: &Path) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        project.delete(path).map_err(|error| error.to_string())?;
        // Close editors for deleted files without saving them back.
        let active_path = self
            .active_document
            .and_then(|index| self.documents.get(index))
            .map(|document| document.path.clone());
        self.documents
            .retain(|document| !document.path.starts_with(path));
        self.active_document = active_path
            .and_then(|active| self.documents.iter().position(|d| d.path == active))
            .or_else(|| self.documents.len().checked_sub(1));
        self.recent_files.retain(|recent| !recent.starts_with(path));
        self.expanded_dirs.retain(|dir| !dir.starts_with(path));
        self.refresh_files();
        self.status = format!("Deleted {}", path.display());
        Ok(())
    }

    /// Commits the given project-relative paths with `message`.
    pub fn commit_paths(&mut self, message: &str, paths: &[PathBuf]) -> Result<String, String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        let git = self
            .git
            .as_ref()
            .ok_or("the project is not a Git repository")?;
        let git_root = git.root.canonicalize().unwrap_or_else(|_| git.root.clone());
        let repo_paths: Vec<PathBuf> = paths
            .iter()
            .map(|path| {
                let absolute = project.root().join(path);
                absolute
                    .strip_prefix(&git_root)
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|_| path.clone())
            })
            .collect();
        // Commits include what is in the editor, like the reference IDE.
        let _ = self.save_all();
        let hash = self
            .git
            .as_ref()
            .ok_or("the project is not a Git repository")?
            .commit(message, &repo_paths)
            .map_err(|error| error.to_string())?;
        self.refresh_git()?;
        self.status = format!(
            "Committed {} file{} ({hash})",
            paths.len(),
            if paths.len() == 1 { "" } else { "s" }
        );
        Ok(hash)
    }

    pub fn save_all(&mut self) -> Result<(), String> {
        for index in 0..self.documents.len() {
            if self.documents[index].text.is_dirty() {
                self.save(index)?;
            }
        }
        Ok(())
    }

    pub fn workspace(&self) -> Workspace {
        Workspace {
            open_files: self.documents.iter().map(|d| d.path.clone()).collect(),
            active_file: self
                .active_document
                .and_then(|index| self.documents.get(index))
                .map(|document| document.path.clone()),
            expanded_dirs: self.expanded_dirs.iter().cloned().collect(),
            left_tool_window: self
                .tool_windows
                .visible(Anchor::Left)
                .map(|window| window.id().to_owned()),
            bottom_tool_window: self
                .tool_windows
                .visible(Anchor::Bottom)
                .map(|window| window.id().to_owned()),
            recent_files: self.recent_files.clone(),
        }
    }

    pub fn save_workspace(&self) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        let path = project.root().join(WORKSPACE_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let json = serde_json::to_string_pretty(&self.workspace()).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|error| error.to_string())?;
        // Completion statistics are per user; flush them with the workspace.
        self.completion_stats.save()
    }

    /// Restores tabs, tree expansion and tool windows; missing files are skipped.
    pub(crate) fn restore_workspace(&mut self) {
        let Some(project) = &self.project else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(project.root().join(WORKSPACE_FILE)) else {
            return;
        };
        let Ok(workspace) = serde_json::from_str::<Workspace>(&text) else {
            self.status = format!("Ignored unreadable {WORKSPACE_FILE}");
            return;
        };
        let known: std::collections::HashSet<PathBuf> = self
            .files
            .iter()
            .map(|entry| entry.relative_path.clone())
            .collect();
        let keep = |paths: &[PathBuf]| -> Vec<PathBuf> {
            paths
                .iter()
                .filter(|p| known.contains(*p))
                .cloned()
                .collect()
        };
        self.expanded_dirs = keep(&workspace.expanded_dirs).into_iter().collect();
        let recent = keep(&workspace.recent_files);
        for path in keep(&workspace.open_files) {
            let _ = self.open_file(&path);
        }
        if let Some(active) = &workspace.active_file
            && let Some(index) = self.documents.iter().position(|d| &d.path == active)
        {
            self.active_document = Some(index);
        }
        self.recent_files = recent;
        self.tool_windows.hide_all();
        for id in [&workspace.left_tool_window, &workspace.bottom_tool_window]
            .into_iter()
            .flatten()
        {
            // The terminal is not restored automatically: it would spawn a shell.
            if let Some(window) = ToolWindow::parse(id).filter(|w| *w != ToolWindow::Terminal) {
                self.tool_windows.show(window);
            }
        }
    }
}

fn path_param(params: &Value) -> Result<PathBuf, CommandError> {
    string_param(params, "path").map(PathBuf::from)
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry.register(Command::new(
        spec("project.create", "Create Project", "Project", None,
            r#"{"parent":"string","name":"string","kind":"empty|rust-binary|rust-library|java-gradle-kotlin|java-gradle-groovy","group":"string?"}"#),
        |_| true,
        |state: &mut IdeState, params| {
            let parent = string_param(&params, "parent")?;
            let name = string_param(&params, "name")?;
            let kind = jetthing_project::generator::ProjectKind::parse(&string_param(&params, "kind")?)
                .ok_or_else(|| CommandError::InvalidParams("unknown project template".into()))?;
            let group = params.get("group").and_then(Value::as_str).unwrap_or("org.example");
            state.prepare_project_switch().map_err(CommandError::Failed)?;
            let project = jetthing_project::generator::create_project(Path::new(&parent), &name, kind, group)
                .map_err(|error| CommandError::Failed(error.to_string()))?;
            let root = project.root().to_owned();
            state.install_project(project).map_err(CommandError::Failed)?;
            Ok(json!({"root": root}))
        },
    )).expect("unique built-in id");
    for (id, label, directory) in [
        ("project.newFile", "New File", false),
        ("project.newDirectory", "New Directory", true),
    ] {
        registry
            .register(Command::new(
                spec(id, label, "Project", None, r#"{"path":"string"}"#),
                |state: &IdeState| state.project.is_some(),
                move |state: &mut IdeState, params| {
                    let path = path_param(&params)?;
                    state
                        .create_path(&path, directory)
                        .map_err(CommandError::Failed)?;
                    Ok(json!({"path": path}))
                },
            ))
            .expect("unique built-in id");
    }
    registry
        .register(Command::new(
            spec(
                "project.rename",
                "Rename File",
                "Project",
                Some("⇧F6"),
                r#"{"path":"string","name":"string"}"#,
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, params| {
                let path = path_param(&params)?;
                let name = string_param(&params, "name")?;
                let renamed = state
                    .rename_path(&path, &name)
                    .map_err(CommandError::Failed)?;
                Ok(json!({"path": renamed}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "project.delete",
                "Delete",
                "Project",
                Some("⌦"),
                r#"{"path":"string"}"#,
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, params| {
                let path = path_param(&params)?;
                state.delete_path(&path).map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.commit",
                "Commit",
                "Git",
                Some("⌘K"),
                r#"{"message":"string","paths":["string"]}"#,
            ),
            |state: &IdeState| state.git.is_some(),
            |state: &mut IdeState, params| {
                let message = string_param(&params, "message")?;
                let paths: Vec<PathBuf> = string_array_param(&params, "paths")?
                    .into_iter()
                    .map(PathBuf::from)
                    .collect();
                let hash = state
                    .commit_paths(&message, &paths)
                    .map_err(CommandError::Failed)?;
                Ok(json!({"commit": hash}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "workspace.save",
                "Save Workspace Layout",
                "Window",
                None,
                "{}",
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, _| {
                state.save_workspace().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use crate::built_in_commands;
    use crate::{IdeState, ToolWindow};
    use serde_json::{Value, json};
    use std::path::Path;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("jetthing-core-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn create_project_command_generates_and_opens_without_overwriting() {
        let parent = temp_root("create-project");
        let commands = built_in_commands();
        let mut state = IdeState::default();
        let params = json!({"parent": parent, "name": "hello", "kind": "java-gradle-kotlin", "group": "org.example"});
        commands
            .execute("project.create", &mut state, params.clone())
            .unwrap();
        let root = parent.join("hello").canonicalize().unwrap();
        assert_eq!(state.project.as_ref().unwrap().root(), root);
        assert!(
            state
                .files
                .iter()
                .any(|entry| entry.relative_path == Path::new("build.gradle.kts"))
        );
        let original = std::fs::read_to_string(root.join("build.gradle.kts")).unwrap();
        assert!(
            commands
                .execute("project.create", &mut state, params)
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(root.join("build.gradle.kts")).unwrap(),
            original
        );
        assert!(
            commands
                .execute(
                    "project.create",
                    &mut state,
                    json!({"parent": parent, "name": "bad", "kind": "missing"})
                )
                .is_err()
        );
        assert!(!parent.join("bad").exists());
        std::fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn file_operations_keep_editors_and_tree_consistent() {
        let root = temp_root("fileops");
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("project.newFile", &mut state, json!({"path": "src/a.rs"}))
            .unwrap();
        assert_eq!(state.documents[0].path, Path::new("src/a.rs"));
        assert!(state.expanded_dirs.contains(Path::new("src")));
        commands
            .execute(
                "project.rename",
                &mut state,
                json!({"path": "src", "name": "lib"}),
            )
            .unwrap();
        assert_eq!(state.documents[0].path, Path::new("lib/a.rs"));
        assert!(
            state
                .files
                .iter()
                .any(|e| e.relative_path == Path::new("lib/a.rs"))
        );
        assert!(
            commands
                .execute("project.newFile", &mut state, json!({"path": "../x"}))
                .is_err()
        );
        commands
            .execute("project.delete", &mut state, json!({"path": "lib"}))
            .unwrap();
        assert!(state.documents.is_empty());
        assert_eq!(state.active_document, None);
        assert!(!root.join("lib").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_round_trips_tabs_tree_and_tool_windows() {
        let root = temp_root("workspace");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/a.rs"), "").unwrap();
        std::fs::write(root.join("b.txt"), "").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        for path in ["src/a.rs", "b.txt"] {
            commands
                .execute("editor.openFile", &mut state, json!({"path": path}))
                .unwrap();
        }
        commands
            .execute("editor.activate", &mut state, json!({"index": 0}))
            .unwrap();
        commands
            .execute(
                "project.toggleDirectory",
                &mut state,
                json!({"path": "src"}),
            )
            .unwrap();
        commands
            .execute("toolwindow.activate.problems", &mut state, Value::Null)
            .unwrap();
        commands
            .execute("workspace.save", &mut state, Value::Null)
            .unwrap();

        let mut reopened = IdeState::default();
        commands
            .execute("project.open", &mut reopened, json!({"path": root}))
            .unwrap();
        assert_eq!(reopened.documents.len(), 2);
        assert_eq!(reopened.active_document, Some(0));
        assert_eq!(reopened.recent_files[0], Path::new("src/a.rs"));
        assert!(reopened.expanded_dirs.contains(Path::new("src")));
        assert!(reopened.tool_windows.is_visible(ToolWindow::Problems));
        assert!(reopened.tool_windows.is_visible(ToolWindow::Project));
        // The workspace directory itself stays out of the Project tree.
        assert!(
            !reopened
                .files
                .iter()
                .any(|e| e.relative_path.starts_with(".jetthing"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn commit_command_commits_selected_files() {
        let root = temp_root("commit");
        let git = |args: &[&str]| {
            assert!(
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(&root)
                    .args(args)
                    .status()
                    .unwrap()
                    .success()
            )
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "test@example.invalid"]);
        git(&["config", "user.name", "Jetthing Test"]);
        std::fs::write(root.join("a.txt"), "a").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        let result = commands
            .execute(
                "git.commit",
                &mut state,
                json!({"message": "Add a", "paths": ["a.txt"]}),
            )
            .unwrap();
        assert!(
            result["commit"]
                .as_str()
                .is_some_and(|hash| !hash.is_empty())
        );
        assert!(state.git.as_ref().unwrap().changes.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
