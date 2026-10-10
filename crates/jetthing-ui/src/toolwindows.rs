//! Docked tool windows rendered as islands.

use crate::icons::{self, Icon};
use crate::popups::Dialog;
use crate::widgets::{self, island_frame, list_row, row_text, tool_window_header};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, RichText, vec2};
use jetthing_core::{Anchor, ToolWindow};
use jetthing_git::ChangeKind;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn find_input_id() -> egui::Id {
    egui::Id::new("find_in_files_input")
}

pub fn terminal_input_id() -> egui::Id {
    egui::Id::new("terminal_input")
}

pub fn left(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(window) = app.state.tool_windows.visible(Anchor::Left) else {
        return;
    };
    egui::SidePanel::left("left_tool_window")
        .default_width(280.0)
        .min_width(160.0)
        .resizable(true)
        .show_separator_line(false)
        .frame(island_frame())
        .show(ctx, |ui| {
            let android = window == ToolWindow::Project && app.state.android.project.is_some();
            let hide = if android {
                crate::android_view::header(app, ui)
            } else {
                tool_window_header(ui, window.title(), |ui| {
                    if window == ToolWindow::Commit
                        && icons::button(ui, Icon::Refresh, "Refresh").clicked()
                    {
                        app.dispatch("git.refresh", Value::Null);
                    }
                })
            };
            if hide {
                app.toggle_tool_window(window);
            }
            match window {
                ToolWindow::Project if crate::android_view::active(app) => {
                    crate::android_view::tree(app, ui)
                }
                ToolWindow::Project => project_tree(app, ui),
                ToolWindow::Commit => commit(app, ui),
                ToolWindow::BuildVariants => crate::build_variants_ui::show(app, ui),
                ToolWindow::Structure => crate::structure_view::tool_window(app, ui),
                _ => {}
            }
        });
}

pub fn bottom(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(window) = app.state.tool_windows.visible(Anchor::Bottom) else {
        return;
    };
    egui::TopBottomPanel::bottom("bottom_tool_window")
        .default_height(260.0)
        .min_height(180.0)
        .resizable(true)
        .show_separator_line(false)
        .frame(island_frame())
        .show(ctx, |ui| match window {
            ToolWindow::Find => find(app, ui),
            ToolWindow::Build => crate::build_window::show(app, ui),
            ToolWindow::Problems => problems(app, ui),
            ToolWindow::Terminal => terminal(app, ui),
            ToolWindow::Logcat => crate::logcat_ui::show(app, ui),
            ToolWindow::Git => crate::git_log_ui::show(app, ui),
            _ => {}
        });
}

/// Git status per project-relative path.
fn git_colors(app: &IdeApp) -> HashMap<PathBuf, ChangeKind> {
    let (Some(git), Some(project)) = (&app.state.git, &app.state.project) else {
        return HashMap::new();
    };
    git.changes
        .iter()
        .map(|change| {
            let absolute = git.root.join(&change.path);
            let relative = absolute
                .strip_prefix(project.root())
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| change.path.clone());
            (relative, change.kind)
        })
        .collect()
}

pub fn change_color(kind: Option<ChangeKind>) -> Color32 {
    match kind {
        Some(ChangeKind::Modified | ChangeKind::Renamed) => *theme::MODIFIED,
        Some(ChangeKind::Added) => *theme::ADDED,
        Some(ChangeKind::Untracked) => *theme::UNTRACKED,
        Some(ChangeKind::Deleted) => *theme::MUTED,
        Some(ChangeKind::Conflicted) => *theme::ERROR,
        None => *theme::TEXT,
    }
}

const TREE_INDENT: f32 = 18.0;

fn project_tree(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(project) = app.state.project.as_ref() else {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No project open").color(theme::TEXT_SECONDARY));
            ui.label(
                RichText::new("Start Jetthing with a project path: jetthing <dir>")
                    .color(theme::TEXT_INFO),
            );
        });
        return;
    };
    let root_name = IdeApp::file_name(project.root());
    let root_path = project.root().display().to_string();
    let colors = git_colors(app);
    app.state.refresh_problems();
    let root_has_errors = app.state.problems.counts().errors > 0;
    let entries: Vec<(PathBuf, bool)> = app
        .state
        .visible_files()
        .into_iter()
        .map(|entry| (entry.relative_path.clone(), entry.is_dir))
        .collect();
    let mut toggle = None;
    let mut open = None;
    let mut dialog = None;
    let mut context_choice = None;
    // Like "Always Select Opened File": the selection follows the active
    // editor and the row scrolls into view when it is off-screen.
    let active = app
        .state
        .active_document
        .and_then(|index| app.state.documents.get(index))
        .map(|document| document.path.clone());
    let mut scroll_target = None;
    if active.is_some() && active != app.tree_revealed {
        app.tree_revealed = active.clone();
        if let Some(path) = active
            && let Some(row) = entries.iter().position(|(entry, _)| *entry == path)
        {
            app.tree_selection = Some(path);
            let id = ui.make_persistent_id("project_tree");
            let offset = egui::scroll_area::State::load(ui.ctx(), id)
                .map(|state| state.offset.y)
                .unwrap_or(0.0);
            scroll_target =
                reveal_offset(row + 1, widgets::ROW_HEIGHT, offset, ui.available_height());
        }
    }
    let mut area = egui::ScrollArea::vertical()
        .id_salt("project_tree")
        .auto_shrink([false, false]);
    if let Some(target) = scroll_target {
        area = area.vertical_scroll_offset(target);
    }
    area.show_rows(ui, widgets::ROW_HEIGHT, entries.len() + 1, |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in range {
            if row == 0 {
                let (rect, response) = list_row(ui, false);
                tree_context_menu(app, &response, Path::new(""), true, &mut context_choice);
                let mut x = rect.left() + 12.0 + 16.0;
                icons::paint(
                    ui.painter(),
                    Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
                    Icon::Folder,
                    icons::ICON,
                );
                x += 22.0;
                let name_start = x;
                x = row_text(
                    ui,
                    rect,
                    x,
                    &root_name,
                    theme::semibold(theme::UI_FONT_SIZE),
                    *theme::TEXT_BRIGHT,
                );
                if root_has_errors {
                    crate::problems_ui::squiggle(
                        ui.painter(),
                        name_start,
                        x,
                        rect.center().y + 8.0,
                        *theme::editor::ERROR_EFFECT,
                    );
                }
                row_text(
                    ui,
                    rect,
                    x + 8.0,
                    &root_path,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT_INFO,
                );
                continue;
            }
            let (path, is_dir) = &entries[row - 1];
            let depth = path.components().count();
            let selected = app.tree_selection.as_deref() == Some(path.as_path());
            let (rect, response) = list_row(ui, selected);
            let mut x = rect.left() + 12.0 + depth as f32 * TREE_INDENT;
            let chevron =
                Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0));
            if *is_dir {
                let expanded = app.state.expanded_dirs.contains(path);
                icons::paint(
                    ui.painter(),
                    chevron,
                    if expanded {
                        Icon::ChevronDown
                    } else {
                        Icon::ChevronRight
                    },
                    theme::TEXT_SECONDARY,
                );
            }
            x += 16.0;
            let name = IdeApp::file_name(path);
            let icon = if *is_dir {
                Icon::Folder
            } else {
                Icon::for_file(&name)
            };
            icons::paint(
                ui.painter(),
                Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
                icon,
                icons::ICON,
            );
            x += 22.0;
            let kind = colors.get(path).copied().or_else(|| {
                is_dir
                    .then(|| {
                        colors
                            .keys()
                            .any(|changed| changed.starts_with(path))
                            .then_some(ChangeKind::Modified)
                    })
                    .flatten()
            });
            let (end, elided) = widgets::row_text_elided(
                ui,
                rect,
                x,
                &name,
                egui::FontId::proportional(theme::UI_FONT_SIZE),
                change_color(kind),
            );
            if app.state.problems.has_errors(path) {
                crate::problems_ui::squiggle(
                    ui.painter(),
                    x,
                    end,
                    rect.center().y + 8.0,
                    *theme::editor::ERROR_EFFECT,
                );
            }
            let response = if elided {
                response.on_hover_text(&name)
            } else {
                response
            };
            if response.secondary_clicked() {
                app.tree_selection = Some(path.clone());
            }
            tree_context_menu(app, &response, path, *is_dir, &mut context_choice);
            if response.clicked() {
                app.tree_selection = Some(path.clone());
                let on_chevron = response
                    .interact_pointer_pos()
                    .is_some_and(|pos| chevron.expand(4.0).contains(pos));
                if *is_dir && on_chevron {
                    toggle = Some(path.clone());
                }
            }
            if response.double_clicked() {
                if *is_dir {
                    toggle = Some(path.clone());
                } else {
                    open = Some(path.clone());
                }
            }
        }
    });
    if ui.ui_contains_pointer()
        && ui.memory(|m| m.focused().is_none())
        && ui.input(|input| input.key_pressed(egui::Key::Enter))
        && let Some(selected) = app.tree_selection.clone()
    {
        if entries
            .iter()
            .any(|(path, is_dir)| *path == selected && *is_dir)
        {
            toggle = Some(selected);
        } else {
            open = Some(selected);
        }
    }
    // Keyboard actions on the selected node while the pointer is over the tree.
    // Only when no text field (editor, terminal, …) holds keyboard focus.
    let tree_has_keys = ui.ui_contains_pointer() && ui.memory(|m| m.focused().is_none());
    if tree_has_keys
        && dialog.is_none()
        && let Some(selected) = app.tree_selection.clone()
    {
        let is_dir = entries.iter().any(|(p, d)| *p == selected && *d);
        let (delete, rename, new_file) = ui.input_mut(|input| {
            let mac = cfg!(target_os = "macos");
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::Delete)
                    || (mac && input.consume_key(egui::Modifiers::COMMAND, egui::Key::Backspace)),
                input.consume_key(egui::Modifiers::SHIFT, egui::Key::F6),
                input.consume_key(egui::Modifiers::ALT, egui::Key::Insert)
                    || (mac && input.consume_key(egui::Modifiers::COMMAND, egui::Key::N)),
            )
        });
        if delete {
            dialog = Some(Dialog::Delete {
                path: selected.clone(),
            });
        } else if rename {
            dialog = Some(Dialog::Rename {
                name: IdeApp::file_name(&selected),
                path: selected.clone(),
            });
        } else if new_file {
            dialog = Some(Dialog::NewEntry {
                parent: new_entry_parent(&selected, is_dir),
                directory: false,
                name: String::new(),
            });
        }
    }
    if let Some(dialog) = dialog {
        app.open_dialog(dialog);
    }
    if let Some(path) = toggle {
        app.dispatch("project.toggleDirectory", json!({"path": path}));
    }
    if let Some(path) = open {
        app.dispatch("editor.openFile", json!({"path": path}));
    }
    if let Some((action, target)) = context_choice {
        let ctx = ui.ctx().clone();
        app.run_context_action(&action, &target, &ctx);
    }
}

/// Scroll offset that centres `row`, or `None` when it is already fully visible.
fn reveal_offset(row: usize, row_height: f32, offset: f32, viewport: f32) -> Option<f32> {
    let top = row as f32 * row_height;
    let bottom = top + row_height;
    if top >= offset && bottom <= offset + viewport {
        return None;
    }
    Some((top + row_height / 2.0 - viewport / 2.0).max(0.0))
}

fn new_entry_parent(path: &Path, is_dir: bool) -> PathBuf {
    if is_dir {
        path.to_owned()
    } else {
        path.parent().map(Path::to_path_buf).unwrap_or_default()
    }
}

/// Right-click menu of a Project tree node.
fn tree_context_menu(
    app: &IdeApp,
    response: &egui::Response,
    path: &Path,
    is_dir: bool,
    chosen: &mut Option<(String, crate::context_menus::Target)>,
) {
    response.context_menu(|ui| {
        let target = crate::context_menus::Target::Tree {
            path: path.to_owned(),
            is_dir,
        };
        let model = app.context_menu_model(&target);
        let mut action = None;
        crate::context_menus::show(ui, &model, &mut action);
        if let Some(action) = action {
            *chosen = Some((action, target));
        }
    });
}

/// Paints a checkbox in the reference style and returns whether it was clicked.
fn checkbox(ui: &egui::Ui, rect: Rect, state: Option<bool>) {
    let fill = match state {
        Some(true) | None => *theme::ACCENT,
        Some(false) => Color32::TRANSPARENT,
    };
    ui.painter().rect_filled(rect, 3.0, fill);
    if state == Some(false) {
        ui.painter().rect_stroke(
            rect,
            3.0,
            egui::Stroke::new(1.0_f32, theme::TEXT_INFO),
            egui::StrokeKind::Inside,
        );
    }
    let white = egui::Stroke::new(1.5_f32, Color32::WHITE);
    match state {
        Some(true) => {
            let points = vec![
                rect.left_top() + vec2(3.0, 7.0),
                rect.left_top() + vec2(6.0, 10.0),
                rect.left_top() + vec2(11.0, 4.0),
            ];
            ui.painter().add(egui::Shape::line(points, white));
        }
        None => {
            ui.painter().line_segment(
                [
                    rect.left_center() + vec2(3.5, 0.0),
                    rect.right_center() - vec2(3.5, 0.0),
                ],
                white,
            );
        }
        Some(false) => {}
    }
}

fn commit(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(git) = app.state.git.clone() else {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("The project is not a Git repository.").color(theme::TEXT_SECONDARY),
            );
        });
        return;
    };
    let project_root = app.state.project.as_ref().map(|p| p.root().to_path_buf());
    let git_root = git.root.canonicalize().unwrap_or_else(|_| git.root.clone());
    // Project-relative path for each change (commands take project paths).
    let to_project = |path: &Path| -> PathBuf {
        let absolute = git_root.join(path);
        project_root
            .as_ref()
            .and_then(|root| absolute.strip_prefix(root).ok())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_owned())
    };
    let included = |app: &IdeApp, path: &Path, kind: ChangeKind| {
        app.commit_overrides
            .get(path)
            .copied()
            .unwrap_or(kind != ChangeKind::Untracked)
    };
    let groups: Vec<(&str, Vec<(PathBuf, ChangeKind)>)> =
        [("Changes", false), ("Unversioned Files", true)]
            .into_iter()
            .map(|(title, untracked)| {
                let files = git
                    .changes
                    .iter()
                    .filter(|c| (c.kind == ChangeKind::Untracked) == untracked)
                    .map(|c| (to_project(&c.path), c.kind))
                    .collect::<Vec<_>>();
                (title, files)
            })
            .collect();
    let selected: Vec<PathBuf> = groups
        .iter()
        .flat_map(|(_, files)| files.iter())
        .filter(|(path, kind)| included(app, path, *kind))
        .map(|(path, _)| path.clone())
        .collect();
    let mut open = None;
    let mut toggles: Vec<(PathBuf, bool)> = Vec::new();
    let selection_id = egui::Id::new("commit_selected_row");
    let mut selected_row: Option<PathBuf> = ui.data(|data| data.get_temp(selection_id));
    // Toolbar: rollback checked files, diff of the selected row.
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.x = 2.0;
        let rollback = icons::button(
            ui,
            Icon::Rollback,
            &format!("Rollback… ({})", crate::keys::display("⌥⌘Z")),
        );
        if rollback.clicked() && !selected.is_empty() {
            app.git_dialog = Some(crate::branches_popup::GitDialog::Rollback {
                paths: selected
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
            });
        }
        let diff = icons::button(
            ui,
            Icon::Diff,
            &format!("Show Diff ({})", crate::keys::display("⌘D")),
        );
        if diff.clicked()
            && let Some(path) = selected_row.clone().or_else(|| selected.first().cloned())
        {
            open = Some(path);
        }
    });
    let form_height = 160.0;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height((ui.available_height() - form_height).max(60.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (title, files) in &groups {
                if files.is_empty() && *title != "Changes" {
                    continue;
                }
                let checked = files.iter().filter(|(p, k)| included(app, p, *k)).count();
                let state = match checked {
                    0 => Some(false),
                    n if n == files.len() => Some(true),
                    _ => None,
                };
                let (rect, response) = list_row(ui, false);
                icons::paint(
                    ui.painter(),
                    Rect::from_min_size(
                        egui::pos2(rect.left() + 10.0, rect.center().y - 8.0),
                        vec2(16.0, 16.0),
                    ),
                    Icon::ChevronDown,
                    *theme::TEXT_SECONDARY,
                );
                let check = Rect::from_min_size(
                    egui::pos2(rect.left() + 30.0, rect.center().y - 7.0),
                    vec2(14.0, 14.0),
                );
                if !files.is_empty() {
                    checkbox(ui, check, state);
                }
                let x = row_text(
                    ui,
                    rect,
                    rect.left() + 52.0,
                    title,
                    theme::semibold(theme::UI_FONT_SIZE),
                    *theme::TEXT_BRIGHT,
                );
                row_text(
                    ui,
                    rect,
                    x + 8.0,
                    &widgets::pluralize(files.len(), "file", "files"),
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    *theme::TEXT_INFO,
                );
                if response.clicked() && !files.is_empty() {
                    let value = state != Some(true);
                    toggles.extend(files.iter().map(|(p, _)| (p.clone(), value)));
                }
                for (path, kind) in files {
                    let (rect, response) = list_row(ui, selected_row.as_ref() == Some(path));
                    if response.clicked() {
                        selected_row = Some(path.clone());
                    }
                    let check = Rect::from_min_size(
                        egui::pos2(rect.left() + 46.0, rect.center().y - 7.0),
                        vec2(14.0, 14.0),
                    );
                    let is_included = included(app, path, *kind);
                    checkbox(ui, check, Some(is_included));
                    let name = IdeApp::file_name(path);
                    let mut x = rect.left() + 68.0;
                    icons::paint(
                        ui.painter(),
                        Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
                        Icon::for_file(&name),
                        *icons::ICON,
                    );
                    x += 22.0;
                    x = row_text(
                        ui,
                        rect,
                        x,
                        &name,
                        egui::FontId::proportional(theme::UI_FONT_SIZE),
                        change_color(Some(*kind)),
                    );
                    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                        row_text(
                            ui,
                            rect,
                            x + 8.0,
                            &parent.display().to_string(),
                            egui::FontId::proportional(theme::UI_FONT_SIZE),
                            *theme::TEXT_INFO,
                        );
                    }
                    let on_check = response
                        .interact_pointer_pos()
                        .is_some_and(|pos| check.expand(4.0).contains(pos));
                    if response.clicked() && on_check {
                        toggles.push((path.clone(), !is_included));
                    }
                    // Like the reference IDE, double-click shows the diff.
                    if response.double_clicked() {
                        open = Some(path.clone());
                    }
                }
            }
            if git.changes.is_empty() {
                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No changes").color(theme::TEXT_SECONDARY));
                });
            }
        });
    for (path, value) in toggles {
        app.commit_overrides.insert(path, value);
    }
    ui.data_mut(|data| match &selected_row {
        Some(path) => data.insert_temp(selection_id, path.clone()),
        None => data.remove::<PathBuf>(selection_id),
    });
    // Ctrl+D (⌘D) shows the diff of the selected row.
    if selected_row.is_some()
        // Only while no text field or the editor has keyboard focus, so the
        // editor's own ⌘D (duplicate line) keeps working.
        && ui.memory(|memory| memory.focused().is_none())
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::D))
    {
        open = selected_row.clone();
    }
    // Commit message and actions.
    let mut commit_now = false;
    let mut push_after = false;
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.painter().hline(
                ui.max_rect().x_range(),
                ui.max_rect().top(),
                egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
            );
            ui.horizontal(|ui| {
                let mut amend = app.commit_amend;
                if ui
                    .checkbox(&mut amend, "Amend")
                    .on_hover_text("Amend the last commit instead of creating a new one")
                    .changed()
                {
                    app.commit_amend = amend;
                    if amend && app.commit_message.trim().is_empty() {
                        app.commit_message = app.state.last_commit_message().unwrap_or_default();
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let modified = git
                        .changes
                        .iter()
                        .filter(|change| change.kind != ChangeKind::Untracked)
                        .count();
                    if modified > 0 {
                        ui.label(
                            RichText::new(widgets::pluralize(modified, "modified", "modified"))
                                .color(theme::ACCENT),
                        );
                    }
                });
            });
            let response = ui.add(
                egui::TextEdit::multiline(&mut app.commit_message)
                    .id(commit_message_id())
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .hint_text("Commit Message"),
            );
            if response.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter))
            {
                commit_now = true;
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let enabled = !app.commit_message.trim().is_empty()
                    && (!selected.is_empty() || app.commit_amend);
                let label = if app.commit_amend {
                    "Amend Commit"
                } else {
                    "Commit"
                };
                let button = egui::Button::new(RichText::new(label).color(Color32::WHITE))
                    .fill(if enabled {
                        theme::ACCENT
                    } else {
                        theme::SELECTION_DISABLED
                    })
                    .min_size(vec2(72.0, 26.0));
                if ui
                    .add_enabled(enabled, button)
                    .on_hover_text(format!(
                        "Commit selected files ({})",
                        crate::keys::display("⌘↩")
                    ))
                    .clicked()
                {
                    commit_now = true;
                }
                if ui
                    .add_enabled(enabled, egui::Button::new("Commit and Push…"))
                    .on_hover_text("Commit, then open the Push dialog")
                    .clicked()
                {
                    commit_now = true;
                    push_after = true;
                }
                ui.label(
                    RichText::new(widgets::pluralize(
                        selected.len(),
                        "file selected",
                        "files selected",
                    ))
                    .color(theme::TEXT_INFO),
                );
            });
        });
    if commit_now
        && !app.commit_message.trim().is_empty()
        && (!selected.is_empty() || app.commit_amend)
    {
        let result = app.commands.execute(
            "git.commit",
            &mut app.state,
            json!({
                "message": app.commit_message.trim_end(),
                "paths": selected,
                "amend": app.commit_amend,
            }),
        );
        match result {
            Ok(_) => {
                app.commit_message.clear();
                app.commit_overrides.clear();
                app.commit_amend = false;
                if push_after {
                    crate::push_dialog::open(app);
                }
            }
            Err(error) => app.state.status = error.to_string(),
        }
    }
    if let Some(path) = open {
        app.dispatch("git.showDiff", json!({"path": path}));
    }
}

pub fn commit_message_id() -> egui::Id {
    egui::Id::new("commit_message")
}

impl IdeApp {
    /// Copies the query and toggles of a search started outside the Find
    /// inputs into them; typing an unsubmitted query is left alone.
    fn sync_find_inputs(&mut self) {
        let state = (&self.state.search_query, &self.state.search_options);
        if state == (&self.find_synced.0, &self.find_synced.1) {
            return;
        }
        self.find_synced = (
            self.state.search_query.clone(),
            self.state.search_options.clone(),
        );
        self.search_input = self.state.search_query.clone();
        self.find_options = self.state.search_options.clone();
    }
}

fn find(app: &mut IdeApp, ui: &mut egui::Ui) {
    app.sync_find_inputs();
    let hide = tool_window_header(ui, "Find in Files", |ui| {
        if app.state.search.is_running()
            && icons::button_colored(ui, Icon::Stop, *theme::STOP_RED, "Cancel", false).clicked()
        {
            app.dispatch("search.cancel", Value::Null);
        }
    });
    if hide {
        app.toggle_tool_window(ToolWindow::Find);
    }
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        let input = widgets::search_field(
            ui,
            find_input_id(),
            &mut app.search_input,
            "Text to find",
            360.0,
        );
        let mut submit = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        // Option toggles re-run the search, like the reference popup.
        let options = &mut app.find_options;
        for (label, tooltip, value) in [
            ("Cc", "Match Case", &mut options.match_case),
            ("W", "Words", &mut options.whole_words),
            (".*", "Regex", &mut options.regex),
        ] {
            let toggle = ui
                .selectable_label(*value, RichText::new(label).monospace())
                .on_hover_text(tooltip);
            if toggle.clicked() {
                *value = !*value;
                submit = true;
            }
        }
        ui.add_space(8.0);
        let mask = widgets::search_field(
            ui,
            egui::Id::new("find_file_mask"),
            &mut options.file_mask,
            "File mask, e.g. *.java, !*Test.java",
            220.0,
        );
        submit |= mask.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if submit && !app.search_input.is_empty() {
            app.dispatch(
                "search.project",
                find_params(&app.search_input, &app.find_options),
            );
        }
        let count = app.state.search_results.len();
        let mut summary = widgets::pluralize(count, "match", "matches");
        if app.state.search.is_running() {
            summary.push('…');
        } else if app.state.search_truncated {
            summary.push_str(" (limit reached)");
        }
        if !app.state.search_query.is_empty() {
            ui.label(RichText::new(summary).color(theme::TEXT_SECONDARY));
        }
    });
    ui.add_space(4.0);
    let results = app.state.search_results.clone();
    let mut open = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, widgets::ROW_HEIGHT, results.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for result in &results[range] {
                let (rect, response) = list_row(ui, false);
                // The location stays visible at the right; the preview (already
                // elided around the match by the search engine) is clipped before it.
                let location = format!("{} {}", IdeApp::file_name(&result.path), result.line);
                let location = ui.painter().layout_no_wrap(
                    location,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    *theme::TEXT_INFO,
                );
                let location_x =
                    (rect.right() - location.size().x - 16.0).max(rect.left() + rect.width() * 0.5);
                let job = highlighted_preview(&result.preview, &result.highlights);
                let galley = ui.painter().layout_job(job);
                ui.painter()
                    .with_clip_rect(rect.with_max_x(location_x - 12.0))
                    .galley(
                        egui::pos2(rect.left() + 16.0, rect.center().y - galley.size().y / 2.0),
                        galley,
                        *theme::editor::TEXT,
                    );
                let pos = egui::pos2(location_x, rect.center().y - location.size().y / 2.0);
                ui.painter()
                    .with_clip_rect(rect)
                    .galley(pos, location, *theme::TEXT_INFO);
                if response
                    .on_hover_text(result.path.display().to_string())
                    .clicked()
                {
                    open = Some(result.clone());
                }
            }
        });
    if let Some(result) = open {
        app.dispatch(
            "editor.openFile",
            json!({
                "path": result.path,
                "line": result.line,
                "column": result.column,
                "length": result.length,
            }),
        );
        ui.memory_mut(|memory| memory.request_focus(crate::editor::editor_id()));
    }
}

/// Parameters of `search.project` for the query and the toggled options.
fn find_params(query: &str, options: &jetthing_core::SearchOptions) -> Value {
    json!({
        "query": query,
        "matchCase": options.match_case,
        "wholeWords": options.whole_words,
        "regex": options.regex,
        "fileMask": options.file_mask,
    })
}

/// Result preview with every occurrence on a search-result background.
fn highlighted_preview(
    preview: &str,
    highlights: &[std::ops::Range<usize>],
) -> egui::text::LayoutJob {
    let plain = egui::TextFormat {
        font_id: egui::FontId::monospace(theme::EDITOR_FONT_SIZE),
        color: *theme::editor::TEXT,
        ..Default::default()
    };
    let matched = egui::TextFormat {
        background: *theme::editor::SEARCH_RESULT,
        color: *theme::TEXT_BRIGHT,
        ..plain.clone()
    };
    let chars: Vec<char> = preview.chars().collect();
    let mut job = egui::text::LayoutJob::default();
    let mut cursor = 0;
    for range in highlights {
        let start = range.start.min(chars.len()).max(cursor);
        let end = range.end.min(chars.len());
        if start >= end {
            continue;
        }
        job.append(
            &chars[cursor..start].iter().collect::<String>(),
            0.0,
            plain.clone(),
        );
        job.append(
            &chars[start..end].iter().collect::<String>(),
            0.0,
            matched.clone(),
        );
        cursor = end;
    }
    job.append(&chars[cursor..].iter().collect::<String>(), 0.0, plain);
    job
}

fn problems(app: &mut IdeApp, ui: &mut egui::Ui) {
    crate::problems_ui::tool_window(app, ui);
}

fn terminal(app: &mut IdeApp, ui: &mut egui::Ui) {
    crate::terminal_ui::show(app, ui);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_options_become_command_parameters() {
        let options = jetthing_core::SearchOptions {
            match_case: true,
            regex: true,
            file_mask: "*.kt".into(),
            ..Default::default()
        };
        let params = find_params("gr.*t", &options);
        assert_eq!(
            params,
            json!({"query": "gr.*t", "matchCase": true, "wholeWords": false, "regex": true, "fileMask": "*.kt"})
        );
        let specs = jetthing_core::built_in_commands().specs();
        let spec = specs
            .iter()
            .find(|spec| spec.id == "search.project")
            .unwrap();
        for key in ["matchCase", "wholeWords", "regex", "fileMask"] {
            assert!(spec.params.contains(key), "{key} documented in the schema");
        }
    }

    #[test]
    fn a_search_started_by_command_fills_the_find_inputs() {
        let (mut app, _ctx) = crate::tests::headless_app("find-sync", "a.txt", "needle\n");
        app.dispatch(
            "search.project",
            json!({"query": "ne+dle", "regex": true, "matchCase": true}),
        );
        app.sync_find_inputs();
        assert_eq!(app.search_input, "ne+dle");
        assert!(app.find_options.regex && app.find_options.match_case);
        assert!(!app.find_options.whole_words);
        // A query being typed is not overwritten while the last search is unchanged.
        app.search_input = "typing".into();
        app.sync_find_inputs();
        assert_eq!(app.search_input, "typing");
    }

    #[test]
    fn find_preview_highlights_each_occurrence() {
        let job = highlighted_preview("val greeting = greet(name)", &[4..9, 15..20]);
        let texts: Vec<&str> = job
            .sections
            .iter()
            .map(|section| &job.text[section.byte_range.clone()])
            .collect();
        assert_eq!(texts, ["val ", "greet", "ing = ", "greet", "(name)"]);
        assert_eq!(
            job.sections[1].format.background,
            *theme::editor::SEARCH_RESULT
        );
        assert_eq!(job.sections[2].format.background, Color32::TRANSPARENT);
        // Out-of-range or overlapping highlights never panic.
        let job = highlighted_preview("ab", &[1..9, 0..1]);
        assert_eq!(job.text, "ab");
    }

    #[test]
    fn reveal_scrolls_only_when_the_row_is_hidden() {
        assert_eq!(reveal_offset(3, 24.0, 0.0, 400.0), None);
        assert_eq!(
            reveal_offset(40, 24.0, 0.0, 400.0),
            Some(40.0 * 24.0 + 12.0 - 200.0)
        );
        assert_eq!(reveal_offset(0, 24.0, 300.0, 400.0), Some(0.0));
    }
}
