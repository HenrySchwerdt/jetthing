# Java + Gradle feature inventory (target: IntelliJ IDEA 2026.2)

Status: research spec, clean-room. Every description below is written in our own words from public
documentation, observable behaviour, and facts (IDs, names, defaults, shortcuts) read from the
Apache-2.0 IntelliJ Community sources at tag `idea/262.10968.63`. No code, artwork, or documentation prose
is copied. Short UI labels are quoted as names only.

Jetthing models IntelliJ IDEA's Java/Gradle support as plugins on the platform core
(see `docs/architecture.md`): a **Java language plugin** (PSI, resolve, completion, inspections,
refactorings, generation, run/debug), a **Gradle/external-system plugin** (project model import,
sync, tool window, task execution) and a **JVM build plugin** (internal builder / compiler integration).
This file is the inventory those plugins must cover. It makes **no parity claim**; parity evidence lives
in `docs/parity/`.

## How to read this document

- **ID**: stable identifier. `GRADLE-NNN` for build-tool/project-model features, `JAVA-<AREA>-NNN` for
  everything else. IDs are never reused; deleted features are struck through, not renumbered.
- **Shortcut** columns give the default keymaps: **Win/Linux** = the `$default` keymap ("Windows" /
  "Linux" in the UI); **macOS** = the `Mac OS X 10.5+` keymap (shown as "macOS" in the UI). `—` means no
  default binding (the action is still reachable through Find Action, `Ctrl+Shift+A` / `Cmd+Shift+A`).
  macOS keys are written as `Cmd`, `Opt`, `Ctrl`, `Shift`.
- **Entry point**: main menu path (`Menu | Item`), tool window, popup, or gutter/editor affordance.
- **Pri**: `P0` = needed for a usable everyday Java + Gradle workflow; `P1` = expected by most users,
  ship soon after P0; `P2` = advanced / niche.
- **Paid**: `$` marks features that require an IntelliJ IDEA (Ultimate) subscription in the unified
  distribution (see [Community vs Ultimate](#community-vs-ultimate)). Unmarked = free core.
- "(inferred)" marks behaviour deduced from observation or from resource files rather than stated in
  documentation.
- Shortcut facts were resolved from the keymap XML files and the action registrations in the
  Community sources (`platform/platform-resources/src/keymaps/*.xml`, `*Actions.xml`); for macOS, actions
  that the macOS keymap does not override inherit the default binding with `Ctrl` mapped to `Cmd`
  (inferred from keymap inheritance rules; verify against the published macOS keymap PDF).

## Contents

1. [Community vs Ultimate](#community-vs-ultimate)
2. [Gradle project model, sync and tooling](#1-gradle-project-model-sync-and-tooling) (GRADLE)
3. [JDK management and language level](#2-jdk-management-and-language-level) (JAVA-JDK)
4. [Java editing](#3-java-editing) (JAVA-EDIT, JAVA-LT, JAVA-PFX, JAVA-FMT, JAVA-LANG, JAVA-LOMBOK)
5. [Navigation and search](#4-navigation-and-search) (JAVA-NAV)
6. [Inspections, intentions and quick-fixes](#5-inspections-intentions-and-quick-fixes) (JAVA-INSP)
7. [Refactorings](#6-refactorings) (JAVA-REF)
8. [Code generation](#7-code-generation) (JAVA-GEN)
9. [Run, test, debug, coverage, profiling](#8-run-test-debug-coverage-profiling) (JAVA-RUN, JAVA-TEST, JAVA-DBG, JAVA-COV, JAVA-PROF)
10. [Build](#9-build) (JAVA-BUILD)
11. [VCS touchpoints](#10-vcs-touchpoints-for-java-projects) (JAVA-VCS)
12. [Summary counts](#summary-counts)
13. [Open questions](#open-questions)

---

## Community vs Ultimate

Since release 2025.3, IntelliJ IDEA ships as a **single unified distribution**. There is no separate
"Community Edition" download any more: one IDE installs, and its core (Java, Kotlin, Groovy, Gradle,
Maven, the Java debugger, JUnit/TestNG runners, code coverage, VCS, basic database tools, and more)
works for free without a subscription. "Advanced tooling" (historically the Ultimate feature set) is
unlocked with a subscription, and a 30-day trial is offered. When a subscription lapses the free core
keeps working.

For this Java + Gradle inventory almost everything is in the free core. Items that require a paid
subscription (marked `$` in tables) are, to our knowledge:

| Feature | Paid? | Note |
|---|---|---|
| IntelliJ Profiler (CPU/allocation profiling, live charts, memory snapshots UI) | `$` (inferred) | Historically Ultimate-only; the unified-product page does not list it as free. See Open questions. |
| Spring / Spring Boot support, Spring debugger insights, Jakarta EE, Micronaut, Quarkus, Helidon | `$` | Framework support; out of scope here but they hook into Java run configurations. |
| HTTP Client, endpoints tool window | `$` (inferred) | Out of scope. |
| Full database tools (Database tool window beyond the basic set, SQL in Java strings with schema awareness) | partly `$` | Basic DB tools became free in 2025.3. |
| Remote development / Code With Me host features | `$` (inferred) | Platform, out of scope. |
| JVM debugger, HotSwap, coverage, JUnit/TestNG, Gradle, Maven, inspections, refactorings | free | Core. |
| AI Assistant / next-edit suggestions | separate licence | Not part of this inventory. |

Sources:
- https://www.jetbrains.com/help/idea/intellij-idea-single-distribution.html
- https://lp.jetbrains.com/intellij-idea-unified-faq/
- https://www.jetbrains.com/idea/whatsnew/2025-3/
- https://www.jetbrains.com/idea/features/

---

## 1. Gradle project model, sync and tooling

Terminology note: in 2026.x the UI wording moved from "reload/re-import" to **"sync"**
(e.g. "Sync Gradle Changes", "Sync All Gradle Projects", "Auto-Sync Settings…"). Older help pages still
say "Load Gradle Changes" / "Reload All Gradle Projects". Jetthing should use the "sync" vocabulary and
treat the others as aliases in Find Action.

### 1.1 Opening, importing, linking

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-001 | Open Gradle project | Opening a directory (or a `build.gradle`, `build.gradle.kts`, `settings.gradle(.kts)` file) that contains Gradle build files detects it as a Gradle project, links it and starts a sync that builds the IDE project model. | — | — | File \| Open…; Welcome screen \| Open | P0 |
| GRADLE-002 | "Open as project" choice | When a build file is opened and several build systems are detected (e.g. both `pom.xml` and `build.gradle`), the user is asked which model to use. | — | — | Open dialog follow-up popup | P1 |
| GRADLE-003 | Trust project prompt | Before running any build logic of a newly opened project the IDE asks whether the project is trusted; untrusted projects open in safe mode without executing Gradle. | — | — | Modal on open | P0 |
| GRADLE-004 | New Gradle project wizard | Creates a Java project with Gradle as build system: JDK, Gradle DSL (Kotlin or Groovy), optional sample code, group/artifact/version, Gradle distribution and version, wrapper generation, JUnit 5 test setup by default. | — | — | File \| New \| Project…; Welcome screen \| New Project | P0 |
| GRADLE-005 | New Gradle module | Adds a sub-project to an existing build (creates directory, build script and an `include(...)` entry in settings). | — | — | File \| New \| Module… | P1 |
| GRADLE-006 | Link Gradle Project | Attaches an additional Gradle build (another root build script) to the open IDE project; several linked builds may coexist. Action id `Gradle.ImportExternalProject`. | — | — | Gradle tool window toolbar "+"; context menu on build file "Link Gradle Project" / "Import Gradle Project" | P1 |
| GRADLE-007 | Unlink Gradle Project | Detaches a linked build; a dialog lets the user keep or remove the derived modules. | Delete (in tool window) | Delete | Gradle tool window toolbar / context menu | P1 |
| GRADLE-008 | Ignore / Unignore Gradle project | Marks a (sub)project as ignored: it stays visible but is excluded from import (e.g. `buildSrc`). | — | — | Gradle tool window context menu | P2 |
| GRADLE-009 | Orphan modules dialog | After sync removes modules that previously existed, offers to restore modules that carried user settings. | — | — | Modal after sync | P2 |
| GRADLE-010 | Convert plain project to Gradle | Adding build/settings scripts to an unlinked project triggers a "Gradle build script found" notification offering to link it. | — | — | Editor notification / balloon | P1 |
| GRADLE-011 | Gradle Project Data To Import | Selects which modules/data of a linked build are imported (partial import for large multi-project builds); has Select All, Unselect All, Show Selected Only, Select Required. Action id `ExternalSystem.SelectProjectDataToImport`. | — | — | Gradle tool window toolbar | P2 |

### 1.2 Sync / reload

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-020 | Initial sync on open | Runs Gradle (Tooling API) to fetch the build model: projects, source sets, dependencies, Java toolchain/language level, tasks; maps it into modules, content roots, libraries and SDKs. | — | — | Automatic | P0 |
| GRADLE-021 | Floating "Sync Gradle Changes" button | When build scripts, `gradle.properties`, wrapper properties or version catalogs change in the editor, a floating button appears in the editor's top-right corner; clicking it syncs. Action id `ExternalSystem.ProjectRefreshAction`; it can be dismissed (`ExternalSystem.HideProjectRefreshAction`). Older label: "Load Gradle Changes". | Ctrl+Shift+O | Cmd+Shift+I | Floating editor toolbar; Find Action | P0 |
| GRADLE-022 | Sync All Gradle Projects | Re-syncs every linked Gradle build. Action id `ExternalSystem.RefreshAllProjects` (label "Sync All Gradle Projects"; older "Reload All Gradle Projects"). | — | — | Gradle tool window toolbar | P0 |
| GRADLE-023 | Sync single Gradle project | Re-syncs only the selected linked build. Action id `ExternalSystem.RefreshProject`. | — | — | Gradle tool window context menu "Sync Gradle Project"; context menu on build script | P1 |
| GRADLE-024 | Auto-sync settings | Settings page with "Sync project after changes in the build scripts", choices **Any changes** vs **External changes** (only VCS / outside-IDE changes); enabled by default with External changes (inferred default). Action `ExternalSystem.ShowCommonSettings` ("Auto-Sync Settings…"). | — | — | Settings \| Build, Execution, Deployment \| Build Tools; Gradle tool window settings menu | P0 |
| GRADLE-025 | Sync progress and errors in Build tool window | Sync runs as a background task with a "Sync" tab in the Build tool window: tree of phases, console output, warnings, errors with links to the script line, and quick-fix links (e.g. "Download Gradle JDK", "Upgrade wrapper"). | Alt+4 shows Run; Build has no default shortcut | — | Build tool window \| Sync tab; status bar | P0 |
| GRADLE-026 | Sync failure notifications with fixes | Well-known failures (incompatible JDK/Gradle version, missing wrapper, unresolved dependency, offline cache miss, proxy issues) are recognised and offered fixes. | — | — | Build tool window, balloon | P1 |
| GRADLE-027 | Refresh dependencies | Re-sync forcing Gradle to re-resolve dynamic/changing dependencies (`--refresh-dependencies`). Action id `Gradle.RefreshDependencies`. | — | — | Gradle tool window context menu | P2 |
| GRADLE-028 | Download Sources | Downloads sources jars for all libraries (sources are normally downloaded on demand when opening a decompiled class; since 2026.1 sources/docs may download automatically). Action id `Gradle.DownloadSources`. | — | — | Gradle tool window toolbar; editor banner on decompiled class "Download Sources" | P1 |
| GRADLE-029 | Parallel model fetch | Option to fetch Gradle models in parallel for Gradle 7.4+ (setting "Enable parallel Gradle model fetching"). | — | — | Settings \| … \| Build Tools \| Gradle | P2 |
| GRADLE-030 | Execute task before/after sync | Gradle tasks can be bound to run before or after sync (task activation). Action ids `ExternalSystem.BeforeSync`, `ExternalSystem.AfterSync`. | — | — | Gradle tool window task context menu | P2 |
| GRADLE-031 | Declarative Gradle sync (experimental) | Support for the experimental Declarative Gradle (`.gradle.dcl`) project model and its sync (bundled module `gradle-declarative-sync`). | — | — | Automatic when DCL files present | P2 |

### 1.3 Project model mapping

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-040 | Module per Gradle project | Every Gradle (sub)project becomes an IDE module group named after its path (`root`, `root.app`, …). | — | — | Project Structure \| Modules | P0 |
| GRADLE-041 | Module per source set | Each source set (`main`, `test`, custom ones like `integrationTest`) becomes its own module (`root.app.main`, `root.app.test`) with its own dependencies, output and roots; the legacy "single module per project" mode is deprecated and kept only for troubleshooting. | — | — | Project Structure; Project tool window | P0 |
| GRADLE-042 | Content roots and root types | `src/<set>/java` → Sources/Test Sources, `src/<set>/resources` → Resources/Test Resources, `build/generated/...` → Generated Sources, `build` and `.gradle` excluded. | — | — | Project tool window icons | P0 |
| GRADLE-043 | Test source-set detection | Source sets used by `Test` tasks (or by the `jvm-test-suite` plugin) are marked as test roots. | — | — | Automatic | P0 |
| GRADLE-044 | Dependency scopes mapping | Gradle configurations map to module dependency scopes: `implementation`/`api` → Compile, `compileOnly` → Provided, `runtimeOnly` → Runtime, `testImplementation` → Test (on the test module), `annotationProcessor` → annotation processor path. | — | — | Project Structure \| Modules \| Dependencies | P0 |
| GRADLE-045 | Inter-project dependencies | `project(":lib")` dependencies become module dependencies (source navigation instead of jar). Test-fixtures dependencies supported. | — | — | Automatic | P0 |
| GRADLE-046 | External libraries | Resolved artifacts become project-level libraries named `Gradle: group:artifact:version`, with sources/javadoc attached when available. | — | — | Project tool window \| External Libraries | P0 |
| GRADLE-047 | Composite builds | Builds included via `includeBuild` (Gradle 4.5.1+ for IDE support) are linked; substituted dependencies become module dependencies. Composite participants can also be chosen in a dialog. Action id `Gradle.OpenProjectCompositeConfiguration`. | — | — | Automatic; Gradle tool window context menu "Composite Build Configuration" | P1 |
| GRADLE-048 | buildSrc support | `buildSrc` is imported as its own build with its own module(s); its classes resolve in build scripts. | — | — | Automatic | P1 |
| GRADLE-049 | Included plugin builds (`pluginManagement { includeBuild }`) | Convention-plugin builds (e.g. `build-logic`) are imported and navigable. | — | — | Automatic | P1 |
| GRADLE-050 | Language level per module | Each module's language level comes from `java.sourceCompatibility`/`release`/toolchain; project-level value is the lowest used (single linked build). | — | — | Project Structure \| Modules \| Sources | P0 |
| GRADLE-051 | Module SDK from Java toolchain | If `java.toolchain.languageVersion` is set, the module SDK is matched to a registered JDK with that version (inferred: picks a matching registered JDK, otherwise offers download). | — | — | Automatic | P1 |
| GRADLE-052 | Compiler args / encoding from build | `options.encoding`, `compilerArgs`, `--enable-preview` etc. are propagated to the IDE compiler and highlighting (inferred for compiler args). | — | — | Automatic | P1 |
| GRADLE-053 | Annotation processor config import | `annotationProcessor` dependencies become per-module processor profiles; generated sources directories registered. | — | — | Settings \| Build \| Compiler \| Annotation Processors (read-only "Gradle Imported" profile) | P0 |
| GRADLE-054 | IDE settings via `idea-ext` plugin | The `org.jetbrains.gradle.plugin.idea-ext` plugin can set package prefixes, encodings, run configurations, code style, etc. which are applied on sync. | — | — | Build script | P2 |
| GRADLE-055 | Generate `.iml` files option | Store generated module files in `.idea` / project dir (for mixed projects) instead of the system directory. | — | — | Settings \| … \| Gradle "Generate *.iml files for modules imported from Gradle" | P2 |
| GRADLE-056 | Manual changes are transient | Dependencies edited in Project Structure on Gradle modules are overwritten by the next sync (Gradle is the source of truth); the UI warns about it. | — | — | Project Structure | P1 |
| GRADLE-057 | External annotations download | Optionally downloads external nullability/contract annotations for known libraries from a public repository. | — | — | Settings \| … \| Gradle "Download external annotations for dependencies" | P2 |
| GRADLE-058 | Java module system (module-info) in Gradle | JPMS modules declared in `module-info.java` are resolved against Gradle module path. | — | — | Automatic | P1 |

### 1.4 Gradle JVM, distribution, wrapper

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-060 | Gradle JVM selection | Chooses the JDK that runs the Gradle daemon for sync and tasks. Choices: Project SDK, `JAVA_HOME`, any registered JDK, `#GRADLE_LOCAL_JAVA_HOME` (from `gradle.properties` `org.gradle.java.home`), Download JDK…, Add JDK…. | — | — | Settings \| … \| Build Tools \| Gradle \| Gradle JVM | P0 |
| GRADLE-061 | Gradle JVM auto-resolution | On first open, if no suitable JVM is configured the IDE picks a compatible one or offers to download one ("Download Java to run Gradle …"); JRE is rejected (JDK required). | — | — | Automatic, Build tool window fix link | P0 |
| GRADLE-062 | Gradle/JDK compatibility check | Validates the Gradle version against the selected JVM (e.g. "JDK X isn't supported by Gradle Y") and offers to change JVM or upgrade the wrapper. | — | — | Sync errors; settings validation | P0 |
| GRADLE-063 | Daemon JVM criteria (daemon toolchain) | For Gradle 8.8+, `gradle/gradle-daemon-jvm.properties` defines version/vendor criteria; the settings page then shows "Gradle JVM criteria" (Version, Vendor) instead of a fixed JVM; enabled by default since 2025.1 for existing criteria files. Task "Configure Daemon JVM Criteria" can generate the file. | — | — | Settings \| … \| Gradle | P2 |
| GRADLE-064 | Distribution source | "Distribution": **Wrapper** (from `gradle-wrapper.properties`, recommended default), **Wrapper task** (from `wrapper` task in the script), **Local installation** (path). | — | — | Settings \| … \| Gradle | P0 |
| GRADLE-065 | Gradle user home | Overrides the Gradle user home (default `~/.gradle`, or `GRADLE_USER_HOME`). | — | — | Settings \| … \| Gradle | P2 |
| GRADLE-066 | Wrapper generation and upgrade | New projects get a wrapper; quick-fixes/notifications can upgrade the wrapper version ("Upgrade Gradle wrapper"); inspection flags non-latest minor version in `gradle-wrapper.properties`. | — | — | Build tool window link; intention in properties file | P1 |
| GRADLE-067 | Gradle VM options migration | Legacy IDE-side Gradle VM options are migrated into `org.gradle.jvmargs` in `gradle.properties`. | — | — | Settings link "Migrate" | P2 |
| GRADLE-068 | Gradle Daemons dialog | Lists running/stopped daemons with status and description; actions Refresh, Stop Selected, Stop All, Stop When Idle All, Show Stopped. Action id `Gradle.ShowDaemons`. | — | — | Find Action "Show Gradle Daemons" | P2 |
| GRADLE-069 | Per-linked-build settings | Each linked Gradle build has its own JVM, distribution and build/run delegation settings (list of linked projects on the settings page). | — | — | Settings \| … \| Gradle | P1 |

### 1.5 Offline mode, proxies, repositories

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-075 | Toggle Offline Mode | Passes `--offline` to all Gradle invocations; dependencies resolve only from cache, missing ones fail with a hint. Action id `Gradle.ToggleOfflineAction`. | — | — | Gradle tool window toolbar; Find Action | P1 |
| GRADLE-076 | IDE proxy propagation | IDE HTTP proxy settings are offered/passed to Gradle (inferred: prompts to copy proxy settings into `gradle.properties`). | — | — | Settings \| Appearance & Behavior \| System Settings \| HTTP Proxy | P2 |

### 1.6 Gradle tool window

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-080 | Gradle tool window | Right-side tool window listing linked builds as a tree: project → Tasks (grouped by `group`), Dependencies (per configuration / source set), Run Configurations, and sub-projects. | — (no default) | — | View \| Tool Windows \| Gradle | P0 |
| GRADLE-081 | Run task by double-click | Double-clicking a task runs it, creating a temporary Gradle run configuration. | — | — | Gradle tool window | P0 |
| GRADLE-082 | Task context menu: Run / Debug / Modify Run Configuration | Run, debug (attaches debugger to forked JVMs such as tests/`JavaExec`), or create/edit a run configuration for a task. | — | — | Gradle tool window context menu | P0 |
| GRADLE-083 | Execute Gradle Task (Run Anything) | Popup to type one or more task names with arguments (completion of task names and options), runs them in the selected project. Action id `Gradle.ExecuteTask`; also via Run Anything with `gradle` prefix. | Ctrl, Ctrl (Run Anything) | Ctrl, Ctrl | Gradle tool window toolbar "Execute Gradle Task" | P0 |
| GRADLE-084 | Task activation (before/after build, rebuild, sync, run) | Binds tasks to IDE events; nodes show the binding. Actions `ExternalSystem.BeforeCompile`, `AfterCompile`, `BeforeRebuild`, `AfterRebuild`, `BeforeSync`, `AfterSync`, `BeforeRun`; dialog `ExternalSystem.OpenTasksActivationManager`. | — | — | Task context menu | P2 |
| GRADLE-085 | Assign shortcut to task | Binds a keyboard shortcut to a task or run configuration (`ExternalSystem.AssignShortcut`). | — | — | Task context menu | P2 |
| GRADLE-086 | Open Gradle Config | Opens the build script of the selected project node. Action id `ExternalSystem.OpenConfig`. | F4 | Cmd+Down / F4 | Gradle tool window context menu | P1 |
| GRADLE-087 | View options | Group Modules, Group Tasks, Show Ignored, Display Tasks Inherited from Sub-Projects (`ExternalSystem.ShowInheritedTasks`), Expand All / Collapse All. | Ctrl+NumPad+ / Ctrl+NumPad- | Cmd+= / Cmd+- | Gradle tool window options menu | P1 |
| GRADLE-088 | Speed search in tree | Typing filters/highlights tasks and dependencies. | type | type | Gradle tool window | P1 |
| GRADLE-089 | Dependencies tree node | Shows each configuration's resolved dependency tree, with navigation to declaration. | — | — | Gradle tool window \| Dependencies | P1 |
| GRADLE-090 | Dependency Analyzer | Dedicated view: list of all resolved dependencies per scope/module, conflict and omitted-version highlighting, "show usages" paths explaining why a dependency is present, navigation to declaration (`Gradle.DependencyAnalyzer.GoTo`), open build config. Entry ids `Gradle.ViewDependencyAnalyzer`, `Gradle.ToolbarDependencyAnalyzer`, `Gradle.ProjectViewDependencyAnalyzer`. | — | — | Gradle tool window toolbar "Analyze Dependencies"; Project view context menu | P1 |
| GRADLE-091 | Dependencies diagram | Graph visualisation of module/dependency relations. | — | — | Gradle tool window "Show Diagram" (inferred `$`) | P2 |
| GRADLE-092 | Build Tools settings shortcut | Toolbar menu opening Gradle settings and Auto-Sync settings. | — | — | Gradle tool window toolbar | P1 |
| GRADLE-093 | Run Configurations node | Lists saved Gradle run configurations; double-click runs; context menu edits/removes (`ExternalSystem.EditRunConfiguration`, `ExternalSystem.RemoveRunConfiguration`). | — | — | Gradle tool window | P1 |

### 1.7 Build/run delegation

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-100 | "Build and run using" | Per linked build: **Gradle** (default) or **IntelliJ IDEA** (internal JPS builder). With Gradle, Build Project, Run and Debug of Java applications invoke Gradle tasks (`classes`, `testClasses`, generated init scripts for running main classes); with IntelliJ IDEA the internal incremental compiler is used. | — | — | Settings \| … \| Build Tools \| Gradle | P0 |
| GRADLE-101 | "Run tests using" | Per linked build: **Gradle** (default), **IntelliJ IDEA**, or **Choose per test**. Determines whether gutter/context test runs create Gradle or JUnit/TestNG configurations. | — | — | Same page | P0 |
| GRADLE-102 | Gradle-backed Build Project | Build \| Build Project maps to Gradle compile tasks of all modules; Rebuild maps to clean + compile (inferred). | Ctrl+F9 | Cmd+F9 | Build menu | P0 |
| GRADLE-103 | Gradle-backed application run | Running a `main` method from the gutter generates a Gradle-executed Application run (init script adds a JavaExec task). | Ctrl+Shift+F10 | Ctrl+Shift+R | Gutter, context menu | P0 |
| GRADLE-104 | HotSwap with Gradle delegation | When debugging with delegated build, recompiling via Gradle still allows class reload into the debuggee. | Ctrl+F9 | Cmd+F9 | Build menu during debug | P1 |

### 1.8 Gradle run configurations and execution

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-110 | Gradle run configuration type | Fields: Name, Run (tasks and arguments in one command line with completion; "Default tasks will be executed if no tasks are specified"), Gradle project (path), Environment variables, VM options (for the Gradle client), "Store as project file", Before launch list. Modify options: Run as test ("shows test view and forces test re-execution"), Debug Gradle scripts, Debug forked Gradle tasks in separate debug tabs, Debug all tasks on the execution graph, Debugging/Profiling/Coverage disabled toggles, Open/redirect output, Show console when output, logs. | — | — | Run \| Edit Configurations… \| + \| Gradle | P0 |
| GRADLE-111 | Task and argument completion | Completion of task paths (`:app:test`), command-line options (`--info`, `--tests`), and source-set names in the run field. | Ctrl+Space | Ctrl+Space | Run configuration editor, Run Anything | P1 |
| GRADLE-112 | Gradle console output | Task output appears in the Run tool window (tree of task executions on the left, console on the right; "Toggle view" between tree and raw text). Failed tasks link to the failing line. | — | — | Run tool window | P0 |
| GRADLE-113 | Build output in Build tool window | Build Project via Gradle shows a "Build Output" tab with task tree, compiler errors navigable to source, and raw console. | — | — | Build tool window | P0 |
| GRADLE-114 | Debug Gradle build scripts | Breakpoints in `build.gradle(.kts)` are honoured when running tasks in debug with "Debug Gradle scripts" enabled. | Shift+F9 | Ctrl+D | Run configuration option | P2 |
| GRADLE-115 | Rerun / stop | Rerun last Gradle execution; stop terminates the build (cancellation via Tooling API). | Ctrl+F5 / Ctrl+F2 | Cmd+R / Cmd+F2 | Run tool window | P0 |
| GRADLE-116 | Continuous build | Running with `--continuous` keeps watching; inferred support in console. (inferred) | — | — | Run configuration args | P2 |
| GRADLE-117 | Environment / gradle.properties handoff | Project and system properties (`-P`, `-D`) and env vars per configuration; "Pass parent environment variables" toggle (inferred). | — | — | Run configuration | P1 |

### 1.9 Test runner integration (Gradle)

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-120 | Gradle test execution with IDE test tree | Running tests via Gradle (`test --tests <filter>`) streams results into the standard test runner UI (tree, pass/fail, timing, output per test, diff for assertion failures). | Ctrl+Shift+F10 | Ctrl+Shift+R | Gutter run icons, context menu | P0 |
| GRADLE-121 | Test filter generation | Selecting a class/method/package generates the right `--tests` filter and target `Test` task (task chooser when several test tasks cover the source set, e.g. `test` vs `integrationTest`). | — | — | Automatic; popup | P0 |
| GRADLE-122 | Rerun failed tests via Gradle | Reruns only the failed tests with filters. | — | — | Test runner toolbar | P1 |
| GRADLE-123 | Debug tests via Gradle | Debugging a test attaches to the forked test JVM. | Shift+F9 (rerun debug) | Ctrl+D | Gutter, context menu | P0 |
| GRADLE-124 | Up-to-date test rerun | IDE forces test task re-execution (`--rerun` / cleanTest) so "up-to-date" does not hide results (inferred mechanism). | — | — | Automatic | P1 |

### 1.10 Build script editing: Kotlin DSL and Groovy DSL

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-130 | Kotlin DSL highlighting and resolve | `.gradle.kts` scripts are analysed as Kotlin scripts with the Gradle script classpath (requires a successful sync to supply script dependencies); full Kotlin editing features. | — | — | Editor | P0 |
| GRADLE-131 | Script configuration loading | Kotlin script definitions/classpath are loaded during sync; notification if script configuration is out of date. | — | — | Editor notification | P0 |
| GRADLE-132 | Groovy DSL support | `.gradle` scripts get Groovy highlighting, Gradle-API-aware completion (closures delegate to Gradle types), navigation into Gradle API sources. | — | — | Editor | P0 |
| GRADLE-133 | Completion in dependencies/plugins blocks | Completion for configuration names, plugin ids, task names, extension properties; dependency coordinate completion (group/artifact/version, from local cache and repository search). 2026.2 adds improved "dependency completion" (details unknown). | Ctrl+Space | Ctrl+Space | Editor | P1 |
| GRADLE-134 | Generate / add dependency | Alt+Insert in a build script offers "Add Maven artifact dependency" / plugin (groups `GenerateGradleDslGroup`, `Gradle.GenerateGroup`, actions `Gradle.AddGradleDslDependencyAction`, `AddGradleDslPluginAction`) with artifact search. | Alt+Insert | Cmd+N | Editor in build script | P1 |
| GRADLE-135 | Navigation in build scripts | Ctrl+Click on `project(":x")`, on included builds, on task names, on plugin ids, on version catalog accessors, and into Gradle/plugin API sources. | Ctrl+B | Cmd+B | Editor | P1 |
| GRADLE-136 | Quick documentation for Gradle DSL | Ctrl+Q shows docs for Gradle DSL elements and configurations (messages in `GradleDocumentationBundle`). | Ctrl+Q | F1 | Editor | P2 |
| GRADLE-137 | Gutter run icons in build scripts | Tasks declared in scripts get gutter "run" icons. | — | — | Gutter | P1 |
| GRADLE-138 | `gradle.properties` support | Completion/docs for well-known `org.gradle.*` properties; navigation to usages (module `intellij.gradle.java.properties`). | — | — | Editor | P2 |
| GRADLE-139 | Settings script support | `settings.gradle(.kts)`: `include` entries navigate to sub-projects; unresolvable includes flagged (inferred). | — | — | Editor | P1 |

### 1.11 Version catalogs

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| GRADLE-145 | TOML editing for `libs.versions.toml` | Syntax highlighting, structure for `[versions]`, `[libraries]`, `[bundles]`, `[plugins]`. | — | — | Editor | P1 |
| GRADLE-146 | Catalog accessor resolve | `libs.foo.bar` in Kotlin/Groovy scripts resolves to the TOML entry; Ctrl+Click navigates; Find Usages from TOML entry to scripts. | Ctrl+B / Alt+F7 | Cmd+B / Opt+F7 | Editor | P1 |
| GRADLE-147 | Catalog completion | Completion of `libs.` accessors in build scripts, and of version refs in TOML. | Ctrl+Space | Ctrl+Space | Editor | P1 |
| GRADLE-148 | Unused version catalog entry inspection | `UnusedVersionCatalogEntry` (Style, warning, on by default) flags entries no script uses. | — | — | Editor highlighting | P2 |
| GRADLE-149 | Rename catalog entries | Renaming a TOML key updates accessor usages (inferred). | Shift+F6 | Shift+F6 | Refactor \| Rename | P2 |

### 1.12 Gradle inspections (build scripts)

All enabled by default unless noted; registered for Groovy/Kotlin build scripts, properties or TOML.

| ID | Inspection (short name) | Group | Default level | What it flags | Pri |
|---|---|---|---|---|---|
| GRADLE-160 | Configuration avoidance (`ConfigurationAvoidance`) | Best practices | Warning | Eager task APIs (`create`, `getByName`, `tasks.all`) where lazy `register`/`named`/`configureEach` should be used. | P2 |
| GRADLE-161 | Unrecognized dependency notation (`DependencyNotationArgument`) | Probable bugs | Warning | Dependency arguments Gradle cannot interpret. | P2 |
| GRADLE-162 | Possibly misplaced call to Gradle method (`ForeignDelegate`) | Probable bugs | Weak warning | Calls resolved against an outer closure delegate unexpectedly. | P2 |
| GRADLE-163 | Plugin DSL structure (`IncorrectPluginDslStructure`) | Validity issues | Error | Illegal statements inside `plugins {}`. | P1 |
| GRADLE-164 | Deprecated configurations (`DeprecatedConfigurations`) | Validity issues | Warning | Use of removed/deprecated configurations such as `compile`, `testCompile`. | P1 |
| GRADLE-165 | Redundant Kotlin standard library dependency (`RedundantKotlinStdLibDependency`) | Best practices | Warning | Explicit `kotlin-stdlib` already added by the Kotlin plugin. | P2 |
| GRADLE-166 | Named-arguments dependency notation (`AvoidDependencyNamedArgumentsNotation`) | Best practices | Weak warning | Map-style `group:, name:, version:` notation; quick-fix to GAV string. | P2 |
| GRADLE-167 | Legacy `apply` plugin notation (`AvoidApplyPluginMethod`) | Best practices | Weak warning | `apply plugin:` instead of `plugins {}`. | P2 |
| GRADLE-168 | Duplicate dependencies (`AvoidDuplicateDependencies`) | Best practices | Weak warning | Same dependency declared twice in a configuration. | P2 |
| GRADLE-169 | Duplicate repositories (`AvoidDuplicateRepositories`) | Best practices | Weak warning | Repeated repository declarations. | P2 |
| GRADLE-170 | Describe custom tasks (`TaskMissingDescription`) | Best practices | Weak warning | Registered tasks without `description`. | P2 |
| GRADLE-171 | Repositories in build scripts (`AvoidRepositoriesInBuildGradle`) | Best practices | Weak warning, **off** by default | Repositories declared per-project instead of centrally in settings. | P2 |
| GRADLE-172 | Latest Gradle minor version (`LatestMinorVersion`) | Best practices | Warning | Wrapper not on latest minor of its major line (in `gradle-wrapper.properties`). | P2 |
| GRADLE-173 | Multiple repository URLs (`MultipleRepositoryUrls`) | Probable bugs | Warning | More than one `url` in a single Maven repository block (Groovy). | P2 |
| GRADLE-174 | JCenter repository (`JCenterRepository`) | Probable bugs | Warning | Use of the sunset JCenter repository (Groovy). | P2 |
| GRADLE-175 | Bintray publishing plugin (`BintrayPublishingPlugin`) | Probable bugs | Warning | Obsolete Bintray plugin (Groovy). | P2 |
| GRADLE-176 | Unused version catalog entry | see GRADLE-148 | Warning | — | P2 |

The 2026.1 release describes these as the first batch of "official Gradle best practices" turned into
inspections with quick-fixes.

### 1.13 Settings | Build, Execution, Deployment | Build Tools (| Gradle) — option inventory

| ID | Option | Values / default | Pri |
|---|---|---|---|
| GRADLE-180 | Sync project after changes in the build scripts | checkbox (on) + Any changes / External changes | P0 |
| GRADLE-181 | Gradle user home | path; default `~/.gradle` / `GRADLE_USER_HOME` | P2 |
| GRADLE-182 | Generate *.iml files for modules imported from Gradle | checkbox (off, inferred) | P2 |
| GRADLE-183 | Enable parallel Gradle model fetching for Gradle 7.4+ | checkbox (off for new projects per docs) | P2 |
| GRADLE-184 | Download external annotations for dependencies | checkbox | P2 |
| GRADLE-185 | Linked Gradle projects list | select which build the options below apply to | P1 |
| GRADLE-186 | Build and run using | Gradle (default) / IntelliJ IDEA | P0 |
| GRADLE-187 | Run tests using | Gradle (default) / IntelliJ IDEA / Choose per test | P0 |
| GRADLE-188 | Distribution | Wrapper (default) / Wrapper task / Local installation (+ location) | P0 |
| GRADLE-189 | Gradle JVM | registered JDKs, Project SDK, JAVA_HOME, `#GRADLE_LOCAL_JAVA_HOME`, Download/Add JDK | P0 |
| GRADLE-190 | Gradle JVM criteria | Version + Vendor (only with daemon toolchain file) | P2 |
| GRADLE-191 | "Use these settings for future projects" | checkbox on distribution in New Project flow | P2 |
| GRADLE-192 | Generate module per source set | deprecated, troubleshooting only (hidden/advanced) | P2 |
| GRADLE-193 | Gradle VM options | deprecated; migrated to `org.gradle.jvmargs` | P2 |

Sources (section 1):
- https://www.jetbrains.com/help/idea/gradle.html
- https://www.jetbrains.com/help/idea/work-with-gradle-projects.html
- https://www.jetbrains.com/help/idea/jetgradle-tool-window.html
- https://www.jetbrains.com/help/idea/gradle-settings.html
- https://www.jetbrains.com/help/idea/gradle-jvm-selection.html
- https://www.jetbrains.com/help/idea/work-with-gradle-dependency-diagram.html
- https://www.jetbrains.com/help/idea/run-debug-gradle.html
- https://www.jetbrains.com/help/idea/gradle-run-configuration.html
- https://www.jetbrains.com/idea/whatsnew/2026-1/ , https://www.jetbrains.com/idea/whatsnew/2026-2/
- https://docs.gradle.org/current/userguide/tooling_api.html , https://docs.gradle.org/current/userguide/platforms.html (version catalogs), https://docs.gradle.org/current/userguide/gradle_daemon.html , https://docs.gradle.org/current/userguide/toolchains.html , https://docs.gradle.org/current/userguide/composite_builds.html , https://docs.gradle.org/current/userguide/task_configuration_avoidance.html
- Community sources: `plugins/gradle/resources/messages/GradleBundle.properties`, `platform/external-system-impl/resources/META-INF/ExternalSystemActions.xml`, `plugins/gradle/java/resources/intellij.gradle.java.xml` (and `.groovy`, `.toml`, `.properties` variants), `ExternalSystemBundle.properties`.

---

## 2. JDK management and language level

| ID | Feature | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| JAVA-JDK-001 | Project Structure dialog | Central dialog: Project (SDK, language level, compiler output), Modules, Libraries, Facets, Artifacts, Platform Settings (SDKs, Global Libraries), Problems. | Ctrl+Alt+Shift+S | Cmd+; | File \| Project Structure… | P0 |
| JAVA-JDK-002 | Project SDK | Default JDK for all modules; used for resolve, highlighting and (internal) build. | — | — | Project Structure \| Project | P0 |
| JAVA-JDK-003 | Module SDK | Per-module override ("Project SDK" or specific JDK); for Gradle modules derived from toolchain (GRADLE-051). | — | — | Project Structure \| Modules \| Dependencies | P0 |
| JAVA-JDK-004 | SDK registry (Platform Settings \| SDKs) | List of registered JDKs: add from disk, rename, remove; per-SDK classpath, sourcepath, annotations and documentation paths. | — | — | Project Structure \| SDKs | P0 |
| JAVA-JDK-005 | Auto-detection of installed JDKs | Detects JDKs in standard locations (`JAVA_HOME`, `~/.jdks`, `/usr/lib/jvm`, SDKMAN, `/Library/Java/JavaVirtualMachines`, Windows registry) and offers them in pickers. | — | — | SDK combo boxes "Detected SDKs" | P0 |
| JAVA-JDK-006 | Download JDK dialog | Pick Version and Vendor (e.g. Oracle OpenJDK, Eclipse Temurin, Amazon Corretto, Azul Zulu, GraalVM, JetBrains Runtime, BellSoft Liberica, IBM Semeru, SAP Machine, Microsoft); choose install location (default `~/.jdks`); downloads, verifies and registers. Includes Java early-access builds (2026.1). | — | — | SDK combo \| Download JDK…; Project Structure \| SDKs \| + | P0 |
| JAVA-JDK-007 | Missing SDK banner | Editor notification "Project JDK is not defined"/"Setup SDK" with quick picker. | — | — | Editor banner | P0 |
| JAVA-JDK-008 | Project language level | Combo of Java versions (e.g. 8 … 27, plus "X (Preview)" levels and "SDK default"). Determines allowed syntax and inspections. | — | — | Project Structure \| Project | P0 |
| JAVA-JDK-009 | Module language level | Per-module override (from Gradle `sourceCompatibility`/`release`). | — | — | Project Structure \| Modules \| Sources | P0 |
| JAVA-JDK-010 | Language-level quick-fixes | Using newer syntax offers "Set language level to N" (for Gradle: edits or suggests build file change, inferred). | Alt+Enter | Opt+Enter | Intention | P0 |
| JAVA-JDK-011 | Preview features handling | Preview language levels enable preview syntax; `PreviewFeature` warning; `--enable-preview` passed to compiler/run (inferred). | — | — | Language level combo | P1 |
| JAVA-JDK-012 | JDK sources & docs | Attaches `src.zip` for navigation; external Javadoc URL for Shift+F1. | — | — | SDK settings | P1 |
| JAVA-JDK-013 | Compiler output paths | Project compiler output and per-module output/test output (internal builder only). | — | — | Project Structure \| Project / Modules \| Paths | P1 |
| JAVA-JDK-014 | Libraries management | Project/global libraries, add jar/dir, attach sources/Javadoc, "From Maven…" download. | — | — | Project Structure \| Libraries | P1 |
| JAVA-JDK-015 | Problems page | Lists invalid SDKs/libraries/dependencies with fixes. | — | — | Project Structure \| Problems | P2 |
| JAVA-JDK-016 | Choose Boot Java Runtime for the IDE | Selects the runtime the IDE itself runs on (platform-level). | — | — | Find Action | P2 |
| JAVA-JDK-017 | Supported Java versions | 2026.2 announces Java 27 support; 2026.1 Java 26 (incl. preview pattern features, lazy constants). | — | — | — | P0 |

Sources (section 2):
- https://www.jetbrains.com/help/idea/sdk.html
- https://www.jetbrains.com/help/idea/project-settings-and-structure.html
- https://www.jetbrains.com/help/idea/supported-java-versions.html
- https://www.jetbrains.com/help/idea/whatsnew (2026.1, 2026.2 pages)

---
