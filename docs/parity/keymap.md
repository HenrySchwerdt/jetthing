# Keymap parity

Engine: `crates/jetthing-keymap` (no UI-toolkit dependencies). Commands live in
`crates/jetthing-core/src/keymap.rs`; egui dispatch in
`crates/jetthing-ui/src/keymap_dispatch.rs`.

Sources (public documentation only, transcribed as facts — action IDs, labels and
key combinations; no JetBrains files, artwork or branding are copied):

- Windows/Linux default keymap: <https://www.jetbrains.com/help/idea/reference-keymap-win-default.html>
- macOS default keymap: <https://www.jetbrains.com/help/idea/reference-keymap-mac-default.html>
- Go to Class/File/Symbol shortcuts: <https://www.jetbrains.com/help/idea/searching-everywhere.html>
- Keymap concepts (inheritance, two-stroke shortcuts, double modifiers):
  <https://www.jetbrains.com/help/idea/configuring-keyboard-and-mouse-shortcuts.html>

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| KEY-001 | IntelliJ Platform | Keystroke parsing (`ctrl shift A`, `Ctrl+Shift+A`, `⇧⌘A`) | Implemented | `keymap.resolve` | `keystroke::tests::*` | structured errors for malformed input; no IME/dead-key handling |
| KEY-002 | IntelliJ Platform | Double-modifier gestures (Double Shift, Double Ctrl, Double Ctrl+Arrow) | Implemented | `keymap.resolve` | `matcher::tests::double_modifier_*`, `tap_detector_reports_clean_taps_only` | fixed 400 ms tap window rather than the OS double-click interval; egui only reports modifier state per frame |
| KEY-003 | IntelliJ Platform | Two-stroke chords (`ctrl K, ctrl C`) with prefix fallback and timeout | Implemented | `keymap.resolve` | `matcher::tests::two_stroke_chord_and_prefix_fallback` | no status-bar hint for a pending first stroke yet |
| KEY-004 | IntelliJ Platform | Platform-aware modifiers and labels (`mod`, ⌘ vs Ctrl) | Implemented | `keymap.list` | `primary_modifier_is_platform_aware`, `labels_follow_platform_conventions` | label glyph set is jetthing's own |
| KEY-005 | IntelliJ Platform | Keymap inheritance and user overrides | Implemented | `keymap.loadUser`, `keymap.select` | `child_keymap_overrides_and_removes_inherited_shortcuts`, `user_keymap_loads_selects_and_rolls_back_on_error` | child entry replaces the parent's whole shortcut list (IntelliJ semantics); no Settings UI editor |
| KEY-006 | IntelliJ Platform | User keymap TOML (load/serialize) | Implemented | `keymap.loadUser` | `user_keymap_toml_round_trips`, `user_override_file_inherits_from_default_and_dispatches` | jetthing TOML schema; IntelliJ XML keymaps are not imported |
| KEY-007 | IntelliJ Platform | Conflict detection (ambiguous, shadowed, chord prefix) | Implemented | `keymap.conflicts` | `detects_ambiguous_and_prefix_conflicts`, `built_in_keymaps_parse_without_diagnostics_or_ambiguity` | contexts are a jetthing approximation of IntelliJ's component-based shortcut scoping |
| KEY-008 | IntelliJ IDEA keymap reference | Default "Windows/Linux" keymap data | Partial | `keymap.list` | `documented_top_shortcuts_resolve_per_platform`, `every_catalog_action_is_bound_in_both_default_keymaps` | 225 actions: 205 from all 17 reference sections plus 20 digit-bookmark actions; only actions on the public reference page; desktop-environment variants (GNOME/KDE/XWin) not shipped |
| KEY-009 | IntelliJ IDEA keymap reference | Default "macOS" keymap data | Partial | `keymap.list` | `documented_top_shortcuts_resolve_per_platform` | same coverage as KEY-008; "Next Occurrence of the Word at Caret" is N/A on the macOS page and left unbound |
| KEY-010 | IntelliJ Platform | Action → command mapping with unbound marker | Partial | `keymap.list` | `every_mapped_command_id_is_registered`, `unbound_marker_and_command_mapping` | 7 of 225 actions map to jetthing commands (Build Project, Save All, Undo, Redo, Basic Completion, Stop, Show Terminal window); the rest (e.g. Search Everywhere, Find Action, Go to Class, Reformat Code, Run, Debug) carry `unbound = true` until features land |
| KEY-011 | IntelliJ Platform | Keystroke dispatch from the UI | Partial | registry dispatch | `dispatches_mapped_keystrokes_and_passes_unbound_ones_through` | only command-mapped bindings are consumed; contexts limited to Global/Editor; NumPad keys are not distinguishable in egui |
| KEY-012 | IntelliJ Platform | Mouse shortcuts (Alt+Shift+Click etc.) | Partial | `keymap.list` | `mouse_shortcuts` | parsed, stored and listed; not dispatched |
| KEY-013 | IntelliJ Platform | Keymap settings UI (search by shortcut, assign, remove) | Missing | — | — | engine supports `conflicts_for`; no UI yet |
| KEY-014 | IntelliJ Platform | Other bundled keymaps (Eclipse, Emacs, VS Code, NetBeans …) | Missing | — | — | not shipped |

Statuses are intentionally conservative. "Implemented" means the listed scope works;
only "Verified" will count as independently parity-validated.
