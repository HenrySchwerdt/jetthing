use jetthing_command::{Command, CommandError, CommandRegistry, CommandSpec};
use jetthing_language::{LanguageAnalysis, analyze};
use jetthing_plugins::PluginManifest;
use jetthing_project::{FileEntry, Project};
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
    pub status: String,
}

impl IdeState {
    pub fn open_project(&mut self, path: &Path) -> Result<(), String> {
        let project = Project::open(path).map_err(|e| e.to_string())?;
        self.files = project.entries(20_000);
        self.documents.clear();
        self.active_document = None;
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
}
