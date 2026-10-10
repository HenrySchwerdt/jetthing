//! Settings › Plugins page (reference scene 24): Marketplace / Installed tabs, the
//! plugin list with enable checkboxes, and a detail pane. Jetthing has no plugin
//! marketplace; the Installed tab lists the manifest plugins found on disk.

use crate::{theme, widgets};
use eframe::egui::{self, RichText, Sense, vec2};
use jetthing_core::plugin_commands::{PluginEntry, plugin_dirs};
use serde_json::{Value, json};

#[derive(Default)]
pub struct PluginsPage {
    marketplace: bool,
    query: String,
    selected: Option<usize>,
}

pub fn show(
    ui: &mut egui::Ui,
    page: &mut PluginsPage,
    entries: &[PluginEntry],
    project: Option<&std::path::Path>,
    requests: &mut Vec<(String, Value)>,
) {
    ui.horizontal(|ui| {
        for (marketplace, label) in [
            (true, "Marketplace".to_owned()),
            (false, format!("Installed {}", entries.len())),
        ] {
            let selected = page.marketplace == marketplace;
            let text = RichText::new(label).color(if selected {
                *theme::TEXT_BRIGHT
            } else {
                *theme::TEXT_SECONDARY
            });
            let response = ui.add(egui::Button::new(text).frame(false));
            if selected {
                let rect = response.rect;
                ui.painter().hline(
                    rect.x_range(),
                    rect.bottom() + 2.0,
                    egui::Stroke::new(2.0_f32, theme::ACCENT),
                );
            }
            if response.clicked() {
                page.marketplace = marketplace;
            }
            ui.add_space(12.0);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Reload")
                .on_hover_text("Discover plugin manifests again")
                .clicked()
            {
                requests.push(("plugin.refresh".into(), Value::Null));
            }
        });
    });
    ui.separator();
    if page.marketplace {
        marketplace(ui, project);
        return;
    }
    let area = ui.available_rect_before_wrap();
    let list_width = (area.width() * 0.45).clamp(220.0, 380.0);
    let list_rect = egui::Rect::from_min_size(area.min, vec2(list_width, area.height()));
    let detail_rect =
        egui::Rect::from_min_max(egui::pos2(list_rect.right() + 16.0, area.top()), area.max);
    ui.painter().vline(
        list_rect.right() + 8.0,
        area.y_range(),
        egui::Stroke::new(1.0_f32, theme::POPUP_BORDER),
    );
    let mut list_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(list_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    widgets::search_field(
        &mut list_ui,
        egui::Id::new("plugins_search"),
        &mut page.query,
        "Search installed plugins",
        list_width - 40.0,
    );
    list_ui.add_space(4.0);
    let query = page.query.trim().to_lowercase();
    egui::ScrollArea::vertical()
        .id_salt("plugins_list")
        .auto_shrink([false, false])
        .show(&mut list_ui, |ui| {
            if entries.is_empty() {
                ui.label(RichText::new("No plugins installed").color(theme::TEXT_INFO));
            }
            for (index, entry) in entries.iter().enumerate() {
                if !query.is_empty() && !matches(entry, &query) {
                    continue;
                }
                plugin_row(ui, page, index, entry, requests);
            }
        });
    let mut detail_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(detail_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    match page.selected.and_then(|index| entries.get(index)) {
        Some(entry) => details(&mut detail_ui, entry),
        None => {
            detail_ui.add_space(24.0);
            detail_ui
                .label(RichText::new("Select a plugin to see its details").color(theme::TEXT_INFO));
        }
    }
    ui.allocate_rect(area, Sense::hover());
}

fn matches(entry: &PluginEntry, query: &str) -> bool {
    match &entry.manifest {
        Ok(manifest) => {
            manifest.name.to_lowercase().contains(query)
                || manifest.id.to_lowercase().contains(query)
                || manifest
                    .capabilities
                    .iter()
                    .any(|capability| capability.to_lowercase().contains(query))
        }
        Err(_) => entry
            .path
            .display()
            .to_string()
            .to_lowercase()
            .contains(query),
    }
}

fn plugin_row(
    ui: &mut egui::Ui,
    page: &mut PluginsPage,
    index: usize,
    entry: &PluginEntry,
    requests: &mut Vec<(String, Value)>,
) {
    let selected = page.selected == Some(index);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 4.0, theme::SELECTION);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, theme::HOVER);
    }
    let (title, subtitle, color) = match &entry.manifest {
        Ok(manifest) => (
            manifest.name.clone(),
            format!("{}   {}", manifest.version, entry.source.label()),
            *theme::TEXT_BRIGHT,
        ),
        Err(_) => (
            entry
                .path
                .parent()
                .and_then(|dir| dir.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "plugin".into()),
            "Failed to load".to_owned(),
            *theme::ERROR,
        ),
    };
    let font = theme::semibold(theme::UI_FONT_SIZE);
    ui.painter().text(
        rect.left_top() + vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        title,
        font,
        color,
    );
    ui.painter().text(
        rect.left_top() + vec2(10.0, 26.0),
        egui::Align2::LEFT_TOP,
        subtitle,
        egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
        *theme::TEXT_INFO,
    );
    if let Some(id) = entry.id() {
        let mut enabled = entry.enabled;
        let checkbox_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 18.0, rect.center().y),
            vec2(20.0, 20.0),
        );
        let checkbox = ui
            .put(checkbox_rect, egui::Checkbox::without_text(&mut enabled))
            .on_hover_text(if entry.enabled { "Disable" } else { "Enable" });
        if checkbox.changed() {
            requests.push((
                "plugin.setEnabled".into(),
                json!({"id": id, "enabled": enabled}),
            ));
        }
    }
    if response.clicked() {
        page.selected = Some(index);
    }
}

fn details(ui: &mut egui::Ui, entry: &PluginEntry) {
    ui.add_space(4.0);
    match &entry.manifest {
        Ok(manifest) => {
            ui.label(
                RichText::new(entry.source.label())
                    .color(theme::TEXT_SECONDARY)
                    .background_color(theme::HOVER),
            );
            ui.label(
                RichText::new(&manifest.name)
                    .font(theme::semibold(theme::UI_FONT_SIZE + 5.0))
                    .color(theme::TEXT_BRIGHT),
            );
            ui.label(RichText::new(&manifest.id).color(theme::MODIFIED));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let status = if entry.enabled { "Enabled" } else { "Disabled" };
                ui.label(RichText::new(status).color(if entry.enabled {
                    *theme::SUCCESS
                } else {
                    *theme::TEXT_INFO
                }));
                ui.label(RichText::new(&manifest.version).color(theme::TEXT_SECONDARY));
            });
            ui.add_space(10.0);
            ui.label(RichText::new("Capabilities").color(theme::TEXT_BRIGHT));
            if manifest.capabilities.is_empty() {
                ui.label(RichText::new("None declared").color(theme::TEXT_INFO));
            }
            ui.horizontal_wrapped(|ui| {
                for capability in &manifest.capabilities {
                    ui.label(
                        RichText::new(capability)
                            .color(theme::TEXT)
                            .background_color(theme::HOVER),
                    );
                }
            });
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!("Plugin API version {}", manifest.api_version))
                    .color(theme::TEXT_SECONDARY),
            );
        }
        Err(error) => {
            ui.label(
                RichText::new("This plugin failed to load")
                    .font(theme::semibold(theme::UI_FONT_SIZE + 2.0))
                    .color(theme::ERROR),
            );
            ui.add(egui::Label::new(RichText::new(error).color(theme::TEXT)).wrap());
        }
    }
    ui.add_space(10.0);
    ui.label(RichText::new("Manifest").color(theme::TEXT_BRIGHT));
    ui.add(
        egui::Label::new(
            RichText::new(entry.path.display().to_string()).color(theme::TEXT_SECONDARY),
        )
        .wrap(),
    );
    ui.add_space(10.0);
    ui.add(
        egui::Label::new(
            RichText::new(
                "Manifest plugins declare capabilities only; Jetthing does not execute plugin code yet.",
            )
            .color(theme::TEXT_INFO),
        )
        .wrap(),
    );
}

fn marketplace(ui: &mut egui::Ui, project: Option<&std::path::Path>) {
    ui.add_space(16.0);
    ui.label(
        RichText::new("No plugin marketplace yet")
            .font(theme::semibold(theme::UI_FONT_SIZE + 2.0))
            .color(theme::TEXT_BRIGHT),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new(
            "Jetthing loads manifest plugins (a folder with a plugin.toml) from these directories:",
        )
        .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
    for (source, dir) in plugin_dirs(project) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(source.label()).color(theme::TEXT_INFO));
            ui.label(RichText::new(dir.display().to_string()).color(theme::TEXT));
        });
    }
}
