//! Scriptable fake language server used by Jetthing's integration tests.
//!
//! Flags:
//! - `--progress`: create a work-done progress token and report indexing.
//! - `--crash-once <file>`: exit with code 3 after `initialized` unless `<file>`
//!   exists (the file is created first, so the restarted server survives).
//! - `--completion-delay-ms <n>`: answer completion requests after a delay.
//! - `--publish <relative path>`: after `initialized`, publish one error for a
//!   file below the workspace root that was never opened.
//!
//! Completion returns a fixed Java-like member list. `textDocument/didOpen`
//! and `didChange` publish one diagnostic per `@error`, `@warning`, `@weak`,
//! `@hint` and `@unused` marker in the text, or one warning at 0:0 when the
//! text has no markers.

use jetthing_lsp::transport::{read_message, write_message};
use serde_json::{Value, json};
use std::io::{BufReader, stdin, stdout};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|arg| arg == name);
    let value = |name: &str| {
        args.iter()
            .position(|arg| arg == name)
            .and_then(|index| args.get(index + 1))
            .cloned()
    };
    let progress = flag("--progress");
    let crash_file = value("--crash-once");
    let delay = value("--completion-delay-ms")
        .and_then(|ms| ms.parse().ok())
        .map(Duration::from_millis);
    let publish = value("--publish");
    let mut root = String::new();
    let mut input = BufReader::new(stdin().lock());
    let mut output = stdout().lock();
    let mut send = |message: Value| {
        if write_message(&mut output, &message).is_err() {
            std::process::exit(1);
        }
    };
    loop {
        let Ok(message) = read_message(&mut input) else {
            std::process::exit(0);
        };
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        match (method, id) {
            ("initialize", Some(id)) => {
                root = params["rootUri"].as_str().unwrap_or_default().to_owned();
                if progress {
                    send(
                        json!({"jsonrpc":"2.0","id":"p1","method":"window/workDoneProgress/create","params":{"token":"index"}}),
                    );
                    send(
                        json!({"jsonrpc":"2.0","method":"$/progress","params":{"token":"index","value":{"kind":"begin","title":"Indexing","percentage":0}}}),
                    );
                    send(
                        json!({"jsonrpc":"2.0","method":"$/progress","params":{"token":"index","value":{"kind":"report","message":"fake","percentage":45}}}),
                    );
                }
                send(json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{
                    "textDocumentSync": {"openClose": true, "change": 1, "save": {"includeText": false}},
                    "completionProvider": {"triggerCharacters": ["."], "resolveProvider": true},
                    "hoverProvider": true,
                    "signatureHelpProvider": {"triggerCharacters": ["(", ","]}
                },"serverInfo":{"name":"fake"}}}));
            }
            ("initialized", None) => {
                if let Some(file) = &crash_file
                    && !std::path::Path::new(file).exists()
                {
                    let _ = std::fs::write(file, "crashed");
                    eprintln!("fake server crashing on purpose");
                    std::process::exit(3);
                }
                send(
                    json!({"jsonrpc":"2.0","method":"workspace/configuration","id":"c1","params":{"items":[{"section":"fake"}]}}),
                );
                if let Some(relative) = &publish {
                    let uri = format!("{}/{relative}", root.trim_end_matches('/'));
                    send(
                        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                            "uri": uri, "diagnostics": [{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},
                            "severity":1,"source":"fake","message":"fake project error"}]
                        }}),
                    );
                }
            }
            ("textDocument/didOpen" | "textDocument/didChange", None) => {
                let uri = params["textDocument"]["uri"].clone();
                let text = params["textDocument"]["text"]
                    .as_str()
                    .or_else(|| params["contentChanges"][0]["text"].as_str())
                    .unwrap_or_default();
                let mut diagnostics = marker_diagnostics(text);
                if diagnostics.is_empty() {
                    diagnostics.push(json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                        "severity":2,"message":format!("fake diagnostic ({method})")}));
                }
                send(
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                        "uri": uri, "diagnostics": diagnostics
                    }}),
                );
            }
            ("textDocument/didSave", None) => {
                let uri = params["textDocument"]["uri"].clone();
                send(
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                        "uri": uri, "diagnostics": [{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                        "severity":2,"source":"check","message":"fake check warning"}]
                    }}),
                );
            }
            ("textDocument/completion", Some(id)) => {
                if let Some(delay) = delay {
                    std::thread::sleep(delay);
                }
                send(
                    json!({"jsonrpc":"2.0","id":id,"result":{"isIncomplete":false,"items":[
                        {"label":"greeting","kind":2,"detail":"String","labelDetails":{"detail":"(String name)","description":"String"},"sortText":"0"},
                        {"label":"getRequiredField","kind":2,"detail":"Field","sortText":"1"},
                        {"label":"getClass","kind":2,"detail":"Class<?>","sortText":"2"},
                        {"label":"hashCode","kind":2,"detail":"int","sortText":"3"}
                    ]}}),
                );
            }
            ("shutdown", Some(id)) => send(json!({"jsonrpc":"2.0","id":id,"result":null})),
            ("exit", None) => std::process::exit(0),
            ("$/cancelRequest", None) => {}
            (_, Some(id)) if !method.is_empty() => send(json!({"jsonrpc":"2.0","id":id,
                "error":{"code":-32601,"message":format!("unhandled {method}")}})),
            _ => {}
        }
    }
}

/// One diagnostic per marker; positions are UTF-16 like a real server's.
fn marker_diagnostics(text: &str) -> Vec<Value> {
    let markers = [
        ("@error", 1, None),
        ("@warning", 2, None),
        ("@weak", 3, None),
        ("@hint", 4, None),
        ("@unused", 2, Some(1)),
    ];
    let mut diagnostics = Vec::new();
    for (line, content) in text.lines().enumerate() {
        for (marker, severity, tag) in markers {
            let mut from = 0;
            while let Some(found) = content[from..].find(marker) {
                let byte = from + found;
                let start: usize = content[..byte].chars().map(char::len_utf16).sum();
                let end = start + marker.len();
                let mut diagnostic = json!({
                    "range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}},
                    "severity": severity,
                    "source": "fake",
                    "code": format!("F{severity}"),
                    "message": format!("marker {marker}")
                });
                if let Some(tag) = tag {
                    diagnostic["tags"] = json!([tag]);
                }
                diagnostics.push(diagnostic);
                from = byte + marker.len();
            }
        }
    }
    diagnostics
}
