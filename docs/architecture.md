# Architecture

Jetthing is a native Rust workspace with a headless engine and replaceable UI shell.
The initial shell uses eframe/egui with wgpu. This choice provides a portable native
window, GPU rendering, IME/input handling, accessibility work in progress upstream,
and fast delivery of the first usable slice. It is not an architectural dependency:
no crate below `jetthing-ui` imports UI types.

## Layers

- `jetthing-command`: typed command metadata, JSON parameters/results, discovery,
  validation, execution, and structured errors.
- `jetthing-project`: safe project-root model, bounded file tree, UTF-8 document IO.
- `jetthing-text`: revisioned rope storage, Unicode-safe multi-selection transactions,
  typing coalescence, and history.
- `jetthing-language`: data-driven Tree-sitter language registry (grammar crates plus their
  bundled queries), a generic highlight engine producing `HighlightLayer`s keyed by IntelliJ
  text-attribute keys, and the editor color-scheme model loaded from `docs/spec/data/`.
- `jetthing-terminal`: native PTY process lifecycle, asynchronous output, input, and resizing.
- `jetthing-search`: cancellable generation-based background text search with streamed,
  bounded results and binary/large-file exclusion.
- `jetthing-git`: repository/branch/status modeling and argument-safe Git operations;
  it delegates repository semantics to the installed Git executable.
- `jetthing-runner`: detected build plans and cancellable child-process execution with
  streamed, separated stdout/stderr and explicit exit state.
- `jetthing-lsp`: JSON-RPC framing (`transport`), one server process with typed
  request/response plumbing, cancellation, `$/progress` and diagnostics (`client`), the
  server registry behind the `LanguageServerProvider` trait with PATH/`lsp.toml` discovery
  (`registry`), and `LspManager`, which starts servers lazily per workspace root and
  restarts crashed ones (`manager`). Uses `lsp-types`; no UI code. Popups live in
  `jetthing-ui`. `jetthing-fake-lsp` is a scriptable test server.
- `jetthing-plugins`: versioned TOML manifests, capability declarations, discovery,
  and validation. Executable/Wasm loading is deliberately not claimed yet.
- `jetthing-settings`: typed settings registry (bool/int/float/text/path/enum/list with
  defaults and validation), the Settings tree keyed by the reference configurable ids in
  `docs/spec/data/settings.json`, application/project layering, TOML persistence,
  search, and the dialog's draft/Apply/Reset model. No UI types.
- `jetthing-core`: application state, tool window layout model (one visible tool window
  per anchor), and built-in command handlers.
- `jetthing-ui`: native workbench shell split into `chrome` (main toolbar, stripe, status
  bar, window controls), `toolwindows` (Project, Commit, Find, Build/Run, Problems,
  Terminal), `editor` (tabs, viewport-native highlighted editor, completion popup),
  `palette` (Search Everywhere), `theme` (Islands Dark tokens and fonts), and `icons`
  (original 16×16 vector icons).
- `jetthing-app`: process startup, native window (custom title bar on Windows/Linux),
  and wgpu → OpenGL renderer fallback.
- `jetthing-control`: UI automation wire protocol (JSON lines), local-socket listener and
  blocking client; toolkit-independent. The workbench drains requests each frame.
- `jetthing-cli`: headless command discovery and execution, and `ui …` requests to a
  running workbench over the control socket.

## Command contract

Commands have stable IDs, human labels, JSON schemas expressed as documentation,
capability requirements, an enabled predicate, and an execution handler. GUI and CLI
use the same registry, and the control socket's `execute` request reaches every command of
a running workbench. Cancellation/progress and MCP transports are next-step extensions of
this contract.

## Settings extension point

Settings pages are data, not UI code. A subsystem that needs a page registers it with
the registry held in `IdeState::settings` (normally from its `jetthing-core` command
module, before the UI starts):

```rust
use jetthing_settings::{PageDef, Scope, SettingDef, SettingKind, SettingValue};

let registry = &mut state.settings.registry;
// Reuse the reference page id when the page exists in docs/spec/data/settings.json
// (e.g. `reference.settingsdialog.project.gradle`); register a new page only otherwise.
registry.register_page(PageDef::new("jetthing.rust", Some("language"), "Rust"))?;
registry.register_setting(
    SettingDef::new("rust.analyzerPath", "jetthing.rust", "rust-analyzer executable",
        SettingKind::Path, SettingValue::Text(String::new()))
        .group("Language Server")
        .scope(Scope::Project)
        .description("Empty searches PATH"),
)?;
// Read the effective value wherever it is used:
let path = state.settings.text("rust.analyzerPath");
```

The Settings dialog renders registered settings generically (checkbox, combo box,
number and text fields, grouped under headings), includes them in search, marks
modified pages and applies edits through `settings.apply`. Values are stored in
`<config dir>/settings.toml` (application) or `<project>/.jetthing/settings.toml`
(project); the config dir is `JETTHING_CONFIG_DIR` or the platform default. Pages that
need bespoke UI (Keymap, Plugins) get a page renderer in `jetthing-ui::settings_dialog`.
Commands: `settings.open {page?}`, `settings.get/set/reset {id}`,
`settings.apply {changes}`, `settings.search {query}`.

## Long-term direction

The semantic platform will layer VFS snapshots, incremental syntax, symbol/stub
indexes, dependency-aware invalidation, and prioritized cancellable jobs. LSP is one
provider behind a richer language-service interface, not the semantic architecture.
Untrusted plugins will use the WebAssembly Component Model with explicit capabilities;
native plugins will require an explicit trusted tier and process isolation.

## Reference findings

PhotoCraft validates the engine-first command-registry approach and generated parity
reports. IntelliJ's VFS/PSI/stub indexes motivate stable virtual identities and compact
externally-visible symbol indexes. Zed demonstrates GPU-first native UI; Lapce validates
rope-based text storage and language-service isolation. Jetthing reimplements behavior
cleanly and does not reuse protected assets or branding.
