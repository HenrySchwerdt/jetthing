//! A single language server process: lifecycle, document synchronisation,
//! request/response plumbing, progress and published diagnostics.
//!
//! The client never blocks the caller: messages are read on a background
//! thread and consumed by [`LspService::poll`], which the UI calls once per
//! frame. Requests return an id immediately; their results are picked up
//! later with [`LspService::take_response`] or cancelled with
//! [`LspService::cancel`].

use crate::{LspError, transport};
use lsp_types::{
    ClientCapabilities, ClientInfo, CompletionClientCapabilities, CompletionItemCapability,
    CompletionItemCapabilityResolveSupport, CompletionItemKind, CompletionItemKindCapability,
    GeneralClientCapabilities, HoverClientCapabilities, InitializeParams, InitializeResult,
    MarkupKind, ParameterInformationSettings, PositionEncodingKind,
    PublishDiagnosticsClientCapabilities, ServerCapabilities, SignatureHelpClientCapabilities,
    SignatureInformationSettings, TextDocumentClientCapabilities,
    TextDocumentSyncClientCapabilities, WindowClientCapabilities, WorkspaceFolder,
    request::Request,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};
use url::Url;

/// How to launch a language server process.
#[derive(Debug, Clone, PartialEq)]
pub struct LaunchConfig {
    /// Display name used in the status bar and in error messages.
    pub name: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub initialization_options: Option<Value>,
}

impl LaunchConfig {
    pub fn new(name: impl Into<String>, program: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            initialization_options: None,
        }
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }
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

/// A `$/progress` work-done report that has begun and not yet ended.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct WorkProgress {
    pub title: String,
    pub message: Option<String>,
    pub percentage: Option<u32>,
}

/// JSON-RPC error object returned by a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
}

/// A response to a request sent with [`LspService::request`].
#[derive(Debug, Clone, PartialEq)]
pub struct LspResponse {
    pub method: String,
    pub result: Result<Value, ResponseError>,
    pub latency: Duration,
}

/// A message the server asked the client to show (`window/showMessage`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMessage {
    /// LSP `MessageType`: 1 error, 2 warning, 3 info, 4 log.
    pub kind: u8,
    pub text: String,
}

struct PendingRequest {
    method: String,
    sent: Instant,
}

struct QueuedDocument {
    language_id: String,
    text: String,
    version: i32,
}

/// The JSON-RPC id of the `initialize` request.
const INITIALIZE_ID: u64 = 1;
/// Lines of stderr kept for crash reports.
const STDERR_LINES: usize = 40;
/// Responses nobody collected are dropped beyond this many.
const MAX_UNCLAIMED_RESPONSES: usize = 256;

pub struct LspService {
    name: String,
    root: PathBuf,
    child: Child,
    writer: Arc<Mutex<ChildStdin>>,
    messages: Receiver<Result<Value, LspError>>,
    stderr: Arc<Mutex<VecDeque<String>>>,
    state: ServerState,
    next_id: u64,
    capabilities: ServerCapabilities,
    queued_documents: BTreeMap<PathBuf, QueuedDocument>,
    open_documents: HashMap<PathBuf, i32>,
    diagnostics: HashMap<PathBuf, Vec<LspDiagnostic>>,
    progress: BTreeMap<String, WorkProgress>,
    status_message: Option<String>,
    shown_messages: Vec<ServerMessage>,
    pending: HashMap<u64, PendingRequest>,
    responses: HashMap<u64, LspResponse>,
    latencies: HashMap<String, Duration>,
    exit_reason: Option<String>,
}

impl LspService {
    /// Starts rust-analyzer from `PATH` (kept for callers that do not use the registry).
    pub fn start_rust_analyzer(root: &Path) -> Result<Self, LspError> {
        Self::launch(&LaunchConfig::new("rust-analyzer", "rust-analyzer"), root)
    }

    /// Starts an arbitrary program as a language server.
    pub fn start(program: &str, args: &[&str], root: &Path) -> Result<Self, LspError> {
        Self::launch(
            &LaunchConfig::new(program, program).args(args.iter().copied()),
            root,
        )
    }

    pub fn launch(config: &LaunchConfig, root: &Path) -> Result<Self, LspError> {
        let root = root.canonicalize()?;
        let mut child = Command::new(&config.program)
            .args(&config.args)
            .envs(config.env.iter().map(|(k, v)| (k, v)))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().ok_or(LspError::MissingStream)?;
        let stdout = child.stdout.take().ok_or(LspError::MissingStream)?;
        let stderr_pipe = child.stderr.take().ok_or(LspError::MissingStream)?;
        let (sender, messages) = mpsc::channel();
        let reader_thread = thread::Builder::new()
            .name(format!("lsp-reader-{}", config.name))
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let message = transport::read_message(&mut reader);
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
        let stderr = Arc::new(Mutex::new(VecDeque::new()));
        let tail = Arc::clone(&stderr);
        let _ = thread::Builder::new()
            .name(format!("lsp-stderr-{}", config.name))
            .spawn(move || {
                for line in BufReader::new(stderr_pipe).lines() {
                    let Ok(line) = line else { break };
                    if let Ok(mut tail) = tail.lock() {
                        if tail.len() == STDERR_LINES {
                            tail.pop_front();
                        }
                        tail.push_back(line);
                    }
                }
            });
        let mut service = Self {
            name: config.name.clone(),
            root,
            child,
            writer: Arc::new(Mutex::new(stdin)),
            messages,
            stderr,
            state: ServerState::Starting,
            next_id: INITIALIZE_ID + 1,
            capabilities: ServerCapabilities::default(),
            queued_documents: BTreeMap::new(),
            open_documents: HashMap::new(),
            diagnostics: HashMap::new(),
            progress: BTreeMap::new(),
            status_message: None,
            shown_messages: Vec::new(),
            pending: HashMap::new(),
            responses: HashMap::new(),
            latencies: HashMap::new(),
            exit_reason: None,
        };
        let params = initialize_params(&service.root, config.initialization_options.clone())?;
        service.pending.insert(
            INITIALIZE_ID,
            PendingRequest {
                method: "initialize".into(),
                sent: Instant::now(),
            },
        );
        service.send(&json!({
            "jsonrpc": "2.0", "id": INITIALIZE_ID, "method": "initialize", "params": params,
        }))?;
        Ok(service)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn state(&self) -> ServerState {
        self.state
    }
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.capabilities
    }
    pub fn diagnostics(&self) -> &HashMap<PathBuf, Vec<LspDiagnostic>> {
        &self.diagnostics
    }
    /// Work-done progress currently running on the server, by token.
    pub fn progress(&self) -> &BTreeMap<String, WorkProgress> {
        &self.progress
    }
    /// Latest server status line (`language/status` from jdtls, for example).
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }
    /// Messages from `window/showMessage` since the last call.
    pub fn take_shown_messages(&mut self) -> Vec<ServerMessage> {
        std::mem::take(&mut self.shown_messages)
    }
    /// Why the server exited, including the tail of its stderr.
    pub fn exit_reason(&self) -> Option<&str> {
        self.exit_reason.as_deref()
    }
    /// Round-trip time of the last completed request for `method`.
    pub fn last_latency(&self, method: &str) -> Option<Duration> {
        self.latencies.get(method).copied()
    }
    pub fn is_open(&self, path: &Path) -> bool {
        self.open_documents.contains_key(path) || self.queued_documents.contains_key(path)
    }

    /// Reads all available messages. Never blocks.
    pub fn poll(&mut self) -> Result<(), LspError> {
        while let Ok(message) = self.messages.try_recv() {
            match message {
                Ok(message) => self.handle(message)?,
                Err(error) => {
                    self.exit_reason.get_or_insert_with(|| error.to_string());
                    break;
                }
            }
        }
        if self.state != ServerState::Exited
            && let Some(status) = self.child.try_wait()?
        {
            self.state = ServerState::Exited;
            self.progress.clear();
            let tail = self
                .stderr
                .lock()
                .map(|tail| tail.iter().cloned().collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            let mut reason = format!("{} exited ({status})", self.name);
            if !tail.is_empty() {
                reason.push_str(":\n");
                reason.push_str(&tail);
            }
            self.exit_reason = Some(reason);
        }
        Ok(())
    }

    /// Opens a document, or sends its full new text if it is already open.
    pub fn sync_document(
        &mut self,
        path: &Path,
        language_id: &str,
        text: &str,
        version: i32,
    ) -> Result<(), LspError> {
        if self.state != ServerState::Ready {
            self.queued_documents.insert(
                path.to_owned(),
                QueuedDocument {
                    language_id: language_id.into(),
                    text: text.into(),
                    version,
                },
            );
            return Ok(());
        }
        match self.open_documents.get(path) {
            Some(&open) if open == version => Ok(()),
            Some(_) => {
                self.open_documents.insert(path.to_owned(), version);
                self.send(
                    &json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                        "textDocument":{"uri":file_uri(path)?,"version":version},
                        "contentChanges":[{"text":text}]
                    }}),
                )
            }
            None => self.send_open(path, language_id, text, version),
        }
    }

    /// Opens a Rust document (compatibility shim for single-language callers).
    pub fn open_document(&mut self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        self.sync_document(path, "rust", text, version)
    }

    /// Sends a Rust document change (compatibility shim for single-language callers).
    pub fn change_document(
        &mut self,
        path: &Path,
        text: &str,
        version: i32,
    ) -> Result<(), LspError> {
        self.sync_document(path, "rust", text, version)
    }

    pub fn close_document(&mut self, path: &Path) -> Result<(), LspError> {
        self.queued_documents.remove(path);
        if self.open_documents.remove(path).is_some() {
            self.send(
                &json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{
                    "textDocument":{"uri":file_uri(path)?}
                }}),
            )?;
        }
        Ok(())
    }

    /// Sends a request and returns its id. The response is collected with
    /// [`take_response`](Self::take_response).
    pub fn request_raw(&mut self, method: &str, params: Value) -> Result<u64, LspError> {
        if self.state != ServerState::Ready {
            return Err(LspError::NotReady(self.name.clone()));
        }
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        self.pending.insert(
            id,
            PendingRequest {
                method: method.into(),
                sent: Instant::now(),
            },
        );
        Ok(id)
    }

    /// Sends a typed request.
    pub fn request<R: Request>(&mut self, params: R::Params) -> Result<u64, LspError> {
        self.request_raw(R::METHOD, serde_json::to_value(params)?)
    }

    pub fn is_pending(&self, id: u64) -> bool {
        self.pending.contains_key(&id)
    }

    /// Takes the response for `id` if it has arrived.
    pub fn take_response(&mut self, id: u64) -> Option<LspResponse> {
        self.responses.remove(&id)
    }

    /// Takes and decodes the response for a typed request.
    pub fn take_typed<R: Request>(&mut self, id: u64) -> Option<Result<R::Result, LspError>> {
        let response = self.take_response(id)?;
        Some(match response.result {
            Ok(value) => serde_json::from_value(value).map_err(LspError::from),
            Err(error) => Err(LspError::Server {
                code: error.code,
                message: error.message,
            }),
        })
    }

    /// Cancels a pending request; a late response is discarded.
    pub fn cancel(&mut self, id: u64) {
        self.responses.remove(&id);
        if self.pending.remove(&id).is_some() && self.state == ServerState::Ready {
            let _ =
                self.send(&json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}}));
        }
    }

    /// Sends a notification.
    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        self.send(&json!({"jsonrpc":"2.0","method":method,"params":params}))
    }

    /// Asks the server to shut down and exit, then kills it if it lingers.
    pub fn shutdown(&mut self) {
        if self.state == ServerState::Exited {
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        let _ = self.send(&json!({"jsonrpc":"2.0","id":id,"method":"shutdown","params":null}));
        let deadline = Instant::now() + Duration::from_millis(300);
        // Give the server a moment to acknowledge before `exit`.
        while Instant::now() < deadline {
            match self.messages.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(message)) if message.get("id").and_then(Value::as_u64) == Some(id) => break,
                Ok(Ok(_)) => {}
                _ => break,
            }
        }
        let _ = self.send(&json!({"jsonrpc":"2.0","method":"exit","params":null}));
        let deadline = Instant::now() + Duration::from_millis(200);
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.state = ServerState::Exited;
        self.progress.clear();
    }

    fn handle(&mut self, message: Value) -> Result<(), LspError> {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();
        match (method, id) {
            (Some(method), Some(id)) => self.handle_server_request(method, id, &message),
            (Some(method), None) => {
                self.handle_notification(method, message.get("params").unwrap_or(&Value::Null));
                Ok(())
            }
            (None, Some(id)) => self.handle_response(&id, &message),
            (None, None) => Ok(()),
        }
    }

    fn handle_response(&mut self, id: &Value, message: &Value) -> Result<(), LspError> {
        let Some(id) = id.as_u64() else {
            return Ok(());
        };
        let Some(pending) = self.pending.remove(&id) else {
            // Cancelled or unknown: drop it.
            return Ok(());
        };
        let latency = pending.sent.elapsed();
        self.latencies.insert(pending.method.clone(), latency);
        let result = match message.get("error") {
            Some(error) => Err(ResponseError {
                code: error.get("code").and_then(Value::as_i64).unwrap_or(0),
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
                    .into(),
            }),
            None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
        };
        if id == INITIALIZE_ID {
            return self.finish_initialize(result);
        }
        if self.responses.len() >= MAX_UNCLAIMED_RESPONSES {
            self.responses.clear();
        }
        self.responses.insert(
            id,
            LspResponse {
                method: pending.method,
                result,
                latency,
            },
        );
        Ok(())
    }

    fn finish_initialize(&mut self, result: Result<Value, ResponseError>) -> Result<(), LspError> {
        let value = result
            .map_err(|error| LspError::Protocol(format!("initialize failed: {}", error.message)))?;
        if let Ok(result) = serde_json::from_value::<InitializeResult>(value) {
            self.capabilities = result.capabilities;
        }
        self.state = ServerState::Ready;
        self.notify("initialized", json!({}))?;
        let queued = std::mem::take(&mut self.queued_documents);
        for (path, document) in queued {
            self.send_open(
                &path,
                &document.language_id,
                &document.text,
                document.version,
            )?;
        }
        Ok(())
    }

    fn handle_server_request(
        &mut self,
        method: &str,
        id: Value,
        message: &Value,
    ) -> Result<(), LspError> {
        let params = message.get("params").unwrap_or(&Value::Null);
        let result = match method {
            "window/workDoneProgress/create"
            | "client/registerCapability"
            | "client/unregisterCapability"
            | "window/showMessageRequest"
            | "workspace/diagnostic/refresh"
            | "workspace/semanticTokens/refresh"
            | "workspace/inlayHint/refresh"
            | "workspace/codeLens/refresh" => Value::Null,
            "workspace/configuration" => {
                let count = params
                    .get("items")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len);
                Value::Array(vec![Value::Null; count])
            }
            "workspace/workspaceFolders" => json!([workspace_folder(&self.root)?]),
            "workspace/applyEdit" => {
                json!({"applied": false, "failureReason": "not supported yet"})
            }
            _ => {
                return self.send(&json!({"jsonrpc":"2.0","id":id,"error":{
                    "code": -32601, "message": format!("{method} is not supported by Jetthing")
                }}));
            }
        };
        self.send(&json!({"jsonrpc":"2.0","id":id,"result":result}))
    }

    fn handle_notification(&mut self, method: &str, params: &Value) {
        match method {
            "textDocument/publishDiagnostics" => {
                let Some(path) = params
                    .get("uri")
                    .and_then(Value::as_str)
                    .and_then(|uri| Url::parse(uri).ok())
                    .and_then(|url| url.to_file_path().ok())
                else {
                    return;
                };
                let diagnostics = params
                    .get("diagnostics")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(parse_diagnostic)
                    .collect();
                self.diagnostics.insert(path, diagnostics);
            }
            "$/progress" => self.handle_progress(params),
            "window/showMessage" => {
                let kind = params
                    .get("type")
                    .and_then(Value::as_u64)
                    .and_then(|kind| u8::try_from(kind).ok())
                    .unwrap_or(3);
                if let Some(text) = params.get("message").and_then(Value::as_str) {
                    self.shown_messages.push(ServerMessage {
                        kind,
                        text: text.into(),
                    });
                    self.shown_messages.truncate(32);
                }
            }
            // Eclipse JDT LS reports its startup phases here.
            "language/status" => {
                self.status_message = params
                    .get("message")
                    .and_then(Value::as_str)
                    .map(Into::into);
            }
            _ => {}
        }
    }

    fn handle_progress(&mut self, params: &Value) {
        let Some(token) = params.get("token").map(token_key) else {
            return;
        };
        let Some(value) = params.get("value") else {
            return;
        };
        let text = |name: &str| value.get(name).and_then(Value::as_str).map(String::from);
        let percentage = value
            .get("percentage")
            .and_then(Value::as_u64)
            .and_then(|p| u32::try_from(p.min(100)).ok());
        match value.get("kind").and_then(Value::as_str) {
            Some("begin") => {
                self.progress.insert(
                    token,
                    WorkProgress {
                        title: text("title").unwrap_or_default(),
                        message: text("message"),
                        percentage,
                    },
                );
            }
            Some("report") => {
                if let Some(progress) = self.progress.get_mut(&token) {
                    if let Some(message) = text("message") {
                        progress.message = Some(message);
                    }
                    if percentage.is_some() {
                        progress.percentage = percentage;
                    }
                }
            }
            Some("end") => {
                self.progress.remove(&token);
            }
            _ => {}
        }
    }

    fn send_open(
        &mut self,
        path: &Path,
        language_id: &str,
        text: &str,
        version: i32,
    ) -> Result<(), LspError> {
        self.open_documents.insert(path.to_owned(), version);
        self.send(
            &json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{
                "uri":file_uri(path)?,"languageId":language_id,"version":version,"text":text
            }}}),
        )
    }

    fn send(&mut self, message: &Value) -> Result<(), LspError> {
        let mut writer = self.writer.lock().map_err(|_| LspError::Lock)?;
        transport::write_message(&mut *writer, message)
    }
}

impl Drop for LspService {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn token_key(token: &Value) -> String {
    match token {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn workspace_folder(root: &Path) -> Result<WorkspaceFolder, LspError> {
    Ok(WorkspaceFolder {
        uri: Url::from_directory_path(root).map_err(|_| LspError::InvalidPath(root.to_owned()))?,
        name: root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace")
            .into(),
    })
}

/// The capabilities Jetthing advertises.
pub fn client_capabilities() -> ClientCapabilities {
    let markup = Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]);
    ClientCapabilities {
        general: Some(GeneralClientCapabilities {
            position_encodings: Some(vec![PositionEncodingKind::UTF16]),
            ..Default::default()
        }),
        window: Some(WindowClientCapabilities {
            work_done_progress: Some(true),
            ..Default::default()
        }),
        text_document: Some(TextDocumentClientCapabilities {
            synchronization: Some(TextDocumentSyncClientCapabilities {
                did_save: Some(false),
                ..Default::default()
            }),
            publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                related_information: Some(true),
                ..Default::default()
            }),
            completion: Some(CompletionClientCapabilities {
                completion_item: Some(CompletionItemCapability {
                    snippet_support: Some(true),
                    insert_replace_support: Some(true),
                    documentation_format: markup.clone(),
                    deprecated_support: Some(true),
                    label_details_support: Some(true),
                    resolve_support: Some(CompletionItemCapabilityResolveSupport {
                        properties: vec![
                            "documentation".into(),
                            "detail".into(),
                            "additionalTextEdits".into(),
                        ],
                    }),
                    ..Default::default()
                }),
                completion_item_kind: Some(CompletionItemKindCapability {
                    value_set: Some(
                        (1..=25)
                            .filter_map(|kind| {
                                serde_json::from_value::<CompletionItemKind>(json!(kind)).ok()
                            })
                            .collect(),
                    ),
                }),
                context_support: Some(true),
                ..Default::default()
            }),
            hover: Some(HoverClientCapabilities {
                content_format: markup.clone(),
                ..Default::default()
            }),
            signature_help: Some(SignatureHelpClientCapabilities {
                signature_information: Some(SignatureInformationSettings {
                    documentation_format: markup,
                    parameter_information: Some(ParameterInformationSettings {
                        label_offset_support: Some(true),
                    }),
                    active_parameter_support: Some(true),
                }),
                context_support: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[allow(deprecated)] // rootUri/rootPath are still read by several servers.
fn initialize_params(
    root: &Path,
    initialization_options: Option<Value>,
) -> Result<InitializeParams, LspError> {
    let folder = workspace_folder(root)?;
    Ok(InitializeParams {
        process_id: Some(std::process::id()),
        root_path: Some(root.display().to_string()),
        root_uri: Some(folder.uri.clone()),
        initialization_options,
        capabilities: client_capabilities(),
        workspace_folders: Some(vec![folder]),
        client_info: Some(ClientInfo {
            name: "Jetthing".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
        ..Default::default()
    })
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

/// `file://` URI of `path` (canonicalized when it exists).
pub fn file_uri(path: &Path) -> Result<Url, LspError> {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    Url::from_file_path(&absolute).map_err(|_| LspError::InvalidPath(path.to_owned()))
}
