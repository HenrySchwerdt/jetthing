# Jetthing

Jetthing is an experimental, clean-room, native Rust IDE platform with a workbench
modelled on the observable layout of the IntelliJ New UI ("Islands Dark"): merged main
toolbar with project, VCS and run widgets, a tool window stripe, rounded editor and tool
window islands, Search Everywhere, and a status bar with navigation breadcrumbs.

![Jetthing editor](docs/parity/evidence/linux-editor-project-tree.png)

## Run

```bash
cargo run -p jetthing-app -- .                      # open a project
cargo run -p jetthing-app -- . src/main.rs          # open a project and files
cargo run -p jetthing-cli -- commands               # headless command discovery
cargo run -p xtask -- parity                        # regenerate the parity dashboard
```

If wgpu finds no usable adapter (VMs, old drivers) the app falls back to OpenGL
automatically; `JETTHING_RENDERER=glow` forces it.

## Build a distribution

```bash
cargo run -p xtask -- dist     # release build staged in dist/jetthing-<version>-<os>-<arch>/
```

Pushing a `v*` tag runs `.github/workflows/release.yml`, which builds Linux, macOS and
Windows archives and attaches them to the GitHub release.

## Keyboard

Shortcuts follow the default keymap of the reference IDE (macOS / Windows & Linux):

| Action | macOS | Windows / Linux |
|---|---|---|
| Search Everywhere | Double ⇧ | Double Shift |
| Find Action | ⇧⌘A | Ctrl+Shift+A |
| Project / Commit tool window | ⌘1 / ⌘0 | Alt+1 / Alt+0 |
| Find in Files | ⇧⌘F | Ctrl+Shift+F |
| Terminal | ⌥F12 | Alt+F12 |
| Run / Build | ⌃R / ⌘F9 | Shift+F10 / Ctrl+F9 |
| Close tab | ⌘W | Ctrl+F4 |
| Hide all tool windows | ⇧⌘F12 | Ctrl+Shift+F12 |

## Assets and trademarks

UI colors are facts extracted from IntelliJ Community theme files (Apache-2.0, see
`docs/spec/data/NOTICE.md`). Fonts are Inter and JetBrains Mono under the SIL Open Font
License (`crates/jetthing-ui/assets/fonts/`). Icons are original vector drawings; no
JetBrains icons or artwork are bundled. IntelliJ and JetBrains are trademarks of
JetBrains s.r.o.; Jetthing is not affiliated with or endorsed by JetBrains.

See `docs/roadmap.md` for honest status and limitations.
