//! Settings › Plugins page (reference scene 24): Marketplace / Installed tabs, the
//! plugin list with enable checkboxes, and a detail pane. Jetthing has no plugin
//! marketplace; the Installed tab lists the plugin registry: compiled-in plugins,
//! manifests found in the plugin directories, and manifests that failed to load.

use crate::{theme, widgets};
use eframe::egui::{self, RichText, Sense, vec2};
use jetthing_core::plugin_dirs;
use jetthing_plugins::{PluginRegistry, PluginSource, PluginStatus};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Default)]
pub struct PluginsPage {
    marketplace: bool,
    query: String,
    selected: Option<usize>,
}

/// One row of the Installed list.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginRow {
    /// `None` for a manifest that failed to load.
    pub id: Option<String>,
    pub name: String,
    pub version: String,
    pub source: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub api_version: Option<u32>,
    pub status: Option<PluginStatus>,
    pub enabled: bool,
    pub can_toggle: bool,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

/// Rows for every registry entry, followed by the manifests that failed to load.
pub fn rows(registry: &PluginRegistry) -> Vec<PluginRow> {
    let mut rows: Vec<PluginRow> = registry
        .entries()
        .iter()
        .map(|entry| {
            let manifest = &entry.manifest;
            let (source, path) = match &entry.source {
                PluginSource::Bundled => ("Bundled".to_owned(), None),
                PluginSource::Directory(dir) => ("Installed".to_owned(), Some(dir.clone())),
            };
            PluginRow {
                id: Some(manifest.id.clone()),
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                source,
                description: manifest.description.clone(),
                capabilities: manifest.capabilities.clone(),
                api_version: Some(manifest.api_version),
                status: Some(entry.status.clone()),
                enabled: !matches!(
                    entry.status,
                    PluginStatus::Disabled | PluginStatus::Unsupported(_)
                ),
                can_toggle: entry.can_toggle(),
                path,
                error: match &entry.status {
                    PluginStatus::Failed(message) | PluginStatus::Unsupported(message) => {
                        Some(message.clone())
                    }
                    _ => None,
                },
            }
        })
        .collect();
    rows.extend(registry.problems.iter().map(|(path, error)| {
        PluginRow {
            id: None,
            name: path
                .parent()
                .and_then(|dir| dir.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "plugin".into()),
            version: String::new(),
            source: "Failed to load".into(),
            description: String::new(),
            capabilities: Vec::new(),
            api_version: None,
            status: None,
            enabled: false,
            can_toggle: false,
            path: Some(path.clone()),
            error: Some(error.to_string()),
        }
    }));
    rows
}

pub fn show(
    ui: &mut egui::Ui,
    page: &mut PluginsPage,
    registry: &PluginRegistry,
    project: Option<&std::path::Path>,
    requests: &mut Vec<(String, Value)>,
) {
    let entries = rows(registry);
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

pub fn matches(entry: &PluginRow, query: &str) -> bool {
    entry.name.to_lowercase().contains(query)
        || entry
            .id
            .as_deref()
            .is_some_and(|id| id.to_lowercase().contains(query))
        || entry
            .capabilities
            .iter()
            .any(|capability| capability.to_lowercase().contains(query))
        || entry
            .path
            .as_ref()
            .is_some_and(|path| path.display().to_string().to_lowercase().contains(query))
}

fn plugin_row(
    ui: &mut egui::Ui,
    page: &mut PluginsPage,
    index: usize,
    entry: &PluginRow,
    requests: &mut Vec<(String, Value)>,
) {
    let selected = page.selected == Some(index);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 4.0, theme::SELECTION);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, theme::HOVER);
    }
    let (subtitle, color) = match (&entry.id, &entry.status) {
        (None, _) => ("Failed to load".to_owned(), *theme::ERROR),
        (Some(_), Some(PluginStatus::Failed(_))) => {
            (format!("{}   Failed", entry.version), *theme::ERROR)
        }
        _ => (
            format!("{}   {}", entry.version, entry.source),
            *theme::TEXT_BRIGHT,
        ),
    };
    let font = theme::semibold(theme::UI_FONT_SIZE);
    ui.painter().text(
        rect.left_top() + vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        &entry.name,
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
    if let Some(id) = &entry.id
        && entry.can_toggle
    {
        let mut enabled = entry.enabled;
        let checkbox_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 18.0, rect.center().y),
            vec2(20.0, 20.0),
        );
        let checkbox = ui
            .put(checkbox_rect, egui::Checkbox::without_text(&mut enabled))
            .on_hover_text(if entry.enabled { "Disable" } else { "Enable" });
        if checkbox.changed() {
            let command = if enabled {
                "plugin.enable"
            } else {
                "plugin.disable"
            };
            requests.push((command.into(), json!({ "id": id })));
        }
    }
    if response.clicked() {
        page.selected = Some(index);
    }
}

fn details(ui: &mut egui::Ui, entry: &PluginRow) {
    ui.add_space(4.0);
    if let Some(id) = &entry.id {
        ui.label(
            RichText::new(&entry.source)
                .color(theme::TEXT_SECONDARY)
                .background_color(theme::HOVER),
        );
        ui.label(
            RichText::new(&entry.name)
                .font(theme::semibold(theme::UI_FONT_SIZE + 5.0))
                .color(theme::TEXT_BRIGHT),
        );
        ui.label(RichText::new(id).color(theme::MODIFIED));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let status = entry
                .status
                .as_ref()
                .map(PluginStatus::label)
                .unwrap_or_default();
            ui.label(RichText::new(status).color(if entry.enabled {
                *theme::SUCCESS
            } else {
                *theme::TEXT_INFO
            }));
            ui.label(RichText::new(&entry.version).color(theme::TEXT_SECONDARY));
        });
        if !entry.description.is_empty() {
            ui.add_space(6.0);
            ui.add(egui::Label::new(RichText::new(&entry.description).color(theme::TEXT)).wrap());
        }
        ui.add_space(10.0);
        ui.label(RichText::new("Capabilities").color(theme::TEXT_BRIGHT));
        if entry.capabilities.is_empty() {
            ui.label(RichText::new("None declared").color(theme::TEXT_INFO));
        }
        ui.horizontal_wrapped(|ui| {
            for capability in &entry.capabilities {
                ui.label(
                    RichText::new(capability)
                        .color(theme::TEXT)
                        .background_color(theme::HOVER),
                );
            }
        });
        if let Some(version) = entry.api_version {
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!("Plugin API version {version}")).color(theme::TEXT_SECONDARY),
            );
        }
    } else {
        ui.label(
            RichText::new("This plugin failed to load")
                .font(theme::semibold(theme::UI_FONT_SIZE + 2.0))
                .color(theme::ERROR),
        );
    }
    if let Some(error) = &entry.error {
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(error).color(theme::TEXT)).wrap());
    }
    if let Some(path) = &entry.path {
        ui.add_space(10.0);
        ui.label(RichText::new("Location").color(theme::TEXT_BRIGHT));
        ui.add(
            egui::Label::new(
                RichText::new(path.display().to_string()).color(theme::TEXT_SECONDARY),
            )
            .wrap(),
        );
    }
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
            "Jetthing loads plugin manifests (a folder with a jetthing-plugin.toml) from these directories:",
        )
        .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
    for (source, dir) in plugin_dirs(project) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(source).color(theme::TEXT_INFO));
            ui.label(RichText::new(dir.display().to_string()).color(theme::TEXT));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_plugins::PluginStore;

    #[test]
    fn rows_list_discovered_and_failed_manifests() {
        let root =
            std::env::temp_dir().join(format!("jetthing-plugins-page-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let good = root.join("good");
        let bad = root.join("bad");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(
            good.join("jetthing-plugin.toml"),
            "id = \"dev.example.good\"\nname = \"Good\"\nversion = \"1.0.0\"\napi_version = 1\ncapabilities = [\"language\"]\n",
        )
        .unwrap();
        std::fs::write(bad.join("jetthing-plugin.toml"), "id = ").unwrap();
        let mut registry = PluginRegistry::new(PluginStore::in_memory());
        registry.discover(&root);
        let rows = rows(&registry);
        let good_row = rows
            .iter()
            .find(|row| row.id.as_deref() == Some("dev.example.good"))
            .expect("good plugin listed");
        assert_eq!(good_row.name, "Good");
        assert!(good_row.enabled && good_row.can_toggle);
        assert!(matches(good_row, "lang"));
        let failed = rows
            .iter()
            .find(|row| row.id.is_none())
            .expect("failed row");
        assert_eq!(failed.name, "bad");
        assert!(failed.error.is_some());
        let _ = std::fs::remove_dir_all(root);
    }
}
