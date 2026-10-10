//! Keymap files, inheritance, user overrides and conflict detection.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::keystroke::{KeyParseError, KeySequence, MouseShortcut, Platform};

/// Where a binding is active. Specific contexts take priority over `Global`.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum KeyContext {
    #[default]
    Global,
    Editor,
    Debugger,
    Diff,
    ToolWindow,
    ProjectView,
    ChangesView,
}

impl KeyContext {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Editor => "editor",
            Self::Debugger => "debugger",
            Self::Diff => "diff",
            Self::ToolWindow => "toolWindow",
            Self::ProjectView => "projectView",
            Self::ChangesView => "changesView",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "global" => Self::Global,
            "editor" => Self::Editor,
            "debugger" => Self::Debugger,
            "diff" => Self::Diff,
            "toolWindow" => Self::ToolWindow,
            "projectView" => Self::ProjectView,
            "changesView" => Self::ChangesView,
            _ => return None,
        })
    }

    /// Two contexts overlap when they can be active at the same time for the
    /// same key press: identical contexts, or either one is `Global`.
    pub fn overlaps(self, other: Self) -> bool {
        self == other || self == Self::Global || other == Self::Global
    }
}

/// One entry of the action catalog: an IntelliJ action ID and the jetthing
/// command it maps to, or an explicit unbound marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSpec {
    /// IntelliJ action ID, e.g. `GotoClass`.
    pub id: String,
    /// Label as shown in the public keymap reference.
    pub label: String,
    /// Section of the public keymap reference.
    pub category: String,
    #[serde(default)]
    pub context: KeyContext,
    /// Jetthing command ID this action dispatches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Explicit marker: no jetthing command implements this action yet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unbound: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    #[serde(default)]
    action: Vec<ActionSpec>,
}

/// The set of known actions.
#[derive(Debug, Clone, Default)]
pub struct ActionCatalog {
    actions: BTreeMap<String, ActionSpec>,
}

impl ActionCatalog {
    pub fn from_toml_str(text: &str) -> Result<Self, KeymapError> {
        let file: CatalogFile = toml::from_str(text).map_err(|e| KeymapError::Toml {
            source_name: "action catalog".into(),
            message: e.to_string(),
        })?;
        let mut catalog = Self::default();
        for action in file.action {
            match (&action.command, action.unbound) {
                (Some(_), true) => {
                    return Err(KeymapError::InvalidAction {
                        action: action.id,
                        reason: "has both `command` and `unbound = true`".into(),
                    });
                }
                (None, false) => {
                    return Err(KeymapError::InvalidAction {
                        action: action.id,
                        reason: "needs either `command` or the `unbound = true` marker".into(),
                    });
                }
                _ => {}
            }
            let id = action.id.clone();
            if catalog.actions.insert(id.clone(), action).is_some() {
                return Err(KeymapError::InvalidAction {
                    action: id,
                    reason: "is declared twice".into(),
                });
            }
        }
        Ok(catalog)
    }

    pub fn get(&self, id: &str) -> Option<&ActionSpec> {
        self.actions.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ActionSpec> {
        self.actions.values()
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// One binding in a keymap file. Exactly one of `action` / `command` is set.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingSpec {
    /// IntelliJ action ID from the catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Direct jetthing command ID (for actions IntelliJ does not have).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Keyboard shortcuts. An empty list in a child keymap removes the
    /// parent's shortcuts for this target.
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mouse: Vec<String>,
    /// Overrides the catalog context (required for direct commands that are
    /// not global).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<KeyContext>,
}

/// A keymap as stored on disk (built-in data file or user keymap).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeymapFile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Platform the shortcuts are written for; inherited from the parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
    /// Public reference the data was transcribed from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, rename = "binding", skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BindingSpec>,
}

impl KeymapFile {
    pub fn from_toml_str(text: &str) -> Result<Self, KeymapError> {
        let file: Self = toml::from_str(text).map_err(|e| KeymapError::Toml {
            source_name: "keymap".into(),
            message: e.to_string(),
        })?;
        if file.name.trim().is_empty() {
            return Err(KeymapError::Invalid(
                "keymap `name` must not be empty".into(),
            ));
        }
        Ok(file)
    }

    pub fn to_toml_string(&self) -> Result<String, KeymapError> {
        toml::to_string_pretty(self).map_err(|e| KeymapError::Toml {
            source_name: self.name.clone(),
            message: e.to_string(),
        })
    }

    /// Creates an empty user keymap that inherits from `parent`.
    pub fn derived(name: impl Into<String>, parent: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: Some(parent.into()),
            ..Self::default()
        }
    }

    /// Replaces the shortcuts of `target` in this keymap (IntelliJ semantics:
    /// a child keymap's entry for an action replaces the inherited list).
    pub fn set_keys(&mut self, target: &BindingTarget, keys: Vec<String>) {
        let (action, command) = target.to_fields();
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .find(|b| b.action == action && b.command == command)
        {
            binding.keys = keys;
            return;
        }
        self.bindings.push(BindingSpec {
            action,
            command,
            keys,
            ..BindingSpec::default()
        });
    }
}

/// What a binding triggers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum BindingTarget {
    Action(String),
    Command(String),
}

impl BindingTarget {
    pub fn id(&self) -> &str {
        match self {
            Self::Action(id) | Self::Command(id) => id,
        }
    }

    fn to_fields(&self) -> (Option<String>, Option<String>) {
        match self {
            Self::Action(id) => (Some(id.clone()), None),
            Self::Command(id) => (None, Some(id.clone())),
        }
    }
}

/// A fully inherited, parsed binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBinding {
    pub target: BindingTarget,
    pub label: String,
    pub category: String,
    pub context: KeyContext,
    /// Jetthing command dispatched by this binding, `None` when unbound.
    pub command: Option<String>,
    pub keys: Vec<KeySequence>,
    pub mouse: Vec<MouseShortcut>,
    /// Name of the keymap in the inheritance chain that set these shortcuts.
    pub defined_in: String,
}

/// Non-fatal problems found while resolving a keymap.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum KeymapDiagnostic {
    #[error("keymap `{keymap}`: unknown action `{action}`")]
    UnknownAction { keymap: String, action: String },
    #[error("keymap `{keymap}`: {target}: {message}")]
    InvalidShortcut {
        keymap: String,
        target: String,
        message: String,
    },
    #[error("keymap `{keymap}`: binding must set exactly one of `action` or `command`")]
    InvalidBinding { keymap: String },
}

/// Fatal keymap errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KeymapError {
    #[error("could not parse {source_name}: {message}")]
    Toml {
        source_name: String,
        message: String,
    },
    #[error("unknown keymap `{0}`")]
    UnknownKeymap(String),
    #[error("keymap inheritance cycle: {}", .0.join(" -> "))]
    InheritanceCycle(Vec<String>),
    #[error("keymap `{0}` has no platform and no parent to inherit one from")]
    MissingPlatform(String),
    #[error("action `{action}` {reason}")]
    InvalidAction { action: String, reason: String },
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Parse(#[from] KeyParseError),
}

/// Kinds of shortcut conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictKind {
    /// Same sequence, same context, different targets: resolution is ambiguous.
    Ambiguous,
    /// Same sequence; a context-specific binding hides a global one there.
    Shadowed,
    /// One sequence is the first stroke of another (chord) in an overlapping
    /// context; the shorter one fires only after the chord times out.
    Prefix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Conflict {
    pub kind: ConflictKind,
    /// The shorter (or equal) sequence, canonical form.
    pub keys: String,
    pub first: BindingTarget,
    pub first_context: KeyContext,
    /// The longer sequence for `Prefix` conflicts, equal to `keys` otherwise.
    pub other_keys: String,
    pub second: BindingTarget,
    pub second_context: KeyContext,
}

/// Result of looking a sequence up in a keymap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lookup {
    /// Bindings matching exactly in an active context, best first.
    pub exact: Vec<usize>,
    /// True when some active binding has this sequence as a strict prefix.
    pub is_prefix: bool,
}

/// A keymap with inheritance applied and every shortcut parsed.
#[derive(Debug, Clone)]
pub struct ResolvedKeymap {
    pub name: String,
    pub platform: Platform,
    /// Inheritance chain from root to this keymap.
    pub chain: Vec<String>,
    bindings: Vec<ResolvedBinding>,
    by_sequence: HashMap<KeySequence, Vec<usize>>,
    prefixes: HashMap<KeySequence, Vec<usize>>,
}

impl ResolvedKeymap {
    fn new(
        name: String,
        platform: Platform,
        chain: Vec<String>,
        bindings: Vec<ResolvedBinding>,
    ) -> Self {
        let mut keymap = Self {
            name,
            platform,
            chain,
            bindings,
            by_sequence: HashMap::new(),
            prefixes: HashMap::new(),
        };
        keymap.reindex();
        keymap
    }

    /// A keymap with no bindings.
    pub fn empty(name: impl Into<String>, platform: Platform) -> Self {
        let name = name.into();
        Self::new(name.clone(), platform, vec![name], Vec::new())
    }

    fn reindex(&mut self) {
        self.by_sequence.clear();
        self.prefixes.clear();
        for (index, binding) in self.bindings.iter().enumerate() {
            for keys in &binding.keys {
                self.by_sequence
                    .entry(keys.clone())
                    .or_default()
                    .push(index);
                for len in 1..keys.len() {
                    if let Ok(prefix) = KeySequence::new(keys.strokes()[..len].to_vec()) {
                        self.prefixes.entry(prefix).or_default().push(index);
                    }
                }
            }
        }
    }

    pub fn bindings(&self) -> &[ResolvedBinding] {
        &self.bindings
    }

    pub fn binding(&self, index: usize) -> Option<&ResolvedBinding> {
        self.bindings.get(index)
    }

    pub fn find(&self, target: &BindingTarget) -> Option<&ResolvedBinding> {
        self.bindings.iter().find(|b| &b.target == target)
    }

    pub fn find_action(&self, action: &str) -> Option<&ResolvedBinding> {
        self.find(&BindingTarget::Action(action.into()))
    }

    /// Bindings that dispatch a jetthing command; unbound actions are dropped.
    pub fn dispatchable(&self) -> Self {
        Self::new(
            self.name.clone(),
            self.platform,
            self.chain.clone(),
            self.bindings
                .iter()
                .filter(|b| b.command.is_some())
                .cloned()
                .collect(),
        )
    }

    /// Looks up `keys` given the currently active contexts, most specific
    /// first. `Global` is always active.
    pub fn lookup(&self, keys: &KeySequence, active: &[KeyContext]) -> Lookup {
        let rank = |context: KeyContext| -> Option<usize> {
            if context == KeyContext::Global {
                Some(usize::MAX)
            } else {
                active.iter().position(|c| *c == context)
            }
        };
        let mut exact: Vec<(usize, usize)> = self
            .by_sequence
            .get(keys)
            .into_iter()
            .flatten()
            .filter_map(|&i| rank(self.bindings[i].context).map(|r| (r, i)))
            .collect();
        exact.sort();
        let is_prefix = self
            .prefixes
            .get(keys)
            .into_iter()
            .flatten()
            .any(|&i| rank(self.bindings[i].context).is_some());
        Lookup {
            exact: exact.into_iter().map(|(_, i)| i).collect(),
            is_prefix,
        }
    }

    /// The binding that would fire for `keys` in `active` contexts.
    pub fn resolve(&self, keys: &KeySequence, active: &[KeyContext]) -> Option<&ResolvedBinding> {
        self.lookup(keys, active)
            .exact
            .first()
            .and_then(|&i| self.bindings.get(i))
    }

    /// All shortcut conflicts, deterministic order.
    pub fn conflicts(&self) -> Vec<Conflict> {
        let mut found = BTreeSet::new();
        let mut out = Vec::new();
        let mut push = |kind: ConflictKind,
                        short: &KeySequence,
                        a: &ResolvedBinding,
                        long: &KeySequence,
                        b: &ResolvedBinding| {
            let (first, second, short_ctx, long_ctx) =
                if kind != ConflictKind::Prefix && b.target < a.target {
                    (b, a, b.context, a.context)
                } else {
                    (a, b, a.context, b.context)
                };
            let key = (
                kind,
                short.canonical(),
                long.canonical(),
                first.target.clone(),
                second.target.clone(),
            );
            if found.insert(key) {
                out.push(Conflict {
                    kind,
                    keys: short.canonical(),
                    first: first.target.clone(),
                    first_context: short_ctx,
                    other_keys: long.canonical(),
                    second: second.target.clone(),
                    second_context: long_ctx,
                });
            }
        };
        let mut sequences: Vec<&KeySequence> = self.by_sequence.keys().collect();
        sequences.sort();
        for keys in sequences {
            let indices = &self.by_sequence[keys];
            for (n, &i) in indices.iter().enumerate() {
                for &j in &indices[n + 1..] {
                    let (a, b) = (&self.bindings[i], &self.bindings[j]);
                    if a.target == b.target || !a.context.overlaps(b.context) {
                        continue;
                    }
                    let kind = if a.context == b.context {
                        ConflictKind::Ambiguous
                    } else {
                        ConflictKind::Shadowed
                    };
                    push(kind, keys, a, keys, b);
                }
            }
            if let Some(longer) = self.prefixes.get(keys) {
                for &i in indices {
                    for &j in longer {
                        let (a, b) = (&self.bindings[i], &self.bindings[j]);
                        if !a.context.overlaps(b.context) {
                            continue;
                        }
                        for long in b.keys.iter().filter(|k| keys.is_strict_prefix_of(k)) {
                            push(ConflictKind::Prefix, keys, a, long, b);
                        }
                    }
                }
            }
        }
        out.sort_by(|x, y| {
            (x.kind, &x.keys, &x.first, &x.second).cmp(&(y.kind, &y.keys, &y.first, &y.second))
        });
        out
    }

    /// Bindings already using `keys` in a context overlapping `context`; what
    /// a keymap editor should warn about before assigning a shortcut.
    pub fn conflicts_for(&self, keys: &KeySequence, context: KeyContext) -> Vec<&ResolvedBinding> {
        self.bindings
            .iter()
            .filter(|b| b.context.overlaps(context))
            .filter(|b| {
                b.keys.iter().any(|k| {
                    k == keys || k.is_strict_prefix_of(keys) || keys.is_strict_prefix_of(k)
                })
            })
            .collect()
    }
}

/// A named collection of keymap files that can inherit from each other.
#[derive(Debug, Clone, Default)]
pub struct KeymapSet {
    catalog: ActionCatalog,
    files: BTreeMap<String, KeymapFile>,
}

impl KeymapSet {
    pub fn new(catalog: ActionCatalog) -> Self {
        Self {
            catalog,
            files: BTreeMap::new(),
        }
    }

    pub fn catalog(&self) -> &ActionCatalog {
        &self.catalog
    }

    /// Adds or replaces a keymap.
    pub fn insert(&mut self, file: KeymapFile) {
        self.files.insert(file.name.clone(), file);
    }

    pub fn remove(&mut self, name: &str) -> Option<KeymapFile> {
        self.files.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&KeymapFile> {
        self.files.get(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }

    /// Root-first inheritance chain of `name`.
    pub fn chain(&self, name: &str) -> Result<Vec<&KeymapFile>, KeymapError> {
        let mut chain = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        let mut current = Some(name.to_owned());
        while let Some(name) = current {
            if seen.contains(&name) {
                seen.push(name);
                return Err(KeymapError::InheritanceCycle(seen));
            }
            let file = self
                .files
                .get(&name)
                .ok_or_else(|| KeymapError::UnknownKeymap(name.clone()))?;
            seen.push(name);
            chain.push(file);
            current = file.parent.clone();
        }
        chain.reverse();
        Ok(chain)
    }

    /// Applies inheritance and parses every shortcut. Invalid shortcuts and
    /// unknown actions are reported as diagnostics and skipped, so one bad
    /// line in a user keymap never disables the whole keymap.
    pub fn resolve(
        &self,
        name: &str,
    ) -> Result<(ResolvedKeymap, Vec<KeymapDiagnostic>), KeymapError> {
        let chain = self.chain(name)?;
        let platform = chain
            .iter()
            .rev()
            .find_map(|f| f.platform)
            .ok_or_else(|| KeymapError::MissingPlatform(name.into()))?;
        let mut diagnostics = Vec::new();
        // target -> (spec, defined_in); later keymaps replace earlier ones.
        let mut merged: BTreeMap<BindingTarget, (BindingSpec, String)> = BTreeMap::new();
        for file in &chain {
            for spec in &file.bindings {
                let target = match (&spec.action, &spec.command) {
                    (Some(action), None) => BindingTarget::Action(action.clone()),
                    (None, Some(command)) => BindingTarget::Command(command.clone()),
                    _ => {
                        diagnostics.push(KeymapDiagnostic::InvalidBinding {
                            keymap: file.name.clone(),
                        });
                        continue;
                    }
                };
                if let BindingTarget::Action(action) = &target
                    && self.catalog.get(action).is_none()
                {
                    diagnostics.push(KeymapDiagnostic::UnknownAction {
                        keymap: file.name.clone(),
                        action: action.clone(),
                    });
                    continue;
                }
                match merged.get_mut(&target) {
                    // The same keymap may list an action twice (e.g. documented in
                    // two sections); accumulate instead of replacing.
                    Some((existing, defined_in)) if *defined_in == file.name => {
                        for key in &spec.keys {
                            if !existing.keys.contains(key) {
                                existing.keys.push(key.clone());
                            }
                        }
                        existing.mouse.extend(spec.mouse.iter().cloned());
                    }
                    _ => {
                        merged.insert(target, (spec.clone(), file.name.clone()));
                    }
                }
            }
        }
        let mut bindings = Vec::new();
        for (target, (spec, defined_in)) in merged {
            let action = match &target {
                BindingTarget::Action(id) => self.catalog.get(id),
                BindingTarget::Command(_) => None,
            };
            let mut keys = Vec::new();
            for text in &spec.keys {
                match KeySequence::parse(text, platform) {
                    Ok(seq) if !keys.contains(&seq) => keys.push(seq),
                    Ok(_) => {}
                    Err(error) => diagnostics.push(KeymapDiagnostic::InvalidShortcut {
                        keymap: defined_in.clone(),
                        target: target.id().into(),
                        message: error.to_string(),
                    }),
                }
            }
            let mut mouse = Vec::new();
            for text in &spec.mouse {
                match MouseShortcut::parse(text, platform) {
                    Ok(m) => mouse.push(m),
                    Err(error) => diagnostics.push(KeymapDiagnostic::InvalidShortcut {
                        keymap: defined_in.clone(),
                        target: target.id().into(),
                        message: error.to_string(),
                    }),
                }
            }
            if keys.is_empty() && mouse.is_empty() {
                // Explicitly removed in a child keymap; keep it out of lookups.
                continue;
            }
            bindings.push(ResolvedBinding {
                label: action
                    .map(|a| a.label.clone())
                    .unwrap_or_else(|| target.id().to_owned()),
                category: action
                    .map(|a| a.category.clone())
                    .unwrap_or_else(|| "Custom".into()),
                context: spec
                    .context
                    .or(action.map(|a| a.context))
                    .unwrap_or_default(),
                command: match &target {
                    BindingTarget::Action(_) => action.and_then(|a| a.command.clone()),
                    BindingTarget::Command(id) => Some(id.clone()),
                },
                target,
                keys,
                mouse,
                defined_in,
            });
        }
        let chain_names = chain.iter().map(|f| f.name.clone()).collect();
        Ok((
            ResolvedKeymap::new(name.into(), platform, chain_names, bindings),
            diagnostics,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOG: &str = r#"
[[action]]
id = "GotoClass"
label = "Go to Class..."
category = "Find everything"
unbound = true

[[action]]
id = "SaveAll"
label = "Save All"
category = "Basic editing"
command = "editor.saveAll"

[[action]]
id = "EditorDuplicate"
label = "Duplicate Line or Selection"
category = "Basic editing"
context = "editor"
unbound = true

[[action]]
id = "CommentByLineComment"
label = "Comment with Line Comment"
category = "Coding assistance"
context = "editor"
unbound = true
"#;

    fn set() -> KeymapSet {
        let mut set = KeymapSet::new(ActionCatalog::from_toml_str(CATALOG).unwrap());
        set.insert(
            KeymapFile::from_toml_str(
                r#"
name = "Base"
platform = "linux"
[[binding]]
action = "GotoClass"
keys = ["ctrl N"]
[[binding]]
action = "SaveAll"
keys = ["ctrl S"]
[[binding]]
action = "EditorDuplicate"
keys = ["ctrl D"]
"#,
            )
            .unwrap(),
        );
        set
    }

    #[test]
    fn child_keymap_overrides_and_removes_inherited_shortcuts() {
        let mut set = set();
        let mut user = KeymapFile::derived("Mine", "Base");
        user.set_keys(
            &BindingTarget::Action("GotoClass".into()),
            vec!["ctrl alt N".into()],
        );
        user.set_keys(&BindingTarget::Action("SaveAll".into()), vec![]);
        user.set_keys(
            &BindingTarget::Command("terminal.toggle".into()),
            vec!["ctrl BACK_QUOTE".into()],
        );
        set.insert(user);
        let (keymap, diagnostics) = set.resolve("Mine").unwrap();
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(keymap.chain, vec!["Base", "Mine"]);
        let goto = keymap.find_action("GotoClass").unwrap();
        assert_eq!(goto.keys[0].canonical(), "ctrl alt N");
        assert_eq!(goto.defined_in, "Mine");
        assert!(keymap.find_action("SaveAll").is_none());
        let duplicate = keymap.find_action("EditorDuplicate").unwrap();
        assert_eq!(duplicate.defined_in, "Base");
        let lin = Platform::Linux;
        let toggle = keymap
            .resolve(&KeySequence::parse("ctrl `", lin).unwrap(), &[])
            .unwrap();
        assert_eq!(toggle.command.as_deref(), Some("terminal.toggle"));
        assert!(
            keymap
                .resolve(&KeySequence::parse("ctrl N", lin).unwrap(), &[])
                .is_none()
        );
    }

    #[test]
    fn unbound_marker_and_command_mapping() {
        let (keymap, _) = set().resolve("Base").unwrap();
        assert_eq!(keymap.find_action("GotoClass").unwrap().command, None);
        assert_eq!(
            keymap.find_action("SaveAll").unwrap().command.as_deref(),
            Some("editor.saveAll")
        );
        let dispatchable = keymap.dispatchable();
        assert_eq!(dispatchable.bindings().len(), 1);
        assert!(
            ActionCatalog::from_toml_str(
                "[[action]]\nid = \"X\"\nlabel = \"X\"\ncategory = \"C\"\n"
            )
            .is_err()
        );
    }

    #[test]
    fn context_resolution_prefers_specific_contexts() {
        let mut set = set();
        let mut user = KeymapFile::derived("Mine", "Base");
        user.set_keys(
            &BindingTarget::Action("GotoClass".into()),
            vec!["ctrl D".into()],
        );
        set.insert(user);
        let (keymap, _) = set.resolve("Mine").unwrap();
        let keys = KeySequence::parse("ctrl D", Platform::Linux).unwrap();
        assert_eq!(
            keymap.resolve(&keys, &[KeyContext::Editor]).unwrap().target,
            BindingTarget::Action("EditorDuplicate".into())
        );
        assert_eq!(
            keymap.resolve(&keys, &[]).unwrap().target,
            BindingTarget::Action("GotoClass".into())
        );
        let conflicts = keymap.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, ConflictKind::Shadowed);
    }

    #[test]
    fn detects_ambiguous_and_prefix_conflicts() {
        let mut set = set();
        let mut user = KeymapFile::derived("Mine", "Base");
        user.set_keys(
            &BindingTarget::Action("CommentByLineComment".into()),
            vec!["ctrl D".into(), "ctrl K, ctrl C".into()],
        );
        user.set_keys(
            &BindingTarget::Command("custom.k".into()),
            vec!["ctrl K".into()],
        );
        set.insert(user);
        let (keymap, _) = set.resolve("Mine").unwrap();
        let kinds: Vec<_> = keymap.conflicts().iter().map(|c| c.kind).collect();
        assert_eq!(kinds, vec![ConflictKind::Ambiguous, ConflictKind::Prefix]);
        let prefix = &keymap.conflicts()[1];
        assert_eq!(prefix.keys, "ctrl K");
        assert_eq!(prefix.other_keys, "ctrl K, ctrl C");
        let lookup = keymap.lookup(
            &KeySequence::parse("ctrl K", Platform::Linux).unwrap(),
            &[KeyContext::Editor],
        );
        assert!(lookup.is_prefix);
        assert_eq!(lookup.exact.len(), 1);
        let assigning = KeySequence::parse("ctrl K", Platform::Linux).unwrap();
        assert_eq!(
            keymap.conflicts_for(&assigning, KeyContext::Editor).len(),
            2
        );
    }

    #[test]
    fn reports_cycles_unknown_parents_and_bad_entries() {
        let mut set = set();
        set.insert(KeymapFile::derived("A", "B"));
        set.insert(KeymapFile::derived("B", "A"));
        assert!(matches!(
            set.resolve("A"),
            Err(KeymapError::InheritanceCycle(_))
        ));
        set.insert(KeymapFile::derived("Orphan", "Nope"));
        assert_eq!(
            set.resolve("Orphan").unwrap_err(),
            KeymapError::UnknownKeymap("Nope".into())
        );
        let mut bad = KeymapFile::derived("Bad", "Base");
        bad.set_keys(&BindingTarget::Action("NoSuchAction".into()), vec![]);
        bad.set_keys(
            &BindingTarget::Action("GotoClass".into()),
            vec!["ctrl BOGUS".into(), "ctrl G".into()],
        );
        bad.bindings.push(BindingSpec::default());
        set.insert(bad);
        let (keymap, diagnostics) = set.resolve("Bad").unwrap();
        assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
        assert_eq!(keymap.find_action("GotoClass").unwrap().keys.len(), 1);
        assert!(KeymapFile::from_toml_str("name = 3").is_err());
        assert!(KeymapFile::from_toml_str("name = \"x\"\nbogus = 1").is_err());
    }

    #[test]
    fn user_keymap_toml_round_trips() {
        let mut user = KeymapFile::derived("Mine", "Base");
        user.set_keys(
            &BindingTarget::Action("GotoClass".into()),
            vec!["ctrl alt N".into(), "ctrl K, ctrl N".into()],
        );
        user.set_keys(&BindingTarget::Action("SaveAll".into()), vec![]);
        let text = user.to_toml_string().unwrap();
        assert!(text.contains("parent = \"Base\""), "{text}");
        assert_eq!(KeymapFile::from_toml_str(&text).unwrap(), user);
    }
}
