# Debugger (DAP)

Captured 2026-10-10 under Xvfb (1600x1000, `JETTHING_RENDERER=glow`,
`JETTHING_NO_UPDATE_CHECK=1`, debug build), driven through the control socket
(`jetthing-cli ui execute …`). Adapter: `lldb-dap-18`. Sample: a 15-line Cargo binary
(`square(n)` called from a loop in `main`) with a breakpoint on line 2.
Reference: `reference/android-studio/23-debugger.png`.

| File | Steps → result |
|---|---|
| `debugger/23-debugger-breakpoint.png` | `debug.start` builds with `cargo build`, launches through lldb-dap and stops at the line breakpoint: execution-line highlight and arrow in the gutter, Debug tool window with status `"dbgsample"@16357: suspended (breakpoint 1.1)`, frames list, Locals (`n = {int} 1`), Globals, Registers and the evaluate field |
| `debugger/23-debugger-step-over.png` | `debug.stepOver` (F8): line 2 → 4, status `suspended (step over)`, frame `main.rs:4` |

Breakpoint and execution-line colours come from the theme engine
(`BREAKPOINT_ATTRIBUTES` / `EXECUTIONPOINT_ATTRIBUTES` of the active colour scheme; the dot
uses the `ERROR` token, the arrow `ACCENT`).

Found while capturing: `cargo_package_name` parsed Cargo.toml with `toml::Value::from_str`
(single value in toml 0.9), so Debug on a Cargo project without a selected configuration
always failed with "cannot tell which binary to debug". Fixed, with a test.

## Remaining differences from the reference

- Java debugging works against real jdtls (see below); the remaining items apply to it as well.
- No Breakpoints dialog (Ctrl+Shift+F8 edits the breakpoint at the caret), no suspend policy, Smart Step Into or Quick Evaluate.
- The tool window has no separate Threads and Variables panes, only one combined view.
- Locals only list the adapter's `Locals` scope; Rust-aware pretty printing is lldb's.

## Java (jdtls + java-debug)

`debug.start` on a Gradle configuration (or the gutter `Debug 'App.main()'` entry, Shift+F9) runs
`vscode.java.buildWorkspace`, `vscode.java.resolveMainClass`, `vscode.java.resolveClasspath` and
`vscode.java.startDebugSession` through jdtls, then connects `jetthing-dap` to the returned
localhost port and launches the main class. The `java-debug` toolchain component (Maven Central,
`com.microsoft.java.debug.plugin`) is installed with jdtls and passed in `initializationOptions.bundles`.

Reproducible evidence: `cargo test -p jetthing-core --test java_debug_flow` (scripted jdtls and DAP
server: start, breakpoint stop, execution point) and `cargo test -p jetthing-dap --test java_tcp`.


### Live Java verification (#146, 2026-10-11)

Real jdtls 1.61.0 + java-debug 0.53.1, installed by `toolchain.setup` into an empty `$HOME`, Gradle
Java sample (`App.square(int)` called from a loop in `main`, breakpoint on line 5), Xvfb 1600x1000.

| File | Steps → result |
|---|---|
| `debugger/146-java-breakpoint.png` | `debug.start` → build, jdtls resolves main class and classpath, java-debug starts a DAP session; stops at the breakpoint (`"Thread [main]"@1: suspended (breakpoint)`), frames `App.square(int)` / `App.main(String[])`, local `n = {int} 1` |
| `debugger/146-java-step-over.png` | Step Over: line 5 → 6, `suspended (step)`, locals now include `result = {int} 1` |
| `debugger/146-java-second-stop.png` | Step Into / Out then Resume: stops at the breakpoint again with the next `n` |
| `debugger/146-java-shift-f9.png` | `debug.stop`, then the key chord Shift+F9 starts a fresh session and stops at the breakpoint |

Found and fixed: the `java-debug` install failed (`…0.53.1.jar.sha256` → HTTP 404) because Maven Central
publishes only `.sha1`/`.md5` for it, so the debugger could never be provisioned. The provider now pins the
SHA-256 of 0.53.1 (computed from a download whose SHA-1 matched Central's `.sha1`). Central rate-limited
(HTTP 429) once during capture; the download was retried with backoff.


## Watches, conditions, logpoints, Evaluate, inline values, Run to Cursor (#19)

Protocol-level evidence (scripted adapter, no I/O): `cargo test -p jetthing-dap session::` covers
conditions/logpoints/hit counts in `setBreakpoints`, watch evaluation on each stop (`context: watch`),
Run to Cursor (temporary breakpoint removed at the next stop), Force Step Into (`stepInTargets` →
`stepIn` + `targetId`, plain `stepIn` fallback) and `attach` sessions. Core tests:
`cargo test -p jetthing-core debug_commands inline_values java_debug`.

Screenshots (Xvfb 1600x1000, `JETTHING_RENDERER=glow`, debug build, gdb DAP adapter, a small C
program with `square(int)` called from a loop; shell run configuration `./app`):

| File | Steps → result |
|---|---|
| `debugger/19-breakpoint-properties.png` | right-click the line number of a breakpoint → properties popup with Condition `i == 3`, Log message and Hit count |
| `debugger/19-watches-inline-values.png` | stopped at the conditional breakpoint: Watches `total`, `i * 2` evaluated in the Variables pane and `name = value` hints at the end of the lines of the current function |
| `debugger/19-evaluate-dialog.png` | Alt+F8: Evaluate Expression dialog with the result of `total + i` |
| `debugger/19-run-to-cursor.png` | `debug.runToCursor` (caret on line 15, past the loop): execution moves there; the watch `i * 2` shows the out-of-scope error. Alt+F9 is the default chord only in some keymaps (the XWin keymap used here binds another one) |

JUnit: the gutter Debug entry of a Gradle test runs `test --tests … --debug-jvm` and attaches
java-debug to the test JVM. Verified at the unit level only (banner parsing, `attach` request); no
live jdtls run in this session.
