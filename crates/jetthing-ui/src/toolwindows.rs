//! Docked tool windows rendered as islands.

use crate::icons::{self, Icon};
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
        .min_height(110.0)
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
    let entries: Vec<(PathBuf, bool)> = app
        .state
        .visible_files()
        .into_iter()
        .map(|entry| (entry.relative_path.clone(), entry.is_dir))
        .collect();
    let mut toggle = None;
    let mut open = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, widgets::ROW_HEIGHT, entries.len() + 1, |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for row in range {
                if row == 0 {
                    let (rect, _) = list_row(ui, false);
                    let mut x = rect.left() + 12.0 + 16.0;
                    icons::paint(
                        ui.painter(),
                        Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
                        Icon::Folder,
                        icons::ICON,
                    );
                    x += 22.0;
                    x = row_text(
                        ui,
                        rect,
                        x,
                        &root_name,
                        theme::semibold(theme::UI_FONT_SIZE),
                        theme::TEXT_BRIGHT,
                    );
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
                row_text(
                    ui,
                    rect,
                    x,
                    &name,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    change_color(kind),
                );
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
    if let Some(path) = toggle {
        app.dispatch("project.toggleDirectory", json!({"path": path}));
    }
    if let Some(path) = open {
        app.dispatch("editor.openFile", json!({"path": path}));
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
    let mut action = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (title, staged, untracked) in [
                ("Staged", true, false),
                ("Changes", false, false),
                ("Unversioned Files", false, true),
            ] {
                let changes: Vec<_> = git
                    .changes
                    .iter()
                    .filter(|change| {
                        let is_untracked = change.kind == ChangeKind::Untracked;
                        if staged {
                            change.staged
                        } else if untracked {
                            is_untracked
                        } else {
                            !is_untracked && (change.unstaged || !change.staged)
                        }
                    })
                    .collect();
                if changes.is_empty() && (staged || untracked) {
                    continue;
                }
                let (rect, _) = list_row(ui, false);
                icons::paint(
                    ui.painter(),
                    Rect::from_min_size(
                        egui::pos2(rect.left() + 12.0, rect.center().y - 8.0),
                        vec2(16.0, 16.0),
                    ),
                    Icon::ChevronDown,
                    theme::TEXT_SECONDARY,
                );
                let x = row_text(
                    ui,
                    rect,
                    rect.left() + 32.0,
                    title,
                    theme::semibold(theme::UI_FONT_SIZE),
                    theme::TEXT_BRIGHT,
                );
                row_text(
                    ui,
                    rect,
                    x + 8.0,
                    &widgets::pluralize(changes.len(), "file", "files"),
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT_INFO,
                );
                for change in changes {
                    let (rect, response) = list_row(ui, false);
                    let name = IdeApp::file_name(&change.path);
                    let mut x = rect.left() + 34.0;
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
                        change_color(Some(change.kind)),
                    );
                    if let Some(parent) = change.path.parent().filter(|p| !p.as_os_str().is_empty())
                    {
                        row_text(
                            ui,
                            rect,
                            x + 8.0,
                            &parent.display().to_string(),
                            egui::FontId::proportional(theme::UI_FONT_SIZE),
                            theme::TEXT_INFO,
                        );
                    }
                    if response.hovered() {
                        let button = Rect::from_min_size(
                            egui::pos2(rect.right() - 32.0, rect.top()),
                            vec2(24.0, 24.0),
                        );
                        icons::paint(
                            ui.painter(),
                            button.shrink(4.0),
                            if staged { Icon::Minus } else { Icon::Plus },
                            icons::ICON,
                        );
                        if response.clicked()
                            && response
                                .interact_pointer_pos()
                                .is_some_and(|pos| button.contains(pos))
                        {
                            action = Some((
                                if staged { "git.unstage" } else { "git.stage" },
                                change.path.clone(),
                            ));
                            continue;
                        }
                    }
                    let response = response.on_hover_text(if staged {
                        "Double-click to open · − to unstage"
                    } else {
                        "Double-click to open · + to stage"
                    });
                    if response.double_clicked() && change.kind != ChangeKind::Deleted {
                        let absolute = git.root.join(&change.path);
                        let relative = app
                            .state
                            .project
                            .as_ref()
                            .and_then(|project| absolute.strip_prefix(project.root()).ok())
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| change.path.clone());
                        action = Some(("editor.openFile", relative));
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
    if let Some((id, path)) = action {
        app.dispatch(id, json!({"path": path}));
    }
}

fn find(app: &mut IdeApp, ui: &mut egui::Ui) {
    let hide = tool_window_header(ui, "Find in Files", |ui| {
        if app.state.search.is_running()
            && icons::button_colored(ui, Icon::Stop, theme::STOP_RED, "Cancel", false).clicked()
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
        let submit = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if submit && !app.search_input.is_empty() {
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
    if tool_window_header(ui, "Problems", |_| {}) {
        app.toggle_tool_window(ToolWindow::Problems);
    }
    let Some(index) = app.state.active_document else {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No file is open").color(theme::TEXT_SECONDARY));
        });
        if let Some(error) = &app.state.lsp_error {
            ui.label(RichText::new(error).color(theme::TEXT_INFO));
        }
        return;
    };
    let path = app.state.documents[index].path.clone();
    let mut rows: Vec<(Icon, String, usize, usize)> = app.state.documents[index]
        .analysis
        .diagnostics
        .iter()
        .map(|d| (Icon::Error, d.message.clone(), d.row, d.column))
        .collect();
    rows.extend(app.state.active_lsp_diagnostics().into_iter().map(|d| {
        let icon = if d.severity == Some(1) {
            Icon::Error
        } else {
            Icon::Warning
        };
        (icon, d.message, d.start_line + 1, d.start_character + 1)
    }));
    let mut jump = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let (rect, _) = list_row(ui, false);
            let name = IdeApp::file_name(&path);
            icons::paint(
                ui.painter(),
                Rect::from_min_size(
                    egui::pos2(rect.left() + 12.0, rect.center().y - 8.0),
                    vec2(16.0, 16.0),
                ),
                Icon::for_file(&name),
                icons::ICON,
            );
            let x = row_text(
                ui,
                rect,
                rect.left() + 34.0,
                &name,
                egui::FontId::proportional(theme::UI_FONT_SIZE),
                theme::TEXT_BRIGHT,
            );
            row_text(
                ui,
                rect,
                x + 8.0,
                &widgets::pluralize(rows.len(), "problem", "problems"),
                egui::FontId::proportional(theme::UI_FONT_SIZE),
                theme::TEXT_INFO,
            );
            for (icon, message, line, column) in &rows {
                let (rect, response) = list_row(ui, false);
                icons::paint(
                    ui.painter(),
                    Rect::from_min_size(
                        egui::pos2(rect.left() + 34.0, rect.center().y - 7.0),
                        vec2(14.0, 14.0),
                    ),
                    *icon,
                    icons::ICON,
                );
                let first_line = message.lines().next().unwrap_or_default();
                let x = row_text(
                    ui,
                    rect,
                    rect.left() + 56.0,
                    first_line,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT,
                );
                row_text(
                    ui,
                    rect,
                    x + 8.0,
                    &format!(":{line}:{column}"),
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT_INFO,
                );
                if response.on_hover_text(message).clicked() {
                    jump = Some((*line, *column));
                }
            }
            if rows.is_empty() {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(34.0);
                    ui.label(RichText::new("No problems found").color(theme::TEXT_SECONDARY));
                });
            }
        });
    if let Some((line, column)) = jump {
        crate::editor::move_caret_to(app, line.saturating_sub(1), column.saturating_sub(1));
    }
}

fn terminal(app: &mut IdeApp, ui: &mut egui::Ui) {
    let exited = app.state.terminal.as_ref().is_some_and(|t| t.is_exited());
    let hide = tool_window_header(ui, "Terminal", |ui| {
        if exited {
            ui.label(RichText::new("process exited").color(theme::ERROR));
        }
    });
    if hide {
        app.toggle_tool_window(ToolWindow::Terminal);
        return;
    }
    let transcript = app
        .state
        .terminal
        .as_mut()
        .map(|terminal| strip_ansi(terminal.poll()))
        .unwrap_or_default();
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 4))
        .show(ui, |ui| {
            let input_height = 28.0;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .max_height((ui.available_height() - input_height).max(0.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(transcript)
                            .monospace()
                            .color(theme::editor::TEXT),
                    );
                });
            ui.horizontal(|ui| {
                ui.label(RichText::new("❯").monospace().color(theme::SUCCESS));
                let input = ui.add(
                    egui::TextEdit::singleline(&mut app.terminal_input)
                        .id(terminal_input_id())
                        .font(egui::TextStyle::Monospace)
                        .frame(false)
                        .desired_width(f32::INFINITY)
                        .hint_text("Type a command and press Enter"),
                );
                if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let command = std::mem::take(&mut app.terminal_input) + "\r";
                    app.dispatch("terminal.sendText", json!({"text": command}));
                    input.request_focus();
                }
            });
        });
}

/// Removes ANSI escape sequences until the terminal grid renderer lands.
fn strip_ansi(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c != '\r' {
            output.push(c);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_csi_and_osc_sequences() {
        assert_eq!(strip_ansi("\u{1b}[1;32mok\u{1b}[0m\r\n"), "ok\n");
        assert_eq!(strip_ansi("\u{1b}]0;title\u{7}$ "), "$ ");
        assert_eq!(strip_ansi("plain"), "plain");
    }

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
