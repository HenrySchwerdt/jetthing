//! Small reusable pieces of the workbench chrome.

use crate::icons::{self, Icon};
use crate::theme;
use eframe::egui::{self, Color32, Rect, Sense, vec2};

/// Rounded "island" container used for the editor and docked tool windows.
pub fn island_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(*theme::ISLAND)
        .corner_radius(theme::ISLAND_RADIUS)
        .outer_margin(egui::Margin::same(theme::ISLAND_GAP as i8))
}

/// Flat frame for chrome areas (toolbar, stripes, status bar).
pub fn chrome_frame() -> egui::Frame {
    egui::Frame::new().fill(*theme::FRAME)
}

pub const TOOL_WINDOW_HEADER_HEIGHT: f32 = 31.0;
pub const ROW_HEIGHT: f32 = 24.0;

/// Tool window title bar. `actions` adds right-aligned icon buttons; returns
/// `true` when the hide button was clicked.
pub fn tool_window_header(
    ui: &mut egui::Ui,
    title: &str,
    actions: impl FnOnce(&mut egui::Ui),
) -> bool {
    let mut hide = false;
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), TOOL_WINDOW_HEADER_HEIGHT),
        Sense::hover(),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(12.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.label(
        egui::RichText::new(title)
            .font(theme::semibold(theme::UI_FONT_SIZE))
            .color(theme::TEXT_BRIGHT),
    );
    child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if icons::button(ui, Icon::Minimize, "Hide").clicked() {
            hide = true;
        }
        actions(ui);
    });
    hide
}

/// Full-width list row with hover/selection painting.
pub fn list_row(ui: &mut egui::Ui, selected: bool) -> (Rect, egui::Response) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    let fill = if selected {
        *theme::SELECTION
    } else if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect.shrink2(vec2(4.0, 0.0)), 4.0, fill);
    }
    (rect, response)
}

/// Paints left-aligned text inside a row starting at `x`, returns the end x.
pub fn row_text(
    ui: &egui::Ui,
    rect: Rect,
    x: f32,
    text: &str,
    font: egui::FontId,
    color: impl Into<Color32>,
) -> f32 {
    let color = color.into();
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let width = galley.size().x;
    let pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
    ui.painter().with_clip_rect(rect).galley(pos, galley, color);
    x + width
}

/// Hoverable text widget for the status bar and toolbar dropdowns.
pub fn flat_widget(
    ui: &mut egui::Ui,
    icon: Option<(Icon, Color32)>,
    text: &str,
    font: egui::FontId,
    color: impl Into<Color32>,
    chevron: bool,
    height: f32,
) -> egui::Response {
    let color = color.into();
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let icon_width = if icon.is_some() { 22.0 } else { 0.0 };
    let chevron_width = if chevron { 18.0 } else { 0.0 };
    let width = 8.0 + icon_width + galley.size().x + chevron_width + 8.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let fill = if response.is_pointer_button_down_on() {
        *theme::PRESSED
    } else if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    let mut x = rect.left() + 8.0;
    if let Some((icon, icon_color)) = icon {
        icons::paint(
            ui.painter(),
            Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
            icon,
            icon_color,
        );
        x += icon_width;
    }
    let text_pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
    x += galley.size().x;
    ui.painter().galley(text_pos, galley, color);
    if chevron {
        icons::paint(
            ui.painter(),
            Rect::from_min_size(egui::pos2(x + 2.0, rect.center().y - 7.0), vec2(14.0, 14.0)),
            Icon::ChevronDown,
            *theme::TEXT_SECONDARY,
        );
    }
    response
}

/// Single-line search/input field in the reference look.
pub fn search_field(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    let frame = egui::Frame::new()
        .fill(*theme::FIELD)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 2));
    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::show(ui, Icon::Search, 14.0, *theme::TEXT_SECONDARY);
                ui.add(
                    egui::TextEdit::singleline(text)
                        .id(id)
                        .frame(false)
                        .desired_width(width)
                        .hint_text(hint),
                )
            })
            .inner
        })
        .inner
}

pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pluralizes() {
        assert_eq!(pluralize(1, "problem", "problems"), "1 problem");
        assert_eq!(pluralize(3, "problem", "problems"), "3 problems");
    }
}
