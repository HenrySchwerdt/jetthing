//! Logcat tool window: device chooser, filter query, level-coloured log rows
//! and the clear / pause / restart / scroll-to-end / soft-wrap actions.

use crate::{IdeApp, icons, theme, widgets};
use eframe::egui::{self, Color32, RichText, vec2};
use icons::Icon;
use jetthing_android::{LogEntry, LogLevel};
use jetthing_core::ToolWindow;
use serde_json::{Value, json};

/// Rows rendered in soft-wrap mode, where rows have variable height.
const SOFT_WRAP_ROWS: usize = 5_000;

/// UI-only Logcat state.
#[derive(Debug)]
pub struct LogcatView {
    query: String,
    synced: bool,
    scroll_to_end: bool,
}

impl Default for LogcatView {
    fn default() -> Self {
        Self {
            query: String::new(),
            synced: false,
            scroll_to_end: true,
        }
    }
}

pub fn level_colors(level: LogLevel) -> (Color32, Color32) {
    // (badge background, message text)
    match level {
        LogLevel::Verbose => (Color32::from_rgb(0x4E, 0x51, 0x57), *theme::TEXT_SECONDARY),
        LogLevel::Debug => (Color32::from_rgb(0x30, 0x5D, 0x78), *theme::TEXT),
        LogLevel::Info => (Color32::from_rgb(0x3F, 0x6E, 0x4A), *theme::TEXT),
        LogLevel::Warn => (Color32::from_rgb(0x8A, 0x6A, 0x1F), *theme::WARNING),
        LogLevel::Error => (Color32::from_rgb(0x9C, 0x3B, 0x40), *theme::ERROR),
        LogLevel::Assert => (Color32::from_rgb(0x74, 0x45, 0x91), *theme::ERROR),
    }
}

/// Stable per-tag colour so the same tag is easy to follow.
pub fn tag_color(tag: &str) -> Color32 {
    const PALETTE: [Color32; 6] = [
        Color32::from_rgb(0x56, 0xA8, 0xF5),
        Color32::from_rgb(0x6A, 0xAB, 0x73),
        Color32::from_rgb(0xC7, 0x7D, 0xBB),
        Color32::from_rgb(0x2A, 0xAC, 0xB8),
        Color32::from_rgb(0xCF, 0x8E, 0x6D),
        Color32::from_rgb(0xB3, 0xAE, 0x60),
    ];
    let hash = tag
        .bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
    PALETTE[hash as usize % PALETTE.len()]
}

fn device_label(app: &IdeApp, serial: &str) -> String {
    let hub = &app.state.android.devices;
    match hub.devices.iter().find(|d| d.serial == serial) {
        Some(device) => {
            let mut label = format!("{} ({serial})", device.display_name());
            if let Some(api) = device.api_level() {
                label.push_str(&format!("  API {api}"));
            }
            if !device.is_online() {
                label.push_str(&format!(" [{}]", device.state.label()));
            }
            label
        }
        None => format!("{serial} [disconnected]"),
    }
}

pub fn show(app: &mut IdeApp, ui: &mut egui::Ui) {
    if !app.logcat.synced {
        app.logcat.query = app.state.android.logcat.query.clone();
        app.logcat.synced = true;
    }
    let hide = widgets::tool_window_header(ui, ToolWindow::Logcat.title(), |_| {});
    if hide {
        app.toggle_tool_window(ToolWindow::Logcat);
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let current = app
            .state
            .android
            .logcat
            .serial
            .clone()
            .map(|serial| device_label(app, &serial))
            .unwrap_or_else(|| "No connected devices".into());
        let mut choice = None;
        egui::ComboBox::from_id_salt("logcat_device")
            .width(360.0)
            .selected_text(current)
            .show_ui(ui, |ui| {
                for device in &app.state.android.devices.devices {
                    let selected =
                        app.state.android.logcat.serial.as_deref() == Some(device.serial.as_str());
                    if ui
                        .selectable_label(selected, device_label(app, &device.serial))
                        .clicked()
                    {
                        choice = Some(device.serial.clone());
                    }
                }
            });
        if let Some(serial) = choice {
            app.dispatch("android.logcat.start", json!({"serial": serial}));
        }
        ui.add_space(8.0);
        let has_error = app.state.android.logcat.filter_error.is_some();
        let field_width = (ui.available_width() - 70.0).max(120.0);
        let response = ui.add(
            egui::TextEdit::singleline(&mut app.logcat.query)
                .id(egui::Id::new("logcat_filter"))
                .hint_text("Filter, e.g. package:mine level:warn tag:MyTag")
                .font(egui::FontId::monospace(theme::EDITOR_FONT_SIZE))
                .text_color(if has_error {
                    *theme::ERROR
                } else {
                    *theme::TEXT_BRIGHT
                })
                .desired_width(field_width),
        );
        let response = match &app.state.android.logcat.filter_error {
            Some(error) => response.on_hover_text(error),
            None => response.on_hover_text(
                "key:value contains, key=:value exact, key~:regex, -key: excludes. \
                 Keys: tag, message, package (package:mine), level.",
            ),
        };
        if response.changed() {
            let query = app.logcat.query.clone();
            // Invalid queries keep the last valid filter; the error is shown.
            let _ = app.commands.execute(
                "android.logcat.filter",
                &mut app.state,
                json!({"query": query}),
            );
        }
        let match_case = app.state.android.logcat.match_case;
        if ui
            .add(
                egui::Button::new("Cc")
                    .selected(match_case)
                    .min_size(vec2(28.0, 22.0)),
            )
            .on_hover_text("Match case")
            .clicked()
        {
            let query = app.logcat.query.clone();
            let _ = app.commands.execute(
                "android.logcat.filter",
                &mut app.state,
                json!({"query": query, "matchCase": !match_case}),
            );
        }
    });
    ui.add_space(4.0);
    let area = ui.available_rect_before_wrap();
    let toolbar_rect = egui::Rect::from_min_size(area.min, vec2(36.0, area.height()));
    let rows_rect = egui::Rect::from_min_max(egui::pos2(area.left() + 40.0, area.top()), area.max);
    ui.painter().vline(
        toolbar_rect.right() + 1.0,
        area.y_range(),
        egui::Stroke::new(1.0_f32, theme::BORDER),
    );
    let mut toolbar = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(toolbar_rect.shrink2(vec2(4.0, 0.0)))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    {
        let ui = &mut toolbar;
        ui.spacing_mut().item_spacing.y = 4.0;
        {
            if icons::button(ui, Icon::Close, "Clear Logcat").clicked() {
                app.dispatch("android.logcat.clear", Value::Null);
            }
            let paused = app.state.android.logcat.paused;
            let (icon, tip) = if paused {
                (Icon::Run, "Resume Logcat")
            } else {
                (Icon::Pause, "Pause Logcat")
            };
            if icons::button_colored(ui, icon, *icons::ICON, tip, paused).clicked() {
                app.dispatch("android.logcat.pause", Value::Null);
            }
            if icons::button(ui, Icon::Refresh, "Restart Logcat").clicked() {
                let serial = app.state.android.logcat.serial.clone();
                app.dispatch("android.logcat.start", json!({"serial": serial}));
            }
            if icons::button_colored(
                ui,
                Icon::ScrollEnd,
                *icons::ICON,
                "Scroll to the End",
                app.logcat.scroll_to_end,
            )
            .clicked()
            {
                app.logcat.scroll_to_end = !app.logcat.scroll_to_end;
            }
            if icons::button_colored(
                ui,
                Icon::SoftWrap,
                *icons::ICON,
                "Soft-Wrap",
                app.state.android.logcat.soft_wrap,
            )
            .clicked()
            {
                app.dispatch("android.logcat.softWrap", Value::Null);
            }
        }
    }
    let mut rows = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rows_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    log_rows(app, &mut rows);
    ui.advance_cursor_after_rect(area);
}

fn log_rows(app: &mut IdeApp, ui: &mut egui::Ui) {
    let logcat = &app.state.android.logcat;
    if logcat.visible_len() == 0 {
        let message = logcat
            .error
            .clone()
            .or_else(|| {
                (logcat.serial.is_none()).then(|| {
                    "No connected devices. Connect a device or start an emulator from the Device Manager."
                        .to_owned()
                })
            })
            .unwrap_or_else(|| {
                if logcat.is_empty() {
                    "Waiting for log output…".into()
                } else {
                    format!("No lines match the filter ({} hidden)", logcat.len())
                }
            });
        ui.vertical(|ui| {
            ui.add_space(12.0);
            ui.label(RichText::new(message).color(theme::TEXT_INFO));
        });
        return;
    }
    let font = egui::FontId::monospace(theme::EDITOR_FONT_SIZE);
    let row_height = ui.fonts_mut(|f| f.row_height(&font)) + 2.0;
    let mut scroll = egui::ScrollArea::both()
        .id_salt("logcat_rows")
        .auto_shrink([false, false])
        .stick_to_bottom(app.logcat.scroll_to_end);
    if logcat.soft_wrap {
        scroll = egui::ScrollArea::vertical()
            .id_salt("logcat_rows_wrapped")
            .auto_shrink([false, false])
            .stick_to_bottom(app.logcat.scroll_to_end);
        scroll.show(ui, |ui| {
            let total = logcat.visible_len();
            let start = total.saturating_sub(SOFT_WRAP_ROWS);
            for index in start..total {
                if let Some(entry) = logcat.visible_entry(index) {
                    let job = row_job(
                        entry,
                        logcat.process_name(entry.pid),
                        &font,
                        ui.available_width(),
                    );
                    ui.label(job);
                }
            }
        });
    } else {
        scroll.show_rows(ui, row_height, logcat.visible_len(), |ui, range| {
            for index in range {
                if let Some(entry) = logcat.visible_entry(index) {
                    let job = row_job(entry, logcat.process_name(entry.pid), &font, f32::INFINITY);
                    ui.add(egui::Label::new(job).extend());
                }
            }
        });
    }
}

/// Builds one row: time, pid-tid, tag, package, level badge, message.
fn row_job(
    entry: &LogEntry,
    package: Option<&str>,
    font: &egui::FontId,
    wrap_width: f32,
) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let plain = |color| TextFormat::simple(font.clone(), color);
    job.append(&entry.timestamp, 0.0, plain(*theme::TEXT_INFO));
    job.append(
        &format!("{:>6}-{:<6}", entry.pid, entry.tid),
        12.0,
        plain(*theme::TEXT_INFO),
    );
    job.append(&pad(&entry.tag, 23), 8.0, plain(tag_color(&entry.tag)));
    job.append(
        &pad(package.unwrap_or("pid-"), 28),
        8.0,
        plain(*theme::TEXT_SECONDARY),
    );
    let (badge, text) = level_colors(entry.level);
    let mut badge_format = plain(Color32::WHITE);
    badge_format.background = badge;
    job.append(&format!(" {} ", entry.level.letter()), 8.0, badge_format);
    job.append(&entry.message, 8.0, plain(text));
    job
}

/// Pads or truncates `text` to `width` characters, like fixed log columns.
fn pad(text: &str, width: usize) -> String {
    let count = text.chars().count();
    if count > width {
        let mut short: String = text.chars().take(width - 1).collect();
        short.push('…');
        short
    } else {
        format!("{text}{}", " ".repeat(width - count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_and_colours() {
        assert_eq!(pad("abc", 5), "abc  ");
        assert_eq!(pad("abcdef", 4), "abc…");
        assert_eq!(tag_color("MainActivity"), tag_color("MainActivity"));
        assert_ne!(
            level_colors(LogLevel::Error).0,
            level_colors(LogLevel::Info).0
        );
    }
}
