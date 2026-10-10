use jetthing_command::{Command, CommandError, CommandRegistry, CommandSpec};
use jetthing_git::RepositoryStatus;
use jetthing_language::{LanguageAnalysis, analyze};
use jetthing_lsp::{LspDiagnostic, LspManager};
use jetthing_plugins::PluginManifest;
use jetthing_project::{FileEntry, Project};
use jetthing_runner::{TaskPlan, TaskService, TaskState};
use jetthing_search::{SearchEvent, SearchService, TextMatch};
use jetthing_terminal::TerminalSession;
use jetthing_text::{Selection, SelectionSet, TextDocument};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub mod build_info;
mod diagnostic_commands;
pub mod diagnostics;
mod editor_commands;
mod lsp_commands;
mod navigation;
mod problems;
mod project_commands;
mod terminal_commands;
mod toolchain_commands;
mod toolwindow;
mod update_commands;
pub use navigation::{NavigationItem, NavigationKind, NavigationList, NavigationState, Place};
pub use problems::{FileProblems, Problem, ProblemCounts, ProblemIndex, Severity};
pub use toolchain_commands::ToolchainState;
pub use toolwindow::{Anchor, ToolWindow, ToolWindowLayout};
pub use update_commands::{
    CURRENT_VERSION, UpdateState, installable_executable as update_installable,
};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub text: TextDocument,
    pub analysis: LanguageAnalysis,
    pub selections: SelectionSet,
}

#[derive(Default)]
pub struct IdeState {
    pub project: Option<Project>,
    pub files: Vec<FileEntry>,
    pub documents: Vec<Document>,
    pub active_document: Option<usize>,
    pub plugins: Vec<PluginManifest>,
    pub terminal: Option<TerminalSession>,
    pub search: SearchService,
    pub search_results: Vec<TextMatch>,
    pub search_query: String,
    pub search_files_scanned: usize,
    pub search_truncated: bool,
    pub git: Option<RepositoryStatus>,
    pub runner: TaskService,
    /// Language servers of the open project, started lazily per language.
    pub lsp: LspManager,
    /// Project-wide problems, refreshed by [`IdeState::refresh_problems`].
    pub problems: ProblemIndex,
    /// Navigation history and pending/finished navigation requests.
    pub navigation: NavigationState,
    /// Toolchain provisioning: discovery, pins and background installs.
    pub toolchains: ToolchainState,
    pub tool_windows: ToolWindowLayout,
    /// Project-relative directories expanded in the Project tool window.
    pub expanded_dirs: BTreeSet<PathBuf>,
    /// Most recently used files, newest first (Recent Files, ⌘E).
    pub recent_files: Vec<PathBuf>,
    pub updates: jetthing_update::UpdateService,
    pub update_state: UpdateState,
    /// Whether the update balloon is showing.
    pub update_notification: bool,
    /// Whether Help → About is showing (`app.about`).
    pub about_open: bool,
    /// Unseen crash report from a previous session (`app.dismissCrashReport`).
    pub crash_report: Option<PathBuf>,
    update_silent: bool,
    /// Original install path, retained when the running binary is renamed.
    update_executable: Option<PathBuf>,
    pub status: String,
}

/// Status bar summary after a Git refresh: "1 change on main", "3 changes on main".
pub fn changes_summary(count: usize, branch: &str) -> String {
    let noun = if count == 1 { "change" } else { "changes" };
    format!("{count} {noun} on {branch}")
}

impl IdeState {
    pub fn open_project(&mut self, path: &Path) -> Result<(), String> {
        let project = Project::open(path).map_err(|e| e.to_string())?;
        self.prepare_project_switch()?;
        self.install_project(project)
    }

    pub(crate) fn prepare_project_switch(&mut self) -> Result<(), String> {
        if self.project.is_some() {
            if self.runner.is_running() {
                return Err("Stop the running task before opening another project.".into());
            }
            self.save_all()?;
            self.save_workspace()?;
        }
        Ok(())
    }

    pub(crate) fn install_project(&mut self, project: Project) -> Result<(), String> {
        self.files = project.entries(20_000);
        self.documents.clear();
        self.active_document = None;
        self.search.cancel();
        self.search_results.clear();
        self.search_query.clear();
        self.search_files_scanned = 0;
        self.search_truncated = false;
        self.terminal = None;
        self.runner = TaskService::default();
        self.plugins.clear();
        self.git = RepositoryStatus::load(project.root()).ok();
        self.lsp.set_project(Some(project.root()));
        self.navigation = NavigationState::default();
        self.toolchains.reload(Some(project.root()));
        self.expanded_dirs.clear();
        self.recent_files.clear();
        self.tool_windows = ToolWindowLayout::default();
        self.tool_windows.show(ToolWindow::Project);
        self.status = format!("Opened {}", project.root().display());
        self.project = Some(project);
        self.restore_workspace();
        Ok(())
    }
    pub fn open_file(&mut self, path: &Path) -> Result<(), String> {
        if let Some(index) = self.documents.iter().position(|d| d.path == path) {
            self.active_document = Some(index);
            self.touch_recent(path);
            return Ok(());
        }
        let project = self.project.as_ref().ok_or("no project is open")?;
        let text = project.read_text(path).map_err(|e| e.to_string())?;
        self.documents.push(Document {
            path: path.to_owned(),
            text: TextDocument::new(&text),
            analysis: analyze(path, &text),
            selections: SelectionSet::default(),
        });
        self.active_document = Some(self.documents.len() - 1);
        self.touch_recent(path);
        self.status = format!("Opened {}", path.display());
        self.sync_document_lsp(path, &text, 0);
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
        let absolute = project.root().join(&document.path);
        let _ = self.lsp.save_document(&absolute);
        Ok(())
    }

    /// Closes a tab. Like the reference IDE, modified content is saved first
    /// rather than discarded.
    pub fn close_document(&mut self, index: usize) -> Result<(), String> {
        let document = self.documents.get(index).ok_or("document does not exist")?;
        if document.text.is_dirty() {
            self.save(index)?;
        }
        let document = self.documents.remove(index);
        self.active_document = match self.active_document {
            _ if self.documents.is_empty() => None,
            Some(active) if active > index => Some(active - 1),
            Some(active) if active == index => Some(index.min(self.documents.len() - 1)),
            other => other,
        };
        self.lsp.clear_completions();
        if let Some(project) = &self.project {
            let _ = self
                .lsp
                .close_document(&project.root().join(&document.path));
        }
        self.status = format!("Closed {}", document.path.display());
        Ok(())
    }

    pub fn activate_document(&mut self, index: usize) -> Result<(), String> {
        if index >= self.documents.len() {
            return Err("document does not exist".into());
        }
        self.active_document = Some(index);
        let path = self.documents[index].path.clone();
        self.touch_recent(&path);
        self.lsp.clear_completions();
        Ok(())
    }

    /// Moves `path` to the front of the recent-files list.
    pub fn touch_recent(&mut self, path: &Path) {
        self.recent_files.retain(|recent| recent != path);
        self.recent_files.insert(0, path.to_owned());
        self.recent_files.truncate(50);
    }

    /// Expands or collapses a directory in the Project tool window.
    /// Returns whether the directory is expanded afterwards.
    pub fn toggle_directory(&mut self, path: &Path) -> Result<bool, String> {
        if !self
            .files
            .iter()
            .any(|e| e.is_dir && e.relative_path == path)
        {
            return Err(format!("{} is not a project directory", path.display()));
        }
        if self.expanded_dirs.remove(path) {
            Ok(false)
        } else {
            self.expanded_dirs.insert(path.to_owned());
            Ok(true)
        }
    }

    /// Entries visible in the Project tree: every ancestor directory is expanded.
    pub fn visible_files(&self) -> Vec<&FileEntry> {
        self.files
            .iter()
            .filter(|entry| {
                entry
                    .relative_path
                    .ancestors()
                    .skip(1)
                    .filter(|ancestor| !ancestor.as_os_str().is_empty())
                    .all(|ancestor| self.expanded_dirs.contains(ancestor))
            })
            .collect()
    }

    pub fn undo_active(&mut self) -> bool {
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get_mut(index))
        else {
            return false;
        };
        if !document.text.undo() {
            return false;
        }
        document.selections.clamp(document.text.len_chars());
        true
    }

    pub fn redo_active(&mut self) -> bool {
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get_mut(index))
        else {
            return false;
        };
        if !document.text.redo() {
            return false;
        }
        document.selections.clamp(document.text.len_chars());
        true
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
        if self
            .terminal
            .as_ref()
            .is_none_or(|terminal| terminal.is_exited())
        {
            let cwd = self
                .project
                .as_ref()
                .map(Project::root)
                .unwrap_or_else(|| Path::new("."));
            self.terminal = Some(TerminalSession::spawn_shell(cwd).map_err(|e| e.to_string())?);
        }
        self.tool_windows.show(ToolWindow::Terminal);
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
        self.tool_windows.show(ToolWindow::Find);
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
            self.status = changes_summary(git.changes.len(), &git.branch);
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
        self.tool_windows.show(ToolWindow::Build);
        self.status = format!("Running {name}");
        Ok(())
    }

    pub fn start_run(&mut self) -> Result<(), String> {
        let root = self.project.as_ref().ok_or("no project is open")?.root();
        let plan = TaskPlan::detect_run(root).map_err(|error| error.to_string())?;
        let name = plan.name.clone();
        self.runner.start(plan).map_err(|error| error.to_string())?;
        self.tool_windows.show(ToolWindow::Build);
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
            let name = self
                .runner
                .plan()
                .map(|plan| plan.name.clone())
                .unwrap_or_else(|| "Task".into());
            self.status = match self.runner.state() {
                TaskState::Exited(0) => format!("{name} finished successfully"),
                TaskState::Exited(code) => format!("{name} failed with exit code {code}"),
                TaskState::Cancelled => format!("{name} cancelled"),
                _ => self.status.clone(),
            };
        }
    }

    /// Pumps the language servers and re-opens documents on restarted servers.
    pub fn poll_lsp(&mut self) {
        let restarted = self.lsp.poll();
        self.refresh_problems();
        self.poll_navigation();
        let Some(root) = self.project.as_ref().map(|p| p.root().to_owned()) else {
            return;
        };
        for absolute in restarted {
            let Some(document) = self
                .documents
                .iter()
                .find(|document| root.join(&document.path) == absolute)
            else {
                continue;
            };
            let version = document.text.revision().min(i32::MAX as u64) as i32;
            let _ = self
                .lsp
                .sync_document(&absolute, &document.text.text(), version);
        }
    }

    /// Opens or updates a document on its language server, starting the
    /// server on first use. A missing server raises a notice, not an error.
    pub fn sync_document_lsp(&mut self, relative_path: &Path, text: &str, version: i32) {
        let Some(project) = &self.project else {
            return;
        };
        self.lsp.clear_completions();
        let absolute = project.root().join(relative_path);
        match self.lsp.sync_document(&absolute, text, version) {
            Ok(()) | Err(jetthing_lsp::LspError::ServerNotFound { .. }) => {}
            Err(error) => self.status = error.to_string(),
        }
    }

    pub fn sync_active_lsp(&mut self) {
        let Some(index) = self.active_document else {
            return;
        };
        let Some(document) = self.documents.get(index) else {
            return;
        };
        let path = document.path.clone();
        let text = document.text.text();
        let version = document.text.revision().min(i32::MAX as u64) as i32;
        self.sync_document_lsp(&path, &text, version);
    }

    pub fn active_caret(&self) -> Option<usize> {
        self.active_document
            .and_then(|index| self.documents.get(index))
            .map(|document| document.selections.primary().head)
    }

    pub fn request_active_completion(&mut self) -> Result<u64, String> {
        let index = self.active_document.ok_or("no active document")?;
        let document = self
            .documents
            .get(index)
            .ok_or("active document does not exist")?;
        let (line, character) = document
            .text
            .lsp_position(document.selections.primary().head)
            .map_err(|e| e.to_string())?;
        let absolute = self
            .project
            .as_ref()
            .ok_or("no project is open")?
            .root()
            .join(&document.path);
        self.lsp
            .request_completion(&absolute, line, character)
            .map_err(|e| e.to_string())
    }

    /// Runs an edit on the active document and refreshes analysis/LSP if it changed.
    pub fn edit_active<T>(
        &mut self,
        edit: impl FnOnce(&mut Document) -> Result<T, String>,
    ) -> Result<T, String> {
        let index = self.active_document.ok_or("no active document")?;
        let document = self
            .documents
            .get_mut(index)
            .ok_or("active document does not exist")?;
        let revision = document.text.revision();
        let result = edit(document)?;
        if document.text.revision() != revision {
            document.analysis = analyze(&document.path, &document.text.text());
            self.sync_active_lsp();
            self.lsp.clear_completions();
        }
        Ok(result)
    }

    pub fn insert_active_text(&mut self, position: usize, text: &str) -> Result<(), String> {
        let index = self.active_document.ok_or("no active document")?;
        let document = self
            .documents
            .get_mut(index)
            .ok_or("active document does not exist")?;
        document
            .text
            .edit(position, position, text)
            .map_err(|e| e.to_string())?;
        document.analysis = analyze(&document.path, &document.text.text());
        document
            .selections
            .set_single(Selection::caret(position + text.chars().count()));
        self.sync_active_lsp();
        self.lsp.clear_completions();
        Ok(())
    }

    pub fn active_lsp_diagnostics(&self) -> Vec<LspDiagnostic> {
        let Some(index) = self.active_document else {
            return Vec::new();
        };
        let Some(document) = self.documents.get(index) else {
            return Vec::new();
        };
        let Some(project) = &self.project else {
            return Vec::new();
        };
        self.lsp
            .diagnostics_for(&project.root().join(&document.path))
    }
}

pub(crate) fn spec(
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
pub(crate) fn string_param(params: &Value, name: &str) -> Result<String, CommandError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .map(Into::into)
        .ok_or_else(|| CommandError::InvalidParams(format!("expected string field `{name}`")))
}

pub(crate) fn string_array_param(params: &Value, name: &str) -> Result<Vec<String>, CommandError> {
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

fn tool_window_param(params: &Value) -> Result<ToolWindow, CommandError> {
    let id = string_param(params, "id")?;
    ToolWindow::parse(&id)
        .ok_or_else(|| CommandError::InvalidParams(format!("unknown tool window `{id}`")))
}

pub(crate) fn usize_param(params: &Value, name: &str) -> Result<usize, CommandError> {
    params
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            CommandError::InvalidParams(format!("expected non-negative integer field `{name}`"))
        })
}

pub fn built_in_commands() -> CommandRegistry<IdeState> {
    let mut registry = CommandRegistry::default();
    editor_commands::register(&mut registry);
    project_commands::register(&mut registry);
    update_commands::register(&mut registry);
    diagnostic_commands::register(&mut registry);
    terminal_commands::register(&mut registry);
    lsp_commands::register(&mut registry);
    problems::register(&mut registry);
    navigation::register(&mut registry);
    toolchain_commands::register(&mut registry);
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
            spec(
                "editor.insertText",
                "Insert Text",
                "Editor",
                None,
                r#"{"position":0,"text":"string"}"#,
            ),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, params| {
                let position = usize_param(&params, "position")?;
                let text = string_param(&params, "text")?;
                state
                    .insert_active_text(position, &text)
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
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
            spec("run.project", "Run", "Run", Some("⌃R"), "{}"),
            |state: &IdeState| state.project.is_some() && !state.runner.is_running(),
            |state: &mut IdeState, _| {
                state.start_run().map_err(CommandError::Failed)?;
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
                state.tool_windows.show(ToolWindow::Build);
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
                    state.tool_windows.toggle(ToolWindow::Terminal);
                }
                Ok(json!({"visible": state.tool_windows.is_visible(ToolWindow::Terminal)}))
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
            spec("editor.save", "Save", "Editor", None, "{}"),
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
            spec("editor.saveAll", "Save All", "Editor", Some("⌘S"), "{}"),
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
                    state.sync_active_lsp();
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
                    state.sync_active_lsp();
                }
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolwindow.toggle",
                "Toggle Tool Window",
                "Window",
                None,
                r#"{"id":"project|commit|find|build|problems|terminal"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let window = tool_window_param(&params)?;
                if window == ToolWindow::Terminal && state.terminal.is_none() {
                    state.open_terminal().map_err(CommandError::Failed)?;
                    return Ok(json!({"visible": true}));
                }
                if window == ToolWindow::Commit && !state.tool_windows.is_visible(window) {
                    let _ = state.refresh_git();
                }
                Ok(json!({"visible": state.tool_windows.toggle(window)}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "toolwindow.hideAll",
                "Hide All Tool Windows",
                "Window",
                Some("⇧⌘F12"),
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| {
                state.tool_windows.hide_all();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    for window in ToolWindow::ALL {
        let id = format!("toolwindow.activate.{}", window.id());
        registry
            .register(Command::new(
                spec(
                    &id,
                    &format!("{} Tool Window", window.title()),
                    "Window",
                    Some(window.shortcut()),
                    "{}",
                ),
                |_| true,
                move |state: &mut IdeState, _| {
                    if window == ToolWindow::Terminal {
                        state.open_terminal().map_err(CommandError::Failed)?;
                    }
                    if window == ToolWindow::Commit {
                        let _ = state.refresh_git();
                    }
                    state.tool_windows.show(window);
                    Ok(Value::Null)
                },
            ))
            .expect("unique built-in id");
    }
    registry
        .register(Command::new(
            spec(
                "editor.close",
                "Close Tab",
                "Editor",
                Some("⌘W"),
                r#"{"index":"number?"}"#,
            ),
            |s: &IdeState| s.active_document.is_some(),
            |state: &mut IdeState, params| {
                let index = match params.get("index") {
                    Some(_) => usize_param(&params, "index")?,
                    None => state
                        .active_document
                        .ok_or_else(|| CommandError::Disabled("editor.close".into()))?,
                };
                state.close_document(index).map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "editor.activate",
                "Select Tab",
                "Editor",
                None,
                r#"{"index":"number"}"#,
            ),
            |s: &IdeState| !s.documents.is_empty(),
            |state: &mut IdeState, params| {
                state
                    .activate_document(usize_param(&params, "index")?)
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "project.toggleDirectory",
                "Expand/Collapse Directory",
                "Project",
                None,
                r#"{"path":"string"}"#,
            ),
            |s: &IdeState| s.project.is_some(),
            |state: &mut IdeState, params| {
                let expanded = state
                    .toggle_directory(Path::new(&string_param(&params, "path")?))
                    .map_err(CommandError::Failed)?;
                Ok(json!({"expanded": expanded}))
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
    #[test]
    fn change_summary_uses_singular_for_one_change() {
        assert_eq!(super::changes_summary(1, "master"), "1 change on master");
        assert_eq!(super::changes_summary(0, "main"), "0 changes on main");
        assert_eq!(super::changes_summary(2, "main"), "2 changes on main");
    }

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
    fn switching_projects_saves_documents_and_restores_the_previous_workspace() {
        let root = std::env::temp_dir().join(format!("jetthing-switch-{}", std::process::id()));
        let first = root.join("first");
        let second = root.join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        std::fs::write(first.join("notes.txt"), "before").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": first}))
            .unwrap();
        state.open_file(Path::new("notes.txt")).unwrap();
        state.documents[0].text.edit(0, 6, "after").unwrap();
        state.search_query = "old query".into();
        commands
            .execute("project.open", &mut state, json!({"path": second}))
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(first.join("notes.txt")).unwrap(),
            "after"
        );
        assert!(state.documents.is_empty());
        assert!(state.search_query.is_empty());
        commands
            .execute("project.open", &mut state, json!({"path": first}))
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "after");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_project_open_preserves_the_current_editor() {
        let root =
            std::env::temp_dir().join(format!("jetthing-switch-error-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("notes.txt"), "before").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        state.open_project(&root).unwrap();
        state.open_file(Path::new("notes.txt")).unwrap();
        state.documents[0].text.edit(0, 6, "unsaved").unwrap();
        assert!(
            commands
                .execute(
                    "project.open",
                    &mut state,
                    json!({"path": root.join("missing")})
                )
                .is_err()
        );
        assert_eq!(state.documents[0].text.text(), "unsaved");
        assert!(state.documents[0].text.is_dirty());
        assert_eq!(
            std::fs::read_to_string(root.join("notes.txt")).unwrap(),
            "before"
        );
        // A failed save also prevents discarding the old project.
        std::fs::remove_file(root.join("notes.txt")).unwrap();
        std::fs::create_dir(root.join("notes.txt")).unwrap();
        let next = root.join("next");
        std::fs::create_dir(&next).unwrap();
        assert!(
            commands
                .execute("project.open", &mut state, json!({"path": next}))
                .is_err()
        );
        assert_eq!(
            state.project.as_ref().unwrap().root(),
            root.canonicalize().unwrap()
        );
        assert!(state.documents[0].text.is_dirty());
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

    #[test]
    fn insert_text_command_is_transactional_and_updates_the_caret() {
        let root =
            std::env::temp_dir().join(format!("jetthing-core-insert-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("sample.rs"), "fn main() {}\n").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": "sample.rs"}))
            .unwrap();
        commands
            .execute(
                "editor.insertText",
                &mut state,
                json!({"position": 3, "text": "async "}),
            )
            .unwrap();

        assert_eq!(state.documents[0].text.text(), "fn async main() {}\n");
        assert_eq!(state.documents[0].selections.primary().head, 9);
        assert!(state.documents[0].text.can_undo());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_tree_starts_collapsed_and_expands_by_command() {
        let root = std::env::temp_dir().join(format!("jetthing-core-tree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src/nested")).unwrap();
        std::fs::write(root.join("src/nested/deep.rs"), "").unwrap();
        std::fs::write(root.join("README.md"), "").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        let visible = |state: &IdeState| {
            state
                .visible_files()
                .into_iter()
                .map(|entry| entry.relative_path.to_string_lossy().replace('\\', "/"))
                .collect::<Vec<_>>()
        };
        assert_eq!(visible(&state), ["src", "README.md"]);
        commands
            .execute(
                "project.toggleDirectory",
                &mut state,
                json!({"path": "src"}),
            )
            .unwrap();
        assert_eq!(visible(&state), ["src", "src/nested", "README.md"]);
        assert!(
            commands
                .execute(
                    "project.toggleDirectory",
                    &mut state,
                    json!({"path": "README.md"})
                )
                .is_err()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn closing_a_tab_saves_and_keeps_a_sensible_active_tab() {
        let root = std::env::temp_dir().join(format!("jetthing-core-close-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("a.txt"), "a").unwrap();
        std::fs::write(root.join("b.txt"), "b").unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        for path in ["a.txt", "b.txt"] {
            commands
                .execute("editor.openFile", &mut state, json!({"path": path}))
                .unwrap();
        }
        commands
            .execute(
                "editor.insertText",
                &mut state,
                json!({"position": 1, "text": "!"}),
            )
            .unwrap();
        commands
            .execute("editor.close", &mut state, Value::Null)
            .unwrap();
        assert_eq!(std::fs::read_to_string(root.join("b.txt")).unwrap(), "b!");
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.active_document, Some(0));
        commands
            .execute("editor.close", &mut state, json!({"index": 0}))
            .unwrap();
        assert_eq!(state.active_document, None);
        assert!(
            commands
                .execute("editor.close", &mut state, Value::Null)
                .is_err()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tool_window_commands_keep_one_window_per_anchor() {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("toolwindow.toggle", &mut state, json!({"id": "problems"}))
            .unwrap();
        commands
            .execute("toolwindow.activate.find", &mut state, Value::Null)
            .unwrap();
        assert!(state.tool_windows.is_visible(ToolWindow::Find));
        assert!(!state.tool_windows.is_visible(ToolWindow::Problems));
        assert!(
            commands
                .execute("toolwindow.toggle", &mut state, json!({"id": "bogus"}))
                .is_err()
        );
    }
}
