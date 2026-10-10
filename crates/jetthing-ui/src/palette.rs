//! "Search Everywhere" popup: Classes, Files, Symbols and Actions behind one field.
//!
//! What each tab lists and how it ranks is decided in `jetthing_core::navigation`;
//! this file only renders it and turns a chosen row into a command.

use crate::icons::{self, Icon, TypeKind};
use crate::{IdeApp, theme};
use eframe::egui::{self, Rect, Sense, vec2};
use jetthing_core::navigation::{
    ActionCandidate, Hit, Ranked, SearchSources, SearchTab, SymbolEntry, file_name, search,
};
use jetthing_language::structure::SymbolKind;
use serde_json::{Value, json};
use std::time::Duration;

const MAX_ROWS: usize = 14;
/// Time spent indexing symbols per frame while the popup is open.
const INDEX_BUDGET: Duration = Duration::from_millis(15);

pub type Tab = SearchTab;

#[derive(Default)]
pub struct Palette {
    pub open: bool,
    query: String,
    selected: usize,
    just_opened: bool,
    tab: SearchTab,
    /// Query last sent to the language server.
    server_query: String,
}

impl Palette {
    /// Selected tab, as reported by the automation `state` request.
    pub fn tab_name(&self) -> &'static str {
        self.tab.id()
    }

    pub fn show_tab(&mut self, tab: SearchTab) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.just_opened = true;
        self.tab = tab;
        self.server_query.clear();
    }
}

fn actions(app: &IdeApp) -> Vec<ActionCandidate> {
    app.commands
        .specs()
        .into_iter()
        .filter(IdeApp::parameterless)
        .map(|spec| ActionCandidate {
            enabled: app.commands.is_enabled(&spec.id, &app.state),
            shortcut: app.command_shortcut(&spec),
            id: spec.id,
            label: spec.label,
            category: spec.category,
        })
        .collect()
}

fn items(app: &mut IdeApp) -> Vec<Ranked<Hit>> {
    let query = app.palette.query.clone();
    let tab = app.palette.tab;
    if matches!(
        tab,
        SearchTab::All | SearchTab::Classes | SearchTab::Symbols
    ) {
        app.state.refresh_symbol_index(INDEX_BUDGET);
        if app.palette.server_query != query {
            app.palette.server_query = query.clone();
            app.state.request_server_symbols(&query);
        }
        app.state.poll_server_symbols();
    }
    let actions = if matches!(tab, SearchTab::All | SearchTab::Actions) {
        actions(app)
    } else {
        Vec::new()
    };
    let navigation = &app.state.navigation;
    let sources = SearchSources {
        files: &app.state.files,
        recent: &app.state.recent_files,
        symbols: navigation.index.all().collect(),
        server_symbols: &navigation.server.results,
        actions: &actions,
    };
    let mut found = search(tab, &query, &sources);
    if tab == SearchTab::All && query.is_empty() {
        // Nothing typed: recent files only, like the reference IDE.
        found.retain(|hit| matches!(hit.item, Hit::File { .. }));
    }
    found.truncate(if tab == SearchTab::All { 16 } else { MAX_ROWS });
    found
}

fn symbol_icon(symbol: &SymbolEntry) -> Icon {
    match symbol.kind {
        SymbolKind::Class => Icon::JavaType(TypeKind::Class),
        SymbolKind::Interface => Icon::JavaType(TypeKind::Interface),
        SymbolKind::Enum => Icon::JavaType(TypeKind::Enum),
        _ => Icon::File,
    }
}

/// Paints `text` with the characters at `positions` emphasised; returns the end x.
fn highlighted_text(
    ui: &egui::Ui,
    row: Rect,
    x: f32,
    text: &str,
    positions: &[usize],
    color: egui::Color32,
) -> f32 {
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE);
    let mut job = egui::text::LayoutJob::default();
    for (index, ch) in text.chars().enumerate() {
        let matched = positions.contains(&index);
        job.append(
            &ch.to_string(),
            0.0,
            egui::text::TextFormat {
                font_id: font.clone(),
                color: if matched { *theme::MODIFIED } else { color },
                background: if matched {
                    theme::SELECTION_DISABLED.gamma_multiply(0.6)
                } else {
                    egui::Color32::TRANSPARENT
                },
                ..Default::default()
            },
        );
    }
    let galley = ui.painter().layout_job(job);
    let width = galley.size().x;
    let pos = egui::pos2(x, row.center().y - galley.size().y / 2.0);
    ui.painter().with_clip_rect(row).galley(pos, galley, color);
    x + width
}

/// Moves the caret to a symbol or `file:line` and focuses the editor.
fn open_location(
    app: &mut IdeApp,
    ctx: &egui::Context,
    path: &std::path::Path,
    line: Option<usize>,
) {
    let mut params = json!({"path": path});
    if let Some(line) = line {
        params["line"] = json!(line);
    }
    app.dispatch("editor.openFile", params);
    ctx.memory_mut(|memory| memory.request_focus(crate::editor::editor_id()));
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    if !app.palette.open {
        return;
    }
    for (modifiers, backward) in [
        (egui::Modifiers::NONE, false),
        (egui::Modifiers::SHIFT, true),
    ] {
        if ctx.input_mut(|input| input.consume_key(modifiers, egui::Key::Tab)) {
            app.palette.tab = app.palette.tab.step(backward);
            app.palette.selected = 0;
        }
    }
    let found = items(app);
    if found.len() > MAX_ROWS {
        ctx.request_repaint();
    }
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        app.palette.open = false;
        return;
    }
    if !found.is_empty() {
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
            app.palette.selected = (app.palette.selected + 1) % found.len();
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
            app.palette.selected = (app.palette.selected + found.len() - 1) % found.len();
        }
    }
    app.palette.selected = app.palette.selected.min(found.len().saturating_sub(1));
    let enter = ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let screen = ctx.content_rect();
    let width = 680.0_f32.min(screen.width() - 40.0);
    let mut chosen = enter.then_some(app.palette.selected);
    let mut close = false;
    egui::Area::new(egui::Id::new("search_everywhere"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.center().x - width / 2.0,
            screen.top() + screen.height() * 0.16,
        ))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(10)
                .inner_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    // Header tabs (Tab switches).
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        for tab in Tab::ALL {
                            let galley = ui.painter().layout_no_wrap(
                                tab.label().to_owned(),
                                egui::FontId::proportional(theme::UI_FONT_SIZE),
                                *theme::TEXT_BRIGHT,
                            );
                            let (rect, response) = ui.allocate_exact_size(
                                vec2(galley.size().x + 20.0, 32.0),
                                Sense::click(),
                            );
                            let pill = rect.shrink2(vec2(0.0, 5.0));
                            if tab == app.palette.tab {
                                ui.painter().rect_filled(pill, 4.0, theme::SELECTION);
                            } else if response.hovered() {
                                ui.painter().rect_filled(pill, 4.0, theme::HOVER);
                            }
                            let color = if tab == app.palette.tab {
                                *theme::TEXT_BRIGHT
                            } else {
                                *theme::TEXT_SECONDARY
                            };
                            ui.painter()
                                .galley(rect.center() - galley.size() / 2.0, galley, color);
                            if response.clicked() {
                                app.palette.tab = tab;
                                app.palette.selected = 0;
                            }
                        }
                    });
                    ui.painter().hline(
                        ui.min_rect().x_range(),
                        ui.min_rect().bottom(),
                        egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        icons::show(ui, Icon::Search, 16.0, *theme::TEXT_SECONDARY);
                        let input = ui.add(
                            egui::TextEdit::singleline(&mut app.palette.query)
                                .frame(false)
                                .font(egui::FontId::proportional(theme::UI_FONT_SIZE + 2.0))
                                .desired_width(width - 60.0)
                                .margin(egui::Margin::symmetric(4, 10))
                                .hint_text("Type to search classes, files, symbols and actions"),
                        );
                        if app.palette.just_opened || !input.has_focus() {
                            input.request_focus();
                            app.palette.just_opened = false;
                        }
                        if input.changed() {
                            app.palette.selected = 0;
                        }
                    });
                    ui.painter().hline(
                        ui.min_rect().x_range(),
                        ui.min_rect().bottom(),
                        egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
                    );
                    ui.add_space(4.0);
                    if found.is_empty() {
                        ui.add_space(12.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Nothing found").color(theme::TEXT_SECONDARY),
                            );
                        });
                        ui.add_space(12.0);
                    }
                    for (index, ranked) in found.iter().enumerate() {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, 26.0), Sense::click());
                        let row = rect.shrink2(vec2(6.0, 1.0));
                        if index == app.palette.selected {
                            ui.painter().rect_filled(row, 4.0, theme::SELECTION);
                        } else if response.hovered() {
                            ui.painter().rect_filled(row, 4.0, theme::HOVER);
                        }
                        let icon_rect = Rect::from_min_size(
                            egui::pos2(row.left() + 8.0, row.center().y - 8.0),
                            vec2(16.0, 16.0),
                        );
                        let small = egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0);
                        let (title, detail, trailing, icon, color) = match &ranked.item {
                            Hit::Action(action) => (
                                action.label.clone(),
                                action.category.clone(),
                                action.shortcut.clone(),
                                Icon::Run,
                                if action.enabled {
                                    *theme::TEXT_BRIGHT
                                } else {
                                    *theme::TEXT_DISABLED
                                },
                            ),
                            Hit::File { path, line } => (
                                file_name(path),
                                path.parent()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default(),
                                line.map(|line| format!(":{line}")).unwrap_or_default(),
                                app.file_icon(path),
                                *theme::TEXT_BRIGHT,
                            ),
                            Hit::Symbol(symbol) => (
                                symbol.name.clone(),
                                symbol.container.clone().unwrap_or_default(),
                                format!("{}:{}", file_name(&symbol.path), symbol.line + 1),
                                symbol_icon(symbol),
                                *theme::TEXT_BRIGHT,
                            ),
                        };
                        let icon_color = if matches!(ranked.item, Hit::Action(_)) {
                            *theme::TEXT_INFO
                        } else {
                            *icons::ICON
                        };
                        icons::paint(ui.painter(), icon_rect, icon, icon_color);
                        let x = highlighted_text(
                            ui,
                            row,
                            row.left() + 32.0,
                            &title,
                            &ranked.positions,
                            color,
                        );
                        crate::widgets::row_text(
                            ui,
                            row,
                            x + 10.0,
                            &detail,
                            small,
                            *theme::TEXT_INFO,
                        );
                        ui.painter().text(
                            row.right_center() - vec2(10.0, 0.0),
                            egui::Align2::RIGHT_CENTER,
                            trailing,
                            egui::FontId::proportional(theme::UI_FONT_SIZE),
                            *theme::TEXT_SECONDARY,
                        );
                        if response.hovered()
                            && ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO)
                        {
                            app.palette.selected = index;
                        }
                        if response.clicked() {
                            chosen = Some(index);
                        }
                    }
                    ui.add_space(4.0);
                    let (footer, _) = ui.allocate_exact_size(vec2(width, 28.0), Sense::hover());
                    ui.painter().rect_filled(
                        footer,
                        egui::CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 10,
                            se: 10,
                        },
                        theme::FRAME,
                    );
                    ui.painter().text(
                        footer.left_center() + vec2(14.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        "↑↓ to navigate · Enter to run · Esc to close",
                        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                        *theme::TEXT_INFO,
                    );
                });
        });
    if ctx.input(|input| input.pointer.any_click()) && !ctx.is_pointer_over_area() {
        close = true;
    }
    if let Some(index) = chosen
        && let Some(ranked) = found.get(index)
    {
        match &ranked.item {
            Hit::Action(action) => {
                if action.enabled {
                    let id = action.id.clone();
                    app.palette.open = false;
                    app.dispatch(&id, Value::Null);
                }
            }
            Hit::File { path, line } => {
                app.palette.open = false;
                open_location(app, ctx, path, *line);
            }
            Hit::Symbol(symbol) => {
                app.palette.open = false;
                open_location(app, ctx, &symbol.path, Some(symbol.line + 1));
            }
        }
    }
    if close {
        app.palette.open = false;
    }
}
