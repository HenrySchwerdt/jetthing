//! Built-in keymaps shipped with jetthing.
//!
//! The data files are transcribed from the public IntelliJ IDEA keymap
//! reference pages (see each file's `source`), mapping IntelliJ action IDs to
//! jetthing command IDs in `data/actions.toml`.

use crate::keymap::{ActionCatalog, KeymapError, KeymapFile, KeymapSet};
use crate::keystroke::Platform;

pub const ACTIONS_TOML: &str = include_str!("../data/actions.toml");
pub const WINDOWS_LINUX_TOML: &str = include_str!("../data/keymaps/windows-linux.toml");
pub const MACOS_TOML: &str = include_str!("../data/keymaps/macos.toml");

/// Name of the built-in Windows/Linux default keymap.
pub const WINDOWS_LINUX: &str = "Windows/Linux";
/// Name of the built-in macOS default keymap.
pub const MACOS: &str = "macOS";

/// The default keymap name for `platform`.
pub fn default_keymap_name(platform: Platform) -> &'static str {
    if platform.is_mac() {
        MACOS
    } else {
        WINDOWS_LINUX
    }
}

/// The action catalog shipped with jetthing.
pub fn action_catalog() -> Result<ActionCatalog, KeymapError> {
    ActionCatalog::from_toml_str(ACTIONS_TOML)
}

/// A keymap set containing both built-in default keymaps.
pub fn builtin_keymaps() -> Result<KeymapSet, KeymapError> {
    let mut set = KeymapSet::new(action_catalog()?);
    for text in [WINDOWS_LINUX_TOML, MACOS_TOML] {
        set.insert(KeymapFile::from_toml_str(text)?);
    }
    Ok(set)
}
