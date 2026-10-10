//! Collection tab: query bar, paging toolbar and the Table / Tree / JSON result views.

use crate::app::{CollectionTab, Load, MongoRaabit, ResultView, TabId};
use crate::icons::{self, Icon};
use crate::results::{self, TableEvent, TreeEvent};
use crate::style::{self, Palette};
use crate::widgets;
use egui::{CornerRadius, RichText, Stroke, Ui};
use mongo_raabit_core::connection::CollectionKind;

pub const PAGE_SIZES: [u64; 5] = [20, 50, 100, 250, 1000];

enum Action {
    Run { recount: bool },
    Page(u64),
    PageSize(u64),
}

pub fn show(app: &mut MongoRaabit, tab_id: TabId, ui: &mut Ui) {
    let p = Palette::of(ui);
    let server = {
        let Some(tab) = app.collection_tab_mut(tab_id) else {
            return;
        };
        tab.session
    };
    let server_name = app
        .session(server)
        .map(|s| s.profile.name.clone())
        .unwrap_or_default();
    let Some(tab) = app.collection_tab_mut(tab_id) else {
        return;
    };
    let mut actions = Vec::new();
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 14,
            right: 14,
            top: 10,
            bottom: 6,
        })
        .show(ui, |ui| {
            header(ui, &p, tab, &server_name);
            ui.add_space(8.0);
            query_bar(ui, &p, tab, &mut actions);
            if let Some(err) = &tab.query_error {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    icons::icon(ui, Icon::Warning, p.error);
                    ui.label(RichText::new(err).color(p.error));
                });
            }
            ui.add_space(8.0);
            result_toolbar(ui, &p, tab, &mut actions);
        });
    widgets::separator(ui);
    result_area(ui, &p, tab);
    for action in actions {
        match action {
            Action::Run { recount } => {
                if let Some(tab) = app.collection_tab_mut(tab_id) {
                    tab.page = 0;
                    tab.drill.clear();
                    tab.selected = None;
                }
                app.run_query(tab_id, recount);
            }
            Action::Page(page) => {
                if let Some(tab) = app.collection_tab_mut(tab_id) {
                    tab.page = page;
                    tab.selected = None;
                }
                app.run_query(tab_id, false);
            }
            Action::PageSize(size) => {
                if let Some(tab) = app.collection_tab_mut(tab_id) {
                    tab.page_size = size;
                    tab.page = 0;
                }
                app.run_query(tab_id, false);
            }
        }
    }
}

fn header(ui: &mut Ui, p: &Palette, tab: &CollectionTab, server: &str) {
    ui.horizontal(|ui| {
        let icon = match tab.kind {
            CollectionKind::View => Icon::View,
            CollectionKind::TimeSeries => Icon::TimeSeries,
            CollectionKind::GridFsBucket => Icon::Bucket,
            _ => Icon::Collection,
        };
        icons::icon(ui, icon, p.leaf);
        ui.label(RichText::new(server).color(p.text_info));
        icons::icon(ui, Icon::ChevronRight, p.text_info);
        ui.label(RichText::new(&tab.db).color(p.text_secondary));
        icons::icon(ui, Icon::ChevronRight, p.text_info);
        ui.label(
            RichText::new(&tab.collection)
                .size(15.0)
                .strong()
                .color(p.text_strong),
        );
        if tab.kind == CollectionKind::View {
            widgets::badge(ui, "view", p.keyword);
        }
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| match &tab.count {
                Load::Ready(n) => {
                    widgets::badge(ui, &format!("{} documents", group(*n)), p.accent_text);
                }
                Load::Loading => {
                    ui.spinner();
                }
                Load::Failed(err) => {
                    widgets::badge(ui, "count failed", p.error).on_hover_text(err);
                }
                Load::Idle => {}
            },
        );
    });
}

fn query_bar(ui: &mut Ui, p: &Palette, tab: &mut CollectionTab, actions: &mut Vec<Action>) {
    egui::Frame::new()
        .fill(p.raised)
        .stroke(Stroke::new(1.0_f32, p.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            let mut enter = false;
            ui.horizontal(|ui| {
                let width = ui.available_width() - 110.0;
                let r = widgets::query_field(
                    ui,
                    "Filter",
                    &mut tab.query.filter,
                    "{ field: \"value\" }",
                    width,
                );
                enter |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if widgets::primary_button(ui, Some(Icon::Play), "Run")
                    .on_hover_text("Run query (Enter)")
                    .clicked()
                {
                    actions.push(Action::Run { recount: true });
                }
            });
            ui.horizontal(|ui| {
                let total = ui.available_width() - 6.0 * 5.0;
                let r = widgets::query_field(
                    ui,
                    "Projection",
                    &mut tab.query.projection,
                    "{ name: 1 }",
                    total * 0.34,
                );
                enter |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let r = widgets::query_field(
                    ui,
                    "Sort",
                    &mut tab.query.sort,
                    "{ _id: -1 }",
                    total * 0.30,
                );
                enter |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let r = widgets::query_field(ui, "Skip", &mut tab.query.skip, "0", total * 0.11);
                enter |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let r = widgets::query_field(ui, "Limit", &mut tab.query.limit, "0", total * 0.11);
                enter |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if widgets::icon_button(ui, Icon::Close, "Reset query").clicked() {
                    tab.query = Default::default();
                    actions.push(Action::Run { recount: true });
                }
            });
            if enter {
                actions.push(Action::Run { recount: true });
            }
        });
}

/// `1234567` → `1,234,567`.
pub fn group(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn result_toolbar(ui: &mut Ui, p: &Palette, tab: &mut CollectionTab, actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        widgets::segmented(
            ui,
            &mut tab.view,
            &[
                (ResultView::Table, Icon::Table, "Table"),
                (ResultView::Tree, Icon::Tree, "Tree"),
                (ResultView::Json, Icon::Json, "JSON"),
            ],
        );
        if tab.view == ResultView::Table && !tab.drill.is_empty() {
            ui.add_space(8.0);
            if let Some(depth) = results::breadcrumb(ui, p, &tab.drill) {
                tab.drill.truncate(depth);
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let page = tab.result.ready();
            let has_more = page.is_some_and(|pg| pg.has_more);
            if widgets::icon_button_tinted(ui, Icon::ChevronRight, "Next page", None, has_more)
                .clicked()
            {
                actions.push(Action::Page(tab.page + 1));
            }
            let first = page.map(|pg| pg.first_index()).unwrap_or(0);
            let last = page
                .map(|pg| first + pg.documents.len() as u64)
                .unwrap_or(0)
                .saturating_sub(1);
            let total = match &tab.count {
                Load::Ready(n) => format!(" of {}", group(*n)),
                _ => String::new(),
            };
            let range = if first == 0 {
                "No documents".to_owned()
            } else {
                format!("{} – {}{}", group(first), group(last), total)
            };
            ui.label(RichText::new(range).color(p.text_secondary));
            if widgets::icon_button_tinted(
                ui,
                Icon::ChevronLeft,
                "Previous page",
                None,
                tab.page > 0,
            )
            .clicked()
            {
                actions.push(Action::Page(tab.page.saturating_sub(1)));
            }
            let mut size = tab.page_size;
            egui::ComboBox::from_id_salt(("raabit-page-size", &tab.db, &tab.collection))
                .selected_text(format!("{size} / page"))
                .width(96.0)
                .show_ui(ui, |ui| {
                    for option in PAGE_SIZES {
                        ui.selectable_value(&mut size, option, format!("{option} / page"));
                    }
                });
            if size != tab.page_size {
                actions.push(Action::PageSize(size));
            }
            if let Some(page) = page {
                ui.add_space(6.0);
                ui.label(RichText::new(style::duration(page.elapsed)).color(p.text_info))
                    .on_hover_text("Query time of this page");
                icons::icon(ui, Icon::TimeSeries, p.text_info);
            }
        });
    });
}

fn result_area(ui: &mut Ui, p: &Palette, tab: &mut CollectionTab) {
    match &tab.result {
        Load::Idle => {}
        Load::Loading => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.label(RichText::new("Running query…").color(p.text_secondary));
            });
        }
        Load::Failed(err) => {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                icons::icon(ui, Icon::Warning, p.error);
                ui.add(egui::Label::new(RichText::new(err).color(p.error)).wrap());
            });
        }
        Load::Ready(page) => {
            if page.documents.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    icons::icon(ui, Icon::Filter, p.text_info);
                    ui.add_space(4.0);
                    ui.label(RichText::new("No documents match the query").color(p.text_secondary));
                });
                return;
            }
            let docs = page.documents.clone();
            let first = page.first_index();
            match tab.view {
                ResultView::Table => match results::table(ui, tab, &docs, first) {
                    Some(TableEvent::Drill(field)) => {
                        tab.drill.push(field);
                    }
                    Some(TableEvent::Select(i)) | Some(TableEvent::Open(i)) => {
                        tab.selected = Some(i)
                    }
                    None => {}
                },
                ResultView::Tree => match results::tree(ui, tab, &docs, first) {
                    Some(TreeEvent::Toggle(key)) => {
                        if !tab.open.remove(&key) {
                            tab.open.insert(key);
                        }
                    }
                    Some(TreeEvent::Select(i)) | Some(TreeEvent::Open(i)) => tab.selected = Some(i),
                    None => {}
                },
                ResultView::Json => results::json(ui, &docs, first),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn groups_digits() {
        assert_eq!(super::group(0), "0");
        assert_eq!(super::group(1200), "1,200");
        assert_eq!(super::group(1_234_567), "1,234,567");
    }
}
