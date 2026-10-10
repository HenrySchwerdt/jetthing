//! Languages & Frameworks › Android SDK page (`android.sdk.manager`).
//!
//! Lists the installed SDK packages on "SDK Platforms" and "SDK Tools" tabs and
//! lets the user change the SDK location. Installing or removing packages from
//! the remote repository is not implemented; use `sdkmanager` for that.

use crate::{IdeApp, theme};
use eframe::egui::{self, RichText, vec2};
use jetthing_android::{AndroidSdk, PackageCategory, SdkPackage};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Platforms,
    Tools,
}

/// View state of the page; the SDK itself lives in `IdeState::android`.
#[derive(Debug, Default)]
pub struct SdkPage {
    tab: Tab,
    /// Location being edited; `None` while the field shows the detected SDK.
    location_edit: Option<String>,
    error: Option<String>,
    show_details: bool,
    show_obsolete: bool,
}

/// One table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub api_level: String,
    pub revision: String,
    pub detail: Option<String>,
}

fn row(package: &SdkPackage, name: String, details: bool) -> Row {
    Row {
        name,
        api_level: package.api_level.clone().unwrap_or_default(),
        revision: package.revision.clone(),
        detail: details.then(|| package.path.clone()),
    }
}

/// Rows of the "SDK Platforms" tab: one per platform, plus system images and
/// sources when package details are shown.
pub fn platform_rows(sdk: &AndroidSdk, details: bool, obsolete: bool) -> Vec<Row> {
    sdk.platforms()
        .filter(|p| obsolete || !p.obsolete)
        .filter(|p| details || p.category == PackageCategory::Platform)
        .map(|p| {
            let name = if details {
                p.display_name.clone()
            } else {
                p.platform_title()
            };
            row(p, name, details)
        })
        .collect()
}

/// Rows of the "SDK Tools" tab: the newest package of each kind, or every
/// installed version when package details are shown.
pub fn tool_rows(sdk: &AndroidSdk, details: bool, obsolete: bool) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for package in sdk.tools().filter(|p| obsolete || !p.obsolete) {
        if details {
            rows.push(row(package, package.display_name.clone(), true));
            continue;
        }
        // Group versioned packages (`build-tools;35.0.0`) under their kind.
        let kind = package
            .path
            .split(';')
            .next()
            .unwrap_or_default()
            .to_owned();
        if seen.contains(&kind) {
            continue;
        }
        seen.push(kind);
        let name = match package.category {
            PackageCategory::BuildTools => "Android SDK Build-Tools".to_owned(),
            PackageCategory::CommandLineTools => "Android SDK Command-line Tools".to_owned(),
            PackageCategory::Ndk => "NDK (Side by side)".to_owned(),
            PackageCategory::CMake => "CMake".to_owned(),
            _ => package.display_name.clone(),
        };
        rows.push(row(package, name, false));
    }
    rows
}

const NAME_X: f32 = 32.0;

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    if !app.state.android.sdk_manager_open {
        return;
    }
    let mut close = false;
    let mut apply = false;
    let screen = ctx.content_rect();
    let size = vec2(
        (screen.width() - 80.0).clamp(480.0, 980.0),
        (screen.height() - 120.0).clamp(360.0, 700.0),
    );
    let page = &mut app.android_sdk;
    egui::Area::new(egui::Id::new("android_sdk_page"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.center() - size / 2.0)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(10)
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.set_width(size.x - 36.0);
                    ui.set_height(size.y - 36.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Languages & Frameworks")
                                .font(theme::semibold(theme::UI_FONT_SIZE)),
                        );
                        ui.label(RichText::new("›").color(theme::TEXT_SECONDARY));
                        ui.label(
                            RichText::new("Android SDK")
                                .font(theme::semibold(theme::UI_FONT_SIZE))
                                .color(theme::TEXT_BRIGHT),
                        );
                    });
                    ui.add_space(14.0);
                    ui.label("Manager for the Android SDK and Tools used by the IDE");
                    ui.add_space(8.0);
                    let detection = app.state.android.sdk.as_ref();
                    let sdk = detection.and_then(|d| d.sdk.as_ref());
                    ui.horizontal(|ui| {
                        ui.label("Android SDK Location:");
                        let field_width = (ui.available_width() - 60.0).max(120.0);
                        match &mut page.location_edit {
                            Some(text) => {
                                let response = ui.add(
                                    egui::TextEdit::singleline(text)
                                        .id(egui::Id::new("android_sdk_location"))
                                        .desired_width(field_width),
                                );
                                if response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    apply = true;
                                }
                            }
                            None => {
                                let mut shown = sdk
                                    .map(|sdk| sdk.root.display().to_string())
                                    .unwrap_or_default();
                                ui.add_enabled(
                                    false,
                                    egui::TextEdit::singleline(&mut shown)
                                        .hint_text("No SDK found")
                                        .desired_width(field_width),
                                );
                                if ui
                                    .link(RichText::new("Edit").color(theme::MODIFIED))
                                    .clicked()
                                {
                                    page.location_edit = Some(shown);
                                    page.error = None;
                                }
                            }
                        }
                    });
                    if let Some(sdk) = sdk {
                        ui.label(
                            RichText::new(format!("Found via {}", sdk.source.label()))
                                .color(theme::TEXT_INFO),
                        );
                    }
                    if let Some(error) = &page.error {
                        ui.label(RichText::new(error).color(theme::ERROR));
                    } else if let Some(problem) = detection.and_then(|d| d.problem()) {
                        ui.label(RichText::new(problem).color(theme::ERROR));
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        for (tab, label) in [(Tab::Platforms, "SDK Platforms"), (Tab::Tools, "SDK Tools")] {
                            let selected = page.tab == tab;
                            let button = egui::Button::new(label)
                                .fill(if selected { theme::SELECTION } else { egui::Color32::TRANSPARENT })
                                .stroke(egui::Stroke::new(
                                     1.0_f32,
                                    if selected { theme::ACCENT } else { egui::Color32::TRANSPARENT },
                                ))
                                .min_size(vec2(0.0, 26.0));
                            if ui.add(button).clicked() {
                                page.tab = tab;
                            }
                        }
                    });
                    ui.add_space(6.0);
                    let description = match page.tab {
                        Tab::Platforms => {
                            "Each Android SDK Platform package includes the Android platform and sources pertaining to an API level by default. Check \"Show package details\" to display individual SDK components."
                        }
                        Tab::Tools => {
                            "Below are the installed SDK developer tools. Check \"Show package details\" to display individual versions. Install or update packages with sdkmanager."
                        }
                    };
                    ui.label(RichText::new(description).color(theme::TEXT_SECONDARY));
                    ui.add_space(8.0);
                    let footer_height = 76.0;
                    let table_height = (ui.available_height() - footer_height).max(80.0);
                    let rows = match (sdk, page.tab) {
                        (Some(sdk), Tab::Platforms) => {
                            platform_rows(sdk, page.show_details, page.show_obsolete)
                        }
                        (Some(sdk), Tab::Tools) => {
                            tool_rows(sdk, page.show_details, page.show_obsolete)
                        }
                        (None, _) => Vec::new(),
                    };
                    table(ui, page.tab, &rows, table_height);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut page.show_details, "Show package details");
                            ui.add_space(12.0);
                            ui.checkbox(&mut page.show_obsolete, "Show obsolete packages");
                        });
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Packages are installed with sdkmanager; this page lists what is installed.")
                                .color(theme::TEXT_INFO),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(page.location_edit.is_some(), egui::Button::new("Apply"))
                                .clicked()
                            {
                                apply = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close = true;
                            }
                            let ok = egui::Button::new(RichText::new("OK").color(theme::TEXT_BRIGHT))
                                .fill(theme::ACCENT)
                                .min_size(vec2(72.0, 0.0));
                            if ui.add(ok).clicked() {
                                apply = page.location_edit.is_some();
                                close = true;
                            }
                        });
                    });
                });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
        apply = false;
    }
    if apply && let Some(path) = app.android_sdk.location_edit.clone() {
        match app.commands.execute(
            "android.sdk.setLocation",
            &mut app.state,
            json!({"path": path}),
        ) {
            Ok(_) => {
                app.android_sdk.location_edit = None;
                app.android_sdk.error = None;
            }
            Err(error) => {
                app.android_sdk.error = Some(error.to_string());
                close = false;
            }
        }
    }
    if close {
        app.state.android.sdk_manager_open = false;
        app.android_sdk.location_edit = None;
        app.android_sdk.error = None;
    }
}

fn table(ui: &mut egui::Ui, tab: Tab, rows: &[Row], height: f32) {
    let width = ui.available_width();
    let columns: &[(&str, f32)] = match tab {
        Tab::Platforms => &[("API Level", 0.62), ("Revision", 0.76), ("Status", 0.86)],
        Tab::Tools => &[("Version", 0.70), ("Status", 0.86)],
    };
    egui::Frame::new()
        .fill(theme::ISLAND)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(4)
        .show(ui, |ui| {
            ui.set_width(width - 2.0);
            ui.set_height(height);
            let (header, _) = ui.allocate_exact_size(vec2(width - 2.0, 26.0), egui::Sense::hover());
            let painter = ui.painter();
            let text = |painter: &egui::Painter, x: f32, y: f32, label: &str, color| {
                painter.text(
                    egui::pos2(x, y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    color,
                );
            };
            text(
                painter,
                header.left() + NAME_X,
                header.center().y,
                "Name",
                theme::TEXT,
            );
            for (label, at) in columns {
                text(
                    painter,
                    header.left() + width * at,
                    header.center().y,
                    label,
                    theme::TEXT,
                );
            }
            painter.hline(
                header.x_range(),
                header.bottom(),
                egui::Stroke::new(1.0_f32, theme::BORDER),
            );
            if rows.is_empty() {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No installed packages").color(theme::TEXT_INFO));
                });
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for row in rows {
                        let row_height = if row.detail.is_some() { 38.0 } else { 24.0 };
                        let (rect, response) = ui.allocate_exact_size(
                            vec2(ui.available_width(), row_height),
                            egui::Sense::hover(),
                        );
                        let painter = ui.painter();
                        if response.hovered() {
                            painter.rect_filled(rect, 0.0, theme::HOVER);
                        }
                        let y = rect.top() + 12.0;
                        // Installed packages render as checked, read-only boxes.
                        let check = egui::Rect::from_center_size(
                            egui::pos2(rect.left() + 16.0, y),
                            vec2(13.0, 13.0),
                        );
                        painter.rect_filled(check, 3.0, theme::ACCENT);
                        painter.line_segment(
                            [
                                check.left_center() + vec2(3.0, 0.0),
                                check.center_bottom() + vec2(-1.0, -3.0),
                            ],
                            egui::Stroke::new(1.5_f32, theme::TEXT_BRIGHT),
                        );
                        painter.line_segment(
                            [
                                check.center_bottom() + vec2(-1.0, -3.0),
                                check.right_top() + vec2(-3.0, 3.0),
                            ],
                            egui::Stroke::new(1.5_f32, theme::TEXT_BRIGHT),
                        );
                        text(
                            painter,
                            rect.left() + NAME_X,
                            y,
                            &row.name,
                            theme::TEXT_BRIGHT,
                        );
                        if let Some(detail) = &row.detail {
                            text(
                                painter,
                                rect.left() + NAME_X,
                                y + 15.0,
                                detail,
                                theme::TEXT_INFO,
                            );
                        }
                        let values: Vec<&str> = match tab {
                            Tab::Platforms => vec![&row.api_level, &row.revision, "Installed"],
                            Tab::Tools => vec![&row.revision, "Installed"],
                        };
                        for ((_, at), value) in columns.iter().zip(values) {
                            text(painter, rect.left() + width * at, y, value, theme::TEXT);
                        }
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_android::SdkSource;
    use std::path::PathBuf;

    fn package(path: &str, name: &str, rev: &str, api: Option<&str>) -> SdkPackage {
        SdkPackage {
            path: path.into(),
            display_name: name.into(),
            revision: rev.into(),
            category: match path.split(';').next().unwrap() {
                "platforms" => PackageCategory::Platform,
                "system-images" => PackageCategory::SystemImage,
                "build-tools" => PackageCategory::BuildTools,
                _ => PackageCategory::PlatformTools,
            },
            api_level: api.map(Into::into),
            obsolete: false,
            location: PathBuf::from(path.replace(';', "/")),
        }
    }

    fn sdk() -> AndroidSdk {
        AndroidSdk {
            root: PathBuf::from("/sdk"),
            source: SdkSource::AndroidHome,
            packages: vec![
                package(
                    "platforms;android-35",
                    "Android SDK Platform 35",
                    "2",
                    Some("35"),
                ),
                package(
                    "system-images;android-35;google_apis;x86_64",
                    "Google APIs Intel x86_64 Atom System Image",
                    "9",
                    Some("35"),
                ),
                package(
                    "build-tools;35.0.0",
                    "Android SDK Build-Tools 35",
                    "35.0.0",
                    None,
                ),
                package(
                    "build-tools;34.0.0",
                    "Android SDK Build-Tools 34",
                    "34.0.0",
                    None,
                ),
                package(
                    "platform-tools",
                    "Android SDK Platform-Tools",
                    "37.0.1",
                    None,
                ),
            ],
        }
    }

    #[test]
    fn platform_tab_shows_release_names_and_details_on_demand() {
        let rows = platform_rows(&sdk(), false, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Android 15.0 (\"VanillaIceCream\")");
        assert_eq!(rows[0].api_level, "35");
        assert_eq!(rows[0].revision, "2");
        let rows = platform_rows(&sdk(), true, false);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1].detail.as_deref(),
            Some("system-images;android-35;google_apis;x86_64")
        );
    }

    #[test]
    fn tools_tab_groups_versions_unless_details() {
        let rows = tool_rows(&sdk(), false, false);
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            ["Android SDK Build-Tools", "Android SDK Platform-Tools"]
        );
        assert_eq!(rows[0].revision, "35.0.0");
        assert_eq!(tool_rows(&sdk(), true, false).len(), 3);
    }
}
