//! Small modal popups: New File/Directory, Rename, Delete, Go to Line, Recent Files.

use crate::icons::{self, Icon};
use crate::widgets::row_text;
use crate::{IdeApp, theme};
use eframe::egui::{self, Rect, RichText, Sense, vec2};
use serde_json::json;
use std::path::{Path, PathBuf};

pub enum Dialog {
    OpenProject {
        path: String,
        error: Option<String>,
    },
    NewEntry {
        parent: PathBuf,
        directory: bool,
        name: String,
    },
    Rename {
        path: PathBuf,
        name: String,
    },
    Delete {
        path: PathBuf,
    },
    GotoLine {
        input: String,
    },
    Recent {
        query: String,
        selected: usize,
    },
}

fn input_id() -> egui::Id {
    egui::Id::new("dialog_input")
}

/// Parses "line[:column]" (1-based) as typed into Go to Line.
pub fn parse_line_column(input: &str) -> Option<(usize, Option<usize>)> {
    let mut parts = input.trim().splitn(2, [':', ',']);
    let line = parts.next()?.trim().parse::<usize>().ok()?;
    let column = match parts.next() {
        Some(column) if !column.trim().is_empty() => Some(column.trim().parse::<usize>().ok()?),
        _ => None,
    };
    Some((line, column))
}

fn popup_frame(
    ctx: &egui::Context,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.center().x - width / 2.0,
            screen.top() + screen.height() * 0.2,
        ))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(10)
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(width - 24.0);
                    add(ui);
                });
        })
        .response
}

fn title(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(theme::semibold(theme::UI_FONT_SIZE))
            .color(theme::TEXT_BRIGHT),
    );
    ui.add_space(6.0);
}

fn name_field(ui: &mut egui::Ui, text: &mut String, focus: bool) -> (bool, bool) {
    // TextEdit consumes Enter itself; capture submission before drawing it.
    let submit = ui.memory(|memory| memory.has_focus(input_id()))
        && ui.input(|input| input.key_pressed(egui::Key::Enter));
    let response = ui.add(
        egui::TextEdit::singleline(text)
            .id(input_id())
            .desired_width(f32::INFINITY)
            .margin(egui::Margin::symmetric(6, 5)),
    );
    if focus {
        response.request_focus();
    }
    (submit, response.changed())
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(mut dialog) = app.dialog.take() else {
        return;
    };
    let focus = std::mem::take(&mut app.dialog_focus);
    let mut keep = true;
    let mut error = None;
    match &mut dialog {
        Dialog::OpenProject {
            path,
            error: open_error,
        } => {
            let mut submit = false;
            popup_frame(ctx, "dialog_open_project", 520.0, |ui| {
                title(ui, "Open Project");
                ui.label("Project directory");
                let changed;
                (submit, changed) = name_field(ui, path, focus);
                if changed {
                    *open_error = None;
                }
                ui.label(
                    RichText::new("Enter the full path to an existing folder.")
                        .color(theme::TEXT_INFO),
                );
                if let Some(message) = open_error.as_ref() {
                    ui.label(RichText::new(message).color(theme::ERROR));
                }
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!path.trim().is_empty(), egui::Button::new("Open"))
                        .clicked()
                    {
                        submit = true;
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
            if submit && keep && !path.trim().is_empty() {
                match app.open_project_path(path, ctx) {
                    Ok(()) => keep = false,
                    Err(message) => *open_error = Some(message),
                }
            }
        }
        Dialog::NewEntry {
            parent,
            directory,
            name,
        } => {
            let label = if *directory {
                "New Directory"
            } else {
                "New File"
            };
            let mut submit = false;
            popup_frame(ctx, "dialog_new", 360.0, |ui| {
                title(ui, label);
                (submit, _) = name_field(ui, name, focus);
                let hint = if *directory {
                    "Name, or a path like a/b/c"
                } else {
                    "Name with extension, or a path like src/lib.rs"
                };
                ui.label(RichText::new(hint).color(theme::TEXT_INFO));
            });
            if submit && !name.trim().is_empty() {
                let path = parent.join(name.trim());
                let id = if *directory {
                    "project.newDirectory"
                } else {
                    "project.newFile"
                };
                match app
                    .commands
                    .execute(id, &mut app.state, json!({"path": path}))
                {
                    Ok(_) => {
                        keep = false;
                        app.tree_selection = Some(path);
                        if !*directory {
                            ctx.memory_mut(|m| m.request_focus(crate::editor::editor_id()));
                        }
                    }
                    Err(e) => error = Some(e.to_string()),
                }
            }
        }
        Dialog::Rename { path, name } => {
            let mut submit = false;
            let label = format!("Rename {}", IdeApp::file_name(path));
            popup_frame(ctx, "dialog_rename", 360.0, |ui| {
                title(ui, &label);
                (submit, _) = name_field(ui, name, focus);
            });
            if submit && !name.trim().is_empty() {
                match app.commands.execute(
                    "project.rename",
                    &mut app.state,
                    json!({"path": path, "name": name.trim()}),
                ) {
                    Ok(result) => {
                        keep = false;
                        app.tree_selection = result["path"].as_str().map(PathBuf::from);
                    }
                    Err(e) => error = Some(e.to_string()),
                }
            }
        }
        Dialog::Delete { path } => {
            let mut confirm =
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let mut cancel = false;
            let is_dir = app
                .state
                .files
                .iter()
                .any(|e| e.is_dir && e.relative_path == *path);
            popup_frame(ctx, "dialog_delete", 380.0, |ui| {
                title(ui, "Delete");
                let what = if is_dir { "directory" } else { "file" };
                ui.label(format!("Delete {what} \"{}\"?", path.display()));
                if is_dir {
                    ui.label(
                        RichText::new("All files inside will be deleted.").color(theme::TEXT_INFO),
                    );
                }
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let delete =
                        egui::Button::new(RichText::new("Delete").color(egui::Color32::WHITE))
                            .fill(theme::ACCENT);
                    if ui.add(delete).clicked() {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
            if cancel {
                keep = false;
            } else if confirm {
                keep = false;
                app.dispatch("project.delete", json!({"path": path}));
                app.tree_selection = None;
            }
        }
        Dialog::GotoLine { input } => {
            let mut submit = false;
            let caret = app
                .state
                .active_document
                .and_then(|index| app.state.documents.get(index))
                .and_then(|d| d.text.char_to_line_column(d.selections.primary().head).ok())
                .map(|(l, c)| format!("{}:{}", l + 1, c + 1))
                .unwrap_or_default();
            popup_frame(ctx, "dialog_goto", 300.0, |ui| {
                title(ui, "Go to Line:Column");
                (submit, _) = name_field(ui, input, focus);
                ui.label(
                    RichText::new(format!("Current position {caret}")).color(theme::TEXT_INFO),
                );
            });
            if submit {
                match parse_line_column(input) {
                    Some((line, column)) => {
                        let mut params = json!({"line": line});
                        if let Some(column) = column {
                            params["column"] = json!(column);
                        }
                        app.dispatch("editor.gotoLine", params);
                        ctx.memory_mut(|m| m.request_focus(crate::editor::editor_id()));
                        keep = false;
                    }
                    None => {
                        error = Some("Enter a line number, optionally followed by :column".into())
                    }
                }
            }
        }
        Dialog::Recent { query, selected } => {
            let entries: Vec<PathBuf> = app
                .state
                .recent_files
                .iter()
                .filter(|path| {
                    query.is_empty()
                        || crate::palette::fuzzy_score(&IdeApp::file_name(path), query).is_some()
                })
                .take(30)
                .cloned()
                .collect();
            if !entries.is_empty() {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                    *selected = (*selected + 1) % entries.len();
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                    *selected = (*selected + entries.len() - 1) % entries.len();
                }
            }
            *selected = (*selected).min(entries.len().saturating_sub(1));
            let mut open = ctx
                .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
                .then_some(*selected);
            popup_frame(ctx, "dialog_recent", 520.0, |ui| {
                ui.horizontal(|ui| {
                    title(ui, "Recent Files");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(query)
                                .id(input_id())
                                .frame(false)
                                .desired_width(160.0)
                                .hint_text("Type to filter"),
                        )
                        .request_focus();
                    });
                });
                if entries.is_empty() {
                    ui.label(RichText::new("No recent files").color(theme::TEXT_SECONDARY));
                }
                ui.spacing_mut().item_spacing.y = 0.0;
                for (index, path) in entries.iter().enumerate() {
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
                    if index == *selected {
                        ui.painter().rect_filled(rect, 4.0, theme::SELECTION);
                    } else if response.hovered() {
                        ui.painter().rect_filled(rect, 4.0, theme::HOVER);
                    }
                    let name = IdeApp::file_name(path);
                    icons::paint(
                        ui.painter(),
                        Rect::from_min_size(rect.left_center() + vec2(8.0, -8.0), vec2(16.0, 16.0)),
                        Icon::for_file(&name),
                        icons::ICON,
                    );
                    let x = row_text(
                        ui,
                        rect,
                        rect.left() + 32.0,
                        &name,
                        egui::FontId::proportional(theme::UI_FONT_SIZE),
                        theme::TEXT_BRIGHT,
                    );
                    let parent = path
                        .parent()
                        .map(Path::display)
                        .map(|d| d.to_string())
                        .unwrap_or_default();
                    row_text(
                        ui,
                        rect,
                        x + 10.0,
                        &parent,
                        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                        theme::TEXT_INFO,
                    );
                    if response.clicked() {
                        open = Some(index);
                    }
                }
            });
            if let Some(index) = open
                && let Some(path) = entries.get(index)
            {
                app.dispatch("editor.openFile", json!({"path": path}));
                ctx.memory_mut(|m| m.request_focus(crate::editor::editor_id()));
                keep = false;
            }
        }
    }
    if let Some(error) = error {
        app.state.status = error;
    }
    if keep {
        app.dialog = Some(dialog);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_name_field_submits_on_enter() {
        let ctx = egui::Context::default();
        let mut text = String::from("/project");
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                name_field(ui, &mut text, true);
            });
        });
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut submitted = false;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                submitted = name_field(ui, &mut text, false).0;
            });
        });
        assert!(submitted);
    }

    #[test]
    fn parses_line_and_column() {
        assert_eq!(parse_line_column("12"), Some((12, None)));
        assert_eq!(parse_line_column(" 12:5 "), Some((12, Some(5))));
        assert_eq!(parse_line_column("3,"), Some((3, None)));
        assert_eq!(parse_line_column("x"), None);
        assert_eq!(parse_line_column("4:y"), None);
    }
}
