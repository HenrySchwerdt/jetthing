//! The reference action catalog: every action and group id with its text and
//! the ordered children of each group, from `docs/spec/data/actions.json`.

use crate::MenuError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

const ACTIONS_JSON: &str = include_str!("../../../docs/spec/data/actions.json");

/// Whether a catalog entry is a leaf action or a group of actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    Action,
    Group,
}

/// One `<action>` or `<group>` registration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDef {
    pub id: String,
    pub kind: ActionKind,
    /// Menu text as declared (absent for actions that set it at runtime).
    pub text: Option<String>,
    pub description: Option<String>,
    /// For groups: shown as a submenu (`true`) or inlined into the parent.
    pub popup: Option<bool>,
    /// Action whose keyboard shortcut this action shares.
    pub use_shortcut_of: Option<String>,
    #[serde(default)]
    pub searchable: bool,
    /// Hidden outside internal mode.
    #[serde(default)]
    pub internal: bool,
}

/// One entry of a group: a reference to an action/group, or a separator.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum GroupChild {
    Ref {
        #[serde(rename = "ref")]
        id: String,
    },
    Separator {
        separator: bool,
        text: Option<String>,
    },
}

#[derive(Deserialize)]
struct ActionsFile {
    actions: Vec<ActionDef>,
    groups: BTreeMap<String, Vec<GroupChild>>,
}

/// Groups hidden in a normal installation (internal / UI debugging tools); the
/// same list as `docs/spec/tools/extract_intellij_data.py`.
pub const INTERNAL_GROUPS: &[&str] = &[
    "Internal",
    "InternalActions",
    "Internal.UI",
    "Internal.Errors",
    "UiDebugger",
];

/// Labels the reference IDE computes at runtime instead of declaring them (same
/// table as the extractor, so labels match `docs/spec/menus.md`).
const RUNTIME_LABELS: &[(&str, &str)] = &[
    ("ShowSettings", "Settings…"),
    (
        "ConvertModuleGroupsToQualifiedNames",
        "Convert Module Groups to Qualified Names…",
    ),
    ("ConvertToWindowsLineSeparators", "CRLF - Windows (\\r\\n)"),
    ("ConvertToUnixLineSeparators", "LF - Unix and macOS (\\n)"),
    ("ConvertToMacLineSeparators", "CR - Classic Mac OS (\\r)"),
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
];

/// Parsed action catalog.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    actions: BTreeMap<String, ActionDef>,
    groups: BTreeMap<String, Vec<GroupChild>>,
}

impl Catalog {
    /// Parses an `actions.json` document.
    pub fn from_json(json: &str) -> Result<Self, MenuError> {
        let file: ActionsFile =
            serde_json::from_str(json).map_err(|error| MenuError::Data(error.to_string()))?;
        let mut actions = BTreeMap::new();
        for action in file.actions {
            if actions.contains_key(&action.id) {
                return Err(MenuError::Data(format!(
                    "duplicate action id {}",
                    action.id
                )));
            }
            actions.insert(action.id.clone(), action);
        }
        Ok(Self {
            actions,
            groups: file.groups,
        })
    }

    /// The catalog bundled from `docs/spec/data/actions.json`, parsed once.
    pub fn bundled() -> Result<&'static Catalog, MenuError> {
        static BUNDLED: LazyLock<Result<Catalog, MenuError>> =
            LazyLock::new(|| Catalog::from_json(ACTIONS_JSON));
        BUNDLED.as_ref().map_err(Clone::clone)
    }

    pub fn get(&self, id: &str) -> Option<&ActionDef> {
        self.actions.get(id)
    }

    pub fn actions(&self) -> impl Iterator<Item = &ActionDef> {
        self.actions.values()
    }

    pub fn group_ids(&self) -> impl Iterator<Item = &str> {
        self.groups.keys().map(String::as_str)
    }

    /// Children of `group` in declaration order (after `add-to-group` anchors).
    pub fn children(&self, group: &str) -> &[GroupChild] {
        self.groups.get(group).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn is_group(&self, id: &str) -> bool {
        self.actions
            .get(id)
            .map(|action| action.kind == ActionKind::Group)
            .unwrap_or_else(|| self.groups.contains_key(id))
    }

    /// Display text: the declared text, a runtime label, the tool window name for
    /// `Activate…ToolWindow`, else the id itself.
    pub fn label(&self, id: &str) -> String {
        if let Some(text) = self
            .actions
            .get(id)
            .and_then(|action| action.text.as_deref())
            .filter(|text| !text.is_empty())
        {
            return text.to_owned();
        }
        if let Some((_, label)) = RUNTIME_LABELS.iter().find(|(action, _)| *action == id) {
            return (*label).to_owned();
        }
        if let Some(name) = id
            .strip_prefix("Activate")
            .and_then(|rest| rest.strip_suffix("ToolWindow"))
            .filter(|name| !name.is_empty())
        {
            return format!("{} (tool window)", split_camel_case(name));
        }
        id.to_owned()
    }
}

/// `ProblemsView` → `Problems View`.
fn split_camel_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let mut previous_lower = false;
    for c in name.chars() {
        if c.is_ascii_uppercase() && previous_lower {
            out.push(' ');
        }
        previous_lower = c.is_ascii_lowercase();
        out.push(c);
    }
    out
}

/// Menu text for display: `&&` (an escaped ampersand in the reference data) → `&`.
pub fn display_text(text: &str) -> String {
    text.replace("&&", "&")
}
