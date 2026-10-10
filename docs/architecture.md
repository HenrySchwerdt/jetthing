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
- `jetthing-toolchains`: toolchain provisioning (language servers, JDKs/SDKs, build tools,
  debuggers, formatters). Component model and `ComponentProvider` catalog trait, version
  requests, resumable SHA-256-verified downloads through `HTTPS_PROXY`/`NO_PROXY`, safe
  `tar.gz`/`tar.xz`/`zip`/`gz` extraction, atomic installs under
  `<data dir>/jetthing/toolchains/<component>/<version>/`, PATH-first discovery, project pins
  and a background install service. No UI code; see "Toolchain provisioning" below.
- `jetthing-plugins`: versioned TOML manifests, capability declarations, discovery,
  and validation. Executable/Wasm loading is deliberately not claimed yet.
- `jetthing-core`: application state, tool window layout model (one visible tool window
  per anchor), the problems model (`problems`: syntax errors and LSP diagnostics merged
  into per-file problems with severities, counts and ancestor error propagation), and
  built-in command handlers.
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

## Toolchain provisioning

`jetthing-toolchains` is the single place that finds or installs the external programs a
language needs. Other subsystems (LSP registry, build/run providers, Android, Bazel,
debuggers) do not search `PATH` themselves; they ask the resolver:

```rust
let toolchains: &Toolchains = &state.toolchains.toolchains; // jetthing-core
toolchains.executable("jdtls");            // Option<PathBuf>
toolchains.locate("jdk");                  // Option<Located { executable, home, version, source }>
toolchains.missing(&["jdtls"]);            // ["jdtls", "jdk"] incl. `requires`, when not found
toolchains.environment(&["jdk", "jdtls"]); // PATH prefix + JAVA_HOME for child processes
```

Lookup order: `path` configured in `toolchains.toml` → managed install matching the project
pin (when a version is pinned) → home variable (`JAVA_HOME`, `ANDROID_HOME`) → `PATH` → newest
managed install. Pins come from `.jetthing/toolchains.toml` first, then `rust-toolchain(.toml)`,
`.bazelversion`, Gradle `JavaLanguageVersion.of(n)`/`jvmToolchain(n)`,
`gradle/gradle-daemon-jvm.properties`, `.sdkmanrc`, `.nvmrc`/`.node-version`,
`.python-version` and `go.mod`.

Language bundles describe components with a `ComponentSpec` (id, executables, bin dirs, home
variables, required components) and resolve versions to a `Release` (URL, SHA-256 or checksum
URL, archive kind) by implementing `ComponentProvider`; `StaticProvider` and `IndexProvider`
(JSON index, e.g. a corporate mirror) cover simple sources. Nothing downloads without consent:
installs start only from the `toolchain.install` command. Downloads resume with `Range`
requests, are verified before extraction (a mismatch deletes the file), extract into
`.staging/` and are renamed into place, so a version directory is either complete or absent.

Commands: `toolchain.list`, `toolchain.locate`, `toolchain.missing`, `toolchain.install`
(`wait: true` blocks, for the CLI), `toolchain.cancel`, `toolchain.jobs`, `toolchain.uninstall`,
`toolchain.usePath` ("use existing installation", stored in the user `toolchains.toml`) and
`toolchain.pins`. Running installs show in the status bar with a progress bar; clicking it cancels.
`JETTHING_TOOLCHAINS_DIR` overrides the install root.

## Command contract

Commands have stable IDs, human labels, JSON schemas expressed as documentation,
capability requirements, an enabled predicate, and an execution handler. GUI and CLI
use the same registry, and the control socket's `execute` request reaches every command of
a running workbench. Cancellation/progress and MCP transports are next-step extensions of
this contract.

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
