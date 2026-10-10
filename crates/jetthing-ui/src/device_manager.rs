//! Android device selector (main toolbar) and the Device Manager tool window.

use crate::{IdeApp, icons, theme, widgets};
use eframe::egui::{self, Color32, Rect, RichText, Sense, vec2};
use icons::Icon;
use jetthing_android::api_release_name;
use jetthing_core::{Anchor, ToolWindow};
use serde_json::{Value, json};

/// Label of the toolbar device selector.
pub fn selector_label(app: &IdeApp) -> (String, bool) {
    let hub = &app.state.android.devices;
    match hub.selected_device() {
        Some(device) => (device.display_name(), true),
        None if hub.devices.is_empty() => ("No Devices".into(), false),
        None => ("No Device Selected".into(), false),
    }
}

/// Device selector shown left of the run configuration for Android projects.
pub fn device_selector(app: &mut IdeApp, ui: &mut egui::Ui) {
    if app.state.android.project.is_none() {
        return;
    }
    let (label, active) = selector_label(app);
    let icon_color = if active {
        icons::ICON
    } else {
        theme::TEXT_DISABLED
    };
    let response = widgets::flat_widget(
        ui,
        Some((Icon::Device, *icon_color)),
        &label,
        egui::FontId::proportional(theme::UI_FONT_SIZE),
        if active {
            *theme::TEXT_BRIGHT
        } else {
            *theme::TEXT_INFO
        },
        true,
        28.0,
    );
    egui::Popup::menu(&response).show(|ui| {
        ui.set_min_width(260.0);
        let hub = &app.state.android.devices;
        let mut select = None;
        let mut start = None;
        ui.label(RichText::new("Running devices").color(theme::TEXT_INFO));
        if hub.devices.is_empty() {
            ui.label(RichText::new("  None").color(theme::TEXT_DISABLED));
        }
        for device in &hub.devices {
            let selected = hub.selected.as_deref() == Some(device.serial.as_str());
            let mut text = device.display_name();
            if !device.is_online() {
                text.push_str(&format!(" [{}]", device.state.label()));
            }
            if ui.selectable_label(selected, text).clicked() {
                select = Some(device.serial.clone());
                ui.close();
            }
        }
        let available: Vec<_> = hub
            .avds
            .iter()
            .filter(|avd| hub.running_avd(&avd.name).is_none())
            .collect();
        if !available.is_empty() {
            ui.separator();
            ui.label(RichText::new("Available devices").color(theme::TEXT_INFO));
            for avd in available {
                let launching = hub.is_launching(&avd.name);
                let text = if launching {
                    format!("{} (starting…)", avd.display_name)
                } else {
                    avd.display_name.clone()
                };
                if ui
                    .add_enabled(!launching, egui::Button::new(text))
                    .on_hover_text("Start this virtual device")
                    .clicked()
                {
                    start = Some(avd.name.clone());
                    ui.close();
                }
            }
        }
        ui.separator();
        if ui.button("Device Manager").clicked() {
            app.dispatch("toolwindow.activate.deviceManager", Value::Null);
            ui.close();
        }
        if let Some(serial) = select {
            app.dispatch("android.devices.select", json!({"serial": serial}));
        }
        if let Some(avd) = start {
            app.dispatch("android.emulator.start", json!({"avd": avd}));
        }
    });
}

fn api_text(api: Option<&str>) -> String {
    api.unwrap_or("—").to_owned()
}

fn release_text(api: Option<&str>) -> Option<String> {
    api.and_then(|a| a.split(['.', '-']).next()?.parse().ok())
        .and_then(api_release_name)
}

struct Row {
    name: String,
    detail: String,
    api: String,
    kind: String,
    /// `Some(true)` running, `Some(false)` stopped, `None` physical device.
    running: Option<bool>,
    launching: bool,
    avd: Option<String>,
    status: Option<(String, Color32)>,
}

fn rows(app: &IdeApp) -> Vec<Row> {
    let hub = &app.state.android.devices;
    let mut rows: Vec<Row> = hub
        .avds
        .iter()
        .map(|avd| {
            let api = avd.api_level.as_deref();
            let mut detail = release_text(api).unwrap_or_else(|| "Unknown API".into());
            if let Some(abi) = &avd.abi {
                detail.push_str(&format!(" | {abi}"));
            }
            let running = hub.running_avd(&avd.name).is_some();
            Row {
                name: avd.display_name.clone(),
                detail,
                api: api_text(api),
                kind: avd.kind.label().into(),
                running: Some(running),
                launching: hub.is_launching(&avd.name),
                avd: Some(avd.name.clone()),
                status: None,
            }
        })
        .collect();
    for device in hub.devices.iter().filter(|d| !d.is_emulator()) {
        let api = device.api_level();
        let status = (!device.is_online()).then(|| {
            let hint = match device.state {
                jetthing_android::DeviceState::Unauthorized => {
                    "Unauthorized: accept the debugging prompt on the device"
                }
                _ => device.state.label(),
            };
            (hint.to_owned(), *theme::WARNING)
        });
        rows.push(Row {
            name: device.display_name(),
            detail: format!(
                "{} | {}",
                release_text(api).unwrap_or_else(|| "Physical device".into()),
                device.serial
            ),
            api: api_text(api),
            kind: "Phone".into(),
            running: None,
            launching: false,
            avd: None,
            status,
        });
    }
    // Emulators not started from a known AVD home (e.g. another AVD folder).
    for device in hub.devices.iter().filter(|d| {
        d.is_emulator()
            && !d
                .properties
                .as_ref()
                .and_then(|p| p.avd_name.as_deref())
                .is_some_and(|name| hub.avds.iter().any(|a| a.name == name))
    }) {
        rows.push(Row {
            name: device.display_name(),
            detail: format!("Emulator | {}", device.serial),
            api: api_text(device.api_level()),
            kind: "Phone".into(),
            running: Some(device.is_online()),
            launching: false,
            avd: None,
            status: None,
        });
    }
    rows
}

/// Right-anchored Device Manager tool window.
pub fn right(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(window) = app.state.tool_windows.visible(Anchor::Right) else {
        return;
    };
    if window != ToolWindow::DeviceManager {
        return;
    }
    egui::SidePanel::right("right_tool_window")
        .default_width(520.0)
        .min_width(280.0)
        .resizable(true)
        .show_separator_line(false)
        .frame(widgets::island_frame())
        .show(ctx, |ui| {
            let hide = widgets::tool_window_header(ui, window.title(), |ui| {
                if icons::button(ui, Icon::Refresh, "Refresh devices").clicked() {
                    app.dispatch("android.avd.list", Value::Null);
                    if !app.state.android.devices.is_tracking() {
                        app.dispatch("android.devices.track", Value::Null);
                    }
                }
            });
            if hide {
                app.toggle_tool_window(window);
            }
            device_table(app, ui);
        });
}

fn device_table(app: &mut IdeApp, ui: &mut egui::Ui) {
    let width = ui.available_width();
    let api_x = (width - 190.0).max(140.0);
    let type_x = api_x + 52.0;
    let actions_x = width - 44.0;
    let (header, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let painter = ui.painter();
    let text = |painter: &egui::Painter, x: f32, y: f32, label: &str, font, color| {
        painter.text(
            egui::pos2(x, y),
            egui::Align2::LEFT_CENTER,
            label,
            font,
            color,
        );
    };
    let font = || egui::FontId::proportional(theme::UI_FONT_SIZE);
    text(
        painter,
        header.left() + 46.0,
        header.center().y,
        "Name",
        font(),
        *theme::TEXT,
    );
    text(
        painter,
        header.left() + api_x,
        header.center().y,
        "API",
        font(),
        *theme::TEXT,
    );
    text(
        painter,
        header.left() + type_x,
        header.center().y,
        "Type",
        font(),
        *theme::TEXT,
    );
    painter.hline(
        header.x_range(),
        header.bottom(),
        egui::Stroke::new(1.0_f32, theme::BORDER),
    );
    if let Some(error) = &app.state.android.devices.error {
        let error = error.split_whitespace().collect::<Vec<_>>().join(" ");
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 6))
            .show(ui, |ui| {
                ui.label(RichText::new(error).color(theme::ERROR));
            });
    }
    let rows = rows(app);
    if rows.is_empty() {
        ui.add_space((ui.available_height() / 2.0 - 40.0).max(12.0));
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No devices connected.").color(theme::TEXT_INFO));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Create a virtual device with avdmanager create avd, or connect a device with USB debugging enabled.")
                    .color(theme::TEXT_INFO),
            );
        });
        return;
    }
    let mut action: Option<(&str, String)> = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &rows {
                let (rect, response) = ui.allocate_exact_size(vec2(width, 44.0), Sense::hover());
                let painter = ui.painter();
                if response.hovered() {
                    painter.rect_filled(rect.shrink2(vec2(4.0, 0.0)), 4.0, theme::HOVER);
                }
                let icon_color = match row.running {
                    Some(true) => theme::SUCCESS,
                    _ => icons::ICON,
                };
                icons::paint(
                    painter,
                    Rect::from_center_size(
                        egui::pos2(rect.left() + 26.0, rect.center().y),
                        vec2(16.0, 16.0),
                    ),
                    Icon::Device,
                    *icon_color,
                );
                text(
                    painter,
                    rect.left() + 46.0,
                    rect.top() + 14.0,
                    &row.name,
                    theme::semibold(theme::UI_FONT_SIZE),
                    *theme::TEXT_BRIGHT,
                );
                let (detail, detail_color) = row
                    .status
                    .clone()
                    .unwrap_or((row.detail.clone(), *theme::TEXT_INFO));
                text(
                    painter,
                    rect.left() + 46.0,
                    rect.top() + 31.0,
                    &detail,
                    egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0),
                    detail_color,
                );
                text(
                    painter,
                    rect.left() + api_x,
                    rect.center().y,
                    &row.api,
                    font(),
                    *theme::TEXT,
                );
                text(
                    painter,
                    rect.left() + type_x,
                    rect.center().y,
                    &row.kind,
                    font(),
                    *theme::TEXT,
                );
                let Some(avd) = &row.avd else { continue };
                let button_rect = Rect::from_center_size(
                    egui::pos2(rect.left() + actions_x + 12.0, rect.center().y),
                    vec2(26.0, 26.0),
                );
                let button = ui.interact(
                    button_rect,
                    ui.id().with(("avd_action", avd)),
                    Sense::click(),
                );
                let (icon, color, tooltip) = match (row.running, row.launching) {
                    (Some(true), _) => (Icon::Stop, theme::STOP_RED, "Stop"),
                    (_, true) => (Icon::Run, theme::TEXT_DISABLED, "Starting…"),
                    _ => (Icon::Run, theme::RUN_GREEN, "Start"),
                };
                if button.hovered() {
                    ui.painter().rect_filled(button_rect, 6.0, theme::HOVER);
                }
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(button_rect.center(), vec2(14.0, 14.0)),
                    icon,
                    *color,
                );
                if button.on_hover_text(tooltip).clicked() && !row.launching {
                    let id = if row.running == Some(true) {
                        "android.emulator.stop"
                    } else {
                        "android.emulator.start"
                    };
                    action = Some((id, avd.clone()));
                }
            }
        });
    if let Some((id, avd)) = action {
        app.dispatch(id, json!({"avd": avd}));
    }
}
