use jetthing_command::{Command, CommandError, CommandRegistry, CommandSpec};
use jetthing_git::RepositoryStatus;
use jetthing_language::{LanguageAnalysis, analyze};
use jetthing_plugins::PluginManifest;
use jetthing_project::{FileEntry, Project};
use jetthing_runner::{TaskPlan, TaskService, TaskState};
use jetthing_search::{SearchEvent, SearchService, TextMatch};
use jetthing_terminal::TerminalSession;
use jetthing_text::TextDocument;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub text: TextDocument,
    pub analysis: LanguageAnalysis,
}

#[derive(Default)]
pub struct IdeState {
    pub project: Option<Project>,
    pub files: Vec<FileEntry>,
    pub documents: Vec<Document>,
    pub active_document: Option<usize>,
    pub plugins: Vec<PluginManifest>,
    pub terminal: Option<TerminalSession>,
    pub terminal_visible: bool,
    pub search: SearchService,
    pub search_results: Vec<TextMatch>,
    pub search_query: String,
    pub search_visible: bool,
    pub search_files_scanned: usize,
    pub search_truncated: bool,
    pub git: Option<RepositoryStatus>,
    pub git_visible: bool,
    pub runner: TaskService,
    pub build_visible: bool,
    pub status: String,
}

impl IdeState {
    pub fn open_project(&mut self, path: &Path) -> Result<(), String> {
        let project = Project::open(path).map_err(|e| e.to_string())?;
        self.files = project.entries(20_000);
        self.documents.clear();
        self.active_document = None;
        self.search.cancel();
        self.search_results.clear();
        self.git = RepositoryStatus::load(project.root()).ok();
        self.status = format!("Opened {}", project.root().display());
        self.project = Some(project);
        Ok(())
    }
    pub fn open_file(&mut self, path: &Path) -> Result<(), String> {
        if let Some(index) = self.documents.iter().position(|d| d.path == path) {
            self.active_document = Some(index);
            return Ok(());
        }
        let project = self.project.as_ref().ok_or("no project is open")?;
        let text = project.read_text(path).map_err(|e| e.to_string())?;
        self.documents.push(Document {
            path: path.to_owned(),
            text: TextDocument::new(&text),
            analysis: analyze(path, &text),
        });
        self.active_document = Some(self.documents.len() - 1);
        self.status = format!("Opened {}", path.display());
        Ok(())
    }
    pub fn save(&mut self, index: usize) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("no project is open")?;
        let document = self
            .documents
            .get_mut(index)
            .ok_or("document does not exist")?;
        project
            .write_text(&document.path, &document.text.text())
            .map_err(|e| e.to_string())?;
        document.text.mark_saved();
        self.status = format!("Saved {}", document.path.display());
        Ok(())
    }

    pub fn undo_active(&mut self) -> bool {
        self.active_document
            .and_then(|index| self.documents.get_mut(index))
            .is_some_and(|document| document.text.undo())
    }

    pub fn redo_active(&mut self) -> bool {
        self.active_document
            .and_then(|index| self.documents.get_mut(index))
            .is_some_and(|document| document.text.redo())
    }

    pub fn refresh_active_analysis(&mut self) {
        if let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get_mut(index))
        {
            document.analysis = analyze(&document.path, &document.text.text());
        }
    }

    pub fn open_terminal(&mut self) -> Result<(), String> {
        if self.terminal.is_none() {
            let cwd = self
                .project
                .as_ref()
                .map(Project::root)
                .unwrap_or_else(|| Path::new("."));
            self.terminal = Some(TerminalSession::spawn_shell(cwd).map_err(|e| e.to_string())?);
        }
        self.terminal_visible = true;
        self.status = "Terminal ready".into();
        Ok(())
    }

    pub fn start_search(&mut self, query: String) -> Result<u64, String> {
        if query.is_empty() {
            return Err("search query cannot be empty".into());
        }
        let project = self.project.as_ref().ok_or("no project is open")?;
        let files = self
            .files
            .iter()
            .filter(|entry| !entry.is_dir)
            .map(|entry| entry.relative_path.clone())
            .collect();
        self.search_results.clear();
        self.search_files_scanned = 0;
        self.search_truncated = false;
        self.search_query = query.clone();
        self.search_visible = true;
        let job = self
            .search
            .start(project.root().to_owned(), files, query)
            .map_err(|error| format!("failed to start search: {error}"))?;
        self.status = format!("Searching project (job {job})");
        Ok(job)
    }

    pub fn poll_search(&mut self) {
        for event in self.search.poll() {
            match event {
                SearchEvent::Match(result) => self.search_results.push(result),
                SearchEvent::Finished {
                    files_scanned,
                    truncated,
                    ..
                } => {
                    self.search_files_scanned = files_scanned;
                    self.search_truncated = truncated;
                    self.status = format!(
                        "Found {} result{} in {} files{}",
                        self.search_results.len(),
                        if self.search_results.len() == 1 {
                            ""
                        } else {
                            "s"
                        },
                        files_scanned,
                        if truncated { " (limit reached)" } else { "" }
                    );
                }
            }
        }
    }

    pub fn refresh_git(&mut self) -> Result<(), String> {
        let root = self.project.as_ref().ok_or("no project is open")?.root();
        self.git = Some(RepositoryStatus::load(root).map_err(|error| error.to_string())?);
        if let Some(git) = &self.git {
            self.status = format!("{} changes on {}", git.changes.len(), git.branch);
        }
        Ok(())
    }

    pub fn stage_path(&mut self, path: &Path) -> Result<(), String> {
        self.git
            .as_ref()
            .ok_or("project is not a Git repository")?
            .stage(path)
            .map_err(|error| error.to_string())?;
        self.refresh_git()
    }

    pub fn unstage_path(&mut self, path: &Path) -> Result<(), String> {
        self.git
            .as_ref()
            .ok_or("project is not a Git repository")?
            .unstage(path)
            .map_err(|error| error.to_string())?;
        self.refresh_git()
    }

    pub fn start_build(&mut self) -> Result<(), String> {
        let root = self.project.as_ref().ok_or("no project is open")?.root();
        let plan = TaskPlan::detect_build(root).map_err(|error| error.to_string())?;
        let name = plan.name.clone();
        self.runner.start(plan).map_err(|error| error.to_string())?;
        self.build_visible = true;
        self.status = format!("Running {name}");
        Ok(())
    }

    pub fn poll_runner(&mut self) {
        let was_running = self.runner.is_running();
        if let Err(error) = self.runner.poll() {
            self.status = error.to_string();
            return;
        }
        if was_running && !self.runner.is_running() {
            self.status = match self.runner.state() {
                TaskState::Exited(0) => "Build finished successfully".into(),
                TaskState::Exited(code) => format!("Build failed with exit code {code}"),
                TaskState::Cancelled => "Build cancelled".into(),
                _ => self.status.clone(),
            };
        }
    }
}

fn spec(
    id: &str,
    label: &str,
    category: &str,
    shortcut: Option<&str>,
    params: &str,
) -> CommandSpec {
    CommandSpec {
        id: id.into(),
        label: label.into(),
        category: category.into(),
        shortcut: shortcut.map(Into::into),
        params: params.into(),
    }
}
fn string_param(params: &Value, name: &str) -> Result<String, CommandError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .map(Into::into)
        .ok_or_else(|| CommandError::InvalidParams(format!("expected string field `{name}`")))
}

fn string_array_param(params: &Value, name: &str) -> Result<Vec<String>, CommandError> {
    params
        .get(name)
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| {
                    CommandError::InvalidParams(format!("expected array field `{name}`"))
                })?
                .iter()
                .map(|item| {
                    item.as_str().map(Into::into).ok_or_else(|| {
                        CommandError::InvalidParams(format!("expected strings in `{name}`"))
                    })
                })
                .collect()
        })
        .transpose()
        .map(|value| value.unwrap_or_default())
}

pub fn built_in_commands() -> CommandRegistry<IdeState> {
    let mut registry = CommandRegistry::default();
    registry
        .register(Command::new(
            spec(
                "project.open",
                "Open Project",
                "Project",
                Some("⌘O"),
                r#"{"path":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                state
                    .open_project(Path::new(&string_param(&params, "path")?))
                    .map_err(CommandError::Failed)?;
                Ok(json!({"root": state.project.as_ref().unwrap().root()}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("build.project", "Build Project", "Build", Some("⌘F9"), "{}"),
            |state: &IdeState| state.project.is_some() && !state.runner.is_running(),
            |state: &mut IdeState, _| {
                state.start_build().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "run.task",
                "Run Task",
                "Run",
                None,
                r#"{"program":"string","args":["string"]}"#,
            ),
            |state: &IdeState| state.project.is_some() && !state.runner.is_running(),
            |state: &mut IdeState, params| {
                let program = string_param(&params, "program")?;
                let args = string_array_param(&params, "args")?;
                let cwd = state
                    .project
                    .as_ref()
                    .ok_or_else(|| CommandError::Disabled("run.task".into()))?
                    .root()
                    .to_owned();
                state
                    .runner
                    .start(TaskPlan {
                        name: program.clone(),
                        program,
                        args,
                        cwd,
                    })
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                state.build_visible = true;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("run.cancel", "Stop Running Task", "Run", None, "{}"),
            |state: &IdeState| state.runner.is_running(),
            |state: &mut IdeState, _| {
                state
                    .runner
                    .cancel()
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                state.status = "Task cancelled".into();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("git.refresh", "Refresh Git Status", "Git", None, "{}"),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, _| {
                state.refresh_git().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.stage",
                "Stage File",
                "Git",
                None,
                r#"{"path":"string"}"#,
            ),
            |state: &IdeState| state.git.is_some(),
            |state: &mut IdeState, params| {
                state
                    .stage_path(Path::new(&string_param(&params, "path")?))
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "git.unstage",
                "Unstage File",
                "Git",
                None,
                r#"{"path":"string"}"#,
            ),
            |state: &IdeState| state.git.is_some(),
            |state: &mut IdeState, params| {
                state
                    .unstage_path(Path::new(&string_param(&params, "path")?))
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "search.project",
                "Find in Files",
                "Search",
                Some("⇧⌘F"),
                r#"{"query":"string"}"#,
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, params| {
                let query = string_param(&params, "query")?;
                let job = state.start_search(query).map_err(CommandError::Failed)?;
                Ok(json!({"job": job}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("search.cancel", "Cancel Search", "Search", None, "{}"),
            |state: &IdeState| state.search.is_running(),
            |state: &mut IdeState, _| {
                state.search.cancel();
                state.status = "Search cancelled".into();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "terminal.open",
                "Open Terminal",
                "Terminal",
                Some("⌥F12"),
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| {
                state.open_terminal().map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("terminal.toggle", "Toggle Terminal", "Terminal", None, "{}"),
            |_| true,
            |state: &mut IdeState, _| {
                if state.terminal.is_none() {
                    state.open_terminal().map_err(CommandError::Failed)?;
                } else {
                    state.terminal_visible = !state.terminal_visible;
                }
                Ok(json!({"visible": state.terminal_visible}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "terminal.sendText",
                "Send Text to Terminal",
                "Terminal",
                None,
                r#"{"text":"string"}"#,
            ),
            |state: &IdeState| state.terminal.is_some(),
            |state: &mut IdeState, params| {
                let text = string_param(&params, "text")?;
                state
                    .terminal
                    .as_mut()
                    .ok_or_else(|| CommandError::Disabled("terminal.sendText".into()))?
                    .send(&text)
                    .map_err(|e| CommandError::Failed(e.to_string()))?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "editor.openFile",
                "Open File",
                "Editor",
                None,
                r#"{"path":"string"}"#,
            ),
            |s: &IdeState| s.project.is_some(),
            |state: &mut IdeState, params| {
                state
                    .open_file(Path::new(&string_param(&params, "path")?))
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("editor.save", "Save", "Editor", Some("⌘S"), "{}"),
            |s: &IdeState| s.active_document.is_some(),
            |state: &mut IdeState, _| {
                state
                    .save(
                        state
                            .active_document
                            .ok_or_else(|| CommandError::Disabled("editor.save".into()))?,
                    )
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("editor.saveAll", "Save All", "Editor", Some("⌥⌘S"), "{}"),
            |s: &IdeState| s.documents.iter().any(|d| d.text.is_dirty()),
            |state: &mut IdeState, _| {
                for index in 0..state.documents.len() {
                    if state.documents[index].text.is_dirty() {
                        state.save(index).map_err(CommandError::Failed)?;
                    }
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("editor.undo", "Undo", "Editor", Some("⌘Z"), "{}"),
            |s: &IdeState| {
                s.active_document
                    .and_then(|i| s.documents.get(i))
                    .is_some_and(|d| d.text.can_undo())
            },
            |state: &mut IdeState, _| {
                if state.undo_active() {
                    state.refresh_active_analysis();
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("editor.redo", "Redo", "Editor", Some("⇧⌘Z"), "{}"),
            |s: &IdeState| {
                s.active_document
                    .and_then(|i| s.documents.get(i))
                    .is_some_and(|d| d.text.can_redo())
            },
            |state: &mut IdeState, _| {
                if state.redo_active() {
                    state.refresh_active_analysis();
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("plugin.refresh", "Refresh Plugins", "Plugins", None, "{}"),
            |_| true,
            |state: &mut IdeState, _| {
                let root = state.project.as_ref().map(|p| p.root().join("plugins"));
                state.plugins = root
                    .as_deref()
                    .map(jetthing_plugins::discover)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(Result::ok)
                    .collect();
                Ok(json!({"count": state.plugins.len()}))
            },
        ))
        .expect("unique built-in id");
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        thread,
        time::{Duration, Instant},
    };
    #[test]
    fn project_open_command_is_shared_engine_behavior() {
        let root = std::env::temp_dir().join(format!("jetthing-core-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        assert!(state.project.is_some());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_search_command_streams_results_into_state() {
        let root =
            std::env::temp_dir().join(format!("jetthing-core-search-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("sample.txt"), "alpha needle omega\n").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("search.project", &mut state, json!({"query": "needle"}))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while state.search.is_running() && Instant::now() < deadline {
            state.poll_search();
            thread::sleep(Duration::from_millis(5));
        }
        state.poll_search();
        assert_eq!(state.search_results.len(), 1);
        assert_eq!(state.search_results[0].path, Path::new("sample.txt"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
