//! "Search Everywhere" popup: actions and project files with keyboard navigation.

use crate::icons::{self, Icon};
use crate::{IdeApp, theme};
use eframe::egui::{self, Rect, Sense, vec2};
use serde_json::{Value, json};
use std::path::PathBuf;

const MAX_ROWS: usize = 14;

#[derive(Default)]
pub struct Palette {
    pub open: bool,
    query: String,
    selected: usize,
    just_opened: bool,
}

impl Palette {
    pub fn show(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.just_opened = true;
    }
}

enum Item {
    Action {
        id: String,
        label: String,
        category: String,
        shortcut: String,
        enabled: bool,
    },
    File(PathBuf),
}

/// Case-insensitive subsequence match score; lower is better, `None` means no match.
pub fn fuzzy_score(candidate: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }
    let candidate = candidate.to_lowercase();
    let query = query.to_lowercase();
    if let Some(position) = candidate.find(&query) {
        return Some(position);
    }
    let mut chars = candidate.char_indices();
    let mut gaps = 0;
    let mut last = None;
    for wanted in query.chars() {
        let (index, _) = chars.find(|(_, c)| *c == wanted)?;
        if let Some(previous) = last
            && index > previous + 1
        {
            gaps += 1;
        }
        last = Some(index);
    }
    Some(100 + gaps)
}

fn items(app: &IdeApp, query: &str) -> Vec<Item> {
    let actions: Vec<(usize, Item)> = app
        .commands
        .specs()
        .into_iter()
        .filter(IdeApp::parameterless)
        .filter_map(|spec| {
            let score = fuzzy_score(&spec.label, query)
                .or_else(|| fuzzy_score(&spec.id, query).map(|score| score + 50))?;
            Some((
                score,
                Item::Action {
                    enabled: app.commands.is_enabled(&spec.id, &app.state),
                    id: spec.id,
                    label: spec.label,
                    category: spec.category,
                    shortcut: crate::keys::display(spec.shortcut.as_deref().unwrap_or_default()),
                },
            ))
        })
        .collect();
    let mut files: Vec<(usize, Item)> = if query.is_empty() {
        Vec::new()
    } else {
        app.state
            .files
            .iter()
            .filter(|entry| !entry.is_dir)
            .filter_map(|entry| {
                let name = IdeApp::file_name(&entry.relative_path);
                let score = fuzzy_score(&name, query)?;
                Some((score, Item::File(entry.relative_path.clone())))
            })
            .take(500)
            .collect()
    };
    files.sort_by_key(|(score, _)| *score);
    files.truncate(MAX_ROWS / 2);
    // Stable sort keeps files ahead of actions with an equal score.
    let mut all: Vec<(usize, Item)> = files.into_iter().chain(actions).collect();
    all.sort_by_key(|(score, _)| *score);
    all.into_iter()
        .map(|(_, item)| item)
        .take(MAX_ROWS)
        .collect()
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    if !app.palette.open {
        return;
    }
    let found = items(app, &app.palette.query);
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
                    // Header tabs.
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        let (rect, _) = ui.allocate_exact_size(vec2(44.0, 32.0), Sense::hover());
                        ui.painter().rect_filled(
                            rect.shrink2(vec2(0.0, 5.0)),
                            4.0,
                            theme::SELECTION,
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "All",
                            egui::FontId::proportional(theme::UI_FONT_SIZE),
                            theme::TEXT_BRIGHT,
                        );
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Actions · Files").color(theme::TEXT_INFO));
                    });
                    ui.painter().hline(
                        ui.min_rect().x_range(),
                        ui.min_rect().bottom(),
                        egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        icons::show(ui, Icon::Search, 16.0, theme::TEXT_SECONDARY);
                        let input = ui.add(
                            egui::TextEdit::singleline(&mut app.palette.query)
                                .frame(false)
                                .font(egui::FontId::proportional(theme::UI_FONT_SIZE + 2.0))
                                .desired_width(width - 60.0)
                                .margin(egui::Margin::symmetric(4, 10))
                                .hint_text("Type to search actions and files"),
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
                    for (index, item) in found.iter().enumerate() {
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
                        match item {
                            Item::Action {
                                label,
                                category,
                                shortcut,
                                enabled,
                                ..
                            } => {
                                icons::paint(ui.painter(), icon_rect, Icon::Run, theme::TEXT_INFO);
                                let color = if *enabled {
                                    theme::TEXT_BRIGHT
                                } else {
                                    theme::TEXT_DISABLED
                                };
                                let x = crate::widgets::row_text(
                                    ui,
                                    row,
                                    row.left() + 32.0,
                                    label,
                                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                                    color,
                                );
                                crate::widgets::row_text(
                                    ui,
                                    row,
                                    x + 10.0,
                                    category,
                                    egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                                    theme::TEXT_INFO,
                                );
                                ui.painter().text(
                                    row.right_center() - vec2(10.0, 0.0),
                                    egui::Align2::RIGHT_CENTER,
                                    shortcut,
                                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                                    theme::TEXT_SECONDARY,
                                );
                            }
                            Item::File(path) => {
                                let name = IdeApp::file_name(path);
                                icons::paint(
                                    ui.painter(),
                                    icon_rect,
                                    Icon::for_file(&name),
                                    icons::ICON,
                                );
                                let x = crate::widgets::row_text(
                                    ui,
                                    row,
                                    row.left() + 32.0,
                                    &name,
                                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                                    theme::TEXT_BRIGHT,
                                );
                                let parent = path
                                    .parent()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default();
                                crate::widgets::row_text(
                                    ui,
                                    row,
                                    x + 10.0,
                                    &parent,
                                    egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                                    theme::TEXT_INFO,
                                );
                            }
                        }
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
                        theme::TEXT_INFO,
                    );
                });
        });
    if ctx.input(|input| input.pointer.any_click()) && !ctx.is_pointer_over_area() {
        close = true;
    }
    if let Some(index) = chosen
        && let Some(item) = found.get(index)
    {
        match item {
            Item::Action { id, enabled, .. } => {
                if *enabled {
                    let id = id.clone();
                    app.palette.open = false;
                    app.dispatch(&id, Value::Null);
                }
            }
            Item::File(path) => {
                let path = path.clone();
                app.palette.open = false;
                app.dispatch("editor.openFile", json!({"path": path}));
                ctx.memory_mut(|memory| memory.request_focus(crate::editor::editor_id()));
            }
        }
    }
    if close {
        app.palette.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_prefers_substrings_then_subsequences() {
        assert_eq!(fuzzy_score("Save All", "save"), Some(0));
        assert_eq!(fuzzy_score("Save All", "all"), Some(5));
        assert!(fuzzy_score("Toggle Terminal", "tgt").is_some_and(|score| score >= 100));
        assert_eq!(fuzzy_score("Build", "xyz"), None);
        assert_eq!(fuzzy_score("anything", ""), Some(0));
    }
}
