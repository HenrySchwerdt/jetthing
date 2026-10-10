# Typing latency on a 50,000-line file

Target (editor P0 workstream): a keystroke frame takes < 16 ms on a 50k-line file, matching the
reference IDE's rule that typing is never blocked by background analysis (`PLATFORM-PERF-008`).

Reproduce: `cargo bench -p jetthing-ui --bench typing` (release). The bench opens a generated
50,000-line Java class as a project file, drives `IdeApp::frame` on a headless `egui::Context`
(input handling, editing, analysis refresh, LSP sync attempt, layout of visible lines; no GPU
painting) and types 200 characters below the first screen. It asserts that the keystrokes reached
the buffer and exits non-zero when p95 exceeds 16 ms.

| Date | Build | File | p50 | p95 | max | Notes |
|---|---|---|---:|---:|---:|---|
| 2026-10-10 | before (main + #96) | `Big.txt` | 1.35 ms | 1.86 ms | 3.57 ms | status bar copied and scanned the whole buffer every frame |
| 2026-10-10 | this change | `Big.txt` | 0.70 ms | 1.70 ms | 3.83 ms | bounded indent/line-separator detection; cached find matches |
| 2026-10-10 | this change | `Big.java` | 358.8 ms | 405.5 ms | 428.5 ms | whole-file tree-sitter analysis per keystroke (~430 ms standalone) |

Machine: shared Linux CI container; absolute numbers vary between runs, the ratio between the
two files does not. The Java row is the remaining gap; it is expected to drop once incremental
reparse (#72) lands.
