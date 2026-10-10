//! Editor island: tabs, viewport-native text surface, and completion popup.

use crate::icons::{self, Icon};
use crate::widgets::island_frame;
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, Sense, Stroke, vec2};
use jetthing_core::Document;
use jetthing_language::LanguageAnalysis;
use jetthing_text::Selection;
use jetthing_text::ops;
use serde_json::{Value, json};

pub const COMPLETION_LIMIT: usize = 30;
const TAB_BAR_HEIGHT: f32 = 38.0;
const ROW_HEIGHT: f32 = 20.0;
const TEXT_LEFT_PADDING: f32 = 14.0;

/// Work the editor surface hands back to the app: named actions go through the
/// command registry; clipboard requests need the platform clipboard.
pub enum EditorRequest {
    Command(&'static str, Value),
    Copy,
    Cut,
    Paste(String),
}

pub struct EditorOutput {
    pub changed: bool,
    pub caret_pos: Option<egui::Pos2>,
    pub requests: Vec<EditorRequest>,
}

pub fn editor_id() -> egui::Id {
    egui::Id::new("jetthing_editor_surface")
}

pub fn central(app: &mut IdeApp, ctx: &egui::Context) {
    let mut changed_document = None;
    let mut requests = Vec::new();
    egui::CentralPanel::default()
        .frame(island_frame())
        .show(ctx, |ui| {
            tabs(app, ui);
            if app.state.active_document.is_none() {
                empty_state(ui);
                app.caret_screen_pos = None;
                return;
            }
            if app.find.open {
                crate::findbar::show(app, ui);
            }
            let Some(index) = app.state.active_document else {
                return;
            };
            let diagnostics = app.state.active_lsp_diagnostics();
            let highlights = if app.find.open {
                app.find.matches(&app.state.documents[index].text.text())
            } else {
                Vec::new()
            };
            let doc = &mut app.state.documents[index];
            match editor_surface(ui, doc, &diagnostics, &highlights) {
                Ok(EditorOutput {
                    changed,
                    caret_pos,
                    requests: editor_requests,
                }) => {
                    app.caret_screen_pos = caret_pos;
                    requests = editor_requests;
                    if changed {
                        let snapshot = doc.text.text();
                        doc.refresh_analysis(&snapshot);
                        changed_document = Some((
                            doc.path.clone(),
                            snapshot,
                            doc.text.revision().min(i32::MAX as u64) as i32,
                        ));
                    }
                }
                Err(error) => app.state.status = error,
            }
        });
    if let Some((path, text, version)) = changed_document {
        app.state.sync_document_lsp(&path, &text, version);
    }
    for request in requests {
        handle_request(app, ctx, request);
    }
}

fn handle_request(app: &mut IdeApp, ctx: &egui::Context, request: EditorRequest) {
    match request {
        EditorRequest::Command(id, params) => app.dispatch(id, params),
        EditorRequest::Copy | EditorRequest::Cut => {
            let id = if matches!(request, EditorRequest::Copy) {
                "editor.copy"
            } else {
                "editor.cut"
            };
            match app.commands.execute(id, &mut app.state, Value::Null) {
                Ok(result) => {
                    let text = result["text"].as_str().unwrap_or_default().to_owned();
                    let whole_line = result["wholeLine"].as_bool().unwrap_or(false);
                    app.clipboard_line = whole_line.then(|| text.clone());
                    ctx.copy_text(text);
                }
                Err(error) => app.state.status = error.to_string(),
            }
        }
        EditorRequest::Paste(text) => {
            let whole_line = app.clipboard_line.as_deref() == Some(text.as_str());
            app.dispatch(
                "editor.paste",
                json!({"text": text.replace("\r\n", "\n"), "wholeLine": whole_line}),
            );
        }
    }
}

fn empty_state(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let lines = [
        ("Search Everywhere", "Double ⇧"),
        ("Find Action", "⇧⌘A"),
        ("Project View", "⌘1"),
        ("Find in Files", "⇧⌘F"),
        ("Terminal", "⌥F12"),
    ];
    let mut y = rect.center().y - lines.len() as f32 * 14.0;
    for (label, shortcut) in lines {
        let painter = ui.painter();
        painter.text(
            egui::pos2(rect.center().x - 8.0, y),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::proportional(theme::UI_FONT_SIZE),
            theme::TEXT_SECONDARY,
        );
        painter.text(
            egui::pos2(rect.center().x + 8.0, y),
            egui::Align2::LEFT_CENTER,
            crate::keys::display(shortcut),
            egui::FontId::proportional(theme::UI_FONT_SIZE),
            theme::MODIFIED,
        );
        y += 28.0;
    }
}

fn tabs(app: &mut IdeApp, ui: &mut egui::Ui) {
    let (bar, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), TAB_BAR_HEIGHT), Sense::hover());
    let mut x = bar.left() + 6.0;
    let mut activate = None;
    let mut close = None;
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE);
    for index in 0..app.state.documents.len() {
        let doc = &app.state.documents[index];
        let name = IdeApp::file_name(&doc.path);
        let active = app.state.active_document == Some(index);
        let color = if active {
            theme::TEXT_BRIGHT
        } else {
            theme::TEXT_SECONDARY
        };
        let galley = ui
            .painter()
            .layout_no_wrap(name.clone(), font.clone(), color);
        let width = 10.0 + 16.0 + 6.0 + galley.size().x + 6.0 + 16.0 + 8.0;
        let rect = Rect::from_min_size(egui::pos2(x, bar.top() + 5.0), vec2(width, 28.0));
        x += width + 2.0;
        if rect.left() > bar.right() {
            break;
        }
        let response = ui.interact(rect, egui::Id::new(("editor_tab", index)), Sense::click());
        let fill = if active {
            Color32::from_rgb(0x23, 0x35, 0x58) // EditorTabs.underlinedTabBackground
        } else if response.hovered() {
            theme::HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, 6.0, fill);
        if active {
            ui.painter().rect_stroke(
                rect,
                6.0,
                Stroke::new(1.0_f32, Color32::from_rgb(0x2E, 0x4D, 0x89)), // EditorTabs.underlinedBorderColor
                egui::StrokeKind::Inside,
            );
        }
        let layout = TabLayout::new(rect, galley.size());
        icons::paint(
            ui.painter(),
            layout.icon,
            Icon::for_file(&name),
            icons::ICON,
        );
        ui.painter().galley(layout.text.min, galley, color);
        let close_rect = layout.close;
        let dirty = doc.text.is_dirty();
        let pointer_on_close = response
            .hover_pos()
            .is_some_and(|pos| close_rect.expand(2.0).contains(pos));
        if response.hovered() || active {
            if pointer_on_close {
                ui.painter()
                    .rect_filled(close_rect.expand(2.0), 4.0, theme::HOVER);
            }
            icons::paint(
                ui.painter(),
                close_rect.shrink(2.0),
                Icon::Close,
                theme::TEXT_SECONDARY,
            );
        } else if dirty {
            ui.painter()
                .circle_filled(close_rect.center(), 3.0, theme::TEXT_SECONDARY);
        }
        if dirty && (response.hovered() || active) && !pointer_on_close {
            ui.painter()
                .circle_filled(layout.modified_dot, 2.5, theme::MODIFIED);
        }
        let response = response.on_hover_text(doc.path.display().to_string());
        if response.clicked() {
            if pointer_on_close {
                close = Some(index);
            } else {
                activate = Some(index);
            }
        }
        if response.middle_clicked() {
            close = Some(index);
        }
    }
    if let Some(index) = close {
        app.dispatch("editor.close", json!({"index": index}));
    } else if let Some(index) = activate {
        app.dispatch("editor.activate", json!({"index": index}));
        ui.memory_mut(|memory| memory.request_focus(editor_id()));
    }
}

/// Geometry of one editor tab: file icon, title, close button, and the
/// modified badge shown while the close button is visible. The badge sits on
/// the icon's top-right corner so it never covers the title.
struct TabLayout {
    icon: Rect,
    text: Rect,
    close: Rect,
    modified_dot: egui::Pos2,
}

impl TabLayout {
    fn new(rect: Rect, text_size: egui::Vec2) -> Self {
        let icon = Rect::from_min_size(rect.left_center() + vec2(10.0, -8.0), vec2(16.0, 16.0));
        let text = Rect::from_min_size(
            rect.left_center() + vec2(32.0, -text_size.y / 2.0),
            text_size,
        );
        let close = Rect::from_min_size(
            egui::pos2(rect.right() - 24.0, rect.center().y - 8.0),
            vec2(16.0, 16.0),
        );
        Self {
            icon,
            text,
            close,
            modified_dot: icon.right_top() + vec2(-1.0, 2.0),
        }
    }
}

fn highlighted_line_job(
    text: &str,
    line_start_byte: usize,
    analysis: &LanguageAnalysis,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let base = egui::TextFormat {
        font_id: egui::FontId::monospace(theme::EDITOR_FONT_SIZE),
        color: theme::editor::TEXT,
        ..Default::default()
    };
    let line_end_byte = line_start_byte + text.len();
    let mut cursor = line_start_byte;
    for span in analysis.highlights.in_range(line_start_byte..line_end_byte) {
        let (start, end) = (span.start_byte, span.end_byte);
        if start < cursor
            || end < start
            || !text.is_char_boundary(start - line_start_byte)
            || !text.is_char_boundary(end - line_start_byte)
        {
            continue;
        }
        job.append(
            &text[cursor - line_start_byte..start - line_start_byte],
            0.0,
            base.clone(),
        );
        let fragment = &text[start - line_start_byte..end - line_start_byte];
        job.append(fragment, 0.0, crate::syntax::text_format(span.key, &base));
        cursor = end;
    }
    job.append(&text[cursor - line_start_byte..], 0.0, base);
    job
}

/// Squiggle underline used for errors and warnings.
fn squiggle(painter: &egui::Painter, x1: f32, x2: f32, y: f32, color: Color32) {
    let step = 2.0;
    let mut points = Vec::new();
    let mut x = x1;
    let mut up = true;
    while x <= x2.max(x1 + 6.0) {
        points.push(egui::pos2(x, if up { y - 1.5 } else { y + 0.5 }));
        x += step;
        up = !up;
    }
    painter.add(egui::Shape::line(points, Stroke::new(1.0_f32, color)));
}

/// Moves the primary caret to a zero-based line/column and focuses the editor.
pub fn move_caret_to(app: &mut IdeApp, line: usize, column: usize) {
    let Some(index) = app.state.active_document else {
        return;
    };
    let doc = &mut app.state.documents[index];
    let line = line.min(doc.text.len_lines().saturating_sub(1));
    let line_text = doc.text.line(line).unwrap_or_default();
    let length = line_text.trim_end_matches(['\r', '\n']).chars().count();
    let offset = doc.text.line_to_char(line).unwrap_or(0) + column.min(length);
    doc.selections.set_single(Selection::caret(offset));
    app.state.status.clear();
}

fn line_length(document: &Document, line: usize) -> usize {
    document
        .text
        .line(line)
        .unwrap_or_default()
        .trim_end_matches(['\r', '\n'])
        .chars()
        .count()
}

/// Character offset for moving `head` vertically by `delta` lines, keeping the column.
fn vertical_target(document: &Document, head: usize, delta: isize) -> usize {
    let Ok((line, column)) = document.text.char_to_line_column(head) else {
        return head;
    };
    let last = document.text.len_lines().saturating_sub(1) as isize;
    let target = (line as isize + delta).clamp(0, last) as usize;
    if target == line {
        return if delta < 0 {
            document.text.line_to_char(line).unwrap_or(0)
        } else {
            document.text.line_to_char(line).unwrap_or(0) + line_length(document, line)
        };
    }
    document.text.line_to_char(target).unwrap_or(0) + column.min(line_length(document, target))
}

fn move_primary(document: &mut Document, head: usize, extend: bool) {
    let current = document.selections.primary();
    let anchor = if extend { current.anchor } else { head };
    document.selections.set_single(Selection { anchor, head });
}

fn editor_surface(
    ui: &mut egui::Ui,
    document: &mut Document,
    lsp_diagnostics: &[jetthing_lsp::LspDiagnostic],
    highlights: &[(usize, usize)],
) -> Result<EditorOutput, String> {
    let mut requests = Vec::new();
    let font = egui::FontId::monospace(theme::EDITOR_FONT_SIZE);
    let char_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'm'));
    let line_count = document.text.len_lines();
    let digits = line_count.max(1).to_string().len().max(2);
    let gutter_width = 20.0 + digits as f32 * char_width + 18.0;
    let text_left = gutter_width + TEXT_LEFT_PADDING;
    let caret_line = document
        .text
        .char_to_line_column(document.selections.primary().head)
        .map(|(line, _)| line)
        .unwrap_or(0);
    let mut caret_pos = None;
    let brace_pair: Vec<usize> = {
        let head = document.selections.primary().head;
        ops::matching_bracket(&document.text, head)
            .or_else(|| {
                head.checked_sub(1)
                    .and_then(|h| ops::matching_bracket(&document.text, h))
            })
            .into_iter()
            .flat_map(|(a, b)| [a, b])
            .collect()
    };
    let output = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, viewport| {
            let longest = 400.0_f32;
            let content_size = egui::vec2(
                ui.available_width().max(text_left + longest * char_width),
                line_count as f32 * ROW_HEIGHT + ui.available_height() * 0.5,
            );
            let (rect, response) = ui.allocate_exact_size(content_size, Sense::click_and_drag());
            let response = ui
                .interact(rect, editor_id(), Sense::click_and_drag())
                .union(response)
                .on_hover_cursor(egui::CursorIcon::Text);
            if response.clicked() || response.drag_started() {
                response.request_focus();
            }
            let position_to_offset = |position: egui::Pos2| {
                let line = (((position.y - rect.top()) / ROW_HEIGHT).floor().max(0.0) as usize)
                    .min(line_count.saturating_sub(1));
                let line_text = document.text.line(line).unwrap_or_default();
                let content = line_text.trim_end_matches(['\r', '\n']);
                let column = (((position.x - rect.left() - text_left) / char_width)
                    .round()
                    .max(0.0) as usize)
                    .min(content.chars().count());
                document.text.line_to_char(line).unwrap_or(0) + column
            };
            if response.clicked()
                && let Some(position) = response.interact_pointer_pos()
            {
                let offset = position_to_offset(position);
                let modifiers = ui.input(|input| input.modifiers);
                if modifiers.alt {
                    document.selections.add(Selection::caret(offset));
                } else if modifiers.shift {
                    let anchor = document.selections.primary().anchor;
                    document.selections.set_single(Selection {
                        anchor,
                        head: offset,
                    });
                } else {
                    document.selections.set_single(Selection::caret(offset));
                }
            }
            if response.double_clicked()
                && let Some(position) = response.interact_pointer_pos()
            {
                let (start, end) = ops::word_at(&document.text, position_to_offset(position));
                document.selections.set_single(Selection {
                    anchor: start,
                    head: end,
                });
            }
            if response.triple_clicked()
                && let Some(position) = response.interact_pointer_pos()
            {
                let line = document.text.char_to_line(position_to_offset(position));
                let (start, _, full_end) = ops::line_bounds(&document.text, line);
                document.selections.set_single(Selection {
                    anchor: start,
                    head: full_end,
                });
            }
            if response.drag_started()
                && let Some(position) = response.interact_pointer_pos()
            {
                document
                    .selections
                    .set_single(Selection::caret(position_to_offset(position)));
            }
            response.context_menu(|ui| {
                ui.set_min_width(220.0);
                for (label, shortcut, request) in [
                    ("Cut", "⌘X", EditorRequest::Cut),
                    ("Copy", "⌘C", EditorRequest::Copy),
                ] {
                    if ui
                        .add(egui::Button::new(label).shortcut_text(crate::keys::display(shortcut)))
                        .clicked()
                    {
                        requests.push(request);
                        ui.close();
                    }
                }
                if ui
                    .add(egui::Button::new("Paste").shortcut_text(crate::keys::display("⌘V")))
                    .clicked()
                {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                    ui.close();
                }
                ui.separator();
                for (label, shortcut, id) in [
                    ("Comment with Line Comment", "⌘/", "editor.commentLine"),
                    ("Duplicate Line or Selection", "⌘D", "editor.duplicate"),
                    ("Delete Line", "⌘⌫", "editor.deleteLine"),
                    ("Move Line Up", "⌥⇧↑", "editor.moveLineUp"),
                    ("Move Line Down", "⌥⇧↓", "editor.moveLineDown"),
                ] {
                    if ui
                        .add(egui::Button::new(label).shortcut_text(crate::keys::display(shortcut)))
                        .clicked()
                    {
                        requests.push(EditorRequest::Command(id, Value::Null));
                        ui.close();
                    }
                }
                ui.separator();
                if ui
                    .add(egui::Button::new("Select All").shortcut_text(crate::keys::display("⌘A")))
                    .clicked()
                {
                    requests.push(EditorRequest::Command("editor.selectAll", Value::Null));
                    ui.close();
                }
            });
            if response.dragged()
                && let Some(position) = response.interact_pointer_pos()
            {
                let offset = position_to_offset(position);
                let anchor = document.selections.primary().anchor;
                document.selections.set_single(Selection {
                    anchor,
                    head: offset,
                });
            }

            let painter = ui.painter().clone();
            // Gutter background matches the editor in Islands; draw caret row first.
            let first_line =
                ((viewport.top() / ROW_HEIGHT).floor().max(0.0) as usize).min(line_count);
            let last_line = ((viewport.bottom() / ROW_HEIGHT).ceil() as usize + 1).min(line_count);
            let focused = response.has_focus();
            let blink_on = {
                let time = ui.input(|input| input.time);
                (time * 2.0) as i64 % 2 == 0
            };
            for line in first_line..last_line {
                let y = rect.top() + line as f32 * ROW_HEIGHT;
                let line_start = document.text.line_to_char(line).unwrap_or(0);
                let line_text = document.text.line(line).unwrap_or_default();
                let content = line_text.trim_end_matches(['\r', '\n']);
                let line_end = line_start + content.chars().count();
                let row =
                    Rect::from_min_size(egui::pos2(rect.left(), y), vec2(rect.width(), ROW_HEIGHT));
                if line == caret_line && document.selections.primary().is_caret() {
                    painter.rect_filled(row, 0.0, theme::editor::CARET_ROW);
                }
                painter.text(
                    egui::pos2(rect.left() + gutter_width - 18.0, y + ROW_HEIGHT / 2.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{}", line + 1),
                    font.clone(),
                    if line == caret_line {
                        theme::editor::LINE_NUMBER_CARET
                    } else {
                        theme::editor::LINE_NUMBER
                    },
                );
                // Indent guides every four columns of leading whitespace.
                let indent = content.chars().take_while(|c| *c == ' ').count();
                for level in (4..=indent.saturating_sub(1)).step_by(4) {
                    let x = rect.left() + text_left + level as f32 * char_width;
                    painter.line_segment(
                        [egui::pos2(x, y), egui::pos2(x, y + ROW_HEIGHT)],
                        Stroke::new(1.0_f32, theme::editor::INDENT_GUIDE),
                    );
                }
                for &(match_start, match_end) in highlights {
                    let start = match_start.max(line_start).min(line_end);
                    let end = match_end.max(line_start).min(line_end);
                    if start < end {
                        let x1 = rect.left() + text_left + (start - line_start) as f32 * char_width;
                        let x2 = rect.left() + text_left + (end - line_start) as f32 * char_width;
                        painter.rect_filled(
                            Rect::from_min_max(egui::pos2(x1, y), egui::pos2(x2, y + ROW_HEIGHT)),
                            2.0,
                            theme::editor::SEARCH_RESULT,
                        );
                    }
                }
                for &offset in &brace_pair {
                    if offset >= line_start && offset < line_end {
                        let x1 =
                            rect.left() + text_left + (offset - line_start) as f32 * char_width;
                        painter.rect_filled(
                            Rect::from_min_size(egui::pos2(x1, y), vec2(char_width, ROW_HEIGHT)),
                            2.0,
                            theme::editor::MATCHED_BRACE,
                        );
                    }
                }
                for selection in document.selections.selections() {
                    let start = selection.start().max(line_start).min(line_end);
                    let end = selection.end().max(line_start).min(line_end);
                    let spans_newline = selection.end() > line_end && selection.start() <= line_end;
                    if start < end || (spans_newline && !selection.is_caret()) {
                        let x1 = rect.left() + text_left + (start - line_start) as f32 * char_width;
                        let mut x2 =
                            rect.left() + text_left + (end - line_start) as f32 * char_width;
                        if spans_newline {
                            x2 += char_width * 0.6;
                        }
                        painter.rect_filled(
                            Rect::from_min_max(egui::pos2(x1, y), egui::pos2(x2, y + ROW_HEIGHT)),
                            2.0,
                            theme::editor::SELECTION,
                        );
                    }
                }
                let line_start_byte = document.text.line_to_byte(line).unwrap_or(0);
                let job = highlighted_line_job(content, line_start_byte, &document.analysis);
                let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                let text_y = y + (ROW_HEIGHT - galley.size().y) / 2.0;
                painter.galley(
                    egui::pos2(rect.left() + text_left, text_y),
                    galley,
                    Color32::WHITE,
                );

                let column_x = |column: usize| rect.left() + text_left + column as f32 * char_width;
                for diagnostic in &document.analysis.diagnostics {
                    if diagnostic.row == line + 1 {
                        let start = diagnostic.column.saturating_sub(1);
                        squiggle(
                            &painter,
                            column_x(start),
                            column_x(start + 1),
                            y + ROW_HEIGHT - 2.0,
                            theme::editor::ERROR_EFFECT,
                        );
                    }
                }
                for diagnostic in lsp_diagnostics {
                    if diagnostic.start_line <= line && diagnostic.end_line >= line {
                        let start = if diagnostic.start_line == line {
                            diagnostic.start_character
                        } else {
                            0
                        };
                        let end = if diagnostic.end_line == line {
                            diagnostic.end_character
                        } else {
                            content.chars().count()
                        };
                        let color = if diagnostic.severity == Some(1) {
                            theme::editor::ERROR_EFFECT
                        } else {
                            theme::editor::WARNING_EFFECT
                        };
                        squiggle(
                            &painter,
                            column_x(start),
                            column_x(end.max(start + 1)),
                            y + ROW_HEIGHT - 2.0,
                            color,
                        );
                    }
                }
                for selection in document.selections.selections() {
                    if selection.head >= line_start && selection.head <= line_end {
                        let x = column_x(selection.head - line_start);
                        if *selection == document.selections.primary() {
                            caret_pos = Some(egui::pos2(x, y + ROW_HEIGHT));
                        }
                        if focused && blink_on {
                            painter.line_segment(
                                [egui::pos2(x, y + 2.0), egui::pos2(x, y + ROW_HEIGHT - 2.0)],
                                Stroke::new(2.0_f32, theme::editor::CARET),
                            );
                        }
                    }
                }
            }
            if focused {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(500));
            }
            // Keep the primary caret visible after keyboard navigation.
            if focused
                && ui.input(|input| !input.events.is_empty())
                && let Some(pos) = caret_pos
            {
                let caret_rect =
                    Rect::from_min_size(pos - vec2(0.0, ROW_HEIGHT), vec2(char_width, ROW_HEIGHT));
                if !viewport
                    .translate(rect.min.to_vec2())
                    .contains_rect(caret_rect)
                {
                    ui.scroll_to_rect(caret_rect.expand2(vec2(char_width * 4.0, ROW_HEIGHT)), None);
                }
            }
            response
        });

    let response = output.inner;
    if !response.has_focus() {
        return Ok(EditorOutput {
            changed: false,
            caret_pos,
            requests,
        });
    }
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            response.id,
            egui::EventFilter {
                tab: true,
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: false,
            },
        )
    });
    let events = ui.input(|input| input.events.clone());
    let initial_revision = document.text.revision();
    let page = (ui.available_height() / ROW_HEIGHT).max(1.0) as isize;
    let mac = cfg!(target_os = "macos");
    let text_error = |error: jetthing_text::TextError| error.to_string();
    for event in events {
        let (key, modifiers) = match event {
            egui::Event::Text(text) if !text.is_empty() => {
                let mut chars = text.chars();
                let single = chars.next().filter(|_| chars.next().is_none());
                let handled = match single {
                    Some(c) => ops::type_char(&mut document.text, &mut document.selections, c)
                        .map_err(text_error)?,
                    None => false,
                };
                if !handled {
                    document
                        .selections
                        .insert(&mut document.text, &text)
                        .map_err(text_error)?;
                }
                continue;
            }
            egui::Event::Copy => {
                requests.push(EditorRequest::Copy);
                continue;
            }
            egui::Event::Cut => {
                requests.push(EditorRequest::Cut);
                continue;
            }
            egui::Event::Paste(text) if !text.is_empty() => {
                requests.push(EditorRequest::Paste(text));
                continue;
            }
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => (key, modifiers),
            _ => continue,
        };
        let head = document.selections.primary().head;
        let len = document.text.len_chars();
        let extend = modifiers.shift;
        // Word-wise movement: ⌥ on macOS, Ctrl elsewhere.
        let word = if mac { modifiers.alt } else { modifiers.ctrl };
        let command = |id: &'static str| EditorRequest::Command(id, Value::Null);
        match key {
            egui::Key::D if modifiers.command => requests.push(command("editor.duplicate")),
            egui::Key::Y if modifiers.command && !mac => {
                requests.push(command("editor.deleteLine"))
            }
            egui::Key::Backspace if modifiers.command && mac => {
                requests.push(command("editor.deleteLine"))
            }
            egui::Key::Slash if modifiers.command => requests.push(command("editor.commentLine")),
            // Extend/Shrink Selection: Ctrl+W / Ctrl+Shift+W, ⌥↑ / ⌥↓ on macOS.
            egui::Key::W if modifiers.ctrl && !mac => requests.push(command(if modifiers.shift {
                "editor.shrinkSelection"
            } else {
                "editor.extendSelection"
            })),
            egui::Key::ArrowUp if mac && modifiers.alt && !modifiers.shift => {
                requests.push(command("editor.extendSelection"))
            }
            egui::Key::ArrowDown if mac && modifiers.alt && !modifiers.shift => {
                requests.push(command("editor.shrinkSelection"))
            }
            egui::Key::ArrowUp if modifiers.alt && modifiers.shift => {
                requests.push(command("editor.moveLineUp"))
            }
            egui::Key::ArrowDown if modifiers.alt && modifiers.shift => {
                requests.push(command("editor.moveLineDown"))
            }
            egui::Key::Enter if modifiers.shift => {
                // Start New Line: open a line below without splitting the current one.
                let line = document.text.char_to_line(head);
                let (_, content_end, _) = ops::line_bounds(&document.text, line);
                document
                    .selections
                    .set_single(Selection::caret(content_end));
                ops::newline(&mut document.text, &mut document.selections).map_err(text_error)?;
            }
            egui::Key::Enter => {
                ops::newline(&mut document.text, &mut document.selections).map_err(text_error)?
            }
            egui::Key::Tab if modifiers.shift => {
                requests.push(command("editor.unindent"));
            }
            egui::Key::Tab => {
                let primary = document.selections.primary();
                let multi_line = document.text.char_to_line(primary.start())
                    != document.text.char_to_line(primary.end());
                if multi_line {
                    requests.push(command("editor.indent"));
                } else {
                    document
                        .selections
                        .insert(&mut document.text, "    ")
                        .map_err(text_error)?;
                }
            }
            egui::Key::Backspace if word && document.selections.primary().is_caret() => {
                let target = ops::word_left(&document.text, head);
                document.selections.set_single(Selection {
                    anchor: head,
                    head: target,
                });
                document
                    .selections
                    .insert(&mut document.text, "")
                    .map_err(text_error)?;
            }
            egui::Key::Backspace => {
                ops::backspace(&mut document.text, &mut document.selections).map_err(text_error)?
            }
            egui::Key::Delete if word && document.selections.primary().is_caret() => {
                let target = ops::word_right(&document.text, head);
                document.selections.set_single(Selection {
                    anchor: head,
                    head: target,
                });
                document
                    .selections
                    .insert(&mut document.text, "")
                    .map_err(text_error)?;
            }
            egui::Key::Delete => document
                .selections
                .delete_forward(&mut document.text)
                .map_err(text_error)?,
            egui::Key::A if modifiers.command => {
                document.selections.set_single(Selection {
                    anchor: 0,
                    head: len,
                });
            }
            egui::Key::ArrowLeft if word => {
                move_primary(document, ops::word_left(&document.text, head), extend)
            }
            egui::Key::ArrowRight if word => {
                move_primary(document, ops::word_right(&document.text, head), extend)
            }
            egui::Key::ArrowLeft => {
                let target = if !extend && !document.selections.primary().is_caret() {
                    document.selections.primary().start()
                } else {
                    head.saturating_sub(1)
                };
                move_primary(document, target, extend);
            }
            egui::Key::ArrowRight => {
                let target = if !extend && !document.selections.primary().is_caret() {
                    document.selections.primary().end()
                } else {
                    (head + 1).min(len)
                };
                move_primary(document, target, extend);
            }
            egui::Key::ArrowUp => {
                move_primary(document, vertical_target(document, head, -1), extend)
            }
            egui::Key::ArrowDown => {
                move_primary(document, vertical_target(document, head, 1), extend)
            }
            egui::Key::PageUp => {
                move_primary(document, vertical_target(document, head, -page), extend)
            }
            egui::Key::PageDown => {
                move_primary(document, vertical_target(document, head, page), extend)
            }
            egui::Key::Home => {
                let target = if modifiers.command {
                    0
                } else {
                    let line = document.text.char_to_line(head);
                    let start = document.text.line_to_char(line).unwrap_or(0);
                    let indent = document
                        .text
                        .line(line)
                        .unwrap_or_default()
                        .chars()
                        .take_while(|c| c.is_whitespace() && *c != '\n')
                        .count();
                    // Smart home: toggle between first non-blank character and column 0.
                    if head == start + indent {
                        start
                    } else {
                        start + indent
                    }
                };
                move_primary(document, target, extend);
            }
            egui::Key::End => {
                let target = if modifiers.command {
                    len
                } else {
                    let line = document.text.char_to_line(head);
                    document.text.line_to_char(line).unwrap_or(0) + line_length(document, line)
                };
                move_primary(document, target, extend);
            }
            egui::Key::Escape if document.selections.selections().len() > 1 => {
                let primary = document.selections.primary();
                document
                    .selections
                    .set_single(Selection::caret(primary.head));
            }
            _ => {}
        }
    }
    Ok(EditorOutput {
        changed: document.text.revision() != initial_revision,
        caret_pos,
        requests,
    })
}

pub fn completion_popup(app: &mut IdeApp, ctx: &egui::Context) {
    let completions = app.state.lsp.completions().to_vec();
    if completions.is_empty() {
        app.completion_selected = 0;
        return;
    }
    let anchor = app
        .caret_screen_pos
        .unwrap_or_else(|| ctx.content_rect().center());
    let mut accept = None;
    egui::Area::new(egui::Id::new("completion_popup"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor + vec2(-20.0, 2.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::same(4))
                .show(ui, |ui| {
                    ui.set_width(420.0);
                    egui::ScrollArea::vertical()
                        .max_height(260.0)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for (index, item) in
                                completions.iter().take(COMPLETION_LIMIT).enumerate()
                            {
                                let selected = index == app.completion_selected;
                                let (rect, response) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 22.0),
                                    Sense::click(),
                                );
                                if selected {
                                    ui.painter().rect_filled(
                                        rect,
                                        4.0,
                                        theme::COMPLETION_SELECTION,
                                    );
                                    ui.scroll_to_rect(rect, None);
                                } else if response.hovered() {
                                    ui.painter().rect_filled(rect, 4.0, theme::HOVER);
                                }
                                let x = crate::widgets::row_text(
                                    ui,
                                    rect,
                                    rect.left() + 8.0,
                                    &item.label,
                                    egui::FontId::monospace(theme::EDITOR_FONT_SIZE),
                                    theme::TEXT_BRIGHT,
                                );
                                if let Some(detail) = &item.detail {
                                    let galley = ui.painter().layout_no_wrap(
                                        detail.clone(),
                                        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                                        theme::TEXT_INFO,
                                    );
                                    let pos = egui::pos2(
                                        (rect.right() - galley.size().x - 8.0).max(x + 16.0),
                                        rect.center().y - galley.size().y / 2.0,
                                    );
                                    ui.painter().with_clip_rect(rect).galley(
                                        pos,
                                        galley,
                                        theme::TEXT_INFO,
                                    );
                                }
                                if response.clicked() {
                                    accept = Some(index);
                                }
                            }
                        });
                });
        });
    if let Some(index) = accept {
        app.accept_completion(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_badge_does_not_cover_the_tab_title() {
        for text_width in [20.0, 87.0, 240.0] {
            let width = 10.0 + 16.0 + 6.0 + text_width + 6.0 + 16.0 + 8.0;
            let rect = Rect::from_min_size(egui::pos2(100.0, 5.0), vec2(width, 28.0));
            let layout = TabLayout::new(rect, vec2(text_width, 16.0));
            let badge = Rect::from_center_size(layout.modified_dot, vec2(5.0, 5.0));
            assert!(
                !badge.intersects(layout.text),
                "badge over title at {text_width}"
            );
            assert!(!badge.intersects(layout.close));
            assert!(!layout.text.intersects(layout.close));
            assert!(rect.contains_rect(badge));
        }
    }

    #[test]
    fn highlighting_skips_spans_that_split_characters() {
        let analysis = LanguageAnalysis {
            language: Some("Rust"),
            highlights: jetthing_language::HighlightLayer::from_spans([
                jetthing_language::HighlightSpan {
                    start_byte: 1,
                    end_byte: 2,
                    key: "DEFAULT_KEYWORD",
                },
            ]),
            ..Default::default()
        };
        let job = highlighted_line_job("ä", 0, &analysis);
        assert_eq!(job.text, "ä");
    }
}
