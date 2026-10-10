# Platform parity

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| PLAT-001 | IntelliJ Platform | Native application window | Implemented | `app.quit` | manual launch | macOS tested; packaging pending |
| PLAT-002 | IntelliJ Platform | Project opening | Implemented | `project.open` | command unit test | startup path/CLI; picker pending |
| PLAT-003 | IntelliJ Platform | Project tree | Partial | `project.open`, `editor.openFile` | project tree unit test | lazy refresh/watchers pending |
| PLAT-004 | IntelliJ Platform | Editable tabs | Partial | `editor.openFile`, `editor.save`, `editor.saveAll`, `editor.undo`, `editor.redo` | rope unit/property tests | plain text only; multi-cursor pending |
| PLAT-005 | IntelliJ Platform | Command search | Implemented | `command.palette` | registry unit tests | fuzzy ranking pending |
| PLAT-006 | IntelliJ Platform | Plugin discovery | Partial | `plugin.refresh` | manifest tests | manifests only; no execution |
| PLAT-007 | IntelliJ Platform | Dockable tool windows | Missing | — | — | planned M3 |
| PLAT-008 | IntelliJ Platform | Integrated terminal | Missing | — | — | planned M1 |
| PLAT-009 | IntelliJ Platform | Persistent indexing | Missing | — | — | planned M2 |
| PLAT-010 | IntelliJ Platform | CLI command surface | Implemented | registry | CLI smoke invocation | JSON execution available |
| PLAT-011 | IntelliJ Platform | Syntax parsing/highlighting | Partial | editor edit pipeline | Rust parser tests | Rust only; full reparse pending incremental edits |
| PLAT-012 | IntelliJ Platform | Parse diagnostics | Partial | editor edit pipeline | malformed Rust test | syntax errors only; no semantic diagnostics |

Statuses are intentionally conservative. “Implemented” means the listed scope works;
only “Verified” will count as independently parity-validated.
