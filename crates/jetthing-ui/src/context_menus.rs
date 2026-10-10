//! Context menus built from the reference popup groups (`ProjectViewPopupMenu`,
//! `EditorPopupMenu`, `EditorTabPopupMenu`; see `docs/spec/menus.md`).
//!
//! Like the reference IDE's popups, a context menu shows only the actions available
//! for its target: Jetthing's implemented actions, in reference order and grouping.
//! The model is computed before the menu is drawn so the drawing code needs no
//! mutable access to the app; the chosen action runs afterwards.

use crate::{IdeApp, keybindings, popups::Dialog};
use eframe::egui;
use jetthing_actions::{Catalog, MenuItem, filter_menu};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// What a context menu acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A Project view node (project-relative path; empty for the project root).
    Tree {
        path: PathBuf,
        is_dir: bool,
    },
    Editor,
    /// An editor tab by index.
    Tab(usize),
}

impl Target {
    fn group(&self) -> &'static str {
        match self {
            Target::Tree { .. } => "ProjectViewPopupMenu",
            Target::Editor => "EditorPopupMenu",
            Target::Tab(_) => "EditorTabPopupMenu",
        }
    }
}

/// Actions that only make sense for a target (they act on its path or tab).
const TARGET_ACTIONS: [&str; 12] = [
    "RenameElement",
    "$Delete",
    "CopyAbsolutePath",
    "CopyFileName",
    "CopyContentRootPath",
    "ProjectViewEditSource",
    "$Paste",
    "CloseContent",
    "CloseAllEditorsButActive",
    "CloseAllEditors",
    "CloseAllToTheLeft",
    "CloseAllToTheRight",
];

fn applies(action: &str, target: &Target) -> bool {
    match target {
        Target::Tree { path, is_dir } => {
            let root = path.as_os_str().is_empty();
            match action {
                "NewFile"
                | "NewDir"
                | "CopyAbsolutePath"
                | "CopyContentRootPath"
                | "FindInPath" => true,
                "RenameElement" | "$Delete" | "CopyFileName" => !root,
                "ProjectViewEditSource" => !is_dir,
                _ => false,
            }
        }
        Target::Editor => {
            action == "$Paste"
                || (!TARGET_ACTIONS.contains(&action) && keybindings::is_implemented(action))
        }
        Target::Tab(_) => matches!(
            action,
            "CloseContent"
                | "CloseAllEditorsButActive"
                | "CloseAllEditors"
                | "CloseAllToTheLeft"
                | "CloseAllToTheRight"
                | "CloseAllUnmodifiedEditors"
                | "PinActiveEditorTab"
                | "PinActiveTabToggle"
                | "ReopenClosedTab"
                | "CopyAbsolutePath"
                | "CopyFileName"
                | "CopyContentRootPath"
        ),
    }
}

/// A context menu ready to draw.
pub struct Model {
    items: Vec<MenuItem>,
    enabled: BTreeSet<String>,
    shortcuts: BTreeMap<String, String>,
}

impl IdeApp {
    pub(crate) fn context_menu_model(&self, target: &Target) -> Model {
        let menu = Catalog::shared_menu(target.group());
        let mut items = filter_menu(&menu, &|action| applies(action, target));
        if matches!(target, Target::Tree { .. }) {
            relabel(&mut items, "$Delete", "Delete…");
        }
        if let Target::Tab(index) = target {
            let label = if self.state.is_pinned(*index) {
                "Unpin Tab"
            } else {
                "Pin Tab"
            };
            relabel(&mut items, "PinActiveEditorTab", label);
            relabel(&mut items, "PinActiveTabToggle", label);
        }
        let mut enabled = BTreeSet::new();
        let mut shortcuts = BTreeMap::new();
        collect(&items, &mut |action| {
            if self.context_action_enabled(action, target) {
                enabled.insert(action.to_owned());
            }
            if let Some(shortcut) = self.action_shortcut(action) {
                shortcuts.insert(action.to_owned(), shortcut);
            }
        });
        Model {
            items,
            enabled,
            shortcuts,
        }
    }

    fn context_action_enabled(&self, action: &str, target: &Target) -> bool {
        let documents = self.state.documents.len();
        match (action, target) {
            ("$Paste", Target::Editor) => self.state.active_document.is_some(),
            ("CloseAllEditorsButActive", Target::Tab(_)) => documents > 1,
            ("CloseAllToTheLeft", Target::Tab(index)) => *index > 0,
            ("CloseAllToTheRight", Target::Tab(index)) => index + 1 < documents,
            (
                "CloseAllEditors"
                | "CloseAllUnmodifiedEditors"
                | "PinActiveEditorTab"
                | "PinActiveTabToggle",
                Target::Tab(_),
            ) => true,
            ("ReopenClosedTab", Target::Tab(_)) => !self.state.tabs.closed.is_empty(),
            (_, Target::Tree { .. } | Target::Tab(_)) if TARGET_ACTIONS.contains(&action) => true,
            _ => self.menu_action_enabled(action),
        }
    }

    /// Absolute path of a target, if it has one.
    fn target_path(&self, target: &Target) -> Option<(PathBuf, PathBuf)> {
        let root = self.state.project.as_ref()?.root().to_owned();
        let relative = match target {
            Target::Tree { path, .. } => path.clone(),
            Target::Tab(index) => self.state.documents.get(*index)?.path.clone(),
            Target::Editor => {
                let index = self.state.active_document?;
                self.state.documents.get(index)?.path.clone()
            }
        };
        Some((root.join(&relative), relative))
    }

    /// Runs the action chosen in a context menu.
    pub(crate) fn run_context_action(
        &mut self,
        action: &str,
        target: &Target,
        ctx: &egui::Context,
    ) {
        if let Target::Tree { path, .. } = target {
            // Context actions act on (and select) the clicked node.
            self.tree_selection = (!path.as_os_str().is_empty()).then(|| path.clone());
        }
        match action {
            "CopyAbsolutePath" | "CopyFileName" | "CopyContentRootPath" => {
                if let Some((absolute, relative)) = self.target_path(target) {
                    let text = match action {
                        "CopyAbsolutePath" => absolute.display().to_string(),
                        "CopyFileName" => IdeApp::file_name(&relative),
                        _ => relative.display().to_string(),
                    };
                    self.state.status = format!("Copied {text}");
                    ctx.copy_text(text);
                }
            }
            "RenameElement" => {
                if let Target::Tree { path, .. } = target {
                    self.open_dialog(Dialog::Rename {
                        path: path.clone(),
                        name: IdeApp::file_name(path),
                    });
                }
            }
            "$Delete" => {
                if let Target::Tree { path, .. } = target {
                    self.open_dialog(Dialog::Delete { path: path.clone() });
                }
            }
            "ProjectViewEditSource" => {
                if let Target::Tree { path, .. } = target {
                    self.dispatch("editor.openFile", json!({"path": path}));
                }
            }
            "$Paste" => ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste),
            "CloseContent" => {
                if let Target::Tab(index) = target {
                    self.dispatch("editor.close", json!({"index": index}));
                }
            }
            "CloseAllEditors"
            | "CloseAllEditorsButActive"
            | "CloseAllUnmodifiedEditors"
            | "CloseAllToTheLeft"
            | "CloseAllToTheRight"
            | "PinActiveEditorTab"
            | "PinActiveTabToggle" => {
                if let Target::Tab(index) = target {
                    // The tab commands skip pinned tabs, like the reference IDE.
                    let command = match action {
                        "CloseAllEditors" => "editor.closeAll",
                        "CloseAllEditorsButActive" => "editor.closeOthers",
                        "CloseAllUnmodifiedEditors" => "editor.closeUnmodified",
                        "CloseAllToTheLeft" => "editor.closeLeft",
                        "CloseAllToTheRight" => "editor.closeRight",
                        _ => "editor.togglePin",
                    };
                    self.dispatch(command, json!({"index": index}));
                }
            }
            "ReopenClosedTab" => self.dispatch("editor.reopenClosed", serde_json::Value::Null),
            _ => self.run_menu_action(action, ctx),
        }
    }
}

/// Some labels are set at runtime per place (the Project view shows "Delete…").
fn relabel(items: &mut [MenuItem], action: &str, text: &str) {
    for item in items {
        match item {
            MenuItem::Action { id, label } if id == action => *label = text.to_owned(),
            MenuItem::Submenu { items, .. } => relabel(items, action, text),
            _ => {}
        }
    }
}

fn collect(items: &[MenuItem], visit: &mut impl FnMut(&str)) {
    for item in items {
        match item {
            MenuItem::Action { id, .. } => visit(id),
            MenuItem::Submenu { items, .. } => collect(items, visit),
            MenuItem::Separator(_) => {}
        }
    }
}

/// Draws `model` into an open context menu; sets `chosen` when an item is clicked.
pub fn show(ui: &mut egui::Ui, model: &Model, chosen: &mut Option<String>) {
    ui.set_min_width(240.0);
    render(ui, &model.items, model, chosen);
}

fn render(ui: &mut egui::Ui, items: &[MenuItem], model: &Model, chosen: &mut Option<String>) {
    for item in items {
        match item {
            MenuItem::Action { id, label } => {
                let shortcut = model.shortcuts.get(id).cloned().unwrap_or_default();
                let button = egui::Button::new(label).shortcut_text(shortcut);
                if ui.add_enabled(model.enabled.contains(id), button).clicked() {
                    *chosen = Some(id.clone());
                    ui.close();
                }
            }
            MenuItem::Submenu { label, items, .. } => {
                ui.menu_button(label, |ui| {
                    ui.set_min_width(200.0);
                    render(ui, items, model, chosen);
                });
            }
            MenuItem::Separator(_) => {
                ui.separator();
            }
        }
    }
}

/// Labels of the top-level entries (tests and evidence).
#[cfg(test)]
fn labels(items: &[MenuItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            MenuItem::Action { label, .. } => label.clone(),
            MenuItem::Submenu { label, .. } => format!("{label} ▸"),
            MenuItem::Separator(_) => "---".into(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(target: Target) -> Vec<String> {
        let menu = Catalog::shared().menu(target.group());
        let mut items = filter_menu(&menu, &|action| applies(action, &target));
        if matches!(target, Target::Tree { .. }) {
            relabel(&mut items, "$Delete", "Delete…");
        }
        labels(&items)
    }

    #[test]
    fn project_view_menu_follows_the_reference_order() {
        let file = model(Target::Tree {
            path: "src/main.rs".into(),
            is_dir: false,
        });
        assert_eq!(file[0], "New ▸");
        let position = |label: &str| file.iter().position(|l| l == label).unwrap();
        assert!(position("Copy Path/Reference… ▸") < position("Rename…"));
        assert!(position("Rename…") < position("Delete…"));
        assert!(file.contains(&"Edit Source".to_owned()));
        // The root has no Rename/Delete/File Name.
        let root = model(Target::Tree {
            path: PathBuf::new(),
            is_dir: true,
        });
        assert!(!root.contains(&"Rename…".to_owned()));
        assert!(!root.contains(&"Delete…".to_owned()));
        assert!(!root.contains(&"Edit Source".to_owned()));
    }

    #[test]
    fn editor_and_tab_menus_list_implemented_actions() {
        let editor = model(Target::Editor);
        for label in ["Cut", "Copy", "Paste"] {
            assert!(editor.contains(&label.to_owned()), "{label} in {editor:?}");
        }
        let tab = model(Target::Tab(0));
        assert_eq!(
            &tab[..3],
            ["Close Tab", "Close Other Tabs", "Close All Tabs"]
        );
        assert!(tab.contains(&"Close Tabs to the Left".to_owned()));
    }
}
