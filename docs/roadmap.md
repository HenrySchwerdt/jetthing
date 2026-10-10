# Roadmap

## M0 — repository and contracts (complete)

- Modular workspace and enforced dependency direction by convention.
- Typed command registry shared by native UI and CLI.
- Honest parity inventory and generated summary command.
- Host/Android Studio inventory documented.

## M1 — usable native shell (in progress)

- [x] Native window and dark IDE layout
- [x] Open project from startup path
- [x] Bounded project tree
- [x] Real editable files in tabs
- [x] Save and Save All commands
- [x] Command palette
- [x] Manifest plugin discovery
- [ ] Rope-backed editor with multi-cursor/undo model
- [ ] Syntax highlighting and incremental Tree-sitter parsing
- [ ] Integrated PTY terminal
- [ ] UI automation/control socket and screenshot tests

## M2 — language intelligence

VFS snapshots, background indexing, LSP transport, diagnostics, completion, symbol
navigation, formatting, cancellation, and persistent caches. First verified integration:
Rust via rust-analyzer.

## M3 — professional workflows

Git, search/replace, build/run configurations, test UI, DAP debugger, terminal,
workspace layout persistence, plugin manager, and recovery.

## M4 — web/JVM/Android bundles

Product-neutral bundles start with Web and Rust, then Java/Kotlin/Gradle and Android.
Use installed SDK, ADB, emulator, and Gradle; do not depend on Android Studio.

## Next task

Replace the per-tab `String` with a rope document model containing revisions,
transactional edits, undo/redo, dirty tracking, and property tests. Keep the existing
command and UI APIs operational while doing so.

