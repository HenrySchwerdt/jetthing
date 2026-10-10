//! Docked tool windows rendered as islands.

use crate::icons::{self, Icon};
use crate::popups::Dialog;
use crate::widgets::{self, island_frame, list_row, row_text, tool_window_header};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, RichText, vec2};
use jetthing_core::{Anchor, ToolWindow};
use jetthing_git::ChangeKind;
use jetthing_runner::{OutputStream, TaskState};
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
            let hide = tool_window_header(ui, window.title(), |ui| {
                if window == ToolWindow::Commit
                    && icons::button(ui, Icon::Refresh, "Refresh").clicked()
                {
                    app.dispatch("git.refresh", Value::Null);
                }
            });
            if hide {
                app.toggle_tool_window(window);
            }
            match window {
                ToolWindow::Project => project_tree(app, ui),
                ToolWindow::Commit => commit(app, ui),
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
            ToolWindow::Build => build(app, ui),
            ToolWindow::Problems => problems(app, ui),
            ToolWindow::Terminal => terminal(app, ui),
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
        Some(ChangeKind::Modified | ChangeKind::Renamed) => theme::MODIFIED,
        Some(ChangeKind::Added) => theme::ADDED,
        Some(ChangeKind::Untracked) => theme::UNTRACKED,
        Some(ChangeKind::Deleted) => theme::MUTED,
        Some(ChangeKind::Conflicted) => theme::ERROR,
        None => theme::TEXT,
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
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, widgets::ROW_HEIGHT, entries.len() + 1, |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for row in range {
                if row == 0 {
                    let (rect, response) = list_row(ui, false);
                    tree_context_menu(&response, Path::new(""), true, &mut dialog, &mut open);
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
                        theme::TEXT_BRIGHT,
                    );
                    if root_has_errors {
                        crate::problems_ui::squiggle(
                            ui.painter(),
                            name_start,
                            x,
                            rect.center().y + 8.0,
                            theme::editor::ERROR_EFFECT,
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
                        theme::editor::ERROR_EFFECT,
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
                tree_context_menu(&response, path, *is_dir, &mut dialog, &mut open);
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
    response: &egui::Response,
    path: &Path,
    is_dir: bool,
    dialog: &mut Option<Dialog>,
    open: &mut Option<PathBuf>,
) {
    let is_root = path.as_os_str().is_empty();
    response.context_menu(|ui| {
        ui.set_min_width(220.0);
        ui.menu_button("New", |ui| {
            ui.set_min_width(160.0);
            for (label, directory) in [("File", false), ("Directory", true)] {
                if ui.button(label).clicked() {
                    *dialog = Some(Dialog::NewEntry {
                        parent: new_entry_parent(path, is_dir),
                        directory,
                        name: String::new(),
                    });
                    ui.close();
                }
            }
        });
        if !is_dir && ui.button("Open").clicked() {
            *open = Some(path.to_owned());
            ui.close();
        }
        if !is_root {
            ui.separator();
            if ui
                .add(egui::Button::new("Rename…").shortcut_text(crate::keys::display("⇧F6")))
                .clicked()
            {
                *dialog = Some(Dialog::Rename {
                    path: path.to_owned(),
                    name: IdeApp::file_name(path),
                });
                ui.close();
            }
            if ui
                .add(egui::Button::new("Delete…").shortcut_text("Delete"))
                .clicked()
            {
                *dialog = Some(Dialog::Delete {
                    path: path.to_owned(),
                });
                ui.close();
            }
        }
    });
}

/// Paints a checkbox in the reference style and returns whether it was clicked.
fn checkbox(ui: &egui::Ui, rect: Rect, state: Option<bool>) {
    let fill = match state {
        Some(true) | None => theme::ACCENT,
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
    let form_height = 132.0;
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
                    theme::TEXT_SECONDARY,
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
                    theme::TEXT_BRIGHT,
                );
                row_text(
                    ui,
                    rect,
                    x + 8.0,
                    &widgets::pluralize(files.len(), "file", "files"),
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT_INFO,
                );
                if response.clicked() && !files.is_empty() {
                    let value = state != Some(true);
                    toggles.extend(files.iter().map(|(p, _)| (p.clone(), value)));
                }
                for (path, kind) in files {
                    let (rect, response) = list_row(ui, false);
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
                        icons::ICON,
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
                            theme::TEXT_INFO,
                        );
                    }
                    let on_check = response
                        .interact_pointer_pos()
                        .is_some_and(|pos| check.expand(4.0).contains(pos));
                    if response.clicked() && on_check {
                        toggles.push((path.clone(), !is_included));
                    }
                    if response.double_clicked() && *kind != ChangeKind::Deleted {
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
    // Commit message and actions.
    let mut commit_now = false;
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.painter().hline(
                ui.max_rect().x_range(),
                ui.max_rect().top(),
                egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
            );
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
                let enabled = !app.commit_message.trim().is_empty() && !selected.is_empty();
                let button = egui::Button::new(RichText::new("Commit").color(Color32::WHITE))
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
    if commit_now && !app.commit_message.trim().is_empty() && !selected.is_empty() {
        let result = app.commands.execute(
            "git.commit",
            &mut app.state,
            json!({"message": app.commit_message.trim_end(), "paths": selected}),
        );
        match result {
            Ok(_) => {
                app.commit_message.clear();
                app.commit_overrides.clear();
            }
            Err(error) => app.state.status = error.to_string(),
        }
    }
    if let Some(path) = open {
        app.dispatch("editor.openFile", json!({"path": path}));
    }
}

pub fn commit_message_id() -> egui::Id {
    egui::Id::new("commit_message")
}

fn find(app: &mut IdeApp, ui: &mut egui::Ui) {
    if crate::navigation_ui::find_header(app, ui) {
        crate::navigation_ui::usages_view(app, ui);
        return;
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
        let submit = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if submit && !app.search_input.is_empty() {
            app.state.navigation.usages_selected = false;
            app.dispatch("search.project", json!({"query": app.search_input.clone()}));
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
                let mono = egui::FontId::monospace(theme::EDITOR_FONT_SIZE);
                let x = row_text(
                    ui,
                    rect,
                    rect.left() + 16.0,
                    result.preview.trim(),
                    mono,
                    theme::editor::TEXT,
                );
                let location = format!("{} {}", IdeApp::file_name(&result.path), result.line);
                let galley = ui.painter().layout_no_wrap(
                    location,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT_INFO,
                );
                let pos = egui::pos2(
                    (rect.right() - galley.size().x - 16.0).max(x + 16.0),
                    rect.center().y - galley.size().y / 2.0,
                );
                ui.painter()
                    .with_clip_rect(rect)
                    .galley(pos, galley, theme::TEXT_INFO);
                if response
                    .on_hover_text(result.path.display().to_string())
                    .clicked()
                {
                    open = Some(result.path.clone());
                }
            }
        });
    if let Some(path) = open {
        app.dispatch("editor.openFile", json!({"path": path}));
    }
}

fn build(app: &mut IdeApp, ui: &mut egui::Ui) {
    let title = app
        .state
        .runner
        .plan()
        .map(|plan| plan.name.clone())
        .unwrap_or_else(|| "Build".into());
    let running = app.state.runner.is_running();
    let hide = tool_window_header(ui, &title, |ui| {
        if running {
            if icons::button_colored(ui, Icon::Stop, theme::STOP_RED, "Stop", false).clicked() {
                app.dispatch("run.cancel", Value::Null);
            }
        } else if app.state.project.is_some() {
            if icons::button(ui, Icon::Build, "Build Project").clicked() {
                app.dispatch("build.project", Value::Null);
            }
            if icons::button_colored(ui, Icon::Run, theme::RUN_GREEN, "Run", false).clicked() {
                app.dispatch("run.project", Value::Null);
            }
        }
    });
    if hide {
        app.toggle_tool_window(ToolWindow::Build);
    }
    let (state_text, state_color) = match app.state.runner.state() {
        TaskState::Idle => ("Nothing has run yet".to_owned(), theme::TEXT_INFO),
        TaskState::Running => ("Running…".to_owned(), theme::TEXT_SECONDARY),
        TaskState::Exited(0) => ("Finished successfully".to_owned(), theme::SUCCESS),
        TaskState::Exited(code) => (format!("Failed with exit code {code}"), theme::ERROR),
        TaskState::Cancelled => ("Cancelled".to_owned(), theme::WARNING),
    };
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        if running {
            ui.add(egui::Spinner::new().size(12.0).color(theme::ACCENT));
        }
        ui.label(RichText::new(state_text).color(state_color));
        if let Some(plan) = app.state.runner.plan() {
            ui.label(
                RichText::new(format!("{} {}", plan.program, plan.args.join(" ")))
                    .monospace()
                    .color(theme::TEXT_INFO),
            );
        }
    });
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 4))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for chunk in app.state.runner.output() {
                        for line in chunk.text.trim_end_matches('\n').split('\n') {
                            let color = match chunk.stream {
                                OutputStream::Stdout => theme::editor::TEXT,
                                OutputStream::Stderr => console_stderr_color(line),
                            };
                            ui.label(RichText::new(line).monospace().color(color));
                        }
                    }
                });
        });
}

/// Cargo writes progress to stderr; only highlight lines that are real errors/warnings.
fn console_stderr_color(text: &str) -> Color32 {
    let trimmed = text.trim_start();
    if trimmed.starts_with("error") {
        theme::ERROR
    } else if trimmed.starts_with("warning") {
        theme::WARNING
    } else {
        theme::editor::TEXT
    }
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
    fn stderr_lines_are_only_red_for_errors() {
        assert_eq!(
            console_stderr_color("error[E0308]: mismatched"),
            theme::ERROR
        );
        assert_eq!(
            console_stderr_color("   Compiling foo"),
            theme::editor::TEXT
        );
    }
}
