//! Installed manifest plugins: discovery from the bundled, user and project plugin
//! directories, enable/disable state, and `plugin.*` commands.

use crate::{IdeState, spec, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_plugins::PluginManifest;
use jetthing_settings::SettingValue;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use jetthing_settings::ids::DISABLED_PLUGINS;

/// Where a plugin was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginSource {
    /// Shipped next to the Jetthing executable (or in the source tree for dev builds).
    Bundled,
    /// `<config dir>/plugins`.
    User,
    /// `<project>/plugins`.
    Project,
}

impl PluginSource {
    pub fn label(self) -> &'static str {
        match self {
            PluginSource::Bundled => "Bundled",
            PluginSource::User => "User",
            PluginSource::Project => "Project",
        }
    }
}

/// One discovered plugin manifest (or a manifest that failed to load).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEntry {
    pub path: PathBuf,
    pub source: PluginSource,
    pub manifest: Result<PluginManifest, String>,
    pub enabled: bool,
}

impl PluginEntry {
    pub fn id(&self) -> Option<&str> {
        self.manifest.as_ref().ok().map(|m| m.id.as_str())
    }
}

/// Plugin directories in priority order; the first manifest with an id wins.
pub fn plugin_dirs(project: Option<&Path>) -> Vec<(PluginSource, PathBuf)> {
    let mut dirs = Vec::new();
    if let Some(dir) = std::env::var_os("JETTHING_PLUGINS_DIR").filter(|v| !v.is_empty()) {
        dirs.push((PluginSource::Bundled, PathBuf::from(dir)));
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("plugins")))
    {
        dirs.push((PluginSource::Bundled, dir));
    }
    // Development builds run from the source tree.
    let source_tree = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
    if source_tree.is_dir() {
        dirs.push((PluginSource::Bundled, source_tree));
    }
    if let Some(config) = jetthing_settings::default_config_dir() {
        dirs.push((PluginSource::User, config.join("plugins")));
    }
    if let Some(project) = project {
        dirs.push((PluginSource::Project, project.join("plugins")));
    }
    dirs
}

/// Discovers plugins in `dirs`, skipping duplicate ids and duplicate directories.
pub fn discover(dirs: &[(PluginSource, PathBuf)], disabled: &[String]) -> Vec<PluginEntry> {
    let mut entries: Vec<PluginEntry> = Vec::new();
    let mut seen_dirs = Vec::new();
    for (source, dir) in dirs {
        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if seen_dirs.contains(&canonical) {
            continue;
        }
        seen_dirs.push(canonical.clone());
        for (path, result) in jetthing_plugins::discover_with_paths(&canonical) {
            let manifest = result.map_err(|error| error.to_string());
            if let Ok(manifest) = &manifest
                && entries
                    .iter()
                    .any(|entry| entry.id() == Some(manifest.id.as_str()))
            {
                continue;
            }
            let enabled = manifest
                .as_ref()
                .is_ok_and(|manifest| !disabled.contains(&manifest.id));
            entries.push(PluginEntry {
                path,
                source: *source,
                manifest,
                enabled,
            });
        }
    }
    entries
}

impl IdeState {
    fn disabled_plugins(&self) -> Vec<String> {
        match self
            .settings
            .store
            .get(&self.settings.registry, DISABLED_PLUGINS)
        {
            Ok(SettingValue::List(ids)) => ids,
            _ => Vec::new(),
        }
    }

    /// Rediscovers plugins; `plugins` keeps the enabled, valid manifests.
    pub fn refresh_plugins(&mut self) {
        let root = self.project.as_ref().map(|p| p.root().to_owned());
        let dirs = plugin_dirs(root.as_deref());
        self.plugin_entries = discover(&dirs, &self.disabled_plugins());
        self.plugins = self
            .plugin_entries
            .iter()
            .filter(|entry| entry.enabled)
            .filter_map(|entry| entry.manifest.clone().ok())
            .collect();
    }
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec("plugin.refresh", "Refresh Plugins", "Plugins", None, "{}"),
            |_| true,
            |state: &mut IdeState, _| {
                state.refresh_plugins();
                let failed = state
                    .plugin_entries
                    .iter()
                    .filter(|entry| entry.manifest.is_err())
                    .count();
                Ok(json!({"count": state.plugins.len(), "failed": failed}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "plugin.setEnabled",
                "Enable or Disable Plugin",
                "Plugins",
                None,
                r#"{"id":"string","enabled":"bool"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let id = string_param(&params, "id")?;
                let enabled = params
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        CommandError::InvalidParams("expected bool field `enabled`".into())
                    })?;
                if !state
                    .plugin_entries
                    .iter()
                    .any(|entry| entry.id() == Some(&id))
                {
                    return Err(CommandError::InvalidParams(format!(
                        "unknown plugin `{id}`"
                    )));
                }
                let mut disabled = state.disabled_plugins();
                disabled.retain(|other| *other != id);
                if !enabled {
                    disabled.push(id);
                    disabled.sort();
                }
                let crate::Settings { registry, store } = &mut state.settings;
                store
                    .set(registry, DISABLED_PLUGINS, SettingValue::List(disabled))
                    .and_then(|_| store.save())
                    .map_err(|error| CommandError::Failed(error.to_string()))?;
                state.refresh_plugins();
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_plugin(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir.join(name)).unwrap();
        std::fs::write(dir.join(name).join("plugin.toml"), body).unwrap();
    }

    #[test]
    fn discovery_keeps_errors_dedupes_ids_and_respects_disabled() {
        let base = std::env::temp_dir().join(format!("jetthing-plugins-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let bundled = base.join("bundled");
        let user = base.join("user");
        write_plugin(
            &bundled,
            "rust",
            "id='dev.jetthing.rust'\nname='Rust'\nversion='0.1.0'\napi_version=1\n",
        );
        write_plugin(
            &bundled,
            "broken",
            "id='bad'\nname='Bad'\nversion='1'\napi_version=1\n",
        );
        write_plugin(
            &user,
            "rust-copy",
            "id='dev.jetthing.rust'\nname='Rust (user)'\nversion='9'\napi_version=1\n",
        );
        write_plugin(
            &user,
            "go",
            "id='dev.jetthing.go'\nname='Go'\nversion='0.2.0'\napi_version=1\n",
        );
        let dirs = vec![
            (PluginSource::Bundled, bundled.clone()),
            (PluginSource::Bundled, bundled.clone()),
            (PluginSource::User, user),
        ];
        let entries = discover(&dirs, &["dev.jetthing.go".into()]);
        assert_eq!(entries.len(), 3, "{entries:?}");
        let rust = entries
            .iter()
            .find(|e| e.id() == Some("dev.jetthing.rust"))
            .unwrap();
        assert_eq!(rust.source, PluginSource::Bundled);
        assert_eq!(rust.manifest.as_ref().unwrap().name, "Rust");
        assert!(rust.enabled);
        let go = entries
            .iter()
            .find(|e| e.id() == Some("dev.jetthing.go"))
            .unwrap();
        assert!(!go.enabled);
        let broken = entries.iter().find(|e| e.manifest.is_err()).unwrap();
        assert!(
            broken
                .manifest
                .as_ref()
                .unwrap_err()
                .contains("reverse-domain")
        );
        assert!(!broken.enabled);
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn set_enabled_persists_the_disabled_list() {
        let commands = crate::built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("plugin.refresh", &mut state, Value::Null)
            .unwrap();
        // The source tree's bundled Rust plugin is always found in tests.
        assert!(
            state
                .plugin_entries
                .iter()
                .any(|e| e.id() == Some("dev.jetthing.rust"))
        );
        commands
            .execute(
                "plugin.setEnabled",
                &mut state,
                json!({"id": "dev.jetthing.rust", "enabled": false}),
            )
            .unwrap();
        assert!(!state.plugins.iter().any(|p| p.id == "dev.jetthing.rust"));
        assert_eq!(state.disabled_plugins(), ["dev.jetthing.rust"]);
        commands
            .execute(
                "plugin.setEnabled",
                &mut state,
                json!({"id": "dev.jetthing.rust", "enabled": true}),
            )
            .unwrap();
        assert!(state.plugins.iter().any(|p| p.id == "dev.jetthing.rust"));
        assert!(
            commands
                .execute(
                    "plugin.setEnabled",
                    &mut state,
                    json!({"id": "nope.plugin", "enabled": true})
                )
                .is_err()
        );
    }
}
