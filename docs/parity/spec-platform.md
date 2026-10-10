# Spec features not covered by the catalog: platform.md

Rows for feature ids in [`docs/spec/features/platform.md`](../spec/features/platform.md)
that no IntelliJ catalog row links in its `Spec` column. `cargo run -p xtask -- parity
--sync-spec` appends a `Missing` row here for every new uncovered id; edit statuses and
evidence in place. Weights follow the spec priority: P0 = 5, P1 = 3, P2 = 1. Rows tagged
`[Ultimate]` are paid features in the spec. Keep the table last in this file.

| ID | Reference | Feature | Status | Weight | Spec | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|---|
| PLATFORM-EDIT-015 | [platform.md](../spec/features/platform.md) | Virtual space | Missing | 1 | `PLATFORM-EDIT-015` | — | — | not started (P2) |
| PLATFORM-EDIT-017 | [platform.md](../spec/features/platform.md) | Insert/Overwrite toggle (`EditorToggleInsertState`) | Missing | 3 | `PLATFORM-EDIT-017` | — | — | not started (P1) |
| PLATFORM-EDIT-019 | [platform.md](../spec/features/platform.md) | Scroll without moving caret (`EditorScrollUp/Down`, `EditorScrollToCenter`) | Missing | 3 | `PLATFORM-EDIT-019` | — | — | not started (P1) |
| PLATFORM-EDIT-020 | [platform.md](../spec/features/platform.md) | Jump outside bracket/quote (`BraceOrQuoteOut`) | Missing | 3 | `PLATFORM-EDIT-020` | — | — | not started (P1) |
| PLATFORM-EDIT-036 | [platform.md](../spec/features/platform.md) | Delete to word start/end (`EditorDeleteToWordStart/End`) | Partial | 5 | `PLATFORM-EDIT-036` | — | `word_navigation_and_selection` | Ctrl+Backspace / Ctrl+Delete (Alt on macOS) delete to word start/end in editor input; not a registered command |
| PLATFORM-EDIT-044 | [platform.md](../spec/features/platform.md) | Sort / Reverse Lines (`EditorSortLines`, `EditorReverseLines`) | Implemented | 3 | `PLATFORM-EDIT-044` | `editor.sortLines`, `editor.reverseLines` | `sorts_and_reverses_selected_lines_or_whole_file` | selected lines or whole file, code-point order (reference collation unverified); see PLAT EDIT-044 (P1) |
| PLATFORM-EDIT-045 | [platform.md](../spec/features/platform.md) | Fill Paragraph (`FillParagraph`) | Missing | 1 | `PLATFORM-EDIT-045` | — | — | not started (P2) |
| PLATFORM-EDIT-048 | [platform.md](../spec/features/platform.md) | Convert Indents (`ConvertIndentsToSpaces`, `ConvertIndentsToTabs`) | Missing | 3 | `PLATFORM-EDIT-048` | — | — | not started (P1) |
| PLATFORM-EDIT-051 | [platform.md](../spec/features/platform.md) | Emoji & Symbols picker (`EmojiPicker.Open`) (plugin) | Missing | 1 | `PLATFORM-EDIT-051` | — | — | not started (P2) |
| PLATFORM-EDIT-082 | [platform.md](../spec/features/platform.md) | Image and binary viewers (`Images.Editor.*`) | Missing | 1 | `PLATFORM-EDIT-082` | — | — | not started (P2) |
| PLATFORM-EDIT-083 | [platform.md](../spec/features/platform.md) | Hex/byte view of binary files (inferred) | Missing | 1 | `PLATFORM-EDIT-083` | — | — | not started (P2) |
| PLATFORM-EDIT-114 | [platform.md](../spec/features/platform.md) | Multi-line search field and search history (`ShowSearchHistory`) | Missing | 3 | `PLATFORM-EDIT-114` | — | — | not started (P1) |
| PLATFORM-EDIT-115 | [platform.md](../spec/features/platform.md) | Select occurrences from find bar | Missing | 3 | `PLATFORM-EDIT-115` | — | — | not started (P1) |
| PLATFORM-EDIT-121 | [platform.md](../spec/features/platform.md) | Reformat File dialog (`ShowReformatFileDialog`) | Missing | 3 | `PLATFORM-EDIT-121` | — | — | not started (P1) |
| PLATFORM-EDIT-127 | [platform.md](../spec/features/platform.md) | Unwrap/Remove (`Unwrap`) | Missing | 1 | `PLATFORM-EDIT-127` | — | — | not started (P2) |
| PLATFORM-EDIT-129 | [platform.md](../spec/features/platform.md) | Emmet / zen-coding expansion (`EmmetNextEditPoint`, `EmmetPreviousEditPoint`) | Missing | 1 | `PLATFORM-EDIT-129` | — | — | not started (P2) |
| PLATFORM-EDIT-154 | [platform.md](../spec/features/platform.md) | Presentation assistant (`TogglePresentationAssistantAction`) | Missing | 1 | `PLATFORM-EDIT-154` | — | — | not started (P2) |
| PLATFORM-NAV-031 | [platform.md](../spec/features/platform.md) | Jump to Source / Show Source (`EditSource`, `ViewSource`) | Partial | 5 | `PLATFORM-NAV-031` | `editor.openFile` | `project_search_command_streams_results_into_state` | Problems rows and Find results open the file; exact caret positioning from Find pending |
| PLATFORM-NAV-033 | [platform.md](../spec/features/platform.md) | File Path popup (`ShowFilePath`) | Missing | 1 | `PLATFORM-NAV-033` | — | — | not started (P2) |
| PLATFORM-NAV-037 | [platform.md](../spec/features/platform.md) | Context-aware "Go to Changed File" (`GotoChangedFile`) | Missing | 1 | `PLATFORM-NAV-037` | — | — | not started (P2) |
| PLATFORM-PROJ-007 | [platform.md](../spec/features/platform.md) | Trusted projects / Safe mode | Missing | 5 | `PLATFORM-PROJ-007` | — | — | not started (P0) |
| PLATFORM-PROJ-013 | [platform.md](../spec/features/platform.md) | Preview tab | Missing | 3 | `PLATFORM-PROJ-013` | — | — | not started (P1) |
| PLATFORM-PROJ-015 | [platform.md](../spec/features/platform.md) | Excluded folders & ignored files | Missing | 5 | `PLATFORM-PROJ-015` | — | — | not started (P0) |
| PLATFORM-PROJ-017 | [platform.md](../spec/features/platform.md) | Workspace model (multi-root, attached projects) | Missing | 1 | `PLATFORM-PROJ-017` | — | — | not started (P2) |
| PLATFORM-PROJ-021 | [platform.md](../spec/features/platform.md) | Open file in associated application / system (`ShowLog` sibling actions; "Open In") | Missing | 3 | `PLATFORM-PROJ-021` | — | — | not started (P1) |
| PLATFORM-PROJ-022 | [platform.md](../spec/features/platform.md) | File Watchers (plugin) | Missing | 1 | `PLATFORM-PROJ-022` | — | — | not started (P2) |
| PLATFORM-PROJ-026 | [platform.md](../spec/features/platform.md) | Power Save Mode (`TogglePowerSave`) | Missing | 3 | `PLATFORM-PROJ-026` | — | — | not started (P1) |
| PLATFORM-PROJ-029 | [platform.md](../spec/features/platform.md) | Copyright profiles (plugin) | Missing | 1 | `PLATFORM-PROJ-029` | — | — | not started (P2) |
| PLATFORM-VCS-022 | [platform.md](../spec/features/platform.md) | Combined (multi-file) diff (`Vcs.Diff.ShowCombinedDiff`, `Vcs.CombinedDiff.*`) | Missing | 1 | `PLATFORM-VCS-022` | — | — | not started (P2) |
| PLATFORM-VCS-027 | [platform.md](../spec/features/platform.md) | External diff/merge tools | Missing | 1 | `PLATFORM-VCS-027` | — | — | not started (P2) |
| PLATFORM-VCS-038 | [platform.md](../spec/features/platform.md) | Clean up branches (`Git.Cleanup.Branches`, `Git.FindMergedLocalBranches`) | Missing | 1 | `PLATFORM-VCS-038` | — | — | not started (P2) |
| PLATFORM-VCS-049 | [platform.md](../spec/features/platform.md) | VCS console (`Vcs.ShowConsoleTab`) | Missing | 3 | `PLATFORM-VCS-049` | — | — | not started (P1) |
| PLATFORM-VCS-051 | [platform.md](../spec/features/platform.md) | Incoming/outgoing indicators & auto-fetch (inferred) | Missing | 3 | `PLATFORM-VCS-051` | — | — | not started (P1) |
| PLATFORM-VCS-053 | [platform.md](../spec/features/platform.md) | Copy commit hash / path from repository root (`Vcs.CopyRevisionNumberAction`, `CopyPathFromRepositoryRootProvider`) | Missing | 1 | `PLATFORM-VCS-053` | — | — | not started (P2) |
| PLATFORM-VCS-064 | [platform.md](../spec/features/platform.md) | Other hosts (Bitbucket, Gitea, Space) | Missing | 1 | `PLATFORM-VCS-064` | — | — | not started (P2) |
| PLATFORM-RUN-013 | [platform.md](../spec/features/platform.md) | Run Dashboard (`RunDashboard.*`) | Missing | 1 | `PLATFORM-RUN-013` | — | — | not started (P2) |
| PLATFORM-RUN-017 | [platform.md](../spec/features/platform.md) | Build output tool window (`CompileDirty`, `Compile`) | Partial | 3 | `PLATFORM-RUN-017` | `build.project`, `run.cancel` | `streams_real_process_output_and_exit_code`, `stderr_lines_are_only_red_for_errors` | Build tool window streams the detected build command; build tree and problem navigation pending |
| PLATFORM-TERM-002 | [platform.md](../spec/features/platform.md) | Terminal engine selection | Missing | 3 | `PLATFORM-TERM-002` | — | — | not started (P1) |
| PLATFORM-TERM-003 | [platform.md](../spec/features/platform.md) | Sessions & tabs (`Terminal.NewTab`, `Terminal.CloseTab`, `Terminal.MoveToolWindowTabLeft/Right`, `Terminal.CloseSession`) | Missing | 5 | `PLATFORM-TERM-003` | — | — | not started (P0) |
| PLATFORM-TERM-004 | [platform.md](../spec/features/platform.md) | Shell integration | Missing | 3 | `PLATFORM-TERM-004` | — | — | not started (P1) |
| PLATFORM-TERM-005 | [platform.md](../spec/features/platform.md) | Command blocks & navigation (`Terminal.SelectBlockAbove/Below`, `Terminal.SelectLastBlock`, `Terminal.CopyBlock`, `Terminal.ExpandBlockSelectionAbove/Below`) | Missing | 1 | `PLATFORM-TERM-005` | — | — | not started (P2) |
| PLATFORM-TERM-006 | [platform.md](../spec/features/platform.md) | Command completion (`Terminal.CommandCompletion*`) | Missing | 3 | `PLATFORM-TERM-006` | — | — | not started (P1) |
| PLATFORM-TERM-007 | [platform.md](../spec/features/platform.md) | Command history search (`Terminal.SearchInCommandHistory`) | Missing | 3 | `PLATFORM-TERM-007` | — | — | not started (P1) |
| PLATFORM-TERM-008 | [platform.md](../spec/features/platform.md) | Smart command execution (`Terminal.SmartCommandExecution.Run/Debug`) | Missing | 1 | `PLATFORM-TERM-008` | — | — | not started (P2) |
| PLATFORM-TERM-009 | [platform.md](../spec/features/platform.md) | Hyperlinks and file navigation | Missing | 3 | `PLATFORM-TERM-009` | — | — | not started (P1) |
| PLATFORM-TERM-010 | [platform.md](../spec/features/platform.md) | Copy/paste & selection (`Terminal.CopySelectedText`, `Terminal.Paste`, `Terminal.CreateRectangularSelectionOnMouseDrag`) | Partial | 5 | `PLATFORM-TERM-010` | `terminal.copy`, `terminal.paste` | `forward_and_backward_cell_selection_copy_the_same_text`, `terminal_commands_validate_dimensions_and_selection_boundaries` | mouse selection, copy and paste; rectangular selection pending |
| PLATFORM-TERM-011 | [platform.md](../spec/features/platform.md) | Scrolling (`Terminal.LineUp/Down`, `Terminal.PageUp/Down`) | Partial | 5 | `PLATFORM-TERM-011` | `terminal.scroll` | `terminal_commands_validate_dimensions_and_selection_boundaries` | bounded scrollback with wheel and command scrolling; keymap bindings for line/page scrolling pending |
| PLATFORM-TERM-014 | [platform.md](../spec/features/platform.md) | Remote / WSL / container shells | Missing | 1 | `PLATFORM-TERM-014` | — | — | not started (P2) |
| PLATFORM-TERM-015 | [platform.md](../spec/features/platform.md) | Find in terminal | Missing | 3 | `PLATFORM-TERM-015` | — | — | not started (P1) |
| PLATFORM-TOOL-014 | [platform.md](../spec/features/platform.md) | Documentation tool window | Missing | 1 | `PLATFORM-TOOL-014` | — | — | not started (P2) |
| PLATFORM-TOOL-019 | [platform.md](../spec/features/platform.md) | Quick lists | Missing | 1 | `PLATFORM-TOOL-019` | — | — | not started (P2) |
| PLATFORM-TOOL-022 | [platform.md](../spec/features/platform.md) | Commander-style two-pane view (plugin `commander`) | Missing | 1 | `PLATFORM-TOOL-022` | — | — | not started (P2) |
| PLATFORM-SET-011 | [platform.md](../spec/features/platform.md) | Registry (`ShowRegistry`) | Missing | 1 | `PLATFORM-SET-011` | — | — | not started (P2) |
| PLATFORM-SET-014 | [platform.md](../spec/features/platform.md) | IDE Scripting Console (`IdeScriptingConsole`) | Missing | 1 | `PLATFORM-SET-014` | — | — | not started (P2) |
| PLATFORM-SET-017 | [platform.md](../spec/features/platform.md) | Configuration directories layout | Missing | 3 | `PLATFORM-SET-017` | — | — | not started (P1) |
| PLATFORM-SET-019 | [platform.md](../spec/features/platform.md) | Restart IDE (`RestartIde`) | Partial | 3 | `PLATFORM-SET-019` | `app.prepareRestart`, `app.installUpdate` | `restart_preparation_saves_dirty_files_and_propagates_save_errors` | restart after installing an update; standalone Restart IDE action pending |
| PLATFORM-SET-022 | [platform.md](../spec/features/platform.md) | About & license | Partial | 1 | `PLATFORM-SET-022` | `app.about` | `version_comes_from_the_workspace` | About shows the version; license and build details pending |
| PLATFORM-AI-002 | [platform.md](../spec/features/platform.md) | Junie / coding agents & ACP | Missing | 1 | `PLATFORM-AI-002` | — | — | not started (P2) |
| PLATFORM-AI-003 | [platform.md](../spec/features/platform.md) | Native GitHub Copilot integration (2026.2) | Missing | 1 | `PLATFORM-AI-003` | — | — | not started (P2) |
| PLATFORM-AI-004 | [platform.md](../spec/features/platform.md) | Inline/cloud completion providers | Missing | 1 | `PLATFORM-AI-004` | — | — | not started (P2) |
| PLATFORM-AI-005 | [platform.md](../spec/features/platform.md) | Next edit suggestions (2026.1) | Missing | 1 | `PLATFORM-AI-005` | — | — | not started (P2) |
| PLATFORM-AI-007 | [platform.md](../spec/features/platform.md) | MCP server (plugin `mcp-server`) | Missing | 1 | `PLATFORM-AI-007` | — | — | not started (P2) |
| PLATFORM-REMOTE-004 | [platform.md](../spec/features/platform.md) | WSL support | Missing | 1 | `PLATFORM-REMOTE-004` | — | — | not started (P2) |
| PLATFORM-REMOTE-006 | [platform.md](../spec/features/platform.md) | SSH configurations & remote hosts | Missing | 1 | `PLATFORM-REMOTE-006` | — | — | not started (P2) |
| PLATFORM-A11Y-003 | [platform.md](../spec/features/platform.md) | Colour-deficiency adjustments | Missing | 1 | `PLATFORM-A11Y-003` | — | — | not started (P2) |
| PLATFORM-A11Y-006 | [platform.md](../spec/features/platform.md) | Contrast-scrollbars, animations off | Missing | 1 | `PLATFORM-A11Y-006` | — | — | not started (P2) |
| PLATFORM-A11Y-007 | [platform.md](../spec/features/platform.md) | Audible / read-aloud notifications | Missing | 1 | `PLATFORM-A11Y-007` | — | — | not started (P2) |
| PLATFORM-PERF-001 | [platform.md](../spec/features/platform.md) | Startup behaviour | Missing | 5 | `PLATFORM-PERF-001` | — | — | not started (P0) |
| PLATFORM-PERF-004 | [platform.md](../spec/features/platform.md) | Memory indicator (`ShowMemoryView` / status widget) | Missing | 3 | `PLATFORM-PERF-004` | — | — | not started (P1) |
| PLATFORM-PERF-005 | [platform.md](../spec/features/platform.md) | Low-memory notifications | Missing | 3 | `PLATFORM-PERF-005` | — | — | not started (P1) |
| PLATFORM-PERF-006 | [platform.md](../spec/features/platform.md) | Freeze detection & reporting | Missing | 3 | `PLATFORM-PERF-006` | — | — | not started (P1) |
| PLATFORM-PERF-007 | [platform.md](../spec/features/platform.md) | Plugin performance attribution | Missing | 1 | `PLATFORM-PERF-007` | — | — | not started (P2) |
| PLATFORM-PERF-008 | [platform.md](../spec/features/platform.md) | Cancellable read actions / responsiveness rules | Missing | 5 | `PLATFORM-PERF-008` | — | — | not started (P0) |
| PLATFORM-PERF-009 | [platform.md](../spec/features/platform.md) | Power Save Mode | Missing | 3 | `PLATFORM-PERF-009` | — | — | not started (P1) |
| PLATFORM-PERF-010 | [platform.md](../spec/features/platform.md) | Antivirus/Defender exclusion prompt (Windows) | Missing | 1 | `PLATFORM-PERF-010` | — | — | not started (P2) |
| PLATFORM-PERF-012 | [platform.md](../spec/features/platform.md) | Large-repository handling | Missing | 3 | `PLATFORM-PERF-012` | — | — | not started (P1) |
| PLATFORM-UI-006 | [platform.md](../spec/features/platform.md) | Multi-editor file tabs (`NextEditorTab`, `PreviousEditorTab`) | Missing | 1 | `PLATFORM-UI-006` | — | — | not started (P2) |
| PLATFORM-UI-007 | [platform.md](../spec/features/platform.md) | Popups & choosers conventions | Partial | 5 | `PLATFORM-UI-007` | — | `double_shift_frames_open_search_everywhere` | Escape closes popups and dialogs; type-to-filter in Search Everywhere and Recent Files; pin/resize and remembered size pending |
| PLATFORM-UI-009 | [platform.md](../spec/features/platform.md) | Native file choosers / IDE file chooser | Missing | 5 | `PLATFORM-UI-009` | — | — | not started (P0) |
| PLATFORM-UI-012 | [platform.md](../spec/features/platform.md) | Window title & frame | Partial | 3 | `PLATFORM-UI-012` | `toolwindow.hideAll` | manual launch, Xvfb screenshots | merged custom title bar with window controls on Windows/Linux, native frame on macOS (see PLAT-001); project name in title pending |
| PLATFORM-UI-013 | [platform.md](../spec/features/platform.md) | UI Inspector (internal) (`UiInspector`) | Missing | 1 | `PLATFORM-UI-013` | — | — | not started (P2) |
| SPEC-TW-ANT | [toolwindows.json](../spec/data/toolwindows.json) | Ant tool window (default: right stripe) | Missing | 1 | `toolwindow:Ant` | — | — | not started |
| SPEC-TW-BACKUP-AND-SYNC-HISTORY | [toolwindows.json](../spec/data/toolwindows.json) | Backup and Sync History tool window (default: left stripe) | Missing | 1 | `toolwindow:Backup and Sync History` | — | — | not started |
| SPEC-TW-COMMANDER | [toolwindows.json](../spec/data/toolwindows.json) | Commander tool window (default: right stripe) | Missing | 1 | `toolwindow:Commander` | — | — | not started |
| SPEC-TW-UI-THEME-COLOR-PICKER | [toolwindows.json](../spec/data/toolwindows.json) | UI Theme Color Picker tool window (default: right stripe) | Missing | 1 | `toolwindow:UI Theme Color Picker` | — | — | not started |
