//! Integration checks against installed language servers. Ignored by default;
//! run with `cargo test -p jetthing-lsp --test real_servers -- --ignored`.

use jetthing_lsp::LspManager;
use std::{
    thread,
    time::{Duration, Instant},
};

fn installed(program: &str) -> bool {
    jetthing_lsp::registry::resolve_program(program).is_some()
}

#[test]
#[ignore = "requires an installed rust-analyzer"]
fn rust_analyzer_publishes_diagnostics_and_completes() {
    if !installed("rust-analyzer") {
        return;
    }
    let root = std::env::temp_dir().join(format!("jetthing-lsp-real-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='jetthing-lsp-smoke'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    let source =
        "pub fn broken() { let value: u32 = \"not a number\"; let s = String::new(); s.\n}\n";
    let file = root.join("src/lib.rs");
    std::fs::write(&file, source).unwrap();
    let file = file.canonicalize().unwrap();
    let mut manager = LspManager::default();
    manager.set_project(Some(&root));
    manager.sync_document(&file, source, 0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while manager.diagnostics_for(&file).is_empty() && Instant::now() < deadline {
        manager.poll();
        thread::sleep(Duration::from_millis(25));
    }
    assert!(!manager.diagnostics_for(&file).is_empty());
    // rust-analyzer answers with an empty list until the workspace is loaded.
    while manager.completions().is_empty() && Instant::now() < deadline {
        manager.request_completion(&file, 0, 76).unwrap();
        while manager.completion_pending() && Instant::now() < deadline {
            manager.poll();
            thread::sleep(Duration::from_millis(25));
        }
    }
    let labels: Vec<_> = manager
        .completions()
        .iter()
        .map(|i| i.label.as_str())
        .collect();
    assert!(
        labels.iter().any(|label| label.starts_with("push_str")),
        "{labels:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires an installed jdtls and JDK 21+"]
fn jdtls_completes_java_members() {
    if !installed("jdtls") {
        return;
    }
    let root = std::env::temp_dir().join(format!("jetthing-jdtls-real-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    let source = "public class App {\n    public String greeting(String name) { return \"Hello, \" + name; }\n    public static void main(String[] args) {\n        App app = new App();\n        app.\n    }\n}\n";
    let file = root.join("src/App.java");
    std::fs::write(&file, source).unwrap();
    let file = file.canonicalize().unwrap();
    let mut manager = LspManager::default();
    manager.set_project(Some(&root));
    manager.sync_document(&file, source, 0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(180);
    while manager.completions().is_empty() && Instant::now() < deadline {
        manager.poll();
        if manager.ready_server_for(&file).is_ok() {
            manager.request_completion(&file, 4, 12).unwrap();
            while manager.completion_pending() && Instant::now() < deadline {
                manager.poll();
                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    let labels: Vec<_> = manager
        .completions()
        .iter()
        .map(|i| i.label.as_str())
        .collect();
    assert!(
        labels.iter().any(|label| label.starts_with("greeting")),
        "{labels:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
