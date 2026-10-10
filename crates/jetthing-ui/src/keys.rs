//! Platform-appropriate shortcut labels.
//!
//! Command metadata stores macOS notation (see `docs/spec/keymaps.md`). On
//! Windows/Linux the default keymap differs, so labels are translated: modifier
//! glyphs become words and a few bindings use their documented non-macOS form.

/// Translates a macOS-notation shortcut for display on the current platform.
pub fn display(mac: &str) -> String {
    if cfg!(target_os = "macos") {
        mac.to_owned()
    } else {
        for_windows_linux(mac)
    }
}

pub fn for_windows_linux(mac: &str) -> String {
    match mac {
        "" => return String::new(),
        "Double ⇧" => return "Double Shift".into(),
        // Default keymap differences between macOS and Windows/Linux.
        "⌃R" => return "Shift+F10".into(),
        "⌘W" => return "Ctrl+F4".into(),
        "⌥⌘S" => return "Ctrl+S".into(),
        "⌥F12" => return "Alt+F12".into(),
        _ => {}
    }
    let mut chars = mac.chars().peekable();
    let (mut ctrl, mut alt, mut shift, mut command) = (false, false, false, false);
    while let Some(&c) = chars.peek() {
        match c {
            '⌃' => ctrl = true,
            '⌥' => alt = true,
            '⇧' => shift = true,
            '⌘' => command = true,
            _ => break,
        }
        chars.next();
    }
    let key: String = chars.collect();
    // Tool window activation (⌘1 … ⌘9, ⌘0) is Alt+digit on Windows/Linux.
    if command
        && !ctrl
        && !alt
        && !shift
        && key.len() == 1
        && key.chars().all(|c| c.is_ascii_digit())
    {
        return format!("Alt+{key}");
    }
    let mut parts = Vec::new();
    if ctrl || command {
        parts.push("Ctrl");
    }
    if alt {
        parts.push("Alt");
    }
    if shift {
        parts.push("Shift");
    }
    parts.push(&key);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_mac_notation() {
        assert_eq!(for_windows_linux("⇧⌘A"), "Ctrl+Shift+A");
        assert_eq!(for_windows_linux("⌘F9"), "Ctrl+F9");
        assert_eq!(for_windows_linux("⌘1"), "Alt+1");
        assert_eq!(for_windows_linux("⇧⌘F12"), "Ctrl+Shift+F12");
        assert_eq!(for_windows_linux("⌃R"), "Shift+F10");
        assert_eq!(for_windows_linux("Double ⇧"), "Double Shift");
        assert_eq!(for_windows_linux(""), "");
    }
}
