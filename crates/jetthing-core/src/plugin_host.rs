//! Plugin host: activates plugins, gives each a scoped [`PluginHost`], routes
//! contributed commands, and keeps plugin tool window / editor tab state.

use crate::{IdeState, spec, string_param, usize_param};
use jetthing_command::{Command, CommandError, CommandRegistry, CommandResult};
use jetthing_plugins::{
    EditorTabRequest, Notification, Permission, Plugin, PluginError, PluginFactory, PluginHost,
    PluginRegistry, PluginStatus, PluginStore, StatusWidgetState, ViewContext, ViewPlacement,
    user_config_dir,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A plugin view open in the editor area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginTab {
    pub plugin: String,
    pub view: String,
    pub title: String,
    pub key: String,
    pub path: Option<PathBuf>,
}

impl PluginTab {
    pub fn context(&self) -> ViewContext {
        ViewContext {
            view: self.view.clone(),
            placement: ViewPlacement::EditorTab {
                key: self.key.clone(),
                path: self.path.clone(),
            },
        }
    }
}

/// A status bar widget contributed by an active plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginStatusWidget {
    pub plugin: String,
    pub id: String,
    pub state: StatusWidgetState,
    pub command: Option<String>,
}

#[derive(Default)]
pub struct PluginHostState {
    pub registry: PluginRegistry,
    /// Commands queued by plugins via [`PluginHost::dispatch`]; the UI runs them.
    pub queued_commands: Vec<(String, Value)>,
    /// Visible plugin tool window `(plugin, tool window id)` on the right anchor.
    pub tool_window: Option<(String, String)>,
    pub tabs: Vec<PluginTab>,
    /// Active plugin tab; while set it is shown instead of the active document.
    pub active_tab: Option<usize>,
    /// Unread notifications `(plugin, notification)`, newest last.
    pub notifications: Vec<(String, Notification)>,
    /// Plugin shown alone in standalone app mode (`--app <id>`).
    pub app_mode: Option<String>,
    pub repaint: bool,
    refresh: BTreeSet<String>,
}

/// The per-user directory with installed plugins (`<config>/plugins`).
pub fn user_plugin_dir() -> Option<PathBuf> {
    user_config_dir().map(|dir| dir.join("plugins"))
}

struct HostAdapter<'a> {
    state: &'a mut IdeState,
    id: String,
}

impl PluginHost for HostAdapter<'_> {
    fn plugin_id(&self) -> &str {
        &self.id
    }

    fn project_root(&self) -> Option<&Path> {
        self.state.project.as_ref().map(|project| project.root())
    }

    fn show_status(&mut self, message: &str) {
        self.state.status = message.into();
    }

    fn notify(&mut self, notification: Notification) {
        let plugins = &mut self.state.plugin_host;
        plugins.notifications.push((self.id.clone(), notification));
        let excess = plugins.notifications.len().saturating_sub(20);
        plugins.notifications.drain(..excess);
    }

    fn dispatch(&mut self, command: &str, params: Value) {
        self.state
            .plugin_host
            .queued_commands
            .push((command.into(), params));
    }

    fn open_tool_window(&mut self, id: &str) {
        self.state.show_plugin_tool_window(&self.id.clone(), id);
    }

    fn open_editor_tab(&mut self, request: EditorTabRequest) {
        self.state.open_plugin_tab(&self.id.clone(), request);
    }

    fn setting(&self, key: &str) -> Option<Value> {
        self.state.plugin_setting(&self.id, key)
    }

    fn set_setting(&mut self, key: &str, value: Value) -> Result<(), PluginError> {
        self.state
            .plugin_host
            .registry
            .store
            .set_setting(&self.id, key, value)
    }

    fn data_dir(&self) -> Result<PathBuf, PluginError> {
        let dir = self
            .state
            .plugin_host
            .registry
            .store
            .data_dir(&self.id)
            .ok_or_else(|| {
                PluginError::Io(
                    self.id.clone(),
                    "no per-user configuration directory (set JETTHING_CONFIG_DIR)".into(),
                )
            })?;
        std::fs::create_dir_all(&dir)
            .map_err(|error| PluginError::Io(dir.display().to_string(), error.to_string()))?;
        Ok(dir)
    }

    fn has_permission(&self, permission: Permission) -> bool {
        self.state
            .plugin_host
            .registry
            .get(&self.id)
            .is_some_and(|entry| entry.manifest.has_permission(permission))
    }

    fn refresh_contributions(&mut self) {
        self.state.plugin_host.refresh.insert(self.id.clone());
    }

    fn request_repaint(&mut self) {
        self.state.plugin_host.repaint = true;
    }
}

impl IdeState {
    /// Sets up the registry with the user's store, compiled-in plugins and the
    /// per-user plugin directory, then activates enabled plugins.
    pub fn init_plugins(&mut self, store: PluginStore, bundled: Vec<PluginFactory>) {
        let mut registry = PluginRegistry::new(store);
        for factory in bundled {
            if let Err(error) = registry.add_bundled(factory) {
                registry.problems.push((PathBuf::from("<bundled>"), error));
            }
        }
        self.plugin_host.registry = registry;
        self.discover_plugins();
    }

    /// Rediscovers manifests in the per-user and project plugin directories
    /// and activates newly enabled plugins.
    pub fn discover_plugins(&mut self) {
        let roots: Vec<PathBuf> = user_plugin_dir()
            .into_iter()
            .chain(self.project.as_ref().map(|p| p.root().join("plugins")))
            .collect();
        for root in roots {
            self.plugin_host.registry.discover(&root);
        }
        self.activate_pending_plugins();
    }

    pub fn activate_pending_plugins(&mut self) {
        for id in self.plugin_host.registry.pending_activation() {
            if let Some(problem) = self.plugin_host.registry.dependency_problem(&id) {
                self.plugin_host
                    .registry
                    .set_status(&id, PluginStatus::Failed(problem));
                continue;
            }
            if !self.plugin_host.registry.has_code(&id) {
                // Declarative plugins have nothing to run.
                self.plugin_host
                    .registry
                    .set_status(&id, PluginStatus::Active);
                continue;
            }
            self.plugin_host.registry.ensure_instance(&id);
            let Some(mut instance) = self.plugin_host.registry.take_instance(&id) else {
                continue;
            };
            let result = {
                let mut host = HostAdapter {
                    state: self,
                    id: id.clone(),
                };
                instance.activate(&mut host)
            };
            let contributions = instance.contributions();
            let problems = contributions.problems();
            self.plugin_host.registry.restore_instance(&id, instance);
            let status = match result {
                Err(error) => PluginStatus::Failed(error.to_string()),
                Ok(()) if !problems.is_empty() => PluginStatus::Failed(problems.join("; ")),
                Ok(()) => PluginStatus::Active,
            };
            self.plugin_host
                .registry
                .set_contributions(&id, contributions);
            self.plugin_host.registry.set_status(&id, status);
        }
    }

    /// Runs `f` with the active plugin `id` and a host scoped to it.
    pub fn with_plugin<R>(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut dyn Plugin, &mut dyn PluginHost) -> R,
    ) -> Option<R> {
        if !self.plugin_host.registry.is_active(id) {
            return None;
        }
        let mut instance = self.plugin_host.registry.take_instance(id)?;
        let result = {
            let mut host = HostAdapter {
                state: self,
                id: id.into(),
            };
            f(instance.as_mut(), &mut host)
        };
        if self.plugin_host.refresh.remove(id) {
            self.plugin_host
                .registry
                .set_contributions(id, instance.contributions());
        }
        self.plugin_host.registry.restore_instance(id, instance);
        Some(result)
    }

    /// Enables or disables a plugin, (de)activating its code immediately.
    pub fn set_plugin_enabled(&mut self, id: &str, enabled: bool) -> Result<(), PluginError> {
        if !enabled && self.plugin_host.registry.is_active(id) {
            self.with_plugin(id, |plugin, host| plugin.deactivate(host));
            // A fresh instance is created on re-enable.
            drop(self.plugin_host.registry.take_instance(id));
            self.plugin_host.tabs.retain(|tab| tab.plugin != id);
            self.plugin_host.active_tab = None;
            if self
                .plugin_host
                .tool_window
                .as_ref()
                .is_some_and(|(plugin, _)| plugin == id)
            {
                self.plugin_host.tool_window = None;
            }
        }
        self.plugin_host.registry.set_enabled(id, enabled)?;
        if enabled {
            self.activate_pending_plugins();
        }
        Ok(())
    }

    /// Deactivates every plugin (IDE exit).
    pub fn shutdown_plugins(&mut self) {
        let active: Vec<String> = self
            .plugin_host
            .registry
            .entries()
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| entry.id().to_owned())
            .collect();
        for id in active {
            self.with_plugin(&id, |plugin, host| plugin.deactivate(host));
        }
    }

    pub fn execute_plugin_command(&mut self, command: &str, params: Value) -> CommandResult {
        let owner = self
            .plugin_host
            .registry
            .command_owner(command)
            .ok_or_else(|| CommandError::Disabled(command.into()))?
            .to_owned();
        self.with_plugin(&owner, |plugin, host| plugin.execute(command, params, host))
            .unwrap_or_else(|| Err(CommandError::Disabled(command.into())))
    }

    /// Lets active plugins collect background results.
    pub fn poll_plugins(&mut self) {
        let ids: Vec<String> = self
            .plugin_host
            .registry
            .entries()
            .iter()
            .filter(|entry| entry.is_active() && self.plugin_host.registry.has_code(entry.id()))
            .map(|entry| entry.id().to_owned())
            .collect();
        for id in ids {
            self.with_plugin(&id, |plugin, host| plugin.poll(host));
        }
    }

    /// Takes and clears the repaint request set by plugins.
    pub fn take_plugin_repaint(&mut self) -> bool {
        std::mem::take(&mut self.plugin_host.repaint)
    }

    pub fn plugin_status_widgets(&self) -> Vec<PluginStatusWidget> {
        let registry = &self.plugin_host.registry;
        registry
            .entries()
            .iter()
            .filter(|entry| entry.is_active())
            .flat_map(|entry| {
                entry
                    .contributions
                    .status_widgets
                    .iter()
                    .filter_map(|widget| {
                        // Instances are only absent while a plugin call is running.
                        let state = self.plugin_status_widget(entry.id(), &widget.id)?;
                        Some(PluginStatusWidget {
                            plugin: entry.id().to_owned(),
                            id: widget.id.clone(),
                            state,
                            command: widget.command.clone(),
                        })
                    })
            })
            .collect()
    }

    fn plugin_status_widget(&self, plugin: &str, id: &str) -> Option<StatusWidgetState> {
        self.plugin_host
            .registry
            .instance(plugin)
            .and_then(|instance| instance.status_widget(id))
    }

    /// Stored value of a plugin setting, or the default declared on its settings page.
    pub fn plugin_setting(&self, plugin: &str, key: &str) -> Option<Value> {
        let registry = &self.plugin_host.registry;
        registry.store.setting(plugin, key).cloned().or_else(|| {
            registry.get(plugin).and_then(|entry| {
                entry
                    .contributions
                    .settings_pages
                    .iter()
                    .flat_map(|page| &page.fields)
                    .find(|field| field.key == key)
                    .map(|field| field.kind.default_value())
            })
        })
    }

    pub fn set_plugin_setting(
        &mut self,
        plugin: &str,
        key: &str,
        value: Value,
    ) -> Result<(), String> {
        let entry = self
            .plugin_host
            .registry
            .get(plugin)
            .ok_or_else(|| format!("plugin not found: {plugin}"))?;
        let field = entry
            .contributions
            .settings_pages
            .iter()
            .flat_map(|page| &page.fields)
            .find(|field| field.key == key)
            .ok_or_else(|| format!("plugin {plugin} has no setting `{key}`"))?;
        let value = field.kind.coerce(value)?;
        self.plugin_host
            .registry
            .store
            .set_setting(plugin, key, value)
            .map_err(|error| error.to_string())
    }

    fn show_plugin_tool_window(&mut self, plugin: &str, id: &str) {
        self.plugin_host.tool_window = Some((plugin.into(), id.into()));
    }

    /// Toggles a contributed tool window; returns whether it is visible.
    pub fn toggle_plugin_tool_window(&mut self, plugin: &str, id: &str) -> Result<bool, String> {
        let known = self.plugin_host.registry.get(plugin).is_some_and(|entry| {
            entry.is_active() && entry.contributions.tool_windows.iter().any(|w| w.id == id)
        });
        if !known {
            return Err(format!("plugin {plugin} has no active tool window `{id}`"));
        }
        let target = (plugin.to_owned(), id.to_owned());
        if self.plugin_host.tool_window.as_ref() == Some(&target) {
            self.plugin_host.tool_window = None;
            Ok(false)
        } else {
            self.plugin_host.tool_window = Some(target);
            Ok(true)
        }
    }

    /// Opens (or focuses the tab with the same key of) a plugin editor view.
    pub fn open_plugin_tab(&mut self, plugin: &str, request: EditorTabRequest) {
        let tabs = &mut self.plugin_host.tabs;
        let index = match tabs
            .iter()
            .position(|tab| tab.plugin == plugin && tab.key == request.key)
        {
            Some(index) => {
                tabs[index].title = request.title;
                index
            }
            None => {
                tabs.push(PluginTab {
                    plugin: plugin.into(),
                    view: request.view,
                    title: request.title,
                    key: request.key,
                    path: request.path,
                });
                tabs.len() - 1
            }
        };
        self.plugin_host.active_tab = Some(index);
    }

    pub fn close_plugin_tab(&mut self, index: usize) -> Result<(), String> {
        let tabs = &mut self.plugin_host.tabs;
        if index >= tabs.len() {
            return Err("plugin tab does not exist".into());
        }
        tabs.remove(index);
        self.plugin_host.active_tab = match self.plugin_host.active_tab {
            Some(active) if active == index => None,
            Some(active) if active > index => Some(active - 1),
            other => other,
        };
        Ok(())
    }

    pub fn active_plugin_tab(&self) -> Option<&PluginTab> {
        self.plugin_host
            .active_tab
            .and_then(|index| self.plugin_host.tabs.get(index))
    }
}

/// Re-registers contributed commands after plugin changes. `registered`
/// tracks the ids this function added; built-in commands are never replaced.
pub fn sync_plugin_commands(
    registry: &mut CommandRegistry<IdeState>,
    registered: &mut Vec<String>,
    state: &mut IdeState,
) {
    for id in registered.drain(..) {
        registry.unregister(&id);
    }
    for (plugin, spec) in state.plugin_host.registry.active_commands() {
        let id = spec.id.clone();
        if registry.contains(&id) {
            state.status =
                format!("Plugin {plugin}: command {id} conflicts with an existing command");
            continue;
        }
        let command_id = id.clone();
        let owner = plugin.clone();
        let result = registry.register(Command::new(
            spec,
            move |state: &IdeState| state.plugin_host.registry.is_active(&owner),
            move |state: &mut IdeState, params| state.execute_plugin_command(&command_id, params),
        ));
        if result.is_ok() {
            registered.push(id);
        }
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    let mut add = |command| registry.register(command).expect("unique built-in id");
    add(Command::new(
        spec("plugin.refresh", "Refresh Plugins", "Plugins", None, "{}"),
        |_| true,
        |state: &mut IdeState, _| {
            state.discover_plugins();
            Ok(json!({"count": state.plugin_host.registry.entries().len()}))
        },
    ));
    add(Command::new(
        // Not under "Plugins": the JSON result is for the CLI, not the File menu.
        spec("plugin.list", "List Plugins", "Developer", None, "{}"),
        |_| true,
        |state: &mut IdeState, _| {
            let plugins: Vec<Value> = state
                .plugin_host
                .registry
                .entries()
                .iter()
                .map(|entry| {
                    json!({
                        "id": entry.id(),
                        "name": entry.manifest.name,
                        "version": entry.manifest.version,
                        "kind": entry.manifest.kind.label(),
                        "status": entry.status.label(),
                        "reason": match &entry.status {
                            PluginStatus::Failed(reason) | PluginStatus::Unsupported(reason) => Some(reason),
                            _ => None,
                        },
                    })
                })
                .collect();
            Ok(json!({"plugins": plugins}))
        },
    ));
    for (id, label, enabled) in [
        ("plugin.enable", "Enable Plugin", true),
        ("plugin.disable", "Disable Plugin", false),
    ] {
        add(Command::new(
            spec(id, label, "Plugins", None, r#"{"id":"string"}"#),
            |_| true,
            move |state: &mut IdeState, params| {
                let id = string_param(&params, "id")?;
                state
                    .set_plugin_enabled(&id, enabled)
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                Ok(
                    json!({"id": id, "status": state.plugin_host.registry.get(&id).map(|e| e.status.label())}),
                )
            },
        ));
    }
    add(Command::new(
        spec(
            "plugin.toolwindow.toggle",
            "Toggle Plugin Tool Window",
            "Window",
            None,
            r#"{"plugin":"string","id":"string"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let visible = state
                .toggle_plugin_tool_window(
                    &string_param(&params, "plugin")?,
                    &string_param(&params, "id")?,
                )
                .map_err(CommandError::Failed)?;
            Ok(json!({"visible": visible}))
        },
    ));
    add(Command::new(
        spec(
            "plugin.tab.activate",
            "Activate Plugin Tab",
            "Window",
            None,
            r#"{"index":"number"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let index = usize_param(&params, "index")?;
            if index >= state.plugin_host.tabs.len() {
                return Err(CommandError::InvalidParams(
                    "plugin tab does not exist".into(),
                ));
            }
            state.plugin_host.active_tab = Some(index);
            Ok(Value::Null)
        },
    ));
    add(Command::new(
        spec(
            "plugin.tab.close",
            "Close Plugin Tab",
            "Window",
            None,
            r#"{"index":"number?"}"#,
        ),
        |state: &IdeState| state.plugin_host.active_tab.is_some(),
        |state: &mut IdeState, params| {
            let index = match params.get("index") {
                Some(_) => usize_param(&params, "index")?,
                None => state
                    .plugin_host
                    .active_tab
                    .ok_or_else(|| CommandError::Failed("no plugin tab is active".into()))?,
            };
            state
                .close_plugin_tab(index)
                .map_err(CommandError::Failed)?;
            Ok(Value::Null)
        },
    ));
    add(Command::new(
        spec(
            "plugin.setSetting",
            "Change Plugin Setting",
            "Plugins",
            None,
            r#"{"plugin":"string","key":"string","value":"any"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let plugin = string_param(&params, "plugin")?;
            let key = string_param(&params, "key")?;
            let value = params
                .get("value")
                .cloned()
                .ok_or_else(|| CommandError::InvalidParams("missing value".into()))?;
            state
                .set_plugin_setting(&plugin, &key, value)
                .map_err(CommandError::Failed)?;
            Ok(Value::Null)
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in_commands;
    use jetthing_plugins::{
        CommandContribution, Contributions, PluginManifest, SettingField, SettingKind,
        SettingsPage, StatusWidgetContribution, ToolWindowContribution,
    };
    use std::rc::Rc;

    struct Counter {
        manifest: PluginManifest,
        count: u32,
        fail: bool,
    }

    impl Plugin for Counter {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }

        fn contributions(&self) -> Contributions {
            Contributions::new()
                .command(CommandContribution::new("counter.increment", "Increment"))
                .command(CommandContribution::new("counter.open", "Open Counter"))
                .tool_window(ToolWindowContribution::new("counter", "Counter", "main"))
                .status_widget(StatusWidgetContribution::new("count"))
                .settings_page(
                    SettingsPage::new("counter", "Counter").field(SettingField::new(
                        "step",
                        "Step",
                        SettingKind::Number {
                            default: 1.0,
                            min: Some(1.0),
                            max: Some(5.0),
                        },
                    )),
                )
        }

        fn activate(&mut self, _host: &mut dyn PluginHost) -> Result<(), PluginError> {
            if self.fail {
                return Err(PluginError::Activation {
                    plugin: self.manifest.id.clone(),
                    message: "boom".into(),
                });
            }
            Ok(())
        }

        fn execute(&mut self, command: &str, _: Value, host: &mut dyn PluginHost) -> CommandResult {
            match command {
                "counter.increment" => {
                    let step = host.setting("step").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    self.count += step as u32;
                    host.show_status(&format!("count {}", self.count));
                    host.dispatch("toolwindow.hideAll", Value::Null);
                    Ok(json!(self.count))
                }
                "counter.open" => {
                    host.open_tool_window("counter");
                    host.open_editor_tab(EditorTabRequest::new("main", "Counter"));
                    host.open_editor_tab(EditorTabRequest::new("main", "Counter (again)"));
                    Ok(Value::Null)
                }
                other => Err(CommandError::Unknown(other.into())),
            }
        }

        fn status_widget(&self, _id: &str) -> Option<StatusWidgetState> {
            Some(StatusWidgetState {
                text: self.count.to_string(),
                tooltip: None,
            })
        }
    }

    fn factory(id: &'static str, fail: bool) -> PluginFactory {
        Rc::new(move || {
            Box::new(Counter {
                manifest: PluginManifest::parse(&format!(
                    "id='{id}'\nname='Counter'\nversion='1'\napi_version=1\nkind='rust'\n[app]\nview='main'\n"
                ))
                .unwrap(),
                count: 0,
                fail,
            })
        })
    }

    fn setup(fail: bool) -> (IdeState, CommandRegistry<IdeState>, Vec<String>) {
        let mut state = IdeState::default();
        state.init_plugins(
            PluginStore::in_memory(),
            vec![factory("test.counter", fail)],
        );
        let mut registry = built_in_commands();
        let mut registered = Vec::new();
        sync_plugin_commands(&mut registry, &mut registered, &mut state);
        (state, registry, registered)
    }

    #[test]
    fn contributed_commands_run_through_the_shared_registry() {
        let (mut state, registry, registered) = setup(false);
        assert_eq!(registered, ["counter.increment", "counter.open"]);
        assert!(
            registry
                .specs()
                .iter()
                .any(|spec| spec.id == "counter.increment")
        );
        assert_eq!(
            registry
                .execute("counter.increment", &mut state, Value::Null)
                .unwrap(),
            json!(1)
        );
        assert_eq!(state.status, "count 1");
        assert_eq!(state.plugin_host.queued_commands[0].0, "toolwindow.hideAll");
        registry
            .execute(
                "plugin.setSetting",
                &mut state,
                json!({"plugin": "test.counter", "key": "step", "value": 99}),
            )
            .unwrap();
        // Clamped to the declared maximum.
        assert_eq!(
            state.plugin_setting("test.counter", "step"),
            Some(json!(5.0))
        );
        assert!(
            registry
                .execute(
                    "plugin.setSetting",
                    &mut state,
                    json!({"plugin": "test.counter", "key": "nope", "value": 1})
                )
                .is_err()
        );
        registry
            .execute("counter.increment", &mut state, Value::Null)
            .unwrap();
        assert_eq!(state.plugin_status_widgets()[0].state.text, "6");
    }

    #[test]
    fn views_open_as_tool_windows_and_deduplicated_tabs() {
        let (mut state, registry, _) = setup(false);
        registry
            .execute("counter.open", &mut state, Value::Null)
            .unwrap();
        assert_eq!(
            state.plugin_host.tool_window,
            Some(("test.counter".into(), "counter".into()))
        );
        assert_eq!(state.plugin_host.tabs.len(), 1);
        assert_eq!(state.active_plugin_tab().unwrap().title, "Counter (again)");
        registry
            .execute(
                "plugin.toolwindow.toggle",
                &mut state,
                json!({"plugin": "test.counter", "id": "counter"}),
            )
            .unwrap();
        assert_eq!(state.plugin_host.tool_window, None);
        assert!(
            registry
                .execute(
                    "plugin.toolwindow.toggle",
                    &mut state,
                    json!({"plugin": "test.counter", "id": "x"})
                )
                .is_err()
        );
        registry
            .execute("plugin.tab.close", &mut state, Value::Null)
            .unwrap();
        assert!(state.plugin_host.tabs.is_empty());
        assert!(
            registry
                .execute("plugin.tab.activate", &mut state, json!({"index": 3}))
                .is_err()
        );
    }

    #[test]
    fn disabling_unregisters_commands_and_closes_views() {
        let (mut state, mut registry, mut registered) = setup(false);
        registry
            .execute("counter.open", &mut state, Value::Null)
            .unwrap();
        registry
            .execute("counter.increment", &mut state, Value::Null)
            .unwrap();
        registry
            .execute("plugin.disable", &mut state, json!({"id": "test.counter"}))
            .unwrap();
        assert!(state.plugin_host.tabs.is_empty());
        assert_eq!(state.plugin_host.tool_window, None);
        sync_plugin_commands(&mut registry, &mut registered, &mut state);
        assert!(registered.is_empty());
        assert!(matches!(
            registry.execute("counter.increment", &mut state, Value::Null),
            Err(CommandError::Unknown(_))
        ));
        registry
            .execute("plugin.enable", &mut state, json!({"id": "test.counter"}))
            .unwrap();
        sync_plugin_commands(&mut registry, &mut registered, &mut state);
        // Re-enabling starts a fresh instance.
        assert_eq!(
            registry
                .execute("counter.increment", &mut state, Value::Null)
                .unwrap(),
            json!(1)
        );
        let list = registry
            .execute("plugin.list", &mut state, Value::Null)
            .unwrap();
        assert_eq!(list["plugins"][0]["status"], "Active");
        assert!(
            registry
                .execute("plugin.enable", &mut state, json!({"id": "no.such"}))
                .is_err()
        );
    }

    #[test]
    fn failed_activation_is_reported_and_contributes_nothing() {
        let (mut state, registry, registered) = setup(true);
        assert!(registered.is_empty());
        let list = registry
            .execute("plugin.list", &mut state, Value::Null)
            .unwrap();
        assert_eq!(list["plugins"][0]["status"], "Failed");
        assert!(
            list["plugins"][0]["reason"]
                .as_str()
                .unwrap()
                .contains("boom")
        );
        assert!(state.with_plugin("test.counter", |_, _| ()).is_none());
    }

    #[test]
    fn plugin_commands_never_replace_built_ins() {
        let mut state = IdeState::default();
        state.init_plugins(
            PluginStore::in_memory(),
            vec![factory("test.counter", false)],
        );
        let mut registry = built_in_commands();
        registry
            .register(Command::new(
                spec("counter.increment", "Built-in", "Test", None, "{}"),
                |_| true,
                |_, _| Ok(json!("built-in")),
            ))
            .unwrap();
        let mut registered = Vec::new();
        sync_plugin_commands(&mut registry, &mut registered, &mut state);
        assert_eq!(registered, ["counter.open"]);
        assert!(state.status.contains("conflicts"));
        assert_eq!(
            registry
                .execute("counter.increment", &mut state, Value::Null)
                .unwrap(),
            json!("built-in")
        );
    }
}
