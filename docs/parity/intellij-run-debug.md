# IntelliJ IDEA parity: run, debug, and test

Run/debug configurations, the Run and Debug tool windows, breakpoints, stepping,
evaluation, test runners, and coverage.

Sources: IntelliJ IDEA public documentation (<https://www.jetbrains.com/help/idea/>), the predefined Windows
keymap reference (<https://www.jetbrains.com/help/idea/reference-keymap-win-default.html>) and the unified-distribution FAQ (<https://lp.jetbrains.com/intellij-idea-unified-faq/>), which lists
the features that need an Ultimate subscription. Rows were written from public
documentation and observable behavior only; no JetBrains code, artwork, or branding
is reused.

Rows tagged `[Ultimate]` require an IntelliJ IDEA Ultimate subscription. Untagged rows are
available in the free distribution. `Weight` (1-5) is the relative importance used by the
weighted dashboard: 5 = core daily workflow, 3 = significant, 2 = moderate, 1 = minor.
Statuses describe Jetthing, conservatively: nothing in this catalog is `Verified`.

| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
| IJ-RUN-001 | [run-debug-configuration](https://www.jetbrains.com/help/idea/run-debug-configuration.html) | Run/Debug Configurations dialog | Missing | 5 | — | — | not started |
| IJ-RUN-002 | [run-debug-configuration](https://www.jetbrains.com/help/idea/run-debug-configuration.html) | Temporary configurations created from context (Ctrl+Shift+F10) | Missing | 5 | — | — | not started |
| IJ-RUN-003 | [run-debug-configuration](https://www.jetbrains.com/help/idea/run-debug-configuration.html) | Configuration templates and stored (shared) configurations | Missing | 2 | — | — | not started |
| IJ-RUN-004 | [program-arguments-and-environment-variables](https://www.jetbrains.com/help/idea/program-arguments-and-environment-variables.html) | Program arguments, VM options, environment variables, working directory | Missing | 3 | — | — | not started |
| IJ-RUN-005 | [run-debug-configuration](https://www.jetbrains.com/help/idea/run-debug-configuration.html) | Before launch tasks (build, Gradle task, external tool) | Missing | 2 | — | — | not started |
| IJ-RUN-006 | [run-debug-multiple](https://www.jetbrains.com/help/idea/run-debug-multiple.html) | Compound configurations / run multiple | Missing | 1 | — | — | not started |
| IJ-RUN-007 | [list-of-run-debug-configurations](https://www.jetbrains.com/help/idea/list-of-run-debug-configurations.html) | Configuration types: Application, JAR, Gradle, Maven, JUnit, Shell, npm | Missing | 3 | — | — | not started |
| IJ-RUN-008 | [run-targets](https://www.jetbrains.com/help/idea/run-targets.html) | Run targets (Docker, SSH, WSL) | Missing | 1 | — | — | not started |
| IJ-RUN-009 | [shebang-scripts](https://www.jetbrains.com/help/idea/shebang-scripts.html) | Run shebang scripts | Missing | 1 | — | — | not started |
| IJ-RUN-010 | [running-applications](https://www.jetbrains.com/help/idea/running-applications.html) | Run detected project build/run task | Partial | 5 | `build.project`, `run.task`, `run.cancel` | real process/output/cancellation tests | one detected plan per project (Cargo/Gradle/Maven/npm/Go); no configuration model |
| IJ-RUN-011 | [running-applications](https://www.jetbrains.com/help/idea/running-applications.html) | Gutter run icons for main methods and tests | Missing | 3 | — | — | not started |
| IJ-RUN-012 | [running-applications](https://www.jetbrains.com/help/idea/running-applications.html) | Rerun (Ctrl+F5), Stop (Ctrl+F2), Stop background processes | Partial | 3 | `run.cancel` | real process/output/cancellation tests | cancel only; no rerun |
| IJ-RUN-020 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Console output streaming with stdout/stderr distinction | Partial | 5 | `run.task` | real process/output/cancellation tests | streamed lines; no stderr coloring or ANSI |
| IJ-RUN-021 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Console stdin input | Missing | 2 | — | — | not started |
| IJ-RUN-022 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Clickable stack traces and file links in console | Missing | 3 | — | — | not started |
| IJ-RUN-023 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Multiple run tabs, pin tab, soft wrap, scroll to end, clear | Missing | 2 | — | — | not started |
| IJ-RUN-024 | [setting-log-options](https://www.jetbrains.com/help/idea/setting-log-options.html) | Log files in run console | Missing | 1 | — | — | not started |
| IJ-RUN-025 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Exit code and elapsed time reporting | Partial | 2 | `run.task` | real process/output/cancellation tests | exit status recorded; no elapsed time |
| IJ-RUN-030 | [starting-the-debugger-session](https://www.jetbrains.com/help/idea/starting-the-debugger-session.html) | Start debug session (Shift+F9) | Missing | 5 | — | — | not started |
| IJ-RUN-031 | [attach-to-process](https://www.jetbrains.com/help/idea/attach-to-process.html) | Attach to process (Ctrl+Alt+F5) / remote JVM debug | Missing | 2 | — | — | not started |
| IJ-RUN-032 | [using-breakpoints](https://www.jetbrains.com/help/idea/using-breakpoints.html) | Line breakpoints (Ctrl+F8) | Missing | 5 | — | — | not started |
| IJ-RUN-033 | [using-breakpoints](https://www.jetbrains.com/help/idea/using-breakpoints.html) | Conditional breakpoints, hit counts, suspend policy | Missing | 3 | — | — | not started |
| IJ-RUN-034 | [logpoints](https://www.jetbrains.com/help/idea/logpoints.html) | Logpoints / non-suspending breakpoints | Missing | 2 | — | — | not started |
| IJ-RUN-035 | [using-breakpoints](https://www.jetbrains.com/help/idea/using-breakpoints.html) | Method, field watchpoint, exception breakpoints | Missing | 2 | — | — | not started |
| IJ-RUN-036 | [using-breakpoints](https://www.jetbrains.com/help/idea/using-breakpoints.html) | Temporary breakpoints, dependent breakpoints | Missing | 1 | — | — | not started |
| IJ-RUN-037 | [using-breakpoints](https://www.jetbrains.com/help/idea/using-breakpoints.html) | View Breakpoints dialog (Ctrl+Shift+F8) | Missing | 2 | — | — | not started |
| IJ-RUN-038 | [stepping-through-the-program](https://www.jetbrains.com/help/idea/stepping-through-the-program.html) | Step Over / Into / Out (F8 / F7 / Shift+F8) | Missing | 5 | — | — | not started |
| IJ-RUN-039 | [stepping-through-the-program](https://www.jetbrains.com/help/idea/stepping-through-the-program.html) | Smart Step Into, Force Step, Run to Cursor (Alt+F9) | Missing | 2 | — | — | not started |
| IJ-RUN-040 | [examining-suspended-program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) | Frames and threads views | Missing | 5 | — | — | not started |
| IJ-RUN-041 | [examining-suspended-program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) | Variables view with object inspection | Missing | 5 | — | — | not started |
| IJ-RUN-042 | [examining-suspended-program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) | Watches | Missing | 3 | — | — | not started |
| IJ-RUN-043 | [examining-suspended-program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) | Evaluate Expression (Alt+F8) / Quick Evaluate | Missing | 3 | — | — | not started |
| IJ-RUN-044 | [examining-suspended-program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) | Inline variable values in editor | Missing | 3 | — | — | not started |
| IJ-RUN-045 | [tutorial-set-value](https://www.jetbrains.com/help/idea/tutorial-set-value.html) | Set value of a variable | Missing | 1 | — | — | not started |
| IJ-RUN-046 | [altering-the-program-s-execution-flow](https://www.jetbrains.com/help/idea/altering-the-program-s-execution-flow.html) | Drop frame / reset frame, force return, throw exception | Missing | 1 | — | — | not started |
| IJ-RUN-047 | [altering-the-program-s-execution-flow](https://www.jetbrains.com/help/idea/altering-the-program-s-execution-flow.html) | HotSwap (reload changed classes) | Missing | 2 | — | — | not started |
| IJ-RUN-048 | [customizing-views](https://www.jetbrains.com/help/idea/customizing-views.html) | Data renderers and type renderers | Missing | 1 | — | — | not started |
| IJ-RUN-049 | [analyze-objects-in-the-jvm-heap](https://www.jetbrains.com/help/idea/analyze-objects-in-the-jvm-heap.html) | Memory view: object counts, instances | Missing | 1 | — | — | not started |
| IJ-RUN-050 | [analyze-java-stream-operations](https://www.jetbrains.com/help/idea/analyze-java-stream-operations.html) | Java Stream debugger (trace current stream chain) | Missing | 1 | — | — | not started |
| IJ-RUN-051 | [debug-asynchronous-code](https://www.jetbrains.com/help/idea/debug-asynchronous-code.html) | Async stack traces, coroutine debugging | Missing | 2 | — | — | not started |
| IJ-RUN-052 | [thread-dumps](https://www.jetbrains.com/help/idea/thread-dumps.html) | Get thread dump | Missing | 1 | — | — | not started |
| IJ-RUN-053 | [detect-concurrency-issues](https://www.jetbrains.com/help/idea/detect-concurrency-issues.html) | Detect concurrency issues (thread suspension policies) | Missing | 1 | — | — | not started |
| IJ-RUN-054 | [monitor-debugger-overhead](https://www.jetbrains.com/help/idea/monitor-debugger-overhead.html) | Debugger overhead monitor | Missing | 1 | — | — | not started |
| IJ-RUN-055 | [debugging-code](https://www.jetbrains.com/help/idea/debugging-code.html) | Debug Adapter Protocol backends for non-JVM languages | Missing | 3 | `debug.start` | — | command name reserved in Android inventory; DAP design pending |
| IJ-RUN-060 | [testing](https://www.jetbrains.com/help/idea/testing.html) | Run tests from gutter, file, directory, or package | Missing | 5 | — | — | not started |
| IJ-RUN-061 | [viewing-and-exploring-test-results](https://www.jetbrains.com/help/idea/viewing-and-exploring-test-results.html) | Test results tree with pass/fail/ignored and durations | Missing | 5 | — | — | not started |
| IJ-RUN-062 | [viewing-and-exploring-test-results](https://www.jetbrains.com/help/idea/viewing-and-exploring-test-results.html) | Rerun failed tests, toggle auto-test, filter passed | Missing | 3 | — | — | not started |
| IJ-RUN-063 | [viewing-and-exploring-test-results](https://www.jetbrains.com/help/idea/viewing-and-exploring-test-results.html) | Comparison failure diff (expected vs actual) | Missing | 2 | — | — | not started |
| IJ-RUN-064 | [create-tests](https://www.jetbrains.com/help/idea/create-tests.html) | Create test for class | Missing | 2 | — | — | not started |
| IJ-RUN-065 | [junit](https://www.jetbrains.com/help/idea/junit.html) | JUnit 4/5 runner | Missing | 5 | — | — | not started |
| IJ-RUN-066 | [testng](https://www.jetbrains.com/help/idea/testng.html) | TestNG runner | Missing | 1 | — | — | not started |
| IJ-RUN-067 | [spock](https://www.jetbrains.com/help/idea/spock.html) | Spock runner | Missing | 1 | — | — | not started |
| IJ-RUN-068 | [performing-tests](https://www.jetbrains.com/help/idea/performing-tests.html) | Test history and import of test results | Missing | 1 | — | — | not started |
| IJ-RUN-069 | [code-coverage](https://www.jetbrains.com/help/idea/code-coverage.html) | Run with coverage and editor coverage highlighting | Missing | 2 | — | — | not started |
| IJ-RUN-080 | [profiler-intro](https://www.jetbrains.com/help/idea/profiler-intro.html) | [Ultimate] IntelliJ Profiler: CPU and allocation profiling | Missing | 2 | — | — | not started |
| IJ-RUN-081 | [cpu-and-memory-live-charts](https://www.jetbrains.com/help/idea/cpu-and-memory-live-charts.html) | [Ultimate] CPU and memory live charts | Missing | 1 | — | — | not started |
| IJ-RUN-082 | [read-the-profiling-report](https://www.jetbrains.com/help/idea/read-the-profiling-report.html) | [Ultimate] Flame graph, call tree, method list views | Missing | 1 | — | — | not started |
| IJ-RUN-083 | [create-a-memory-snapshot](https://www.jetbrains.com/help/idea/create-a-memory-snapshot.html) | [Ultimate] Memory snapshots and analysis | Missing | 1 | — | — | not started |
| IJ-RUN-084 | [compare-profiler-snapshots](https://www.jetbrains.com/help/idea/compare-profiler-snapshots.html) | [Ultimate] Compare profiler snapshots | Missing | 1 | — | — | not started |
