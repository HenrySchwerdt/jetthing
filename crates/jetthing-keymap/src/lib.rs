//! Keymap engine for jetthing.
//!
//! * [`keystroke`] – keystrokes, double-modifier gestures and two-stroke
//!   chords, with platform-aware parsing and display.
//! * [`keymap`] – keymap files (TOML), inheritance and user overrides,
//!   context-aware resolution and conflict detection.
//! * [`matcher`] – a toolkit-independent state machine that turns key events
//!   into bindings.
//! * [`defaults`] – the built-in "Windows/Linux" and "macOS" keymaps.
//!
//! This crate has no UI-toolkit dependencies.

pub mod defaults;
pub mod keymap;
pub mod keystroke;
pub mod matcher;

pub use defaults::{MACOS, WINDOWS_LINUX, builtin_keymaps, default_keymap_name};
pub use keymap::{
    ActionCatalog, ActionSpec, BindingSpec, BindingTarget, Conflict, ConflictKind, KeyContext,
    KeymapDiagnostic, KeymapError, KeymapFile, KeymapSet, Lookup, ResolvedBinding, ResolvedKeymap,
};
pub use keystroke::{
    Key, KeyParseError, KeySequence, Keystroke, Modifier, Modifiers, MouseButton, MouseShortcut,
    Platform, Stroke,
};
pub use matcher::{KeyInput, KeystrokeMatcher, MatchResult, MatcherConfig, ModifierTapDetector};
