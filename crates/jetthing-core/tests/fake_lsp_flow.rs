//! Completion → resolve → auto-import and quick documentation through the
//! command registry, against the scriptable `jetthing-fake-lsp` server.
//!
//! The server binary is built by `cargo test --workspace`; when this crate's
//! tests run alone and it is missing, the test is skipped with a note.

use jetthing_core::{IdeState, built_in_commands};
use jetthing_lsp::{
    LanguageServerProvider, LaunchConfig, LaunchContext, LspError, LspManager, ServerRegistry,
    ServerSpec,
};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

struct Fake(PathBuf, ServerSpec);

impl LanguageServerProvider for Fake {
    fn spec(&self) -> &ServerSpec {
        &self.1
    }
    fn launch(&self, _: &LaunchContext<'_>) -> Result<LaunchConfig, LspError> {
        Ok(LaunchConfig::new("fake", &self.0))
    }
}

fn fake_server() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.parent()?;
    let path = dir.join(format!("jetthing-fake-lsp{}", std::env::consts::EXE_SUFFIX));
    path.is_file().then_some(path)
}

fn wait(state: &mut IdeState, mut done: impl FnMut(&IdeState) -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        state.poll_lsp();
        if done(state) {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn completion_resolves_docs_applies_imports_and_shows_quick_doc() {
    let Some(server) = fake_server() else {
        eprintln!("jetthing-fake-lsp not built; run `cargo test --workspace`");
        return;
    };
    let root = std::env::temp_dir().join(format!("jetthing-core-fake-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("App.fake"), "app.").unwrap();
    let mut registry = ServerRegistry::empty();
    registry.register(Arc::new(Fake(
        server,
        ServerSpec {
            id: "fake",
            language: "Fake",
            extensions: &[("fake", "fake")],
            file_names: &[],
            candidates: &[],
            root_markers: &[],
            install_hint: "",
            notify_missing: false,
            initialization_options: None,
        },
    )));
    let commands = built_in_commands();
    let mut state = IdeState::default();
    state.lsp = LspManager::new(registry);
    commands
        .execute("project.open", &mut state, json!({"path": root}))
        .unwrap();
    commands
        .execute("editor.openFile", &mut state, json!({"path": "App.fake"}))
        .unwrap();
    state.documents[0]
        .selections
        .set_single(jetthing_text::Selection::caret(4));
    assert!(wait(&mut state, |state| {
        state
            .active_absolute_path()
            .is_some_and(|path| !state.lsp.trigger_characters(&path).is_empty())
    }));

    // Ctrl+Space, then the docs panel (Ctrl+Q while the popup is open).
    commands
        .execute("language.complete", &mut state, json!({}))
        .unwrap();
    assert!(wait(&mut state, |state| {
        state.completion.as_ref().is_some_and(|s| s.visible())
    }));
    let toggled = commands
        .execute("language.quickDoc", &mut state, json!({}))
        .unwrap();
    assert_eq!(toggled["panel"], true);
    assert!(wait(&mut state, |state| {
        state
            .completion_documentation()
            .is_some_and(|doc| doc.contains("**greeting**"))
    }));

    // Accepting an unresolved row resolves it and applies its import edit.
    commands
        .execute("completion.move", &mut state, json!({"delta": 1}))
        .unwrap();
    commands
        .execute("completion.accept", &mut state, json!({}))
        .unwrap();
    assert!(wait(&mut state, |state| {
        state.documents[0]
            .text
            .text()
            .starts_with("import fake.Greeting;\n")
    }));
    let document = &state.documents[0];
    assert_eq!(
        document.text.text(),
        "import fake.Greeting;\napp.getRequiredField"
    );
    assert_eq!(
        document.selections.primary().head,
        document.text.len_chars()
    );

    // Ctrl+Q without the popup shows hover documentation at the caret.
    commands
        .execute("language.quickDoc", &mut state, json!({}))
        .unwrap();
    assert!(wait(&mut state, |state| state.docs.popup.is_some()));
    let popup = state.docs.popup.clone().unwrap();
    assert!(popup.markdown.contains("Greets *name*"));
    // Moving the caret closes it.
    state.documents[0]
        .selections
        .set_single(jetthing_text::Selection::caret(0));
    state.poll_lsp();
    assert!(state.docs.popup.is_none());
    std::fs::remove_dir_all(root).unwrap();
}
