# IntelliJ IDEA parity: tool windows

Tool windows of the New UI and the shared tool-window mechanics (docking, view modes,
speed search, layouts). Content of each tool window is catalogued in the relevant area
file; these rows track the tool window itself.

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
| IJ-TW-001 | [manipulating-the-tool-windows](https://www.jetbrains.com/help/idea/manipulating-the-tool-windows.html) | Tool window stripes / bars with show/hide toggles | Missing | 5 | — | — | fixed panels; see PLAT-007 |
| IJ-TW-002 | [viewing-modes](https://www.jetbrains.com/help/idea/viewing-modes.html) | View modes: Dock pinned, Dock unpinned, Undock, Float, Window | Missing | 3 | — | — | not started |
| IJ-TW-003 | [manipulating-the-tool-windows](https://www.jetbrains.com/help/idea/manipulating-the-tool-windows.html) | Drag tool windows between sides; split mode | Missing | 3 | — | — | not started |
| IJ-TW-004 | [manipulating-the-tool-windows](https://www.jetbrains.com/help/idea/manipulating-the-tool-windows.html) | Resize/stretch tool windows from keyboard (Ctrl+Alt+Shift+arrows) | Missing | 1 | — | — | not started |
| IJ-TW-005 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Jump to last tool window (F12) / hide active tool window (Shift+Esc) | Missing | 2 | — | — | not started |
| IJ-TW-006 | [speed-search-in-the-tool-windows](https://www.jetbrains.com/help/idea/speed-search-in-the-tool-windows.html) | Speed search in tree and list tool windows | Missing | 3 | — | — | not started |
| IJ-TW-007 | [tool-window-layouts](https://www.jetbrains.com/help/idea/tool-window-layouts.html) | Persist tool window layout per project | Missing | 3 | — | — | not started |
| IJ-TW-010 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | Project tool window (Alt+1): file tree | Partial | 5 | `project.open`, `editor.openFile` | project tree unit test | bounded eager tree; no lazy loading, watchers, or views |
| IJ-TW-011 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | Project tool window views: Project, Packages, Project Files, Open Files, Scratches | Missing | 2 | — | — | not started |
| IJ-TW-012 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | Project tool window: file operations (new, rename, delete, copy, move, drag and drop) | Partial | 5 | `project.newFile`, `project.newDirectory`, `project.newJavaClass`, `project.newKotlinFile`, `project.rename`, `project.delete` | core command tests, Xvfb screenshots | New (Java Class, Kotlin Class/File, File, Directory/Package), Rename and Delete from the context menu; copy, move and drag and drop missing. Evidence: [new-file.md](evidence/new-file.md) |
| IJ-TW-013 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | Project tool window: Always Select Opened File / Select In (Alt+F1) | Partial | 3 | `editor.openFile` | `opening_a_nested_file_reveals_it_in_the_tree`, `reveal_scrolls_only_when_the_row_is_hidden`, scene 04 | opened files are always revealed, selected and scrolled into view; no toggle and no Select In (Alt+F1) popup yet (`evidence/find-reveal-symlinks.md`) |
| IJ-TW-014 | [file-nesting-dialog](https://www.jetbrains.com/help/idea/file-nesting-dialog.html) | Project tool window: file nesting and compact middle packages | Missing | 1 | — | — | not started |
| IJ-TW-015 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | Project tool window: VCS status coloring of files | Missing | 2 | — | — | not started |
| IJ-TW-020 | [viewing-structure-of-a-source-file](https://www.jetbrains.com/help/idea/viewing-structure-of-a-source-file.html#structure-tool-window) | Structure tool window (Alt+7) | Partial | 3 | `toolwindow.activate.structure` | structure/signature tests, Xvfb screenshot | tags-query declarations nested by containment with signatures, highlights the caret declaration, click navigates; no sorting/filters/autoscroll toggles; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-TW-021 | [bookmarks](https://www.jetbrains.com/help/idea/bookmarks.html) | Bookmarks tool window (Alt+2) | Missing | 2 | — | — | not started |
| IJ-TW-022 | [find-tool-window](https://www.jetbrains.com/help/idea/find-tool-window.html) | Find tool window (Alt+3): persisted usage / search result tabs | Partial | 3 | `search.project`, `search.cancel` | async search tests | single streamed result list; no tabs, grouping, or preview |
| IJ-TW-023 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Run tool window (Alt+4) | Partial | 5 | `run.task`, `run.cancel`, `build.project` | real process/output/cancellation tests | streamed output for one task; no tabs, rerun, or console hyperlinks Header no longer nests the configuration name twice (#151). |
| IJ-TW-024 | [debug-tool-window](https://www.jetbrains.com/help/idea/debug-tool-window.html) | Debug tool window (Alt+5) | Missing | 5 | — | — | not started |
| IJ-TW-025 | [problems-tool-window](https://www.jetbrains.com/help/idea/problems-tool-window.html) | Problems tool window (Alt+6): current file and project errors | Missing | 3 | — | — | diagnostics render inline only |
| IJ-TW-026 | [using-services-tool-window](https://www.jetbrains.com/help/idea/using-services-tool-window.html) | Services tool window (Alt+8) | Missing | 2 | — | — | not started |
| IJ-TW-027 | [version-control-integration](https://www.jetbrains.com/help/idea/version-control-integration.html) | Git tool window (Alt+9): Log, Console tabs | Missing | 3 | — | — | not started |
| IJ-TW-028 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Commit tool window (Alt+0): changes tree with stage checkboxes | Partial | 5 | `git.refresh`, `git.stage`, `git.unstage` | real repository integration test | status list with stage/unstage; no commit message editor or changelists |
| IJ-TW-029 | [terminal-emulator](https://www.jetbrains.com/help/idea/terminal-emulator.html) | Terminal tool window (Alt+F12) | Partial | 5 | `terminal.open`, `terminal.toggle`, `terminal.sendText` | real PTY process test | transcript view; no ANSI grid or tabs |
| IJ-TW-030 | [using-todo](https://www.jetbrains.com/help/idea/using-todo.html) | TODO tool window | Missing | 2 | — | — | not started |
| IJ-TW-031 | [work-with-gradle-projects](https://www.jetbrains.com/help/idea/work-with-gradle-projects.html) | Gradle tool window | Missing | 3 | — | — | not started |
| IJ-TW-032 | [maven-tab](https://www.jetbrains.com/help/idea/maven-tab.html) | Maven tool window | Missing | 3 | — | — | not started |
| IJ-TW-033 | [viewing-structure-and-hierarchy-of-the-source-code](https://www.jetbrains.com/help/idea/viewing-structure-and-hierarchy-of-the-source-code.html) | Hierarchy tool window (type / method / call) | Missing | 2 | — | — | not started |
| IJ-TW-034 | [local-history](https://www.jetbrains.com/help/idea/local-history.html) | Local History view | Missing | 2 | — | — | not started |
| IJ-TW-035 | [notifications](https://www.jetbrains.com/help/idea/notifications.html) | Notifications tool window | Missing | 1 | — | — | not started |
| IJ-TW-036 | [notifications](https://www.jetbrains.com/help/idea/notifications.html) | Event log / background tasks view | Missing | 1 | — | — | not started |
| IJ-TW-037 | [viewing-and-exploring-test-results](https://www.jetbrains.com/help/idea/viewing-and-exploring-test-results.html) | Test runner tab (tree of test results) | Missing | 5 | — | — | not started |
| IJ-TW-038 | [code-coverage](https://www.jetbrains.com/help/idea/code-coverage.html) | Coverage tool window | Missing | 2 | — | — | not started |
| IJ-TW-039 | [work-with-maven-dependencies](https://www.jetbrains.com/help/idea/work-with-maven-dependencies.html) | Dependency Analyzer tool window | Missing | 2 | — | — | not started |
| IJ-TW-040 | [database-tool-window](https://www.jetbrains.com/help/idea/database-tool-window.html) | [Ultimate] Database tool window (data sources and schema tree) | Missing | 3 | — | — | not started |
| IJ-TW-041 | [spring-projects](https://www.jetbrains.com/help/idea/spring-projects.html) | [Ultimate] Spring tool window (beans, endpoints, contexts) | Missing | 2 | — | — | not started |
| IJ-TW-042 | [endpoints-tool-window](https://www.jetbrains.com/help/idea/endpoints-tool-window.html) | [Ultimate] Endpoints tool window | Missing | 2 | — | — | not started |
| IJ-TW-043 | [persistence-tool-window](https://www.jetbrains.com/help/idea/persistence-tool-window.html) | [Ultimate] Persistence tool window (JPA) | Missing | 1 | — | — | not started |
| IJ-TW-044 | [profiler-intro](https://www.jetbrains.com/help/idea/profiler-intro.html) | [Ultimate] Profiler tool window | Missing | 2 | — | — | not started |
| IJ-TW-045 | [ai-assistant-in-jetbrains-ides](https://www.jetbrains.com/help/idea/ai-assistant-in-jetbrains-ides.html) | [Ultimate] AI Assistant chat tool window | Partial | 2 | `ai.configure`, `ai.disable`, `ai.chat.send`, `ai.chat.stop`, `ai.chat.new`, `ai.chat.copyCode`, `ai.chat.insertCode`, `ai.chat.previewApply`, `ai.chat.applyConfirm`, `ai.chat.applyCancel`, `ai.chat.setContext`, `toolwindow.activate.aiChat` | canned-stream provider tests (Anthropic SSE incl. tool use/refusal/fallback, OpenAI-compatible, Ollama), cancellation, chat/apply/policy command tests, Xvfb mock-provider walkthrough | AI Chat tool window on the right stripe: off until configured; Anthropic (default `claude-opus-5-5` / `claude-haiku-5-5`), OpenAI-compatible and Ollama; streaming Markdown, file/selection chips, slash commands, per-project history, Copy/Insert/Apply with diff preview, status bar widget; no real-provider recording yet, no layout reference; `evidence/ai-assistant.md`; agent mode in its Agent tab (AI-005) |
| IJ-TW-046 | [work-with-github-pull-requests](https://www.jetbrains.com/help/idea/work-with-github-pull-requests.html) | Pull Requests tool window (GitHub / GitLab) | Missing | 2 | — | — | not started |
| IJ-TW-047 | [feature-trainer](https://www.jetbrains.com/help/idea/feature-trainer.html) | Learn tool window | Missing | 1 | — | — | not started |
| IJ-TW-048 | [docker](https://www.jetbrains.com/help/idea/docker.html) | Docker / Services nodes for containers | Missing | 2 | — | — | not started |
| IJ-TW-090 | [project-tool-window](https://www.jetbrains.com/help/idea/project-tool-window.html) | File-type and folder icons in the Project tree, editor tabs, Find results and Recent Files | Partial | 3 | — | `icons` unit tests (extension/name table, folder roles), Xvfb screenshots | original 16px vector glyphs that carry the type: Java type circles (C/I/E/R/@, kind peeked from the source), Kotlin triangle, Gradle bricks, `{}`, `</>`, settings sheet, `M↓`, `>_`, Rust gear, full-size letter tiles (Py/JS/TS/Go/…), picture frame, zipper; config files keep a page with a corner badge; folders tinted for source (blue), test (green), resources, excluded (orange), hidden and package roles; the source/test colouring needs the `src/<set>/java\|kotlin` layout, no module configuration. Evidence: [file-type-icons.md](evidence/file-type-icons.md), [before/after](evidence/file-type-icons-2.md) |
