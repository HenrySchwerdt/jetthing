//! UI-independent action system.
//!
//! The reference IDE builds its main menu, context menus and toolbars from
//! action groups. This crate loads that structure from the Apache-2.0 derived
//! spec data (`docs/spec/data/actions.json`, rendered in `docs/spec/menus.md`),
//! binds the action ids Jetthing implements to typed commands or workbench
//! intents ([`bindings`]), shows shortcuts from the bundled keymaps ([`keymap`]),
//! and presents each menu level with enabled/visible predicates ([`present`]).
//! Actions without an implementation stay in place, disabled, so every menu
//! keeps the reference shape.
//!
//! No UI toolkit is involved: a UI layer walks [`Presented`] rows.

pub mod bindings;
pub mod catalog;
pub mod keymap;
mod model;

#[cfg(test)]
mod tests;

pub use bindings::{BINDINGS, Binding, Target, WorkbenchAction, binding};
pub use catalog::{ActionDef, ActionKind, Catalog, GroupChild};
pub use keymap::{Keymap, NoShortcuts, ShortcutSource, mac_glyphs};
pub use model::{
    ActionContext, ActionItem, BuiltInDynamicItems, DynamicItem, DynamicItems, MenuItem,
    NoDynamicItems, Presented, Resolver, Submenu, actions, present,
};

use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MenuError {
    #[error("invalid bundled action data: {0}")]
    Data(String),
    #[error("unknown action group: {0}")]
    UnknownGroup(String),
    #[error("unknown keymap: {0}")]
    UnknownKeymap(String),
}

/// The main menu bar (`File` … `Help`).
pub const MAIN_MENU: &str = "MainMenu";
/// Right-click menu of the editor.
pub const EDITOR_POPUP: &str = "EditorPopupMenu";
/// Right-click menu of the Project view.
pub const PROJECT_VIEW_POPUP: &str = "ProjectViewPopupMenu";
/// Right-click menu of an editor tab.
pub const EDITOR_TAB_POPUP: &str = "EditorTabPopupMenu";
/// Right-click menu of a tool window header.
pub const TOOL_WINDOW_POPUP: &str = "ToolWindowContextMenu";
/// New UI main toolbar (left, center and right parts).
pub const MAIN_TOOLBAR: &str = "MainToolbarNewUI";
/// Generate popup (Alt+Insert / ⌘N).
pub const GENERATE: &str = "GenerateGroup";

/// Inlined group of the VCS-less main-menu entry (shown when no VCS is active).
pub const VCS_MENU_WITHOUT_VCS: &str = "Vcs.MainMenu";
/// Inlined group of the Git main-menu entry (shown when Git is active).
pub const VCS_MENU_GIT: &str = "Git.MainMenu";
/// The `VCS`/`Git` main-menu entry.
pub const VCS_MENU: &str = "VcsGroups";

/// The resolved main menu: one submenu per top-level entry.
pub fn main_menu(dynamic: &dyn DynamicItems) -> Result<Vec<Submenu>, MenuError> {
    let catalog = Catalog::bundled()?;
    Ok(Resolver::new(catalog, dynamic)
        .resolve(MAIN_MENU)?
        .into_iter()
        .filter_map(|item| match item {
            MenuItem::Submenu(menu) => Some(menu),
            _ => None,
        })
        .collect())
}

/// Any bundled group resolved with `dynamic` contributions.
pub fn group(group_id: &str, dynamic: &dyn DynamicItems) -> Result<Vec<MenuItem>, MenuError> {
    Resolver::new(Catalog::bundled()?, dynamic).resolve(group_id)
}
