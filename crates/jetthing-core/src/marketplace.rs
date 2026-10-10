//! JetBrains Marketplace browsing and install. Network work runs on
//! background threads; results are applied by [`IdeState::poll_marketplace`].

use crate::{IdeState, spec, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_marketplace::{
    Converted, MarketplaceClient, MarketplaceError, PluginDetails, SearchPage, SearchPlugin,
};
use serde_json::{Value, json};
use std::sync::mpsc::{Receiver, Sender, channel};

enum Reply {
    Search(u64, Result<SearchPage, MarketplaceError>),
    Details(u64, Result<PluginDetails, MarketplaceError>),
    Install(u64, Result<Converted, MarketplaceError>),
}

/// Outcome of the last install, shown on the Plugins page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed { plugin: String, summary: String },
    Failed(String),
}

pub struct MarketplaceState {
    pub query: String,
    pub results: Vec<SearchPlugin>,
    pub total: u64,
    pub searching: bool,
    pub search_error: Option<String>,
    pub details: Option<PluginDetails>,
    pub details_loading: Option<u64>,
    pub details_error: Option<String>,
    pub installing: Option<u64>,
    /// Last install result per Marketplace plugin id.
    pub outcomes: Vec<(u64, InstallOutcome)>,
    generation: u64,
    sender: Sender<Reply>,
    receiver: Receiver<Reply>,
}

impl Default for MarketplaceState {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            query: String::new(),
            results: Vec::new(),
            total: 0,
            searching: false,
            search_error: None,
            details: None,
            details_loading: None,
            details_error: None,
            installing: None,
            outcomes: Vec::new(),
            generation: 0,
            sender,
            receiver,
        }
    }
}

impl MarketplaceState {
    pub fn is_busy(&self) -> bool {
        self.searching || self.details_loading.is_some() || self.installing.is_some()
    }

    pub fn outcome(&self, id: u64) -> Option<&InstallOutcome> {
        self.outcomes
            .iter()
            .rev()
            .find(|(plugin, _)| *plugin == id)
            .map(|(_, outcome)| outcome)
    }

    fn spawn(&self, work: impl FnOnce(MarketplaceClient) -> Reply + Send + 'static) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let _ = sender.send(work(MarketplaceClient::default()));
        });
    }

    pub fn search(&mut self, query: &str) {
        self.generation += 1;
        let generation = self.generation;
        self.query = query.to_owned();
        self.searching = true;
        self.search_error = None;
        let query = query.to_owned();
        self.spawn(move |client| Reply::Search(generation, client.search(&query, jetthing_marketplace::api::MAX_PAGE, 0)));
    }

    pub fn load_details(&mut self, id: u64) {
        self.details_loading = Some(id);
        self.details_error = None;
        self.spawn(move |client| Reply::Details(id, client.details(id)));
    }

    /// Downloads, inspects and converts `id` (an explicit user action).
    pub fn install(&mut self, id: u64) -> Result<(), String> {
        if let Some(busy) = self.installing {
            return Err(format!("already installing Marketplace plugin {busy}"));
        }
        self.installing = Some(id);
        self.spawn(move |client| {
            Reply::Install(id, jetthing_marketplace::fetch_and_convert(&client, id))
        });
        Ok(())
    }
}

impl IdeState {
    /// Applies finished Marketplace work.
    pub fn poll_marketplace(&mut self) {
        while let Ok(reply) = self.marketplace.receiver.try_recv() {
            match reply {
                Reply::Search(generation, result) => {
                    if generation != self.marketplace.generation {
                        continue;
                    }
                    self.marketplace.searching = false;
                    match result {
                        Ok(page) => {
                            self.marketplace.total = page.total;
                            self.marketplace.results = page.plugins;
                        }
                        Err(error) => self.marketplace.search_error = Some(error.to_string()),
                    }
                }
                Reply::Details(id, result) => {
                    if self.marketplace.details_loading != Some(id) {
                        continue;
                    }
                    self.marketplace.details_loading = None;
                    match result {
                        Ok(details) => self.marketplace.details = Some(details),
                        Err(error) => self.marketplace.details_error = Some(error.to_string()),
                    }
                }
                Reply::Install(id, result) => {
                    self.marketplace.installing = None;
                    let outcome = match result.map_err(|e| e.to_string()).and_then(|converted| {
                        self.install_converted(&converted).map(|()| converted)
                    }) {
                        Ok(converted) => {
                            let kinds: Vec<String> = converted
                                .manifest
                                .assets
                                .iter()
                                .map(|asset| {
                                    format!(
                                        "{} “{}”",
                                        asset.kind.label(),
                                        asset.name.clone().unwrap_or_default()
                                    )
                                })
                                .collect();
                            let summary = format!(
                                "Installed {}: {}",
                                converted.manifest.name,
                                kinds.join(", ")
                            );
                            self.status = summary.clone();
                            InstallOutcome::Installed {
                                plugin: converted.manifest.id.clone(),
                                summary,
                            }
                        }
                        Err(error) => {
                            self.status = error.clone();
                            InstallOutcome::Failed(error)
                        }
                    };
                    self.marketplace.outcomes.push((id, outcome));
                }
            }
        }
    }

    /// Installs a converted plugin through the regular `.jtplugin` path.
    fn install_converted(&mut self, converted: &Converted) -> Result<(), String> {
        let dir = self
            .plugin_host
            .install_dir
            .clone()
            .ok_or("no per-user configuration directory; set JETTHING_CONFIG_DIR")?;
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let archive = dir.join(format!(".{}.download.jtplugin", converted.manifest.id));
        std::fs::write(&archive, &converted.archive)
            .map_err(|e| format!("{}: {e}", archive.display()))?;
        let result = self.install_plugin_archive(&archive);
        let _ = std::fs::remove_file(&archive);
        result.map(|_| ()).map_err(|error| error.to_string())
    }
}

fn id_param(params: &Value) -> Result<u64, CommandError> {
    params.get("id").and_then(Value::as_u64).ok_or_else(|| {
        CommandError::InvalidParams("expected numeric field `id` (Marketplace plugin id)".into())
    })
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    let mut add = |command| registry.register(command).expect("unique built-in id");
    add(Command::new(
        spec(
            "marketplace.search",
            "Search JetBrains Marketplace",
            // Used from the Plugins page; not a File menu item.
            "Marketplace",
            None,
            r#"{"query":"string?"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let query = string_param(&params, "query").unwrap_or_default();
            state.marketplace.search(&query);
            Ok(json!({"query": query}))
        },
    ));
    add(Command::new(
        spec(
            "marketplace.details",
            "Show Marketplace Plugin",
            "Marketplace",
            None,
            r#"{"id":"number"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let id = id_param(&params)?;
            state.marketplace.load_details(id);
            Ok(Value::Null)
        },
    ));
    add(Command::new(
        spec(
            "marketplace.install",
            "Install Marketplace Plugin",
            "Marketplace",
            None,
            r#"{"id":"number"}"#,
        ),
        |state: &IdeState| state.marketplace.installing.is_none(),
        |state: &mut IdeState, params| {
            let id = id_param(&params)?;
            state
                .marketplace
                .install(id)
                .map_err(CommandError::Failed)?;
            Ok(json!({"id": id, "status": "downloading"}))
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_search_replies_are_ignored_and_errors_are_reported() {
        let mut state = IdeState::default();
        state.marketplace.generation = 2;
        state.marketplace.searching = true;
        let sender = state.marketplace.sender.clone();
        sender
            .send(Reply::Search(
                1,
                Ok(SearchPage {
                    plugins: Vec::new(),
                    total: 99,
                }),
            ))
            .unwrap();
        sender
            .send(Reply::Search(
                2,
                Err(MarketplaceError::Network("offline".into())),
            ))
            .unwrap();
        state.poll_marketplace();
        assert_eq!(state.marketplace.total, 0);
        assert!(!state.marketplace.searching);
        assert!(
            state
                .marketplace
                .search_error
                .as_deref()
                .unwrap()
                .contains("offline")
        );
    }

    #[test]
    fn unsupported_installs_record_the_reason() {
        let mut state = IdeState::default();
        state.marketplace.installing = Some(7);
        state
            .marketplace
            .sender
            .send(Reply::Install(
                7,
                Err(MarketplaceError::Unsupported(
                    "Requires IntelliJ Platform (JVM)".into(),
                )),
            ))
            .unwrap();
        state.poll_marketplace();
        assert_eq!(state.marketplace.installing, None);
        assert!(
            matches!(state.marketplace.outcome(7), Some(InstallOutcome::Failed(reason)) if reason.contains("JVM"))
        );
        assert!(state.status.contains("JVM"));
        let registry = crate::built_in_commands();
        assert!(
            registry
                .execute("marketplace.install", &mut state, json!({}))
                .is_err()
        );
    }
}
