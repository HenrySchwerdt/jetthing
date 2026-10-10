//! Main menu rendered from the spec action model (`jetthing-menu`).
//!
//! The menu structure (File … Help, submenus, separators, order) comes from
//! `docs/spec/data/actions.json`; this module only decides enabled state from
//! the command registry and performs workbench intents (dialogs, popups).
//! Actions Jetthing does not implement are rendered disabled.

use crate::{IdeApp, popups};
use eframe::egui;
use jetthing_core::ToolWindow;
use jetthing_menu::{
    ActionContext, Binding, BuiltInDynamicItems, DynamicItem, DynamicItems, Keymap, MenuError,
    MenuItem, Presented, ShortcutSource, Submenu, Target, VCS_MENU, VCS_MENU_GIT,
    VCS_MENU_WITHOUT_VCS, WorkbenchAction, present,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Tool windows Jetthing provides, with the reference activation action ids.
const TOOL_WINDOWS: [(ToolWindow, &str, &str); 6] = [
    (ToolWindow::Project, "ActivateProjectToolWindow", "Project"),
    (ToolWindow::Commit, "ActivateCommitToolWindow", "Commit"),
    (ToolWindow::Find, "ActivateFindToolWindow", "Find"),
    (ToolWindow::Build, "ActivateBuildToolWindow", "Build"),
    (
        ToolWindow::Problems,
        "ActivateProblemsViewToolWindow",
        "Problems",
    ),
    (
        ToolWindow::Terminal,
        "ActivateTerminalToolWindow",
        "Terminal",
    ),
];

/// Runtime groups: the built-in ones plus View | Tool Windows.
struct WorkbenchDynamicItems;

impl DynamicItems for WorkbenchDynamicItems {
    fn items(&self, group_id: &str) -> Vec<DynamicItem> {
        match group_id {
            "ToolWindowsGroup" => TOOL_WINDOWS
                .iter()
                .map(|(_, id, text)| DynamicItem::new(*id, *text))
                .collect(),
            other => BuiltInDynamicItems.items(other),
        }
    }
}

static MAIN_MENU: LazyLock<Result<Vec<Submenu>, MenuError>> =
    LazyLock::new(|| jetthing_menu::main_menu(&WorkbenchDynamicItems));

static KEYMAP: LazyLock<Option<Keymap>> = LazyLock::new(|| Keymap::platform_default().ok());

fn shortcuts() -> &'static dyn ShortcutSource {
    match KEYMAP.as_ref() {
        Some(keymap) => keymap,
        None => &jetthing_menu::NoShortcuts,
    }
}

/// Enabled/visible predicates over the workbench state.
struct WorkbenchContext<'a> {
    app: &'a IdeApp,
}

impl ActionContext for WorkbenchContext<'_> {
    fn is_enabled(&self, binding: &Binding) -> bool {
        let app = self.app;
        let project = app.state.project.is_some();
        let document = app.state.active_document.is_some();
        match binding.target {
            Target::Command(id) => app.commands.is_enabled(id, &app.state),
            Target::Workbench(intent) => match intent {
                WorkbenchAction::NewProject
                | WorkbenchAction::OpenProject
                | WorkbenchAction::FindAction
                | WorkbenchAction::SearchEverywhere
                | WorkbenchAction::Exit => true,
                WorkbenchAction::NewFile
                | WorkbenchAction::NewDirectory
                | WorkbenchAction::GotoFile
                | WorkbenchAction::FindInFiles => project,
                WorkbenchAction::RecentFiles => !app.state.recent_files.is_empty(),
                WorkbenchAction::GotoLine
                | WorkbenchAction::Find
                | WorkbenchAction::Replace
                | WorkbenchAction::Paste => document,
                WorkbenchAction::FindNext | WorkbenchAction::FindPrevious => {
                    document && !app.find.query.is_empty()
                }
            },
        }
    }

    /// The reference shows the `VCS` menu without a repository and the `Git`
    /// menu inside one.
    fn is_visible(&self, id: &str) -> bool {
        match id {
            VCS_MENU_WITHOUT_VCS => self.app.state.git.is_none(),
            VCS_MENU_GIT => self.app.state.git.is_some(),
            _ => true,
        }
    }

    fn text(&self, id: &str) -> Option<String> {
        (id == VCS_MENU && self.app.state.git.is_none()).then(|| "VCS".into())
    }
}

/// Contents of the main menu popup: one submenu per top-level entry.
pub fn show(app: &mut IdeApp, ui: &mut egui::Ui) {
    let menus = match MAIN_MENU.as_ref() {
        Ok(menus) => menus,
        Err(error) => {
            ui.label(error.to_string());
            return;
        }
    };
    for menu in menus {
        let context = WorkbenchContext { app };
        if !context.is_visible(&menu.id) {
            continue;
        }
        let text = context.text(&menu.id).unwrap_or_else(|| menu.text.clone());
        ui.menu_button(text, |ui| level(app, ui, &menu.items));
    }
}

/// One menu level; submenus recurse when opened.
fn level(app: &mut IdeApp, ui: &mut egui::Ui, items: &'static [MenuItem]) {
    ui.set_min_width(260.0);
    let rows = present(items, &WorkbenchContext { app }, shortcuts());
    let mut invoke = None;
    for row in rows {
        match row {
            Presented::Separator(text) => {
                ui.separator();
                if let Some(text) = text.filter(|text| !text.starts_with("type.")) {
                    ui.weak(text);
                }
            }
            Presented::Action {
                item,
                text,
                shortcut,
                enabled,
            } => {
                let button = egui::Button::new(text).shortcut_text(shortcut.unwrap_or_default());
                let mut response = ui.add_enabled(enabled, button);
                if let Some(description) = &item.description {
                    response = response.on_hover_text(description);
                }
                if response.clicked() {
                    invoke = item.binding.map(|binding| binding.target);
                    ui.close();
                }
            }
            Presented::Submenu {
                menu,
                text,
                enabled,
            } => {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.menu_button(text, |ui| level(app, ui, &menu.items));
                });
            }
        }
    }
    if let Some(target) = invoke {
        perform(app, ui.ctx(), target);
    }
}

/// Runs an action's target.
fn perform(app: &mut IdeApp, ctx: &egui::Context, target: Target) {
    match target {
        Target::Command(id) => {
            app.dispatch(id, Value::Null);
            if id == "toolwindow.activate.find" {
                app.search_focus_requested(ctx);
            }
        }
        Target::Workbench(intent) => perform_workbench(app, ctx, intent),
    }
}

fn perform_workbench(app: &mut IdeApp, ctx: &egui::Context, intent: WorkbenchAction) {
    match intent {
        WorkbenchAction::NewProject => app.new_project_dialog(),
        WorkbenchAction::NewFile | WorkbenchAction::NewDirectory => {
            app.open_dialog(popups::Dialog::NewEntry {
                parent: PathBuf::new(),
                directory: intent == WorkbenchAction::NewDirectory,
                name: String::new(),
            })
        }
        WorkbenchAction::OpenProject => app.open_project_dialog(),
        WorkbenchAction::FindAction => app.palette.show_tab(crate::palette::Tab::Actions),
        WorkbenchAction::SearchEverywhere => app.palette.show(),
        WorkbenchAction::GotoFile => app.palette.show_tab(crate::palette::Tab::Files),
        WorkbenchAction::RecentFiles => app.open_dialog(popups::Dialog::Recent {
            query: String::new(),
            selected: usize::from(app.state.recent_files.len() > 1),
        }),
        WorkbenchAction::GotoLine => app.open_dialog(popups::Dialog::GotoLine {
            input: String::new(),
        }),
        WorkbenchAction::Find | WorkbenchAction::Replace => {
            let selection = app.primary_selection_text();
            app.find.show(intent == WorkbenchAction::Replace, selection);
        }
        WorkbenchAction::FindNext => app.dispatch("editor.findNext", app.find.params()),
        WorkbenchAction::FindPrevious => app.dispatch("editor.findPrevious", app.find.params()),
        WorkbenchAction::FindInFiles => {
            app.dispatch("toolwindow.activate.find", Value::Null);
            app.search_focus_requested(ctx);
        }
        WorkbenchAction::Paste => {
            ctx.memory_mut(|memory| memory.request_focus(crate::editor::editor_id()));
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
        }
        WorkbenchAction::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_menu::{BINDINGS, actions};

    fn presented_texts(app: &IdeApp, items: &[MenuItem]) -> Vec<(String, bool)> {
        present(
            items,
            &WorkbenchContext { app },
            &jetthing_menu::NoShortcuts,
        )
        .iter()
        .filter_map(|row| row.text().map(|text| (text.to_owned(), row.is_enabled())))
        .collect()
    }

    #[test]
    fn command_bindings_name_registered_parameterless_commands() {
        let specs = jetthing_core::built_in_commands().specs();
        for binding in BINDINGS {
            if let Target::Command(id) = binding.target {
                let spec = specs
                    .iter()
                    .find(|spec| spec.id == id)
                    .unwrap_or_else(|| panic!("{} binds unknown command {id}", binding.action));
                assert!(
                    IdeApp::parameterless(spec),
                    "{} binds {id}, which needs parameters",
                    binding.action
                );
            }
        }
    }

    #[test]
    fn main_menu_renders_every_spec_entry_with_unbound_actions_disabled() {
        let app = IdeApp::with_update_check(None, Vec::new(), false);
        let menus = MAIN_MENU.as_ref().unwrap();
        let entries: Vec<MenuItem> = menus.iter().cloned().map(MenuItem::Submenu).collect();
        let titles: Vec<String> = presented_texts(&app, &entries)
            .into_iter()
            .map(|(text, _)| text)
            .collect();
        assert_eq!(
            titles,
            [
                "File", "Edit", "View", "Navigate", "Code", "Refactor", "Build", "Run", "Tools",
                "VCS", "Window", "Help"
            ]
        );
        let file = presented_texts(&app, &menus[0].items);
        let enabled = |rows: &[(String, bool)], text: &str| {
            rows.iter()
                .find(|(row, _)| row == text)
                .unwrap_or_else(|| panic!("{text} missing from {rows:?}"))
                .1
        };
        assert!(enabled(&file, "Open…"));
        assert!(enabled(&file, "Exit"));
        // Unimplemented: present but disabled.
        assert!(!enabled(&file, "Print…"));
        assert!(!enabled(&file, "Invalidate Caches…"));
        // Without a project, Find in Files is unavailable.
        let edit = &menus[1].items;
        let all: Vec<&str> = actions(edit).iter().map(|a| a.id.as_str()).collect();
        assert!(all.contains(&"FindInPath"));
        let help = presented_texts(&app, &menus[11].items);
        assert!(enabled(&help, "About"));
        assert!(enabled(&help, "Find Action…"));
    }

    #[test]
    fn tool_windows_submenu_lists_workbench_tool_windows() {
        let app = IdeApp::with_update_check(None, Vec::new(), false);
        let menus = MAIN_MENU.as_ref().unwrap();
        let view = menus.iter().find(|menu| menu.id == "ViewMenu").unwrap();
        let Some(MenuItem::Submenu(tool_windows)) = view
            .items
            .iter()
            .find(|item| matches!(item, MenuItem::Submenu(menu) if menu.id == "ToolWindowsGroup"))
        else {
            panic!("View | Tool Windows missing");
        };
        let rows = presented_texts(&app, &tool_windows.items);
        let names: Vec<&str> = rows.iter().map(|(text, _)| text.as_str()).collect();
        assert_eq!(
            names,
            ["Project", "Commit", "Find", "Build", "Problems", "Terminal"]
        );
    }

    #[test]
    fn menu_levels_render_headless() {
        let mut app = IdeApp::with_update_check(None, Vec::new(), false);
        let ctx = egui::Context::default();
        let menus = MAIN_MENU.as_ref().unwrap();
        let mut rendered = 0;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show(&mut app, ui);
                // Every level of every menu, as if each submenu were open.
                for menu in menus {
                    level(&mut app, ui, &menu.items);
                    rendered += 1;
                }
            });
        });
        assert_eq!(rendered, 12);
        assert!(app.dialog.is_none(), "rendering must not invoke actions");
    }

    #[test]
    fn menu_actions_dispatch_their_commands() {
        let mut app = IdeApp::with_update_check(None, Vec::new(), false);
        let ctx = egui::Context::default();
        perform(&mut app, &ctx, Target::Command("app.about"));
        assert!(
            app.state.status.to_lowercase().contains("jetthing"),
            "{}",
            app.state.status
        );
        perform(
            &mut app,
            &ctx,
            Target::Workbench(WorkbenchAction::FindAction),
        );
        assert!(app.palette.open);
        assert_eq!(app.palette.tab_name(), "actions");
        perform(
            &mut app,
            &ctx,
            Target::Workbench(WorkbenchAction::OpenProject),
        );
        assert_eq!(app.dialog.as_ref().map(|d| d.kind()), Some("openProject"));
    }
}
