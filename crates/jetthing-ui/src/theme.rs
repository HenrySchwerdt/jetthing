//! "Islands Dark" look.
//!
//! Color values are facts taken from `docs/spec/data/themes.json` and
//! `docs/spec/data/color-schemes.json` (theme `Islands Dark`, keys noted per constant).
//! Fonts are Inter and JetBrains Mono, both bundled under the SIL Open Font License.

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke};
use std::sync::Arc;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

const fn white_alpha(alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha)
}

/// `MainWindow.background`, `MainToolbar.background`, `StatusBar.background`,
/// `ToolWindow.Stripe.background`.
pub const FRAME: Color32 = hex(0x26282C);
/// `ToolWindow.background`, `EditorTabs.background`, editor `TEXT` background.
pub const ISLAND: Color32 = hex(0x191A1C);
/// `Popup.background`.
pub const POPUP: Color32 = hex(0x26282C);
/// `Popup.borderColor`, `*.separatorColor`.
pub const POPUP_BORDER: Color32 = hex(0x33353B);
/// `*.foreground`.
pub const TEXT: Color32 = hex(0xD1D3D9);
/// `Gray12`, used for emphasized labels.
pub const TEXT_BRIGHT: Color32 = hex(0xDFE1E5);
/// `StatusBar.Widget.foreground`, `ToolWindow.Button.foreground`.
pub const TEXT_SECONDARY: Color32 = hex(0x9FA2A8);
/// `*.infoForeground`.
pub const TEXT_INFO: Color32 = hex(0x73767C);
/// `*.disabledForeground`.
pub const TEXT_DISABLED: Color32 = hex(0x4C4F56);
/// `*.hoverBackground`, `ActionButton.hoverBackground`.
pub const HOVER: Color32 = white_alpha(0x17);
/// `ActionButton.pressedBackground`.
pub const PRESSED: Color32 = white_alpha(0x29);
/// `*.selectionBackground`.
pub const SELECTION: Color32 = hex(0x2A4371);
/// `ToolWindow.Button.selectedBackground`, `*.focusColor`.
pub const ACCENT: Color32 = hex(0x3871E1);
/// Disabled default button fill (palette `Gray5`).
pub const SELECTION_DISABLED: Color32 = hex(0x4E5157);
/// `Component.borderColor`.
pub const BORDER: Color32 = hex(0x40434A);
/// `TextField.background`.
pub const FIELD: Color32 = hex(0x191A1C);
/// `MainToolbar.separatorColor`.
pub const TOOLBAR_SEPARATOR: Color32 = white_alpha(0x21);
/// `RunWidget.runIconColor`.
pub const RUN_GREEN: Color32 = hex(0x4E9D6C);
/// `RunWidget.stopBackground`.
pub const STOP_RED: Color32 = hex(0xC54E58);
/// `Label.errorForeground`.
pub const ERROR: Color32 = hex(0xF57E84);
/// `Label.warningForeground`.
pub const WARNING: Color32 = hex(0xD59637);
/// `Label.successForeground`.
pub const SUCCESS: Color32 = hex(0x6DB083);
/// `*.modifiedItemForeground`.
pub const MODIFIED: Color32 = hex(0x71A1FE);
/// `VersionControl` added file color (palette `Green8`).
pub const ADDED: Color32 = hex(0x73BD79);
/// Untracked file color (palette `Red8`).
pub const UNTRACKED: Color32 = hex(0xE37774);
/// Palette `Gray7`, ignored/deleted text.
pub const MUTED: Color32 = hex(0x6F737A);
/// `CompletionPopup.selectionBackground`.
pub const COMPLETION_SELECTION: Color32 = hex(0x113A5C);

/// `Island.arc` is 20 (diameter); egui radii are half of that.
pub const ISLAND_RADIUS: u8 = 10;
/// Half of `Island.borderWidth.compact`.
pub const ISLAND_GAP: f32 = 2.0;

pub mod editor {
    //! Islands Dark editor color scheme attributes.
    use super::hex;
    use eframe::egui::Color32;

    pub const TEXT: Color32 = hex(0xBCBEC4); // TEXT.FOREGROUND
    pub const CARET: Color32 = hex(0xCED0D6); // CARET_COLOR
    pub const CARET_ROW: Color32 = hex(0x1F2024); // CARET_ROW_COLOR
    pub const LINE_NUMBER: Color32 = hex(0x4B5059); // LINE_NUMBERS_COLOR
    pub const LINE_NUMBER_CARET: Color32 = hex(0xA1A3AB); // LINE_NUMBER_ON_CARET_ROW_COLOR
    pub const SELECTION: Color32 = hex(0x214283); // SELECTION_BACKGROUND
    pub const BACKGROUND_PANEL: Color32 = hex(0x212326); // Editor.SearchField.background
    pub const SEARCH_RESULT: Color32 = hex(0x114957); // TEXT_SEARCH_RESULT_ATTRIBUTES
    pub const MATCHED_BRACE: Color32 = hex(0x43454A); // MATCHED_BRACE_ATTRIBUTES
    pub const INDENT_GUIDE: Color32 = hex(0x323438); // INDENT_GUIDE
    pub const KEYWORD: Color32 = hex(0xCF8E6D); // DEFAULT_KEYWORD
    pub const STRING: Color32 = hex(0x6AAB73); // DEFAULT_STRING
    pub const NUMBER: Color32 = hex(0x2AACB8); // DEFAULT_NUMBER
    pub const COMMENT: Color32 = hex(0x7A7E85); // DEFAULT_LINE_COMMENT
    pub const DOC_COMMENT: Color32 = hex(0x5F826B); // DEFAULT_DOC_COMMENT (italic)
    pub const FUNCTION: Color32 = hex(0x56A8F5); // DEFAULT_FUNCTION_DECLARATION
    pub const ERROR_EFFECT: Color32 = hex(0xFA6675); // ERRORS_ATTRIBUTES.EFFECT_COLOR
    pub const WARNING_EFFECT: Color32 = hex(0xF2C55C); // WARNING_ATTRIBUTES.EFFECT_COLOR
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

        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(TEXT);
        visuals.panel_fill = FRAME;
        visuals.window_fill = POPUP;
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
        visuals.extreme_bg_color = FIELD;
        visuals.faint_bg_color = HOVER;
        visuals.code_bg_color = ISLAND;
        visuals.hyperlink_color = MODIFIED;
        visuals.selection.bg_fill = SELECTION;
        visuals.selection.stroke = Stroke::new(1.0_f32, TEXT_BRIGHT);
        visuals.text_cursor.stroke = Stroke::new(1.5_f32, editor::CARET);
        visuals.error_fg_color = ERROR;
        visuals.warn_fg_color = WARNING;
        visuals.resize_corner_size = 0.0;

        let radius = CornerRadius::same(4);
        let widgets = &mut visuals.widgets;
        widgets.noninteractive.bg_fill = FRAME;
        widgets.noninteractive.weak_bg_fill = FRAME;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, POPUP_BORDER);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        widgets.noninteractive.corner_radius = radius;
        widgets.inactive.bg_fill = FIELD;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        widgets.inactive.corner_radius = radius;
        widgets.hovered.bg_fill = FIELD;
        widgets.hovered.weak_bg_fill = HOVER;
        widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER);
        widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_BRIGHT);
        widgets.hovered.corner_radius = radius;
        widgets.hovered.expansion = 0.0;
        widgets.active.bg_fill = FIELD;
        widgets.active.weak_bg_fill = PRESSED;
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
        assert_eq!(FRAME, Color32::from_rgb(0x26, 0x28, 0x2C));
    }
}
