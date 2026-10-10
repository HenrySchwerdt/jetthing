# Keymap data

- `actions.toml`: IntelliJ action IDs, documented labels and categories, the
  jetthing command each maps to (`command = "..."`), or the explicit
  `unbound = true` marker when jetthing has no implementation yet.
- `keymaps/windows-linux.toml`, `keymaps/macos.toml`: default shortcuts per action.

Transcribed by hand from the public IntelliJ IDEA keymap reference
(https://www.jetbrains.com/help/idea/reference-keymap-win-default.html and
https://www.jetbrains.com/help/idea/reference-keymap-mac-default.html). Only facts
(action names and key combinations) are recorded; no JetBrains files are copied.
Notation: `ctrl shift A`, `shift shift` (double press), `ctrl K, ctrl C` (two strokes),
`mod` (Cmd on macOS, Ctrl elsewhere). See `docs/parity/keymap.md`.
