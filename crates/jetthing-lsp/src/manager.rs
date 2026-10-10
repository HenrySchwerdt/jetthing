//! Runs the language servers of a project.
//!
//! Servers start lazily the first time a document of their language is
//! synchronised, one per (server, workspace root). Crashed servers are
//! restarted with exponential backoff and their documents re-opened; missing
//! servers produce one actionable [`LspNotice`] each.

use crate::{
    LspError,
    client::{LspDiagnostic, LspService, ServerState, WorkProgress},
    registry::{LaunchContext, LspSettings, ServerRegistry, cache_dir, workspace_root},
};
use lsp_types::{CompletionItem, CompletionResponse, CompletionTriggerKind, request::Completion};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Crashes after which a server is not restarted automatically.
const MAX_RESTARTS: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct ServerKey {
    pub server: String,
    pub root: PathBuf,
}

/// Lifecycle as shown in the status bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ServerStatus {
    Starting,
    Ready,
    /// Crashed; restarting at `attempt` after a backoff.
    Restarting {
        attempt: u32,
    },
    /// Gave up after repeated crashes or failed to launch.
    Failed {
        reason: String,
    },
}

/// Snapshot of one server for the status bar widget and `language.servers`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub language: String,
    pub root: PathBuf,
    pub status: ServerStatus,
    pub progress: Vec<WorkProgress>,
    pub message: Option<String>,
}

/// A notification balloon about a language server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LspNotice {
    /// Stable id used to dismiss the notice (`missing:jdtls`).
    pub id: String,
    pub title: String,
    pub body: String,
    pub error: bool,
}

struct ManagedServer {
    key: ServerKey,
    language: &'static str,
    service: Option<LspService>,
    crashes: u32,
    restart_at: Option<Instant>,
    failure: Option<String>,
}

struct OpenDocument {
    server: ServerKey,
    language_id: &'static str,
}

struct CompletionRequest {
    server: ServerKey,
    id: u64,
}

pub struct LspManager {
    registry: ServerRegistry,
    settings: LspSettings,
    project_root: Option<PathBuf>,
    servers: Vec<ManagedServer>,
    documents: HashMap<PathBuf, OpenDocument>,
    /// Servers whose executable is missing, by id; checked once per project.
    missing: BTreeSet<String>,
    notices: Vec<LspNotice>,
    dismissed: BTreeSet<String>,
    completion: Option<CompletionRequest>,
    completion_result: Option<CompletionResult>,
}

/// An in-flight request sent with [`LspManager::send_request`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHandle {
    pub server: ServerKey,
    pub id: u64,
}

/// Why completion was requested (LSP `CompletionTriggerKind`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionTrigger {
    /// Ctrl+Space or typing an identifier character.
    Invoked,
    /// A server trigger character such as `.`.
    Character(String),
    /// Re-query because the previous result was `isIncomplete`.
    Incomplete,
}

/// A completion response.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub incomplete: bool,
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new(ServerRegistry::default())
    }
}

impl LspManager {
    pub fn new(registry: ServerRegistry) -> Self {
        Self {
            registry,
            settings: LspSettings::default(),
            project_root: None,
            servers: Vec::new(),
            documents: HashMap::new(),
            missing: BTreeSet::new(),
            notices: Vec::new(),
            dismissed: BTreeSet::new(),
            completion: None,
            completion_result: None,
        }
    }

    pub fn registry(&self) -> &ServerRegistry {
        &self.registry
    }

    /// Stops all servers and switches to `root`, reloading `lsp.toml`.
    pub fn set_project(&mut self, root: Option<&Path>) {
        self.shutdown();
        self.missing.clear();
        self.notices.clear();
        self.project_root = root.map(|root| root.canonicalize().unwrap_or_else(|_| root.into()));
        self.settings = match LspSettings::load(self.project_root.as_deref()) {
            Ok(settings) => settings,
            Err(error) => {
                self.push_notice(LspNotice {
                    id: "settings".into(),
                    title: "Invalid language server settings".into(),
                    body: error.to_string(),
                    error: true,
                });
                LspSettings::default()
            }
        };
    }

    /// Stops every server.
    pub fn shutdown(&mut self) {
        self.cancel_completion();
        self.documents.clear();
        for server in &mut self.servers {
            if let Some(service) = server.service.as_mut() {
                service.shutdown();
            }
        }
        self.servers.clear();
    }

    /// Restarts every server and re-opens its documents on the next sync.
    /// Returns the documents that need to be re-synchronised.
    pub fn restart_all(&mut self) -> Vec<PathBuf> {
        let paths: Vec<PathBuf> = self.documents.keys().cloned().collect();
        self.shutdown();
        self.missing.clear();
        self.notices
            .retain(|notice| !notice.id.starts_with("missing:"));
        self.dismissed.clear();
        paths
    }

    /// Whether some enabled server handles `path`.
    pub fn handles(&self, path: &Path) -> bool {
        self.registry
            .for_path(path)
            .is_some_and(|(provider, _)| self.enabled(provider.spec().id))
    }

    fn enabled(&self, id: &str) -> bool {
        self.settings
            .servers
            .get(id)
            .is_none_or(|settings| settings.enabled)
    }

    /// Opens or updates `path` (absolute) on its server, starting the server if needed.
    pub fn sync_document(&mut self, path: &Path, text: &str, version: i32) -> Result<(), LspError> {
        let Some((provider, language_id)) = self.registry.for_path(path) else {
            return Ok(());
        };
        let spec = provider.spec();
        if !self.enabled(spec.id) || self.missing.contains(spec.id) {
            return Ok(());
        }
        let project_root = self
            .project_root
            .clone()
            .unwrap_or_else(|| path.parent().unwrap_or(path).to_owned());
        let key = ServerKey {
            server: spec.id.into(),
            root: workspace_root(path, &project_root, spec.root_markers),
        };
        let language = spec.language;
        let index = match self.servers.iter().position(|server| server.key == key) {
            Some(index) => index,
            None => {
                let service = match self.launch(&key) {
                    Ok(service) => Some(service),
                    Err(error @ LspError::ServerNotFound { .. }) => {
                        let notify = self
                            .registry
                            .get(&key.server)
                            .is_some_and(|provider| provider.spec().notify_missing);
                        self.missing.insert(key.server.clone());
                        if notify {
                            self.push_notice(LspNotice {
                                id: format!("missing:{}", key.server),
                                title: format!("{language} language server not found"),
                                body: error.to_string(),
                                error: false,
                            });
                        }
                        return Err(error);
                    }
                    Err(error) => {
                        self.servers.push(ManagedServer {
                            key: key.clone(),
                            language,
                            service: None,
                            crashes: MAX_RESTARTS,
                            restart_at: None,
                            failure: Some(error.to_string()),
                        });
                        return Err(error);
                    }
                };
                self.servers.push(ManagedServer {
                    key: key.clone(),
                    language,
                    service,
                    crashes: 0,
                    restart_at: None,
                    failure: None,
                });
                self.servers.len() - 1
            }
        };
        self.documents.insert(
            path.to_owned(),
            OpenDocument {
                server: key,
                language_id,
            },
        );
        match self.servers[index].service.as_mut() {
            Some(service) if service.state() != ServerState::Exited => {
                service.sync_document(path, language_id, text, version)
            }
            // Restarting: the document is re-sent once the server is back.
            _ => Ok(()),
        }
    }

    pub fn close_document(&mut self, path: &Path) -> Result<(), LspError> {
        let Some(document) = self.documents.remove(path) else {
            return Ok(());
        };
        match self.service_mut(&document.server) {
            Some(service) => service.close_document(path),
            None => Ok(()),
        }
    }

    fn launch(&self, key: &ServerKey) -> Result<LspService, LspError> {
        let provider = self
            .registry
            .get(&key.server)
            .ok_or_else(|| LspError::Protocol(format!("unknown server {}", key.server)))?;
        let config = provider.launch(&LaunchContext {
            root: &key.root,
            cache_dir: cache_dir(),
            settings: self.settings.servers.get(&key.server),
        })?;
        LspService::launch(&config, &key.root)
    }

    /// Pumps every server, restarts crashed ones and collects completion
    /// results. Returns documents that must be re-synchronised because their
    /// server restarted.
    pub fn poll(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        let mut restarted = Vec::new();
        for index in 0..self.servers.len() {
            let server = &mut self.servers[index];
            if let Some(service) = server.service.as_mut() {
                if let Err(error) = service.poll() {
                    server.failure = Some(error.to_string());
                    service.shutdown();
                }
                if service.state() == ServerState::Exited && server.restart_at.is_none() {
                    let reason = service
                        .exit_reason()
                        .map(str::to_owned)
                        .or_else(|| server.failure.clone())
                        .unwrap_or_else(|| format!("{} exited", server.key.server));
                    server.service = None;
                    server.crashes += 1;
                    if server.crashes > MAX_RESTARTS {
                        server.failure = Some(reason.clone());
                        let notice = LspNotice {
                            id: format!("crashed:{}", server.key.server),
                            title: format!("{} language server keeps crashing", server.language),
                            body: format!(
                                "{reason}\n\nRun \"Restart Language Servers\" after fixing the problem."
                            ),
                            error: true,
                        };
                        self.push_notice(notice);
                        continue;
                    }
                    let backoff = Duration::from_secs(1 << (server.crashes - 1).min(5));
                    server.restart_at = Some(now + backoff);
                    server.failure = Some(reason);
                }
            }
            let server = &mut self.servers[index];
            if server.service.is_none()
                && server.restart_at.is_some_and(|at| at <= now)
                && server.crashes <= MAX_RESTARTS
            {
                server.restart_at = None;
                let key = server.key.clone();
                match self.launch(&key) {
                    Ok(service) => {
                        self.servers[index].service = Some(service);
                        restarted.extend(
                            self.documents
                                .iter()
                                .filter(|(_, document)| document.server == key)
                                .map(|(path, _)| path.clone()),
                        );
                    }
                    Err(error) => {
                        let server = &mut self.servers[index];
                        server.crashes = MAX_RESTARTS + 1;
                        server.failure = Some(error.to_string());
                    }
                }
            }
        }
        self.poll_completion();
        restarted
    }

    /// The running server for `path`, if any.
    pub fn server_for(&self, path: &Path) -> Option<&LspService> {
        let key = &self.documents.get(path)?.server;
        self.servers
            .iter()
            .find(|server| &server.key == key)?
            .service
            .as_ref()
    }

    pub fn server_for_mut(&mut self, path: &Path) -> Option<&mut LspService> {
        let key = self.documents.get(path)?.server.clone();
        self.service_mut(&key)
    }

    fn service_mut(&mut self, key: &ServerKey) -> Option<&mut LspService> {
        self.servers
            .iter_mut()
            .find(|server| &server.key == key)?
            .service
            .as_mut()
    }

    /// A ready server for `path`, or an actionable explanation of why there is none.
    pub fn ready_server_for(&mut self, path: &Path) -> Result<&mut LspService, LspError> {
        let Some((provider, _)) = self.registry.for_path(path) else {
            return Err(LspError::Unsupported(
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ));
        };
        let spec = provider.spec();
        let (id, language) = (spec.id, spec.language);
        if !self.enabled(id) {
            return Err(LspError::Disabled(id.into()));
        }
        if self.missing.contains(id) {
            return Err(LspError::ServerNotFound {
                server: id.into(),
                language: language.into(),
                hint: spec.install_hint.into(),
            });
        }
        let Some(document) = self.documents.get(path) else {
            return Err(LspError::NotReady(id.into()));
        };
        let key = document.server.clone();
        let server = self
            .servers
            .iter_mut()
            .find(|server| server.key == key)
            .ok_or_else(|| LspError::NotReady(id.into()))?;
        match server.service.as_mut() {
            Some(service) if service.state() == ServerState::Ready => Ok(service),
            Some(service) => {
                let detail = service
                    .progress()
                    .values()
                    .next()
                    .map(describe_progress)
                    .or_else(|| service.status_message().map(str::to_owned));
                Err(LspError::Starting {
                    server: id.into(),
                    detail,
                })
            }
            None => Err(match &server.failure {
                Some(reason) if server.crashes > MAX_RESTARTS => LspError::Protocol(format!(
                    "{id} failed: {}",
                    reason.lines().next().unwrap_or_default()
                )),
                _ => LspError::Starting {
                    server: id.into(),
                    detail: Some("restarting after a crash".into()),
                },
            }),
        }
    }

    /// Diagnostics published for `path` by its server.
    pub fn diagnostics_for(&self, path: &Path) -> Vec<LspDiagnostic> {
        let Some(service) = self.server_for(path) else {
            return Vec::new();
        };
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        service
            .diagnostics()
            .get(&canonical)
            .or_else(|| service.diagnostics().get(path))
            .cloned()
            .unwrap_or_default()
    }

    /// One entry per started server.
    pub fn servers(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|server| {
                let (status, progress, message) = match &server.service {
                    Some(service) => (
                        match service.state() {
                            ServerState::Ready => ServerStatus::Ready,
                            _ => ServerStatus::Starting,
                        },
                        service.progress().values().cloned().collect(),
                        service.status_message().map(str::to_owned),
                    ),
                    None if server.restart_at.is_some() => (
                        ServerStatus::Restarting {
                            attempt: server.crashes,
                        },
                        Vec::new(),
                        server.failure.clone(),
                    ),
                    None => (
                        ServerStatus::Failed {
                            reason: server.failure.clone().unwrap_or_default(),
                        },
                        Vec::new(),
                        None,
                    ),
                };
                ServerInfo {
                    id: server.key.server.clone(),
                    language: server.language.into(),
                    root: server.key.root.clone(),
                    status,
                    progress,
                    message,
                }
            })
            .collect()
    }

    /// Ids of servers that were looked for but are not installed.
    pub fn missing_servers(&self) -> impl Iterator<Item = &str> {
        self.missing.iter().map(String::as_str)
    }

    /// Whether any server is starting or reporting progress.
    pub fn is_busy(&self) -> bool {
        self.servers.iter().any(|server| match &server.service {
            Some(service) => {
                service.state() == ServerState::Starting || !service.progress().is_empty()
            }
            None => server.restart_at.is_some(),
        }) || self.completion.is_some()
    }

    pub fn notices(&self) -> &[LspNotice] {
        &self.notices
    }

    pub fn dismiss_notice(&mut self, id: &str) {
        self.notices.retain(|notice| notice.id != id);
        self.dismissed.insert(id.into());
    }

    fn push_notice(&mut self, notice: LspNotice) {
        if self.dismissed.contains(&notice.id) {
            return;
        }
        self.notices.retain(|existing| existing.id != notice.id);
        self.notices.push(notice);
    }

    /// Requests completion at a UTF-16 position, cancelling any older request.
    pub fn request_completion(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
        trigger: CompletionTrigger,
    ) -> Result<u64, LspError> {
        self.cancel_completion();
        let service = self.ready_server_for(path)?;
        let server = ServerKey {
            server: service.name().into(),
            root: service.root().to_owned(),
        };
        let context = match trigger {
            CompletionTrigger::Invoked => json!({"triggerKind": CompletionTriggerKind::INVOKED}),
            CompletionTrigger::Character(character) => json!({
                "triggerKind": CompletionTriggerKind::TRIGGER_CHARACTER,
                "triggerCharacter": character
            }),
            CompletionTrigger::Incomplete => json!({
                "triggerKind": CompletionTriggerKind::TRIGGER_FOR_INCOMPLETE_COMPLETIONS
            }),
        };
        let id = service.request_raw(
            <Completion as lsp_types::request::Request>::METHOD,
            json!({
                "textDocument": {"uri": crate::client::file_uri(path)?},
                "position": {"line": line, "character": character},
                "context": context
            }),
        )?;
        let key = self
            .documents
            .get(path)
            .map(|document| document.server.clone())
            .unwrap_or(server);
        self.completion = Some(CompletionRequest { server: key, id });
        Ok(id)
    }

    /// Cancels the pending completion request and drops an untaken result.
    pub fn cancel_completion(&mut self) {
        self.completion_result = None;
        if let Some(request) = self.completion.take()
            && let Some(service) = self.service_mut(&request.server)
        {
            service.cancel(request.id);
        }
    }

    fn poll_completion(&mut self) {
        let Some(request) = &self.completion else {
            return;
        };
        let (key, id) = (request.server.clone(), request.id);
        let Some(service) = self.service_mut(&key) else {
            self.completion = None;
            return;
        };
        if !service.is_pending(id) && service.state() == ServerState::Exited {
            self.completion = None;
            return;
        }
        let Some(result) = service.take_typed::<Completion>(id) else {
            return;
        };
        self.completion = None;
        self.completion_result = Some(match result {
            Ok(Some(CompletionResponse::Array(items))) => CompletionResult {
                items,
                incomplete: false,
            },
            Ok(Some(CompletionResponse::List(list))) => CompletionResult {
                items: list.items,
                incomplete: list.is_incomplete,
            },
            Ok(None) | Err(_) => CompletionResult::default(),
        });
    }

    pub fn completion_pending(&self) -> bool {
        self.completion.is_some()
    }

    /// Takes the latest completion response once it has arrived.
    pub fn take_completion(&mut self) -> Option<CompletionResult> {
        self.completion_result.take()
    }

    /// Filters and sorts `items` for `path`'s language with its provider's
    /// relevance hints.
    pub fn rank_completions(
        &self,
        path: &Path,
        items: &[CompletionItem],
        prefix: &str,
        usage: impl Fn(&CompletionItem) -> i32,
    ) -> Vec<crate::completion::RankedItem> {
        match self.registry.for_path(path) {
            Some((provider, _)) => crate::completion::rank_with_usage(
                items,
                prefix,
                |item| provider.completion_relevance(item),
                usage,
            ),
            None => crate::completion::rank_with_usage(items, prefix, |_| 0, usage),
        }
    }

    /// Characters that open completion for `path`'s running server (`.`, `::`, ...).
    pub fn trigger_characters(&self, path: &Path) -> Vec<String> {
        self.server_for(path)
            .and_then(|service| service.capabilities().completion_provider.as_ref())
            .and_then(|provider| provider.trigger_characters.clone())
            .unwrap_or_default()
    }

    /// Sends a typed request to the ready server of `path`. The handle is used
    /// to collect (`take_response`) or cancel (`cancel_request`) the result.
    pub fn send_request<R: lsp_types::request::Request>(
        &mut self,
        path: &Path,
        params: R::Params,
    ) -> Result<RequestHandle, LspError> {
        let service = self.ready_server_for(path)?;
        let server = ServerKey {
            server: service.name().into(),
            root: service.root().to_owned(),
        };
        let id = service.request::<R>(params)?;
        let server = self
            .documents
            .get(path)
            .map(|document| document.server.clone())
            .unwrap_or(server);
        Ok(RequestHandle { server, id })
    }

    /// The decoded response for `handle`, once it has arrived.
    pub fn take_response<R: lsp_types::request::Request>(
        &mut self,
        handle: &RequestHandle,
    ) -> Option<Result<R::Result, LspError>> {
        let service = self.service_mut(&handle.server);
        match service {
            Some(service) => service.take_typed::<R>(handle.id),
            None => Some(Err(LspError::NotReady(handle.server.server.clone()))),
        }
    }

    /// Whether `handle` is still waiting for its response.
    pub fn is_pending(&self, handle: &RequestHandle) -> bool {
        self.servers
            .iter()
            .find(|server| server.key == handle.server)
            .and_then(|server| server.service.as_ref())
            .is_some_and(|service| service.is_pending(handle.id))
    }

    pub fn cancel_request(&mut self, handle: &RequestHandle) {
        if let Some(service) = self.service_mut(&handle.server) {
            service.cancel(handle.id);
        }
    }

    /// Server capabilities for `path`'s running server.
    pub fn capabilities_for(&self, path: &Path) -> Option<&lsp_types::ServerCapabilities> {
        self.server_for(path).map(LspService::capabilities)
    }

    /// The language id the server uses for `path`.
    pub fn language_id(&self, path: &Path) -> Option<&'static str> {
        self.documents
            .get(path)
            .map(|document| document.language_id)
            .or_else(|| self.registry.for_path(path).map(|(_, id)| id))
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// "Indexing 45% (crates/foo)". Servers that repeat the title in the
/// message (jdtls: "Initialize Workspace - 0%") get the message alone.
pub fn describe_progress(progress: &WorkProgress) -> String {
    let message = progress
        .message
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty());
    let mut text = progress.title.clone();
    if let Some(message) = message
        && !progress.title.is_empty()
        && message.starts_with(&progress.title)
    {
        return message.to_owned();
    }
    if let Some(percentage) = progress.percentage {
        text.push_str(&format!(" {percentage}%"));
    }
    if let Some(message) = message {
        if text.is_empty() {
            text = message.into();
        } else {
            text.push_str(&format!(" ({message})"));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_descriptions_avoid_repeating_the_title() {
        let progress = |title: &str, message: Option<&str>, percentage| WorkProgress {
            title: title.into(),
            message: message.map(Into::into),
            percentage,
        };
        assert_eq!(
            describe_progress(&progress("Indexing", Some("crates/foo"), Some(45))),
            "Indexing 45% (crates/foo)"
        );
        assert_eq!(
            describe_progress(&progress(
                "Initialize Workspace",
                Some("Initialize Workspace - 0% "),
                Some(0)
            )),
            "Initialize Workspace - 0%"
        );
        assert_eq!(
            describe_progress(&progress("", Some("Loading"), None)),
            "Loading"
        );
    }
}
