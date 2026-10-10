//! Build / Run tool window: execution tree with timings on the left, console on the right.

use crate::icons::{self, Icon};
use crate::widgets::{list_row, row_text, tool_window_header};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, RichText, vec2};
use jetthing_core::ToolWindow;
use jetthing_runner::{MessageKind, OutputStream, StepOutcome, TaskKind, TaskState};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Duration;

const TREE_INDENT: f32 = 18.0;
const CONSOLE_ROW: f32 = 18.0;

/// Navigation requested from the tree (applied after painting).
enum Action {
    Select(usize, Option<usize>),
    Open {
        node: usize,
        path: String,
        line: u32,
        column: u32,
    },
}

#[derive(Clone, Copy, Default)]
struct View {
    /// Selected tree node id (0 = root, 1 + step, 1_000_000 + message).
    selected: Option<usize>,
    /// Console line to reveal.
    reveal: Option<usize>,
}

fn view_id() -> egui::Id {
    egui::Id::new("build_window_view")
}

/// `3 sec, 520 ms` / `818 ms` / `2 min, 5 sec`, as the reference shows durations.
pub fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        return format!("{millis} ms");
    }
    let secs = duration.as_secs();
    if secs < 60 {
        return format!("{secs} sec, {} ms", millis % 1000);
    }
    let minutes = secs / 60;
    if minutes < 60 {
        return format!("{minutes} min, {} sec", secs % 60);
    }
    format!("{} h, {} min", minutes / 60, minutes % 60)
}

pub fn show(app: &mut IdeApp, ui: &mut egui::Ui) {
    let plan = app.state.runner.plan().cloned();
    let kind = plan.as_ref().map_or(TaskKind::Build, |plan| plan.kind);
    let title = match kind {
        TaskKind::Run => "Run",
        TaskKind::Test => "Test",
        TaskKind::Sync => "Sync",
        TaskKind::Build | TaskKind::Custom => "Build",
    };
    let running = app.state.runner.is_running();
    let hide = tool_window_header(ui, title, |ui| {
        if let Some(plan) = &plan {
            ui.label(
                RichText::new(&plan.name)
                    .color(theme::TEXT_SECONDARY)
                    .size(theme::UI_FONT_SIZE),
            );
        }
    });
    if hide {
        app.toggle_tool_window(ToolWindow::Build);
    }
    let Some(plan) = plan else {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Nothing has run yet").color(theme::TEXT_SECONDARY));
            let hint = match &app.state.detected_build {
                Some(build) => format!(
                    "{} project · Build Project or Run from the toolbar",
                    build.kind.display_name()
                ),
                None if app.state.project.is_some() => {
                    "No supported build system (Cargo, Gradle, Maven, Bazel, npm, Go) was detected"
                        .into()
                }
                None => "Open a project to build it".into(),
            };
            ui.label(RichText::new(hint).color(theme::TEXT_INFO));
        });
        return;
    };
    let mut view: View = ui.data(|data| data.get_temp(view_id())).unwrap_or_default();
    let mut action = None;
    let available = ui.available_rect_before_wrap();
    let toolbar_width = 34.0;
    let tree_width = (available.width() * 0.36).clamp(220.0, 520.0);
    let toolbar_rect =
        egui::Rect::from_min_size(available.min, vec2(toolbar_width, available.height()));
    let tree_rect = egui::Rect::from_min_size(
        available.min + vec2(toolbar_width, 0.0),
        vec2(tree_width, available.height()),
    );
    let console_rect = egui::Rect::from_min_max(
        egui::pos2(tree_rect.right() + 1.0, available.top()),
        available.max,
    );
    ui.allocate_rect(available, egui::Sense::hover());
    ui.painter().line_segment(
        [tree_rect.right_top(), tree_rect.right_bottom()],
        egui::Stroke::new(1.0_f32, theme::TOOLBAR_SEPARATOR),
    );

    let mut toolbar = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(toolbar_rect.shrink2(vec2(4.0, 4.0)))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    toolbar.spacing_mut().item_spacing.y = 2.0;
    let rerun = app.commands.is_enabled("run.rerun", &app.state);
    if toolbar
        .add_enabled_ui(rerun, |ui| icons::button(ui, Icon::Rerun, "Rerun"))
        .inner
        .clicked()
    {
        app.dispatch("run.rerun", Value::Null);
    }
    let stop_color = if running {
        theme::STOP_RED
    } else {
        theme::TEXT_DISABLED
    };
    if toolbar
        .add_enabled_ui(running, |ui| {
            icons::button_colored(ui, Icon::Stop, stop_color, "Stop", false)
        })
        .inner
        .clicked()
    {
        app.dispatch("run.cancel", Value::Null);
    }

    let test_mode = kind == TaskKind::Test && (running || !app.state.runner.tests().is_empty());
    if test_mode {
        let failed = app.commands.is_enabled("test.rerunFailed", &app.state);
        if toolbar
            .add_enabled_ui(failed, |ui| {
                icons::button_colored(ui, Icon::Rerun, *theme::ERROR, "Rerun Failed Tests", false)
            })
            .inner
            .clicked()
        {
            app.dispatch("test.rerunFailed", Value::Null);
        }
    }

    let project_name = app.project_name();
    let project_root = app
        .state
        .project
        .as_ref()
        .map(|project| project.root().to_owned());
    let mut tree = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(tree_rect.shrink2(vec2(0.0, 4.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    if test_mode {
        crate::test_view::tree(app, &mut tree, running);
    } else {
        egui::ScrollArea::both()
            .id_salt("build_tree")
            .auto_shrink([false, false])
            .show(&mut tree, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let state = app.state.runner.state().clone();
                let events = app.state.runner.events();
                let (icon, verb) = match &state {
                    TaskState::Running | TaskState::Idle => (None, "running…".to_owned()),
                    TaskState::Exited(0) => (
                        Some((Icon::Success, *theme::SUCCESS)),
                        match kind {
                            TaskKind::Build => "build finished".into(),
                            TaskKind::Sync => "sync finished".into(),
                            _ => "successful".into(),
                        },
                    ),
                    TaskState::Exited(code) => (
                        Some((Icon::Error, *theme::ERROR)),
                        match kind {
                            TaskKind::Build => format!("build failed (exit code {code})"),
                            TaskKind::Sync => format!("sync failed (exit code {code})"),
                            _ => format!("failed (exit code {code})"),
                        },
                    ),
                    TaskState::Cancelled => {
                        (Some((Icon::Stop, *theme::WARNING)), "cancelled".into())
                    }
                    TaskState::Failed(_) => {
                        (Some((Icon::Error, *theme::ERROR)), "could not start".into())
                    }
                };
                let subject = match kind {
                    TaskKind::Build | TaskKind::Sync => project_name.clone(),
                    _ => format!("{project_name} [{}]", plan.name),
                };
                let (rect, response) = list_row(ui, view.selected == Some(0));
                let mut x = rect.left() + 10.0;
                match icon {
                    Some((icon, color)) => {
                        icons::paint(
                            ui.painter(),
                            egui::Rect::from_min_size(
                                egui::pos2(x, rect.center().y - 7.0),
                                vec2(14.0, 14.0),
                            ),
                            icon,
                            color,
                        );
                    }
                    None => {
                        let spinner = egui::Rect::from_min_size(
                            egui::pos2(x, rect.center().y - 6.0),
                            vec2(12.0, 12.0),
                        );
                        ui.put(
                            spinner,
                            egui::Spinner::new().size(12.0).color(theme::ACCENT),
                        );
                    }
                }
                x += 20.0;
                x = row_text(
                    ui,
                    rect,
                    x,
                    &format!("{subject}: "),
                    theme::semibold(theme::UI_FONT_SIZE),
                    *theme::TEXT_BRIGHT,
                );
                x = row_text(
                    ui,
                    rect,
                    x,
                    &verb,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    theme::TEXT,
                );
                if let Some(elapsed) = app.state.runner.elapsed() {
                    row_text(
                        ui,
                        rect,
                        x + 8.0,
                        &format_duration(elapsed),
                        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                        theme::TEXT_INFO,
                    );
                }
                if response.clicked() {
                    action = Some(Action::Select(0, Some(0)));
                }
                if let TaskState::Failed(reason) = &state {
                    message_row(ui, 1, Icon::Error, *theme::ERROR, reason, None, false);
                }
                let messages = events.messages();
                let paths = Paths {
                    root: project_root.as_deref(),
                    cwd: &plan.cwd,
                };
                for &id in events.orphan_messages() {
                    if let Some(clicked) = message_node(ui, &view, id, &messages[id], 1, &paths) {
                        action = Some(clicked);
                    }
                }
                for (index, step) in events.steps().iter().enumerate() {
                    let node = 1 + index;
                    let (rect, response) = list_row(ui, view.selected == Some(node));
                    let mut x = rect.left() + 10.0 + TREE_INDENT;
                    let (icon, color) = match step.outcome {
                        StepOutcome::Failed => (Some(Icon::Error), *theme::ERROR),
                        StepOutcome::Running => (None, *theme::TEXT),
                        _ if step
                            .messages
                            .iter()
                            .any(|&id| messages[id].kind == MessageKind::Warning) =>
                        {
                            (Some(Icon::Warning), *theme::WARNING)
                        }
                        _ => (Some(Icon::Success), *theme::SUCCESS),
                    };
                    match icon {
                        Some(icon) => icons::paint(
                            ui.painter(),
                            egui::Rect::from_min_size(
                                egui::pos2(x, rect.center().y - 7.0),
                                vec2(14.0, 14.0),
                            ),
                            icon,
                            color,
                        ),
                        None => {
                            let spinner = egui::Rect::from_min_size(
                                egui::pos2(x, rect.center().y - 6.0),
                                vec2(12.0, 12.0),
                            );
                            ui.put(
                                spinner,
                                egui::Spinner::new().size(12.0).color(*theme::ACCENT),
                            );
                        }
                    }
                    x += 20.0;
                    x = row_text(
                        ui,
                        rect,
                        x,
                        &step.name,
                        egui::FontId::proportional(theme::UI_FONT_SIZE),
                        *theme::TEXT,
                    );
                    if let Some(label) = step.outcome.label() {
                        x = row_text(
                            ui,
                            rect,
                            x + 6.0,
                            label,
                            egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                            *theme::TEXT_INFO,
                        );
                    }
                    if let Some(duration) = step.duration().filter(|d| !d.is_zero()) {
                        row_text(
                            ui,
                            rect,
                            x + 8.0,
                            &format_duration(duration),
                            egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                            *theme::TEXT_INFO,
                        );
                    }
                    if response.clicked() {
                        action = Some(Action::Select(node, Some(step.console_line)));
                    }
                    for &id in &step.messages {
                        if let Some(clicked) = message_node(ui, &view, id, &messages[id], 2, &paths)
                        {
                            action = Some(clicked);
                        }
                    }
                }
                if let Some(summary) = events.summary() {
                    let (rect, _) = list_row(ui, false);
                    row_text(
                        ui,
                        rect,
                        rect.left() + 10.0 + TREE_INDENT,
                        summary,
                        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                        *theme::TEXT_INFO,
                    );
                }
            });
    }

    let mut console = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(console_rect.shrink2(vec2(10.0, 6.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let command_line = if plan.program.is_empty() {
        String::new()
    } else {
        plan.command_line()
    };
    if test_mode {
        crate::test_view::header(app, &mut console);
        match crate::test_view::selected_case(app, ui.ctx()) {
            Some(case) => crate::test_view::details(&mut console, &case),
            None => console_view(app, &mut console, &command_line, view.reveal.take()),
        }
    } else {
        console_view(app, &mut console, &command_line, view.reveal.take());
    }

    match action {
        Some(Action::Select(node, reveal)) => {
            view.selected = Some(node);
            view.reveal = reveal;
        }
        Some(Action::Open {
            node,
            path,
            line,
            column,
        }) => {
            view.selected = Some(node);
            app.dispatch("editor.openFile", json!({"path": path}));
            app.dispatch("editor.gotoLine", json!({"line": line, "column": column}));
        }
        None => {}
    }
    ui.data_mut(|data| data.insert_temp(view_id(), view));
}

const MESSAGE_NODE: usize = 1_000_000;

struct Paths<'a> {
    root: Option<&'a Path>,
    cwd: &'a Path,
}

fn message_node(
    ui: &mut egui::Ui,
    view: &View,
    id: usize,
    message: &jetthing_runner::BuildMessage,
    depth: usize,
    paths: &Paths,
) -> Option<Action> {
    let (icon, color) = match message.kind {
        MessageKind::Error => (Icon::Error, *theme::ERROR),
        MessageKind::Warning => (Icon::Warning, *theme::WARNING),
    };
    let location = message.file.as_ref().map(|file| {
        let name = file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        match message.line {
            Some(line) => format!("{name}:{line}"),
            None => name,
        }
    });
    let node = MESSAGE_NODE + id;
    let response = message_row(
        ui,
        depth,
        icon,
        color,
        &message.text,
        location.as_deref(),
        view.selected == Some(node),
    );
    let target = message
        .file
        .as_ref()
        .zip(paths.root)
        .and_then(|(file, root)| project_relative(root, paths.cwd, file));
    if response.clicked()
        && let Some(path) = target
    {
        return Some(Action::Open {
            node,
            path,
            line: message.line.unwrap_or(1),
            column: message.column.unwrap_or(1),
        });
    }
    response
        .clicked()
        .then_some(Action::Select(node, Some(message.console_line)))
}

fn message_row(
    ui: &mut egui::Ui,
    depth: usize,
    icon: Icon,
    color: Color32,
    text: &str,
    location: Option<&str>,
    selected: bool,
) -> egui::Response {
    let (rect, response) = list_row(ui, selected);
    let mut x = rect.left() + 10.0 + TREE_INDENT * depth as f32;
    icons::paint(
        ui.painter(),
        egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 7.0), vec2(14.0, 14.0)),
        icon,
        color,
    );
    x += 20.0;
    x = row_text(
        ui,
        rect,
        x,
        text,
        egui::FontId::proportional(theme::UI_FONT_SIZE),
        theme::TEXT,
    );
    if let Some(location) = location {
        row_text(
            ui,
            rect,
            x + 8.0,
            location,
            egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
            theme::TEXT_INFO,
        );
    }
    response.on_hover_text(text)
}

/// Height for the console viewport that holds whole rows only, so a view
/// stuck to the bottom never shows a half-cut first line under the header.
pub fn whole_rows_height(available: f32, row_height: f32) -> f32 {
    if row_height <= 0.0 {
        return available.max(0.0);
    }
    (available / row_height).floor().max(1.0) * row_height
}

fn console_view(app: &IdeApp, ui: &mut egui::Ui, command_line: &str, reveal: Option<usize>) {
    let console = app.state.runner.console();
    let lines = console.lines();
    let dropped = console.dropped();
    let partial = [OutputStream::Stdout, OutputStream::Stderr]
        .into_iter()
        .map(|stream| (stream, console.partial(stream)))
        .filter(|(_, text)| !text.is_empty())
        .collect::<Vec<_>>();
    // Line 0 echoes the command, like a terminal prompt.
    let total = 1 + lines.len() + partial.len();
    let mut area = egui::ScrollArea::both()
        .id_salt("build_console")
        .auto_shrink([false, false])
        .max_height(whole_rows_height(ui.available_height(), CONSOLE_ROW));
    if let Some(line) = reveal {
        let row = line.saturating_sub(dropped) + 1;
        area = area.vertical_scroll_offset((row as f32 * CONSOLE_ROW - 40.0).max(0.0));
    } else {
        area = area.stick_to_bottom(true);
    }
    let font = egui::FontId::monospace(theme::UI_FONT_SIZE - 0.5);
    if let TaskState::Failed(reason) = app.state.runner.state() {
        // Nothing ran; show the complete, wrapped reason instead of an empty console.
        ui.label(RichText::new(reason).font(font).color(theme::ERROR));
        return;
    }
    area.show_rows(ui, CONSOLE_ROW, total, |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in range {
            let (text, color) = if row == 0 {
                if command_line.is_empty() {
                    continue;
                }
                (format!("$ {command_line}"), *theme::TEXT_INFO)
            } else if let Some(line) = lines.get(row - 1) {
                let color = match line.stream {
                    OutputStream::Stdout => stdout_color(&line.text),
                    OutputStream::Stderr => stderr_color(&line.text),
                };
                (line.text.clone(), color)
            } else if let Some((stream, text)) = partial.get(row - 1 - lines.len()) {
                let color = match stream {
                    OutputStream::Stdout => *theme::editor::TEXT,
                    OutputStream::Stderr => stderr_color(text),
                };
                (text.to_string(), color)
            } else {
                continue;
            };
            let (rect, _) = ui.allocate_exact_size(
                vec2(ui.available_width().max(200.0), CONSOLE_ROW),
                egui::Sense::hover(),
            );
            let galley = ui.painter().layout_no_wrap(text, font.clone(), color);
            let width = galley.size().x;
            ui.painter().galley(
                egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                galley,
                color,
            );
            if width > rect.width() {
                // Extend the content so horizontal scrolling reaches long lines.
                ui.allocate_exact_size(vec2(width - rect.width(), 0.0), egui::Sense::hover());
            }
        }
    });
}

fn stdout_color(text: &str) -> Color32 {
    if text.starts_with("BUILD SUCCESSFUL") {
        *theme::SUCCESS
    } else if text.starts_with("BUILD FAILED") || text.starts_with("FAILURE:") {
        *theme::ERROR
    } else {
        *theme::editor::TEXT
    }
}

/// Tools write progress to stderr; only highlight real errors/warnings.
fn stderr_color(text: &str) -> Color32 {
    let trimmed = text.trim_start();
    if trimmed.starts_with("error") || trimmed.starts_with("ERROR") || trimmed.starts_with("e: ") {
        *theme::ERROR
    } else if trimmed.starts_with("warning")
        || trimmed.starts_with("WARNING")
        || trimmed.starts_with("w: ")
    {
        *theme::WARNING
    } else {
        *theme::editor::TEXT
    }
}

/// Converts a tool-reported path to a project-relative one the editor can open.
pub fn project_relative(root: &Path, cwd: &Path, file: &Path) -> Option<String> {
    let absolute = if file.is_absolute() {
        file.to_owned()
    } else {
        cwd.join(file)
    };
    absolute
        .strip_prefix(root)
        .ok()
        .map(|relative| relative.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stderr_lines_are_only_red_for_errors() {
        assert_eq!(stderr_color("error[E0308]: mismatched"), *theme::ERROR);
        assert_eq!(stderr_color("   Compiling foo"), *theme::editor::TEXT);
        assert_eq!(stdout_color("BUILD SUCCESSFUL in 4s"), *theme::SUCCESS);
    }

    #[test]
    fn viewport_holds_whole_rows() {
        assert_eq!(whole_rows_height(250.0, 18.0), 234.0);
        assert_eq!(whole_rows_height(10.0, 18.0), 18.0);
        assert_eq!(whole_rows_height(100.0, 0.0), 100.0);
    }

    #[test]
    fn durations_read_like_the_reference() {
        assert_eq!(format_duration(Duration::from_millis(818)), "818 ms");
        assert_eq!(
            format_duration(Duration::from_millis(3520)),
            "3 sec, 520 ms"
        );
        assert_eq!(format_duration(Duration::from_secs(125)), "2 min, 5 sec");
        assert_eq!(format_duration(Duration::from_secs(3720)), "1 h, 2 min");
    }

    #[test]
    fn tool_paths_become_project_relative() {
        let root = Path::new("/w/demo");
        assert_eq!(
            project_relative(root, root, Path::new("src/main.rs")).as_deref(),
            Some("src/main.rs")
        );
        assert_eq!(
            project_relative(root, root, Path::new("/w/demo/app/A.java")).as_deref(),
            Some("app/A.java")
        );
        assert_eq!(
            project_relative(root, root, Path::new("/usr/lib/x.h")),
            None
        );
    }
}
