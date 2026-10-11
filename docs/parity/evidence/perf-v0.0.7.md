# Performance check, v0.0.7 release candidate

Release build (`cargo build --release`), Xvfb 1440×900, `JETTHING_RENDERER=glow`, 4 vCPU
container, measured over the control socket on 2026-10-11. "Before" is `main` at
`1725ba7`; "after" is the same tree with the fixes in this PR. The container was also
compiling the test suite during the "before" run and idle during the "after" run, so the
wall-time gains are mostly the quieter machine, not the fixes;
the numbers are for spotting order-of-magnitude problems, not for regression gating.

Method (script: see "Reproduction"): wall time is measured around a control request plus
`wait idle` (input settled for 3 frames). CPU is the app process's own `utime+stime`
from `/proc`, so it excludes `rust-analyzer`, Gradle and the JVM. "Typing" is 20 single
characters at line 200 of a 1 MB file, each followed by `wait idle`; "scrolling" is 20
Page Downs the same way. The per-keystroke figure therefore includes the socket round
trip and three settled frames, roughly 20 ms of overhead.

| Measurement | Before | After |
|---|---:|---:|
| Startup to first control-socket reply (welcome screen) | 177 ms | 129 ms |
| RSS on the welcome screen | 145 MB | 146 MB |
| Idle CPU on the welcome screen | 0 % | 0 % |
| Open the generated Gradle project (4 files) | 172 ms | 55 ms |
| Idle CPU with the project open, `Main.java` active | 10 % | 11 % |
| RSS with the project open | 154 MB | 154 MB |
| Open a 1 MB Rust file (`editor.rs` ×9) | 844 ms | 427 ms |
| Typing, 1 MB Rust file: CPU per keystroke | 102 ms | 92 ms |
| Scrolling, 1 MB Rust file: CPU per Page Down | 108 ms | 110 ms |
| Idle CPU 1 s after settling with the 1 MB Rust file open | 69 % | 227 % (highlighter thread still running, see finding 1) |
| Open a 1 MB Java file (valid classes, ~2,900 of them) | 1,249 ms | 600 ms |
| Typing, 1 MB Java file: CPU per keystroke | 216 ms | 98 ms |
| Scrolling, 1 MB Java file: CPU per Page Down | 119 ms | 106 ms |
| Idle CPU 1 s after settling with the 1 MB Java file open | 44 % | 54 % |
| RSS after opening both 1 MB files | 211 MB | 210 MB |
| Search Everywhere, Double Shift + `main`, until results settle | 283 ms | 184 ms |
| Open this repository (~400 Rust files, rust-analyzer starts) | 539 ms | 339 ms |
| Idle CPU 1 s after settling with this repository open | 59 % | 3 % |
| RSS with this repository open | 227 MB | 224 MB |
| Search Everywhere in this repository, `editor` | 435 ms | 247 ms |

## Findings

1. **Idle CPU is not zero once a file or a real project is open.** With nothing happening
   the process still burns 40–70 % of a core. Two causes were traced in the per-frame
   path and are *not* fixed in this PR because they need a cache with an invalidation
   key, which is more than a night-before-release change:
   - the Project tree is rebuilt from the full file list on every frame
     (`jetthing-core/src/project_view.rs` `tree_rows`), and
   - the active file's problems, blame annotations, code vision and context-menu model
     are recomputed every frame (`jetthing-ui/src/editor.rs:184-188`).
   Both run at the caret-blink rate (2 Hz) when the editor has focus, at 10 Hz while a
   language server is busy, and at the display rate while the terminal timer runs.
   The cheap parts of this loop are fixed here: the HEAD-diff no longer copies the
   whole document out of the rope every frame, the LSP repaint timer no longer builds
   a `Vec` of server infos per frame, and the palette / AI commit-message repaints are
   bounded to 30 ms instead of running at the display rate.
2. **Typing in a 1 MB Java file costs about twice a Rust file.** Syntax analysis is
   incremental for both, so the difference is in the Java-specific per-edit work
   (problem index refresh, live-template context). A pathological file (1 MB of
   non-Java text in a `.java` file) was 3× slower again because of tree-sitter error
   recovery; that is an artefact of the fixture, not something users hit.
3. **Startup, project open, Search Everywhere and memory are fine** for this release:
   under 200 ms to the first frame, under 600 ms to open this repository, under 450 ms
   for a Search Everywhere query, and about 230 MB with the repository open.
4. No hang or stall over 2 s was observed in any step.
5. **Idle CPU snapshots vary with background work**: the 227 % reading with the 1 MB
   Rust file open was taken while the background highlighter was still working through
   the file and the editor repaints until the viewport is fully highlighted; the 3 %
   reading with this repository open was taken after rust-analyzer had already settled.
   The CPU-per-keystroke figures are the comparable ones and are unchanged within noise.

## Reproduction

```bash
Xvfb :99 -screen 0 1440x900x24 &
cargo build --release -p jetthing-app -p jetthing-cli
export DISPLAY=:99 JETTHING_RENDERER=glow JETTHING_NO_UPDATE_CHECK=1 JETTHING_CONTROL_SOCKET=/tmp/jt.sock
cli() { target/release/jetthing-cli ui --socket /tmp/jt.sock "$@"; }
t0=$(date +%s%N); target/release/jetthing-app & until cli state >/dev/null 2>&1; do sleep 0.02; done
echo "startup $(( ($(date +%s%N) - t0) / 1000000 )) ms"
PID=$(pgrep -n -x jetthing-app); cli wait idle
ticks() { awk '{print $14+$15}' /proc/$PID/stat; }      # 100 ticks = 1 s of CPU
c0=$(ticks); sleep 5; echo "idle cpu $(( ($(ticks) - c0) * 100 / 500 )) %"
cli execute project.open '{"path":"/path/to/project"}' && cli wait idle
cli execute editor.openFile '{"path":"big/big.rs"}' && cli wait idle
cli execute editor.gotoLine '{"line":200}'
c0=$(ticks); for i in $(seq 20); do cli type x >/dev/null; cli wait idle >/dev/null; done
echo "typing cpu/key $(( ($(ticks) - c0) * 10 / 20 )) ms"
awk '/VmRSS/ {print "rss", $2/1024, "MB"}' /proc/$PID/status
```
