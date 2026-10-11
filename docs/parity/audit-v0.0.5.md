# Visual diff audit v0.0.5 — Jetthing vs IntelliJ IDEA Community / Android Studio

Scene-by-scene comparison of Jetthing `23270f7` (origin/main after v0.0.5, debug build) against the
reference captures in `evidence/reference/intellij-community/java-gradle/` (IntelliJ IDEA
Community 2025.2.6.3) and `evidence/reference/android-studio/` (Android Studio 2026.2.1). Each
side-by-side image under `evidence/audit-v0.0.5/` has the **reference on the left and Jetthing on
the right** (`convert +append`, 256 colours, < 300 KB).

Capture setup: Xvfb `:96` 1600x1000x24 + openbox, `JETTHING_RENDERER=glow`,
`JETTHING_NO_UPDATE_CHECK=1`, window resized to 1600x980 with `xdotool`, driven over the control
socket (`docs/parity/evidence/ui-control-socket.md`), `import -window root`. Scratch copies of
`/home/user/samples/java-gradle` and `/home/user/samples/MyApplication` (build dirs removed, Git
history kept). `toolchain.install jdtls` was run first so Java diagnostics and completion are
real; the Kotlin language server is not a toolchain component, so Kotlin scenes run without it.
Disk was below the 9 GB threshold after the Jetthing build, so the IDEs were **not** reinstalled;
the existing reference captures were used and scenes without a reference (branches popup, Markdown
split, extra Settings pages) are recorded with the Jetthing capture only.

Classification: **BUG** = exists but looks/behaves differently, **FEATURE** = missing. Weight 1–10
= user impact x visibility, size S/M/L. Jetthing-side observations that are environment artefacts
(language-server balloons, proxy 429s) are noted but not scored.

## Scenes — IntelliJ IDEA Community (java-gradle)

### 01 Welcome — `evidence/audit-v0.0.5/01-welcome.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-032 | FEATURE | No left navigation (Projects / Customize / Plugins / Learn), no product version, no recent-project list, no square icon buttons for New Project / Open / Clone Repository, no onboarding card. Jetthing shows a centred title, two plain buttons and the "Open Project: Ctrl+O" hint. | 8 | L | IJ-MENU-100 |
| AUD-023 | BUG | The main toolbar keeps the project widget ("NP No project"), greyed run icons and the status bar reads "No project"; the reference welcome window has no IDE chrome. | 3 | S | IJ-MENU-100 |
| AUD-005 | BUG | Find Action on the welcome screen returns "Nothing found" for `New Project`, although the button exists. | 5 | S | IJ-MENU-111 |

### 02/03 New Project wizard — `evidence/audit-v0.0.5/02-new-project.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-033 | FEATURE | Reference is a two-pane dialog: generator list with search (Java, Kotlin, Groovy, Empty, Maven Archetype, JavaFX, …), Name, Location + browse, "Project will be created in" hint, Create Git repository, Build system segmented control (IntelliJ / Maven / Gradle / Bazel), JDK combo, Add sample code, Advanced Settings (Gradle DSL, GroupId/ArtifactId), `?` help. Jetthing: a ~560x270 dialog with Name, Location + Browse, Template combo, Group / package, a two-line hint, Create / Cancel. No window title, no JDK, no Git, no build-system choice, no Android templates. | 8 | L | IJ-JAVA-001 |

### 04 Editor + project tree — `evidence/audit-v0.0.5/04-editor-project-tree.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-001 | BUG | After `toolchain.install jdtls` the project root gains `.classpath`, `.project`, `.settings/`, `bin/` (compiled `.class` files) plus `.jetthing/*.toml`, `jetthing-model.json`, `model.json`; they appear as ordinary tree rows and as 10 unversioned files in Commit (scene 12). The reference writes only `.idea/` and marks it. | 8 | M | IJ-VCS-004, IJ-VCS-005 |
| AUD-008 | BUG | Packages are not compacted (`com` > `example` vs `com.example`); the test root has no green tint and `main`/`test` are not bold; `build`, `.gradle`, `bin` are not shown excluded (orange); no `External Libraries` / `Scratches and Consoles`; the root row's path is truncated without ellipsis; the header has no `Project ▾` view switcher. | 5 | M | IJ-TW-014, IJ-TW-011 |
| AUD-036 | FEATURE | No code vision (`2 usages`, author) above declarations, no `name:` parameter inlays, one-line method body not folded with boxed braces. | 7 | M | IJ-ED-104 |
| AUD-011 | BUG | Gutter run icon only on `main`, not on the class line; gutter popup has no "Run with Coverage" and a disabled Debug. | 4 | S | IJ-RUN-011 |
| AUD-053 | FEATURE | Window header has no title text (`java-gradle – App.java [java-gradle.main]` in the reference). | 3 | S | IJ-MENU-104 |
| AUD-021 | BUG | Status bar lacks the line-separator and read-only widgets and shows extra `Java`, `jdtls: …`, `AI: off` text. | 3 | S | IJ-MENU-106 |
| AUD-061 | BUG | Language-server balloons are large (420x120), persistent and cover the bottom-right of tool windows; reference balloons are compact and auto-hide. | 3 | S | PLAT-007 |

### 05 Search Everywhere — `evidence/audit-v0.0.5/05-search-everywhere.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-006 | BUG | For `main` the reference lists the `main(String[])` symbol, `java-gradle.main` module, text hits and actions/settings with inline ON/OFF toggles; Jetthing lists one text hit (`App.java:10`) and three `Markdown: Insert …` actions matched by scattered letters. Popup is ~680x220 vs ~700x700; no `Text` tab, no "Include non-project items", no preview/filter icons, no "Type / to see commands", no footer path / "Open In Right Split". | 6 | M | IJ-NAV-001, IJ-NAV-006 |

### 06 Find Action — `evidence/audit-v0.0.5/06-find-action.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-007 | BUG | Shortcuts are right-aligned in a column instead of inline grey after the name; the group (`Run`, `Code \| Analyze Code`) is missing; no "Include disabled actions" checkbox; disabled actions (`Stop Running Task`, `AI: Run Suggested Command`) are mixed in; `Run 'app' on Device` (Android) appears in a plain Java project; no "Assign Shortcut Alt+Enter" footer; highlight matches scattered letters (`Running`). | 5 | S | IJ-MENU-111 |

### 07 Main menu — `evidence/audit-v0.0.5/07-main-menu.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-004 | BUG | File menu: `Plugins...` listed twice plus `Refresh Plugins` at the bottom; `Close Project`, `Project Structure...`, `Invalidate Caches...`, `Print...` disabled; `Repair IDE` and `Save File as Template...` placement differ; `Settings for New Projects…`, `Run Configuration Templates…`, `Save Project as Template…`, `Manage Project Templates…` are flat items instead of the `New Projects Setup ▸` submenu; `Recent Projects ▸` holds only `Manage Projects...`; `Open...` has no `Ctrl+O`-style accelerator column alignment. Hover over the top-level items after the hamburger click did not open the menu; a second click was needed. | 6 | S | IJ-MENU-104, IJ-MENU-013, IJ-MENU-008 |

### 08 Settings — `evidence/audit-v0.0.5/08-settings.png`, `08b-settings-keymap.png`, `08c-settings-editor.png`, `08d-settings-vcs.png`, `08e-settings-build.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-042 | FEATURE | Many tree pages are stubs ("This page has no options in Jetthing yet"): Build Tools > Gradle (reference: Gradle user home, Build and run using, Run tests using, Distribution, Gradle JVM), Editor > Code Editing, File and Code Templates; Version Control and Editor show link lists only. | 6 | M | IJ-SET-001, IJ-BLD-005 |
| AUD-058 | BUG | Keymap page lists actions flat and alphabetically; reference groups them in a tree (Editor Actions, Main Menu, …) with icons. Appearance page lacks the scheme gear, "red-green deficiency" option, "Main menu: Hide under Hamburger" combo and the `?` footer button; Editor > Font lacks the preview and fallback font rows. | 3 | S | IJ-SET-032, IJ-SET-001 |

### 09 Completion — `evidence/audit-v0.0.5/09-completion.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-017 | BUG | Popup matches the reference structurally (icons, types right-aligned, auto-popup on `.` works). Missing: ML star on the best item, greyed parameter lists, the `Next Tip` footer, "Paused..." indicator, wider popup; `main(String[] args)` (static) is offered on an instance. | 3 | S | IJ-ED-081 |

### 10 Find in Files — `evidence/audit-v0.0.5/10-find-in-files.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-043 | FEATURE | Reference is a floating popup with `In Project \| Module \| Directory \| Scope` tabs, result rows with file:line, a live preview editor, "Open results in new tab" and "Open in Find Window". Jetthing is a bottom tool window with query, Cc/W/.*, file mask and a flat result list; no scopes, no preview. | 5 | M | IJ-NAV-065, IJ-NAV-066 |
| AUD-009 | BUG | Re-opening with Ctrl+Shift+F keeps the previous query without selecting it, so typing appends (`greetinggreeting`); results need Enter instead of updating while typing. | 4 | S | IJ-NAV-064 |

### 11 Terminal — `evidence/audit-v0.0.5/11-terminal.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-012 | BUG | No `Local ×` tab strip with `+` / `▾` (multiple sessions), no per-command blocks; cursor is a hollow `[]`. Colours and prompt match. | 3 | S | IJ-TW-029 |

### 12 Commit — `evidence/audit-v0.0.5/12-commit.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-001 | BUG | "Unversioned Files 10 files" are all IDE artefacts (see scene 04). | — | — | IJ-VCS-004 |
| AUD-022 | BUG | Toolbar has refresh and back only (reference: refresh, rollback, shelve/update, show diff, expand/collapse); commit message box is 3 lines high vs the full panel; `Commit and Push...` lacks the gear. | 3 | S | IJ-TW-028 |

### 13 Diff — `evidence/audit-v0.0.5/13-diff.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-013 | BUG | Opens as an editor tab like the reference. Missing: `Do not ignore ▾`, `Highlight words ▾`, collapse unchanged, sync scroll, settings, help; per-chunk checkbox and `>>` apply arrow (partial commit); read-only lock in the left header; `Current version` checkbox. | 4 | S | IJ-VCS-050, IJ-VCS-051, IJ-VCS-012 |

### 14 Git log — `evidence/audit-v0.0.5/14-git-log.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-014 | BUG | No `Log` tab strip, no User / Date / Paths filters, no `.*` / `Cc` toggles, no toolbar (sort, refresh, cherry-pick, eye); changed-files pane is a flat path list instead of a tree with file counts; no graph colour dot. | 4 | S | IJ-VCS-053 |

### 14b Branches popup — `evidence/audit-v0.0.5/14b-branches.png` (no reference capture; compared with the reference VCS widget behaviour)

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-015 | BUG | Clicking `master ▾` opens a Git action menu (Update Project, Commit, Push, Fetch, New Branch, Checkout Tag…, Stash, Unstash, Show Git Log, Local > master). The reference opens the Branches popup: search field, Recent, Local / Remote groups, per-branch submenu (Checkout, New branch from, Compare…). | 5 | M | IJ-VCS-040 |

### 15 Run widget / Run-Debug Configurations — `evidence/audit-v0.0.5/15-run-config.png`, `15b-run-config-dialog.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-016 | BUG | Widget defaults to `java-gradle [run]` instead of `Current File`; popup has no inline Run / Debug / ⋮ per configuration and mixes actions (Build Project, Run All Tests, Rerun Last Task) into the configuration list. | 4 | S | IJ-MENU-103 |
| AUD-030 | BUG | Dialog has only Gradle configurations (Name, Type, Run, Options, Program arguments, Working directory, Environment variables). Missing: Application type with JDK / `-cp` module / main class rows, "Store as project file", `Modify options ▾`, "Open run/debug tool window when started" chip, Code Coverage section, "Edit configuration templates...", `?`; no window title. | 4 | S | IJ-RUN-001, IJ-RUN-004 |

### 16 Build output — `evidence/audit-v0.0.5/16-build-output.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-020 | BUG | One `Build` panel titled `Gradle Build` replaces the previous Sync content; reference has `Sync \| Build Output` tabs, a two-level tree (build finished > warnings) and a left toolbar (soft-wrap, scroll to end, clear). URLs in the console are not links. Task tree with durations matches. | 4 | S | IJ-JAVA-033 |

### 17 Run output — `evidence/audit-v0.0.5/17-run-output.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-002 | BUG | Tree header reads `java-gradle [java-gradle [run]]: successful` — the configuration name is nested twice. Tab/tool-window title shows `Run` + `java-gradle [run]` on the right instead of a tab named after the configuration. Left toolbar (rerun, stop, eye, ⋮) reduced to one icon. | 7 | S | IJ-TW-023 |

### 18 Test results — `evidence/audit-v0.0.5/18-test-results.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-035 | FEATURE | `test.project` runs `gradle test` in a `Test` panel that shows only the Gradle task tree and console. Reference: `Test Results > AppTest > greets` with green checks and durations, "✓ 1 test passed  1 test total, 21ms" header with a green progress line, Show Passed / Show Ignored toggles, rerun failed, sort, history. | 7 | L | IJ-TW-037, IJ-RUN-061 |

### 19 Project context menu — `evidence/audit-v0.0.5/19-project-context-menu.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-034 | FEATURE | Jetthing: New ▸, Copy Path/Reference ▸, Edit Source F4, Find in Files, Rename..., Delete.... Reference adds Cut / Copy / Paste, Find Usages, Analyze ▸, Refactor ▸, Bookmarks ▸, Reformat Code, Optimize Imports, Override File Type, Run / Debug 'App.main()', More Run/Debug ▸, Open in Right Split, Open In ▸, Local History ▸, Git ▸, Repair IDE on File, Reload from Disk, Compare With..., Convert Java File to Kotlin File, with separators and icons. | 8 | M | IJ-MENU-109 |

### 20 Problems — `evidence/audit-v0.0.5/20-problems.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-049 | BUG | Close match (File / Project Errors tabs, red underline on tab and tree ancestors, editor counter). Missing: left toolbar (preview, quick-fix, group by), `Vulnerable Dependencies` / `Qodana` tabs; problem text comes from jdtls (`The method greting(String) is undefined for the type App :14`). | 3 | S | IJ-TW-025 |

### 21 Gradle tool window — `evidence/audit-v0.0.5/21-gradle-tool-window.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-029 | BUG | Tree matches (project > Tasks > groups). Missing: `Dependencies` node, task gear icons, toolbar items attach / download sources / offline / analyze dependencies / settings; filter box replaces the reference's title bar layout. | 3 | S | IJ-BLD-004 |

### 22 Structure — `evidence/audit-v0.0.5/22-structure.png`, `22b-structure-window.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-019 | BUG | Popup lacks the "Inherited members (Ctrl+F12) / Anonymous classes (Ctrl+I) / Lambdas (Ctrl+L)" checkboxes, gear and visibility markers; popup is ~560x160 vs ~540x520. Structure tool window (Alt+7) exists. | 3 | S | IJ-NAV-040, IJ-TW-020 |

### 23 Debugger — `evidence/audit-v0.0.5/23-debugger.png`, `x03-gutter-run-popup.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-031 | FEATURE | Shift+F9 does nothing (status unchanged), gutter `Debug 'App.main()'` is disabled, no breakpoints on gutter click, no Debug tool window (Threads & Variables / Console, frames, variables, evaluate field), no execution-line highlight or inline values, no step toolbar. | 10 | L | IJ-RUN-030, IJ-RUN-032, IJ-RUN-038, IJ-RUN-040, IJ-RUN-041 |

### 24 Plugins — `evidence/audit-v0.0.5/24-plugins.png`, `24b-plugins-marketplace.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-041 | FEATURE | Marketplace tab shows "No plugin marketplace is configured" + plugin directories and "Install Plugin from Disk...". Reference: Suggested / Staff Picks cards with downloads, rating, vendor, Install, and a detail pane (Overview / What's New / Reviews / Additional Info, screenshots). Installed tab lists one bundled plugin without a detail pane. | 6 | L | IJ-SET-071, IJ-SET-070 |

### 25 Markdown split — `evidence/audit-v0.0.5/25-markdown.png` (no reference capture)

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-051 | BUG | Editor / Split / Preview toggle and rendered preview (heading, bold, list, fenced code) work. Differences from the reference layout: toggle lives inside the preview pane instead of the editor's top-right toolbar, no scroll sync indicator, code block uses the editor background instead of a bordered block, no Markdown tab icon. | 2 | S | IJ-ED-151 |

## Scenes — Android Studio (MyApplication)

### a04 Editor + Android view — `evidence/audit-v0.0.5/a04-editor-project-tree.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-040 | FEATURE | "Kotlin language server not found" balloon; `toolchain.setup` reports everything set up, so kotlin-lsp is not provisioned like jdtls. Consequently no run / override gutter icons, no `1 Usage`, no `name =` inlay, no warning counter in Kotlin files. Android view, `import ...` folding and the `Gradle Scripts` node match. | 6 | M | AND-003, PLAT-016 |

### a01 Device Manager — `evidence/audit-v0.0.5/a01-device-manager.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-025 | BUG | Columns and "No devices connected" match. Missing: toolbar (create virtual device, pair over Wi-Fi, filter, help), "Add a new device..." link, tab for Physical/Virtual. | 4 | S | AND-004, AND-005 |

### a02 Logcat — `evidence/audit-v0.0.5/a02-logcat.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-024 | BUG | Device combo + `package:mine` + Cc match. Missing: `Logcat ×` tab strip with `+`, left toolbar (clear, pause, restart, scroll to end, soft wrap, split, settings), filter history / favourites star, syntax-coloured filter chips. | 5 | M | AND-008 |

### a03 Running Devices — `evidence/audit-v0.0.5/a03-running-devices.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-039 | FEATURE | Not findable (Find Action "Running Devices" → Nothing found); no tool window, no mirroring empty state. | 5 | M | AND-017 (new) |

### a04 Layout editor — `evidence/audit-v0.0.5/a04-layout-editor.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-037 | FEATURE | `activity_main.xml` opens as highlighted XML text only. Reference: Code / Split / Design toggle, rendered device preview + blueprint, device / API / theme / locale toolbar, zoom controls, Palette and Component Tree side tabs, Attributes panel. | 7 | L | AND-015 (new) |

### a05 SDK Manager — `evidence/audit-v0.0.5/a05-sdk-manager.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-027 | BUG | Settings > Languages & Frameworks > Android SDK exists with SDK Platforms / SDK Tools tabs but lists installed packages only ("Packages are installed with sdkmanager"). Missing: available versions with Status column, `SDK Update Sites` tab, Edit / Optimize disk space links, "Hide incompatible system images" / "Hide obsolete packages" toggles, Kotlin / Mermaid siblings. | 4 | M | AND-001 |

### a06 Gradle sync — `evidence/audit-v0.0.5/a06b-gradle-sync-done.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-028 | BUG | Sync window has a `Sync` title and `:jetthingModel UP-TO-DATE` tree. Reference: `Sync \| Build Output \| Build Analyzer` tabs, phases (`Download info`), per-dependency error nodes with "Show in Project Structure dialog" / "Fix with AI" links, cancel in the status bar. | 3 | S | AND-002, IJ-JAVA-033 |

### a07 Merged manifest — `evidence/audit-v0.0.5/a07-manifest.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-026 | BUG | `android.manifest.merged` opens a floating overlay with a `Close` button (and only after a build). Reference: `Text \| Merged Manifest` tabs at the bottom of the manifest editor, merged tree with source tints and a Manifest Sources / Other Manifest Files legend. | 5 | M | AND-014 |

### a08 Resource Manager — `evidence/audit-v0.0.5/a08-resource-manager.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-038 | FEATURE | Not findable (Find Action "Resource Manager" → Nothing found); no tool window (module combo, Drawable / Color / Layout / Mip Map / String tabs, resource grid). | 6 | L | AND-016 (new) |

### a11 Build Variants — `evidence/audit-v0.0.5/a11-build-variants.png`

Matches the reference (Module / Active Build Variant table, `debug` combo, hint text). No backlog item.

### a07b File > New (Android) — `evidence/audit-v0.0.5/a07b-main-menu-new.png`

| # | Type | Difference | W | Size | Row |
|---|---|---|---:|---|---|
| AUD-003 | BUG | File > New shows raw action ids as labels: `AddNewPropertyFile`, `CompactSourceFile`, `NewPackageInfo`, `NewModuleInfo`, `CreateResourceBundle`; `New Project...` / `Import Project...` are not in the submenu; Android entries (Activity ▸, Android Resource File, Vector Asset, …) are absent in an Android project. | 6 | S | IJ-MENU-005 |

### Behaviour notes (not scored separately)

- Settings opened from a command (`android.sdk.manager`, `settings.open`) and the Merged Manifest overlay do not take keyboard focus, so Escape goes to the editor and the dialog stays open; Ctrl+Alt+S + Escape works (AUD-059, weight 3).
- `state` over the control socket does not report the Settings dialog, right-anchored tool windows (Gradle, Device Manager) or the main menu while open; scenes cannot assert them.
- Gradle sync, build, run and test all succeeded through the wrapper; the Build window opens automatically after sync like the reference.

## Ranked backlog

| ID | Type | W | Size | Scene | Summary | IJ row | Issue |
|---|---|---:|---|---|---|---|---|
| AUD-031 | FEATURE | 10 | L | 23 | Java debugger: Shift+F9, breakpoints, Debug tool window, frames/variables, inline values | IJ-RUN-030/032/038/040/041 | #146 |
| AUD-001 | BUG | 8 | M | 04, 12 | jdtls/Jetthing artefacts (`.classpath`, `.project`, `.settings`, `bin/`, `.jetthing`) land in the project tree and Commit | IJ-VCS-004/005 | #147 |
| AUD-032 | FEATURE | 8 | L | 01 | Welcome screen: nav column, recent projects, Open / Clone buttons, version | IJ-MENU-100 | #148 |
| AUD-033 | FEATURE | 8 | L | 02 | New Project wizard: generators, build system, JDK, Git, sample code, advanced settings | IJ-JAVA-001 | #149 |
| AUD-034 | FEATURE | 8 | M | 19 | Project tree context menu: clipboard, Find Usages, Refactor, Reformat, Run/Debug, Open In, Git, Compare | IJ-MENU-109 | #150 |
| AUD-002 | BUG | 7 | S | 17 | Run tool window nests the configuration name twice; missing run toolbar | IJ-TW-023 | #151 |
| AUD-035 | FEATURE | 7 | L | 18 | Test runner tree with pass/fail, durations, summary bar, Show Passed/Ignored | IJ-TW-037, IJ-RUN-061 | #152 |
| AUD-036 | FEATURE | 7 | M | 04 | Code vision (usages/author), parameter inlays, one-line method folding | IJ-ED-104 | #153 |
| AUD-037 | FEATURE | 7 | L | a04 | Android layout editor (Design/Split, preview, palette, attributes) | AND-015 | #154 |
| AUD-003 | BUG | 6 | S | a07b | File > New shows raw action ids and lacks New Project / Android entries | IJ-MENU-005 | #155 |
| AUD-004 | BUG | 6 | S | 07 | File menu duplicates, disabled items and flattened submenus | IJ-MENU-104/013/008 | #156 |
| AUD-006 | BUG | 6 | M | 05 | Search Everywhere: no symbols/actions for `main`, scattered-letter matches, small popup, missing Text tab / options / footer | IJ-NAV-001/006 | #157 |
| AUD-038 | FEATURE | 6 | L | a08 | Resource Manager tool window | AND-016 | #158 |
| AUD-040 | FEATURE | 6 | M | a04 | Provision kotlin-lsp through toolchain setup like jdtls | AND-003, PLAT-016 | #159 |
| AUD-041 | FEATURE | 6 | L | 24 | Plugins Marketplace browse / install / details | IJ-SET-071/070 | #160 |
| AUD-042 | FEATURE | 6 | M | 08 | Settings stub pages: Build Tools > Gradle, Code Editing, File and Code Templates, VCS | IJ-SET-001, IJ-BLD-005 | #161 |
| AUD-005 | BUG | 5 | S | 01, 06 | Find Action: `New Project` not found on welcome; Android `Run 'app' on Device` in Java project | IJ-MENU-111 | #162 |
| AUD-007 | BUG | 5 | S | 06 | Find Action rows: inline shortcut, group column, disabled-actions checkbox, footer | IJ-MENU-111 | #163 |
| AUD-008 | BUG | 5 | M | 04 | Project tree: compact packages, test tint, excluded dirs, External Libraries, view switcher | IJ-TW-014/011 | #164 |
| AUD-015 | BUG | 5 | M | 14b | VCS widget opens an action menu instead of the Branches popup | IJ-VCS-040 | #165 |
| AUD-024 | BUG | 5 | M | a02 | Logcat tabs, toolbar, filter favourites | AND-008 | #166 |
| AUD-026 | BUG | 5 | M | a07 | Merged Manifest as editor tab with sources legend instead of overlay | AND-014 | #167 |
| AUD-039 | FEATURE | 5 | M | a03 | Running Devices tool window | AND-017 | #168 |
| AUD-043 | FEATURE | 5 | M | 10 | Find in Files popup with scope tabs and preview pane | IJ-NAV-065/066 | #169 |
| AUD-009 | BUG | 4 | S | 10 | Find in Files keeps the old query unselected; no live results | IJ-NAV-064 | #170 |
| AUD-011 | BUG | 4 | S | 04 | Gutter run icon on class line; Run with Coverage entry | IJ-RUN-011 | #171 |
| AUD-013 | BUG | 4 | S | 13 | Diff toolbar combos, per-chunk checkboxes, read-only header | IJ-VCS-050/051/012 | #172 |
| AUD-014 | BUG | 4 | S | 14 | Git log filters, toolbar, Log tab, changed-files tree | IJ-VCS-053 | #173 |
| AUD-016 | BUG | 4 | S | 15 | Run widget: `Current File` default, inline Run/Debug per configuration | IJ-MENU-103 | #174 |
| AUD-020 | BUG | 4 | S | 16 | Build window Sync / Build Output tabs, left toolbar, console links | IJ-JAVA-033 | #175 |
| AUD-025 | BUG | 4 | S | a01 | Device Manager toolbar and "Add a new device" | AND-004/005 | #176 |
| AUD-027 | BUG | 4 | M | a05 | SDK Manager: available packages, update sites, obsolete toggles | AND-001 | #177 |
| AUD-030 | BUG | 4 | S | 15b | Run/Debug Configurations dialog: Application type rows, options, coverage | IJ-RUN-001/004 | #178 |
| AUD-012 | BUG | 3 | S | 11 | Terminal tabs, blocks, cursor | IJ-TW-029 | #179 (polish: tool windows) |
| AUD-017 | BUG | 3 | S | 09 | Completion popup polish | IJ-ED-081 | #180 (polish: editor & navigation) |
| AUD-019 | BUG | 3 | S | 22 | Structure popup checkboxes and size | IJ-NAV-040 | #180 (polish: editor & navigation) |
| AUD-021 | BUG | 3 | S | 04 | Status bar widgets | IJ-MENU-106 | #180 (polish: editor & navigation) |
| AUD-022 | BUG | 3 | S | 12 | Commit toolbar and message box | IJ-TW-028 | #181 (polish: VCS) |
| AUD-023 | BUG | 3 | S | 01 | Welcome keeps IDE chrome | IJ-MENU-100 | #180 (polish: editor & navigation) |
| AUD-028 | BUG | 3 | S | a06 | Sync window tabs and phases | AND-002 | #179 (polish: tool windows) |
| AUD-029 | BUG | 3 | S | 21 | Gradle tool window toolbar, Dependencies node | IJ-BLD-004 | #179 (polish: tool windows) |
| AUD-049 | BUG | 3 | S | 20 | Problems toolbar | IJ-TW-025 | #179 (polish: tool windows) |
| AUD-053 | FEATURE | 3 | S | 04 | Window title text | IJ-MENU-104 | #180 (polish: editor & navigation) |
| AUD-058 | BUG | 3 | S | 08 | Settings polish (keymap tree, appearance options) | IJ-SET-032 | #180 (polish: editor & navigation) |
| AUD-059 | BUG | 3 | S | a05 | Command-opened dialogs do not take focus (Escape) | PLAT-025 | #180 (polish: editor & navigation) |
| AUD-061 | BUG | 3 | S | 04 | Language-server balloons oversized and persistent | PLAT-007 | #179 (polish: tool windows) |
| AUD-051 | BUG | 2 | S | 25 | Markdown toggle placement, code block style | IJ-ED-151 | #180 (polish: editor & navigation) |

Totals: 47 differences — 33 BUG, 14 FEATURE. The 33 items with weight ≥ 4 have their own issue
(#146–#178); the 14 items below weight 4 are grouped into three polish issues: #179 (tool windows),
#180 (editor & navigation), #181 (VCS).
`v0.0.6` label = weight ≥ 7 and size S/M: AUD-001, AUD-002, AUD-034, AUD-036.

Proposed new catalog rows (not added here to avoid dashboard conflicts): `AND-015` Layout editor,
`AND-016` Resource Manager, `AND-017` Running Devices.
