# Platform parity

| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
| PLAT-001 | IntelliJ Platform | Native application window | Implemented | 3 | `app.quit` | manual launch | macOS tested; packaging pending |
| PLAT-002 | IntelliJ Platform | Project opening | Implemented | 3 | `project.open` | command unit test | startup path/CLI; picker pending |
| PLAT-003 | IntelliJ Platform | Project tree | Partial | 3 | `project.open`, `editor.openFile` | project tree unit test | lazy refresh/watchers pending |
| PLAT-004 | IntelliJ Platform | Editable tabs | Partial | 5 | `editor.openFile`, `editor.save`, `editor.saveAll`, `editor.undo`, `editor.redo` | rope unit/property tests | viewport-native multi-cursor editing; IME, clipboard, and advanced movement pending |
| PLAT-005 | IntelliJ Platform | Command search | Implemented | 3 | `command.palette` | registry unit tests | fuzzy ranking pending |
| PLAT-006 | IntelliJ Platform | Plugin discovery | Partial | 3 | `plugin.refresh` | manifest tests | manifests only; no execution |
| PLAT-007 | IntelliJ Platform | Dockable tool windows | Missing | 3 | — | — | planned M3 |
| PLAT-008 | IntelliJ Platform | Integrated terminal | Partial | 3 | `terminal.open`, `terminal.toggle`, `terminal.sendText` | real PTY process test | transcript UI; ANSI grid and multiple sessions pending |
| PLAT-009 | IntelliJ Platform | Persistent indexing | Missing | 5 | — | — | planned M2 |
| PLAT-010 | IntelliJ Platform | CLI command surface | Implemented | 1 | registry | CLI smoke invocation | JSON execution available |
| PLAT-011 | IntelliJ Platform | Syntax parsing/highlighting | Partial | 3 | editor edit pipeline | Rust parser tests | Rust only; full reparse pending incremental edits |
| PLAT-012 | IntelliJ Platform | Parse diagnostics | Partial | 2 | editor edit pipeline | malformed Rust test | syntax errors only; no semantic diagnostics |
| PLAT-013 | IntelliJ Platform | Find in Files | Partial | 3 | `search.project`, `search.cancel`, `editor.openFile` | async search tests | opens matching file; exact caret navigation pending |
| PLAT-014 | IntelliJ Platform | Git changes | Partial | 3 | `git.refresh`, `git.stage`, `git.unstage` | real repository integration test | status/staging only; diff/history/remotes pending |
| PLAT-015 | IntelliJ Platform | Build execution | Partial | 3 | `build.project`, `run.task`, `run.cancel` | real process/output/cancellation tests | detected single build plan; configurations/test tree pending |
| PLAT-016 | IntelliJ Platform | Language server integration | Partial | 5 | `language.complete`, `language.completion` | framed fake-server integration test | Rust diagnostics and completion popup work; filtering, snippets, resolve, and navigation remain |

Statuses are intentionally conservative. “Implemented” means the listed scope works;
only “Verified” will count as independently parity-validated.
