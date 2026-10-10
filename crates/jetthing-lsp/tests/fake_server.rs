//! End-to-end tests against the scriptable `jetthing-fake-lsp` server.

use jetthing_lsp::{
    CompletionTrigger, LanguageServerProvider, LaunchConfig, LaunchContext, LspError, LspManager,
    LspService, ServerRegistry, ServerSpec, ServerState, ServerStatus,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const FAKE: &str = env!("CARGO_BIN_EXE_jetthing-fake-lsp");

struct FakeProvider {
    spec: ServerSpec,
    args: Vec<String>,
}

impl LanguageServerProvider for FakeProvider {
    fn spec(&self) -> &ServerSpec {
        &self.spec
    }
    fn launch(&self, _: &LaunchContext<'_>) -> Result<LaunchConfig, LspError> {
        Ok(LaunchConfig::new("fake", FAKE).args(self.args.clone()))
    }
}

fn registry(args: &[&str]) -> ServerRegistry {
    let mut registry = ServerRegistry::empty();
    registry.register(Arc::new(FakeProvider {
        spec: ServerSpec {
            id: "fake",
            language: "Fake",
            extensions: &[("fake", "fake")],
            file_names: &[],
            candidates: &[],
            root_markers: &["fake.project"],
            install_hint: "",
            notify_missing: true,
            initialization_options: None,
        },
        args: args.iter().map(|arg| arg.to_string()).collect(),
    }));
    registry
}

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("jetthing-fake-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("fake.project"), "").unwrap();
    std::fs::write(root.join("src/App.fake"), "app.\n").unwrap();
    root.canonicalize().unwrap()
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn service_initializes_answers_server_requests_and_completes() {
    let root = workspace("service");
    let file = root.join("src/App.fake");
    let mut service = LspService::launch(&LaunchConfig::new("fake", FAKE), &root).unwrap();
    service.sync_document(&file, "fake", "app.\n", 0).unwrap();
    assert!(wait_until(|| {
        service.poll().unwrap();
        service.state() == ServerState::Ready && !service.diagnostics().is_empty()
    }));
    assert!(service.capabilities().completion_provider.is_some());
    let id = service
        .request::<jetthing_lsp::lsp_types::request::Completion>(
            serde_json::from_value(serde_json::json!({
                "textDocument": {"uri": url::Url::from_file_path(&file).unwrap()},
                "position": {"line": 0, "character": 4}
            }))
            .unwrap(),
        )
        .unwrap();
    let mut result = None;
    assert!(wait_until(|| {
        service.poll().unwrap();
        result = service.take_typed::<jetthing_lsp::lsp_types::request::Completion>(id);
        result.is_some()
    }));
    let Some(jetthing_lsp::lsp_types::CompletionResponse::List(list)) = result.unwrap().unwrap()
    else {
        panic!("expected a completion list");
    };
    assert_eq!(list.items[0].label, "trigger0", "no context was sent");
    assert_eq!(list.items[1].label, "greeting");
    assert!(service.last_latency("textDocument/completion").is_some());
    // Unknown requests produce JSON-RPC errors, not hangs.
    let id = service
        .request_raw("custom/unknown", serde_json::Value::Null)
        .unwrap();
    assert!(wait_until(|| {
        service.poll().unwrap();
        service.take_response(id).is_some_and(|r| r.result.is_err())
    }));
    service.shutdown();
    assert_eq!(service.state(), ServerState::Exited);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manager_starts_lazily_reports_progress_and_routes_completion() {
    let root = workspace("manager");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&["--progress"]));
    manager.set_project(Some(&root));
    assert!(manager.servers().is_empty(), "servers start lazily");
    assert!(manager.handles(&file));
    assert!(!manager.handles(&root.join("notes.txt")));
    manager.sync_document(&file, "app.\n", 0).unwrap();
    assert_eq!(manager.servers().len(), 1);
    assert_eq!(manager.servers()[0].root, root);
    assert!(wait_until(|| {
        manager.poll();
        let servers = manager.servers();
        servers[0].status == ServerStatus::Ready && !servers[0].progress.is_empty()
    }));
    let progress = &manager.servers()[0].progress[0];
    assert_eq!(progress.title, "Indexing");
    assert_eq!(progress.percentage, Some(45));
    assert!(wait_until(|| {
        manager.poll();
        !manager.diagnostics_for(&file).is_empty()
    }));
    manager
        .request_completion(&file, 0, 4, CompletionTrigger::Character(".".into()))
        .unwrap();
    let mut result = None;
    assert!(wait_until(|| {
        manager.poll();
        result = manager.take_completion();
        result.is_some()
    }));
    let result = result.unwrap();
    assert_eq!(result.items.len(), 5);
    assert_eq!(
        result.items[0].label, "trigger2",
        "trigger character kind is sent"
    );
    assert!(!result.incomplete);
    assert_eq!(manager.trigger_characters(&file), ["."]);
    let ranked = manager.rank_completions(&file, &result.items, "gRF", |_| 0);
    assert_eq!(result.items[ranked[0].index].label, "getRequiredField");
    manager.close_document(&file).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manager_restarts_crashed_servers_and_resyncs_documents() {
    let root = workspace("crash");
    let marker = root.join("crashed.marker");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&["--crash-once", marker.to_str().unwrap()]));
    manager.set_project(Some(&root));
    manager.sync_document(&file, "app.\n", 0).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        matches!(
            manager.servers()[0].status,
            ServerStatus::Restarting { attempt: 1 }
        )
    }));
    let message = manager.servers()[0].message.clone().unwrap_or_default();
    assert!(message.contains("crashing on purpose"), "{message}");
    let mut resync = Vec::new();
    assert!(wait_until(|| {
        resync.extend(manager.poll());
        !resync.is_empty()
    }));
    assert_eq!(resync, std::slice::from_ref(&file));
    manager.sync_document(&file, "app.\n", 1).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.servers()[0].status == ServerStatus::Ready
    }));
    assert!(manager.ready_server_for(&file).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_completion_requests_are_cancelled() {
    let root = workspace("cancel");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&["--completion-delay-ms", "200"]));
    manager.set_project(Some(&root));
    manager.sync_document(&file, "app.\n", 0).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.ready_server_for(&file).is_ok()
    }));
    let first = manager
        .request_completion(&file, 0, 4, CompletionTrigger::Invoked)
        .unwrap();
    manager.cancel_completion();
    assert!(!manager.completion_pending());
    let service = manager.ready_server_for(&file).unwrap();
    assert!(!service.is_pending(first), "cancelled request is forgotten");
    thread::sleep(Duration::from_millis(400));
    manager.poll();
    assert!(
        manager.take_completion().is_none(),
        "late response is dropped"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_servers_produce_one_actionable_notice() {
    let root = workspace("missing");
    let file = root.join("src/Main.java");
    std::fs::write(&file, "class Main {}\n").unwrap();
    std::fs::write(root.join("settings.gradle"), "").unwrap();
    std::fs::create_dir_all(root.join(".jetthing")).unwrap();
    std::fs::write(
        root.join(".jetthing/lsp.toml"),
        "[servers.jdtls]\ncommand = \"/nonexistent/jdtls\"\n",
    )
    .unwrap();
    let mut manager = LspManager::default();
    manager.set_project(Some(&root));
    let error = manager
        .sync_document(&file, "class Main {}\n", 0)
        .unwrap_err();
    assert!(error.to_string().contains("jdtls"), "{error}");
    // A second document does not retry or duplicate the notice.
    manager.sync_document(&file, "class Main {}\n", 1).unwrap();
    assert_eq!(manager.notices().len(), 1);
    assert_eq!(manager.notices()[0].id, "missing:jdtls");
    assert!(manager.notices()[0].title.contains("Java"));
    let error = manager.ready_server_for(&file).err().unwrap();
    assert!(matches!(error, LspError::ServerNotFound { .. }));
    manager.dismiss_notice("missing:jdtls");
    assert!(manager.notices().is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn disabled_servers_are_not_started() {
    let root = workspace("disabled");
    let file = root.join("src/main.rs");
    std::fs::create_dir_all(root.join(".jetthing")).unwrap();
    std::fs::write(
        root.join(".jetthing/lsp.toml"),
        "[servers.rust-analyzer]\nenabled = false\n",
    )
    .unwrap();
    let mut manager = LspManager::default();
    manager.set_project(Some(&root));
    assert!(!manager.handles(&file));
    manager.sync_document(&file, "fn main() {}", 0).unwrap();
    assert!(manager.servers().is_empty());
    assert!(matches!(
        manager.ready_server_for(Path::new(&file)),
        Err(LspError::Disabled(_))
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn diagnostics_carry_severity_tags_and_project_files() {
    let root = workspace("diagnostics");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&["--publish", "src/Other.fake"]));
    manager.set_project(Some(&root));
    let generation = manager.diagnostics_generation();
    assert_eq!(manager.published_diagnostics(&file), None);
    let text = "ok\nlet x = 1; // @error é @unused\n@weak @hint\n";
    manager.sync_document(&file, text, 0).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.diagnostics_for(&file).len() == 4
            && manager
                .all_diagnostics()
                .any(|(path, _)| path.ends_with("Other.fake"))
    }));
    assert_ne!(manager.diagnostics_generation(), generation);
    let mut diagnostics = manager.diagnostics_for(&file);
    diagnostics.sort_by_key(|d| (d.start_line, d.start_character));
    let error = &diagnostics[0];
    assert_eq!((error.severity, error.start_line), (Some(1), 1));
    assert_eq!(error.start_character, 14);
    assert_eq!(error.code.as_deref(), Some("F1"));
    assert_eq!(error.source.as_deref(), Some("fake"));
    let unused = &diagnostics[1];
    // `é` is one UTF-16 unit, so the marker starts at 23.
    assert_eq!((unused.start_character, unused.unnecessary), (23, true));
    assert_eq!(diagnostics[2].severity, Some(3));
    assert_eq!(diagnostics[3].severity, Some(4));
    let (other, published) = manager
        .all_diagnostics()
        .find(|(path, _)| path.ends_with("Other.fake"))
        .unwrap();
    assert_eq!(other, root.join("src/Other.fake"));
    assert_eq!(published[0].message, "fake project error");
    // Fixing the text clears the markers and bumps the generation.
    let generation = manager.diagnostics_generation();
    manager.sync_document(&file, "ok\n", 1).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.diagnostics_for(&file).len() == 1
    }));
    assert!(manager.diagnostics_generation() > generation);
    // Saving triggers the server's on-save check.
    manager.save_document(&file).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.diagnostics_for(&file)[0].message == "fake check warning"
    }));
    manager.shutdown();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn navigation_requests_return_locations() {
    use jetthing_lsp::{Feature, parse_locations};
    let root = workspace("navigation");
    let app = root.join("src/App.fake");
    let other = root.join("src/Other.fake");
    let mut manager = LspManager::new(registry(&[]));
    manager.set_project(Some(&root));
    manager
        .sync_document(&app, "fn greet() {}\ngreet();\n", 0)
        .unwrap();
    std::fs::write(&other, "greet();\n").unwrap();
    manager.sync_document(&other, "greet();\n", 0).unwrap();
    let wait = |manager: &mut LspManager, feature, line, character| {
        let handle = loop {
            manager.poll();
            match manager.request(&app, feature, line, character, serde_json::json!({})) {
                Ok(handle) => break handle,
                Err(LspError::Starting { .. }) => thread::sleep(Duration::from_millis(10)),
                Err(error) => panic!("{error}"),
            }
        };
        let mut result = None;
        assert!(wait_until(|| {
            manager.poll();
            result = manager.take_response(&handle);
            result.is_some()
        }));
        parse_locations(&result.unwrap().unwrap())
    };
    let definition = wait(&mut manager, Feature::Definition, 1, 2);
    assert_eq!(definition.len(), 1);
    assert_eq!(
        (definition[0].path.clone(), definition[0].start_line),
        (app.clone(), 0)
    );
    assert_eq!(definition[0].start_character, 3);
    let type_definition = wait(&mut manager, Feature::TypeDefinition, 1, 0);
    assert_eq!(
        type_definition, definition,
        "LocationLink resolves to the same place"
    );
    let implementations = wait(&mut manager, Feature::Implementation, 1, 0);
    assert_eq!(implementations.len(), 2);
    let references = wait(&mut manager, Feature::References, 0, 4);
    assert_eq!(references.len(), 3);
    assert!(references.iter().any(|location| location.path == other));
    // Nothing under the caret.
    assert!(wait(&mut manager, Feature::Definition, 0, 12).is_empty());
    manager.shutdown();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn code_actions_resolve_and_server_applied_edits() {
    use jetthing_lsp::Feature;
    use serde_json::json;
    let root = workspace("actions");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&[]));
    manager.set_project(Some(&root));
    manager.sync_document(&file, "x = @error\n", 0).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.diagnostics_for(&file).len() == 1
    }));
    assert!(manager.supports(&file, Feature::CodeAction));
    assert!(manager.supports(&file, Feature::CodeActionResolve));
    let raw = manager.diagnostics_for(&file)[0].raw.clone();
    assert_eq!(raw["message"], "marker @error");
    let uri = format!("file://{}", file.display());
    let call = |manager: &mut LspManager, feature, params| {
        let handle = manager.request_with_params(&file, feature, params).unwrap();
        let mut result = None;
        assert!(wait_until(|| {
            manager.poll();
            result = manager.take_response(&handle);
            result.is_some()
        }));
        result.unwrap().unwrap()
    };
    let actions = call(
        &mut manager,
        Feature::CodeAction,
        json!({"textDocument": {"uri": uri}, "range": raw["range"],
            "context": {"diagnostics": [raw], "triggerKind": 1}}),
    );
    let actions = actions.as_array().unwrap().clone();
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0]["kind"], "quickfix");
    assert_eq!(actions[0]["edit"]["changes"][&uri][0]["newText"], "fixed");
    let resolved = call(&mut manager, Feature::CodeActionResolve, actions[1].clone());
    assert_eq!(
        resolved["edit"]["documentChanges"][0]["edits"][0]["newText"],
        "// resolved\n"
    );
    call(
        &mut manager,
        Feature::ExecuteCommand,
        json!({"command": "fake.apply", "arguments": [uri]}),
    );
    let mut requests = Vec::new();
    assert!(wait_until(|| {
        manager.poll();
        requests.extend(manager.take_apply_edits());
        !requests.is_empty()
    }));
    let (server, request) = requests.remove(0);
    assert_eq!(request.label.as_deref(), Some("Fake command"));
    assert_eq!(request.edit["changes"][&uri][0]["newText"], "// applied\n");
    manager.respond_apply_edit(&server, request.id, Ok(()));
    manager.shutdown();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn typed_requests_resolve_hover_and_signature_help() {
    use jetthing_lsp::lsp_types::{
        CompletionItem, Documentation, HoverContents, HoverParams, Position, SignatureHelpParams,
        TextDocumentIdentifier, TextDocumentPositionParams,
        request::{HoverRequest, ResolveCompletionItem, SignatureHelpRequest},
    };
    let root = workspace("typed");
    let file = root.join("src/App.fake");
    let mut manager = LspManager::new(registry(&[]));
    manager.set_project(Some(&root));
    manager.sync_document(&file, "app.\n", 0).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        manager.ready_server_for(&file).is_ok()
    }));
    let position = TextDocumentPositionParams {
        text_document: TextDocumentIdentifier {
            uri: url::Url::from_file_path(&file).unwrap(),
        },
        position: Position::new(0, 2),
    };
    let item = CompletionItem {
        label: "greeting".into(),
        ..Default::default()
    };
    let resolve = manager
        .send_request::<ResolveCompletionItem>(&file, item)
        .unwrap();
    let hover = manager
        .send_request::<HoverRequest>(
            &file,
            HoverParams {
                text_document_position_params: position.clone(),
                work_done_progress_params: Default::default(),
            },
        )
        .unwrap();
    let signature = manager
        .send_request::<SignatureHelpRequest>(
            &file,
            SignatureHelpParams {
                context: None,
                text_document_position_params: position,
                work_done_progress_params: Default::default(),
            },
        )
        .unwrap();
    let (mut resolved, mut hovered, mut signed) = (None, None, None);
    assert!(wait_until(|| {
        manager.poll();
        resolved = resolved
            .take()
            .or_else(|| manager.take_typed_response::<ResolveCompletionItem>(&resolve));
        hovered = hovered
            .take()
            .or_else(|| manager.take_typed_response::<HoverRequest>(&hover));
        signed = signed
            .take()
            .or_else(|| manager.take_typed_response::<SignatureHelpRequest>(&signature));
        resolved.is_some() && hovered.is_some() && signed.is_some()
    }));
    let resolved = resolved.unwrap().unwrap();
    assert!(matches!(
        resolved.documentation,
        Some(Documentation::MarkupContent(_))
    ));
    assert_eq!(resolved.additional_text_edits.unwrap().len(), 1);
    let hovered = hovered.unwrap().unwrap().unwrap();
    assert!(matches!(hovered.contents, HoverContents::Markup(_)));
    let signed = signed.unwrap().unwrap().unwrap();
    assert_eq!(signed.active_parameter, Some(1));
    assert!(!manager.is_pending(&resolve));
    std::fs::remove_dir_all(root).unwrap();
}
