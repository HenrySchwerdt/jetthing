use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum LspError {
    #[error("failed to start language server: {0}")]
    Start(#[from] std::io::Error),
    #[error("language server stream is unavailable")]
    MissingStream,
    #[error("language server writer lock is unavailable")]
    Lock,
    #[error("invalid file path for LSP URI: {0}")]
    InvalidPath(PathBuf),
    #[error("language server protocol error: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Starting,
    Ready,
    Exited,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspDiagnostic {
    pub message: String,
    pub severity: Option<u8>,
    pub start_line: usize,
    pub start_character: usize,
    pub end_line: usize,
    pub end_character: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

struct PendingDocument {
    path: PathBuf,
    text: String,
    version: i32,
}

pub struct LspService {
    child: Child,
    writer: Arc<Mutex<ChildStdin>>,
    messages: Receiver<Result<Value, LspError>>,
    state: ServerState,
    next_id: u64,
    initialize_id: u64,
    pending_documents: Vec<PendingDocument>,
    diagnostics: HashMap<PathBuf, Vec<LspDiagnostic>>,
    completions: Vec<CompletionItem>,
    pending_completion: Option<u64>,
}

impl LspService {
    pub fn start_rust_analyzer(root: &Path) -> Result<Self, LspError> {
        Self::start("rust-analyzer", &[], root)
    }

    pub fn start(program: &str, args: &[&str], root: &Path) -> Result<Self, LspError> {
        let mut child = Command::new(program)
            .args(args)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().ok_or(LspError::MissingStream)?;
        let stdout = child.stdout.take().ok_or(LspError::MissingStream)?;
        let writer = Arc::new(Mutex::new(stdin));
        let (sender, messages) = mpsc::channel();
        let reader_thread = thread::Builder::new()
            .name("jetthing-lsp-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let message = read_message(&mut reader);
                    let terminal = message.is_err();
                    if sender.send(message).is_err() || terminal {
                        break;
                    }
                }
            });
        if let Err(error) = reader_thread {
            let _ = child.kill();
            return Err(error.into());
        }
        let root_uri = Url::from_directory_path(root.canonicalize()?)
            .map_err(|_| LspError::InvalidPath(root.to_owned()))?;
        let mut service = Self {
            child,
            writer,
            messages,
            state: ServerState::Starting,
            next_id: 2,
            initialize_id: 1,
            pending_documents: Vec::new(),
            diagnostics: HashMap::new(),
            completions: Vec::new(),
            pending_completion: None,
        };
        service.send(json!({
            "jsonrpc": "2.0", "id": service.initialize_id, "method": "initialize",
            "params": {
                "processId": std::process::id(), "rootUri": root_uri,
                "capabilities": {
                    "general": {"positionEncodings": ["utf-16"]},
                    "textDocument": {"publishDiagnostics": {"relatedInformation": true}, "completion": {"completionItem": {"snippetSupport": false}}}
                },
                "workspaceFolders": [{"uri": root_uri, "name": root.file_name().and_then(|n| n.to_str()).unwrap_or("workspace")}]
            }
        }))?;
        Ok(service)
    }

    pub fn poll(&mut self) -> Result<(), LspError> {
        while let Ok(message) = self.messages.try_recv() {
            self.handle(message?)?;
        }
        if self.child.try_wait()?.is_some() {
            self.state = ServerState::Exited;
        }
        Ok(())
    }

    pub fn open_document(&mut self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        if self.state != ServerState::Ready {
            self.pending_documents.push(PendingDocument {
                path: path.to_owned(),
                text: text.into(),
                version,
            });
            return Ok(());
        }
        self.send_open(path, text, version)
    }

    pub fn change_document(
        &mut self,
        path: &Path,
        text: &str,
        version: i32,
    ) -> Result<(), LspError> {
        if self.state != ServerState::Ready {
            return self.open_document(path, text, version);
        }
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
            "textDocument":{"uri":file_uri(path)?,"version":version},"contentChanges":[{"text":text}]
        }}))
    }

    pub fn request_completion(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
    ) -> Result<u64, LspError> {
        let id = self.next_id;
        self.next_id += 1;
        self.completions.clear();
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/completion","params":{
            "textDocument":{"uri":file_uri(path)?},"position":{"line":line,"character":character}
        }}))?;
        self.pending_completion = Some(id);
        Ok(id)
    }

    pub fn state(&self) -> ServerState {
        self.state
    }
    pub fn diagnostics(&self) -> &HashMap<PathBuf, Vec<LspDiagnostic>> {
        &self.diagnostics
    }
    pub fn completions(&self) -> &[CompletionItem] {
        &self.completions
    }
    pub fn clear_completions(&mut self) {
        self.completions.clear();
        self.pending_completion = None;
    }

    fn handle(&mut self, message: Value) -> Result<(), LspError> {
        if message.get("id").and_then(Value::as_u64) == Some(self.initialize_id) {
            self.state = ServerState::Ready;
            self.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}))?;
            let pending = std::mem::take(&mut self.pending_documents);
            for document in pending {
                self.send_open(&document.path, &document.text, document.version)?;
            }
            return Ok(());
        }
        if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        {
            let params = message
                .get("params")
                .ok_or_else(|| LspError::Protocol("diagnostics missing params".into()))?;
            let uri = params
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| LspError::Protocol("diagnostics missing URI".into()))?;
            let path = Url::parse(uri)
                .map_err(|e| LspError::Protocol(e.to_string()))?
                .to_file_path()
                .map_err(|_| LspError::Protocol("diagnostic URI is not a file".into()))?;
            let diagnostics = params
                .get("diagnostics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(parse_diagnostic)
                .collect();
            self.diagnostics.insert(path, diagnostics);
        } else if message.get("id").and_then(Value::as_u64) == self.pending_completion
            && message.get("result").is_some()
        {
            let result = message.get("result").cloned().unwrap_or(Value::Null);
            let items = if result.is_array() {
                result
            } else {
                result.get("items").cloned().unwrap_or_else(|| json!([]))
            };
            if let Ok(completions) = serde_json::from_value(items) {
                self.completions = completions;
            }
            self.pending_completion = None;
        }
        Ok(())
    }

    fn send_open(&mut self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        self.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{
                "uri":file_uri(path)?,"languageId":"rust","version":version,"text":text
            }}}),
        )
    }

    fn send(&mut self, message: Value) -> Result<(), LspError> {
        let bytes = serde_json::to_vec(&message).map_err(|e| LspError::Protocol(e.to_string()))?;
        let mut writer = self.writer.lock().map_err(|_| LspError::Lock)?;
        write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(())
    }
}

fn parse_diagnostic(value: &Value) -> Option<LspDiagnostic> {
    let range = value.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    Some(LspDiagnostic {
        message: value.get("message")?.as_str()?.into(),
        severity: value
            .get("severity")
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok()),
        start_line: start.get("line")?.as_u64()? as usize,
        start_character: start.get("character")?.as_u64()? as usize,
        end_line: end.get("line")?.as_u64()? as usize,
        end_character: end.get("character")?.as_u64()? as usize,
    })
}

impl Drop for LspService {
    fn drop(&mut self) {
        let _ = self.send(json!({"jsonrpc":"2.0","method":"exit","params":{}}));
        let _ = self.child.kill();
    }
}

fn file_uri(path: &Path) -> Result<Url, LspError> {
    Url::from_file_path(path.canonicalize()?).map_err(|_| LspError::InvalidPath(path.to_owned()))
}

fn read_message(reader: &mut impl BufRead) -> Result<Value, LspError> {
    let mut content_length = None;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            return Err(LspError::Protocol("language server closed output".into()));
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| LspError::Protocol(e.to_string()))?,
            );
        }
    }
    let length =
        content_length.ok_or_else(|| LspError::Protocol("missing Content-Length".into()))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|e| LspError::Protocol(e.to_string()))
}

impl From<serde_json::Error> for LspError {
    fn from(error: serde_json::Error) -> Self {
        Self::Protocol(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_framed_json_rpc() {
        let body = br#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}",
            body.len(),
            String::from_utf8_lossy(body)
        );
        let value = read_message(&mut BufReader::new(framed.as_bytes())).unwrap();
        assert_eq!(value["id"], 1);
    }

    #[cfg(unix)]
    #[test]
    fn initializes_against_a_real_framed_fake_server() {
        let response = r#"{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}"#;
        let completion = r#"{"jsonrpc":"2.0","id":2,"result":[{"label":"hello","detail":"fake completion","insertText":"hello"}]}"#;
        let script = format!(
            "sleep 0.05; printf 'Content-Length: {}\\r\\n\\r\\n%s' '{}'; sleep 0.1; printf 'Content-Length: {}\\r\\n\\r\\n%s' '{}'; sleep 1",
            response.len(),
            response,
            completion.len(),
            completion,
        );
        let source_path = Path::new("/tmp/jetthing-lsp-fake.rs");
        std::fs::write(source_path, "fn main() {}\n").unwrap();
        let mut service =
            LspService::start("/bin/sh", &["-c", &script], Path::new("/tmp")).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while service.state() == ServerState::Starting && std::time::Instant::now() < deadline {
            service.poll().unwrap();
            thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(service.state(), ServerState::Ready);
        service.request_completion(source_path, 0, 0).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while service.completions().is_empty() && std::time::Instant::now() < deadline {
            service.poll().unwrap();
            thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(service.completions()[0].label, "hello");
        std::fs::remove_file(source_path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires an installed rust-analyzer; run explicitly for integration verification"]
    fn real_rust_analyzer_publishes_semantic_diagnostics() {
        if !Command::new("rust-analyzer")
            .arg("--version")
            .status()
            .is_ok_and(|status| status.success())
        {
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
        let source = "pub fn broken() { let value: u32 = \"not a number\"; }\n";
        let source_path = root.join("src/lib.rs");
        std::fs::write(&source_path, source).unwrap();
        let canonical_source = source_path.canonicalize().unwrap();
        let mut service = LspService::start_rust_analyzer(&root).unwrap();
        service.open_document(&source_path, source, 0).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while std::time::Instant::now() < deadline {
            service.poll().unwrap();
            if service
                .diagnostics()
                .get(&canonical_source)
                .is_some_and(|items| !items.is_empty())
            {
                std::fs::remove_dir_all(root).unwrap();
                return;
            }
            thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!(
            "rust-analyzer did not publish diagnostics: {:?}",
            service.diagnostics()
        );
    }
}
