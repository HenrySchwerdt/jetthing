# IntelliJ IDEA parity: editor and coding assistance

Editing, caret movement, selection, multiple carets, folding, completion, code analysis,
intentions, templates, and formatting. Shortcuts refer to the default Windows/Linux
keymap so features are identifiable; Jetthing may use different default bindings.

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
| IJ-ED-001 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Text input with Unicode-safe editing | Implemented | 5 | `editor.insertText` | rope unit/property tests | Unicode-safe rope edits; IME composition pending |
| IJ-ED-002 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Backspace / Delete | Implemented | 5 | `editor.insertText` | multi-cursor deletion unit test | — |
| IJ-ED-003 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Enter with smart indent | Partial | 5 | `editor.insertText` | — | Enter inserts newline; language-aware indent pending |
| IJ-ED-004 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Start New Line / Start New Line Before Current (Shift+Enter / Ctrl+Alt+Enter) | Missing | 2 | — | — | not started |
| IJ-ED-005 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Indent / Unindent selection (Tab / Shift+Tab) | Partial | 3 | `editor.insertText` | — | Tab inserts indentation; block unindent pending |
| IJ-ED-006 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Auto-Indent Lines (Ctrl+Alt+I) | Missing | 2 | — | — | not started |
| IJ-ED-007 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Move Line Up/Down (Alt+Shift+Up/Down) | Missing | 3 | — | — | not started |
| IJ-ED-008 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Move Statement Up/Down (Ctrl+Shift+Up/Down) | Missing | 2 | — | — | not started |
| IJ-ED-009 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Move Element Left/Right (Ctrl+Alt+Shift+Left/Right) | Missing | 1 | — | — | not started |
| IJ-ED-010 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Comment with Line Comment (Ctrl+/) | Missing | 5 | — | — | not started |
| IJ-ED-011 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Comment with Block Comment (Ctrl+Shift+/) | Missing | 2 | — | — | not started |
| IJ-ED-012 | [completing-punctuation](https://www.jetbrains.com/help/idea/completing-punctuation.html) | Auto-insert paired brackets and quotes | Missing | 3 | — | — | not started |
| IJ-ED-013 | [completing-punctuation](https://www.jetbrains.com/help/idea/completing-punctuation.html) | Complete Current Statement (Ctrl+Shift+Enter) | Missing | 3 | — | — | not started |
| IJ-ED-014 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Increase / decrease editor font size (Alt+Shift+. / Alt+Shift+,; Ctrl+mouse wheel) | Missing | 2 | — | — | not started |
| IJ-ED-015 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Soft wraps | Missing | 2 | — | — | not started |
| IJ-ED-016 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Show whitespace, indent guides, right margin | Missing | 1 | — | — | not started |
| IJ-ED-017 | [editor-gutter](https://www.jetbrains.com/help/idea/editor-gutter.html) | Gutter: line numbers | Partial | 3 | — | manual launch | line numbers painted; relative/hybrid numbering pending |
| IJ-ED-018 | [editor-gutter](https://www.jetbrains.com/help/idea/editor-gutter.html) | Gutter: icons (run, breakpoints, overrides, VCS change markers) | Missing | 3 | — | — | not started |
| IJ-ED-019 | [sticky-lines](https://www.jetbrains.com/help/idea/sticky-lines.html) | Sticky lines (enclosing scopes pinned at top) | Missing | 2 | — | — | not started |
| IJ-ED-020 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Large-file viewport rendering | Implemented | 3 | `editor.openFile` | manual launch | only visible lines are shaped and painted |
| IJ-ED-021 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Editor tabs with modified markers | Partial | 5 | `editor.openFile`, `editor.save` | manual launch | tabs and dirty state; no pinning, splits, or drag reorder |
| IJ-ED-022 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html#split_screen) | Split editors showing the same document | Missing | 3 | — | — | not started |
| IJ-ED-023 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Scrollbar error stripe / inspection widget | Missing | 2 | — | — | not started |
| IJ-ED-024 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Breadcrumbs at editor bottom/top | Missing | 1 | — | — | not started |
| IJ-ED-025 | [using-product-as-the-vim-editor](https://www.jetbrains.com/help/idea/using-product-as-the-vim-editor.html) | Vim emulation (IdeaVim plugin) | Missing | 1 | — | — | not started |
| IJ-ED-026 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Toggle Case / Join Lines / Duplicate Line (editor behavior) | Missing | 2 | — | — | menu entries tracked in IJ-MENU-041..044 |
| IJ-ED-027 | [reader-mode](https://www.jetbrains.com/help/idea/reader-mode.html) | Reader mode (rendered doc comments, ligatures) | Missing | 1 | — | — | not started |
| IJ-ED-028 | [local-history](https://www.jetbrains.com/help/idea/local-history.html) | Local History: automatic revisions, show history, revert | Missing | 3 | — | — | not started |
| IJ-ED-030 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Caret Left/Right by character | Implemented | 5 | — | manual launch | handled in editor input; not a registered command |
| IJ-ED-031 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Caret Up/Down by line with column memory | Missing | 5 | — | — | not started |
| IJ-ED-032 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Previous/Next word (Ctrl+Left/Right) | Missing | 3 | — | — | not started |
| IJ-ED-033 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Line start/end (Home/End), smart Home | Missing | 3 | — | — | not started |
| IJ-ED-034 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Page Up/Down, text start/end (Ctrl+Home/End) | Missing | 3 | — | — | not started |
| IJ-ED-035 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Move caret to matching brace (Ctrl+Shift+M) | Missing | 2 | — | — | not started |
| IJ-ED-036 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Code block start/end (Ctrl+[ / Ctrl+]) | Missing | 1 | — | — | not started |
| IJ-ED-037 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Shift+movement extends selection | Partial | 5 | — | — | Shift+Left/Right only |
| IJ-ED-038 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Mouse click, drag-select, double-click word, triple-click line | Partial | 5 | — | — | click places caret; drag and multi-click selection pending |
| IJ-ED-039 | [working-with-source-code](https://www.jetbrains.com/help/idea/working-with-source-code.html#editor_code_selection) | Extend / Shrink Selection (Ctrl+W / Ctrl+Shift+W) | Implemented | 3 | `editor.extendSelection`, `editor.shrinkSelection` | selection step tests (Java, Kotlin, Rust), command history test, Xvfb screenshot | Tree-sitter driven for every registered language, multi-caret, Ctrl+W / Ctrl+Shift+W (⌥↑/⌥↓ on macOS); `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-040 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Select All (Ctrl+A) | Partial | 2 | — | — | editor shortcut; see IJ-MENU-038 |
| IJ-ED-050 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Add/Remove Caret (Alt+Shift+Click) | Partial | 3 | `editor.insertText` | multi-cursor unit tests | carets added by modifier-click; removal and keyboard cloning pending |
| IJ-ED-051 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Edits apply atomically at all carets | Implemented | 3 | `editor.insertText` | multi-cursor atomic transaction tests | overlapping edits rejected without mutation |
| IJ-ED-052 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Clone Caret Above/Below | Missing | 2 | — | — | not started |
| IJ-ED-053 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Add Selection for Next Occurrence (Alt+J) / Select All Occurrences | Missing | 3 | — | — | not started |
| IJ-ED-054 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Column (block) selection mode (Alt+Shift+Insert, Alt+drag) | Missing | 2 | — | — | not started |
| IJ-ED-055 | [multicursor](https://www.jetbrains.com/help/idea/multicursor.html) | Add caret to each line in selection (Alt+Shift+G) | Missing | 1 | — | — | not started |
| IJ-ED-060 | [working-with-source-code](https://www.jetbrains.com/help/idea/working-with-source-code.html#code_folding) | Code folding regions from syntax | Implemented | 3 | `editor.collapseRegion`, `editor.expandRegion`, `editor.collapseAllRegions`, `editor.expandAllRegions`, `editor.toggleFold` | fold-region, fold-state and row-mapping tests, Xvfb screenshots | regions from the Tree-sitter tree (blocks, block comments, imports; `folds.scm` where shipped), gutter chevrons, `{...}`/`import ...` placeholders, imports collapsed on open; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-061 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Expand / Collapse / Expand All / Collapse All | Missing | 2 | — | — | not started |
| IJ-ED-062 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Fold Selection (Ctrl+.) and custom folding regions | Missing | 1 | — | — | not started |
| IJ-ED-070 | [configuring-colors-and-fonts](https://www.jetbrains.com/help/idea/configuring-colors-and-fonts.html) | Syntax highlighting | Implemented | 5 | `editor` edit pipeline | registry/query/capture-mapping tests, per-language token tests, incremental-vs-full proptest, Xvfb scene 04 screenshots | Tree-sitter grammar crates + their bundled highlight queries (Rust, Java, Kotlin, XML, JSON, TOML, YAML, Markdown, Starlark/Bazel, Python, JS/TS/TSX, Go, C/C++, Bash, HTML, CSS, Properties), IntelliJ text-attribute keys resolved through Islands Dark; incremental reparse per keystroke; semantic colors (static members, resolved references) are IJ-ED-071; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-071 | [configuring-colors-and-fonts](https://www.jetbrains.com/help/idea/configuring-colors-and-fonts.html) | Semantic highlighting (resolved references, rainbow identifiers) | Missing | 2 | — | — | not started |
| IJ-ED-072 | [find-highlight-usages](https://www.jetbrains.com/help/idea/find-highlight-usages.html) | Highlight usages of symbol at caret | Missing | 3 | — | — | not started |
| IJ-ED-073 | [using-code-editor](https://www.jetbrains.com/help/idea/using-code-editor.html) | Highlight matching brace | Implemented | 2 | editor caret highlight | bracket-matching tests (Java, Kotlin, JSON; strings ignored) | structural matching from the Tree-sitter tree for every registered language (brackets in strings/comments never match, unmatched brackets in ERROR nodes stay unhighlighted); text matching for plain-text files; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-074 | [language-injections](https://www.jetbrains.com/help/idea/language-injections.html) | Language injections (SQL/regex/JSON in strings) | Partial | 1 | — | injection detection and injected-highlight tests, proptest with Markdown/HTML | grammar injections from bundled `injections.scm`: Markdown fences and inline, HTML script/style, Rust macros, JS tagged templates; IntelliJ-style SQL/regex/JSON injection into string literals and Inject Language intention missing; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-080 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Basic completion (Ctrl+Space) | Partial | 5 | `language.complete`, `language.completion` | framed fake-server integration test | Rust via LSP; no prefix filtering or ranking |
| IJ-ED-081 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Automatic completion popup while typing | Missing | 5 | — | — | not started |
| IJ-ED-082 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Type-matching (smart) completion (Ctrl+Shift+Space) | Missing | 2 | — | — | not started |
| IJ-ED-083 | [advanced-code-completion](https://www.jetbrains.com/help/idea/advanced-code-completion.html) | Second basic completion / statics and unimported symbols | Missing | 2 | — | — | not started |
| IJ-ED-084 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Completion with auto-import | Missing | 3 | — | — | not started |
| IJ-ED-085 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Completion lookup: documentation preview, Tab replace vs Enter insert | Missing | 2 | — | — | not started |
| IJ-ED-086 | [postfix-code-completion](https://www.jetbrains.com/help/idea/postfix-code-completion.html) | Postfix completion (.if, .for, .var, .not) | Missing | 2 | — | — | not started |
| IJ-ED-087 | [command-completion](https://www.jetbrains.com/help/idea/command-completion.html) | Command completion (actions in completion popup) | Missing | 1 | — | — | not started |
| IJ-ED-088 | [full-line-code-completion](https://www.jetbrains.com/help/idea/full-line-code-completion.html) | Full line code completion (local ML model) | Missing | 2 | — | — | not started |
| IJ-ED-089 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Snippet/placeholder insertion from completion items | Missing | 2 | — | — | not started |
| IJ-ED-090 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Hippie (cyclic word) completion (Alt+/) | Missing | 1 | — | — | not started |
| IJ-ED-100 | [viewing-reference-information](https://www.jetbrains.com/help/idea/viewing-reference-information.html) | Quick Documentation popup (Ctrl+Q) and hover docs | Missing | 3 | — | — | not started |
| IJ-ED-101 | [viewing-reference-information](https://www.jetbrains.com/help/idea/viewing-reference-information.html) | Parameter Info (Ctrl+P) | Missing | 3 | — | — | not started |
| IJ-ED-102 | [viewing-definition](https://www.jetbrains.com/help/idea/viewing-definition.html) | Quick Definition (Ctrl+Shift+I) | Missing | 2 | — | — | not started |
| IJ-ED-103 | [viewing-reference-information](https://www.jetbrains.com/help/idea/viewing-reference-information.html) | Type Info (Ctrl+Shift+P) | Missing | 1 | — | — | not started |
| IJ-ED-104 | [inlay-hints](https://www.jetbrains.com/help/idea/inlay-hints.html) | Inlay hints (parameter names, types, code vision usages) | Missing | 3 | — | — | not started |
| IJ-ED-105 | [javadocs](https://www.jetbrains.com/help/idea/javadocs.html) | Rendered Javadoc in editor | Missing | 1 | — | — | not started |
| IJ-ED-106 | [keymap](https://www.jetbrains.com/help/idea/reference-keymap-win-default.html) | Error description tooltip (Ctrl+F1) | Missing | 2 | — | — | not started |
| IJ-ED-110 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | On-the-fly error highlighting | Partial | 5 | editor edit pipeline | malformed Rust/Java tests; framed fake-server integration test | Tree-sitter ERROR/MISSING diagnostics for every registered language plus LSP diagnostics; `docs/parity/evidence/syntax-highlighting.md` |
| IJ-ED-111 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Inspection warnings and weak warnings with severities | Partial | 3 | — | framed fake-server integration test | LSP severities displayed; no inspection engine |
| IJ-ED-112 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Next/Previous Highlighted Error (F2 / Shift+F2) | Missing | 3 | — | — | not started |
| IJ-ED-113 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Inspect Code (batch analysis over scope) | Missing | 3 | — | — | not started |
| IJ-ED-114 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Run Inspection by Name (Ctrl+Alt+Shift+I) | Missing | 1 | — | — | not started |
| IJ-ED-115 | [customizing-profiles](https://www.jetbrains.com/help/idea/customizing-profiles.html) | Inspection profiles and suppression comments | Missing | 2 | — | — | not started |
| IJ-ED-116 | [analyzing-duplicates](https://www.jetbrains.com/help/idea/analyzing-duplicates.html) | Duplicate code detection | Missing | 1 | — | — | not started |
| IJ-ED-117 | [analyzing-data-flow](https://www.jetbrains.com/help/idea/analyzing-data-flow.html) | Data flow analysis (Analyze > Data Flow to/from Here) | Missing | 1 | — | — | not started |
| IJ-ED-118 | [spellchecking](https://www.jetbrains.com/help/idea/spellchecking.html) | Spelling and grammar checks | Missing | 1 | — | — | not started |
| IJ-ED-119 | [dependencies-analysis](https://www.jetbrains.com/help/idea/dependencies-analysis.html) | Analyze dependencies / cyclic dependencies | Missing | 1 | — | — | not started |
| IJ-ED-120 | [intention-actions](https://www.jetbrains.com/help/idea/intention-actions.html) | Show Context Actions (Alt+Enter): quick-fixes | Missing | 5 | — | — | not started |
| IJ-ED-121 | [intention-actions](https://www.jetbrains.com/help/idea/intention-actions.html) | Intention actions (non-error transformations) | Missing | 3 | — | — | not started |
| IJ-ED-122 | [generating-code](https://www.jetbrains.com/help/idea/generating-code.html) | Generate (Alt+Insert): constructors, getters/setters, equals/hashCode, toString | Missing | 3 | — | — | not started |
| IJ-ED-123 | [generating-code](https://www.jetbrains.com/help/idea/generating-code.html) | Override / Implement Methods (Ctrl+O / Ctrl+I) | Missing | 3 | — | — | not started |
| IJ-ED-124 | [surrounding-blocks-of-code-with-language-constructs](https://www.jetbrains.com/help/idea/surrounding-blocks-of-code-with-language-constructs.html) | Surround With (Ctrl+Alt+T) | Missing | 2 | — | — | not started |
| IJ-ED-125 | [creating-and-optimizing-imports](https://www.jetbrains.com/help/idea/creating-and-optimizing-imports.html) | Optimize Imports (Ctrl+Alt+O) | Missing | 3 | — | — | not started |
| IJ-ED-126 | [creating-and-optimizing-imports](https://www.jetbrains.com/help/idea/creating-and-optimizing-imports.html) | Auto-import on paste and unambiguous imports on the fly | Missing | 2 | — | — | not started |
| IJ-ED-130 | [using-live-templates](https://www.jetbrains.com/help/idea/using-live-templates.html) | Live templates expansion (Tab) with variables | Missing | 3 | — | — | not started |
| IJ-ED-131 | [using-live-templates](https://www.jetbrains.com/help/idea/using-live-templates.html) | Insert Live Template (Ctrl+J) / Surround with live template | Missing | 2 | — | — | not started |
| IJ-ED-132 | [using-file-and-code-templates](https://www.jetbrains.com/help/idea/using-file-and-code-templates.html) | File templates with variables | Missing | 2 | — | — | not started |
| IJ-ED-140 | [reformat-and-rearrange-code](https://www.jetbrains.com/help/idea/reformat-and-rearrange-code.html) | Reformat Code (Ctrl+Alt+L) for file or selection | Missing | 5 | — | — | not started |
| IJ-ED-141 | [reformat-and-rearrange-code](https://www.jetbrains.com/help/idea/reformat-and-rearrange-code.html) | Reformat on paste and on typing | Missing | 2 | — | — | not started |
| IJ-ED-142 | [rearrange-code](https://www.jetbrains.com/help/idea/rearrange-code.html) | Rearrange Code | Missing | 1 | — | — | not started |
| IJ-ED-143 | [indentation](https://www.jetbrains.com/help/idea/indentation.html) | Detect and use file indentation (tabs/spaces) | Missing | 2 | — | — | not started |
| IJ-ED-144 | [configuring-line-endings-and-line-separators](https://www.jetbrains.com/help/idea/configuring-line-endings-and-line-separators.html) | Preserve line separators and encoding on save | Missing | 2 | — | — | not started |
| IJ-ED-150 | [scratches](https://www.jetbrains.com/help/idea/scratches.html) | Scratch files and buffers | Missing | 2 | — | — | not started |
| IJ-ED-151 | [markdown](https://www.jetbrains.com/help/idea/markdown.html) | Markdown editor with preview | Missing | 2 | — | — | not started |
| IJ-ED-153 | [json](https://www.jetbrains.com/help/idea/json.html) | JSON / YAML / TOML / XML editing with schema validation | Missing | 2 | — | — | plain text editing only |
| IJ-ED-154 | [properties-files](https://www.jetbrains.com/help/idea/properties-files.html) | .properties files and resource bundle editor | Missing | 1 | — | — | not started |
| IJ-ED-155 | [shell-scripts](https://www.jetbrains.com/help/idea/shell-scripts.html) | Shell script editing and ShellCheck | Missing | 1 | — | — | not started |
| IJ-ED-156 | [ai-assistant-in-jetbrains-ides](https://www.jetbrains.com/help/idea/ai-assistant-in-jetbrains-ides.html) | [Ultimate] AI Assistant inline code generation and chat | Missing | 2 | — | — | not started |
