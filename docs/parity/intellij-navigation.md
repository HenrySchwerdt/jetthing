# IntelliJ IDEA parity: navigation and search

Search Everywhere, Go To actions, symbol navigation, hierarchies, bookmarks, and text and
structural search/replace.

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
| IJ-NAV-001 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html) | Search Everywhere (double Shift) with All / Classes / Files / Symbols / Actions / Text tabs | Partial | 5 | `navigate.searchEverywhere` | `navigation::tests` (ranking, providers), scenes 05/21-23 | Double Shift opens All / Classes / Files / Symbols / Actions with camel-hump ranking and match highlighting, Tab / Shift+Tab switch tabs; the Text tab is missing (`evidence/search-everywhere.md`) |
| IJ-NAV-002 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html) | Go to Class (Ctrl+N) | Partial | 5 | `navigate.gotoClass` | `navigation::tests`, scene 21 | classes, interfaces, enums and types from tree-sitter tags outlines of open and indexed Rust and Java files plus `workspace/symbol` of a running language server; other grammars have no tags query yet (`evidence/search-everywhere.md`) |
| IJ-NAV-003 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html) | Go to File (Ctrl+Shift+N) with fuzzy matching and :line suffix | Partial | 5 | `navigate.gotoFile` | `navigation::tests`, scene 22 | camel-hump file-name matching, recent files for an empty query, `:line` suffix opens at that line; project files only (`evidence/search-everywhere.md`) |
| IJ-NAV-004 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html) | Go to Symbol (Ctrl+Alt+Shift+N) | Partial | 3 | `navigate.gotoSymbol` | `navigation::tests` (outlines), `workspace_symbol::tests`, scene 23 | functions, methods, fields and constants from tree-sitter outlines and `workspace/symbol`; file/line shown, no container for tree-sitter symbols (`evidence/search-everywhere.md`) |
| IJ-NAV-005 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html#find_action) | Find Action searches settings and options | Missing | 2 | — | — | palette lists commands only; see IJ-MENU-111 |
| IJ-NAV-006 | [searching-everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html) | Search Everywhere: include non-project items, filters, preview | Missing | 1 | — | — | not started |
| IJ-NAV-007 | [running-anything](https://www.jetbrains.com/help/idea/running-anything.html) | Run Anything: run commands and configurations | Missing | 2 | — | — | not started |
| IJ-NAV-008 | [file-navigation](https://www.jetbrains.com/help/idea/file-navigation.html) | Go to Line:Column (Ctrl+G) | Missing | 5 | — | — | not started |
| IJ-NAV-009 | [recent-files-and-changes](https://www.jetbrains.com/help/idea/recent-files-and-changes.html) | Recent Files popup (Ctrl+E) | Partial | 5 | `navigate.recentFiles` | `navigation::tests` (ordering), scene 24 | newest first, selection starts on the previous file, camel-hump filter; no recently edited / changed-only toggle (`evidence/search-everywhere.md`) |
| IJ-NAV-010 | [recent-files-and-changes](https://www.jetbrains.com/help/idea/recent-files-and-changes.html) | Recent Locations popup (Ctrl+Shift+E) | Partial | 2 | `navigate.recentLocations` | `navigation::tests` (merging, cap), scene 25 | caret positions with a line preview, nearby moves merge; not persisted across restarts, no editor preview pane (`evidence/search-everywhere.md`) |
| IJ-NAV-011 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Switcher (Ctrl+Tab) | Partial | 3 | `navigate.switcher`, `navigate.switcherCommit`, `navigate.switcherCancel` | `navigation::tests` (cycling, ordering), scene 26 | hold Ctrl and press Tab / Shift+Tab to cycle open files (recent first) and visible tool windows, release to switch, Esc cancels; no recently edited/changed-files list (`evidence/search-everywhere.md`) |
| IJ-NAV-012 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Back / Forward navigation history (Ctrl+Alt+Left/Right) | Missing | 5 | — | — | not started |
| IJ-NAV-013 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Last Edit Location (Ctrl+Shift+Backspace) | Missing | 2 | — | — | not started |
| IJ-NAV-014 | [file-navigation](https://www.jetbrains.com/help/idea/file-navigation.html) | Select In (Alt+F1): project view, Finder/Explorer, structure | Missing | 2 | — | — | not started |
| IJ-NAV-015 | [file-navigation](https://www.jetbrains.com/help/idea/file-navigation.html) | Navigate to file from path in console/terminal output | Missing | 2 | — | — | not started |
| IJ-NAV-020 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Declaration or Usages (Ctrl+B / Ctrl+Click) | Missing | 5 | — | — | LSP definition request not yet wired |
| IJ-NAV-021 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Type Declaration (Ctrl+Shift+B) | Missing | 2 | — | — | not started |
| IJ-NAV-022 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Implementation(s) (Ctrl+Alt+B) | Missing | 3 | — | — | not started |
| IJ-NAV-023 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Super Method (Ctrl+U) | Missing | 2 | — | — | not started |
| IJ-NAV-024 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Test / Create Test (Ctrl+Shift+T) | Missing | 2 | — | — | not started |
| IJ-NAV-025 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Next / Previous Method (Alt+Down/Up) | Missing | 1 | — | — | not started |
| IJ-NAV-026 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Go to Related Symbol (Ctrl+Alt+Home) | Missing | 1 | — | — | not started |
| IJ-NAV-027 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Navigate to library sources and decompiled classes | Missing | 2 | — | — | not started |
| IJ-NAV-030 | [find-highlight-usages](https://www.jetbrains.com/help/idea/find-highlight-usages.html) | Find Usages (Alt+F7) into Find tool window | Missing | 5 | — | — | not started |
| IJ-NAV-031 | [find-highlight-usages](https://www.jetbrains.com/help/idea/find-highlight-usages.html) | Show Usages popup (Ctrl+Alt+F7) | Missing | 3 | — | — | not started |
| IJ-NAV-032 | [find-highlight-usages](https://www.jetbrains.com/help/idea/find-highlight-usages.html) | Highlight Usages in File (Ctrl+Shift+F7) | Missing | 2 | — | — | not started |
| IJ-NAV-033 | [find-highlight-usages](https://www.jetbrains.com/help/idea/find-highlight-usages.html) | Usage grouping by module, package, read/write | Missing | 1 | — | — | not started |
| IJ-NAV-040 | [viewing-structure-of-a-source-file](https://www.jetbrains.com/help/idea/viewing-structure-of-a-source-file.html) | File Structure popup (Ctrl+F12) with speed search | Partial | 3 | File Structure (Ctrl+F12) | structure tests, Xvfb screenshot vs scene 22 | preselects the caret declaration, filters as you type, Enter/click navigates; no Inherited members/Lambdas toggles or visibility badges; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-NAV-041 | [viewing-structure-and-hierarchy-of-the-source-code](https://www.jetbrains.com/help/idea/viewing-structure-and-hierarchy-of-the-source-code.html) | Type Hierarchy (Ctrl+H) | Missing | 2 | — | — | not started |
| IJ-NAV-042 | [viewing-structure-and-hierarchy-of-the-source-code](https://www.jetbrains.com/help/idea/viewing-structure-and-hierarchy-of-the-source-code.html) | Method Hierarchy (Ctrl+Shift+H) | Missing | 1 | — | — | not started |
| IJ-NAV-043 | [viewing-structure-and-hierarchy-of-the-source-code](https://www.jetbrains.com/help/idea/viewing-structure-and-hierarchy-of-the-source-code.html) | Call Hierarchy (Ctrl+Alt+H) | Missing | 2 | — | — | not started |
| IJ-NAV-044 | [class-diagram](https://www.jetbrains.com/help/idea/class-diagram.html) | [Ultimate] UML class diagrams | Missing | 1 | — | — | not started |
| IJ-NAV-050 | [bookmarks](https://www.jetbrains.com/help/idea/bookmarks.html) | Toggle anonymous bookmark (F11) | Missing | 2 | — | — | not started |
| IJ-NAV-051 | [bookmarks](https://www.jetbrains.com/help/idea/bookmarks.html) | Mnemonic bookmarks (Ctrl+F11) and Go to bookmark with digit | Missing | 1 | — | — | not started |
| IJ-NAV-052 | [bookmarks](https://www.jetbrains.com/help/idea/bookmarks.html) | Bookmark lists and descriptions | Missing | 1 | — | — | not started |
| IJ-NAV-060 | [finding-and-replacing-text-in-file](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-file.html) | Find in current file (Ctrl+F) with highlights | Missing | 5 | — | — | not started |
| IJ-NAV-061 | [finding-and-replacing-text-in-file](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-file.html) | Find Next / Previous (F3 / Shift+F3) | Missing | 3 | — | — | not started |
| IJ-NAV-062 | [finding-and-replacing-text-in-file](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-file.html) | Replace in file (Ctrl+R) | Missing | 5 | — | — | not started |
| IJ-NAV-063 | [finding-and-replacing-text-in-file](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-file.html) | Match case, whole words, regex toggles | Missing | 3 | — | — | not started |
| IJ-NAV-064 | [finding-and-replacing-text-in-project](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-project.html) | Find in Files (Ctrl+Shift+F) | Partial | 5 | `search.project`, `search.cancel`, `editor.openFile` | async search, per-line highlight, Unicode case-folding, options and open-at-match tests; scene 10 | streamed search with Match Case / Words / Regex toggles; one row per line with highlighted occurrences; results open with the match selected (`evidence/find-in-files-options.md`, `evidence/find-reveal-symlinks.md`); preview pending |
| IJ-NAV-065 | [finding-and-replacing-text-in-project](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-project.html) | Find in Files scopes: project, module, directory, custom scope, file mask | Partial | 3 | `search.project` | file mask include/exclude tests, core options test | file mask (`*.java, !*Test.java`, path globs) with live re-run; project scope only, no module/directory/custom scopes (`evidence/find-in-files-options.md`) |
| IJ-NAV-066 | [finding-and-replacing-text-in-project](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-project.html) | Find in Files preview pane and open in Find tool window | Missing | 2 | — | — | not started |
| IJ-NAV-067 | [finding-and-replacing-text-in-project](https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-project.html) | Replace in Files (Ctrl+Shift+R) | Missing | 3 | — | — | not started |
| IJ-NAV-068 | [regular-expressions](https://www.jetbrains.com/help/idea/regular-expressions.html) | Regex search with capture-group replacement | Missing | 2 | — | — | not started |
| IJ-NAV-069 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Next Occurrence of the Word at Caret (Ctrl+F3) | Missing | 1 | — | — | not started |
| IJ-NAV-070 | [structural-search-and-replace](https://www.jetbrains.com/help/idea/structural-search-and-replace.html) | Structural Search and Replace | Missing | 1 | — | — | not started |
| IJ-NAV-071 | [search-templates](https://www.jetbrains.com/help/idea/search-templates.html) | Structural search templates | Missing | 1 | — | — | not started |
| IJ-NAV-072 | [full-text-search-for-databases](https://www.jetbrains.com/help/idea/full-text-search-for-databases.html) | [Ultimate] Full-text search in databases | Missing | 1 | — | — | not started |
| IJ-NAV-080 | [shared-indexes](https://www.jetbrains.com/help/idea/shared-indexes.html) | Background indexing with progress and dumb mode | Missing | 5 | — | — | see PLAT-009 |
| IJ-NAV-081 | [shared-indexes](https://www.jetbrains.com/help/idea/shared-indexes.html) | Shared / prebuilt indexes for JDK and libraries | Missing | 1 | — | — | not started |
