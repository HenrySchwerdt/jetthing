//! Marketplace tab of the Plugins page: JetBrains Marketplace search
//! results and plugin details. Plugins are downloaded only when the user
//! clicks Install; Jetthing then converts data-only plugins and explains why
//! the others cannot run.

use crate::plugin_manager::{PluginManager, paint_plugin_icon, placeholder};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, RichText, Sense, vec2};
use jetthing_core::InstallOutcome;
use jetthing_marketplace::{convert::html_to_text, convert::plugin_id, tag_hint};
use serde_json::json;

const ROW_HEIGHT: f32 = 64.0;

/// Compact count: 11767296 → "11.8M".
pub fn compact(count: u64) -> String {
    match count {
        0..=999 => count.to_string(),
        1_000..=999_999 => format!("{:.1}K", count as f64 / 1_000.0),
        _ => format!("{:.1}M", count as f64 / 1_000_000.0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Available,
    Installing,
    Installed,
    Unsupported,
}

fn state(app: &IdeApp, id: u64, xml_id: &str) -> State {
    let marketplace = &app.state.marketplace;
    if marketplace.installing == Some(id) {
        return State::Installing;
    }
    match marketplace.outcome(id) {
        Some(InstallOutcome::Failed(_)) => return State::Unsupported,
        Some(InstallOutcome::Installed { .. }) => return State::Installed,
        None => {}
    }
    if app
        .state
        .plugin_host
        .registry
        .get(&plugin_id(xml_id, id))
        .is_some()
    {
        State::Installed
    } else {
        State::Available
    }
}

/// Green outline Install button (or the current state).
fn install_button(ui: &mut egui::Ui, state: State, size: egui::Vec2) -> bool {
    match state {
        State::Available => ui
            .add(
                egui::Button::new(RichText::new("Install").color(theme::SUCCESS))
                    .stroke(egui::Stroke::new(1.0_f32, theme::SUCCESS))
                    .fill(Color32::TRANSPARENT)
                    .min_size(size),
            )
            .clicked(),
        State::Installing => {
            ui.add_sized(size, egui::Spinner::new().size(14.0));
            false
        }
        State::Installed => {
            ui.add_enabled(false, egui::Button::new("Installed").min_size(size));
            false
        }
        State::Unsupported => {
            ui.add_enabled(false, egui::Button::new("Unsupported").min_size(size));
            false
        }
    }
}

pub fn list(app: &mut IdeApp, ui: &mut egui::Ui, manager: &mut PluginManager) {
    if !manager.marketplace_requested {
        manager.marketplace_requested = true;
        if app.state.marketplace.results.is_empty() && !app.state.marketplace.searching {
            app.dispatch("marketplace.search", json!({"query": manager.search}));
        }
    }
    let marketplace = &app.state.marketplace;
    if marketplace.searching && marketplace.results.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| ui.add(egui::Spinner::new().size(18.0)));
        return;
    }
    if let Some(error) = marketplace.search_error.clone() {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Cannot reach JetBrains Marketplace").color(theme::TEXT_BRIGHT));
            ui.label(RichText::new(error).small().color(theme::TEXT_SECONDARY));
            if ui.button("Retry").clicked() {
                app.dispatch("marketplace.search", json!({"query": manager.search}));
            }
        });
        return;
    }
    let title = if marketplace.query.trim().is_empty() {
        "Most downloaded".to_owned()
    } else {
        format!("Search results ({})", marketplace.total)
    };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::NOTIFICATION);
    ui.painter().text(
        rect.left_center() + vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(theme::UI_FONT_SIZE),
        theme::TEXT,
    );
    let results = marketplace.results.clone();
    let selected = marketplace
        .details
        .as_ref()
        .map(|details| details.id)
        .or(marketplace.details_loading);
    let mut select = None;
    let mut install = None;
    egui::ScrollArea::vertical()
        .id_salt("marketplace_list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for plugin in &results {
                let (row, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
                if selected == Some(plugin.id) {
                    ui.painter().rect_filled(row, 0.0, theme::SELECTION);
                } else if response.hovered() {
                    ui.painter().rect_filled(row, 0.0, theme::HOVER);
                }
                paint_plugin_icon(
                    ui.painter(),
                    Rect::from_min_size(row.left_center() + vec2(10.0, -18.0), vec2(36.0, 36.0)),
                    &plugin.name,
                );
                let name = ui.painter().layout(
                    plugin.name.clone(),
                    theme::semibold(theme::UI_FONT_SIZE),
                    theme::TEXT_BRIGHT,
                    row.width() - 160.0,
                );
                ui.painter().galley(
                    egui::pos2(row.left() + 58.0, row.top() + 14.0),
                    name,
                    theme::TEXT_BRIGHT,
                );
                let mut detail = format!("{} downloads", compact(plugin.downloads));
                if let Some(rating) = plugin.rating.filter(|rating| *rating > 0.0) {
                    detail.push_str(&format!("  ·  {rating:.2}"));
                }
                if let Some(vendor) = &plugin.vendor {
                    detail.push_str(&format!("  ·  {}", vendor.display()));
                }
                ui.painter().text(
                    egui::pos2(row.left() + 58.0, row.top() + 36.0),
                    egui::Align2::LEFT_TOP,
                    detail,
                    egui::FontId::proportional(theme::UI_FONT_SIZE - 0.5),
                    theme::TEXT_INFO,
                );
                let button_rect = Rect::from_min_size(
                    egui::pos2(row.right() - 86.0, row.top() + 10.0),
                    vec2(74.0, 24.0),
                );
                let mut button_ui = ui.new_child(egui::UiBuilder::new().max_rect(button_rect));
                if install_button(
                    &mut button_ui,
                    state(app, plugin.id, &plugin.xml_id),
                    vec2(74.0, 24.0),
                ) {
                    install = Some(plugin.id);
                }
                if response.clicked() {
                    select = Some(plugin.id);
                }
            }
        });
    if let Some(id) = install {
        app.dispatch("marketplace.install", json!({"id": id}));
        select.get_or_insert(id);
    }
    if let Some(id) = select
        && app.state.marketplace.details.as_ref().map(|d| d.id) != Some(id)
    {
        app.dispatch("marketplace.details", json!({"id": id}));
    }
}

pub fn details(app: &mut IdeApp, ui: &mut egui::Ui) {
    let marketplace = &app.state.marketplace;
    if marketplace.details_loading.is_some() {
        ui.centered_and_justified(|ui| ui.add(egui::Spinner::new().size(18.0)));
        return;
    }
    if let Some(error) = &marketplace.details_error {
        placeholder(ui, error);
        return;
    }
    let Some(details) = marketplace.details.clone() else {
        placeholder(ui, "Select plugin to preview details");
        return;
    };
    let base = jetthing_marketplace::api::DEFAULT_BASE_URL;
    let tags: Vec<String> = details.tags.iter().map(|tag| tag.name.clone()).collect();
    if let Some(tag) = tags.first() {
        egui::Frame::new()
            .fill(theme::NOTIFICATION)
            .corner_radius(3)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.label(RichText::new(tag).color(theme::TEXT_SECONDARY));
            });
        ui.add_space(4.0);
    }
    ui.label(
        RichText::new(&details.name)
            .font(theme::semibold(20.0))
            .color(theme::TEXT_BRIGHT),
    );
    ui.horizontal(|ui| {
        if let Some(vendor) = &details.vendor {
            ui.label(RichText::new(vendor.display()).color(theme::MODIFIED));
        }
        ui.hyperlink_to("Plugin homepage ↗", details.homepage(base));
    });
    ui.add_space(8.0);
    let current = state(app, details.id, &details.xml_id);
    ui.horizontal(|ui| {
        if install_button(ui, current, vec2(84.0, 28.0)) {
            app.dispatch("marketplace.install", json!({"id": details.id}));
        }
        ui.label(
            RichText::new(format!("{} downloads", compact(details.downloads))).color(theme::TEXT),
        );
    });
    ui.add_space(8.0);
    let (fill, text) = match app.state.marketplace.outcome(details.id) {
        Some(InstallOutcome::Installed { summary, .. }) => {
            (Color32::from_rgb(0x25, 0x3A, 0x2C), format!("{summary}. Find it under Installed."))
        }
        Some(InstallOutcome::Failed(reason)) => (Color32::from_rgb(0x40, 0x25, 0x27), reason.clone()),
        None => (
            theme::NOTIFICATION,
            match tag_hint(&tags) {
                Some(hint) => format!(
                    "{hint}. Jetthing checks the plugin when you click Install and converts its themes, color schemes, keymaps, live templates and TextMate grammars."
                ),
                None => "Jetthing checks the plugin when you click Install. Only themes, color schemes, keymaps, live templates and TextMate grammars work without the IntelliJ Platform; other plugins are reported as unsupported.".into(),
            },
        ),
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(4)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).color(theme::TEXT));
        });
    ui.add_space(10.0);
    egui::ScrollArea::vertical()
        .id_salt(("marketplace_details", details.id))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(RichText::new(html_to_text(&details.description)).color(theme::TEXT));
            ui.add_space(12.0);
            for (key, value) in [
                ("Plugin ID", details.xml_id.clone()),
                ("Tags", tags.join(", ")),
            ] {
                ui.horizontal_top(|ui| {
                    ui.add_sized([90.0, 18.0], egui::Label::new(RichText::new(key).color(theme::TEXT_SECONDARY)));
                    ui.label(value);
                });
            }
            ui.horizontal(|ui| {
                ui.add_sized([90.0, 18.0], egui::Label::new(RichText::new("Marketplace").color(theme::TEXT_SECONDARY)));
                ui.hyperlink(details.page_url(base));
            });
            if let Some(license) = details.urls.license_url.as_deref().filter(|url| !url.is_empty()) {
                ui.horizontal(|ui| {
                    ui.add_sized([90.0, 18.0], egui::Label::new(RichText::new("License").color(theme::TEXT_SECONDARY)));
                    ui.hyperlink(license);
                });
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("Downloaded from plugins.jetbrains.com only when you click Install. Each plugin keeps its own license.")
                    .small()
                    .color(theme::TEXT_INFO),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_compact() {
        assert_eq!(compact(999), "999");
        assert_eq!(compact(35_500), "35.5K");
        assert_eq!(compact(11_767_296), "11.8M");
    }
}
