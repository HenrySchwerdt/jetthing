//! The Jetthing plugin: contributes the mongo-raabit editor tab, a Databases tool window
//! with the connection tree, commands, a status widget and the standalone app view
//! (`jetthing --app dev.jetthing.mongo-raabit`).

use crate::MongoRaabit;
use jetthing_plugin_api::{
    CommandContribution, CommandError, CommandResult, Contributions, EditorContribution,
    EditorTabRequest, MenuLocation, Plugin, PluginError, PluginHost, PluginManifest,
    StatusWidgetContribution, StatusWidgetState, ToolWindowContribution, Value, ViewContext,
    ViewFrame, ViewPlacement, json,
};

pub const MAIN_VIEW: &str = "mongo-raabit.main";
pub const TREE_VIEW: &str = "mongo-raabit.tree";
const STATUS_WIDGET: &str = "mongo-raabit.status";

pub struct MongoRaabitPlugin {
    manifest: PluginManifest,
    app: Option<MongoRaabit>,
}

impl Default for MongoRaabitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MongoRaabitPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest::parse(include_str!("../jetthing-plugin.toml"))
                .expect("bundled manifest is valid"),
            app: None,
        }
    }

    /// The app state, created on first use with connections stored in the plugin's
    /// data directory.
    fn app(&mut self, host: &dyn PluginHost) -> &mut MongoRaabit {
        self.app.get_or_insert_with(|| {
            let store = host.data_dir().ok().map(|dir| dir.join("connections.json"));
            MongoRaabit::new(store)
        })
    }

    fn open_tab(host: &mut dyn PluginHost) {
        host.open_editor_tab(EditorTabRequest::new(MAIN_VIEW, "MongoDB").key("mongo-raabit"));
    }
}

impl Plugin for MongoRaabitPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn contributions(&self) -> Contributions {
        Contributions::new()
            .command(CommandContribution::new("mongoRaabit.open", "Open MongoDB Workspace").category("mongo-raabit"))
            .command(CommandContribution::new("mongoRaabit.connect", "MongoDB Connection Manager…").category("mongo-raabit"))
            .command(
                CommandContribution::new("mongoRaabit.connectUri", "Connect to MongoDB URI")
                    .category("mongo-raabit")
                    .params(r#"{"uri":"string","name":"string?"}"#),
            )
            .command(CommandContribution::new("mongoRaabit.status", "MongoDB Connection Status").category("mongo-raabit"))
            .menu(MenuLocation::Tools, "mongoRaabit.open")
            .menu(MenuLocation::Tools, "mongoRaabit.connect")
            .keybinding("mongoRaabit.connect", "Mod+Alt+M")
            .tool_window(ToolWindowContribution::new("mongo-raabit", "Databases", TREE_VIEW).icon_text("M"))
            .editor(EditorContribution::new(MAIN_VIEW, "MongoDB"))
            .status_widget(StatusWidgetContribution::new(STATUS_WIDGET).command("mongoRaabit.open"))
    }

    fn activate(&mut self, host: &mut dyn PluginHost) -> Result<(), PluginError> {
        host.require_permission(jetthing_plugin_api::Permission::Network)?;
        self.app(host);
        Ok(())
    }

    fn deactivate(&mut self, _host: &mut dyn PluginHost) {
        if let Some(app) = &mut self.app {
            let ids: Vec<_> = app.sessions.iter().map(|s| s.id).collect();
            for id in ids {
                app.disconnect(id);
            }
        }
    }

    fn execute(&mut self, command: &str, params: Value, host: &mut dyn PluginHost) -> CommandResult {
        match command {
            "mongoRaabit.open" => {
                Self::open_tab(host);
                Ok(Value::Null)
            }
            "mongoRaabit.connect" => {
                self.app(host).open_manager();
                Self::open_tab(host);
                Ok(Value::Null)
            }
            "mongoRaabit.connectUri" => {
                let uri = params
                    .get("uri")
                    .and_then(Value::as_str)
                    .ok_or_else(|| CommandError::InvalidParams("expected string field `uri`".into()))?;
                let (mut profile, password) = mongo_raabit_core::profile::ConnectionProfile::from_uri(uri)
                    .map_err(|e| CommandError::InvalidParams(e.to_string()))?;
                if let Some(name) = params.get("name").and_then(Value::as_str) {
                    profile.name = name.to_owned();
                }
                let app = self.app(host);
                if let Some(password) = password {
                    let _ = app.secrets.set(&profile.password_key(), &password);
                }
                let id = app.connect(profile);
                Self::open_tab(host);
                Ok(json!({"session": id}))
            }
            "mongoRaabit.status" => {
                let app = self.app(host);
                let sessions: Vec<Value> = app
                    .sessions
                    .iter()
                    .map(|s| {
                        json!({
                            "name": s.profile.name,
                            "connected": s.connection().is_some(),
                            "version": s.server().map(|i| i.version.clone()),
                        })
                    })
                    .collect();
                Ok(json!({"sessions": sessions, "tabs": app.tabs.len()}))
            }
            other => Err(CommandError::Unknown(other.into())),
        }
    }

    fn render_view(&mut self, view: &ViewContext, frame: &mut ViewFrame<'_>, host: &mut dyn PluginHost) {
        let Some(ui) = jetthing_plugin_ui::ui(frame) else {
            return;
        };
        let open_tab = {
            let app = self.app(host);
            match (view.view.as_str(), &view.placement) {
                (TREE_VIEW, ViewPlacement::ToolWindow { .. }) => app.tree_panel(ui),
                (TREE_VIEW | MAIN_VIEW, _) => {
                    app.ui(ui);
                    false
                }
                (other, _) => {
                    ui.label(format!("Unknown view {other}"));
                    false
                }
            }
        };
        // Opening a collection from the tool window brings the workspace tab forward.
        if open_tab {
            Self::open_tab(host);
        }
    }

    fn status_widget(&self, id: &str) -> Option<StatusWidgetState> {
        (id == STATUS_WIDGET).then(|| {
            let (connected, total) = self.app.as_ref().map_or((0, 0), |app| {
                (app.sessions.iter().filter(|s| s.connection().is_some()).count(), app.sessions.len())
            });
            StatusWidgetState {
                text: if total == 0 { "MongoDB".into() } else { format!("MongoDB · {connected}/{total}") },
                tooltip: Some("mongo-raabit connections — click to open the workspace".into()),
            }
        })
    }

    fn poll(&mut self, host: &mut dyn PluginHost) {
        if let Some(app) = &mut self.app {
            app.poll();
            if app.jobs.busy() {
                host.request_repaint();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_plugin_api::testing::RecordingHost;

    #[test]
    fn contributions_are_consistent() {
        let plugin = MongoRaabitPlugin::new();
        assert_eq!(plugin.manifest().id, crate::PLUGIN_ID);
        assert!(plugin.contributions().problems().is_empty(), "{:?}", plugin.contributions().problems());
        assert!(plugin.manifest().app.is_some());
    }

    #[test]
    fn commands_open_views_and_validate_params() {
        let mut plugin = MongoRaabitPlugin::new();
        let mut host = RecordingHost::new(crate::PLUGIN_ID);
        plugin.execute("mongoRaabit.connect", Value::Null, &mut host).unwrap();
        assert_eq!(host.editor_tabs[0].view, MAIN_VIEW);
        assert!(plugin.app.as_ref().unwrap().manager.is_some());
        assert!(matches!(
            plugin.execute("mongoRaabit.connectUri", json!({}), &mut host),
            Err(CommandError::InvalidParams(_))
        ));
        assert!(matches!(
            plugin.execute("mongoRaabit.connectUri", json!({"uri": "http://nope"}), &mut host),
            Err(CommandError::InvalidParams(_))
        ));
        let status = plugin.execute("mongoRaabit.status", Value::Null, &mut host).unwrap();
        assert_eq!(status["sessions"], json!([]));
        assert_eq!(plugin.status_widget(STATUS_WIDGET).unwrap().text, "MongoDB");
    }

    #[test]
    fn views_render_in_egui() {
        let mut plugin = MongoRaabitPlugin::new();
        let mut host = RecordingHost::new(crate::PLUGIN_ID);
        let ctx = egui::Context::default();
        for placement in [
            ViewPlacement::App,
            ViewPlacement::ToolWindow { id: "mongo-raabit".into() },
        ] {
            let view = ViewContext {
                view: if matches!(placement, ViewPlacement::App) { MAIN_VIEW.into() } else { TREE_VIEW.into() },
                placement,
            };
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    plugin.render_view(&view, &mut ViewFrame::new(ui), &mut host);
                });
            });
        }
        assert!(plugin.app.is_some());
    }
}
