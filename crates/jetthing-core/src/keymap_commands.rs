//! Active keymap, IntelliJ action → Jetthing command mapping, and `keymap.*` commands.

use crate::{IdeState, spec, string_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_keymap::{Keymap, KeymapSet, Overrides, Shortcut};
use jetthing_settings::{SettingValue, ids};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Where a keymap-dispatched command applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    /// Anywhere in the workbench.
    Global,
    /// Only while the editor has keyboard focus (text fields keep their own keys).
    Editor,
}

/// An IntelliJ action id that Jetthing implements as a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionCommand {
    pub action: &'static str,
    pub command: &'static str,
    /// JSON parameters for the command.
    pub params: &'static str,
    pub scope: BindingScope,
}

const fn global(
    action: &'static str,
    command: &'static str,
    params: &'static str,
) -> ActionCommand {
    ActionCommand {
        action,
        command,
        params,
        scope: BindingScope::Global,
    }
}

const fn editor(action: &'static str, command: &'static str) -> ActionCommand {
    ActionCommand {
        action,
        command,
        params: "{}",
        scope: BindingScope::Editor,
    }
}

/// Action ids from `docs/spec/data/actions.json` mapped to typed commands. Actions not
/// listed here are shown in the keymap as unbound ("not available in Jetthing").
pub const ACTION_COMMANDS: &[ActionCommand] = &[
    global("ShowSettings", "settings.open", "{}"),
    global("CloseContent", "editor.close", "{}"),
    global("CompileDirty", "build.project", "{}"),
    global("Run", "run.project", "{}"),
    global("Stop", "run.cancel", "{}"),
    global("CheckinProject", "toolwindow.activate.commit", "{}"),
    global("HideAllWindows", "toolwindow.hideAll", "{}"),
    global(
        "ActivateProjectToolWindow",
        "toolwindow.toggle",
        r#"{"id":"project"}"#,
    ),
    global(
        "ActivateCommitToolWindow",
        "toolwindow.toggle",
        r#"{"id":"commit"}"#,
    ),
    global(
        "ActivateFindToolWindow",
        "toolwindow.toggle",
        r#"{"id":"find"}"#,
    ),
    global(
        "ActivateRunToolWindow",
        "toolwindow.toggle",
        r#"{"id":"build"}"#,
    ),
    global(
        "ActivateProblemsViewToolWindow",
        "toolwindow.toggle",
        r#"{"id":"problems"}"#,
    ),
    global(
        "ActivateTerminalToolWindow",
        "toolwindow.toggle",
        r#"{"id":"terminal"}"#,
    ),
    editor("$Undo", "editor.undo"),
    editor("$Redo", "editor.redo"),
    editor("CodeCompletion", "language.complete"),
    editor("EditorDuplicate", "editor.duplicate"),
    editor("EditorDeleteLine", "editor.deleteLine"),
    editor("CommentByLineComment", "editor.commentLine"),
    editor("MoveLineUp", "editor.moveLineUp"),
    editor("MoveLineDown", "editor.moveLineDown"),
    editor("SelectNextOccurrence", "editor.selectNextOccurrence"),
    editor("UnselectPreviousOccurrence", "editor.unselectOccurrence"),
    editor("SelectAllOccurrences", "editor.selectAllOccurrences"),
    editor(
        "EditorAddCaretPerSelectedLine",
        "editor.addCaretsToLineEnds",
    ),
    editor("EditorToggleColumnMode", "editor.toggleColumnMode"),
    editor("EditorJoinLines", "editor.joinLines"),
    editor("EditorToggleCase", "editor.toggleCase"),
    editor("EditorSplitLine", "editor.splitLine"),
    editor("EditorStartNewLineBefore", "editor.startNewLineBefore"),
];

pub fn command_for_action(action: &str) -> Option<&'static ActionCommand> {
    ACTION_COMMANDS
        .iter()
        .find(|binding| binding.action == action)
}

/// IntelliJ actions that run `command` (used for shortcut labels in menus).
pub fn actions_for_command(command: &str) -> impl Iterator<Item = &'static str> + '_ {
    ACTION_COMMANDS
        .iter()
        .filter(move |binding| binding.command == command)
        .map(|binding| binding.action)
}

/// The effective keymap: the selected bundled keymap plus the user's overrides for it.
#[derive(Debug, Clone)]
pub struct KeymapState {
    pub keymap: Keymap,
    /// Overrides of all keymaps, keyed by keymap id (persisted in `keymap.overrides`).
    pub overrides: BTreeMap<String, Overrides>,
    pub error: Option<String>,
}

impl Default for KeymapState {
    fn default() -> Self {
        let mut state = Self {
            keymap: Keymap::new("", BTreeMap::new()),
            overrides: BTreeMap::new(),
            error: None,
        };
        state.rebuild(jetthing_keymap::Desktop::detect().default_keymap());
        state
    }
}

impl KeymapState {
    fn rebuild(&mut self, id: &str) {
        let overrides = self.overrides.get(id).cloned().unwrap_or_default();
        match KeymapSet::shared().and_then(|set| set.keymap(id, &overrides)) {
            Ok(keymap) => {
                self.keymap = keymap;
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    /// Parses `keymap<TAB>action<TAB>shortcut…` lines; malformed lines are skipped and
    /// reported.
    fn parse_overrides(lines: &[String]) -> (BTreeMap<String, Overrides>, Option<String>) {
        let mut overrides: BTreeMap<String, Overrides> = BTreeMap::new();
        let mut error = None;
        for line in lines {
            let Some((keymap, rest)) = line.split_once('\t') else {
                continue;
            };
            match Overrides::from_lines(&[rest.to_owned()]) {
                Ok(parsed) => overrides
                    .entry(keymap.to_owned())
                    .or_default()
                    .actions
                    .extend(parsed.actions),
                Err(parse) => error = Some(format!("ignored keymap override: {parse}")),
            }
        }
        (overrides, error)
    }

    fn override_lines(&self) -> Vec<String> {
        self.overrides
            .iter()
            .flat_map(|(keymap, overrides)| {
                overrides
                    .to_lines()
                    .into_iter()
                    .map(move |line| format!("{keymap}\t{line}"))
            })
            .collect()
    }
}

impl IdeState {
    /// Rebuilds the keymap from the `keymap.active` and `keymap.overrides` settings.
    pub fn refresh_keymap(&mut self) {
        let id = self
            .settings
            .text(ids::KEYMAP)
            .unwrap_or_else(|| jetthing_keymap::Desktop::detect().default_keymap().into());
        let lines = match self
            .settings
            .store
            .get(&self.settings.registry, ids::KEYMAP_OVERRIDES)
        {
            Ok(SettingValue::List(lines)) => lines,
            _ => Vec::new(),
        };
        let (overrides, error) = KeymapState::parse_overrides(&lines);
        self.keymap.overrides = overrides;
        self.keymap.rebuild(&id);
        if let Some(error) = error.or_else(|| self.keymap.error.clone()) {
            self.status = error;
        }
    }

    fn save_keymap_overrides(&mut self) -> Result<(), CommandError> {
        let lines = self.keymap.override_lines();
        let crate::Settings { registry, store } = &mut self.settings;
        store
            .set(registry, ids::KEYMAP_OVERRIDES, SettingValue::List(lines))
            .and_then(|_| store.save())
            .map_err(|error| CommandError::Failed(error.to_string()))?;
        self.refresh_keymap();
        Ok(())
    }

    /// Replaces the shortcuts of `action` in the active keymap (user override).
    pub fn set_action_shortcuts(
        &mut self,
        action: &str,
        shortcuts: Vec<Shortcut>,
    ) -> Result<(), CommandError> {
        let id = self.keymap.keymap.id.clone();
        let bundled = KeymapSet::shared()
            .ok()
            .and_then(|set| set.get(&id))
            .and_then(|keymap| keymap.bindings.get(action))
            .cloned()
            .unwrap_or_default();
        let overrides = self.keymap.overrides.entry(id).or_default();
        if shortcuts == bundled {
            overrides.actions.remove(action);
        } else {
            overrides.actions.insert(action.to_owned(), shortcuts);
        }
        self.save_keymap_overrides()
    }
}

/// Same shortcut, or one is a single stroke that starts the other's chord.
fn overlaps(a: &Shortcut, b: &Shortcut) -> bool {
    match (a, b) {
        (Shortcut::Keys(a), Shortcut::Keys(b)) if a.len() != b.len() => a[0] == b[0],
        _ => a == b,
    }
}

fn shortcut_param(params: &Value) -> Result<Shortcut, CommandError> {
    string_param(params, "shortcut")?
        .parse::<Shortcut>()
        .map_err(|error| CommandError::InvalidParams(error.to_string()))
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    registry
        .register(Command::new(
            spec(
                "keymap.lookup",
                "Find Actions by Shortcut",
                "Keymap",
                None,
                r#"{"shortcut":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let shortcut = shortcut_param(&params)?;
                let actions: Vec<_> = state
                    .keymap
                    .keymap
                    .actions_for(&shortcut)
                    .iter()
                    .map(|action| {
                        json!({
                            "action": action,
                            "command": command_for_action(action).map(|binding| binding.command),
                        })
                    })
                    .collect();
                Ok(json!({"keymap": state.keymap.keymap.id, "actions": actions}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "keymap.bind",
                "Add Shortcut",
                "Keymap",
                None,
                r#"{"action":"string","shortcut":"string","removeConflicts":"bool?"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let action = string_param(&params, "action")?;
                let shortcut = shortcut_param(&params)?;
                let remove = params
                    .get("removeConflicts")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let conflicts = state.keymap.keymap.conflicts_with(&action, &shortcut);
                if remove {
                    for other in &conflicts {
                        let kept: Vec<_> = state
                            .keymap
                            .keymap
                            .shortcuts(other)
                            .iter()
                            .filter(|existing| !overlaps(existing, &shortcut))
                            .cloned()
                            .collect();
                        state.set_action_shortcuts(other, kept)?;
                    }
                }
                let mut shortcuts = state.keymap.keymap.shortcuts(&action).to_vec();
                if !shortcuts.contains(&shortcut) {
                    shortcuts.push(shortcut);
                }
                state.set_action_shortcuts(&action, shortcuts)?;
                Ok(json!({"conflicts": if remove { Vec::new() } else { conflicts }}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "keymap.unbind",
                "Remove Shortcut",
                "Keymap",
                None,
                r#"{"action":"string","shortcut":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let action = string_param(&params, "action")?;
                let shortcut = shortcut_param(&params)?;
                let shortcuts: Vec<_> = state
                    .keymap
                    .keymap
                    .shortcuts(&action)
                    .iter()
                    .filter(|existing| **existing != shortcut)
                    .cloned()
                    .collect();
                state.set_action_shortcuts(&action, shortcuts)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "keymap.reset",
                "Reset Shortcuts",
                "Keymap",
                None,
                r#"{"action":"string"}"#,
            ),
            |_| true,
            |state: &mut IdeState, params| {
                let action = string_param(&params, "action")?;
                let id = state.keymap.keymap.id.clone();
                if let Some(overrides) = state.keymap.overrides.get_mut(&id) {
                    overrides.actions.remove(&action);
                }
                state.save_keymap_overrides()?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in_commands;

    fn state_with(keymap: &str) -> IdeState {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute(
                "settings.set",
                &mut state,
                json!({"id": ids::KEYMAP, "value": keymap}),
            )
            .unwrap();
        state
    }

    #[test]
    fn every_mapped_action_and_command_exists() {
        let commands = built_in_commands();
        let specs = commands.specs();
        let set = KeymapSet::shared().unwrap();
        let windows = set.get("$default").unwrap();
        let mac = set.get("Mac OS X 10.5+").unwrap();
        for binding in ACTION_COMMANDS {
            assert!(
                specs.iter().any(|spec| spec.id == binding.command),
                "{} maps to missing command {}",
                binding.action,
                binding.command
            );
            assert!(
                windows.bindings.contains_key(binding.action)
                    || mac.bindings.contains_key(binding.action),
                "{} has no default shortcut",
                binding.action
            );
            assert!(serde_json::from_str::<Value>(binding.params).is_ok());
        }
    }

    #[test]
    fn keymap_follows_the_setting() {
        let state = state_with("Default for KDE");
        assert_eq!(state.keymap.keymap.id, "Default for KDE");
        assert_eq!(
            state.keymap.keymap.shortcuts("ReformatCode"),
            &["Alt+Shift+L".parse::<Shortcut>().unwrap()]
        );
        let state = state_with("Mac OS X 10.5+");
        assert_eq!(
            state.keymap.keymap.shortcuts("ShowSettings"),
            &["Cmd+,".parse::<Shortcut>().unwrap()]
        );
    }

    #[test]
    fn bind_unbind_reset_and_conflicts() {
        let commands = built_in_commands();
        let mut state = state_with("$default");
        // Ctrl+D is Duplicate; binding it to Settings reports the conflict.
        let result = commands
            .execute(
                "keymap.bind",
                &mut state,
                json!({"action": "ShowSettings", "shortcut": "Ctrl+D"}),
            )
            .unwrap();
        assert!(
            result["conflicts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a == "EditorDuplicate")
        );
        assert!(
            state
                .keymap
                .keymap
                .shortcuts("ShowSettings")
                .contains(&"Ctrl+D".parse().unwrap())
        );
        // Persisted as an override line for the active keymap.
        let lines = state
            .settings
            .store
            .get(&state.settings.registry, ids::KEYMAP_OVERRIDES)
            .unwrap();
        assert_eq!(
            lines,
            SettingValue::List(vec!["$default\tShowSettings\tCtrl+Alt+S\tCtrl+D".into()])
        );
        // Remove the conflicting assignment.
        commands
            .execute(
                "keymap.bind",
                &mut state,
                json!({"action": "ShowSettings", "shortcut": "Ctrl+D", "removeConflicts": true}),
            )
            .unwrap();
        assert!(state.keymap.keymap.shortcuts("EditorDuplicate").is_empty());
        let found = commands
            .execute("keymap.lookup", &mut state, json!({"shortcut": "Ctrl+D"}))
            .unwrap();
        assert_eq!(found["actions"][0]["action"], "ShowSettings");
        assert_eq!(found["actions"][0]["command"], "settings.open");
        commands
            .execute(
                "keymap.reset",
                &mut state,
                json!({"action": "EditorDuplicate"}),
            )
            .unwrap();
        commands
            .execute(
                "keymap.unbind",
                &mut state,
                json!({"action": "ShowSettings", "shortcut": "Ctrl+D"}),
            )
            .unwrap();
        assert_eq!(
            state.keymap.keymap.shortcuts("EditorDuplicate"),
            &["Ctrl+D".parse::<Shortcut>().unwrap()]
        );
        // Reset and unbind removed their overrides; the other actions that lost Ctrl+D
        // when conflicts were removed keep theirs.
        let SettingValue::List(lines) = state
            .settings
            .store
            .get(&state.settings.registry, ids::KEYMAP_OVERRIDES)
            .unwrap()
        else {
            panic!("overrides are a list");
        };
        assert!(!lines.iter().any(|line| line.contains("EditorDuplicate")));
        assert!(!lines.iter().any(|line| line.contains("ShowSettings")));
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("$default\tSendEOF"))
        );
        assert!(
            commands
                .execute(
                    "keymap.bind",
                    &mut state,
                    json!({"action": "X", "shortcut": "Ctrl+"})
                )
                .is_err()
        );
    }
}
