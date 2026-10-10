//! Resolved menu trees and their per-frame presentation.

use crate::MenuError;
use crate::bindings::{Binding, binding};
use crate::catalog::{ActionKind, Catalog, GroupChild, INTERNAL_GROUPS, display_text};
use crate::keymap::ShortcutSource;

/// Nesting limit of the resolver (same as the extractor that wrote `menus.md`).
const MAX_DEPTH: usize = 8;

/// An action placed into a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionItem {
    pub id: String,
    pub text: String,
    pub description: Option<String>,
    /// Action whose shortcut is displayed (`use-shortcut-of`, else the action itself).
    pub shortcut_id: String,
    /// `None` when Jetthing does not implement the action: shown disabled.
    pub binding: Option<&'static Binding>,
    /// Contributed at runtime by [`DynamicItems`] rather than by the spec data.
    pub dynamic: bool,
}

impl ActionItem {
    pub fn icon(&self) -> Option<&'static str> {
        self.binding.and_then(|binding| binding.icon)
    }
}

/// A popup group: rendered as a submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submenu {
    pub id: String,
    pub text: String,
    pub description: Option<String>,
    pub items: Vec<MenuItem>,
}

/// One row of a resolved menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Action(ActionItem),
    Submenu(Submenu),
    /// A separator, optionally titled (toolbars use `type.*` spacer texts).
    Separator(Option<String>),
    /// Start of an inlined (non-popup) group, so its visibility can be decided at
    /// presentation time (e.g. the VCS menu by active VCS); `len` items follow.
    InlineGroup {
        id: String,
        len: usize,
    },
}

/// Runtime contributions to dynamic groups (Recent Projects, New, Tool Windows,
/// run executors), placed before the group's declared children.
pub trait DynamicItems {
    fn items(&self, group_id: &str) -> Vec<DynamicItem>;
}

/// A runtime-contributed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicItem {
    pub id: String,
    pub text: String,
}

impl DynamicItem {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

/// Exactly what the spec data declares.
pub struct NoDynamicItems;

impl DynamicItems for NoDynamicItems {
    fn items(&self, _group_id: &str) -> Vec<DynamicItem> {
        Vec::new()
    }
}

/// Dynamic groups whose content does not depend on the UI layer.
pub struct BuiltInDynamicItems;

impl DynamicItems for BuiltInDynamicItems {
    fn items(&self, group_id: &str) -> Vec<DynamicItem> {
        match group_id {
            // File | New
            "NewElementInMenuGroup" => vec![
                DynamicItem::new("NewProject", "Project…"),
                DynamicItem::new("NewFile", "File"),
                DynamicItem::new("NewDir", "Directory"),
            ],
            // Run | Run/Debug executors of the selected configuration.
            "RunnerActions" => vec![
                DynamicItem::new("Run", "Run"),
                DynamicItem::new("Debug", "Debug"),
            ],
            _ => Vec::new(),
        }
    }
}

/// Resolves groups into menu trees: popup groups become submenus, other groups
/// are inlined, internal entries are dropped, cycles are cut.
pub struct Resolver<'a> {
    catalog: &'a Catalog,
    dynamic: &'a dyn DynamicItems,
}

impl<'a> Resolver<'a> {
    pub fn new(catalog: &'a Catalog, dynamic: &'a dyn DynamicItems) -> Self {
        Self { catalog, dynamic }
    }

    /// The resolved contents of `group_id`.
    pub fn resolve(&self, group_id: &str) -> Result<Vec<MenuItem>, MenuError> {
        if !self.catalog.is_group(group_id) {
            return Err(MenuError::UnknownGroup(group_id.to_owned()));
        }
        let mut seen = vec![group_id.to_owned()];
        Ok(self.render(group_id, 0, &mut seen))
    }

    fn render(&self, group_id: &str, depth: usize, seen: &mut Vec<String>) -> Vec<MenuItem> {
        let mut items: Vec<MenuItem> = self
            .dynamic
            .items(group_id)
            .into_iter()
            .map(|item| {
                let mut action = self.action(&item.id);
                action.text = item.text;
                action.dynamic = true;
                MenuItem::Action(action)
            })
            .collect();
        for child in self.catalog.children(group_id) {
            let id = match child {
                GroupChild::Separator { text, .. } => {
                    items.push(MenuItem::Separator(text.clone()));
                    continue;
                }
                GroupChild::Ref { id } => id,
            };
            let def = self.catalog.get(id);
            if def.is_some_and(|def| def.internal) || INTERNAL_GROUPS.contains(&id.as_str()) {
                continue;
            }
            let Some(def) = def.filter(|def| def.kind == ActionKind::Group) else {
                items.push(MenuItem::Action(self.action(id)));
                continue;
            };
            if seen.contains(id) || depth > MAX_DEPTH {
                continue;
            }
            seen.push(id.clone());
            if def.popup == Some(true) && def.text.as_deref().is_some_and(|t| !t.is_empty()) {
                let children = self.render(id, depth + 1, seen);
                items.push(MenuItem::Submenu(Submenu {
                    id: id.clone(),
                    text: display_text(&self.catalog.label(id)),
                    description: def.description.clone(),
                    items: children,
                }));
            } else {
                let children = self.render(id, depth, seen);
                items.push(MenuItem::InlineGroup {
                    id: id.clone(),
                    len: children.len(),
                });
                items.extend(children);
            }
            seen.pop();
        }
        items
    }

    fn action(&self, id: &str) -> ActionItem {
        let def = self.catalog.get(id);
        ActionItem {
            id: id.to_owned(),
            text: display_text(&self.catalog.label(id)),
            description: def.and_then(|def| def.description.clone()),
            shortcut_id: def
                .and_then(|def| def.use_shortcut_of.clone())
                .unwrap_or_else(|| id.to_owned()),
            binding: binding(id),
            dynamic: false,
        }
    }
}

/// Runtime state queried while presenting a menu.
pub trait ActionContext {
    /// Whether a bound action can run now. Unbound actions are always disabled.
    fn is_enabled(&self, binding: &Binding) -> bool;

    /// Whether an action, submenu or inlined group is shown at all. The
    /// reference IDE shows unavailable actions disabled, so the default is
    /// `true`; contexts hide only what the reference hides (e.g. the menu of a
    /// VCS that is not active).
    fn is_visible(&self, _id: &str) -> bool {
        true
    }

    /// A runtime text replacing the declared one (e.g. `VCS` vs `Git`).
    fn text(&self, _id: &str) -> Option<String> {
        None
    }
}

/// A row ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Presented<'m> {
    Action {
        item: &'m ActionItem,
        text: String,
        shortcut: Option<String>,
        enabled: bool,
    },
    Submenu {
        menu: &'m Submenu,
        text: String,
        /// An empty submenu (dynamic content not available) is disabled.
        enabled: bool,
    },
    Separator(Option<&'m str>),
}

impl Presented<'_> {
    pub fn text(&self) -> Option<&str> {
        match self {
            Presented::Action { text, .. } | Presented::Submenu { text, .. } => Some(text),
            Presented::Separator(_) => None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            Presented::Action { enabled, .. } | Presented::Submenu { enabled, .. } => *enabled,
            Presented::Separator(_) => false,
        }
    }
}

/// Presents one menu level: applies visibility, enabled state, runtime texts and
/// shortcuts, and normalizes separators the way the reference renders them (no
/// leading, trailing or repeated separators).
pub fn present<'m>(
    items: &'m [MenuItem],
    context: &dyn ActionContext,
    shortcuts: &dyn ShortcutSource,
) -> Vec<Presented<'m>> {
    let mut rows = Vec::new();
    let mut index = 0;
    while index < items.len() {
        let item = &items[index];
        index += 1;
        match item {
            MenuItem::InlineGroup { id, len } => {
                if !context.is_visible(id) {
                    index += len;
                }
            }
            MenuItem::Separator(text) => rows.push(Presented::Separator(text.as_deref())),
            MenuItem::Action(action) => {
                if !context.is_visible(&action.id) {
                    continue;
                }
                rows.push(Presented::Action {
                    item: action,
                    text: context
                        .text(&action.id)
                        .unwrap_or_else(|| action.text.clone()),
                    shortcut: shortcuts.menu_shortcut(&action.shortcut_id),
                    enabled: action
                        .binding
                        .is_some_and(|binding| context.is_enabled(binding)),
                });
            }
            MenuItem::Submenu(menu) => {
                if !context.is_visible(&menu.id) {
                    continue;
                }
                let enabled = menu
                    .items
                    .iter()
                    .any(|item| matches!(item, MenuItem::Action(_) | MenuItem::Submenu(_)));
                rows.push(Presented::Submenu {
                    menu,
                    text: context.text(&menu.id).unwrap_or_else(|| menu.text.clone()),
                    enabled,
                });
            }
        }
    }
    normalize_separators(rows)
}

fn normalize_separators(rows: Vec<Presented<'_>>) -> Vec<Presented<'_>> {
    let mut out: Vec<Presented<'_>> = Vec::with_capacity(rows.len());
    for row in rows {
        if let Presented::Separator(text) = row {
            match out.last_mut() {
                None => {
                    // A titled separator at the top still titles the section.
                    if text.is_some() {
                        out.push(row);
                    }
                }
                Some(Presented::Separator(previous)) => {
                    if text.is_some() {
                        *previous = text;
                    }
                }
                Some(_) => out.push(row),
            }
        } else {
            out.push(row);
        }
    }
    while matches!(out.last(), Some(Presented::Separator(_))) {
        out.pop();
    }
    out
}

/// Every action of a resolved tree, depth first.
pub fn actions(items: &[MenuItem]) -> Vec<&ActionItem> {
    let mut out = Vec::new();
    for item in items {
        match item {
            MenuItem::Action(action) => out.push(action),
            MenuItem::Submenu(menu) => out.extend(actions(&menu.items)),
            MenuItem::Separator(_) | MenuItem::InlineGroup { .. } => {}
        }
    }
    out
}
