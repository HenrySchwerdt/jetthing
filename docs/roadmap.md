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
- [x] Detected Cargo/Gradle/Maven/npm/Go builds with streamed output and cancellation
- [ ] UI automation/control socket and screenshot tests

## M2 — language intelligence

VFS snapshots, background indexing, LSP transport, diagnostics, completion, symbol
navigation, formatting, cancellation, and persistent caches. First verified integration:
Rust via rust-analyzer.

- [x] JSON-RPC/LSP process transport and initialization
- [x] Rust document open/change synchronization and published diagnostics
- [x] Completion request transport and result modeling
- [ ] Completion popup/caret integration, hover, navigation, formatting, and code actions

## M3 — professional workflows

Git, search/replace, build/run configurations, test UI, DAP debugger, terminal,
workspace layout persistence, plugin manager, and recovery.

- [x] Repository detection, branch/status view, stage, and unstage
- [ ] Diff editor, history, blame, commit, branches, fetch/pull/push, and worktrees
- [x] Universal process runner and detected project build command
- [ ] Persistent run configurations, test tree, debug-before-run, and compound tasks

## M4 — web/JVM/Android bundles

Product-neutral bundles start with Web and Rust, then Java/Kotlin/Gradle and Android.
Use installed SDK, ADB, emulator, and Gradle; do not depend on Android Studio.

## Next task

Build the viewport-native editor surface on the rope engine so only visible lines are
shaped and painted. Add cursor/selection primitives, typing coalescence, multi-cursor
transactions, and incremental Tree-sitter edit notifications.
