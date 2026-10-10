//! "Islands" look: color tokens, fonts and egui visuals.
//!
//! Color tokens resolve against the active theme (Islands Dark or Islands Light) from
//! `docs/spec/data/themes.json` and `docs/spec/data/color-schemes.json`; see
//! `theme_engine` for the keys behind each token.
//! Fonts are Inter and JetBrains Mono, both bundled under the SIL Open Font License.

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke};
use std::sync::Arc;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

// Workbench color tokens; values come from the active theme (see `theme_engine`).
pub use crate::theme_engine::Token;
pub use crate::theme_engine::tokens::{
    ACCENT, ADDED, BORDER, COMPLETION_SELECTION, ERROR, FIELD, FOLDER_FILL, FRAME, HOVER, ICON,
    ISLAND, MODIFIED, MUTED, NOTIFICATION, POPUP, POPUP_BORDER, PRESSED, RUN_GREEN, SELECTION,
    SELECTION_DISABLED, STOP_RED, SUCCESS, TAB_SELECTED, TAB_SELECTED_BORDER, TEXT, TEXT_BRIGHT,
    TEXT_DISABLED, TEXT_INFO, TEXT_SECONDARY, TOOLBAR_SEPARATOR, UNTRACKED, WARNING,
};

/// `Island.arc` is 20 (diameter); egui radii are half of that.
pub const ISLAND_RADIUS: u8 = 10;
/// Half of `Island.borderWidth.compact`.
pub const ISLAND_GAP: f32 = 2.0;

pub mod editor {
    //! Editor color scheme attributes of the active theme. Syntax colors are
    //! resolved from the same scheme by text-attribute key (see `crate::syntax`).
    use crate::theme_engine::Token;
    use crate::theme_engine::tokens as t;

    pub const BACKGROUND: Token = t::EDITOR_BACKGROUND;
    pub const TEXT: Token = t::EDITOR_TEXT;
    pub const CARET: Token = t::EDITOR_CARET;
    pub const CARET_ROW: Token = t::EDITOR_CARET_ROW;
    pub const LINE_NUMBER: Token = t::EDITOR_LINE_NUMBER;
    pub const LINE_NUMBER_CARET: Token = t::EDITOR_LINE_NUMBER_CARET;
    pub const SELECTION: Token = t::EDITOR_SELECTION;
    pub const BACKGROUND_PANEL: Token = t::EDITOR_BACKGROUND_PANEL;
    pub const SEARCH_RESULT: Token = t::EDITOR_SEARCH_RESULT;
    pub const MATCHED_BRACE: Token = t::EDITOR_MATCHED_BRACE;
    pub const INDENT_GUIDE: Token = t::EDITOR_INDENT_GUIDE;
    pub const ERROR_EFFECT: Token = t::EDITOR_ERROR_EFFECT;
    pub const WARNING_EFFECT: Token = t::EDITOR_WARNING_EFFECT;
}

/// Avatar gradient start colors (`RecentProject.ColorN.Avatar.Start`).
const AVATAR_COLORS: [Color32; 9] = [
    hex(0xE08855),
    hex(0xB08B14),
    hex(0xA1A359),
    hex(0x3B92B8),
    hex(0x3574F0),
    hex(0xC84D8F),
    hex(0x955AE0),
    hex(0x24A394),
    hex(0x5FAD65),
];

/// Stable avatar color for a project name, like the project widget's badge.
pub fn avatar_color(name: &str) -> Color32 {
    let hash = name.bytes().fold(0u32, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u32)
    });
    AVATAR_COLORS[hash as usize % AVATAR_COLORS.len()]
}

/// Project badge initials: first letters of the first two words, else the first two letters.
pub fn project_initials(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let initials: String = match words.as_slice() {
        [] => "?".into(),
        [single] => single.chars().take(2).collect(),
        [first, second, ..] => first
            .chars()
            .take(1)
            .chain(second.chars().take(1))
            .collect(),
    };
    initials.to_uppercase()
}

pub const UI_FONT_SIZE: f32 = 13.0;
pub const EDITOR_FONT_SIZE: f32 = 13.0;
pub const SEMIBOLD: &str = "semibold";

pub fn semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Editor font: the Monospace text style, sized by Settings | Editor | Font.
pub fn editor_font(style: &egui::Style) -> egui::FontId {
    egui::TextStyle::Monospace.resolve(style)
}

fn line_height_id() -> egui::Id {
    egui::Id::new("editor_line_height")
}

/// Editor row height for `font`: `base` at the default 13 pt / 1.2 line height,
/// scaled by the configured size and line height.
pub fn editor_row_height(ctx: &egui::Context, font: &egui::FontId, base: f32) -> f32 {
    let line_height = ctx
        .data(|data| data.get_temp::<f32>(line_height_id()))
        .unwrap_or(DEFAULT_LINE_HEIGHT);
    (base * font.size / EDITOR_FONT_SIZE * line_height / DEFAULT_LINE_HEIGHT)
        .round()
        .max(8.0)
}

const DEFAULT_LINE_HEIGHT: f32 = 1.2;

/// Applies settings that change the look live: theme, editor color scheme, editor
/// font size and line height.
pub fn apply_settings(ctx: &egui::Context, settings: &jetthing_core::Settings) {
    use jetthing_settings::ids;
    let theme = settings
        .text(ids::THEME)
        .unwrap_or_else(|| crate::theme_engine::THEMES[0].into());
    let scheme = settings.text(ids::EDITOR_SCHEME).filter(|s| !s.is_empty());
    // Settings validate the names; an error leaves the previous theme active.
    if let Ok(true) = crate::theme_engine::set_active(&theme, scheme.as_deref()) {
        apply_visuals(ctx);
    }
    let line_height = settings
        .float(jetthing_settings::ids::EDITOR_LINE_HEIGHT)
        .map(|value| value as f32)
        .unwrap_or(DEFAULT_LINE_HEIGHT);
    ctx.data_mut(|data| data.insert_temp(line_height_id(), line_height));
    let size = settings
        .int(jetthing_settings::ids::EDITOR_FONT_SIZE)
        .map(|size| size as f32)
        .unwrap_or(EDITOR_FONT_SIZE);
    let current = ctx
        .style()
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .map(|font| font.size);
    if current != Some(size) {
        ctx.style_mut(|style| {
            style
                .text_styles
                .insert(egui::TextStyle::Monospace, egui::FontId::monospace(size));
        });
    }
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-semibold".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "jetbrains-mono".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    // Keep egui's bundled fonts as fallbacks for symbols such as ⌘ and ⇧.
    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "jetbrains-mono".into());
    let mut semibold_family = vec!["inter-semibold".to_owned()];
    semibold_family.extend(fallbacks);
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), semibold_family);
    ctx.set_fonts(fonts);

    ctx.style_mut(|style| {
        use egui::{FontId, TextStyle};
        style.text_styles = [
            (TextStyle::Small, FontId::proportional(UI_FONT_SIZE - 2.0)),
            (TextStyle::Body, FontId::proportional(UI_FONT_SIZE)),
            (TextStyle::Button, FontId::proportional(UI_FONT_SIZE)),
            (TextStyle::Heading, semibold(UI_FONT_SIZE)),
            (TextStyle::Monospace, FontId::monospace(EDITOR_FONT_SIZE)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 3.0);
        style.spacing.interact_size.y = 24.0;
        style.spacing.scroll = egui::style::ScrollStyle::floating();
        style.interaction.selectable_labels = false;
    });
    apply_visuals(ctx);
}

/// Rebuilds egui's visuals from the active theme's tokens.
fn apply_visuals(ctx: &egui::Context) {
    let dark = crate::theme_engine::active().dark;
    ctx.style_mut(|style| {
        let visuals = &mut style.visuals;
        *visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.override_text_color = Some(*TEXT);
        visuals.panel_fill = *FRAME;
        visuals.window_fill = *POPUP;
        visuals.window_stroke = Stroke::new(1.0_f32, POPUP_BORDER);
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.menu_corner_radius = CornerRadius::same(8);
        visuals.window_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(90),
        };
        visuals.popup_shadow = visuals.window_shadow;
        visuals.extreme_bg_color = *FIELD;
        visuals.faint_bg_color = *HOVER;
        visuals.code_bg_color = *ISLAND;
        visuals.hyperlink_color = *MODIFIED;
        visuals.selection.bg_fill = *SELECTION;
        visuals.selection.stroke = Stroke::new(1.0_f32, TEXT_BRIGHT);
        visuals.text_cursor.stroke = Stroke::new(1.5_f32, editor::CARET);
        visuals.error_fg_color = *ERROR;
        visuals.warn_fg_color = *WARNING;
        visuals.resize_corner_size = 0.0;

        let radius = CornerRadius::same(4);
        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = *FRAME;
        widgets.noninteractive.weak_bg_fill = *FRAME;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, POPUP_BORDER);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        widgets.noninteractive.corner_radius = radius;
        widgets.inactive.bg_fill = *FIELD;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        widgets.inactive.corner_radius = radius;
        widgets.hovered.bg_fill = *FIELD;
        widgets.hovered.weak_bg_fill = *HOVER;
        widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER);
        widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_BRIGHT);
        widgets.hovered.corner_radius = radius;
        widgets.hovered.expansion = 0.0;
        widgets.active.bg_fill = *FIELD;
        widgets.active.weak_bg_fill = *PRESSED;
        widgets.active.bg_stroke = Stroke::new(2.0_f32, ACCENT);
        widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT_BRIGHT);
        widgets.active.corner_radius = radius;
        widgets.active.expansion = 0.0;
        widgets.open = widgets.active;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_follow_project_widget_rules() {
        assert_eq!(project_initials("jetthing"), "JE");
        assert_eq!(project_initials("my-cool_app"), "MC");
        assert_eq!(project_initials("---"), "?");
        assert_eq!(project_initials("ärger"), "ÄR");
    }

    #[test]
    fn avatar_color_is_stable() {
        assert_eq!(avatar_color("jetthing"), avatar_color("jetthing"));
    }

    #[test]
    fn hex_decodes_channels() {
        assert_eq!(hex(0x26282C), Color32::from_rgb(0x26, 0x28, 0x2C));
    }
}
