//! Editor island: tabs and the viewport-native text surface.

use crate::editor_keys;
use crate::folding::{CollapsedRow, FoldView};
use crate::icons::{self, Icon};
use crate::problems_ui;
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
    /// An entry of the editor context menu (`EditorPopupMenu`).
    ContextAction(String),
    /// A gutter change marker was clicked: hunk index and screen position.
    ShowChange(usize, egui::Pos2),
    /// A blame annotation was clicked: commit hash.
    ShowCommit(String),
    /// Click on a gutter run icon (line, screen position for the popup).
    Gutter(usize, egui::Pos2),
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
                // Offer toolchain setup as soon as the project opens.
                crate::toolchain_banner::show(app, ui);
                empty_state(ui);
                app.caret_screen_pos = None;
                return;
            }
            crate::disk_banner::show(app, ui);
            crate::toolchain_banner::show(app, ui);
            if app.find.open {
                crate::findbar::show(app, ui);
            }
            let Some(index) = app.state.active_document else {
                return;
            };
            let problems = app.state.active_problems();
            let mut highlights = if app.find.open {
                app.find.matches(&app.state.documents[index])
            } else {
                Vec::new()
            };
            // Snippet tab stops are boxed like search results while active.
            if let Some(snippet) = &app.state.snippet {
                highlights.extend(snippet.ranges().filter(|(start, end)| start < end));
                // The painter only visits the matches of visible lines: keep them sorted.
                highlights.sort_unstable();
            }
            let available = ui.available_rect_before_wrap();
            let stripe_rect = Rect::from_min_max(
                egui::pos2(
                    available.right() - problems_ui::STRIPE_WIDTH,
                    available.top(),
                ),
                available.max,
            );
            let text_rect = available.with_max_x(stripe_rect.left());
            ui.allocate_rect(available, Sense::hover());
            let line_count = app.state.documents[index].text.len_lines();
            let row_height =
                theme::editor_row_height(ui.ctx(), &theme::editor_font(ui.style()), ROW_HEIGHT);
            if let Some((line, column)) =
                problems_ui::error_stripe(ui, stripe_rect, &problems, line_count, row_height)
            {
                move_caret_to(app, line, column);
                ui.memory_mut(|memory| memory.request_focus(editor_id()));
            }
            let mut text_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
            let context_model = app.context_menu_model(&crate::context_menus::Target::Editor);
            let changes = app.state.document_changes(index).to_vec();
            let blame = app.state.document_annotations(index).map(<[_]>::to_vec);
            let gutter = run_icons(app, ui.ctx());
            let doc = &mut app.state.documents[index];
            let surface = editor_surface(
                &mut text_ui,
                doc,
                &problems,
                &highlights,
                &context_model,
                GutterData {
                    changes: &changes,
                    blame: blame.as_deref(),
                    run: &gutter,
                },
            );
            let counts = jetthing_core::ProblemCounts::of(&problems);
            match problems_ui::inspection_widget(ui, text_rect.right_top(), counts) {
                Some(problems_ui::WidgetAction::Next) => {
                    requests.push(EditorRequest::Command("editor.nextProblem", Value::Null));
                }
                Some(problems_ui::WidgetAction::Previous) => {
                    requests.push(EditorRequest::Command(
                        "editor.previousProblem",
                        Value::Null,
                    ));
                }
                Some(problems_ui::WidgetAction::ShowProblems) => requests.push(
                    EditorRequest::Command("toolwindow.activate.problems", Value::Null),
                ),
                None => {}
            }
            let doc = &mut app.state.documents[index];
            match surface {
                Ok(EditorOutput {
                    changed,
                    caret_pos,
                    requests: editor_requests,
                    typed: typed_text,
                }) => {
                    app.caret_screen_pos = caret_pos;
                    requests.extend(editor_requests);
                    typed = typed_text;
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
        app.state.snippet_after_change();
        app.state.completion_after_typing(&typed);
    } else if app.state.active_caret() != caret_before {
        app.state.snippet_after_caret_move();
        if app.state.completion.is_some() {
            app.state.refresh_completion();
        }
    }
    for request in requests {
        handle_request(app, ctx, request);
    }
}

pub(crate) fn handle_request(app: &mut IdeApp, ctx: &egui::Context, request: EditorRequest) {
    match request {
        EditorRequest::Command(id, params) => app.dispatch(id, params),
        EditorRequest::ShowCommit(hash) => {
            app.dispatch("git.log.show", Value::Null);
            app.dispatch("git.log.selectCommit", json!({ "hash": hash }));
        }
        EditorRequest::ShowChange(hunk, position) => {
            app.change_popup = Some(crate::change_popup::ChangePopup::new(hunk, position));
        }
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
        EditorRequest::ContextAction(action) => {
            app.run_context_action(&action, &crate::context_menus::Target::Editor, ctx)
        }
        EditorRequest::Gutter(line, position) => crate::gutter_popup::open(ctx, line, position),
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
    let mut tab_action = None;
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
            app.file_icon(&doc.path),
            *icons::ICON,
        );
        let text_rect = layout.text;
        ui.painter().galley(layout.text.min, galley, color);
        if app.state.problems.has_errors(&doc.path) {
            problems_ui::squiggle(
                ui.painter(),
                text_rect.left(),
                text_rect.right(),
                rect.center().y + 8.0,
                *theme::editor::ERROR_EFFECT,
            );
        }
        let close_rect = layout.close;
        let dirty = doc.text.is_dirty();
        let pinned = app.state.is_pinned(index);
        let pointer_on_close = response
            .hover_pos()
            .is_some_and(|pos| close_rect.expand(2.0).contains(pos));
        if pinned {
            // Pinned tabs show a pin in place of the close button; clicking it unpins.
            if pointer_on_close {
                ui.painter()
                    .rect_filled(close_rect.expand(2.0), 4.0, theme::HOVER);
            }
            icons::paint(
                ui.painter(),
                close_rect.shrink(1.0),
                Icon::Pin,
                theme::TEXT_SECONDARY,
            );
        } else if response.hovered() || active {
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
        response.context_menu(|ui| {
            let target = crate::context_menus::Target::Tab(index);
            let model = app.context_menu_model(&target);
            let mut chosen = None;
            crate::context_menus::show(ui, &model, &mut chosen);
            if let Some(action) = chosen {
                tab_action = Some((action, target));
            }
        });
        if response.clicked() {
            if pointer_on_close && pinned {
                tab_action = Some((
                    "PinActiveTabToggle".to_owned(),
                    crate::context_menus::Target::Tab(index),
                ));
            } else if pointer_on_close {
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
    if let Some((action, target)) = tab_action {
        let ctx = ui.ctx().clone();
        app.run_context_action(&action, &target, &ctx);
    } else if let Some(index) = close {
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

/// Paints a collapsed region's placeholder box and the text after the region
/// on its last line, starting at `origin` (top-left of the placeholder).
fn paint_collapsed_tail(
    painter: &egui::Painter,
    document: &Document,
    row: &CollapsedRow,
    origin: egui::Pos2,
    char_width: f32,
    font: &egui::FontId,
    row_height: f32,
) {
    let folded = crate::syntax::attributes("FOLDED_TEXT_ATTRIBUTES");
    let width = row.placeholder.chars().count() as f32 * char_width;
    let to_color = |rgba: jetthing_language::Rgba| Color32::from_rgb(rgba.r, rgba.g, rgba.b);
    painter.rect_filled(
        Rect::from_min_size(origin + vec2(0.0, 3.0), vec2(width, row_height - 6.0)),
        3.0,
        folded
            .background
            .map_or(*theme::editor::MATCHED_BRACE, to_color),
    );
    painter.text(
        origin + vec2(0.0, row_height / 2.0),
        egui::Align2::LEFT_CENTER,
        &row.placeholder,
        font.clone(),
        folded
            .foreground
            .map_or(*theme::editor::LINE_NUMBER_CARET, to_color),
    );
    let end_line_start = document.text.line_to_char(row.end_line).unwrap_or(0);
    let end_text = document.text.line(row.end_line).unwrap_or_default();
    let end_content = end_text.trim_end_matches(['\r', '\n']);
    let skip = row.end_char.saturating_sub(end_line_start);
    let start = end_content
        .char_indices()
        .nth(skip)
        .map_or(end_content.len(), |(index, _)| index);
    let tail = &end_content[start..];
    if tail.is_empty() {
        return;
    }
    let tail_byte = document.text.line_to_byte(row.end_line).unwrap_or(0) + start;
    let job = highlighted_line_job(tail, tail_byte, &document.analysis, font);
    let galley = painter.layout_job(job);
    let text_y = origin.y + (row_height - galley.size().y) / 2.0;
    painter.galley(egui::pos2(origin.x + width, text_y), galley, Color32::WHITE);
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

/// Character offset for moving `head` vertically by `delta` visual rows
/// (collapsed regions count as one row), keeping the column.
fn vertical_target(document: &Document, folds: &FoldView, head: usize, delta: isize) -> usize {
    let Ok((line, column)) = document.text.char_to_line_column(head) else {
        return head;
    };
    let last_row = folds.row_count(document.text.len_lines()).saturating_sub(1) as isize;
    let row = (folds.line_to_row(line) as isize + delta).clamp(0, last_row) as usize;
    let target = folds.row_to_line(row);
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

/// Horizontal layout of the editor gutter. Left to right: blame annotations,
/// line numbers, run icons (when the file has any), folding markers, the
/// change bar, then the text.
#[derive(Debug, Clone, Copy, PartialEq)]
struct GutterLayout {
    /// Right edge of the right-aligned line numbers.
    numbers_right: f32,
    /// Center of the run-icon column (equal to `numbers_right` without icons).
    run_center: f32,
    run_width: f32,
    /// Center of the folding-marker column.
    fold_center: f32,
    /// Total gutter width; the change bar sits just inside it.
    width: f32,
}

impl GutterLayout {
    const FOLD_COLUMN: f32 = 14.0;
    const RUN_COLUMN: f32 = 16.0;
    /// Space for the change bar (`change_popup` paints it 7 px inside the edge).
    const CHANGE_COLUMN: f32 = 14.0;

    fn new(blame_width: f32, numbers_width: f32, run_icons: bool) -> Self {
        let numbers_right = blame_width + 20.0 + numbers_width;
        let run_width = if run_icons { Self::RUN_COLUMN } else { 0.0 };
        let run_left = numbers_right + 4.0;
        let fold_left = run_left + run_width;
        Self {
            numbers_right,
            run_center: run_left + run_width / 2.0,
            run_width,
            fold_center: fold_left + Self::FOLD_COLUMN / 2.0,
            width: fold_left + Self::FOLD_COLUMN + Self::CHANGE_COLUMN,
        }
    }

    fn in_run_column(&self, x: f32) -> bool {
        self.run_width > 0.0 && (x - self.run_center).abs() <= self.run_width / 2.0
    }

    fn in_fold_column(&self, x: f32) -> bool {
        (x - self.fold_center).abs() <= Self::FOLD_COLUMN / 2.0
    }
}

/// Run icons are recomputed this long after the latest edit; until then the
/// previous icons stay (finding runnables walks the whole syntax tree, too slow
/// for every keystroke in large files).
const RUN_ICONS_STALE_FOR: f64 = 0.25;

/// Gutter run icons of the active editor, refreshed once typing pauses.
fn run_icons(app: &mut IdeApp, ctx: &egui::Context) -> Vec<jetthing_core::GutterMark> {
    let id = egui::Id::new("run_icons_last_edit");
    let revision = app
        .state
        .active_document
        .and_then(|index| app.state.documents.get(index))
        .map(|document| document.text.revision());
    let now = ctx.input(|input| input.time);
    let (seen, last_edit) = ctx
        .data(|data| data.get_temp::<(Option<u64>, f64)>(id))
        .unwrap_or((None, f64::NEG_INFINITY));
    let last_edit = if seen != revision { now } else { last_edit };
    ctx.data_mut(|data| data.insert_temp(id, (revision, last_edit)));
    if now - last_edit < RUN_ICONS_STALE_FOR
        && let Some(marks) = app.state.cached_gutter_marks()
    {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(RUN_ICONS_STALE_FOR));
        return marks;
    }
    app.state.gutter_marks()
}

/// What the gutter shows next to the line numbers.
struct GutterData<'a> {
    /// Change markers against `HEAD`.
    changes: &'a [jetthing_git::Hunk],
    /// Blame annotations, when shown.
    blame: Option<&'a [jetthing_git::BlameLine]>,
    /// Run icons (mains, tests, Bazel rules).
    run: &'a [jetthing_core::GutterMark],
}

fn editor_surface(
    ui: &mut egui::Ui,
    document: &mut Document,
    problems: &[jetthing_core::Problem],
    highlights: &[(usize, usize)],
    context_model: &crate::context_menus::Model,
    gutter_data: GutterData<'_>,
) -> Result<EditorOutput, String> {
    let GutterData {
        changes,
        blame,
        run: gutter,
    } = gutter_data;
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
    // Carets never stay inside collapsed text (Go to Line, Find, arrows).
    document.reveal_carets();
    let folds = FoldView::new(document);
    let row_count = folds.row_count(line_count);
    let digits = line_count.max(1).to_string().len().max(2);
    let blame_width = if blame.is_some() {
        crate::blame_gutter::WIDTH
    } else {
        0.0
    };
    let layout = GutterLayout::new(blame_width, digits as f32 * char_width, !gutter.is_empty());
    let gutter_width = layout.width;
    let text_left = gutter_width + TEXT_LEFT_PADDING;
    let caret_line = document
        .text
        .char_to_line_column(document.selections.primary().head)
        .map(|(line, _)| line)
        .unwrap_or(0);
    let mut caret_pos = None;
    let brace_pair: Vec<usize> = {
        let head = document.selections.primary().head;
        // Structural matching from the syntax tree when the language has a
        // grammar (brackets in strings/comments never match); text otherwise.
        let find = |offset: usize| {
            if document.analysis.syntax.is_some() {
                let byte = document.text.char_to_byte(offset);
                document
                    .analysis
                    .matching_bracket(byte)
                    .map(|(a, b)| (document.text.byte_to_char(a), document.text.byte_to_char(b)))
            } else {
                ops::matching_bracket(&document.text, offset)
            }
        };
        find(head)
            .or_else(|| head.checked_sub(1).and_then(find))
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
                row_count as f32 * row_height + ui.available_height() * 0.5,
            );
            let (rect, response) = ui.allocate_exact_size(content_size, Sense::click_and_drag());
            let response = ui
                .interact(rect, editor_id(), Sense::click_and_drag())
                .union(response)
                .on_hover_cursor(egui::CursorIcon::Text);
            if response.clicked() || response.drag_started() {
                response.request_focus();
            }
            let row_at = |position: egui::Pos2| {
                (((position.y - rect.top()) / row_height).floor().max(0.0) as usize)
                    .min(row_count.saturating_sub(1))
            };
            let position_to_offset = |position: egui::Pos2| {
                let line = folds.row_to_line(row_at(position));
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
                line: folds.row_to_line(row_at(position)),
                column: ((position.x - rect.left() - text_left) / char_width)
                    .round()
                    .max(0.0) as usize,
            };
            let block_drag_id = editor_id().with("block_drag_anchor");
            // Clicking a change marker opens the change popup instead of
            // moving the caret.
            let marker_hit = response
                .interact_pointer_pos()
                .filter(|position| {
                    crate::change_popup::in_marker_column(position.x - rect.left(), layout.width)
                })
                .and_then(|position| {
                    let line = folds.row_to_line(row_at(position));
                    jetthing_core::hunk_at_line(changes, line).map(|hunk| (hunk, position))
                });
            let blame_line = response
                .hover_pos()
                .or(response.interact_pointer_pos())
                .filter(|position| position.x - rect.left() < blame_width)
                .map(|position| folds.row_to_line(row_at(position)));
            if let (Some(lines), Some(line)) = (blame, blame_line) {
                if let Some(text) = crate::blame_gutter::tooltip(lines, line) {
                    response.clone().on_hover_text_at_pointer(text);
                }
                if response.clicked()
                    && let Some(entry) = lines.get(line).filter(|entry| !entry.is_uncommitted())
                {
                    requests.push(EditorRequest::ShowCommit(entry.hash.clone()));
                }
            }
            // Run icons sit between the line numbers and the text.
            let gutter_hit = response
                .interact_pointer_pos()
                .filter(|position| layout.in_run_column(position.x - rect.left()))
                .and_then(|position| {
                    let line = folds.row_to_line(row_at(position));
                    gutter
                        .iter()
                        .any(|mark| mark.line == line)
                        .then_some((line, position))
                });
            // Fold markers sit in the gutter just left of the text; placeholders
            // expand their region when clicked.
            let fold_click = response
                .clicked()
                .then(|| response.interact_pointer_pos())
                .flatten()
                .and_then(|position| {
                    let line = folds.row_to_line(row_at(position));
                    let x = position.x - rect.left();
                    let in_marker_column = layout.in_fold_column(x);
                    let on_placeholder = folds.collapsed_on(line).is_some_and(|row| {
                        let line_start = document.text.line_to_char(line).unwrap_or(0);
                        let column = (x - text_left) / char_width;
                        let start = (row.start_char - line_start) as f32;
                        column >= start && column <= start + row.placeholder.chars().count() as f32
                    });
                    (in_marker_column || on_placeholder).then_some(line)
                });
            if response.clicked() && blame_line.is_some() {
                // Handled above: annotation clicks never move the caret.
            } else if let Some(line) = fold_click {
                requests.push(EditorRequest::Command(
                    "editor.toggleFold",
                    serde_json::json!({ "line": line }),
                ));
            } else if response.clicked()
                && let Some((hunk, position)) = marker_hit
            {
                requests.push(EditorRequest::ShowChange(hunk, position));
            } else if response.clicked()
                && let Some((line, position)) = gutter_hit
            {
                requests.push(EditorRequest::Gutter(line, position));
            } else if response.clicked()
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
                let mut chosen = None;
                crate::context_menus::show(ui, context_model, &mut chosen);
                match chosen.as_deref() {
                    // Copy/Cut also fill the system clipboard.
                    Some("$Copy") => requests.push(EditorRequest::Copy),
                    Some("$Cut") => requests.push(EditorRequest::Cut),
                    Some(action) => requests.push(EditorRequest::ContextAction(action.to_owned())),
                    None => {}
                }
                ui.separator();
                ui.menu_button("Git", |ui| {
                    ui.set_min_width(220.0);
                    let annotate = if blame.is_some() {
                        "Close Annotations"
                    } else {
                        "Annotate with Git Blame"
                    };
                    for (label, id) in [
                        (annotate, "git.annotate"),
                        ("Show History", "git.fileHistory"),
                        ("Show Diff", "vcs.showChangeDiff"),
                        ("Rollback Lines", "vcs.rollbackChange"),
                    ] {
                        if ui.button(label).clicked() {
                            requests.push(EditorRequest::Command(id, Value::Null));
                            ui.close();
                        }
                    }
                });
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
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs() as i64);
            // Gutter background matches the editor in Islands; draw caret row first.
            let first_row =
                ((viewport.top() / row_height).floor().max(0.0) as usize).min(row_count);
            let last_row = ((viewport.bottom() / row_height).ceil() as usize + 1).min(row_count);
            // Gutter fold markers for regions starting in the viewport.
            let gutter_hovered = ui
                .input(|input| input.pointer.hover_pos())
                .is_some_and(|pos| rect.contains(pos) && pos.x - rect.left() < layout.width + 4.0);
            let markers: Vec<(usize, bool)> = if first_row < last_row {
                let first_line = folds.row_to_line(first_row);
                let last_line = folds.row_to_line(last_row - 1);
                let start = document.text.line_to_char(first_line).unwrap_or(0);
                let end = document
                    .text
                    .line_to_char(last_line + 1)
                    .unwrap_or(document.text.len_chars());
                document
                    .folds_in_chars(start, end)
                    .iter()
                    .map(|fold| (fold.start_line, document.folds.is_collapsed(fold)))
                    .collect()
            } else {
                Vec::new()
            };
            let focused = response.has_focus();
            let blink_on = {
                let time = ui.input(|input| input.time);
                (time * 2.0) as i64 % 2 == 0
            };
            for row_index in first_row..last_row {
                let line = folds.row_to_line(row_index);
                let y = rect.top() + row_index as f32 * row_height;
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
                    egui::pos2(rect.left() + layout.numbers_right, y + row_height / 2.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{}", line + 1),
                    font.clone(),
                    if line == caret_line {
                        *theme::editor::LINE_NUMBER_CARET
                    } else {
                        *theme::editor::LINE_NUMBER
                    },
                );
                if let Some(lines) = blame {
                    crate::blame_gutter::paint(
                        &painter,
                        lines,
                        line,
                        egui::pos2(rect.left(), y),
                        row_height,
                        now,
                    );
                }
                crate::change_popup::paint_marker(
                    &painter,
                    changes,
                    line,
                    egui::pos2(rect.left() + gutter_width, y),
                    row_height,
                );
                if let Some(mark) = gutter.iter().find(|mark| mark.line == line) {
                    crate::gutter_popup::paint_icon(
                        &painter,
                        egui::pos2(rect.left() + layout.run_center, y + row_height / 2.0),
                        mark.kind,
                    );
                }
                if let Some(&(_, collapsed)) = markers.iter().find(|(start, _)| *start == line)
                    && (collapsed || gutter_hovered)
                {
                    crate::folding::paint_marker(
                        &painter,
                        egui::pos2(rect.left() + layout.fold_center, y + row_height / 2.0),
                        collapsed,
                        *theme::editor::LINE_NUMBER_CARET,
                    );
                }
                // Indent guides every four columns of leading whitespace.
                let indent = content.chars().take_while(|c| *c == ' ').count();
                for level in (4..=indent.saturating_sub(1)).step_by(4) {
                    let x = rect.left() + text_left + level as f32 * char_width;
                    painter.line_segment(
                        [egui::pos2(x, y), egui::pos2(x, y + row_height)],
                        Stroke::new(1.0_f32, theme::editor::INDENT_GUIDE),
                    );
                }
                // Matches are sorted and disjoint: only visit those on this line.
                let first_match = highlights.partition_point(|&(_, end)| end <= line_start);
                let visible_matches = highlights[first_match..]
                    .iter()
                    .take_while(|&&(start, _)| start <= line_end);
                for &(match_start, match_end) in visible_matches {
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
                let collapsed = folds.collapsed_on(line);
                // A collapsed region shows the text before it, its placeholder,
                // then the rest of its last line.
                let visible = match collapsed {
                    Some(row) => {
                        let prefix_chars = row.start_char.saturating_sub(line_start);
                        let end = content
                            .char_indices()
                            .nth(prefix_chars)
                            .map_or(content.len(), |(index, _)| index);
                        &content[..end]
                    }
                    None => content,
                };
                let job = highlighted_line_job(visible, line_start_byte, &document.analysis, &font);
                let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                let text_y = y + (row_height - galley.size().y) / 2.0;
                painter.galley(
                    egui::pos2(rect.left() + text_left, text_y),
                    galley,
                    Color32::WHITE,
                );
                if let Some(row) = collapsed {
                    paint_collapsed_tail(
                        &painter,
                        document,
                        row,
                        egui::pos2(
                            rect.left() + text_left + visible.chars().count() as f32 * char_width,
                            y,
                        ),
                        char_width,
                        &font,
                        row_height,
                    );
                }

                let column_x = |column: usize| rect.left() + text_left + column as f32 * char_width;
                problems_ui::paint_line(
                    &painter,
                    problems,
                    line,
                    content.chars().count(),
                    column_x,
                    row,
                );
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
            if let Some(pointer) = response.hover_pos() {
                let line = folds.row_to_line(row_at(pointer));
                let column = (pointer.x - rect.left() - text_left) / char_width;
                if line < line_count && column >= 0.0 {
                    let line_len = document
                        .text
                        .line(line)
                        .unwrap_or_default()
                        .trim_end_matches(['\r', '\n'])
                        .chars()
                        .count();
                    problems_ui::hover_tooltip(
                        &response,
                        problems,
                        line,
                        column.floor() as usize,
                        line_len,
                    );
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
                // egui surrenders focus on Escape by default; the workbench's
                // Escape handler closes popups instead and the editor keeps focus.
                escape: true,
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
        // Duplicate, Delete Line, Comment and Move Line come from the active keymap
        // (`keybindings`), which consumes their keys before the editor sees them.
        match key {
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
            egui::Key::ArrowUp => editor_keys::move_carets(document, extend, |d, s| {
                vertical_target(d, &folds, s.head, -1)
            }),
            egui::Key::ArrowDown => editor_keys::move_carets(document, extend, |d, s| {
                vertical_target(d, &folds, s.head, 1)
            }),
            egui::Key::PageUp => move_primary(
                document,
                vertical_target(document, &folds, head, -page),
                extend,
            ),
            egui::Key::PageDown => move_primary(
                document,
                vertical_target(document, &folds, head, page),
                extend,
            ),
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
    #[test]
    fn gutter_columns_follow_the_reference_order() {
        let plain = super::GutterLayout::new(0.0, 20.0, false);
        assert!(!plain.in_run_column(plain.numbers_right + 2.0));
        assert!(plain.numbers_right < plain.fold_center);
        assert!(
            plain.fold_center + super::GutterLayout::FOLD_COLUMN / 2.0 <= plain.width - 7.0 - 4.0
        );
        let full = super::GutterLayout::new(70.0, 20.0, true);
        // blame | line numbers | run icons | folding | change bar | text
        assert!(70.0 < full.numbers_right);
        assert!(full.numbers_right < full.run_center);
        assert!(full.run_center < full.fold_center);
        assert!(full.in_run_column(full.run_center));
        assert!(!full.in_fold_column(full.run_center));
        assert!(full.in_fold_column(full.fold_center));
        // The change bar (4 px wide, 7 px inside the edge) is right of the fold column.
        assert!(full.fold_center + super::GutterLayout::FOLD_COLUMN / 2.0 <= full.width - 11.0);
        assert!(!crate::change_popup::in_marker_column(
            full.fold_center,
            full.width
        ));
    }

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
        let job = highlighted_line_job("ä", 0, &analysis, &egui::FontId::monospace(13.0));
        assert_eq!(job.text, "ä");
    }
}
