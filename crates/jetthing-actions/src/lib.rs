//! Catalog of the reference IDE's actions and action groups.
//!
//! Loaded from `docs/spec/data/actions.json` (Apache-2.0 derived data, see
//! `docs/spec/data/NOTICE.md`): 3,451 action and group ids with their menu texts,
//! and the children of each group. The keymap page uses it to name actions; the
//! main menu and context menus are built from its groups.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

const ACTIONS_JSON: &str = include_str!("../../../docs/spec/data/actions.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Action,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Action {
    pub id: String,
    pub kind: Kind,
    pub text: Option<String>,
    pub description: Option<String>,
    /// Groups shown as a submenu (instead of inline).
    pub popup: Option<bool>,
    /// Hidden in a normal IDE installation.
    pub internal: bool,
}

impl Action {
    /// Menu text without mnemonic markers (`_File` → `File`).
    pub fn label(&self) -> String {
        clean_label(self.text.as_deref().unwrap_or(&self.id))
    }
}

/// Removes mnemonic markers (`_` and `&`) and trailing whitespace from a menu text.
pub fn clean_label(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // `__` and `&&` are literal characters.
            '_' | '&' if chars.peek() == Some(&c) => {
                out.push(c);
                chars.next();
            }
            '_' | '&' => {}
            other => out.push(other),
        }
    }
    out.trim().to_owned()
}

/// Labels the reference IDE sets at runtime instead of in its action XML.
const RUNTIME_LABELS: &[(&str, &str)] = &[
    ("ShowSettings", "Settings…"),
    ("TemplateProjectProperties", "Settings for New Projects…"),
    (
        "RunConfigurationTemplatesForNewProjects",
        "Run Configuration Templates for New Projects…",
    ),
    ("Exit", "Exit"),
    ("Back", "Back"),
    ("Forward", "Forward"),
    ("RunInspection", "Run Inspection by Name…"),
    ("XDebugger.AttachToProcess", "Attach to Process…"),
    ("ManageTargets", "Manage Targets…"),
    ("Stop", "Stop"),
    ("tasks.goto", "Open Task…"),
    ("Annotate", "Annotate with Git Blame"),
    ("MaximizeToolWindow", "Maximize Tool Window"),
    ("MaximizeEditorInSplit", "Maximize Editor in Split"),
    ("Unsplit", "Unsplit"),
    ("About", "About"),
    ("ConvertToWindowsLineSeparators", "CRLF - Windows (\\r\\n)"),
    ("ConvertToUnixLineSeparators", "LF - Unix and macOS (\\n)"),
    ("ConvertToMacLineSeparators", "CR - Classic Mac OS (\\r)"),
];

/// `ProblemsView` → `Problems View`.
fn split_camel_case(name: &str) -> String {
    let mut out = String::new();
    let mut previous_lower = false;
    for c in name.chars() {
        if c.is_uppercase() && previous_lower {
            out.push(' ');
        }
        previous_lower = c.is_lowercase();
        out.push(c);
    }
    out
}

/// One entry of a group.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Child {
    Ref {
        r#ref: String,
    },
    Separator {
        separator: bool,
        text: Option<String>,
    },
}

#[derive(Deserialize)]
struct ActionsFile {
    actions: Vec<Action>,
    groups: BTreeMap<String, Vec<Child>>,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    actions: BTreeMap<String, Action>,
    groups: BTreeMap<String, Vec<Child>>,
}

impl Catalog {
    pub fn bundled() -> Result<Self, String> {
        let file: ActionsFile =
            serde_json::from_str(ACTIONS_JSON).map_err(|error| error.to_string())?;
        Ok(Self {
            actions: file
                .actions
                .into_iter()
                .map(|action| (action.id.clone(), action))
                .collect(),
            groups: file.groups,
        })
    }

    /// Shared parsed copy (an empty catalog if the bundled data were invalid; a test
    /// guards against that).
    pub fn shared() -> &'static Catalog {
        static CATALOG: LazyLock<Catalog> =
            LazyLock::new(|| Catalog::bundled().unwrap_or_default());
        &CATALOG
    }

    pub fn get(&self, id: &str) -> Option<&Action> {
        self.actions.get(id)
    }

    /// Display name of an action id: its menu text, a label the reference IDE computes
    /// at runtime (same table as `docs/spec/tools/extract_intellij_data.py`), the
    /// tool window name for `Activate…ToolWindow`, else the id.
    pub fn label(&self, id: &str) -> String {
        if let Some(text) = self.actions.get(id).and_then(|a| a.text.as_deref()) {
            return clean_label(text);
        }
        if let Some((_, label)) = RUNTIME_LABELS.iter().find(|(action, _)| *action == id) {
            return (*label).to_owned();
        }
        if let Some(name) = id
            .strip_prefix("Activate")
            .and_then(|rest| rest.strip_suffix("ToolWindow"))
        {
            return format!("{} (tool window)", split_camel_case(name));
        }
        id.to_owned()
    }

    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.actions.values()
    }

    pub fn children(&self, group: &str) -> &[Child] {
        self.groups.get(group).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_loads() {
        let catalog = Catalog::bundled().unwrap();
        assert_eq!(catalog.actions().count(), 3451);
        assert_eq!(catalog.label("GotoAction"), "Find Action…");
        assert_eq!(catalog.label("ShowSettings"), "Settings…");
        assert_eq!(
            catalog.label("ActivateProblemsViewToolWindow"),
            "Problems View (tool window)"
        );
        assert_eq!(catalog.label("NoSuchAction"), "NoSuchAction");
        assert_eq!(catalog.get("FileMenu").unwrap().kind, Kind::Group);
        let main: Vec<_> = catalog
            .children("MainMenu")
            .iter()
            .filter_map(|child| match child {
                Child::Ref { r#ref } => Some(r#ref.as_str()),
                Child::Separator { .. } => None,
            })
            .collect();
        assert_eq!(main.first(), Some(&"FileMenu"));
        assert!(main.contains(&"CodeMenu"));
    }

    #[test]
    fn mnemonics_are_removed() {
        assert_eq!(clean_label("_File"), "File");
        assert_eq!(clean_label("Save &All"), "Save All");
        assert_eq!(clean_label("a__b"), "a_b");
    }
}
