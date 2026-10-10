# Android Studio reference captures

- Product: Android Studio Rabbit 1 | 2026.2.1, build AI-262.9437.185.2621.16467767
  (bundled plugins report version 262.9437.185.2621.16467767), JBR 25 (`jbr-25` project JDK).
- Install: `/home/user/tools/android-studio`, SDK `/home/user/tools/sdk`.
- Display: Xvfb `:98`, 1600x1000x24, openbox. The main window was maximized by double-clicking the
  openbox title bar; Studio then fills 1600x1000 (openbox title strip at y=0..18 is part of every shot).
- Theme: default "Islands Dark", zoom 100%, New UI (hamburger main menu, compact toolbar, tool-window stripes).
- Project: `/home/user/samples/MyApplication` (AGP 9.4.1, Gradle 9.8.1 wrapper, AGP built-in Kotlin 2.2.10,
  `compileSdk 36`, `minSdk 24`, one module `:app`, `MainActivity.kt` + `activity_main.xml`).
- Captured 2026-10-10. All PNGs are full root-window captures reduced with
  `convert -strip -define png:compression-level=9 -colors 256`.
- Clean-room: screenshots are comparison evidence only. No icons or artwork may be extracted from them.

## Environment events that affect the evidence

- **Gradle sync** (first sync, triggered on open) finished successfully in 1 m 41 s but reported 3 warnings
  `Failed to resolve: org.jetbrains.kotlin:kotlin-stdlib:2.2.10`, `...kotlinx-coroutines-android:1.…`,
  `com.google.guava:listenablefuture:1.0` (see `a06-gradle-sync.png`). The cause was HTTP 429 (Too Many Requests)
  from `repo.maven.apache.org` through the container proxy, not a project problem. No AGP/Kotlin version change was
  needed: re-running `./gradlew assembleDebug` from the CLI (with `ANDROID_HOME=/home/user/tools/sdk`,
  `JAVA_HOME=<studio>/jbr`) after a pause populated the cache, and a later IDE sync (Ctrl+Shift+O) finished clean
  ("MyApplication: finished", 17 s 757 ms, no warnings). Editor highlighting and completion work (scenes 09, 20).
- **First IDE build** (Ctrl+F9) failed in 14 s: `:app:compileDebugKotlin`, `:app:checkDebugDuplicateClasses`,
  `:app:mergeDebugNativeLibs` red; root cause `Could not resolve org.jetbrains.kotlin:kotlin-build-tools-impl:2.2.10`
  / `Received status code 429 from server: Too Many Requests` (`16b-build-output-failed.png`). Rebuild after CLI
  prefetch succeeded (`16-build-output.png`).
- Opening the layout editor raised a modal **"Proxy Authentication: 127.0.0.1"** dialog (Login / Password / Remember,
  OK / Cancel). It was cancelled; the layout still rendered.
- An **"IDE error occurred / See details and submit report"** balloon appeared once (bottom-right) while the
  terminal was opened; the status bar keeps a red "!" icon at the far right afterwards.
- The **SDK Problem** dialog ("Your Android SDK is missing, out of date or corrupted." with `Open SDK Manager` /
  `Cancel`) that earlier blocked New Project from the Welcome screen did **not** appear once the project had synced:
  File > New > New Project... opened the real wizard (scenes 02/03).
- No emulator/device is possible in the container (no system image / KVM): Run on device, Logcat, Running Devices,
  App Inspection and Profiler are captured in their empty states.
- Temporary sample edits (all reverted with `git checkout` + `rm -rf app/src/test` at the end; `git status` shows only
  the pre-existing untracked `gradle/gradle-daemon-jvm.properties`):
  - `MainActivity.kt`: added line 12 `findViewById<TextView>(R.id.greeting).textSize = 24f` (scenes 09-14),
    temporary typo `greet` -> `geet` on line 11 (scene 20).
  - `app/build.gradle.kts`: `testImplementation("junit:junit:4.13.2")`; new
    `app/src/test/java/com/example/myapplication/GreetingTest.kt` with two JUnit4 tests (scenes 18, 23).

## Persistent window chrome (visible in most shots)

- Top toolbar, left to right: hamburger (opens a horizontal menu bar File Edit View Navigate Code Refactor Build Run
  Tools Git Window Help in place of the toolbar), project widget `MA My Application ⌄` (colored 2-letter badge),
  VCS widget `⑂ master ⌄`; centre-right: device selector `No Devices ⌄` (rounded chip), run-config selector
  `<icon> app ⌄`, Run ▷ (green), Debug (green bug), `⋮` more; right: Sync Project with Gradle Files, Apply
  Changes / code actions, a profiler/attach icon, Gemini/agent icon, Search (magnifier), Settings (gear), account.
- Left tool stripe (top group): Project, Commit, Resource Manager, `…` (more tool windows). Bottom group (from top):
  Build (hammer), App Quality Insights (diamond), Logcat (cat), Problems (!), Terminal, Git. Tool windows opened
  later (Debug, Run, App Inspection, Profiler, Build Variants) get added to the stripe dynamically.
- Right tool stripe: Notifications (bell, top, red dot when unread), Gradle (elephant), Device Manager, Running
  Devices, Agent/Gemini.
- Status bar: left = navigation breadcrumb (`MyApplication > app > src > main > java > com > example >
  myapplication > MainActivity > onCreate`), right = background task text + progress bar ("Analyzing project…",
  "Gradle Build Running" with ⓧ cancel), caret `12:61`, a "line separator" toggle icon, `LF`, `UTF-8`,
  indent `4 spaces`, read-only lock, error indicator.
- Window title: `MyApplication – MainActivity.kt [My_Application.app.main]` (file + Gradle source set).

## Scenes

### 01 Welcome — `01-welcome.png`
Steps: launch Studio with no project (`as-start.sh` without args). Window was 800x650 centered (not maximized).
- Title "Welcome to Android Studio (ROOT)". Left column: app logo + "Android Studio / Rabbit 1 | 2026.2.1",
  nav list `Projects` (selected, blue pill), `Customize`, `Plugins`, `Learn`; gear icon bottom-left.
- Right: heading "Welcome to Android Studio (ROOT)", two grey lines "Create a new project to start from scratch." /
  "Open existing project from disk or version control.", three square icon buttons with captions `New Project`
  (focused, blue outline), `Open`, `Clone Repository`, then link `More Actions ⌄`. No recent-project list (none yet).

### 02 New Project wizard, template page — `02-new-project.png`
Steps: in the open project, hamburger > File > New > New Project... (scene 07 shows the submenu).
- Modal "New Project" ~900x670. Left list: header "Templates", entry `✦ Create with AI...`, separator, form factors
  `Phone and large screens` (selected), `Wear OS`, `Television`, `Car`, `XR`.
- Right: 3-column grid of template cards with phone-shaped thumbnails and captions: `No Activity`,
  `Empty Activity` (selected: darker card background), `Gemini API` (purple "Preview" badge), `Navigation UI
  Activity`, `Basic Views Activity`, `Empty Views Activity`, `Responsive Views Activity`, `Game Activity (C++)`,
  `Native C++`, more below (scrollbar).
- Footer buttons: `Previous` (disabled), `Next` (primary blue), `Cancel`, `Finish` (disabled).

### 03 New Project wizard, details page — `03-new-project-details.png`
Steps: on 02 click `Next`.
- Heading "Empty Activity", subtitle "Create a new empty activity with Jetpack Compose".
- Form (label column left, field column right): `Name` = "My Application" (focused), `Package name` =
  "com.example.myapplication", `Save location` = "/root/AndroidStudioProjects/MyApplication" with folder button
  inside the field, `Minimum SDK` combo "API 24 ("Nougat"; Android 7.0)", info line "ⓘ Your app will run on
  approximately **99.2%** of devices." + link "Help me choose", `Build configuration language` (ⓘ help icon) combo
  "Kotlin DSL (build.gradle.kts) [Recommended]".
- No language (Kotlin/Java) selector and no JDK selector on this template (Compose template is Kotlin-only; the
  Gradle JDK is not chosen here). Labels have mnemonic underlines (N, P, S).
- Footer: `Previous`, `Next` (disabled), `Cancel`, `Finish` (primary). The wizard was cancelled.

### 04 Editor + project tree — `04-editor-project-tree.png`
Steps: Ctrl+Shift+N, type `MainActivity`, Enter; Alt+F1 then `1 Project View` (Select In popup) to reveal the file.
- Project tool window header: view selector `Android ⌄` (Android view, not Project view), actions `+` (new),
  locate/crosshair, expand all, collapse all, `⋮`, `—` (hide).
- Tree: `app` > `manifests`, `kotlin+java` > `com.example.myapplication` > `MainActivity` (selected, no `.kt`
  extension shown for a single-class Kotlin file), `res`; `Gradle Scripts`.
- Editor tab `MainActivity.kt ×` (Kotlin class icon). Gutter: line numbers, run ▷ and `</>` (layout link) markers on
  line 7, override marker on line 8; imports folded as `import ...`; inlay hints `name =` before argument and
  "1 Usage" code-vision lines above `class` and `private fun greet`. Unresolved-looking but valid R references
  (`activity_main`, `greeting`, `text`) are italic + underlined (synthetic/Java property style). Warning counter
  `⚠1 ˄ ˅` top-right of editor; right margin guide line at ~col 120; scrollbar marker stripe.

### 05 Search Everywhere — `05-search-everywhere.png`
Steps: focus editor, press Shift twice, type `main`.
- Centered popup ~700x600. Tabs `All` (selected) `Classes` `Files` `Symbols` `Actions` `Text`; checkbox
  "Include non-project items"; right icons (preview, filter, open in tool window).
- Search field with magnifier and hint "Type / to see commands" on the right.
- Mixed results with match highlighted (yellow background on "Main"): actions (`Maintenance`, `Main Menu`, `Main
  Toolbar`, `Focus Main Toolbar Alt+Page Up`, `Build Project`, toggle row `Update Loaded Classes On Debugger Stop`
  with an `OFF` switch), class `MainActivity com.example.myapplication` with module `My_Application.app.main` right,
  resource `activity_main (layout)`, file `activity_main.xml app/src/main/res/layout`, and text hits with file:line on
  the right (`AndroidManifest.xml 4`, `gradlew 60`, `MainActivity.kt 7`...). Footer hint "Assign Shortcut Alt+Enter".
- Ctrl+Shift+N (Go to File) is the same popup with the `Files` tab preselected and a `Project Files ⌄` scope combo;
  bottom bar shows the selected path and `Open In Right Split`; while indexing the field shows "Analyzing project…".

### 06 Find Action — `06-find-action.png`
Steps: Esc, Ctrl+Shift+A, type `run`.
- Same popup with `Actions` tab selected; checkbox "Include disabled actions".
- Rows: action name + shortcut in small grey text + group on the right: `Run`, `Run 'app' Shift+F10 — Run`,
  `Run Alt+4`, `Run... Alt+Shift+F10 — Run`, `Run/Debug`, `Run Inspection by Name... Ctrl+Alt+Shift+I — Code |
  Analyze Code`, `Run Configuration Templates for New Projects... — File`, `Run Without Build`, `Run Code Migrations`,
  `Run Widget`, `Run Configurations` (x2), `Run 'MainActivity' Ctrl+Shift+F10`, `Run VFS Health Check`,
  `Running Devices`, `Run Anything Ctrl Ctrl`, `Run Dashboard (Services)`, `Run Generate Sources Gradle Tasks`,
  `Run Configuration...`, `Remove Unused Resources... — Refactor`. Run actions carry the green ▷ icon.

### 07 Main menu — `07-main-menu.png`
Steps: click hamburger (top-left), hover `File`, hover `New`.
- Hamburger turns into a horizontal menu bar File/Edit/View/Navigate/Code/Refactor/Build/Run/Tools/Git/Window/Help
  (mnemonic underlines). File menu: `New >`, `Open...`, `Profile or Debug APK`, `Recent Projects >`,
  `Close Project`, `Add C++ to Module`, `Settings... Ctrl+Alt+S`, `Project Structure... Ctrl+Alt+Shift+S`,
  `Plugins...`, `File Properties >`, `Local History >`, `Save All Ctrl+S`, `Sync Project with Gradle Files
  Ctrl+Shift+O`, `Refresh Linked C++ Projects` (disabled), `Reload All from Disk Ctrl+Alt+Y`, `Repair IDE`,
  `Invalidate Caches...`, `Manage IDE Settings >`, `New Projects Setup >`, `Export >`, `Print...`,
  `Power Save Mode`, `Exit`.
- New submenu: `New Project...`, `Import Project...`, `Project from Version Control...`, `New Module...`,
  `Import Module...`, `Import Sample...`, `Java Class`, `Kotlin Class/File`, `Generate Code From Image`,
  `Android Resource File`, `Android Resource Directory`, `Sample Data Directory`, `File`, `Package`,
  `Image Asset`, `Vector Asset`, `module-info.java`, `CMakeLists.txt`, then submenus `Activity`, `Fragment`,
  `Folder`, `Service`, `UiComponent`, `Car`, `XML`, `Wear`, `TV`, `AIDL`, `Widget`, `Google`, `Compose`, `Other`,
  then `Resource Bundle`, `Version Catalog`.

### 08 Settings — `08-settings.png`
Steps: Ctrl+Alt+S (opens on Appearance & Behavior > Appearance).
- Modal "Settings – My Application" ~900x700. Left: search field, tree `Appearance & Behavior` (expanded:
  `Appearance` selected, `Menus and Toolbars`, `System Settings >`, `File Colors`, `Scopes`, `Notifications`,
  `Data Editor and Viewer`, `Quick Lists`, `Required Plugins`, `Trusted Locations`, `Path Variables`,
  `Presentation Assistant`), `Keymap`, `Editor` (blue dot = modified), `Plugins`, `Version Control`,
  `Build, Execution, Deployment`, `Languages & Frameworks`, `Tools`, `Backup and Sync`, `Advanced Settings`,
  `Other Settings`. Small "project-level" page icons at the right of some rows.
- Right: breadcrumb `Appearance & Behavior › Appearance`, back/forward arrows, pin. `Theme: Islands Dark ⌄`,
  `Editor color scheme: Islands Dark Theme defa… ⌄` + gear, checkbox "Different tool window background" with grey
  help text. Section "Accessibility": `Zoom: 100% ⌄` + hint "Change with Ctrl+Alt+Shift+= or Ctrl+Alt+Shift+Minus…",
  "Use custom font: Inter / Size: 13" (disabled), "Support screen readers (Requires restart)", "Use contrast
  scrollbars", "Adjust colors for red-green vision deficiency" + link "How it works ↗". Section "UI Options" two
  columns: Compact mode, Always show full path in window header, Use project colors in main toolbar ✓,
  Drag-and-drop with Alt pressed only, Smooth scrolling ✓ (?), Enable mnemonics in controls ✓, Enable mnemonics in
  menu ✓.
- Footer: `?` help, `OK` (primary), `Cancel`, `Apply` (disabled until change).

### 09 Code completion — `09-completion.png`
Steps: click end of line 11, End, Enter, type `findViewById<TextView>(R.id.greeting).se`, Ctrl+Space
(auto-popup did not appear after `.` because typing continued; explicit Ctrl+Space works).
- Popup anchored under the caret, ~500x310, monospace rows: kind icon (m = method, v = property), bold matched
  prefix (`set…`), parameter list in grey, return type right-aligned (`Unit`, `ActionMode.Callback!`). Entries
  `setEms(ems: Int)`, `setHint(resid: Int)`, `setHint(hint: CharSequence!)`, `setError(...)` x2,
  `setActivated(activated: Boolean)`, `setText(...)` x4, `customInsertionActionModeCallback …` (camel-hump /
  middle matching), …
- Footer: "Press Enter to insert, Tab to replace" + `⋮` menu. Platform types shown with `!`.
- The line was then completed to `textSize = 24f` and saved (Ctrl+S) for the VCS scenes.

### 10 Find in Files — `10-find-in-files.png`
Steps: Ctrl+Shift+F, Ctrl+A, type `greeting`.
- Floating (non-modal) popup ~650x870. Header "Find in Files  3 matches in 2 files", "File mask: [*.java] ⌄"
  checkbox, filter and pin icons. Search field with clear ×, regex/newline icon, `Cc` (match case), `W` (words),
  `.*` (regex).
- Scope buttons `In Project` / `Module` / `Directory` / `Scope` (selected) + combo `Project Files ⌄` + `...`.
- Results table: matched line text with match highlighted (orange), right column file + line (`MainActivity.kt 11`,
  `MainActivity.kt 12`, `activity_main.xml 6`). Results update live while typing.
- Lower half: live editable preview of the selected file (`MainActivity.kt app/src/main/…`) with full highlighting and
  the match line highlighted. Footer: checkbox "Open results in new tab", `Ctrl+Enter`, button `Open in Find Window`.

### 11 Terminal — `11-terminal.png`
Steps: Alt+F12, type `ls --color` Enter, `git log --oneline` Enter (typed with 100 ms delay; fast typing was
garbled into `git log --o--oneline` by the shell-completion feature).
- Bottom tool window header: "Terminal", session tab `Local ×` (blue pill), `+` new tab, `⌄` (shell list); right:
  `AI Agents ⌄`, `⋮`, `—`.
- New block-style terminal: each command + output is a block separated by a thin horizontal rule; prompt
  `root@vm:/home/user/samples/MyApplication#`; ANSI colors from `ls --color` (dirs blue, `gradlew` green bold) and git
  (`(HEAD -> master)` coloured). Output: `aebf3c6 (HEAD -> master) init`, `15139f0 init`. Vertical scrollbar right.
- Shift+Esc hides the tool window; Esc moves focus to the editor (so Esc+Enter does not run the command).

### 12 Commit — `12-commit.png`
Steps: after editing MainActivity.kt, Alt+0; tick the file checkbox; type "Enlarge greeting text" in the message.
- Commit tool window replaces the Project window in the left area. Toolbar: refresh, rollback, show diff, update,
  shelve-ish icons, eye (view options), locate, expand/collapse.
- Tree: `☑ Changes 1 file` > `☑ MainActivity.kt app/src/main/java/com/example/myapplica…` (blue = modified);
  `☐ Unversioned Files 1 file`.
- Below: `☐ Amend last commit ⌄` (dropdown), history (clock) and AI-generate (pen) icons, right "1 modified" link;
  multi-line message editor with placeholder "Commit Message" (monospace); buttons `Commit` (primary) and
  `Commit and Push...`, gear (commit options) at right.
- Editor gutter shows a green change bar on the added line 12.

### 13 Diff — `13-diff.png`
Steps: in 12, double-click `MainActivity.kt`.
- Opens as an editor tab `Commit: MainActivity.kt` (not a dialog). Toolbar: ↑ ↓ (prev/next difference), pencil (edit
  source), `← 1 file →`, collapse-unchanged icon; right "1 difference", side-by-side / unified toggle pair, gear.
- Header: left `aebf3c62 app/src/main/java/com/example/myapplication/MainActivity.kt`, right `☑ Current version`.
- Side-by-side panes, both with full syntax highlighting; centre column with both line-number gutters; the
  inserted line 12 is green on the right with `»` (apply) arrow and a checkbox (partial-commit selection) in the
  gutter; left pane shows a thin green connector at the insertion point. Inlay/gutter markers (`</>`, override)
  appear on the right (live) side only.

### 14 Git log — `14-git-log.png`
Steps: close diff tab, Alt+9, click the top commit.
- Bottom tool window "Git" with tabs `Log` (selected), `Console ×`, `+`, `⌄`.
- Three panes: branches (search "Branch or tag", `HEAD (Current Branch)`, `Local` > `master` with tag icon; vertical
  action strip with +, delete, update, search, star, gear), commit table (filter bar "Text or hash" with `.*`/`Cc`,
  `Branch ⌄`, `User ⌄`, `Date ⌄`, `Paths ⌄`, sort, refresh, cherry-pick, eye, search; rows with graph dot, subject
  `init`, branch label `master`, author `sample`, `11 minutes ago`), and details (changed-files tree
  `My_Application 4 files` > `gradle/wrapper 2 files` > files; below: subject, `aebf3c62 sample <a@b> on 10/10/26 at
  1:23 PM`).

### 15 Run configuration — `15-run-config.png`
Steps: click the `app ⌄` run widget, then `Edit Configurations...`.
- The widget popup lists the configuration row `app` with inline ▷ run, debug and `⋮` buttons, then
  `Edit Configurations...` (no separate templates section).
- Dialog "Run/Debug Configurations" ~870x670. Left toolbar `+ — copy folder sort`; tree `Android App` > `app`;
  link `Edit configuration templates...` bottom-left.
- Right: `Name: app`, `☑ Allow multiple instances`, `☐ Store as project file`; tabs `General` / `Miscellaneous` /
  `Debugger`; `Module: My_Application.app ⌄`; "Installation Options": `Deploy: Default APK ⌄`, three checkboxes (install
  for all users, always install with package manager, clear app storage), `Install Flags:` placeholder "Options to
  'pm install' command"; "Launch Options": `Launch: Default Activity ⌄`, `Launch Flags:` placeholder "Options to 'am
  start' command"; collapsible "Before launch" list. Footer: `?`, split button `▷ Run ⌄`, `OK`, `Cancel`, `Apply`.

### 16 Build output — `16-build-output.png` (+ `16b-build-output-failed.png`)
Steps: Ctrl+F9 (Make Project). Build window does not auto-open on success; it opened here because it had been
opened before. Tabs `Sync ×`, `Build Output ×`, `Build Analyzer ×`.
- Left: toolbar column (rebuild, stop, pin, eye/filter); tree `✓ Build MyApplication: finished At 10/10/26, 1:37 PM
  3 sec, 664 ms` > `Download info`. Right: raw Gradle console (monospace) ending "BUILD SUCCESSFUL in 3s", "34
  actionable tasks: 2 executed, 32 up-to-date", URLs as underlined links, final line "Build Analyzer results
  available" (link). Right icon column: soft-wrap, scroll-to-end, clear.
- Failure variant: root `❗ Build MyApplication: failed At …` with children per failing task
  (`:app:compileDebugKotlin 5 sec, 627 ms`, `:app:checkDebugDuplicateClasses`, `:app:mergeDebugNativeLibs`), console
  shows `BUILD FAILED in 14s` in red.

### 17 Run output — `17-run-output.png`
Steps: click Run ▷ with run config `app` and `No Devices`.
- No Run tool window opens. A red error balloon appears bottom-right: "Error running 'app'" / "No target device
  found." Run cannot proceed without a device in this container. The Run tool window with output is shown in 18
  (unit-test run).

### 18 Test results — `18-test-results.png`
Steps: open `GreetingTest.kt` (temporary), caret on line 1, Ctrl+Shift+F10 (runs the class via Gradle
`testDebugUnitTest --tests ...`); then toggle "Show Passed" (✓ icon) and expand all (numpad *).
- Run config widget switches to `GreetingTest` (Gradle icon); while running the toolbar shows a rerun icon and a
  red Stop button.
- Run tool window, tab `GreetingTest ×`. Toolbar: rerun, rerun failed, toggle auto-test, stop, eye, ✓ show passed
  (toggled), ⊘ show ignored, sort, import/export, history clock, `⋮`.
- Tree: `✓ Test Results 4 sec 469 ms` > `✓ GreetingTest 21 ms` > `✓ greetsAndroid 4 ms`, `✓ greetsWorld 0 ms`.
  Right: "✓ 2 tests passed  2 tests total, 4 sec 469 ms" with full-width green progress bar, console
  "Executing tasks: [:app:testDebugUnitTest, --tests, com.example.myapplication.GreetingTest] in project
  /home/user/samples/MyApplication" followed by `> Task :app:… UP-TO-DATE` lines.
- Editor gutter run icons change to green check-run icons after a pass. Initially (before toggling) passed tests
  were hidden and only the root node showed.

### 19 Project context menu — `19-project-context-menu.png`
Steps: right-click `MainActivity` in the Android project view.
- Items: `New >`, `Add C++ to Module`, `Cut Ctrl+X`, `Copy Ctrl+C`, `Copy Path/Reference...`, `Paste Ctrl+V`,
  `Find Usages Alt+F7`, `Analyze >`, `Rename... Shift+F6`, `Refactor >`, `Bookmarks >`, `Reformat Code Ctrl+Alt+L`,
  `Optimize Imports Ctrl+Alt+O`, `Delete... Delete`, `Override File Type`, `▷ Run 'MainActivity'
  Ctrl+Shift+F10`, `Debug 'MainActivity'`, `Modify Run Configuration...`, `Open in Right Split Shift+Enter`,
  `Open In >`, `Local History >`, `Git >`, `Repair IDE on File`, `Reload from Disk`, `Compare With... Ctrl+D`,
  `Mark File As >`. Separators group the items; mnemonics underlined.
- Test source package `com.example.myapplication (test)` is shown with a green-tinted row background.

### 20 Problems / error highlighting — `20-problems.png`
Steps: change `greet(` to `geet(` on line 11, Alt+6, hover the error.
- Error identifier rendered in red text; editor widget top-right `❗1 ⚠1 ˄ ˅`; red stripe mark in the scrollbar.
  Error propagates as red wavy underline to the editor tab name, the project-tree file, its package and `app` node.
- Hover tooltip: "Unresolved reference 'geet'." with `⋮`, quick fixes row `Create function 'geet' Alt+Shift+Enter`
  and `More actions... Alt+Enter`.
- Problems tool window tabs `File 2` (selected, count badge) and `Project Errors`; row
  `MainActivity.kt /home/user/samples/…/myapplication 2 problems` > `❗ Unresolved reference 'geet'. :11`,
  `⚠ Function "greet" is never used :15`. Left toolbar: eye (options), bulb (quick fix), preview toggle.
- For a layout file the Problems window adds a `Layout and Qualifiers` tab ("No layout problems in
  activity_main.xml and qualifiers").

### 21 Gradle tool window — `21-gradle-tool-window.png`
Steps: click the elephant icon in the right stripe; expand `Tasks` and `app`.
- Toolbar: reload (download arrow), new task/run, offline toggle (cloud), search tasks, expand/collapse, settings
  gear, info "ⓘ Task list not built..." link. Clicking the link opens Settings > Experimental showing "Configure all
  Gradle tasks during Gradle Sync (this can make Gradle Sync slower)" unchecked plus a tooltip "Only test tasks are
  configured during Gradle Sync...".
- Tree: `My Application` > `Tasks` > `other` > `testDebugUnitTest`; `Run Configurations`; `app` > `Tasks` > `other` >
  `testDebugUnitTest` (task rows have a gear icon). The full task list is NOT shown by default in Studio.

### 22 File structure — `22-structure.png`
Steps: focus editor, Ctrl+F12.
- Popup ~540x540 titled `MainActivity.kt`, checkbox "Inherited members (Ctrl+F12)" and gear. Tree
  `MainActivity` > `greet(String): String` (private lock icon), `onCreate(Bundle?): Unit` (override icon, blue
  text). Typing filters with speed search.

### 23 Debugger — `23-debugger.png`
Steps: click gutter of `GreetingTest.kt` line 12 to add a breakpoint, click Debug (bug) with config `GreetingTest`.
- Execution line highlighted blue; breakpoint red dot with check; inline values in italic grey after code
  (`message: "Hello Android!"`).
- Debug tool window, tab `GreetingTest ×`. Toolbar: rerun, rerun failed, stop, eye, resume ▷|, pause, step over,
  step into, step out, view/mute breakpoints, `⋮`; segmented tabs `Threads & Variables` (selected) / `Console`;
  layout icon at right.
- Frames pane: thread combo `✓ "Test worker"…main": RUNNING` with filter, frame `greetsAndroid:12, GreetingTest
  (com.example.myapplication)`, "50 hidden frames"; hint "Switch frames from anywhere in the IDE with Ctrl+…".
- Variables pane: expression field "Evaluate expression (Enter) or add a watch (Ctrl+Shift+Enter)" with language
  `Kotlin ⌄`; rows `this = {GreetingTest@2550} com.example.myapplication.GreetingTest@1cdb90a3`, `message = "Hello
  Android!"`.
- After F9 the tests finish and the Debug window shows the test tree like 18.

### 24 Plugins — `24-plugins.png`
Steps: in Settings, click `Plugins`, dismiss "Plugins Auto-Update" got-it balloon, click `Installed`.
- Header `Plugins` with segmented tabs `Marketplace` / `Installed` (selected) and gear. Marketplace initially shows
  a spinner and "Select plugin to preview details".
- Left list (search "Type / to see options" + filter icon), grouped with group headers + `Disable all` links:
  "Android": `Android`, `Android Design Tools`, `Jetpack Compose`, `Smali Support`; "Build Tools": `Artifacts
  Repository Sea...`, `Gradle`, `Gradle for Java` … Each row: plugin icon, bold name, version
  `262.9437.185.2621.16467767`, checkbox (greyed for bundled required ones).
- Right details: name `Android`, `Enable` button (disabled), version, tabs `What's New` / `Reviews` / `Additional
  Info`, `Overview` selected, description "Supports the development of Android applications with IntelliJ IDEA and
  Android Studio."

## Android-specific scenes

### a01 Device Manager — `a01-device-manager.png`
Steps: Find Action "Device Manager" (also right stripe icon).
- Right-docked tool window. Toolbar: device-group toggle, `+` (add device), Wi-Fi pairing, Firebase test lab icon.
  Table header `Name ˄`, `API`, `Type`. Empty state centered: "No devices connected." + link "Add a new device...".

### a02 Logcat — `a02-logcat.png`
Steps: Find Action "Logcat".
- Bottom tool window "Logcat", tab `Logcat ×`, `+` for additional logcat tabs.
- Device combo "No connected devices ⌄" (focused) on the left; filter field with funnel icon and query token
  `package:mine` (highlighted as a token), clear ×, `Cc`, ☆ (favorite filter), `?` help.
- Left vertical toolbar: clear, pause, restart, scroll-to-end (active), up/down, soft-wrap, `>` overflow. Log area empty.

### a03 Running Devices — `a03-running-devices.png`
Steps: Find Action "Running Devices".
- Right-docked window, header "Running Devices" with `+`, detach (↗), `⋮`, `—`. Empty state illustration (devices)
  and text "To mirror a physical device, connect it via USB cable or over WiFi, click + and select the device from the
  list. You may also select the **Activate mirroring when a new physical device is connected** option in the Device
  Mirroring settings." and "To launch a virtual device, click + and select the device from the list, or use the
  Device Manager." (links).

### a04 Layout editor — `a04-layout-editor.png`
Steps: Ctrl+Shift+N `activity_main.xml`; cancel the proxy-auth dialog; click Split icon (editor top-right
`☰ / ⧉ / ▣` = Code / Split / Design); hide Project window (Shift+Esc).
- Design mode (first state, not saved separately): `Palette` (categories Common/Text/Buttons/Widgets/Layouts/
  Containers/Helpers/Google/Legacy; Common items `Ab TextView`, `Button`, `ImageVi…`, `Recycler…` ⤓, `Fragme…` ⤓,
  `ScrollView`, `Switch`), `Component Tree` (`FrameLayout` > `Ab greeting`), `Attributes` panel right, two canvases
  (white Design surface + teal Blueprint), zoom controls (pan hand, +, −, 1:1, fit) bottom-right.
- Split mode (captured): XML editor left (≈55%), preview right; Palette and Component Tree collapse to vertical
  side tabs; design toolbar `activity_main.xml ⌄`, design/blueprint toggle, orientation, night-mode, `Pixel ⌄`,
  API `36 ⌄`, theme `DayNight ⌄`, locale `Default (en-us) ⌄`, error/issue badge (red !) at far right; secondary row
  with eye, ↔, ↕ and `?`. Attributes panel collapsed into a right vertical tab.
- The TextView has no text, so the rendered surface is blank white; the blueprint shows the TextView as a small
  caret-like outline in the centre.

### a05 SDK Manager — `a05-sdk-manager.png`
Steps (earlier capture): Settings > Languages & Frameworks > Android SDK.
- Breadcrumb `Languages & Frameworks › Android SDK`; "Manager for the Android SDK and Tools used by the IDE";
  `Android SDK Location: /home/user/tools/sdk` with `Edit` and `Optimize disk space` links; tabs `SDK Platforms`
  (selected) / `SDK Tools` / `SDK Update Sites`; explanatory paragraph; table columns `Name | API Level | Revision |
  Status` with checkbox per row (`Android CANARY Preview canary-20261007 17 Not installed`, `Android 17.0
  ("CinnamonBun") 37.2`, …, `Android 16.0 ("Baklava") 36.0 2 Partially installed`, …); checkboxes "Hide incompatible
  system images" ✓, "Hide obsolete packages" ✓, "Show package details" ☐. Footer note "Project-level settings will
  be applied to new projects".

### a06 Gradle sync — `a06-gradle-sync.png`
Steps: Build tool window > `Sync` tab, click the first warning.
- Tree root `⚠ MyApplication: finished At 10/10/26, 1:27 PM wi… 1 min, 43 sec, 668 ms` > `Download info`, three
  `⚠ Failed to resolve: …` rows. Right pane for the selected warning: red "Failed to resolve:
  org.jetbrains.kotlin:kotlin-stdlib:2.2.10" and links `Show in Project Structure dialog`, `Show Details`,
  `Affected Modules: app`, `Fix with AI`. Left toolbar: re-sync, stop, pin, eye.
- After CLI prefetch, a later sync was clean (`✓ MyApplication: finished … 17 sec, 757 ms`, console "BUILD
  SUCCESSFUL in 15s"), not captured separately.

### a07 Merged manifest — `a07-manifest.png`
Steps: Ctrl+Shift+N `AndroidManifest`, click bottom editor tab `Merged Manifest` (tabs `Text` / `Merged Manifest`).
- Left: read-only collapsible XML tree of the merged manifest; each line background-tinted by its source
  (`versionCode/versionName/package/uses-sdk` from build.gradle.kts injection, `permission
  …DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` and `appComponentFactory` from `core:1.13.0`, the rest from the main
  manifest).
- Right: "Manifest Sources" legend with colour swatches (`My_Application.app main manifest (this file)`,
  `core:1.13.0 manifest`, `build.gradle.kts injection`), then "Other Manifest Files (Included in merge, but did not
  contribute any elements)" listing ~22 AndroidX library manifests as links.
- In the `Text` tab `android:label="@string/app_name"` is displayed folded as `"My Application"` (resource value
  folding).

### a08 Resource Manager — `a08-resource-manager.png`
Steps: Find Action "Resource Manager", click tab `String`.
- Left tool window "Resource Manager": `+` (add resources), refresh, `Module: My_App ⌄`, search field, filter.
  Tabs `Drawable`, `Color`, `Layout`, `Mip Map`, `String` (selected), `⋮` overflow. Group `My_Application.app.main
  (1)` collapsible; item `app_name` / `My Application  1 version`. Drawable tab empty state "No drawable available".
  Bottom-right: list/grid view toggles and zoom −/+.

### a09 App Inspection — `a09-app-inspection.png`
Steps: Find Action "App Inspection".
- Bottom tool window "App Inspection", process combo `▯ No Process Available ⌄`; centred text "Deploy a debuggable
  app to a device running API level 26 or higher, or select a debuggable process to inspect." and `? Learn More ↗`.
  A new App Inspection icon is added to the left stripe.

### a10 Profiler — `a10-profiler.png`
Steps: Find Action "Profiler".
- Bottom tool window "Profiler" with tabs `Home` (selected) / `Past Recordings`. Left: "No Devices" header and text
  "No device selected. You can select a device via the Devices dropdown at the main window toolbar."
- Right "Tasks" (gear): cards `Capture System Activities / System Trace` (selected, blue background),
  `View Live Telemetry`, `Find CPU Hotspots / Java/Kotlin Method Recording`, `Find CPU Hotspots / Callstack Sample`,
  `Analyze Memory Usage / Heap Dump`. Bottom: "Start profiler task from `Not specified ⌄`", error "❗ No valid device
  is selected", button `Start profiler task` (disabled).

### a11 Build Variants — `a11-build-variants.png`
Steps: Find Action "Build Variants", click the variant cell, click its arrow.
- Left tool window "Build Variants": table `Module | Active Build Variant`, row `:app` / `debug (default) ⌄`; the
  cell turns into a combo whose popup lists `debug (default)` (selected) and `release`. Bottom-right button
  `Re-import with defaults` (disabled).

## Skipped / partial scenes

- None of 01–24 / a01–a11 skipped. Partial: 17 (no device, so Run shows only the "No target device found" balloon
  instead of app output); a01/a02/a03/a09/a10 are empty states (no emulator possible); 01 was captured before the
  window was maximized (Welcome window is a fixed-size 800x650 frame).
