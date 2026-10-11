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

- Java debugging (jdtls java-debug) is not implemented; see the TODO in `jetthing-dap`.
- No Breakpoints dialog, conditional breakpoints, logpoints, watches, inline values or run to cursor.
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

Not verified: a live run against real jdtls + java-debug and Xvfb screenshots. The sandbox that built
this change could not download the plugin (Maven Central rate limit), so no `*-fixed-*.png` was captured.
