//! End-to-end tests against the scriptable `jetthing-fake-lsp` server.

use jetthing_lsp::{
    LanguageServerProvider, LaunchConfig, LaunchContext, LspError, LspManager, LspService,
    ServerRegistry, ServerSpec, ServerState, ServerStatus,
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
    assert_eq!(list.items[0].label, "greeting");
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
    manager.request_completion(&file, 0, 4).unwrap();
    assert!(wait_until(|| {
        manager.poll();
        !manager.completions().is_empty()
    }));
    assert_eq!(manager.completions().len(), 4);
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
    let first = manager.request_completion(&file, 0, 4).unwrap();
    manager.clear_completions();
    assert!(!manager.completion_pending());
    let service = manager.ready_server_for(&file).unwrap();
    assert!(!service.is_pending(first), "cancelled request is forgotten");
    thread::sleep(Duration::from_millis(400));
    manager.poll();
    assert!(manager.completions().is_empty(), "late response is dropped");
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
