# Themes: Islands Dark / Islands Light

Captures in `jetthing/themes/` (Linux, Xvfb 1600×1000, `JETTHING_RENDERER=glow`).

## Steps

1. Put `"appearance.theme" = "Islands Light"` under `[settings]` in
   `$JETTHING_CONFIG_DIR/settings.toml` (or pick it in Settings › Appearance › Theme), then
   `cargo run -p jetthing-app -- . crates/jetthing-settings/src/lib.rs`.
2. Alt+F12, then Double Shift and type `setting` → `islands-light-workbench.png`: toolbar,
   stripes, islands, project tree, editor (scheme "Light"), terminal and Search
   Everywhere all use the Islands Light tokens.
3. Ctrl+Alt+S → `islands-light-settings.png`.
4. Theme › Islands Dark, Apply → `islands-dark-after-live-switch.png`: the whole window
   switches on the next frame without a restart. `settings.toml` drops the override
   because Islands Dark is the default.

## How it works

Every color is a token (`crates/jetthing-ui/src/theme_engine.rs`) naming, per theme,
`ui:` keys and `palette:` entries in `docs/spec/data/themes.json` and `color:` / `attr:`
entries of the theme's editor scheme in `docs/spec/data/color-schemes.json` (falling back
along the scheme's lineage). A test asserts that resolving Islands Dark from the data
reproduces the previous hand-written values exactly, so the dark look is unchanged.

## Gaps

- There are no Islands Light reference screenshots yet (the PR #40 references are dark), so
  light parity is checked against the data, not against pixels.
- File-type icon accents (Rust, TOML, Markdown, …) and the window-close hover red are fixed
  colors in both themes.
- Only Islands Dark and Islands Light are offered as themes. "Sync with OS" is missing.
