//! Editor island: tabs and the viewport-native text surface.

use crate::editor_keys;
use crate::icons::{self, Icon};
use crate::widgets::island_frame;
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, Sense, Stroke, vec2};
use jetthing_core::Document;
use jetthing_language::LanguageAnalysis;
use jetthing_text::Selection;
use jetthing_text::ops;
use serde_json::{Value, json};

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
    /// Text typed this frame (drives completion auto-popup and filtering).
    pub typed: String,
}

pub fn editor_id() -> egui::Id {
    egui::Id::new("jetthing_editor_surface")
}

pub fn central(app: &mut IdeApp, ctx: &egui::Context) {
    let mut changed_document = None;
    let mut requests = Vec::new();
    let mut typed = String::new();
    let caret_before = app.state.active_caret();
    egui::CentralPanel::default()
        .frame(island_frame())
        .show(ctx, |ui| {
            tabs(app, ui);
            if app.state.diff_active && app.state.diff.is_some() {
                crate::diff_ui::show(app, ui);
                app.caret_screen_pos = None;
                return;
            }
            if app.state.plugin_host.active_tab.is_some() {
                crate::plugin_ui::active_tab_body(app, ui);
                app.caret_screen_pos = None;
                return;
            }
            if app.state.active_document.is_none() {
                empty_state(ui);
                app.caret_screen_pos = None;
                return;
            }
            crate::disk_banner::show(app, ui);
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
                    typed: typed_text,
                }) => {
                    app.caret_screen_pos = caret_pos;
                    requests = editor_requests;
                    typed = typed_text;
                    if changed {
                        let snapshot = doc.text.text();
                        doc.analysis = jetthing_language::analyze(&doc.path, &snapshot);
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
        app.state.completion_after_typing(&typed);
    } else if app.state.completion.is_some() && app.state.active_caret() != caret_before {
        app.state.refresh_completion();
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
            *theme::TEXT_SECONDARY,
        );
        painter.text(
            egui::pos2(rect.center().x + 8.0, y),
            egui::Align2::LEFT_CENTER,
            crate::keys::display(shortcut),
            egui::FontId::proportional(theme::UI_FONT_SIZE),
            *theme::MODIFIED,
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
        let active = app.state.active_document == Some(index)
            && !app.state.diff_active
            && app.state.plugin_host.active_tab.is_none();
        let color = if active {
            *theme::TEXT_BRIGHT
        } else {
            *theme::TEXT_SECONDARY
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
            *theme::TAB_SELECTED
        } else if response.hovered() {
            *theme::HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, 6.0, fill);
        if active {
            ui.painter().rect_stroke(
                rect,
                6.0,
                Stroke::new(1.0_f32, theme::TAB_SELECTED_BORDER),
                egui::StrokeKind::Inside,
            );
        }
        let layout = TabLayout::new(rect, galley.size());
        icons::paint(
            ui.painter(),
            layout.icon,
            Icon::for_file(&name),
            *icons::ICON,
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
                *theme::TEXT_SECONDARY,
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
    if let Some(tab) = app.state.diff.as_ref() {
        let title = crate::diff_ui::tab_title(tab);
        let tooltip = format!("Diff of {} against HEAD", tab.path.display());
        let (clicked, closed, end) = diff_tab(ui, bar, x, &title, &tooltip, app.state.diff_active);
        x = end;
        if closed {
            app.dispatch("diff.close", Value::Null);
        } else if clicked {
            app.dispatch("diff.activate", Value::Null);
        }
    }
    crate::plugin_ui::editor_tabs(app, ui, bar, x);
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

/// The diff viewer's tab, after the document tabs. Returns (clicked, closed, x after
/// the tab).
fn diff_tab(
    ui: &mut egui::Ui,
    bar: Rect,
    x: f32,
    title: &str,
    tooltip: &str,
    active: bool,
) -> (bool, bool, f32) {
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE);
    let color = if active {
        *theme::TEXT_BRIGHT
    } else {
        *theme::TEXT_SECONDARY
    };
    let galley = ui.painter().layout_no_wrap(title.to_owned(), font, color);
    let width = 10.0 + 16.0 + 6.0 + galley.size().x + 6.0 + 16.0 + 8.0;
    let rect = Rect::from_min_size(egui::pos2(x, bar.top() + 5.0), vec2(width, 28.0));
    if rect.left() > bar.right() {
        return (false, false, rect.right() + 2.0);
    }
    let response = ui.interact(rect, egui::Id::new("editor_diff_tab"), Sense::click());
    let fill = if active {
        *theme::TAB_SELECTED
    } else if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    if active {
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0_f32, theme::TAB_SELECTED_BORDER),
            egui::StrokeKind::Inside,
        );
    }
    icons::paint(
        ui.painter(),
        Rect::from_min_size(rect.left_center() + vec2(10.0, -8.0), vec2(16.0, 16.0)),
        Icon::Commit,
        *icons::ICON,
    );
    ui.painter().galley(
        rect.left_center() + vec2(32.0, -galley.size().y / 2.0),
        galley,
        color,
    );
    let close_rect = Rect::from_min_size(
        egui::pos2(rect.right() - 24.0, rect.center().y - 8.0),
        vec2(16.0, 16.0),
    );
    let on_close = response
        .hover_pos()
        .is_some_and(|pos| close_rect.expand(2.0).contains(pos));
    if response.hovered() || active {
        if on_close {
            ui.painter()
                .rect_filled(close_rect.expand(2.0), 4.0, theme::HOVER);
        }
        icons::paint(
            ui.painter(),
            close_rect.shrink(2.0),
            Icon::Close,
            *theme::TEXT_SECONDARY,
        );
    }
    let response = response.on_hover_text(tooltip);
    let clicked = response.clicked();
    (
        clicked && !on_close,
        (clicked && on_close) || response.middle_clicked(),
        rect.right() + 2.0,
    )
}

pub(crate) fn highlighted_line_job(
    text: &str,
    line_start_byte: usize,
    analysis: &LanguageAnalysis,
    font: &egui::FontId,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let base = egui::TextFormat {
        font_id: font.clone(),
        color: *theme::editor::TEXT,
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
    let font = theme::editor_font(ui.style());
    // Rows scale with Settings | Editor | Font (size and line height).
    let row_height = theme::editor_row_height(ui.ctx(), &font, ROW_HEIGHT);
    // The color scheme's background (equal to the island in the theme's own scheme).
    ui.painter().rect_filled(
        ui.available_rect_before_wrap(),
        egui::CornerRadius {
            sw: theme::ISLAND_RADIUS,
            se: theme::ISLAND_RADIUS,
            ..Default::default()
        },
        theme::editor::BACKGROUND,
    );
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
                line_count as f32 * row_height + ui.available_height() * 0.5,
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
                let line = (((position.y - rect.top()) / row_height).floor().max(0.0) as usize)
                    .min(line_count.saturating_sub(1));
                let line_text = document.text.line(line).unwrap_or_default();
                let content = line_text.trim_end_matches(['\r', '\n']);
                let column = (((position.x - rect.left() - text_left) / char_width)
                    .round()
                    .max(0.0) as usize)
                    .min(content.chars().count());
                document.text.line_to_char(line).unwrap_or(0) + column
            };
            // Line/column under the pointer; the column may lie past the line end.
            let position_to_point = |position: egui::Pos2| jetthing_text::multicaret::BlockPoint {
                line: (((position.y - rect.top()) / ROW_HEIGHT).floor().max(0.0) as usize)
                    .min(line_count.saturating_sub(1)),
                column: ((position.x - rect.left() - text_left) / char_width)
                    .round()
                    .max(0.0) as usize,
            };
            let block_drag_id = editor_id().with("block_drag_anchor");
            if response.clicked()
                && let Some(position) = response.interact_pointer_pos()
            {
                let offset = position_to_offset(position);
                let modifiers = ui.input(|input| input.modifiers);
                if modifiers.alt && modifiers.shift {
                    editor_keys::toggle_caret(&mut document.selections, offset);
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
                && let Some(position) = ui
                    .input(|input| input.pointer.press_origin())
                    .or(response.interact_pointer_pos())
            {
                // Alt-drag, middle-drag or column mode: rectangular selection.
                let block = ui.input(|input| input.modifiers.alt)
                    || response.dragged_by(egui::PointerButton::Middle)
                    || document.selections.column_mode();
                let anchor = block.then(|| position_to_point(position));
                ui.data_mut(|data| data.insert_temp(block_drag_id, anchor));
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
            let block_anchor: Option<jetthing_text::multicaret::BlockPoint> =
                ui.data(|data| data.get_temp(block_drag_id)).flatten();
            if response.dragged()
                && let Some(position) = response.interact_pointer_pos()
                && let Some(anchor) = block_anchor
            {
                editor_keys::drag_block(
                    &document.text,
                    &mut document.selections,
                    anchor,
                    position_to_point(position),
                );
            } else if response.dragged()
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
                ((viewport.top() / row_height).floor().max(0.0) as usize).min(line_count);
            let last_line = ((viewport.bottom() / row_height).ceil() as usize + 1).min(line_count);
            let focused = response.has_focus();
            let blink_on = {
                let time = ui.input(|input| input.time);
                (time * 2.0) as i64 % 2 == 0
            };
            for line in first_line..last_line {
                let y = rect.top() + line as f32 * row_height;
                let line_start = document.text.line_to_char(line).unwrap_or(0);
                let line_text = document.text.line(line).unwrap_or_default();
                let content = line_text.trim_end_matches(['\r', '\n']);
                let line_end = line_start + content.chars().count();
                let row =
                    Rect::from_min_size(egui::pos2(rect.left(), y), vec2(rect.width(), row_height));
                if line == caret_line && document.selections.primary().is_caret() {
                    painter.rect_filled(row, 0.0, theme::editor::CARET_ROW);
                }
                painter.text(
                    egui::pos2(rect.left() + gutter_width - 18.0, y + row_height / 2.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{}", line + 1),
                    font.clone(),
                    if line == caret_line {
                        *theme::editor::LINE_NUMBER_CARET
                    } else {
                        *theme::editor::LINE_NUMBER
                    },
                );
                // Indent guides every four columns of leading whitespace.
                let indent = content.chars().take_while(|c| *c == ' ').count();
                for level in (4..=indent.saturating_sub(1)).step_by(4) {
                    let x = rect.left() + text_left + level as f32 * char_width;
                    painter.line_segment(
                        [egui::pos2(x, y), egui::pos2(x, y + row_height)],
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
                            Rect::from_min_max(egui::pos2(x1, y), egui::pos2(x2, y + row_height)),
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
                            Rect::from_min_size(egui::pos2(x1, y), vec2(char_width, row_height)),
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
                            Rect::from_min_max(egui::pos2(x1, y), egui::pos2(x2, y + row_height)),
                            2.0,
                            theme::editor::SELECTION,
                        );
                    }
                }
                let line_start_byte = document.text.line_to_byte(line).unwrap_or(0);
                let job = highlighted_line_job(content, line_start_byte, &document.analysis, &font);
                let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                let text_y = y + (row_height - galley.size().y) / 2.0;
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
                            y + row_height - 2.0,
                            *theme::editor::ERROR_EFFECT,
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
                            *theme::editor::ERROR_EFFECT
                        } else {
                            *theme::editor::WARNING_EFFECT
                        };
                        squiggle(
                            &painter,
                            column_x(start),
                            column_x(end.max(start + 1)),
                            y + row_height - 2.0,
                            color,
                        );
                    }
                }
                for selection in document.selections.selections() {
                    if selection.head >= line_start && selection.head <= line_end {
                        let x = column_x(selection.head - line_start);
                        if *selection == document.selections.primary() {
                            caret_pos = Some(egui::pos2(x, y + row_height));
                        }
                        if focused && blink_on {
                            painter.line_segment(
                                [egui::pos2(x, y + 2.0), egui::pos2(x, y + row_height - 2.0)],
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
                    Rect::from_min_size(pos - vec2(0.0, row_height), vec2(char_width, row_height));
                if !viewport
                    .translate(rect.min.to_vec2())
                    .contains_rect(caret_rect)
                {
                    ui.scroll_to_rect(caret_rect.expand2(vec2(char_width * 4.0, row_height)), None);
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
            typed: String::new(),
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
    let page = (ui.available_height() / row_height).max(1.0) as isize;
    let mac = cfg!(target_os = "macos");
    let text_error = |error: jetthing_text::TextError| error.to_string();
    let command = |id: &'static str| EditorRequest::Command(id, Value::Null);
    let clone_armed = editor_keys::clone_gesture_armed(ui.ctx(), response.id, mac);
    let mut typed = String::new();
    for event in events {
        let (key, modifiers) = match event {
            egui::Event::Text(text) if !text.is_empty() => {
                typed.push_str(&text);
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
        if let Some(id) = editor_keys::action_for(key, modifiers, mac) {
            requests.push(command(id));
            continue;
        }
        if clone_armed && matches!(key, egui::Key::ArrowUp | egui::Key::ArrowDown) {
            requests.push(command(if key == egui::Key::ArrowUp {
                "editor.cloneCaretAbove"
            } else {
                "editor.cloneCaretBelow"
            }));
            continue;
        }
        if document.selections.column_mode()
            && let Some((lines, columns)) = editor_keys::column_delta(key, modifiers)
        {
            editor_keys::extend_block(document, lines, columns);
            continue;
        }
        let head = document.selections.primary().head;
        let len = document.text.len_chars();
        let extend = modifiers.shift;
        // Word-wise movement: ⌥ on macOS, Ctrl elsewhere.
        let word = if mac { modifiers.alt } else { modifiers.ctrl };
        match key {
            egui::Key::D if modifiers.command => requests.push(command("editor.duplicate")),
            egui::Key::Y if modifiers.command && !mac => {
                requests.push(command("editor.deleteLine"))
            }
            egui::Key::Backspace if modifiers.command && mac => {
                requests.push(command("editor.deleteLine"))
            }
            egui::Key::Slash if modifiers.command => requests.push(command("editor.commentLine")),
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
                editor_keys::move_carets(document, extend, |d, s| ops::word_left(&d.text, s.head))
            }
            egui::Key::ArrowRight if word => {
                editor_keys::move_carets(document, extend, |d, s| ops::word_right(&d.text, s.head))
            }
            egui::Key::ArrowLeft => editor_keys::move_carets(document, extend, |_, s| {
                if !extend && !s.is_caret() {
                    s.start()
                } else {
                    s.head.saturating_sub(1)
                }
            }),
            egui::Key::ArrowRight => editor_keys::move_carets(document, extend, |_, s| {
                if !extend && !s.is_caret() {
                    s.end()
                } else {
                    (s.head + 1).min(len)
                }
            }),
            egui::Key::ArrowUp => {
                editor_keys::move_carets(document, extend, |d, s| vertical_target(d, s.head, -1))
            }
            egui::Key::ArrowDown => {
                editor_keys::move_carets(document, extend, |d, s| vertical_target(d, s.head, 1))
            }
            egui::Key::PageUp => {
                move_primary(document, vertical_target(document, head, -page), extend)
            }
            egui::Key::PageDown => {
                move_primary(document, vertical_target(document, head, page), extend)
            }
            egui::Key::Home if modifiers.command => move_primary(document, 0, extend),
            egui::Key::Home => editor_keys::move_carets(document, extend, |d, s| {
                let line = d.text.char_to_line(s.head);
                let start = d.text.line_to_char(line).unwrap_or(0);
                let indent = d
                    .text
                    .line(line)
                    .unwrap_or_default()
                    .chars()
                    .take_while(|c| c.is_whitespace() && *c != '\n')
                    .count();
                // Smart home: toggle between first non-blank character and column 0.
                if s.head == start + indent {
                    start
                } else {
                    start + indent
                }
            }),
            egui::Key::End if modifiers.command => move_primary(document, len, extend),
            egui::Key::End => editor_keys::move_carets(document, extend, |d, s| {
                let line = d.text.char_to_line(s.head);
                d.text.line_to_char(line).unwrap_or(0) + line_length(d, line)
            }),
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
        typed,
    })
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
            diagnostics: Vec::new(),
        };
        let job = highlighted_line_job("ä", 0, &analysis, &egui::FontId::monospace(13.0));
        assert_eq!(job.text, "ä");
    }
}
