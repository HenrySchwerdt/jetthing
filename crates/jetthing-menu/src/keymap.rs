//! Shortcut lookup for menu items from the bundled default keymaps
//! (`docs/spec/data/keymaps.json`, already resolved through keymap inheritance).

use crate::MenuError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

const KEYMAPS_JSON: &str = include_str!("../../../docs/spec/data/keymaps.json");

/// Default keymap on Windows and Linux.
pub const DEFAULT_KEYMAP: &str = "$default";
/// Default keymap on macOS.
pub const MAC_KEYMAP: &str = "Mac OS X 10.5+";

/// Anything that can name the shortcuts of an action id. The bundled
/// [`Keymap`] implements it; a user-editable keymap engine can replace it.
pub trait ShortcutSource {
    /// All shortcuts of `action_id`, in keymap notation (`Ctrl+Shift+A`,
    /// `Ctrl+K, S`, `double Shift`, `Alt+Click`).
    fn shortcuts(&self, action_id: &str) -> &[String];

    /// Whether shortcuts are shown with macOS glyphs (`⇧⌘A`).
    fn mac_glyphs(&self) -> bool {
        false
    }

    /// The shortcut a menu shows: the first keyboard shortcut (mouse shortcuts are
    /// not shown in menus), formatted for the platform.
    fn menu_shortcut(&self, action_id: &str) -> Option<String> {
        let first = self
            .shortcuts(action_id)
            .iter()
            .find(|shortcut| is_keyboard(shortcut))?;
        Some(if self.mac_glyphs() {
            mac_glyphs(first)
        } else {
            first.clone()
        })
    }
}

/// No shortcuts at all.
pub struct NoShortcuts;

impl ShortcutSource for NoShortcuts {
    fn shortcuts(&self, _action_id: &str) -> &[String] {
        &[]
    }
}

#[derive(Debug, Clone, Deserialize)]
struct KeymapEntry {
    parent: Option<String>,
    shortcuts: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct KeymapsFile {
    keymaps: BTreeMap<String, KeymapEntry>,
}

/// One bundled keymap's effective shortcuts.
#[derive(Debug, Clone)]
pub struct Keymap {
    name: String,
    parent: Option<String>,
    shortcuts: BTreeMap<String, Vec<String>>,
    mac_glyphs: bool,
}

fn bundled_keymaps() -> Result<&'static BTreeMap<String, KeymapEntry>, MenuError> {
    static BUNDLED: LazyLock<Result<BTreeMap<String, KeymapEntry>, MenuError>> =
        LazyLock::new(|| {
            serde_json::from_str::<KeymapsFile>(KEYMAPS_JSON)
                .map(|file| file.keymaps)
                .map_err(|error| MenuError::Data(error.to_string()))
        });
    BUNDLED.as_ref().map_err(Clone::clone)
}

impl Keymap {
    /// A bundled keymap by its reference name (`$default`, `Mac OS X 10.5+`, …).
    pub fn bundled(name: &str) -> Result<Self, MenuError> {
        let entry = bundled_keymaps()?
            .get(name)
            .ok_or_else(|| MenuError::UnknownKeymap(name.to_owned()))?;
        Ok(Self {
            name: name.to_owned(),
            parent: entry.parent.clone(),
            shortcuts: entry.shortcuts.clone(),
            mac_glyphs: is_mac_keymap(name),
        })
    }

    /// The platform default: [`MAC_KEYMAP`] on macOS, [`DEFAULT_KEYMAP`] elsewhere.
    pub fn platform_default() -> Result<Self, MenuError> {
        Self::bundled(if cfg!(target_os = "macos") {
            MAC_KEYMAP
        } else {
            DEFAULT_KEYMAP
        })
    }

    /// Names of every bundled keymap.
    pub fn bundled_names() -> Result<Vec<String>, MenuError> {
        Ok(bundled_keymaps()?.keys().cloned().collect())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent(&self) -> Option<&str> {
        self.parent.as_deref()
    }
}

impl ShortcutSource for Keymap {
    fn shortcuts(&self, action_id: &str) -> &[String] {
        self.shortcuts
            .get(action_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn mac_glyphs(&self) -> bool {
        self.mac_glyphs
    }
}

/// Keymaps that inherit from the macOS keymap use ⌘ instead of Ctrl.
fn is_mac_keymap(name: &str) -> bool {
    let mut current = Some(name.to_owned());
    let Ok(all) = bundled_keymaps() else {
        return false;
    };
    for _ in 0..8 {
        let Some(name) = current else {
            return false;
        };
        if name == MAC_KEYMAP || name == "Mac OS X" {
            return true;
        }
        current = all.get(&name).and_then(|entry| entry.parent.clone());
    }
    false
}

/// Mouse and touch shortcuts are not displayed in menus.
fn is_keyboard(shortcut: &str) -> bool {
    !shortcut.is_empty()
        && !["Click", "Force touch", "Mouse ", "Wheel"]
            .iter()
            .any(|mouse| shortcut.contains(mouse))
}

/// `Cmd+Shift+A` → `⇧⌘A`, `Ctrl+K, S` → `⌃K, S`, `double Shift` → `Double ⇧`.
/// Modifiers are written in the macOS order ⌃⌥⇧⌘.
pub fn mac_glyphs(shortcut: &str) -> String {
    if let Some(key) = shortcut.strip_prefix("double ") {
        return format!("Double {}", modifier_glyph(key).unwrap_or(key));
    }
    shortcut
        .split(", ")
        .map(|stroke| {
            let parts: Vec<&str> = stroke.split('+').collect();
            // `Cmd++` splits into ["Cmd", "", ""]: the key is a literal plus.
            let (modifiers, key) = match parts.as_slice() {
                [mods @ .., "", ""] => (mods, "+"),
                [mods @ .., key] => (mods, *key),
                [] => (&[][..], ""),
            };
            let mut out = String::new();
            for glyph in ["⌃", "⌥", "⇧", "⌘"] {
                if modifiers
                    .iter()
                    .any(|modifier| modifier_glyph(modifier) == Some(glyph))
                {
                    out.push_str(glyph);
                }
            }
            out.push_str(key_glyph(key));
            out
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn modifier_glyph(modifier: &str) -> Option<&'static str> {
    match modifier.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some("⌃"),
        "alt" | "option" => Some("⌥"),
        "shift" => Some("⇧"),
        "cmd" | "meta" | "command" => Some("⌘"),
        _ => None,
    }
}

fn key_glyph(key: &str) -> &str {
    match key {
        "Backspace" => "⌫",
        "Delete" => "⌦",
        "Enter" => "↩",
        "Tab" => "⇥",
        "Esc" => "⎋",
        "Up" => "↑",
        "Down" => "↓",
        "Left" => "←",
        "Right" => "→",
        "Page Up" => "⇞",
        "Page Down" => "⇟",
        "Home" => "↖",
        "End" => "↘",
        other => other,
    }
}
