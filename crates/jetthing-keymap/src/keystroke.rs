//! Keystrokes, chords and their textual forms.
//!
//! The canonical text form follows the space-separated notation used in
//! IntelliJ keymap files and documentation (`"ctrl shift A"`), extended with:
//!
//! * `"shift shift"` – a double press of a single modifier key;
//! * `", "` – separates the strokes of a two-stroke chord (`"ctrl K, ctrl C"`);
//! * `mod` / `primary` – the platform's primary modifier (Cmd on macOS, Ctrl
//!   elsewhere).
//!
//! The parser also accepts the `+`-separated form shown in documentation
//! (`"Ctrl+Shift+A"`) and the macOS glyphs `⌃ ⌥ ⇧ ⌘`.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// IntelliJ keymaps allow at most two keyboard strokes per shortcut.
pub const MAX_STROKES: usize = 2;

/// Operating-system family, used for primary-modifier resolution and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}

impl Platform {
    /// The platform this binary was compiled for.
    pub fn host() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    pub fn is_mac(self) -> bool {
        self == Self::Macos
    }

    /// Cmd on macOS, Ctrl on Windows and Linux.
    pub fn primary_modifier(self) -> Modifier {
        if self.is_mac() {
            Modifier::Meta
        } else {
            Modifier::Ctrl
        }
    }
}

/// A single modifier key. `Meta` is Cmd on macOS and the Windows/Super key elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

impl Modifier {
    pub const ALL: [Modifier; 4] = [Self::Ctrl, Self::Alt, Self::Shift, Self::Meta];

    fn canonical(self) -> &'static str {
        match self {
            Self::Ctrl => "ctrl",
            Self::Alt => "alt",
            Self::Shift => "shift",
            Self::Meta => "meta",
        }
    }

    fn label(self, platform: Platform) -> &'static str {
        match (self, platform.is_mac()) {
            (Self::Ctrl, true) => "⌃",
            (Self::Alt, true) => "⌥",
            (Self::Shift, true) => "⇧",
            (Self::Meta, true) => "⌘",
            (Self::Ctrl, false) => "Ctrl",
            (Self::Alt, false) => "Alt",
            (Self::Shift, false) => "Shift",
            (Self::Meta, false) => {
                if platform == Platform::Windows {
                    "Win"
                } else {
                    "Super"
                }
            }
        }
    }
}

/// A set of held modifier keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        meta: false,
    };

    pub fn with(mut self, modifier: Modifier) -> Self {
        self.set(modifier, true);
        self
    }

    pub fn contains(self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Ctrl => self.ctrl,
            Modifier::Alt => self.alt,
            Modifier::Shift => self.shift,
            Modifier::Meta => self.meta,
        }
    }

    pub fn set(&mut self, modifier: Modifier, value: bool) {
        match modifier {
            Modifier::Ctrl => self.ctrl = value,
            Modifier::Alt => self.alt = value,
            Modifier::Shift => self.shift = value,
            Modifier::Meta => self.meta = value,
        }
    }

    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }

    pub fn count(self) -> usize {
        Modifier::ALL.iter().filter(|m| self.contains(**m)).count()
    }

    pub fn iter(self) -> impl Iterator<Item = Modifier> {
        Modifier::ALL.into_iter().filter(move |m| self.contains(*m))
    }
}

/// A non-modifier key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    /// `A`..=`Z` (stored upper-case).
    Letter(char),
    /// Main-row digit `0`..=`9`.
    Digit(u8),
    /// Numeric keypad digit `0`..=`9`.
    Numpad(u8),
    /// Function key `F1`..=`F24`.
    F(u8),
    Enter,
    Escape,
    Tab,
    Backspace,
    /// Forward delete.
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    Space,
    Comma,
    Period,
    Slash,
    Backslash,
    Semicolon,
    Quote,
    BackQuote,
    OpenBracket,
    CloseBracket,
    Minus,
    Equals,
    Plus,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    ContextMenu,
}

impl Key {
    /// Canonical IntelliJ-style key name.
    pub fn canonical(self) -> String {
        match self {
            Self::Letter(c) => c.to_string(),
            Self::Digit(d) => d.to_string(),
            Self::Numpad(d) => format!("NUMPAD{d}"),
            Self::F(n) => format!("F{n}"),
            other => other.named_canonical().to_owned(),
        }
    }

    fn named_canonical(self) -> &'static str {
        match self {
            Self::Enter => "ENTER",
            Self::Escape => "ESCAPE",
            Self::Tab => "TAB",
            Self::Backspace => "BACK_SPACE",
            Self::Delete => "DELETE",
            Self::Insert => "INSERT",
            Self::Home => "HOME",
            Self::End => "END",
            Self::PageUp => "PAGE_UP",
            Self::PageDown => "PAGE_DOWN",
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Space => "SPACE",
            Self::Comma => "COMMA",
            Self::Period => "PERIOD",
            Self::Slash => "SLASH",
            Self::Backslash => "BACK_SLASH",
            Self::Semicolon => "SEMICOLON",
            Self::Quote => "QUOTE",
            Self::BackQuote => "BACK_QUOTE",
            Self::OpenBracket => "OPEN_BRACKET",
            Self::CloseBracket => "CLOSE_BRACKET",
            Self::Minus => "MINUS",
            Self::Equals => "EQUALS",
            Self::Plus => "PLUS",
            Self::NumpadAdd => "ADD",
            Self::NumpadSubtract => "SUBTRACT",
            Self::NumpadMultiply => "MULTIPLY",
            Self::NumpadDivide => "DIVIDE",
            Self::ContextMenu => "CONTEXT_MENU",
            Self::Letter(_) | Self::Digit(_) | Self::Numpad(_) | Self::F(_) => "",
        }
    }

    /// Human-readable label for menus and the keymap list.
    pub fn label(self, platform: Platform) -> String {
        let mac = platform.is_mac();
        let text = match self {
            Self::Letter(c) => return c.to_string(),
            Self::Digit(d) => return d.to_string(),
            Self::Numpad(d) => return format!("NumPad {d}"),
            Self::F(n) => return format!("F{n}"),
            Self::Enter => {
                if mac {
                    "↩"
                } else {
                    "Enter"
                }
            }
            Self::Escape => {
                if mac {
                    "⎋"
                } else {
                    "Esc"
                }
            }
            Self::Tab => {
                if mac {
                    "⇥"
                } else {
                    "Tab"
                }
            }
            Self::Backspace => {
                if mac {
                    "⌫"
                } else {
                    "Backspace"
                }
            }
            Self::Delete => {
                if mac {
                    "⌦"
                } else {
                    "Delete"
                }
            }
            Self::Insert => "Insert",
            Self::Home => {
                if mac {
                    "↖"
                } else {
                    "Home"
                }
            }
            Self::End => {
                if mac {
                    "↘"
                } else {
                    "End"
                }
            }
            Self::PageUp => {
                if mac {
                    "⇞"
                } else {
                    "Page Up"
                }
            }
            Self::PageDown => {
                if mac {
                    "⇟"
                } else {
                    "Page Down"
                }
            }
            Self::Up => "↑",
            Self::Down => "↓",
            Self::Left => "←",
            Self::Right => "→",
            Self::Space => "Space",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::BackQuote => "`",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::Minus => "-",
            Self::Equals => "=",
            Self::Plus => "+",
            Self::NumpadAdd => "NumPad +",
            Self::NumpadSubtract => "NumPad -",
            Self::NumpadMultiply => "NumPad *",
            Self::NumpadDivide => "NumPad /",
            Self::ContextMenu => "Menu",
        };
        text.to_owned()
    }

    /// Parses one key token (case-insensitive; `_`, `-` and spaces ignored).
    pub fn parse(token: &str) -> Option<Self> {
        let mut chars = token.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return Self::from_char(c);
        }
        let normalized: String = token
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .flat_map(char::to_uppercase)
            .collect();
        if let Some(rest) = normalized.strip_prefix("NUMPAD")
            && let Ok(digit) = rest.parse::<u8>()
            && digit <= 9
        {
            return Some(Self::Numpad(digit));
        }
        if let Some(rest) = normalized.strip_prefix('F')
            && let Ok(n) = rest.parse::<u8>()
            && (1..=24).contains(&n)
        {
            return Some(Self::F(n));
        }
        let key = match normalized.as_str() {
            "ENTER" | "RETURN" => Self::Enter,
            "ESCAPE" | "ESC" => Self::Escape,
            "TAB" => Self::Tab,
            "BACKSPACE" => Self::Backspace,
            "DELETE" | "DEL" | "FORWARDDELETE" => Self::Delete,
            "INSERT" | "INS" => Self::Insert,
            "HOME" => Self::Home,
            "END" => Self::End,
            "PAGEUP" | "PGUP" => Self::PageUp,
            "PAGEDOWN" | "PGDN" => Self::PageDown,
            "UP" | "ARROWUP" => Self::Up,
            "DOWN" | "ARROWDOWN" => Self::Down,
            "LEFT" | "ARROWLEFT" => Self::Left,
            "RIGHT" | "ARROWRIGHT" => Self::Right,
            "SPACE" => Self::Space,
            "COMMA" => Self::Comma,
            "PERIOD" | "DOT" => Self::Period,
            "SLASH" => Self::Slash,
            "BACKSLASH" => Self::Backslash,
            "SEMICOLON" => Self::Semicolon,
            "QUOTE" => Self::Quote,
            "BACKQUOTE" | "BACKTICK" | "GRAVE" => Self::BackQuote,
            "OPENBRACKET" => Self::OpenBracket,
            "CLOSEBRACKET" => Self::CloseBracket,
            "MINUS" => Self::Minus,
            "EQUALS" => Self::Equals,
            "PLUS" => Self::Plus,
            "ADD" | "NUMPADADD" | "NUMPADPLUS" => Self::NumpadAdd,
            "SUBTRACT" | "NUMPADSUBTRACT" | "NUMPADMINUS" => Self::NumpadSubtract,
            "MULTIPLY" | "NUMPADMULTIPLY" => Self::NumpadMultiply,
            "DIVIDE" | "NUMPADDIVIDE" => Self::NumpadDivide,
            "CONTEXTMENU" | "MENU" => Self::ContextMenu,
            _ => return None,
        };
        Some(key)
    }

    fn from_char(c: char) -> Option<Self> {
        let key = match c {
            'a'..='z' => Self::Letter(c.to_ascii_uppercase()),
            'A'..='Z' => Self::Letter(c),
            '0'..='9' => Self::Digit(c as u8 - b'0'),
            ',' => Self::Comma,
            '.' => Self::Period,
            '/' => Self::Slash,
            '\\' => Self::Backslash,
            ';' => Self::Semicolon,
            '\'' => Self::Quote,
            '`' => Self::BackQuote,
            '[' => Self::OpenBracket,
            ']' => Self::CloseBracket,
            '-' => Self::Minus,
            '=' => Self::Equals,
            '+' => Self::Plus,
            '↑' => Self::Up,
            '↓' => Self::Down,
            '←' => Self::Left,
            '→' => Self::Right,
            '↩' => Self::Enter,
            '⎋' => Self::Escape,
            '⇥' => Self::Tab,
            '⌫' => Self::Backspace,
            '⌦' => Self::Delete,
            _ => return None,
        };
        Some(key)
    }
}

/// One key press with the modifiers held at that moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Keystroke {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Keystroke {
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    pub fn plain(key: Key) -> Self {
        Self::new(Modifiers::NONE, key)
    }

    /// Parses a single keystroke such as `"ctrl shift A"` or `"Ctrl+Shift+A"`.
    pub fn parse(text: &str, platform: Platform) -> Result<Self, KeyParseError> {
        match Stroke::parse(text, platform)? {
            Stroke::Key(stroke) => Ok(stroke),
            Stroke::DoubleModifier(_) => Err(KeyParseError::new(
                text,
                "expected a keystroke with a key, found a double modifier press",
            )),
        }
    }
}

/// One element of a key sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stroke {
    Key(Keystroke),
    /// Pressing and releasing the same modifier twice in quick succession,
    /// e.g. IntelliJ's "Double Shift" for Search Everywhere.
    DoubleModifier(Modifier),
}

impl Stroke {
    pub fn parse(text: &str, platform: Platform) -> Result<Self, KeyParseError> {
        let tokens = tokenize(text);
        if tokens.is_empty() {
            return Err(KeyParseError::new(text, "keystroke is empty"));
        }
        let mut modifiers = Modifiers::NONE;
        let mut modifier_tokens = Vec::new();
        let mut key = None;
        for token in &tokens {
            if let Some(modifier) = parse_modifier(token, platform) {
                if key.is_some() {
                    return Err(KeyParseError::new(
                        text,
                        format!("modifier `{token}` must come before the key"),
                    ));
                }
                modifier_tokens.push(modifier);
                modifiers.set(modifier, true);
                continue;
            }
            if key.is_some() {
                return Err(KeyParseError::new(
                    text,
                    format!("unexpected second key `{token}`; separate chord strokes with `,`"),
                ));
            }
            key = Some(Key::parse(token).ok_or_else(|| {
                KeyParseError::new(text, format!("unknown key or modifier `{token}`"))
            })?);
        }
        match key {
            Some(key) => Ok(Self::Key(Keystroke::new(modifiers, key))),
            None if modifier_tokens.len() == 2 && modifier_tokens[0] == modifier_tokens[1] => {
                Ok(Self::DoubleModifier(modifier_tokens[0]))
            }
            None => Err(KeyParseError::new(
                text,
                "keystroke has no key; use e.g. `shift shift` for a double modifier press",
            )),
        }
    }

    pub fn canonical(&self) -> String {
        match self {
            Self::Key(stroke) => {
                let mut parts: Vec<String> = stroke
                    .modifiers
                    .iter()
                    .map(|m| m.canonical().to_owned())
                    .collect();
                parts.push(stroke.key.canonical());
                parts.join(" ")
            }
            Self::DoubleModifier(m) => format!("{0} {0}", m.canonical()),
        }
    }

    pub fn label(&self, platform: Platform) -> String {
        match self {
            Self::Key(stroke) => {
                // Ctrl, Alt/Option, Shift, Meta/Cmd: the order both macOS
                // menus (⌃⌥⇧⌘) and Windows/Linux labels (Ctrl+Alt+Shift) use.
                let mods = stroke.modifiers.iter().map(|m| m.label(platform));
                if platform.is_mac() {
                    let mut text: String = mods.collect();
                    text.push_str(&stroke.key.label(platform));
                    text
                } else {
                    let mut parts: Vec<String> = mods.map(str::to_owned).collect();
                    parts.push(stroke.key.label(platform));
                    parts.join("+")
                }
            }
            Self::DoubleModifier(m) => format!("Double {}", m.label(platform)),
        }
    }
}

/// A complete shortcut: one or two strokes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeySequence(Vec<Stroke>);

impl KeySequence {
    pub fn new(strokes: Vec<Stroke>) -> Result<Self, KeyParseError> {
        if strokes.is_empty() {
            return Err(KeyParseError::new("", "key sequence is empty"));
        }
        if strokes.len() > MAX_STROKES {
            return Err(KeyParseError::new(
                "",
                format!("key sequences support at most {MAX_STROKES} strokes"),
            ));
        }
        Ok(Self(strokes))
    }

    pub fn single(stroke: Stroke) -> Self {
        Self(vec![stroke])
    }

    /// Parses `"ctrl shift A"`, `"shift shift"` or `"ctrl K, ctrl C"`.
    pub fn parse(text: &str, platform: Platform) -> Result<Self, KeyParseError> {
        let parts: Vec<&str> = text.split(',').collect();
        // `ctrl ,` style: a trailing empty part means the comma key itself.
        let mut strokes = Vec::new();
        let mut index = 0;
        while index < parts.len() {
            let mut part = parts[index].to_owned();
            if part.trim().is_empty() && index > 0 {
                return Err(KeyParseError::new(text, "empty chord stroke"));
            }
            // A part ending with a modifier followed by an empty part is the comma key.
            if index + 1 < parts.len()
                && parts[index + 1].trim().is_empty()
                && tokenize(&part)
                    .iter()
                    .all(|token| parse_modifier(token, platform).is_some())
                && !part.trim().is_empty()
            {
                part.push_str(" COMMA");
                index += 1;
            }
            strokes.push(Stroke::parse(&part, platform).map_err(|e| e.with_input(text))?);
            index += 1;
        }
        Self::new(strokes).map_err(|e| e.with_input(text))
    }

    pub fn strokes(&self) -> &[Stroke] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// True when `self` is a strict prefix of `other`.
    pub fn is_strict_prefix_of(&self, other: &KeySequence) -> bool {
        self.0.len() < other.0.len() && other.0.starts_with(&self.0)
    }

    pub fn canonical(&self) -> String {
        self.0
            .iter()
            .map(Stroke::canonical)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn label(&self, platform: Platform) -> String {
        self.0
            .iter()
            .map(|s| s.label(platform))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

/// Mouse button used by a mouse shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// A mouse shortcut such as IntelliJ's `Alt+Shift+Click` (add/remove caret).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MouseShortcut {
    pub modifiers: Modifiers,
    pub button: MouseButton,
    pub clicks: u8,
}

impl MouseShortcut {
    /// Parses `"alt shift button1"`, `"button2 doubleClick"`, `"Alt+Shift+Click"`.
    pub fn parse(text: &str, platform: Platform) -> Result<Self, KeyParseError> {
        let mut modifiers = Modifiers::NONE;
        let mut button = None;
        let mut clicks = 1;
        for token in tokenize(text) {
            if let Some(modifier) = parse_modifier(&token, platform) {
                modifiers.set(modifier, true);
                continue;
            }
            let normalized = token.to_ascii_lowercase().replace(['-', '_'], "");
            match normalized.as_str() {
                "button1" | "click" | "leftclick" => button = Some(MouseButton::Left),
                "button2" | "middleclick" => button = Some(MouseButton::Middle),
                "button3" | "rightclick" => button = Some(MouseButton::Right),
                "doubleclick" => clicks = 2,
                _ => {
                    return Err(KeyParseError::new(
                        text,
                        format!("unknown mouse token `{token}`"),
                    ));
                }
            }
        }
        let button =
            button.ok_or_else(|| KeyParseError::new(text, "mouse shortcut has no button"))?;
        Ok(Self {
            modifiers,
            button,
            clicks,
        })
    }

    pub fn canonical(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| m.canonical().to_owned())
            .collect();
        parts.push(
            match self.button {
                MouseButton::Left => "button1",
                MouseButton::Middle => "button2",
                MouseButton::Right => "button3",
            }
            .into(),
        );
        if self.clicks == 2 {
            parts.push("doubleClick".into());
        }
        parts.join(" ")
    }

    pub fn label(&self, platform: Platform) -> String {
        let stroke_mods: Vec<&str> = self.modifiers.iter().map(|m| m.label(platform)).collect();
        let button = match (self.button, self.clicks) {
            (MouseButton::Left, 1) => "Click",
            (MouseButton::Left, _) => "Double-Click",
            (MouseButton::Middle, _) => "Middle-Click",
            (MouseButton::Right, _) => "Right-Click",
        };
        if platform.is_mac() {
            format!("{}{}", stroke_mods.concat(), button)
        } else {
            let mut parts = stroke_mods;
            parts.push(button);
            parts.join("+")
        }
    }
}

/// A structured, user-facing parse error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid shortcut `{input}`: {reason}")]
pub struct KeyParseError {
    pub input: String,
    pub reason: String,
}

impl KeyParseError {
    fn new(input: &str, reason: impl Into<String>) -> Self {
        Self {
            input: input.trim().to_owned(),
            reason: reason.into(),
        }
    }

    fn with_input(mut self, input: &str) -> Self {
        self.input = input.trim().to_owned();
        self
    }
}

fn parse_modifier(token: &str, platform: Platform) -> Option<Modifier> {
    let modifier = match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" | "ctl" | "⌃" => Modifier::Ctrl,
        "alt" | "opt" | "option" | "⌥" => Modifier::Alt,
        "shift" | "⇧" => Modifier::Shift,
        "meta" | "cmd" | "command" | "super" | "win" | "⌘" => Modifier::Meta,
        "mod" | "primary" => platform.primary_modifier(),
        _ => return None,
    };
    Some(modifier)
}

/// Splits on whitespace and `+`, keeping a literal `+` key and splitting glued
/// macOS glyphs (`⇧⌘A`).
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for word in text.split_whitespace() {
        if word == "+" {
            tokens.push("+".to_owned());
            continue;
        }
        let trailing_plus = word.len() > 1 && word.ends_with("++");
        for piece in word.split('+').filter(|p| !p.is_empty()) {
            split_glyphs(piece, &mut tokens);
        }
        if trailing_plus {
            tokens.push("+".to_owned());
        }
    }
    tokens
}

fn split_glyphs(piece: &str, tokens: &mut Vec<String>) {
    let mut rest = String::new();
    for c in piece.chars() {
        if matches!(c, '⌃' | '⌥' | '⇧' | '⌘') {
            if !rest.is_empty() {
                tokens.push(std::mem::take(&mut rest));
            }
            tokens.push(c.to_string());
        } else {
            rest.push(c);
        }
    }
    if !rest.is_empty() {
        tokens.push(rest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(text: &str) -> KeySequence {
        KeySequence::parse(text, Platform::Linux).unwrap()
    }

    #[test]
    fn parses_intellij_and_documentation_notations_identically() {
        assert_eq!(seq("ctrl shift A"), seq("Ctrl+Shift+A"));
        assert_eq!(seq("shift ctrl a"), seq("ctrl shift A"));
        assert_eq!(seq("ctrl shift A").canonical(), "ctrl shift A");
        assert_eq!(
            KeySequence::parse("⇧⌘A", Platform::Macos).unwrap(),
            KeySequence::parse("meta shift A", Platform::Macos).unwrap()
        );
    }

    #[test]
    fn parses_double_modifier_and_two_stroke_chords() {
        assert_eq!(
            seq("shift shift").strokes(),
            &[Stroke::DoubleModifier(Modifier::Shift)]
        );
        let chord = seq("ctrl K, ctrl C");
        assert_eq!(chord.len(), 2);
        assert_eq!(chord.canonical(), "ctrl K, ctrl C");
        assert_eq!(chord.label(Platform::Windows), "Ctrl+K, Ctrl+C");
        let clone = seq("ctrl ctrl, ctrl UP");
        assert_eq!(clone.strokes()[0], Stroke::DoubleModifier(Modifier::Ctrl));
    }

    #[test]
    fn primary_modifier_is_platform_aware() {
        let mac = KeySequence::parse("mod S", Platform::Macos).unwrap();
        let win = KeySequence::parse("mod S", Platform::Windows).unwrap();
        assert_eq!(mac.canonical(), "meta S");
        assert_eq!(win.canonical(), "ctrl S");
        assert_eq!(mac.label(Platform::Macos), "⌘S");
        assert_eq!(win.label(Platform::Windows), "Ctrl+S");
    }

    #[test]
    fn labels_follow_platform_conventions() {
        let s = KeySequence::parse("ctrl alt shift meta L", Platform::Macos).unwrap();
        assert_eq!(s.label(Platform::Macos), "⌃⌥⇧⌘L");
        assert_eq!(seq("shift shift").label(Platform::Macos), "Double ⇧");
        assert_eq!(seq("shift shift").label(Platform::Linux), "Double Shift");
        assert_eq!(seq("ctrl ADD").label(Platform::Windows), "Ctrl+NumPad +");
    }

    #[test]
    fn punctuation_keys_including_comma_and_plus() {
        assert_eq!(seq("ctrl ,").canonical(), "ctrl COMMA");
        assert_eq!(seq("ctrl COMMA").canonical(), "ctrl COMMA");
        assert_eq!(seq("Ctrl++").canonical(), "ctrl PLUS");
        assert_eq!(seq("ctrl +").canonical(), "ctrl PLUS");
        assert_eq!(seq("alt `").canonical(), "alt BACK_QUOTE");
        assert_eq!(seq("ctrl OPEN_BRACKET"), seq("ctrl ["));
        assert_eq!(seq("ctrl page_up"), seq("Ctrl+PageUp"));
    }

    #[test]
    fn rejects_malformed_input_with_structured_errors() {
        for bad in [
            "",
            "ctrl",
            "ctrl shift",
            "ctrl A B",
            "A ctrl",
            "ctrl BOGUS",
            "a, b, c",
            "ctrl K,",
            "F99",
        ] {
            let error = KeySequence::parse(bad, Platform::Linux).unwrap_err();
            assert!(!error.reason.is_empty(), "{bad}");
        }
    }

    #[test]
    fn canonical_form_round_trips() {
        for text in [
            "ctrl shift A",
            "shift shift",
            "ctrl K, ctrl C",
            "alt shift F10",
            "ctrl alt shift NUMPAD5",
            "meta BACK_SPACE",
            "ctrl ctrl, ctrl UP",
        ] {
            let parsed = seq(text);
            assert_eq!(seq(&parsed.canonical()), parsed);
        }
    }

    #[test]
    fn mouse_shortcuts() {
        let m = MouseShortcut::parse("alt shift button1", Platform::Linux).unwrap();
        assert_eq!(m.label(Platform::Windows), "Alt+Shift+Click");
        assert_eq!(
            MouseShortcut::parse("Alt+Shift+Middle-Click", Platform::Linux).unwrap(),
            MouseShortcut::parse("alt shift button2", Platform::Linux).unwrap()
        );
        assert!(MouseShortcut::parse("alt", Platform::Linux).is_err());
    }
}
