//! Keymap commands: list, resolve and inspect shortcut bindings, switch the
//! active keymap and load user keymaps from TOML.

use std::path::Path;

use jetthing_command::{Command, CommandError, CommandRegistry, CommandSpec};
use jetthing_keymap::{
    KeyContext, KeySequence, KeymapDiagnostic, KeymapFile, KeymapSet, Platform, ResolvedBinding,
    ResolvedKeymap, builtin_keymaps, default_keymap_name,
};
use serde_json::{Value, json};

use crate::IdeState;

/// Keymaps known to the IDE and the one currently active.
#[derive(Debug, Clone)]
pub struct KeymapState {
    set: KeymapSet,
    active: String,
    resolved: ResolvedKeymap,
    diagnostics: Vec<KeymapDiagnostic>,
    revision: u64,
}

impl Default for KeymapState {
    fn default() -> Self {
        Self::new(Platform::host())
    }
}

impl KeymapState {
    pub fn new(platform: Platform) -> Self {
        // The built-in data is compiled in and covered by tests; fall back to
        // an empty keymap rather than panicking if it were ever malformed.
        let set = builtin_keymaps().unwrap_or_default();
        let active = default_keymap_name(platform).to_owned();
        let (resolved, diagnostics) = set
            .resolve(&active)
            .unwrap_or_else(|_| (ResolvedKeymap::empty(&active, platform), Vec::new()));
        Self {
            set,
            active,
            resolved,
            diagnostics,
            revision: 0,
        }
    }

    pub fn active_name(&self) -> &str {
        &self.active
    }

    pub fn active(&self) -> &ResolvedKeymap {
        &self.resolved
    }

    pub fn diagnostics(&self) -> &[KeymapDiagnostic] {
        &self.diagnostics
    }

    /// Increments whenever the active keymap changes, so front ends can
    /// rebuild their matchers.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn names(&self) -> Vec<String> {
        self.set.names()
    }

    pub fn resolve_named(&self, name: &str) -> Result<ResolvedKeymap, String> {
        self.set
            .resolve(name)
            .map(|(keymap, _)| keymap)
            .map_err(|e| e.to_string())
    }

    pub fn select(&mut self, name: &str) -> Result<(), String> {
        let (resolved, diagnostics) = self.set.resolve(name).map_err(|e| e.to_string())?;
        self.active = name.to_owned();
        self.resolved = resolved;
        self.diagnostics = diagnostics;
        self.revision += 1;
        Ok(())
    }

    /// Adds a user keymap and makes it active. Built-in keymaps cannot be replaced.
    pub fn load_user(&mut self, file: KeymapFile) -> Result<Vec<KeymapDiagnostic>, String> {
        if builtin_keymaps()
            .map(|b| b.get(&file.name).is_some())
            .unwrap_or(false)
        {
            return Err(format!(
                "`{}` is a built-in keymap; give the user keymap another name",
                file.name
            ));
        }
        let name = file.name.clone();
        let previous = self.set.get(&name).cloned();
        self.set.insert(file);
        if let Err(error) = self.select(&name) {
            // Roll back so a broken file never leaves a dangling keymap.
            match previous {
                Some(previous) => self.set.insert(previous),
                None => {
                    self.set.remove(&name);
                }
            }
            return Err(error);
        }
        Ok(self.diagnostics.clone())
    }
}

fn binding_json(binding: &ResolvedBinding, platform: Platform) -> Value {
    json!({
        "target": binding.target,
        "label": binding.label,
        "category": binding.category,
        "context": binding.context.as_str(),
        "command": binding.command,
        "unbound": binding.command.is_none(),
        "keys": binding.keys.iter().map(KeySequence::canonical).collect::<Vec<_>>(),
        "shortcuts": binding.keys.iter().map(|k| k.label(platform)).collect::<Vec<_>>(),
        "mouse": binding.mouse.iter().map(|m| m.label(platform)).collect::<Vec<_>>(),
        "definedIn": binding.defined_in,
    })
}

fn optional_string(params: &Value, name: &str) -> Result<Option<String>, CommandError> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(CommandError::InvalidParams(format!(
            "expected string field `{name}`"
        ))),
    }
}

fn required_string(params: &Value, name: &str) -> Result<String, CommandError> {
    optional_string(params, name)?
        .ok_or_else(|| CommandError::InvalidParams(format!("expected string field `{name}`")))
}

fn contexts_param(params: &Value) -> Result<Vec<KeyContext>, CommandError> {
    let Some(value) = params.get("contexts") else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| CommandError::InvalidParams("expected array field `contexts`".into()))?
        .iter()
        .map(|item| {
            item.as_str().and_then(KeyContext::parse).ok_or_else(|| {
                CommandError::InvalidParams(format!(
                    "unknown context {item}; expected one of global, editor, debugger, diff, \
                     toolWindow, projectView, changesView"
                ))
            })
        })
        .collect()
}

fn keymap_for(state: &IdeState, params: &Value) -> Result<ResolvedKeymap, CommandError> {
    match optional_string(params, "keymap")? {
        Some(name) => state
            .keymaps
            .resolve_named(&name)
            .map_err(CommandError::Failed),
        None => Ok(state.keymaps.active().clone()),
    }
}

fn spec(id: &str, label: &str, params: &str) -> CommandSpec {
    CommandSpec {
        id: id.into(),
        label: label.into(),
        category: "Keymap".into(),
        shortcut: None,
        params: params.into(),
    }
}

/// Registers the `keymap.*` commands.
pub fn register(registry: &mut CommandRegistry<IdeState>) -> Result<(), CommandError> {
    registry.register(Command::new(
        spec(
            "keymap.list",
            "List Keymap Bindings",
            r#"{"keymap":"string?","includeUnbound":true}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let keymap = keymap_for(state, &params)?;
            let include_unbound = params
                .get("includeUnbound")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let bindings: Vec<Value> = keymap
                .bindings()
                .iter()
                .filter(|b| include_unbound || b.command.is_some())
                .map(|b| binding_json(b, keymap.platform))
                .collect();
            Ok(json!({
                "keymap": keymap.name,
                "platform": keymap.platform,
                "chain": keymap.chain,
                "available": state.keymaps.names(),
                "bindings": bindings,
            }))
        },
    ))?;
    registry.register(Command::new(
        spec(
            "keymap.resolve",
            "Resolve Shortcut",
            r#"{"keys":"string","contexts":["editor"],"keymap":"string?"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let keymap = keymap_for(state, &params)?;
            let text = required_string(&params, "keys")?;
            let keys = KeySequence::parse(&text, keymap.platform)
                .map_err(|e| CommandError::InvalidParams(e.to_string()))?;
            let contexts = contexts_param(&params)?;
            let lookup = keymap.lookup(&keys, &contexts);
            let matches: Vec<Value> = lookup
                .exact
                .iter()
                .filter_map(|&i| keymap.binding(i))
                .map(|b| binding_json(b, keymap.platform))
                .collect();
            Ok(json!({
                "keys": keys.canonical(),
                "shortcut": keys.label(keymap.platform),
                "binding": matches.first(),
                "matches": matches,
                "isPrefix": lookup.is_prefix,
            }))
        },
    ))?;
    registry.register(Command::new(
        spec(
            "keymap.conflicts",
            "Show Keymap Conflicts",
            r#"{"keymap":"string?"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let keymap = keymap_for(state, &params)?;
            Ok(json!({"keymap": keymap.name, "conflicts": keymap.conflicts()}))
        },
    ))?;
    registry.register(Command::new(
        spec("keymap.select", "Select Keymap", r#"{"name":"string"}"#),
        |_| true,
        |state: &mut IdeState, params| {
            let name = required_string(&params, "name")?;
            state.keymaps.select(&name).map_err(CommandError::Failed)?;
            state.status = format!("Keymap: {name}");
            Ok(json!({"keymap": name, "diagnostics": state.keymaps.diagnostics()}))
        },
    ))?;
    registry.register(Command::new(
        spec(
            "keymap.loadUser",
            "Load User Keymap",
            r#"{"path":"string"}"#,
        ),
        |_| true,
        |state: &mut IdeState, params| {
            let path = required_string(&params, "path")?;
            let text = std::fs::read_to_string(Path::new(&path)).map_err(|e| {
                CommandError::Failed(format!("could not read keymap file {path}: {e}"))
            })?;
            let file = KeymapFile::from_toml_str(&text)
                .map_err(|e| CommandError::InvalidParams(e.to_string()))?;
            let name = file.name.clone();
            let diagnostics = state
                .keymaps
                .load_user(file)
                .map_err(CommandError::Failed)?;
            state.status = format!("Keymap: {name}");
            Ok(json!({"keymap": name, "diagnostics": diagnostics}))
        },
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{IdeState, built_in_commands};
    use serde_json::json;

    #[test]
    fn every_mapped_command_id_is_registered() {
        let commands = built_in_commands();
        let ids: Vec<String> = commands.specs().into_iter().map(|s| s.id).collect();
        let catalog = jetthing_keymap::defaults::action_catalog().unwrap();
        for action in catalog.iter() {
            if let Some(command) = &action.command {
                assert!(ids.contains(command), "{} -> {command}", action.id);
            }
        }
    }

    #[test]
    fn list_and_resolve_commands_report_bindings() {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        let list = commands
            .execute(
                "keymap.list",
                &mut state,
                json!({"keymap": "Windows/Linux", "includeUnbound": false}),
            )
            .unwrap();
        let bindings = list["bindings"].as_array().unwrap();
        assert!(bindings.iter().all(|b| b["unbound"] == false));
        assert!(bindings.iter().any(|b| b["command"] == "editor.saveAll"));

        let resolved = commands
            .execute(
                "keymap.resolve",
                &mut state,
                json!({"keymap": "macOS", "keys": "shift shift"}),
            )
            .unwrap();
        assert_eq!(resolved["binding"]["target"]["id"], "SearchEverywhere");
        assert_eq!(resolved["binding"]["unbound"], true);
        assert_eq!(resolved["shortcut"], "Double ⇧");

        let in_editor = commands
            .execute(
                "keymap.resolve",
                &mut state,
                json!({"keymap": "Windows/Linux", "keys": "Ctrl+D", "contexts": ["editor"]}),
            )
            .unwrap();
        assert_eq!(in_editor["binding"]["target"]["id"], "EditorDuplicate");

        let error = commands
            .execute("keymap.resolve", &mut state, json!({"keys": "ctrl BOGUS"}))
            .unwrap_err();
        assert!(error.to_string().contains("BOGUS"));
        assert!(
            commands
                .execute(
                    "keymap.resolve",
                    &mut state,
                    json!({"keys": "ctrl A", "contexts": ["nowhere"]}),
                )
                .is_err()
        );
    }

    #[test]
    fn user_keymap_loads_selects_and_rolls_back_on_error() {
        let root = std::env::temp_dir().join(format!("jetthing-keymap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let good = root.join("mine.toml");
        std::fs::write(
            &good,
            "name = \"Mine\"\nparent = \"Windows/Linux\"\n\n[[binding]]\naction = \"SaveAll\"\nkeys = [\"ctrl K, ctrl S\"]\n",
        )
        .unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        let before = state.keymaps.revision();
        commands
            .execute("keymap.loadUser", &mut state, json!({"path": good}))
            .unwrap();
        assert_eq!(state.keymaps.active_name(), "Mine");
        assert!(state.keymaps.revision() > before);
        let resolved = commands
            .execute(
                "keymap.resolve",
                &mut state,
                json!({"keys": "ctrl K, ctrl S"}),
            )
            .unwrap();
        assert_eq!(resolved["binding"]["command"], "editor.saveAll");
        let conflicts = commands
            .execute("keymap.conflicts", &mut state, json!({}))
            .unwrap();
        assert!(
            conflicts["conflicts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["kind"] == "prefix")
        );

        let cyclic = root.join("cyclic.toml");
        std::fs::write(&cyclic, "name = \"Loop\"\nparent = \"Loop\"\n").unwrap();
        assert!(
            commands
                .execute("keymap.loadUser", &mut state, json!({"path": cyclic}))
                .is_err()
        );
        assert_eq!(state.keymaps.active_name(), "Mine");
        assert!(!state.keymaps.names().contains(&"Loop".to_owned()));

        let builtin = root.join("builtin.toml");
        std::fs::write(&builtin, "name = \"macOS\"\nplatform = \"macos\"\n").unwrap();
        assert!(
            commands
                .execute("keymap.loadUser", &mut state, json!({"path": builtin}))
                .is_err()
        );
        assert!(
            commands
                .execute(
                    "keymap.loadUser",
                    &mut state,
                    json!({"path": root.join("missing.toml")})
                )
                .is_err()
        );
        commands
            .execute("keymap.select", &mut state, json!({"name": "macOS"}))
            .unwrap();
        assert_eq!(state.keymaps.active().name, "macOS");
        std::fs::remove_dir_all(root).unwrap();
    }
}
