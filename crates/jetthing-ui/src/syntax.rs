//! Resolves highlight attribute keys to egui text formats through the active
//! editor color scheme (Islands Dark by default).

use eframe::egui::{self, Color32, Stroke};
use jetthing_language::{ColorScheme, EffectType, Rgba, TextAttributes};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn color(rgba: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(rgba.r, rgba.g, rgba.b, rgba.a)
}

fn scheme() -> &'static ColorScheme {
    static SCHEME: OnceLock<ColorScheme> = OnceLock::new();
    SCHEME.get_or_init(ColorScheme::islands_dark)
}

/// Resolved attributes for `key`, cached per key.
pub fn attributes(key: &'static str) -> TextAttributes {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, TextAttributes>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Ok(mut cache) = cache.lock() {
        return *cache.entry(key).or_insert_with(|| scheme().attributes(key));
    }
    scheme().attributes(key)
}

/// Applies the attributes of `key` on top of the editor's base text format.
pub fn text_format(key: &'static str, base: &egui::TextFormat) -> egui::TextFormat {
    let attributes = attributes(key);
    let mut format = base.clone();
    if let Some(foreground) = attributes.foreground {
        format.color = color(foreground);
    }
    if let Some(background) = attributes.background {
        format.background = color(background);
    }
    format.italics = attributes.font.italic;
    if let Some((effect, Some(effect_color))) = attributes.effect {
        let stroke = Stroke::new(1.0_f32, color(effect_color));
        match effect {
            EffectType::Strikeout => format.strikethrough = stroke,
            EffectType::Underline | EffectType::BoldUnderline | EffectType::DottedUnderline => {
                format.underline = stroke
            }
            // Wave underlines are painted by the diagnostics pass; boxes are not
            // supported by egui text formats.
            EffectType::WaveUnderline | EffectType::Boxed => {}
        }
    }
    format
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_resolve_to_islands_dark_formats() {
        let base = egui::TextFormat::default();
        assert_eq!(
            text_format("DEFAULT_KEYWORD", &base).color,
            Color32::from_rgb(0xCF, 0x8E, 0x6D)
        );
        let doc = text_format("DEFAULT_DOC_COMMENT", &base);
        assert_eq!(doc.color, Color32::from_rgb(0x5F, 0x82, 0x6B));
        assert!(doc.italics);
        assert_eq!(text_format("UNKNOWN_KEY", &base).color, base.color);
    }
}
