//! Theme engine: resolves the workbench color tokens from the spec data.
//!
//! Every color the workbench paints is a [`Token`]. A token names, per theme, the keys
//! to look up in `docs/spec/data/themes.json` (theme `ui` map and `palette`) and
//! `docs/spec/data/color-schemes.json` (the theme's editor scheme). The data is
//! Apache-2.0 derived facts, see `docs/spec/data/NOTICE.md`.
//!
//! Tokens dereference to the active theme's color, so `theme::TEXT` can be passed to
//! egui wherever a color is expected and switching the theme takes effect on the next
//! frame without touching call sites.

use eframe::egui::Color32;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};

const THEMES_JSON: &str = include_str!("../../../docs/spec/data/themes.json");
const SCHEMES_JSON: &str = include_str!("../../../docs/spec/data/color-schemes.json");

/// A semantic workbench color, resolved against the active theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(pub(crate) usize);

impl std::ops::Deref for Token {
    type Target = Color32;

    fn deref(&self) -> &Color32 {
        &active().colors[self.0]
    }
}

impl From<Token> for Color32 {
    fn from(token: Token) -> Color32 {
        *token
    }
}

/// Lookup keys for one token: `ui:<key>`, `palette:<name>`, `color:<scheme color>`
/// or `attr:<scheme attribute>.<FIELD>`; the first key present wins.
pub(crate) struct TokenSpec {
    pub name: &'static str,
    pub dark: &'static [&'static str],
    /// Keys for light themes; empty means "same as dark".
    pub light: &'static [&'static str],
    /// Value used when no key resolves (keeps the UI usable with incomplete data).
    pub fallback: u32,
}

macro_rules! tokens {
    ($( $(#[$doc:meta])* $name:ident = [$($dark:literal),+] $([$($light:literal),+])? or $fallback:literal; )+) => {
        tokens!(@consts 0usize; $($(#[$doc])* $name;)+);
        pub(crate) const SPECS: &[TokenSpec] = &[
            $(TokenSpec {
                name: stringify!($name),
                dark: &[$($dark),+],
                light: &[$($($light),+)?],
                fallback: $fallback,
            },)+
        ];
    };
    (@consts $index:expr; $(#[$doc:meta])* $name:ident; $($rest:tt)*) => {
        $(#[$doc])*
        pub const $name: Token = Token($index);
        tokens!(@consts $index + 1usize; $($rest)*);
    };
    (@consts $index:expr;) => {};
}

/// Token constants re-exported by `theme` (workbench) and `theme::editor` (editor scheme).
pub mod tokens {
    use super::{Token, TokenSpec};

    tokens! {
        /// `MainWindow.background`, `MainToolbar.background`, `StatusBar.background`.
        FRAME = ["ui:MainWindow.background"] or 0x26282C;
        /// `ToolWindow.background`, `EditorTabs.background`.
        ISLAND = ["ui:ToolWindow.background"] or 0x191A1C;
        /// `Popup.background`.
        POPUP = ["ui:Popup.background"] or 0x26282C;
        /// `Popup.borderColor`.
        POPUP_BORDER = ["ui:Popup.borderColor"] or 0x33353B;
        /// `*.foreground`.
        TEXT = ["ui:*.foreground"] or 0xD1D3D9;
        /// Emphasized labels (`Gray12` dark, `text-default` light).
        TEXT_BRIGHT = ["palette:Gray12"] ["palette:text-default"] or 0xDFE1E5;
        /// `StatusBar.Widget.foreground`.
        TEXT_SECONDARY = ["ui:StatusBar.Widget.foreground"] or 0x9FA2A8;
        /// `*.infoForeground`.
        TEXT_INFO = ["ui:*.infoForeground"] or 0x73767C;
        /// `*.disabledForeground`.
        TEXT_DISABLED = ["ui:*.disabledForeground"] or 0x4C4F56;
        /// `ActionButton.hoverBackground`.
        HOVER = ["ui:ActionButton.hoverBackground"] or 0xFFFFFF17;
        /// `ActionButton.pressedBackground`.
        PRESSED = ["ui:ActionButton.pressedBackground"] or 0xFFFFFF29;
        /// `*.selectionBackground`.
        SELECTION = ["ui:*.selectionBackground"] or 0x2A4371;
        /// `ToolWindow.Button.selectedBackground`, `*.focusColor`.
        ACCENT = ["ui:ToolWindow.Button.selectedBackground"] or 0x3871E1;
        /// Disabled default button fill (`Gray5` dark, `control-bg-small` light).
        SELECTION_DISABLED = ["palette:Gray5"] ["palette:control-bg-small"] or 0x4E5157;
        /// `Notification.background`.
        NOTIFICATION = ["ui:Notification.background"] or 0x33353B;
        /// `Component.borderColor`.
        BORDER = ["ui:Component.borderColor"] or 0x40434A;
        /// `TextField.background`.
        FIELD = ["ui:TextField.background"] or 0x191A1C;
        /// `MainToolbar.separatorColor`.
        TOOLBAR_SEPARATOR = ["ui:MainToolbar.separatorColor"] or 0xFFFFFF21;
        /// `RunWidget.runIconColor`.
        RUN_GREEN = ["ui:RunWidget.runIconColor"] or 0x4E9D6C;
        /// `RunWidget.stopBackground`.
        STOP_RED = ["ui:RunWidget.stopBackground"] or 0xC54E58;
        /// `Label.errorForeground`.
        ERROR = ["ui:Label.errorForeground"] or 0xF57E84;
        /// `Label.warningForeground`.
        WARNING = ["ui:Label.warningForeground"] or 0xD59637;
        /// `Label.successForeground`.
        SUCCESS = ["ui:Label.successForeground"] or 0x6DB083;
        /// `*.modifiedItemForeground`.
        MODIFIED = ["ui:*.modifiedItemForeground"] or 0x71A1FE;
        /// Added file color (`FILESTATUS_ADDED`).
        ADDED = ["color:FILESTATUS_ADDED"] or 0x73BD79;
        /// Untracked file color (`Red8` dark, `FILESTATUS_UNKNOWN` light).
        UNTRACKED = ["palette:Red8"] ["color:FILESTATUS_UNKNOWN"] or 0xE37774;
        /// Ignored/deleted text (`Gray7`).
        MUTED = ["palette:Gray7"] or 0x6F737A;
        /// `CompletionPopup.selectionBackground`.
        COMPLETION_SELECTION = ["ui:CompletionPopup.selectionBackground"] or 0x113A5C;
        /// Default icon stroke (`Gray11` dark, `icon-default-stroke` light).
        ICON = ["palette:Gray11"] ["palette:icon-default-stroke"] or 0xCED0D6;
        /// Folder icon fill (`Gray4` dark, `Gray12` light).
        FOLDER_FILL = ["palette:Gray4"] ["palette:Gray12"] or 0x43454A;
        /// `EditorTabs.underlinedTabBackground` (selected editor tab).
        TAB_SELECTED = ["ui:EditorTabs.underlinedTabBackground"] or 0x233558;
        /// `EditorTabs.underlinedBorderColor`.
        TAB_SELECTED_BORDER = ["ui:EditorTabs.underlinedBorderColor"] or 0x2E4D89;
        /// `Banner.warningBackground` (editor notification banners).
        BANNER_WARNING = ["ui:Banner.warningBackground"] or 0x44321D;
        /// `Banner.warningBorderColor`.
        BANNER_WARNING_BORDER = ["ui:Banner.warningBorderColor"] or 0x694820;
        /// Editor `TEXT` background.
        EDITOR_BACKGROUND = ["attr:TEXT.BACKGROUND"] or 0x191A1C;
        /// Editor `TEXT` foreground.
        EDITOR_TEXT = ["attr:TEXT.FOREGROUND"] or 0xBCBEC4;
        /// `CARET_COLOR`.
        EDITOR_CARET = ["color:CARET_COLOR"] or 0xCED0D6;
        /// `CARET_ROW_COLOR`.
        EDITOR_CARET_ROW = ["color:CARET_ROW_COLOR"] or 0x1F2024;
        /// `LINE_NUMBERS_COLOR`.
        EDITOR_LINE_NUMBER = ["color:LINE_NUMBERS_COLOR"] or 0x4B5059;
        /// `LINE_NUMBER_ON_CARET_ROW_COLOR`.
        EDITOR_LINE_NUMBER_CARET = ["color:LINE_NUMBER_ON_CARET_ROW_COLOR", "attr:TEXT.FOREGROUND"] or 0xA1A3AB;
        /// `SELECTION_BACKGROUND`.
        EDITOR_SELECTION = ["color:SELECTION_BACKGROUND"] or 0x214283;
        /// `Editor.SearchField.background`.
        EDITOR_BACKGROUND_PANEL = ["ui:Editor.SearchField.background"] or 0x212326;
        /// `TEXT_SEARCH_RESULT_ATTRIBUTES` background.
        EDITOR_SEARCH_RESULT = ["attr:TEXT_SEARCH_RESULT_ATTRIBUTES.BACKGROUND"] or 0x114957;
        /// `MATCHED_BRACE_ATTRIBUTES` background.
        EDITOR_MATCHED_BRACE = ["attr:MATCHED_BRACE_ATTRIBUTES.BACKGROUND"] or 0x43454A;
        /// `INDENT_GUIDE`.
        EDITOR_INDENT_GUIDE = ["color:INDENT_GUIDE"] or 0x323438;
        /// `ERRORS_ATTRIBUTES` effect color.
        EDITOR_ERROR_EFFECT = ["attr:ERRORS_ATTRIBUTES.EFFECT_COLOR"] or 0xFA6675;
        /// `WARNING_ATTRIBUTES` effect (or error-stripe) color.
        EDITOR_WARNING_EFFECT = ["attr:WARNING_ATTRIBUTES.EFFECT_COLOR", "attr:WARNING_ATTRIBUTES.ERROR_STRIPE_COLOR"] or 0xF2C55C;
    }
}

/// A resolved theme.
#[derive(Debug, Clone)]
pub struct Palette {
    pub dark: bool,
    /// Editor color scheme the editor tokens (and `crate::syntax`) use.
    pub scheme: String,
    pub colors: Vec<Color32>,
    /// Token names that fell back to the built-in value (empty for complete data;
    /// asserted by the tests).
    #[cfg_attr(not(test), allow(dead_code))]
    pub missing: Vec<&'static str>,
}

#[derive(Deserialize)]
struct ThemesFile {
    themes: HashMap<String, ThemeData>,
}

#[derive(Deserialize)]
struct ThemeData {
    dark: bool,
    #[serde(rename = "editorScheme")]
    editor_scheme: Option<String>,
    #[serde(default)]
    palette: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    ui: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct SchemesFile {
    schemes: HashMap<String, SchemeData>,
}

#[derive(Deserialize)]
struct SchemeData {
    /// Scheme names from the root to this scheme; lookups fall back along it.
    #[serde(default)]
    lineage: Vec<String>,
    #[serde(default)]
    colors: HashMap<String, serde_json::Value>,
    #[serde(default)]
    attributes: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// Parses `#RRGGBB`, `#RRGGBBAA` and the scheme form without `#` whose leading zeros
/// are dropped (`80808` = `#080808`).
pub fn parse_color(text: &str) -> Option<Color32> {
    let hex = text.trim().trim_start_matches('#');
    if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = u64::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        1..=6 => Some(rgb(value as u32)),
        8 => Some(Color32::from_rgba_unmultiplied(
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        )),
        _ => None,
    }
}

fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

/// Fallback values above 24 bits carry alpha (`0xRRGGBBAA`).
fn fallback_color(value: u32) -> Color32 {
    if value > 0xFF_FFFF {
        Color32::from_rgba_unmultiplied(
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        )
    } else {
        rgb(value)
    }
}

impl Palette {
    /// The built-in Islands Dark values (used if the data cannot be read).
    pub fn builtin_dark() -> Self {
        Self {
            dark: true,
            scheme: "Islands Dark".into(),
            colors: tokens::SPECS
                .iter()
                .map(|spec| fallback_color(spec.fallback))
                .collect(),
            missing: Vec::new(),
        }
    }

    /// Resolves every token for `theme` from the spec data, with the editor colors
    /// from `scheme` (default: the theme's own editor scheme).
    pub fn from_spec(theme: &str, scheme: Option<&str>) -> Result<Self, String> {
        let themes: ThemesFile = serde_json::from_str(THEMES_JSON).map_err(|e| e.to_string())?;
        let schemes: SchemesFile = serde_json::from_str(SCHEMES_JSON).map_err(|e| e.to_string())?;
        let data = themes
            .themes
            .get(theme)
            .ok_or_else(|| format!("theme `{theme}` is not in docs/spec/data/themes.json"))?;
        let scheme_name = scheme.or(data.editor_scheme.as_deref());
        // The scheme and its parents, most specific first.
        let chain: Vec<&SchemeData> = match scheme_name {
            Some(name) => {
                let scheme = schemes.schemes.get(name).ok_or_else(|| {
                    format!("color scheme `{name}` is not in docs/spec/data/color-schemes.json")
                })?;
                let mut chain = vec![scheme];
                chain.extend(
                    scheme
                        .lineage
                        .iter()
                        .rev()
                        .filter(|parent| parent.as_str() != name)
                        .filter_map(|parent| schemes.schemes.get(parent)),
                );
                chain
            }
            None => Vec::new(),
        };
        let lookup = |key: &str| -> Option<Color32> {
            let (kind, name) = key.split_once(':')?;
            match kind {
                "ui" => {
                    let value = data.ui.get(name)?.as_str()?;
                    parse_color(value).or_else(|| {
                        // `ui` values may name a palette entry.
                        data.palette
                            .as_ref()?
                            .get(value)?
                            .as_str()
                            .and_then(parse_color)
                    })
                }
                "palette" => data
                    .palette
                    .as_ref()?
                    .get(name)?
                    .as_str()
                    .and_then(parse_color),
                "color" => chain
                    .iter()
                    .find_map(|scheme| scheme.colors.get(name)?.as_str().and_then(parse_color)),
                "attr" => {
                    let (attribute, field) = name.rsplit_once('.')?;
                    chain.iter().find_map(|scheme| {
                        scheme
                            .attributes
                            .get(attribute)?
                            .get(field)?
                            .as_str()
                            .and_then(parse_color)
                    })
                }
                _ => None,
            }
        };
        let mut missing = Vec::new();
        let colors = tokens::SPECS
            .iter()
            .map(|spec| {
                let keys = if !data.dark && !spec.light.is_empty() {
                    spec.light
                } else {
                    spec.dark
                };
                keys.iter().find_map(|key| lookup(key)).unwrap_or_else(|| {
                    missing.push(spec.name);
                    fallback_color(spec.fallback)
                })
            })
            .collect();
        Ok(Self {
            dark: data.dark,
            scheme: scheme_name.unwrap_or("Islands Dark").to_owned(),
            colors,
            missing,
        })
    }

    #[cfg(test)]
    pub fn get(&self, token: Token) -> Color32 {
        self.colors[token.0]
    }
}

/// Themes offered in Settings | Appearance, in display order.
pub const THEMES: [&str; 2] = ["Islands Dark", "Islands Light"];

/// Editor color schemes offered in Settings | Appearance (besides the theme default);
/// the settings definition must list the same names (checked by a test).
#[cfg(test)]
pub const SCHEMES: [&str; 6] = [
    "Islands Dark",
    "Light",
    "Darcula",
    "IntelliJ Light",
    "Dark",
    "High contrast",
];

type PaletteKey = (String, Option<String>);

/// Resolved palettes are built once per (theme, scheme) pair and kept for the process
/// lifetime (at most `THEMES × (SCHEMES + 1)` small tables), so tokens can hand out
/// `'static` references.
static CACHE: LazyLock<Mutex<HashMap<PaletteKey, &'static Palette>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ACTIVE: LazyLock<RwLock<&'static Palette>> =
    LazyLock::new(|| RwLock::new(palette(THEMES[0], None).unwrap_or_else(|_| builtin())));

fn builtin() -> &'static Palette {
    static BUILTIN: LazyLock<Palette> = LazyLock::new(Palette::builtin_dark);
    &BUILTIN
}

fn palette(theme: &str, scheme: Option<&str>) -> Result<&'static Palette, String> {
    let key = (theme.to_owned(), scheme.map(str::to_owned));
    let mut cache = CACHE.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(palette) = cache.get(&key) {
        return Ok(palette);
    }
    let palette: &'static Palette = Box::leak(Box::new(Palette::from_spec(theme, scheme)?));
    cache.insert(key, palette);
    Ok(palette)
}

pub fn active() -> &'static Palette {
    *ACTIVE.read().unwrap_or_else(|poison| poison.into_inner())
}

/// Activates `theme` with editor `scheme` (`None`: the theme's own scheme). Returns
/// whether the palette changed; unknown names are an error and keep the current one.
pub fn set_active(theme: &str, scheme: Option<&str>) -> Result<bool, String> {
    let next = palette(theme, scheme)?;
    let mut active = ACTIVE.write().unwrap_or_else(|poison| poison.into_inner());
    let changed = !std::ptr::eq(*active, next);
    *active = next;
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn islands_dark_from_data_matches_the_built_in_values() {
        let palette = Palette::from_spec("Islands Dark", None).unwrap();
        assert!(palette.dark);
        assert!(palette.missing.is_empty(), "missing {:?}", palette.missing);
        let builtin = Palette::builtin_dark();
        for (index, spec) in tokens::SPECS.iter().enumerate() {
            assert_eq!(
                palette.colors[index], builtin.colors[index],
                "token {} differs from the data",
                spec.name
            );
        }
    }

    #[test]
    fn islands_light_resolves_every_token() {
        let palette = Palette::from_spec("Islands Light", None).unwrap();
        assert!(!palette.dark);
        assert!(palette.missing.is_empty(), "missing {:?}", palette.missing);
        assert_eq!(
            palette.get(tokens::FRAME),
            Color32::from_rgb(0xE9, 0xEA, 0xEE)
        );
        assert_eq!(palette.get(tokens::ISLAND), Color32::WHITE);
        assert_eq!(palette.get(tokens::TEXT), Color32::BLACK);
        // Editor scheme "Light": TEXT foreground 080808.
        assert_eq!(palette.get(tokens::EDITOR_TEXT), Color32::from_rgb(8, 8, 8));
        assert_eq!(
            palette.get(tokens::HOVER),
            Color32::from_rgba_unmultiplied(0, 0, 0, 0x12)
        );
    }

    #[test]
    fn colors_parse_in_every_spec_form() {
        assert_eq!(
            parse_color("#3871E1"),
            Some(Color32::from_rgb(0x38, 0x71, 0xE1))
        );
        assert_eq!(parse_color("80808"), Some(Color32::from_rgb(8, 8, 8)));
        assert_eq!(parse_color("0"), Some(Color32::BLACK));
        assert_eq!(
            parse_color("#FFFFFF17"),
            Some(Color32::from_rgba_unmultiplied(255, 255, 255, 0x17))
        );
        assert_eq!(parse_color("panel"), None);
        assert_eq!(parse_color(""), None);
        assert_eq!(
            parse_color("#12345"),
            Some(Color32::from_rgb(1, 0x23, 0x45))
        );
    }

    #[test]
    fn unknown_themes_are_rejected() {
        assert!(Palette::from_spec("Solarized", None).is_err());
        assert!(Palette::from_spec("Islands Dark", Some("Nope")).is_err());
        assert!(set_active("Solarized", None).is_err());
    }

    #[test]
    fn every_offered_scheme_resolves_editor_tokens() {
        for scheme in SCHEMES {
            let palette = Palette::from_spec("Islands Dark", Some(scheme)).unwrap();
            for name in palette.missing.iter() {
                assert!(
                    !name.starts_with("EDITOR_") || *name == "EDITOR_BACKGROUND_PANEL",
                    "{scheme}: {name} unresolved"
                );
            }
        }
        let light = Palette::from_spec("Islands Dark", Some("Light")).unwrap();
        assert_eq!(light.get(tokens::EDITOR_BACKGROUND), Color32::WHITE);
    }
}

#[cfg(test)]
mod settings_tests {
    use super::*;
    use jetthing_settings::{SettingKind, SettingsRegistry, ids};

    fn option_ids(id: &str) -> Vec<String> {
        let registry = SettingsRegistry::intellij();
        match &registry.setting(id).unwrap().kind {
            SettingKind::Enum { options } => options.iter().map(|o| o.id.clone()).collect(),
            other => panic!("unexpected kind {other:?}"),
        }
    }

    #[test]
    fn settings_offer_exactly_the_themes_and_schemes_the_engine_knows() {
        assert_eq!(option_ids(ids::THEME), THEMES);
        let schemes = option_ids(ids::EDITOR_SCHEME);
        assert_eq!(schemes[0], "");
        assert_eq!(&schemes[1..], SCHEMES);
    }
}
