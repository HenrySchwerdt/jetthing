//! Scriptable fake language server used by Jetthing's integration tests.
//!
//! Flags:
//! - `--progress`: create a work-done progress token and report indexing.
//! - `--crash-once <file>`: exit with code 3 after `initialized` unless `<file>`
//!   exists (the file is created first, so the restarted server survives).
//! - `--completion-delay-ms <n>`: answer completion requests after a delay.
//!
//! Completion returns a fixed Java-like member list; `textDocument/didOpen`
//! and `didChange` publish one diagnostic per document.

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
                    "textDocumentSync": 1,
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
            }
            ("textDocument/didOpen" | "textDocument/didChange", None) => {
                let uri = params["textDocument"]["uri"].clone();
                send(
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                        "uri": uri, "diagnostics": [{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                        "severity":2,"message":format!("fake diagnostic ({method})")}]
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
