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
- [x] Revisioned rope-backed text engine with Unicode-safe undo/redo
- [ ] Multi-cursor editor view and coalesced typing transactions
- [x] Tree-sitter Rust parsing, syntax highlighting, and parse diagnostics
- [ ] Incremental Tree-sitter edit notifications and multi-language grammars
- [x] Integrated native PTY terminal with shell input and command dispatch
- [ ] ANSI terminal grid, hyperlinks, selection, and multiple sessions
- [x] Cancellable background project text search with streamed result navigation
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

Build the viewport-native editor surface on the rope engine so only visible lines are
shaped and painted. Add cursor/selection primitives, typing coalescence, multi-cursor
transactions, and incremental Tree-sitter edit notifications.
