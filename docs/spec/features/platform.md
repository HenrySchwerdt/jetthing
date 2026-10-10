# Platform feature inventory (language-agnostic IntelliJ Platform)

Status: research inventory, not an implementation claim. Nothing here asserts parity; parity
evidence belongs in `docs/parity/` (see `AGENTS.md`).

Target product: IntelliJ IDEA 2026.2 (platform build line 262). Scope: features that live in the
shared IntelliJ Platform and therefore appear in IDEA, PyCharm, WebStorm, GoLand, etc. Java/JVM-only
features are excluded unless they are the platform's generic framework (for example the
"Type Hierarchy" framework is listed; Java's class hierarchy implementation is not).

## Conventions

- **ID**: `PLATFORM-<AREA>-NNN`. IDs are stable; never renumber. New items are appended with the
  next free number. Retired items stay in the table marked "(retired)".
- **Action id**: where a public action id exists in the Apache-2.0 IntelliJ Community action
  descriptors (tag `idea/262.10968.63`), it is given in backticks. It is a cross-reference for
  mapping onto `jetthing-command` command ids, not something to copy.
- **Shortcuts**: "Win/Linux" is the default (`$default`) keymap; "macOS" is the
  "macOS" keymap (descriptor file `Mac OS X 10.5+`). Notation: `Ctrl`, `Alt`, `Shift`, `Cmd`,
  `Opt`; a comma separates the two strokes of a chord (e.g. `Alt+Shift+6, F`). `Double Shift`
  means tapping the key twice. `—` means no default shortcut. Shortcuts were extracted
  mechanically from the keymap descriptors and cross-checked against the public keymap reference;
  where the macOS keymap does not override an action, the platform derives the macOS binding by
  swapping Ctrl to Cmd (this rule is (inferred) from comparing the descriptors with the published
  macOS reference).
- **Entry point**: main menu path (`View | Tool Windows | ...`), tool window, popup, gutter,
  status bar widget, or Find Action only.
- **Priority**: P0 = required for a credible daily-driver IDE; P1 = expected by regular
  IntelliJ users, ship soon after P0; P2 = niche, commercial-only, optional, or AI.
- **(inferred)** marks behaviour reconstructed from observation or naming rather than an explicit
  public statement.
- **(plugin)** marks platform features that ship as bundled plugins rather than core modules
  (still language-agnostic). **(Ultimate)** marks commercial-only features.

## Area index

| Area code | Section |
|---|---|
| EDIT | 1. Editor core |
| NAV | 2. Navigation and search |
| PROJ | 3. Project model and files |
| VCS | 4. Version control (Git) |
| RUN | 5. Run / Debug infrastructure |
| TERM | 6. Terminal |
| TOOL | 7. Tool windows and workbench services |
| SET | 8. Settings and configuration |
| AI | 9. AI features (optional) |
| REMOTE | 10. Collaboration and remote |
| A11Y | 11. Accessibility and i18n |
| PERF | 12. Performance and background work |
| UI | 13. Window, tab and layout management |

---

## 1. Editor core (EDIT)

### 1.1 Caret, selection, multi-caret

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-001 | Caret movement primitives (`EditorLeft/Right/Up/Down`, `EditorNextWord`, `EditorPreviousWord`, `EditorLineStart/End`, `EditorTextStart/End`, `EditorPageUp/Down`, `EditorMoveToPageTop/Bottom`) | Character, word, line, page and document movement. Word boundaries honour the "camel humps" setting (sub-word stops inside `camelCase`) when enabled. Home toggles between first non-whitespace char and column 0 ("smart home"). Each primitive has a `...WithSelection` twin that extends the selection. | Arrows; Ctrl+Left/Right; Home/End; Ctrl+Home/End; PgUp/PgDn; Ctrl+PgUp/PgDn | Arrows; Opt+Left/Right; Cmd+Left/Right; Cmd+Home/End; PgUp/PgDn; Cmd+PgUp/PgDn | Editor | P0 |
| PLATFORM-EDIT-002 | Selection model | Each caret owns at most one contiguous selection with an anchor and a lead; selection survives scrolling; Shift+click extends; double-click selects word, triple-click selects line; drag-and-drop of selected text moves it (Ctrl/Opt to copy). | Shift+arrows, mouse | Shift+arrows, mouse | Editor | P0 |
| PLATFORM-EDIT-003 | Select All (`$SelectAll`) | Selects whole document (in multi-caret mode collapses to one caret). | Ctrl+A | Cmd+A | Edit menu | P0 |
| PLATFORM-EDIT-004 | Extend Selection (`EditorSelectWord`) | Grows the selection to the next enclosing syntactic unit (word, expression, statement, block, ...). Uses the language's syntax tree; falls back to word/line/brace heuristics for plain text. | Ctrl+W | Opt+Up | Edit \| Extend Selection | P0 |
| PLATFORM-EDIT-005 | Shrink Selection (`EditorUnSelectWord`) | Reverses the last extend step(s). | Ctrl+Shift+W | Opt+Down | Edit \| Shrink Selection | P0 |
| PLATFORM-EDIT-006 | Add/remove caret by click (`EditorAddOrRemoveCaret`) | Clicking with the modifier adds a new caret at the click point or removes an existing one there. | Alt+Shift+Click | Opt+Shift+Click | Editor mouse | P0 |
| PLATFORM-EDIT-007 | Clone caret above/below (`EditorCloneCaretAbove`, `EditorCloneCaretBelow`) | Adds a caret on the adjacent line at the same visual column. Default binding is a hold-gesture: press the modifier twice and keep it held, then press Up/Down (gesture is hard-wired, not in the keymap descriptor (inferred)). | Ctrl, Ctrl (hold) + Up/Down | Opt, Opt (hold) + Up/Down | Find Action; gesture | P0 |
| PLATFORM-EDIT-008 | Add Selection for Next Occurrence (`SelectNextOccurrence`) | First press selects the word at caret; each next press adds a caret+selection at the next match of the selected text (case-sensitive, wraps). | Alt+J | Ctrl+G | Edit \| Find \| Add Selection for Next Occurrence | P0 |
| PLATFORM-EDIT-009 | Unselect Occurrence (`UnselectPreviousOccurrence`) | Removes the most recently added occurrence caret. | Alt+Shift+J | Ctrl+Shift+G | Edit \| Find | P0 |
| PLATFORM-EDIT-010 | Select All Occurrences (`SelectAllOccurrences`) | Places a caret+selection on every match of the current selection/word in the file. | Ctrl+Alt+Shift+J | Ctrl+Cmd+G | Edit \| Find | P0 |
| PLATFORM-EDIT-011 | Add Carets to Ends of Selected Lines (`EditorAddCaretPerSelectedLine`) | Converts a multi-line selection into one caret at the end of each line. | Alt+Shift+G | Opt+Shift+G | Edit \| Find / Find Action | P1 |
| PLATFORM-EDIT-012 | Column (block) selection mode (`EditorToggleColumnMode`) | Toggle where Shift+arrows and mouse drags create rectangular selections (one caret per line). Status bar shows the mode. | Alt+Shift+Insert | Cmd+Shift+8 | Edit \| Column Selection Mode | P0 |
| PLATFORM-EDIT-013 | Rectangular selection by mouse (`EditorCreateRectangularSelectionOnMouseDrag`, `EditorAddRectangularSelectionOnMouseDrag`) | Alt-drag or middle-button drag creates a rectangular selection without toggling the mode; adding Ctrl+Shift adds a further block. | Alt+Drag / Middle-drag; Ctrl+Alt+Shift+Drag | Opt+Drag; Cmd+Opt+Shift+Drag (inferred) | Editor mouse | P1 |
| PLATFORM-EDIT-014 | Multi-caret semantics | All typing, deletion, paste, completion, live templates and most editor actions apply per caret in one undoable command; carets that collide merge. Paste of N clipboard lines onto N carets distributes one line per caret. Esc collapses to the primary caret. | — | — | Editor | P0 |
| PLATFORM-EDIT-015 | Virtual space | Option "Allow placement of caret after end of line" lets the caret sit in whitespace beyond line end; typing there pads with spaces. Also "allow caret inside tab characters". | — | — | Settings \| Editor \| General \| Virtual Space | P2 |
| PLATFORM-EDIT-016 | Caret behaviour options | Caret blinking & width, "use block caret", camel-humps word navigation, "caret position on line/after end", restoring caret on reopen. | — | — | Settings \| Editor \| General \| Caret Movement / Appearance | P1 |
| PLATFORM-EDIT-017 | Insert/Overwrite toggle (`EditorToggleInsertState`) | Switches typing between insert and overwrite; status bar indicator. | Insert | Insert (Fn+Enter on laptops (inferred)) | Editor | P1 |
| PLATFORM-EDIT-018 | Move caret to matching brace (`EditorMatchBrace`) and code block start/end (`EditorCodeBlockStart/End`) | Jumps between paired delimiters / enclosing block bounds, with selecting variants. | Ctrl+Shift+M; Ctrl+[ / Ctrl+] | Ctrl+M; Cmd+Opt+[ / Cmd+Opt+] | Find Action | P1 |
| PLATFORM-EDIT-019 | Scroll without moving caret (`EditorScrollUp/Down`, `EditorScrollToCenter`) | Scrolls the viewport by a line or centres the caret line. | Ctrl+Up/Down; Ctrl+M | — ; Ctrl+L | Find Action | P1 |
| PLATFORM-EDIT-020 | Jump outside bracket/quote (`BraceOrQuoteOut`) | When caret is just before an auto-inserted closing bracket or quote, Tab steps over it instead of indenting. | Tab | Tab | Settings \| Editor \| General \| Smart Keys | P1 |

### 1.2 Clipboard, line and text manipulation

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-030 | Cut/Copy/Paste (`$Cut`, `$Copy`, `$Paste`) | Standard clipboard; with no selection, Cut/Copy act on the whole current line (configurable). Paste reformats/reindents per "Smart Keys \| Reformat on paste" setting and may adjust imports (language plugins). | Ctrl+X / Ctrl+C / Ctrl+V (also Shift+Del, Ctrl+Ins, Shift+Ins) | Cmd+X / Cmd+C / Cmd+V | Edit menu | P0 |
| PLATFORM-EDIT-031 | Paste from History (`PasteMultiple`) | Popup listing recent clipboard entries (default 5 kept, configurable) with preview; choosing one pastes it; multiple entries can be merged. Entries come from copy/cut inside the IDE and from the system clipboard. | Ctrl+Shift+V | Cmd+Shift+V | Edit \| Paste Special \| Paste from History | P0 |
| PLATFORM-EDIT-032 | Paste as Plain Text (`EditorPasteSimple`) | Pastes without reformat/import/escaping processing. | Ctrl+Alt+Shift+V | Cmd+Opt+Shift+V | Edit \| Paste Special | P1 |
| PLATFORM-EDIT-033 | Copy Path / Copy Reference (`CopyPaths`, `CopyReference`) | Copies absolute path, path from content root, file name, or a language-qualified reference (e.g. `file:line` for plain files) of the element at caret. "Copy Path/Reference..." popup offers all variants. | Ctrl+Shift+C; Ctrl+Alt+Shift+C | Cmd+Shift+C; Cmd+Opt+Shift+C | Edit \| Copy Path/Reference | P1 |
| PLATFORM-EDIT-034 | Duplicate Line or Selection (`EditorDuplicate`) | Duplicates the selection, or the current line when nothing is selected; caret moves into the copy. Variant "Duplicate Entire Lines" duplicates whole lines spanned by selection. | Ctrl+D | Cmd+D | Edit \| Duplicate Line or Selection | P0 |
| PLATFORM-EDIT-035 | Delete Line (`EditorDeleteLine`) | Deletes all lines spanned by carets/selections. | Ctrl+Y | Cmd+Backspace | Edit / Find Action | P0 |
| PLATFORM-EDIT-036 | Delete to word start/end (`EditorDeleteToWordStart/End`) | Word-wise delete respecting camel humps setting. | Ctrl+Backspace / Ctrl+Delete | Opt+Backspace / Opt+Delete | Editor | P0 |
| PLATFORM-EDIT-037 | Move Line Up/Down (`MoveLineUp/Down`) | Swaps the current line(s) with the neighbour line, ignoring syntax. | Alt+Shift+Up/Down | Opt+Shift+Up/Down | Code \| Move Line Up/Down | P0 |
| PLATFORM-EDIT-038 | Move Statement Up/Down (`MoveStatementUp/Down`) | Moves the enclosing statement/member/block as a unit, respecting syntax (language-provided mover; falls back to line move). | Ctrl+Shift+Up/Down | Cmd+Shift+Up/Down | Code \| Move Statement Up/Down | P1 |
| PLATFORM-EDIT-039 | Move Element Left/Right (`MoveElementLeft/Right`) | Swaps list elements such as arguments or attributes (language-provided). | Ctrl+Alt+Shift+Left/Right | Cmd+Opt+Shift+Left/Right | Code menu | P2 |
| PLATFORM-EDIT-040 | Join Lines (`EditorJoinLines`) | Joins the next line onto the current one, collapsing whitespace and applying language-specific smart joins (e.g. concatenated strings, comment continuation). | Ctrl+Shift+J | Ctrl+Shift+J | Edit \| Join Lines | P0 |
| PLATFORM-EDIT-041 | Split Line / Start New Line / Start New Line Before (`EditorSplitLine`, `EditorStartNewLine`, `EditorStartNewLineBefore`) | Break line keeping caret; open new indented line below or above regardless of caret column. | Ctrl+Enter; Shift+Enter; Ctrl+Alt+Enter | Cmd+Enter; Shift+Enter; Cmd+Opt+Enter | Editor | P0 |
| PLATFORM-EDIT-042 | Complete Current Statement (`EditorCompleteStatement`) | Adds missing closing tokens/terminators and moves caret to the natural next position (language-provided; generic fallback = new line). | Ctrl+Shift+Enter | Cmd+Shift+Enter | Code \| Code Completion | P1 |
| PLATFORM-EDIT-043 | Toggle Case (`EditorToggleCase`) | Toggles selection (or word at caret) between upper and lower case. | Ctrl+Shift+U | Cmd+Shift+U | Edit \| Toggle Case | P0 |
| PLATFORM-EDIT-044 | Sort / Reverse Lines (`EditorSortLines`, `EditorReverseLines`) | Lexicographically sorts or reverses selected lines (whole file when no selection). | — | — | Edit \| Sort Lines / Reverse Lines | P1 |
| PLATFORM-EDIT-045 | Fill Paragraph (`FillParagraph`) | Re-wraps the paragraph/comment at caret to the right margin. | — | — | Edit \| Fill Paragraph | P2 |
| PLATFORM-EDIT-046 | Indent / Unindent selection (`EditorIndentSelection`, `EditorUnindentSelection`) | Tab/Shift+Tab shift the selected lines by one indent unit. | Tab / Shift+Tab | Tab / Shift+Tab | Edit \| Indent Selection | P0 |
| PLATFORM-EDIT-047 | Auto-Indent Lines (`AutoIndentLines`) | Recomputes indentation for current/selected lines using the formatter's indent model only. | Ctrl+Alt+I | Ctrl+Opt+I | Code \| Auto-Indent Lines | P0 |
| PLATFORM-EDIT-048 | Convert Indents (`ConvertIndentsToSpaces`, `ConvertIndentsToTabs`) | Converts leading whitespace in selection/file. | — | — | Edit \| Convert Indents | P1 |
| PLATFORM-EDIT-049 | Smart keys | Auto-insert pair brackets and quotes; smart Enter indentation; smart Backspace (unindent / remove paired char); "surround selection on typing quote or brace"; smart Home; "insert paired" for markup tags. Each is a toggle. | — | — | Settings \| Editor \| General \| Smart Keys | P0 |
| PLATFORM-EDIT-050 | Cyclic Expand Word (`HippieCompletion`, `HippieBackwardCompletion`) | Completes the current prefix with words found in visible text/open files, cycling on repeat; no language knowledge required. | Alt+/ ; Alt+Shift+/ | Opt+/ ; Opt+Shift+/ | Code \| Code Completion | P1 |
| PLATFORM-EDIT-051 | Emoji & Symbols picker (`EmojiPicker.Open`) (plugin) | Popup to insert emoji/symbols at caret. | Ctrl+Alt+; | Cmd+Opt+; | Edit menu | P2 |
| PLATFORM-EDIT-052 | Trailing whitespace & final newline on save | Strip trailing spaces (none / modified lines / all), keep caret line's whitespace, ensure line feed at EOF, remove extra blank lines at EOF. | — | — | Settings \| Editor \| General \| On Save | P0 |
| PLATFORM-EDIT-053 | Actions on Save | Configurable per-project list run when a file is saved: reformat code (whole file or changed lines), optimize imports, rearrange code, run code cleanup, plus plugin-contributed actions (e.g. Prettier, ESLint (plugins)). | — | — | Settings \| Tools \| Actions on Save | P1 |

### 1.3 Comments, folding, wraps, whitespace, display

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-060 | Comment with Line Comment (`CommentByLineComment`) | Toggles the language's line-comment prefix on each selected line; caret moves to next line when no selection. Options: comment at first column, add space. Plain-text fallback: none (inferred). | Ctrl+/ | Cmd+/ | Code \| Comment with Line Comment | P0 |
| PLATFORM-EDIT-061 | Comment with Block Comment (`CommentByBlockComment`) | Wraps/unwraps selection in block comment delimiters. | Ctrl+Shift+/ | Cmd+Opt+/ | Code \| Comment with Block Comment | P0 |
| PLATFORM-EDIT-062 | Code folding: expand/collapse (`ExpandRegion`, `CollapseRegion`, `ExpandRegionRecursively`, `CollapseRegionRecursively`, `ExpandAllRegions`, `CollapseAllRegions`, `ExpandToLevel1..5`, `ExpandAllToLevel1..5`) | Fold regions come from the language (blocks, imports, comments) plus generic ones. Gutter shows fold markers; collapsed text shows a placeholder (`{...}`) with hover preview. | Ctrl+= / Ctrl+-; Ctrl+Alt+= / -; Ctrl+Shift+= / -; Ctrl+*, N; Ctrl+Shift+*, N | Cmd+= / Cmd+-; Cmd+Opt+= / -; Cmd+Shift+= / -; Cmd+*, N; Cmd+Opt+*, N | Code \| Folding | P0 |
| PLATFORM-EDIT-063 | Fold Selection / Fold Code Block (`CollapseSelection`, `CollapseBlock`) | Creates an ad-hoc fold over the selection, or folds the enclosing block. | Ctrl+. ; Ctrl+Shift+. | Cmd+. ; Cmd+Shift+. | Code \| Folding | P1 |
| PLATFORM-EDIT-064 | Custom folding regions (`GotoCustomRegion`) | Comment markers (`region`/`endregion`, `<editor-fold>`) define user folds; a popup lists them for navigation. | Ctrl+Alt+. | Cmd+Opt+. | Code \| Folding \| Custom Folding | P1 |
| PLATFORM-EDIT-065 | Fold state persistence and defaults | Fold states persist per file across sessions; "collapse by default" toggles per region kind. | — | — | Settings \| Editor \| General \| Code Folding | P1 |
| PLATFORM-EDIT-066 | Soft wraps (`EditorToggleUseSoftWraps`) | Visual wrapping at window edge or right margin, per editor toggle and global per-file-mask setting (default `*.md; *.txt; *.rst; *.adoc`); wrapped lines may keep indentation and show a wrap sign. | — | — | View \| Active Editor \| Soft-Wrap | P0 |
| PLATFORM-EDIT-067 | Hard wrap / right margin | Visual guide at right margin (default 120) plus optional extra visual guides; "wrap on typing" inserts line breaks automatically. | — | — | Settings \| Editor \| Code Style | P1 |
| PLATFORM-EDIT-068 | Show whitespaces (`EditorToggleShowWhitespaces`) | Renders spaces/tabs as glyphs (leading/inner/trailing selectable); also shows indent guides and optional "show line separators between methods". | — | — | View \| Active Editor \| Show Whitespaces | P1 |
| PLATFORM-EDIT-069 | Line numbers, gutter configuration (`EditorGutterToggleGlobalLineNumbers`) | Absolute / relative / hybrid line numbers; toggle gutter icons, breakpoints area, fold markers, VCS change markers. | — | — | Gutter context menu; Settings \| Editor \| General \| Appearance | P0 |
| PLATFORM-EDIT-070 | Sticky lines (`EditorGutterToggleGlobalStickyLines`, `EditorStickyLinesDisable`) | Pins headers of enclosing scopes (class/function/block) at the top of the viewport while scrolling; clickable to navigate; max lines configurable, per-language opt-out. | — | — | Editor context; Settings \| Editor \| General \| Sticky Lines | P1 |
| PLATFORM-EDIT-071 | Breadcrumbs (`EditorToggleShowBreadcrumbs`, `EditorBreadcrumbsShowAbove/Below`) | Strip showing the structural path to the caret (e.g. tag nesting, class > method); clicking navigates. Position top/bottom/hidden, per language. | — | — | View \| Active Editor; Settings \| Editor \| General \| Breadcrumbs | P1 |
| PLATFORM-EDIT-072 | Inlay hints framework (`ToggleInlayHintsGloballyAction`) | Inline non-text annotations (parameter names, types, code vision "N usages"/author) contributed by language plugins; a global toggle and per-provider settings with preview. | — | — | Settings \| Editor \| Inlay Hints | P1 |
| PLATFORM-EDIT-073 | Editor font & zoom (`EditorIncreaseFontSizeGlobal`, `EditorDecreaseFontSizeGlobal`) | Font family, size, line height, ligatures; Ctrl+mouse wheel zoom (opt-in) for current or all editors; reset font size. | Alt+Shift+. / Alt+Shift+, | Ctrl+Shift+. / Ctrl+Shift+, | Settings \| Editor \| Font; View \| Appearance | P0 |
| PLATFORM-EDIT-074 | Color schemes (`QuickChangeScheme`) | Bundled schemes (Light, Dark, Darcula, High Contrast, "Classic Light"...) with per-language attribute overrides, import/export (`.icls`), semantic-highlighting option. Quick Switch Scheme popup lists scheme, keymap, view modes, LaF. | Ctrl+` | Ctrl+` | Settings \| Editor \| Color Scheme | P0 |
| PLATFORM-EDIT-075 | Highlight current line / matching braces / identifier under caret | Caret-row background; matching bracket highlight; automatic highlighting of usages of the symbol under caret after a short delay. | — | — | Settings \| Editor \| General \| Appearance / Code Editing | P0 |
| PLATFORM-EDIT-076 | Highlight Usages in File (`HighlightUsagesInFile`) | Persistently highlights read/write usages of the symbol at caret; F3/Shift+F3 cycle between them; Esc clears. | Ctrl+Shift+F7 | Cmd+Shift+F7 | Edit \| Find \| Highlight Usages in File | P1 |
| PLATFORM-EDIT-077 | Editor minimap / error stripe | Right-hand marker bar showing errors, warnings, find results, VCS changes; click to jump; a traffic-light "inspection widget" at top summarising problem counts with navigation. (A full code minimap is only available via plugins (inferred).) | — | — | Editor | P0 |
| PLATFORM-EDIT-078 | Quick documentation / Context info (`QuickJavaDoc`, `EditorContextInfo`) | Generic popup framework showing docs for symbol at caret (on hover or on demand), pin-able to a tool window; Context Info shows the enclosing scope header when off-screen. Content is language-provided. | Ctrl+Q; Alt+Q | F1; Ctrl+Shift+Q | View \| Quick Documentation | P0 |
| PLATFORM-EDIT-079 | Quick Definition (`QuickImplementations`) | Popup previewing the definition source of the symbol at caret. | Ctrl+Shift+I | Opt+Space / Cmd+Y | View \| Quick Definition | P1 |
| PLATFORM-EDIT-080 | Parameter info / Type info / Error description (`ParameterInfo`, `ExpressionTypeInfo`, `ShowErrorDescription`) | Framework popups filled by language support; error description shows the full message of the highlight under caret. | Ctrl+P; Ctrl+Shift+P; Ctrl+F1 | Cmd+P; Ctrl+Shift+P; Cmd+F1 | View menu | P1 |
| PLATFORM-EDIT-081 | Rendered documentation comments (`ToggleRenderedDocPresentation`) | Shows doc comments rendered as formatted text inline in the editor; toggle back to raw. | Ctrl+Alt+Q | Ctrl+Opt+Q | Gutter icon; Find Action | P2 |
| PLATFORM-EDIT-082 | Image and binary viewers (`Images.Editor.*`) | Built-in image viewer (zoom, grid, background, open externally) and fall-back "file too large / binary" placeholder editor. | Ctrl+' (grid) | Cmd+' | Opens automatically | P2 |
| PLATFORM-EDIT-083 | Hex/byte view of binary files (inferred) | Binary files open in a read-only notice editor; a hex view is plugin territory. | — | — | — | P2 |

### 1.4 File-level settings: encodings, separators, EditorConfig, large files, read-only

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-090 | Line separators (`ConvertToUnixLineSeparators`, `ConvertToWindowsLineSeparators`, `ConvertToMacLineSeparators`) | Detects LF/CRLF/CR per file, preserves on save; status bar widget converts; default for new files is configurable per project. | — | — | Status bar; File \| File Properties \| Line Separators | P0 |
| PLATFORM-EDIT-091 | File encoding (`ChangeFileEncodingAction`) | Status bar widget shows encoding; "Reload in" vs "Convert to" another encoding; per-file/per-directory mapping; BOM handling for UTF-8 (with/without/as-is); properties-file default; warning if chars not representable. | — | — | Status bar; Settings \| Editor \| File Encodings | P0 |
| PLATFORM-EDIT-092 | Indentation detection | When enabled, infers tab vs spaces and indent size from file content and overrides code style for that file; status bar shows the detected indent and lets the user switch/disable. | — | — | Status bar; Settings \| Editor \| Code Style \| Detect and use existing file indents | P0 |
| PLATFORM-EDIT-093 | EditorConfig support (plugin, bundled) | Reads `.editorconfig` hierarchies; standard properties (indent_style/size, tab_width, end_of_line, charset, trim_trailing_whitespace, insert_final_newline, max_line_length) override IDE settings; vendor `ij_*` keys map to code style options; export code style to `.editorconfig`; status bar indicates when overridden. | — | — | Settings \| Editor \| Code Style \| Enable EditorConfig support | P0 |
| PLATFORM-EDIT-094 | Large file handling | Files above a size threshold (Advanced Setting, default 2.5 MB (inferred)) open without code insight/highlighting; above a larger limit open read-only in a paged "large file" viewer with search (inferred); indexing excludes files above the intellisense limit. | — | — | Settings \| Advanced Settings | P0 |
| PLATFORM-EDIT-095 | Read-only files (`ToggleReadOnlyAttribute`) | Toggles the OS writable flag; lock icon in status bar/tab; editing a read-only or VCS-locked file prompts "Make writable?"; library and generated files are non-editable with a banner. | — | — | File \| File Properties \| Make File Read-Only; status bar lock | P0 |
| PLATFORM-EDIT-096 | External change detection (`Synchronize`) | Watches the file system; reloads unmodified editors automatically; prompts on conflicting changes with diff; "Reload All from Disk" forces a refresh. Options: sync on frame activation. | Ctrl+Alt+Y | Cmd+Opt+Y | File \| Reload All from Disk | P0 |
| PLATFORM-EDIT-097 | Save model (`SaveAll`) | Implicit autosave: on frame deactivation, on idle after N seconds, before run/VCS; Save All saves everything; unsaved marker on tabs is optional. | Ctrl+S | Cmd+S | File \| Save All | P0 |

### 1.5 Undo, local history, macros, scratch

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-100 | Undo/Redo (`$Undo`, `$Redo`) | Per-document stacks plus global (cross-file) commands for refactorings; typing is coalesced into groups; caret/selection is restored; undoing a global command prompts if it affects other files; undo after file close works while history kept; optional confirmation for undoing multi-file changes. | Ctrl+Z; Ctrl+Shift+Z | Cmd+Z; Cmd+Shift+Z | Edit \| Undo/Redo | P0 |
| PLATFORM-EDIT-101 | Local History (`LocalHistory.ShowHistory`, `LocalHistory.ShowSelectionHistory`, `LocalHistory.PutLabel`, `RecentChanges`) | IDE-maintained revision store of every file/dir change (saves, refactorings, VCS ops, external changes), independent of VCS; time-ordered list with diff; revert a file, folder, or selection; user labels; retention period configurable (default 5 working days (inferred)). Recent Changes popup lists latest change sets. | — ; Recent Changes Alt+Shift+C | — ; Opt+Shift+C | File / context menu \| Local History | P0 |
| PLATFORM-EDIT-102 | Macro recording (`StartStopMacroRecording`, `PlaybackLastMacro`, `PlaySavedMacrosAction`, `EditMacros`) | Records keystrokes and editor actions (not mouse or dialogs); saved macros get names, can be bound to shortcuts, renamed or deleted. | — | — | Edit \| Macros | P1 |
| PLATFORM-EDIT-103 | Scratch files (`NewScratchFile`, `Scratch.ShowFilesPopup`) | Temporary files of any registered language stored outside projects (IDE config `scratches/`), with full editing support; visible in Project view "Scratches and Consoles"; can be run where language supports it. | Ctrl+Alt+Shift+Insert | Cmd+Shift+N | File \| New \| Scratch File | P1 |
| PLATFORM-EDIT-104 | Scratch buffers (`NewScratchBuffer`) | Numbered plain-text buffers (buffer1.txt, ...) recycled after a fixed count (default 5 (inferred)). | — | — | Find Action | P2 |

### 1.6 Find/replace in file, highlighting search

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-110 | Find in file (`Find`) | Inline search bar over the editor: incremental highlight of all matches, count "N/M", options Match Case, Words, Regex; search in selection when multi-line selection exists; filter (anywhere / in comments / in string literals / except ...); history dropdown. | Ctrl+F | Cmd+F | Edit \| Find \| Find | P0 |
| PLATFORM-EDIT-111 | Find next/previous (`FindNext`, `FindPrevious`, `FindWordAtCaret`, `FindPrevWordAtCaret`) | Cycles matches, also works after the bar closes; word-at-caret variants search without opening the bar. | F3 / Shift+F3 (Ctrl+L / Ctrl+Shift+L); Ctrl+F3 | Cmd+G / Cmd+Shift+G | Edit \| Find | P0 |
| PLATFORM-EDIT-112 | Replace in file (`Replace`) | Adds a replace field; Replace, Replace All, Exclude; regex groups `$1`, case-preserving replace option, `\n` multiline; Ctrl+Alt+E toggles "in selection only". | Ctrl+R | Cmd+R | Edit \| Find \| Replace | P0 |
| PLATFORM-EDIT-113 | Regex engine & help | Java-flavoured regex syntax (named groups, lookaround, `\l \u \L \U` case conversion in replacement); regex syntax checking and a cheat-sheet popup in the search field. | — | — | Find bar | P0 |
| PLATFORM-EDIT-114 | Multi-line search field and search history (`ShowSearchHistory`) | Search field expands to multiline; previous queries via history popup. | Alt+Down | Opt+Down | Find bar | P1 |
| PLATFORM-EDIT-115 | Select occurrences from find bar | "Select All Occurrences" from the bar converts matches to multi-caret selection. | Alt+Enter (in bar) (inferred) | Opt+Enter (inferred) | Find bar | P1 |

### 1.7 Code style, formatting, templates, structure editing

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-120 | Reformat Code (`ReformatCode`) | Formats selection, or whole file (or changed lines only when configured) per code style; works on directories/scopes from Project view with options (optimize imports, rearrange, cleanup, only VCS-changed text). Formatter-off/on markers respected. | Ctrl+Alt+L | Cmd+Opt+L | Code \| Reformat Code | P0 |
| PLATFORM-EDIT-121 | Reformat File dialog (`ShowReformatFileDialog`) | Dialog to choose scope and options before reformatting. | Ctrl+Alt+Shift+L | Cmd+Opt+Shift+L | Code \| Reformat File | P1 |
| PLATFORM-EDIT-122 | Code Style settings & schemes | Per-language tabs (Tabs and Indents, Spaces, Wrapping and Braces, Blank Lines, ...) with live preview; scheme stored at IDE level or in project (`.idea/codeStyles`); "Set from..." predefined styles; import/export. Generic "Other File Types" settings cover plain text. | — | — | Settings \| Editor \| Code Style | P0 |
| PLATFORM-EDIT-123 | Rearrange Code (`RearrangeCode`) | Reorders members per language arrangement rules (matching rules + section rules editable). | — | — | Code \| Rearrange Code | P2 |
| PLATFORM-EDIT-124 | Optimize Imports (`OptimizeImports`) | Framework action; language plugins implement import cleanup; "optimize on the fly" option. | Ctrl+Alt+O | Ctrl+Opt+O | Code \| Optimize Imports | P1 |
| PLATFORM-EDIT-125 | Live templates engine (`InsertLiveTemplate`, `ExpandLiveTemplateByTab`, `NextTemplateVariable`, `PreviousTemplateVariable`) | Abbreviation-triggered snippets with variables (`$NAME$`) bound to expression functions (e.g. date, clipboard, file name, camelCase, complete()), applicability contexts per language, expand key (Tab/Space/Enter), template groups, sharing via export, reformat-on-expand option; in-place variable editing cycles with Tab/Shift+Tab, Enter/Esc to finish. | Ctrl+J (insert); Tab (expand) | Cmd+J; Tab | Code \| Insert Live Template; Settings \| Editor \| Live Templates | P0 |
| PLATFORM-EDIT-126 | Surround With (`SurroundWith`, `SurroundWithLiveTemplate`) | Wraps the selection in a language construct or in a surround-type live template (those using `$SELECTION$`). | Ctrl+Alt+T; Ctrl+Alt+J | Cmd+Opt+T; Cmd+Opt+J | Code \| Surround With | P1 |
| PLATFORM-EDIT-127 | Unwrap/Remove (`Unwrap`) | Removes an enclosing construct, keeping its body (language-provided). | Ctrl+Shift+Delete | Cmd+Shift+Delete | Code \| Unwrap/Remove | P2 |
| PLATFORM-EDIT-128 | Postfix templates framework | Expression-suffix templates (`expr.if`) contributed by languages, editable per language. | Tab | Tab | Settings \| Editor \| General \| Postfix Completion | P2 |
| PLATFORM-EDIT-129 | Emmet / zen-coding expansion (`EmmetNextEditPoint`, `EmmetPreviousEditPoint`) | Abbreviation expansion for markup/CSS shipped with the HTML/XML support (not pure platform core); edit-point navigation; wrap with abbreviation; preview popup. | Tab; Alt+Shift+] / [ | Tab; Ctrl+Opt+Right/Left | Settings \| Editor \| Emmet | P2 |
| PLATFORM-EDIT-130 | Code completion framework (`CodeCompletion`, `SmartTypeCompletion`, `ClassNameCompletion`) | Lookup popup with prefix/camel-hump/fuzzy matching, sorting by relevance (ML ranking plugin), auto-popup delay, insert vs replace (Enter vs Tab), documentation side panel, "show completion variants as you type". Content is language-provided; generic word completion in plain text (inferred). | Ctrl+Space; Ctrl+Shift+Space; Ctrl+Alt+Space | Ctrl+Space; Ctrl+Shift+Space; Ctrl+Opt+Space | Code \| Code Completion | P0 |
| PLATFORM-EDIT-131 | Inline (full-line / multi-line) completion (`CallInlineCompletionAction`, `InsertInlineCompletionAction`, `Next/PrevInlineCompletionSuggestionAction`) | Grey ghost-text proposals; Tab accepts, word-by-word acceptance, cycle proposals. Local full-line models ship with the IDE; cloud providers via AI plugins. | Alt+Shift+\ ; Tab; Alt+] / Alt+[ | Opt+Shift+\ ; Tab; Opt+] / Opt+[ | Settings \| Editor \| General \| Inline Completion | P2 |
| PLATFORM-EDIT-132 | Intention / context actions framework (`ShowIntentionActions`, `QuickActionPopup`) | Light-bulb popup of quick-fixes and intentions at caret, with preview pane, sub-options (suppress, edit inspection settings), and the floating code toolbar. | Alt+Enter; Ctrl+Alt+Enter | Opt+Enter; Cmd+Opt+Enter | Editor bulb | P0 |
| PLATFORM-EDIT-133 | Generate menu (`Generate`) | Framework popup; languages contribute generators. Also used for "New..." in Project view. | Alt+Insert | Cmd+N | Code \| Generate | P1 |
| PLATFORM-EDIT-134 | Rename refactoring framework (`RenameElement`) | In-place rename with live preview of all occurrences; for files/directories opens dialog; language-agnostic for files, text-occurrence search in comments/strings optional. | Shift+F6 | Shift+F6 | Refactor \| Rename | P0 |
| PLATFORM-EDIT-135 | Refactor This & other refactoring framework actions (`Refactorings.QuickListPopupAction`, `Move`, `CopyElement`, `SafeDelete`) | Contextual list of applicable refactorings; Move/Copy/Safe Delete are generic for files/dirs (Safe Delete searches usages before deleting). Language-specific refactorings (extract, inline, change signature) are out of scope here. | Ctrl+Alt+Shift+T; F6; F5; Alt+Delete | Ctrl+T; F6; F5; Cmd+Delete | Refactor menu | P1 |

### 1.8 Spelling, natural language, TODO, injections, inspections

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-140 | Spell checking | Checks identifiers (split by camel case/underscores), comments, string literals, and plain text against bundled English dictionary plus user dictionaries (application- and project-level) and custom `.dic` files; quick-fixes "Typo: change to...", "Save to dictionary". | — | — | Settings \| Editor \| Natural Languages \| Spelling | P0 |
| PLATFORM-EDIT-141 | Grammar & style checking (Grazie, bundled plugin) | Natural-language grammar/style rules in comments, docs, commits and text files; multiple languages downloadable on demand; per-rule toggles; scope selection (comments, literals, commit messages). Grazie Pro adds AI-backed features (P2). | — | — | Settings \| Editor \| Natural Languages \| Grammar and Style | P1 |
| PLATFORM-EDIT-142 | TODO patterns and filters | Regex patterns (defaults `\btodo\b`, `\bfixme\b`, case-insensitive) matched in comments; custom highlight colour/icon per pattern; named filters combining patterns; multi-line TODO continuation by indentation. | — | — | Settings \| Editor \| TODO | P0 |
| PLATFORM-EDIT-143 | Language injection framework | Embeds one language inside another's string literal or region (e.g. SQL/regex in a string) via annotations, comments (`language=...`) or "Inject language or reference" intention; fragment editor opens injected text in a separate editor. Generic mechanism is platform (IntelliLang plugin bundled). | Alt+Enter | Opt+Enter | Intention; Settings \| Editor \| Language Injections | P1 |
| PLATFORM-EDIT-144 | Inspections framework & highlighting levels | Background "daemon" analysis producing highlights by severity (Error, Warning, Weak Warning, Server Problem, Typo, Grammar error, custom); per-file highlighting level (none / syntax / all / essential) via status widget (`PopupHector`); inspection profiles (IDE/project) with scopes; suppression. | Ctrl+Alt+Shift+H | Cmd+Opt+Shift+H | Settings \| Editor \| Inspections; inspection widget | P0 |
| PLATFORM-EDIT-145 | Run inspections / Inspect Code / code cleanup (`RunInspection`, `CodeInspection.OnEditor`) | Batch analysis over a scope with results in the Problems tool window ("Inspection" tab), quick-fix-all, exporting results. Run a single inspection by name. | Ctrl+Alt+Shift+I; Alt+Shift+I | Cmd+Opt+Shift+I; Opt+Shift+I | Code \| Inspect Code / Analyze Code | P1 |
| PLATFORM-EDIT-146 | Navigate between problems (`GotoNextError`, `GotoPreviousError`) | Jumps to next/previous highlighted problem in file; option to prioritise errors over warnings. | F2 / Shift+F2 | F2 / Shift+F2 | Navigate \| Next/Previous Highlighted Error | P0 |
| PLATFORM-EDIT-147 | File types & syntax highlighting fallback via TextMate bundles (plugin) | TextMate grammars give highlighting and basic brace matching for languages lacking native support; users can add bundles. | — | — | Settings \| Editor \| TextMate Bundles | P1 |
| PLATFORM-EDIT-148 | Language Server Protocol client (platform LSP API) | Platform-provided LSP client used by plugins to supply completion, diagnostics, navigation, formatting for languages without native support (since 2023.2, extended in later releases). | — | — | — (plugin-provided) | P0 |

### 1.9 Editor view modes

| ID | Feature (action id) | Behaviour | Win/Linux | macOS | Entry point | Pri |
|---|---|---|---|---|---|---|
| PLATFORM-EDIT-150 | Distraction-free mode (`ToggleDistractionFreeMode`) | Hides tool windows, toolbars, tabs, gutter adornments; centres the editor text column. | — | — | View \| Appearance \| Enter Distraction-Free Mode | P1 |
| PLATFORM-EDIT-151 | Zen mode (`ToggleZenMode`) | Distraction-free plus full screen. | — | — | View \| Appearance \| Enter Zen Mode | P1 |
| PLATFORM-EDIT-152 | Presentation mode (`TogglePresentationMode`) | Full-screen editor with large configurable font size for demos; tool windows hidden. | — | — | View \| Appearance \| Enter Presentation Mode | P2 |
| PLATFORM-EDIT-153 | Full screen (`ToggleFullScreen`) | Native OS full screen for the frame. | — | Ctrl+Cmd+F | View \| Appearance \| Enter Full Screen | P1 |
| PLATFORM-EDIT-154 | Presentation assistant (`TogglePresentationAssistantAction`) | Overlay showing the name and shortcut of each invoked action (for demos/screencasts). | — | — | View \| Appearance \| Presentation Assistant | P2 |
| PLATFORM-EDIT-155 | Reader mode | Library/read-only files render with rendered doc comments, ligatures and reduced highlighting noise by default; toggle in editor corner. | — | — | Settings \| Editor \| Reader Mode | P2 |

### Sources (Editor core)

- https://www.jetbrains.com/help/idea/working-with-source-code.html
- https://www.jetbrains.com/help/idea/multicursor.html
- https://www.jetbrains.com/help/idea/working-with-code-selection.html (inferred URL, extend/shrink selection)
- https://www.jetbrains.com/help/idea/working-with-clipboard.html (inferred URL, Paste from history)
- https://www.jetbrains.com/help/idea/code-folding.html
- https://www.jetbrains.com/help/idea/using-code-editor.html
- https://www.jetbrains.com/help/idea/configuring-line-endings-and-line-separators.html
- https://www.jetbrains.com/help/idea/encoding.html
- https://www.jetbrains.com/help/idea/editorconfig.html
- https://www.jetbrains.com/help/idea/local-history.html
- https://www.jetbrains.com/help/idea/using-macros-in-the-editor.html
- https://www.jetbrains.com/help/idea/scratches.html
- https://www.jetbrains.com/help/idea/finding-and-replacing-text-in-file.html
- https://www.jetbrains.com/help/idea/reformat-and-rearrange-code.html
- https://www.jetbrains.com/help/idea/using-live-templates.html
- https://www.jetbrains.com/help/idea/surrounding-blocks-of-code-with-language-constructs.html
- https://www.jetbrains.com/help/idea/spellchecking.html and https://www.jetbrains.com/help/idea/grammar.html
- https://www.jetbrains.com/help/idea/using-todo.html
- https://www.jetbrains.com/help/idea/using-language-injections.html
- https://www.jetbrains.com/help/idea/editor-modes.html (Zen / Distraction-free / Presentation)
- https://www.jetbrains.com/help/idea/reader-mode.html
- https://plugins.jetbrains.com/docs/intellij/editors.html, https://plugins.jetbrains.com/docs/intellij/language-server-protocol.html
- Keymap descriptors (Apache-2.0): IntelliJ Community `platform/platform-resources/src/keymaps/$default.xml`, `Mac OS X 10.5+.xml`; action ids from `PlatformActions.xml`, `LangActions.xml`; labels from `ActionsBundle.properties` (tag `idea/262.10968.63`).
- Parallel PyCharm pages used to confirm platform scope: https://www.jetbrains.com/help/pycharm/multicursor.html, https://www.jetbrains.com/help/pycharm/local-history.html

---
