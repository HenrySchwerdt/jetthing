//! Language server commands: completion, server status, restart and notices.
use crate::{IdeState, spec, string_param, usize_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use serde_json::{Value, json};
use std::path::PathBuf;

impl IdeState {
    /// Absolute path of the active document.
    pub fn active_absolute_path(&self) -> Option<PathBuf> {
        let document = self.documents.get(self.active_document?)?;
        Some(self.project.as_ref()?.root().join(&document.path))
    }

    /// Whether a language server is registered (and enabled) for the active document.
    pub fn active_has_language_server(&self) -> bool {
        self.active_absolute_path()
            .is_some_and(|path| self.lsp.handles(&path))
    }

    /// Restarts every language server and re-opens the open documents.
    pub fn restart_language_servers(&mut self) {
        self.lsp.restart_all();
        if let Some(root) = self.project.as_ref().map(|p| p.root().to_owned()) {
            self.lsp.set_project(Some(&root));
        }
        let documents: Vec<(PathBuf, String, i32)> = self
            .documents
            .iter()
            .map(|document| {
                (
                    document.path.clone(),
                    document.text.text(),
                    document.text.revision().min(i32::MAX as u64) as i32,
                )
            })
            .collect();
        for (path, text, version) in documents {
            self.sync_document_lsp(&path, &text, version);
        }
        self.status = "Restarted language servers".into();
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec(
                "language.complete",
                "Code Completion",
                "Code",
                Some("⌃Space"),
                "{}",
            ),
            |state: &IdeState| state.active_has_language_server(),
            |state: &mut IdeState, _| {
                let id = state
                    .request_active_completion()
                    .map_err(CommandError::Failed)?;
                Ok(json!({"requestId": id}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "language.completion",
                "Request Completion",
                "Code",
                None,
                r#"{"path":"string","line":0,"character":0}"#,
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, params| {
                let relative = string_param(&params, "path")?;
                let line = usize_param(&params, "line")?;
                let character = usize_param(&params, "character")?;
                let root = state
                    .project
                    .as_ref()
                    .ok_or_else(|| CommandError::Disabled("language.completion".into()))?
                    .root()
                    .to_owned();
                let id = state
                    .lsp
                    .request_completion(&root.join(relative), line, character)
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                Ok(json!({"requestId": id}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "language.restartServers",
                "Restart Language Servers",
                "Code",
                None,
                "{}",
            ),
            |state: &IdeState| state.project.is_some(),
            |state: &mut IdeState, _| {
                state.restart_language_servers();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "language.servers",
                "Language Server Status",
                "Code",
                None,
                "{}",
            ),
            |_| true,
            |state: &mut IdeState, _| {
                Ok(json!({
                    "servers": state.lsp.servers(),
                    "missing": state.lsp.missing_servers().collect::<Vec<_>>(),
                    "notices": state.lsp.notices(),
                }))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "language.dismissNotice",
                "Dismiss Language Server Notice",
                "Code",
                None,
                r#"{"id":"string"}"#,
            ),
            |state: &IdeState| !state.lsp.notices().is_empty(),
            |state: &mut IdeState, params| {
                state.lsp.dismiss_notice(&string_param(&params, "id")?);
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use crate::{IdeState, built_in_commands};
    use serde_json::json;

    #[test]
    fn completion_is_enabled_for_java_files_and_explains_a_missing_server() {
        let root = std::env::temp_dir().join(format!("jetthing-lsp-cmd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".jetthing")).unwrap();
        std::fs::write(root.join("App.java"), "class App {}\n").unwrap();
        std::fs::write(root.join("notes.txt"), "plain\n").unwrap();
        std::fs::write(
            root.join(".jetthing/lsp.toml"),
            "[servers.jdtls]\ncommand = \"/nonexistent/jdtls\"\n",
        )
        .unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": "notes.txt"}))
            .unwrap();
        assert!(!commands.is_enabled("language.complete", &state));
        commands
            .execute("editor.openFile", &mut state, json!({"path": "App.java"}))
            .unwrap();
        assert!(commands.is_enabled("language.complete", &state));
        let error = commands
            .execute("language.complete", &mut state, json!({}))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("Java language server `jdtls` was not found"),
            "{error}"
        );
        let status = commands
            .execute("language.servers", &mut state, json!({}))
            .unwrap();
        assert_eq!(status["missing"], json!(["jdtls"]));
        assert_eq!(status["notices"][0]["id"], "missing:jdtls");
        commands
            .execute(
                "language.dismissNotice",
                &mut state,
                json!({"id": "missing:jdtls"}),
            )
            .unwrap();
        assert!(state.lsp.notices().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
