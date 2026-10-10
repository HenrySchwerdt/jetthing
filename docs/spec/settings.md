# Settings dialog and Project Structure dialog

Reference specification for the **Settings** dialog of IntelliJ IDEA 2026.2 (New UI, Islands
themes) as a user sees it, plus the **Project Structure** dialog. It complements the generated
[settings-tree.md](settings-tree.md) (pages registered declaratively in IntelliJ Community XML)
by covering pages that are registered in code or exist only in the commercial build, and by
listing the options on each page.

Clean-room status: researched from the public IntelliJ IDEA help, the "What's New" pages, and
the Apache-2.0 message bundles (`messages/*.properties`) of IntelliJ Community tag
`idea/262.10968.63`, used only for exact UI label text. No code, artwork or documentation prose
was copied; descriptions are our own words. See [README.md](README.md) for the rules.

## How to read this document

- **Page ID**: `SET-<AREA>-NNN`, stable and never reused. Areas: `DLG` dialog behaviour,
  `AB` Appearance & Behavior, `KM` Keymap, `ED` Editor, `CS` Code Style, `PLG` Plugins,
  `VCS` Version Control, `BLD` Build, Execution, Deployment, `LANG` Languages & Frameworks,
  `TOOL` Tools, `ADV` Advanced Settings, `BAK` Backup and Sync / settings transfer,
  `PSD` Project Structure dialog.
- **Configurable id**: the IntelliJ `id` of the page. Ids printed in `code` without a marker
  appear in [settings-tree.md](settings-tree.md) / `data/settings.json`. Ids marked
  "(inferred)" are believed correct from public plugin metadata or help-page anchors but are
  not in the generated data; implementers should key on them only after verification.
- **Level**: `App` = application-level (stored in the IDE config directory, applies to all
  projects); `Proj` = project-level (stored in `.idea/`, shown with the project marker in the
  dialog). `Proj*` = project-level but also editable through *Settings for New Projects*.
- **Control** vocabulary: `checkbox`, `combobox` (drop-down), `spinner` (numeric field),
  `text` (single-line), `path` (text with Browse button), `radio` (radio group), `table`
  (list/table with +/-/edit toolbar), `tree` (checkbox tree), `link`, `button`, `list`.
  Nested options are indented with `↳` in the label column and are disabled while their parent
  checkbox is off.
- **Default** provenance markers: `[doc]` stated in JetBrains help; `[adv]` from the generated
  Advanced Settings table; `[bundle]` stated in a label/comment string of the message bundle.
  **Every default without a marker is (inferred)** from observed behaviour of recent IntelliJ
  IDEA releases and must be verified against a clean 2026.2 install before it is used in a
  parity claim. `—` means no default (empty or not applicable); `?` means unknown.
- Labels are reproduced exactly where known (trailing colons included as shown in the UI).
  Mnemonic markers (`&`, `_`) from the bundles are dropped.

---

## 1. Settings dialog behaviour

### SET-DLG-001 Opening the dialog

| Item | Behaviour |
|---|---|
| Entry points | **File \| Settings…** (Windows/Linux), **IntelliJ IDEA \| Settings…** (macOS), main-menu gear in the toolbar, Welcome screen **Customize \| All settings…**, `Ctrl+Alt+S` / `⌘,` |
| Direct page open | Many actions open the dialog pre-selected on a page (e.g. Configure… links in notifications, gear icons in tool windows, *Configure Inlay Hints…* from an inlay context menu). The dialog then selects that page and expands its parents. |
| Modality | Non-modal by default: Advanced setting *Show settings in a non-modal dialog* `ide.ui.non.modal.settings.window` = `true` [adv]. While non-modal, the user can keep editing in the main window. |
| Settings as tab (inferred) | The 2026.2 bundles contain strings for a "Settings tab" ("On closing the Settings tab, all unsaved changes will be discarded"; "Settings on this page have been modified in another IDE window" with *Discard local changes and reload*). Settings can therefore be hosted in an editor-like tab in some mode; see Open questions. |
| Size/position | Remembered per IDE between invocations (inferred). Last selected page is restored on next open (inferred). |
| Project switching | If there are unapplied changes and the user switches/closes the project or application, a prompt **Unsaved Changes** offers **Apply** / **Don't Save** / Cancel [bundle]. |

### SET-DLG-002 Layout

| Region | Content |
|---|---|
| Left column | Search field (with history and clear button) above the settings **tree**. |
| Tree | Top-level nodes in this order: Appearance & Behavior, Keymap, Editor, Plugins, Version Control, Build, Execution, Deployment, Languages & Frameworks, Tools, *(plugin-contributed top-level pages such as Settings Sync-related or third-party)*, Backup and Sync, Advanced Settings. Group nodes are expandable; selecting a pure group node shows a page with links to its children. Plugin pages appear under their declared parent group. |
| Right pane header | **Breadcrumbs** of the selected page (e.g. `Editor › General › Auto Import`), each crumb clickable to navigate to the ancestor. To the right of the crumbs: project-level marker (icon plus text *For current project*) on project-level pages, and a **Reset** link (*Revert changes* action [bundle]) that appears only when the current page has unsaved modifications. |
| Right pane body | The page's options; long pages scroll. Some pages have tabs (Code Style per language, Plugins Marketplace/Installed). |
| Footer | **?** help button (opens the web help topic for the page), **OK**, **Cancel**, **Apply**. |
| Navigation | Back/Forward arrows for page history (inferred, present in older UI; verify). |

### SET-DLG-003 Search

| Aspect | Behaviour |
|---|---|
| What is matched | Page names and the *searchable option* index: option labels, combobox values, tab names, and synonyms registered by pages. Matching is case-insensitive, word-prefix based (typing `wrap` finds *Soft-wrap these files*). |
| Tree filtering | While a query is active the tree shows only pages that match (and their ancestors), expanded. Pages with no match are hidden. Clearing the field restores the full tree and keeps the selected page. |
| Highlighting | Matched text in the tree is highlighted, and on the selected page every label containing the query is highlighted with the search-match background (yellow-ish in light themes). |
| Auto-select | The first matching page is selected automatically when the current page does not match (inferred). |
| Empty result | Tree area shows *Nothing found* (inferred label). |
| Keyboard | Focus starts in the search field; `Down` moves into the tree; `Esc` clears the query, a second `Esc` closes (inferred). |
| Advanced Settings | Its own page has a local search field (*Search advanced settings*) and a *Show modified only* checkbox [bundle]; the global search also matches its rows. |

### SET-DLG-004 Apply, OK, Cancel, Reset and modification markers

| Control | Rule |
|---|---|
| Modified page marker | A page whose UI state differs from the stored state is marked in the tree (its name is rendered in the accent/link color, inferred) so the user sees where unapplied changes are. |
| Reset | Per page: *Reset* link in the header reverts that page to the last saved state. A whole-dialog *Revert All* action exists [bundle] (shown in context of the tree, inferred). |
| Apply | Enabled only when at least one page is modified. Applies all modified pages, keeps the dialog open; markers disappear. |
| OK | Applies all modified pages and closes. Always enabled (inferred). |
| Cancel / Esc / close box | Discards unapplied changes and closes. |
| Validation | If a page reports invalid input on Apply/OK, the dialog switches to that page, shows an inline error (red outline + message under the field) and does not close; other valid pages are still applied (inferred). |
| Restart-required options | Changing such options shows a prompt to restart after OK/Apply, with *Restart* / *Not Now* (inferred). |
| External change | If another IDE window or a sync changes a page being edited, a banner offers *Discard local changes and reload* [bundle]. |

### SET-DLG-005 Project-level vs application-level

| Aspect | Behaviour |
|---|---|
| Marker | Project-level pages show the project marker (stacked-pages icon) next to the breadcrumbs with tooltip/text *For current project* (wording inferred). In the tree, no marker is shown on nodes (inferred). |
| Mixed pages | Some pages mix levels: e.g. Code Style and Inspections have a *Scheme/Profile* combobox where `Project` scope schemes are stored in `.idea/` and `Default`/IDE schemes in the config dir; Terminal has *Project Settings* and *Application Settings* sections [bundle]. |
| Copy between levels | Scheme menus offer *Copy to Project…* / *Copy to IDE…* [bundle]. |

### SET-DLG-006 Settings for New Projects

| Aspect | Behaviour |
|---|---|
| Entry | **File \| New Projects Setup \| Settings for New Projects…**; Welcome screen **Customize \| All settings…** also edits these defaults (inferred). |
| Dialog | Same dialog, title *Settings for New Projects*. Project-level pages edit the *default project* template; application-level pages are the normal application settings. Pages that need a real project (e.g. Directory Mappings, Scopes content) are hidden or limited (inferred). |
| Effect | Values are copied into each newly created or newly imported project; existing projects are not changed. |
| Related | **File \| New Projects Setup \| Structure for New Projects…** opens the Project Structure dialog for the default project (SDK, language level); **Run Configuration Templates for New Projects…** opens the run-configuration templates dialog. |

### SET-DLG-007 Import / Export / Restore

| Action (menu **File \| Manage IDE Settings**) | Behaviour |
|---|---|
| Import Settings… | Choose a ZIP (or a settings directory); dialog *Select Components to Import* lists components with checkboxes; on OK the IDE restarts to apply [doc]. |
| Export Settings… | Dialog lists exportable components (checkbox tree), output path field; writes a ZIP [doc]. |
| Restore Default Settings… | Backs up the current config directory and restarts with defaults (inferred). |
| Import Settings from Other IDEs… | Imports from VS Code, Eclipse, Visual Studio, etc. (Welcome screen also offers it; inferred). |
| Backup and Sync | See SET-BAK-001. |
| Settings Repository… | Legacy Git-based sync, page `ics` (Tools); see SET-TOOL-016. |

Sources:
- https://www.jetbrains.com/help/idea/settings-preferences-dialog.html
- https://www.jetbrains.com/help/idea/sharing-your-ide-settings.html
- https://www.jetbrains.com/help/idea/configure-project-settings.html
- IntelliJ Community `platform/platform-api/resources/messages/UIBundle.properties`, `platform/ide-core/resources/messages/ApplicationBundle.properties` (labels only)

---

## 2. Complete tree (as shown in 2026.2 Ultimate with default bundled plugins)

Pages marked † are not in the Community data of settings-tree.md (registered in code or
commercial-only). `[P]` = project-level. This is the navigational outline; options follow in
sections 3–12.

- Appearance & Behavior
  - Appearance (SET-AB-001), New UI options are merged into it
  - Menus and Toolbars (SET-AB-002)
  - System Settings [P] (SET-AB-003)
    - Data Sharing, Date Formats, HTTP Proxy, Language and Region, Passwords, Process Elevation (Windows), Server Certificates, Updates (SET-AB-004…011)
  - File Colors [P] (SET-AB-012)
  - Scopes [P] (SET-AB-013)
  - Notifications (SET-AB-014)
  - Data Editor and Viewer (SET-AB-020)
  - Quick Lists (SET-AB-015)
  - Required Plugins [P] (SET-AB-016)
  - Trusted Locations (SET-AB-017)
  - Path Variables [P] (SET-AB-018)
  - Presentation Assistant (SET-AB-019)
- Keymap (SET-KM-001)
- Editor
  - General (SET-ED-001)
    - Auto Import [P] (SET-ED-002)
    - Appearance † (SET-ED-003)
    - Breadcrumbs (SET-ED-004)
    - Code Completion † (SET-ED-005)
      - Inline Completion † (SET-ED-005a)
    - Code Folding † (SET-ED-006)
    - Console (SET-ED-007)
    - Editor Tabs † (SET-ED-008)
    - Gutter Icons † (SET-ED-009)
    - Postfix Completion † (SET-ED-010)
    - Smart Keys † (SET-ED-011)
      - HTML/CSS †, JavaScript † (commercial), Kotlin options inline
    - Sticky Lines (SET-ED-012)
  - Code Editing (SET-ED-013)
  - Font (SET-ED-014)
    - Console Font (SET-ED-014a)
  - Color Scheme (SET-ED-015) with ~40 language/area subpages
  - Code Style [P] (SET-CS-001) → Java (SET-CS-010) and other languages
  - Inspections [P] (SET-ED-016)
  - File and Code Templates [P] (SET-ED-017)
  - File Encodings [P] (SET-ED-018)
  - Live Templates (SET-ED-019)
  - File Types (SET-ED-020)
  - Copyright [P] (SET-ED-021) → Copyright Profiles, Formatting
  - Inlay Hints [P] (SET-ED-022)
  - Duplicates † (commercial, inferred)
  - Emmet (SET-ED-029)
  - GUI Designer [P] (SET-ED-030)
  - Intentions (SET-ED-023)
  - Language Injections [P] (SET-ED-024)
  - Minimap (SET-ED-031)
  - Natural Languages (SET-ED-025) → Grammar and Style, Spelling [P]
  - Reader Mode [P] (SET-ED-026)
  - TextMate Bundles (SET-ED-032)
  - TODO (SET-ED-027)
- Plugins (SET-PLG-001)
- Version Control [P] (SET-VCS-001)
  - Changelists † [P] (SET-VCS-003)
  - Commit † [P] (SET-VCS-002)
  - Confirmation † [P] (SET-VCS-004)
  - Directory Mappings † [P] (SET-VCS-005)
  - File Status Colors † (SET-VCS-006)
  - Issue Navigation † [P] (SET-VCS-007)
  - Log [P] (SET-VCS-008)
  - Shelf † [P] (SET-VCS-009)
  - Git [P] (SET-VCS-010)
  - GitHub [P] (SET-VCS-011)
  - GitLab [P] (SET-VCS-012)
  - Mercurial [P], Subversion [P] (SET-VCS-013)
- Build, Execution, Deployment
  - Build Tools [P] (SET-BLD-001) → Gradle (SET-BLD-002), Maven (SET-BLD-003…), Gant
  - Compiler [P] (SET-BLD-010) → Annotation Processors, Compiler Cache, Excludes, Groovy Compiler, Java Compiler, Kotlin Compiler, RMI Compiler, Validation
  - Coverage [P] (SET-BLD-020)
  - Debugger (SET-BLD-030) → Data Views (→ Java, Kotlin, Java Type Renderers), Stepping, HotSwap, Async Stack Traces
  - Deployment †, Docker †, Application Servers †, Required Plugins (commercial; listed in SET-BLD-050)
  - Remote Jar Repositories [P] (SET-BLD-040)
  - Run Targets [P] (SET-BLD-041)
  - Run Widget [P] (SET-BLD-042)
  - Environment Dashboard [P], Clouds, Java Profiler, Kubernetes (SET-BLD-050)
  - Trusted locations moved to Appearance & Behavior (2024+)
- Languages & Frameworks (SET-LANG-*): Java-relevant: Kotlin, JVM Logging, Lombok, JavaFX, Markdown, Schemas and DTDs, Template Data Languages; commercial: Spring, Jakarta EE, SQL Dialects, JavaScript, TypeScript, etc.
- Tools (SET-TOOL-*): Actions on Save, Diff & Merge, External Tools, Server Certificates (moved to System Settings), Shared Indexes †, Startup Tasks, Tasks, Terminal, Web Browsers and Preview, Features Trainer, Features Suggester, MCP Server, LSP Servers, Settings Repository, XPath Viewer, CSV Formats, Database (commercial)
- Backup and Sync (SET-BAK-001)
- Advanced Settings (SET-ADV-001)

Sources:
- [settings-tree.md](settings-tree.md)
- https://www.jetbrains.com/help/idea/settings-preferences-dialog.html (and the "Settings reference" sub-tree of the help)

---

## 3. Appearance & Behavior

### SET-AB-001 Appearance — `preferences.lookFeel` — App

| Option | Control | Default | Notes |
|---|---|---|---|
| Theme: | combobox | Islands Dark (new installs; Islands Light if OS is light and sync is on) | Lists bundled themes (Islands Dark, Islands Light, Dark, Light, Darcula, High Contrast, IntelliJ Light Legacy …) and installed plugin themes under *Custom*; *Get more themes* link opens Plugins. |
| Sync with OS | checkbox | off | Gear button next to it opens *Preferred Theme and Editor Color Scheme* (light/dark theme + scheme pairs) [bundle]. |
| Different tool window background | checkbox | on | Islands themes only [doc]; lighter background in dark theme, darker in light theme [bundle]. |
| Editor color scheme: | combobox + *Edit…* link | scheme matching the theme | Same list as Editor › Color Scheme. |
| **Accessibility** group | | | |
| Zoom: | combobox (percent) | 100% | Comment shows zoom shortcuts [bundle]; *Reset to default* link. |
| Use custom font: | checkbox + font combobox + Size combobox | off; system UI font, 13 | *Reset to default* link. |
| Support screen readers | checkbox | off | Requires restart; Tab/Shift+Tab then navigate controls [bundle]. |
| Use contrast scrollbars | checkbox | off | |
| Adjust colors for red-green vision deficiency | checkbox | off | *How it works* link; requires restart. |
| Use simplified splash screen | checkbox | off | Listed in help [doc]; verify presence. |
| **UI Options** group | | | |
| Compact mode | checkbox | off | Reduces toolbar/tab heights. |
| Always show full path in window header | checkbox | off | |
| Use project colors in main toolbar | checkbox | on | Colored project widget gradient [bundle text]. |
| Drag-and-drop with Alt pressed only | checkbox | off [doc] | |
| Smooth scrolling | checkbox | on [doc] (Windows/Linux) | Help says animated smooth scrolling dialog reached via Find Action (duration, curve) [bundle]. |
| Enable mnemonics in controls | checkbox | on | Windows/Linux. |
| Enable mnemonics in menu | checkbox | on | Windows/Linux. |
| Display icons in menu items | checkbox | on | |
| Main menu: | combobox | Hide under Hamburger Button? | Windows/Linux. Values: *Hide under Hamburger Button*, *Merge with Main Toolbar*, *Show above Main Toolbar* [doc labels]. See also advanced `merge.main.menu` = true [adv]. |
| Keep popups open for toggle items | checkbox | off | |
| Background Image… | button | — | Opens image/opacity/fill/anchor dialog for editor and frame backgrounds. |
| Custom application icon | checkbox | off | macOS only [bundle]. |
| **Tree Views** group | | | |
| Show indent guides | checkbox | off | |
| Use smaller indents | checkbox | off | |
| Expand node with single click | checkbox | off | "Some trees may ignore this setting" [bundle]. |
| **Tool Windows** group | | | |
| Show tool window bars | checkbox | on | |
| Show tool window names | checkbox | off | New UI shows icons only by default. |
| Widescreen tool window layout | checkbox | off | |
| Side-by-side layout on the left | checkbox | off | |
| Side-by-side layout on the right | checkbox | off | |
| Remember size for each tool window | checkbox | on | |
| Use transparent mode for floating windows | checkbox + Ratio spinner + Delay (ms) | off | Legacy group *Transparency*; may be hidden in New UI (inferred). |
| **Presentation Mode** group | | | |
| Zoom: | combobox | 175% | Applies only in Presentation Mode. |
| **Antialiasing** group | | | |
| IDE: | combobox: Subpixel / Greyscale / No antialiasing | Subpixel (Win/Linux), Greyscale (macOS) | |
| Editor: | combobox, same values | as IDE | |

### SET-AB-002 Menus and Toolbars — `preferences.customizations` — App

| Element | Description |
|---|---|
| Group list | Left list of customizable roots: Main Menu, Main Toolbar (Left / Center / Right sections), Main Toolbar Run Widget?, Navigation Bar Toolbar, Editor Popup Menu, Editor Gutter Popup Menu, Editor Tab Popup Menu, Project View Popup Menu, Scope View Popup Menu, Navigation Bar Popup Menu, Debug Tool Window Top Toolbar, Touch Bar (macOS) [bundle names partially]. Selecting a root shows its action tree. |
| Action tree | Actions, separators and sub-groups with icons; reorder by drag or Move Up/Down. |
| Toolbar | **+** Add Action… (opens action chooser tree with search) / Add Separator; **−** Remove; Edit Action Icon…; Move Up; Move Down; Restore Selected Groups / Restore Defaults / *Restore All Defaults* [bundle]. |
| Persistence | Stored as diffs against the default layout, application-level. |

### SET-AB-003 System Settings — `preferences.general` — App (contains project-related options)

| Option | Control | Default | Notes |
|---|---|---|---|
| Confirm before exiting the IDE | checkbox | on | |
| When closing a tool window with a running process: | radio: Terminate process / Disconnect / Ask | Ask | [bundle] labels |
| **Project** group | | | |
| Reopen projects on startup | checkbox | on | |
| Open project in | radio: New window / Current window / Ask | Ask | Labels per bundle: *New window*, *Current window*, *Ask*; macOS can also merge windows into tabs [doc]. |
| Default project directory: | path | `~/IdeaProjects` | Preselected in Open and New Project dialogs [bundle]. |
| **Files** group (*Files*) | | | |
| Move files to the bin instead of deleting permanently | checkbox | on | |
| Save files if the IDE is idle for … seconds | checkbox + spinner | on, 15 | |
| Save files when switching to a different application or a built-in terminal | checkbox | on | Comment: *Autosave cannot be disabled completely. How it works* [bundle]. |
| Back up files before saving | checkbox | on | "Safe write" [doc]. |
| Sync external changes: | group | | |
| ↳ When switching to the IDE window or opening an editor tab | checkbox | on | |
| ↳ Periodically when the IDE is inactive (experimental) | checkbox | off | 15 s of inactivity [doc]. |

Subpages:

#### SET-AB-004 Data Sharing — `consents` — App
| Option | Control | Default |
|---|---|---|
| Send usage statistics | checkbox | off (asked on first run; inferred) |
| Share detailed code-related data (AI) | checkbox | off (inferred; label unverified) |

#### SET-AB-005 Date Formats — `ide.date.format` — App
| Option | Control | Default |
|---|---|---|
| Override system date and time format | checkbox [bundle] | off |
| ↳ Date format: | text (pattern) with *Date patterns* help link [bundle] | system |
| ↳ Use 24-hour time | checkbox | system |
| Use pretty formatting (Today, Yesterday) | checkbox | on |

#### SET-AB-006 HTTP Proxy — `http.proxy` — App
| Option | Control | Default |
|---|---|---|
| No proxy / Auto-detect proxy settings / Manual proxy configuration | radio | No proxy (inferred; may be Auto-detect on some platforms) |
| ↳ Automatic proxy configuration URL: | checkbox + text | off |
| ↳ Clear passwords | button | — |
| ↳ HTTP / SOCKS | radio | HTTP |
| ↳ Host name: / Port number: | text / spinner | — / 80 |
| ↳ No proxy for: | text (comma-separated hosts, wildcards) | — |
| ↳ Proxy authentication: Login / Password / Remember | checkbox + text fields | off |
| Check connection | link/button | — |

#### SET-AB-007 Language and Region — `preferences.language.and.region` — App
| Option | Control | Default |
|---|---|---|
| Language: | combobox (installed language packs, English) [bundle] | English |
| Region: | combobox: Not specified, Africa, Americas, Asia (except China Mainland), China Mainland, Europe, Middle East, Oceania [bundle] | Not specified |

#### SET-AB-008 Passwords — `application.passwordSafe` — App
| Option | Control | Default |
|---|---|---|
| In native Keychain / In KeePass / Do not save, forget passwords after restart | radio | native Keychain (macOS/Linux with libsecret; Windows uses KeePass-backed store) |
| ↳ Database: | path (KeePass `c.kdbx`) | config dir |
| ↳ Master password actions (Set Master Password, Import, Clear) | gear menu | — |

#### SET-AB-009 Process Elevation — `application.elevation` — App (Windows)
| Option | Control | Default |
|---|---|---|
| Show the confirmation dialog before elevating permissions / grant time | checkbox / spinner | on (labels unverified) |

#### SET-AB-010 Server Certificates — `http.certificates` — App
| Option | Control | Default |
|---|---|---|
| Accept non-trusted certificates automatically | checkbox | off |
| Certificate list (Issued to, Issued by, Valid until) with + Add / − Remove and details pane | table | empty |

#### SET-AB-011 Updates — `preferences.updates` — App
| Option | Control | Default |
|---|---|---|
| Check IDE updates for: | checkbox + combobox: Stable Releases / Beta Releases or Public Previews / Early Access Program | on, Stable Releases |
| Check for plugin updates | checkbox | on |
| Show What's New in the editor after an IDE update | checkbox | on |
| Check for Updates | button; shows *Last checked:* and current build | — |
| Manage ignored updates | link | — |
| (Toolbox installs) | page shows that updates are managed by the Toolbox App | — |

### SET-AB-012 File Colors — `reference.settings.ide.settings.file-colors` — Proj

| Option | Control | Default |
|---|---|---|
| Enable file colors | checkbox [bundle] | on |
| ↳ Use in editor tabs | checkbox | on |
| ↳ Use in project view | checkbox | on |
| Local colors (stored in workspace) | table: Scope, Color | empty |
| Shared colors (stored in `.idea/fileColors.xml`) | table: Scope, Color | `Non-Project Files` (Yellow), `Tests` (Green) — inferred |
| Manage scopes… | link | — |

### SET-AB-013 Scopes — `project.scopes` — Proj

| Element | Description |
|---|---|
| Scope list | Shared (`.idea/scopes/`) and Local scopes; + Add Scope (Local / Shared), − Remove, Copy, Move Up/Down, Save As. |
| Name / Pattern: | text; pattern syntax `file:`, `src:`, `lib:`, `test:` with `*`, `**`, `&&`, `||`, `!`. |
| Project tree | Chooser tree of project/libraries with buttons **Include**, **Include Recursively**, **Exclude**, **Exclude Recursively** which edit the pattern. Tree filter by view (Project / Packages). |
| Status | "Scope contains N of total M files" [bundle]. |
| Predefined scopes | Project Files, Production, Tests, Changed Files, Open Files … are not editable here. |

### SET-AB-014 Notifications — `reference.settings.ide.settings.notifications` — App

| Option | Control | Default |
|---|---|---|
| Display balloon notifications | checkbox [bundle] | on |
| Enable system notifications | checkbox | on |
| ↳ (macOS) Show for: all / only when IDE is inactive | radio (inferred) | ? |
| Enable sound for notifications (inferred) | checkbox | off |
| Group table: Group, Popup type, Log, Read aloud (macOS) | table; Popup type values: Balloon, Sticky balloon, Tool window balloon, No popup | per group |

### SET-AB-015 Quick Lists — `reference.idesettings.quicklists` — App

| Element | Description |
|---|---|
| List | User quick lists; + / − / copy. |
| Display name: / Description: | text [bundle]. |
| Actions | Ordered action list with Add (action chooser), Add Separator, Remove, Move Up/Down. Each quick list becomes an action assignable in Keymap › Quick Lists. |

### SET-AB-016 Required Plugins — `preferences.externalDependencies` — Proj
Table of plugins the project requires (Plugin, Minimum version, Maximum version); +/−. When the project opens without them, the IDE notifies and offers to install/enable.

### SET-AB-017 Trusted Locations — `trusted.hosts` — App
Table of trusted paths (projects inside are opened without the trust dialog); +/−. Option *Trust projects under …* checkbox per path (inferred).

### SET-AB-018 Path Variables — `preferences.pathVariables` — Proj (help lists it as App)
| Element | Control |
|---|---|
| Table: Name, Value | table with Add / Remove / Edit (*Add Variable*, *Edit Variable* dialogs with Name:, Value:) [bundle] |
| Ignored Variables | text (semicolon separated); variables listed here are not reported as undefined |

### SET-AB-019 Presentation Assistant — `PresentationAssistantConfigurable` — App
| Option | Control | Default |
|---|---|---|
| Show action names and shortcuts | checkbox | off |
| Popup size: | combobox Small/Medium/Large (inferred) | Medium |
| Popup duration | spinner (s) | 4 |
| Position: | combobox | Bottom center |
| Show alternative shortcuts for: | checkbox + keymap combobox | on, macOS/Windows counterpart |

### SET-AB-020 Data Editor and Viewer — `database.data.views.appearance` — App
Commercial/database page for the data grid (font, rows paging, value editor). Out of scope for Java + Gradle; options not specified here.

Sources:
- https://www.jetbrains.com/help/idea/settings-appearance.html
- https://www.jetbrains.com/help/idea/customize-actions-menus-and-toolbars.html
- https://www.jetbrains.com/help/idea/system-settings.html
- https://www.jetbrains.com/help/idea/settings-http-proxy.html
- https://www.jetbrains.com/help/idea/settings-file-colors.html
- https://www.jetbrains.com/help/idea/settings-scopes.html
- https://www.jetbrains.com/help/idea/settings-notifications.html
- https://www.jetbrains.com/help/idea/settings-quick-lists.html
- https://www.jetbrains.com/help/idea/settings-path-variables.html
- IntelliJ Community `IdeBundle.properties`, `ProjectConceptBundle.properties` (labels)

---

## 4. Keymap

### SET-KM-001 Keymap — `preferences.keymap` — App

| Element | Behaviour |
|---|---|
| Keymap combobox | Lists predefined keymaps (bold, read-only: Windows, macOS, macOS System Shortcuts, GNOME, KDE, XWin, Emacs, Eclipse, NetBeans, Visual Studio, VS Code, Sublime Text … — full list in [keymaps.md](keymaps.md)) and user keymaps indented under their parent. Default: *Windows* on Windows, *macOS* on macOS, *GNOME*/*KDE* on Linux depending on desktop (inferred). Selecting a keymap applies on Apply. |
| Scheme actions (⋮) | *Duplicate…*, *Rename…*, *Restore Defaults* (custom keymap → parent), *Delete* [doc]. Editing a predefined keymap automatically creates a child copy named "<parent> copy" [bundle]. |
| Get more keymaps | Link *Get more keymaps in Settings \| Plugins* [bundle]. |
| Toolbar | Expand All, Collapse All, Edit Shortcut, *Show conflicts with system shortcuts* (macOS) [bundle], search field, *Find Actions by Shortcut* toggle, *Clear Filtering* [bundle]. |
| Search field | Filters the action tree by action name/description text. |
| Find Actions by Shortcut | Popup *Find Shortcut* with a keystroke recorder field, *Second stroke* checkbox and *Mouse shortcut* pad [bundle]; tree then shows only actions bound to that shortcut. |
| Action tree | Columns *Action* and *Shortcuts* [bundle]. Roots: Main Menu, Editor Actions, Main Toolbar, Editor Popup Menu, … , Version Control Systems, External Tools, Macros, Quick Lists, Plugins (per plugin), Intentions, Other [bundle]. Modified actions are rendered in the accent color (inferred). |
| Edit menu (Enter / double-click / right-click) | **Add Keyboard Shortcut**, **Add Mouse Shortcut**, **Add Abbreviation**, **Remove <shortcut>** (one entry per existing shortcut), **Reset Shortcuts** (if differs from parent). |
| Keyboard Shortcut dialog | Recorder field (captures keystroke incl. modifiers); **Second stroke:** checkbox with second recorder [bundle]; special-keys **+** menu to insert Enter/Tab/Esc etc.; live *Already assigned to:* list of conflicting actions [bundle]. |
| Mouse Shortcut dialog | Pad: "click, double-click, or scroll the wheel, modified with Ctrl, Alt, Shift" [bundle]; shows conflicts. |
| Abbreviation dialog | Text field; abbreviations are matched by Find Action / Search Everywhere. |
| Conflict dialog | On OK of a shortcut that is already assigned: title *Warning*, message "This shortcut is already assigned to other actions. Do you want to remove the other assignments?" — buttons **Remove**, **Keep**, **Cancel** [bundle]. Remove deletes the shortcut from the other actions in the current keymap; Keep leaves both. |
| Ctrl+Y patch | First run on Windows keymap may ask whether Ctrl+Y should be *Redo* or *Delete Line* [bundle]. |
| Other options | *Use national layouts for shortcuts (requires restart)* — default on for macOS/Windows, off on Linux [doc]; *Use US non-alphanumeric keys for shortcuts (requires restart)* — Windows with Cyrillic layout, default off [doc]; *Show F1, F2, etc. keys on the Touch Bar (requires restart)* — default off [doc]. |
| Plugins section | Keymaps contributed by plugins appear in the combobox; plugin actions under *Plugins › <plugin>*. |

Sources:
- https://www.jetbrains.com/help/idea/settings-keymap.html
- https://www.jetbrains.com/help/idea/configuring-keyboard-and-mouse-shortcuts.html
- IntelliJ Community `KeyMapBundle.properties` (labels)

---

## 5. Editor

### SET-ED-001 General — `preferences.editor` — App

| Option | Control | Default | Notes |
|---|---|---|---|
| **Mouse Control** | | | |
| Change font size with Ctrl+Mouse Wheel in: (macOS: Command+Mouse Wheel) | checkbox + radio Active editor / All editors | off; Active editor | [bundle] |
| Move code fragments with drag-and-drop | checkbox | on | Comment: "To copy, hold Ctrl while dragging" [bundle] |
| **Soft Wraps** | | | |
| Soft-wrap these files: | checkbox + text (`;`-separated masks) | on; `*.md; *.txt; *.rst; *.adoc` | |
| Use the original line's indent for wrapped fragments | checkbox | on | |
| ↳ Add additional indent: … symbols | spinner | 0 | |
| Only show soft-wrap indicators for the current line | checkbox | off | |
| **Virtual Space** | | | |
| Allow caret placement: After the end of line | checkbox | off | |
| Allow caret placement: Inside tabs | checkbox | off | |
| Show virtual space at the bottom of the file | checkbox | off | |
| **Scroll Offset** | | | |
| Vertical scroll offset: | spinner (lines) | 1 | |
| Vertical scroll jump: | spinner | 0 | |
| Horizontal scroll offset: | spinner (columns) | 3 | |
| Horizontal scroll jump: | spinner | 0 | |
| **Caret Movement** | | | |
| When moving by words: | combobox: Jump to the current word boundaries / Always jump to the word start / Always jump to the word end / Jump to the next/previous word boundaries / Stop at both word boundaries | Jump to the current word boundaries [doc] | Items show "<OS> default" hints [bundle] |
| Upon line break: | combobox: Ignore line breaks / Jump to the current line boundaries / Jump to the next/previous line boundaries / Always jump to the line start / Always jump to the line end / Stop at both line boundaries | OS-dependent | |
| **Scrolling** | | | |
| Enable smooth scrolling | checkbox | on | |
| Caret behavior: | radio: Keep the caret in place, scroll editor canvas / Move caret, minimize editor scrolling | Move caret, minimize editor scrolling | [bundle] |
| **Rich-Text Copy** | | | |
| Copy (Ctrl+C) as rich text | checkbox | on | "All formatting will be copied…" [bundle] |
| ↳ Color scheme for copied fragment: | combobox: Active scheme + all schemes | Active scheme | |
| **On Save** | | | |
| Remove trailing spaces on: | checkbox + combobox Modified lines / All lines | on, Modified lines | |
| ↳ Keep trailing spaces on caret line | checkbox | on | |
| Remove trailing blank lines at the end of saved files | checkbox | off | |
| Ensure every saved file ends with a line break | checkbox | off | |
| **Gutter** (group in bundle; placement inferred) | | | |
| Highlight modified lines in the gutter | checkbox | on | VCS change markers |
| ↳ Highlight lines with whitespace-only modifications with a different color | checkbox | on | |
| Highlight modified lines in error stripe on the scrollbar | checkbox | on | |

### SET-ED-002 Auto Import — `editor.preferences.import` — Proj (Java part), App (others)

| Option | Control | Default |
|---|---|---|
| **XML**: Show auto-import tooltip | checkbox | on |
| **Java**: Insert imports on paste: | combobox Always / Ask / Never | Ask |
| Show auto-import tooltip for: Classes | checkbox | on [doc] |
| Show auto-import tooltip for: Static methods and fields | checkbox | on [doc] |
| Optimize imports on the fly | checkbox | off |
| Add unambiguous imports on the fly | checkbox | off |
| Exclude from auto-import and completion: | table: Class or package, Scope (Project / IDE) | `com.sun.*`, `sun.*` (inferred) |
| Include auto-import of static members in completion | table of classes/members (prefix `-` excludes) [doc] | empty |
| **Kotlin**: Add unambiguous imports on the fly / Optimize imports on the fly | checkboxes | off / off |
| **JSP**: Add unambiguous imports on the fly | checkbox | off |
| Other languages (JS/TS, PHP, Scala, Python) | plugin-dependent | — |

### SET-ED-003 Appearance — `editor.preferences.appearance` (inferred) — App

| Option | Control | Default |
|---|---|---|
| Caret blinking (ms): | checkbox + spinner | on, 500 |
| Use smooth caret blinking | checkbox | on |
| Use block caret | checkbox | off |
| Use full line height caret | checkbox | off |
| Use smooth caret movement | checkbox + mode: Snappy / Gliding [bundle] | on, Snappy |
| Highlight occurrences of selected text (inferred placement; label in bundle) | checkbox | on |
| Show hard wrap and visual guides (configured in Code Style options) | checkbox | on |
| Show line numbers: | checkbox + combobox Absolute / Relative / Hybrid [bundle] | on, Absolute |
| Show method separators | checkbox | off |
| Show whitespaces | checkbox | off |
| ↳ Leading / Inner / Trailing / Selection | checkboxes | all on |
| Show indent guides | checkbox | on |
| Show intention bulb | checkbox | on |
| Show preview for intention actions when available | checkbox | on |
| Render documentation comments | checkbox | off |
| ↳ Also in Reader mode | checkbox link [bundle] | on |
| Show code lens on scrollbar hover | checkbox | off |
| Use editor font for inlay hints | checkbox | off [doc: hints use UI font by default] |
| Enable HTML/XML tag tree highlighting (Levels to highlight, Opacity) | checkbox + spinners | off; 6; 0.1 |
| Show CSS color preview as background | checkbox (commercial) | off |

### SET-ED-004 Breadcrumbs — `editor.breadcrumbs` — App

| Option | Control | Default |
|---|---|---|
| Show breadcrumbs | checkbox | on |
| Placement: | radio Top / Bottom | Top |
| Languages: | checkbox list per language (Java, Kotlin, Groovy, XML, HTML, JSON, YAML, Markdown, Properties, …) | all on |

### SET-ED-005 Code Completion — `editor.preferences.completion` (inferred) — App

| Option | Control | Default |
|---|---|---|
| **Popup** | | |
| Match case: | checkbox + radio First letter only / All letters | on, First letter only |
| Automatically insert single suggestions for: Basic Completion | checkbox | on |
| ↳ Type-Matching Completion | checkbox | on |
| Sort suggestions alphabetically | checkbox | off |
| Show suggestions as you type | checkbox | on [doc] |
| ↳ Insert selected suggestion by pressing space, dot, or other context-dependent keys | checkbox | off |
| Show the documentation popup in … ms | checkbox + spinner | off, 1000 |
| Insert parentheses automatically when applicable | checkbox | on |
| **Sort Suggestions with ML** [bundle] | | |
| Sort completion suggestions based on machine learning (per language: Java, Kotlin, …) | checkbox tree | on |
| ↳ Mark position changes in the completion popup | checkbox | off |
| ↳ Mark the most relevant item | checkbox | off |
| **Command Completion** | | |
| Enable command completion | checkbox | on |
| ↳ Show command completion as a separate group | checkbox | off |
| ↳ Enable command completion for read-only files | checkbox | on |
| ↳ Show live templates | checkbox [bundle] | on |
| **Java** | | |
| **Completion of names**: Suggest variable and parameter names | checkbox | off [doc] |
| ↳ Suggest names for class fields | checkbox | off [doc] |
| **Parameter Info** | | |
| Show parameter name hints on completion | checkbox | off |
| Show the parameter info popup in … ms | checkbox + spinner | on, 1000 |
| Show full method signatures | checkbox | off |

#### SET-ED-005a Inline Completion — (inferred id `editor.preferences.inline.completion`) — App
| Option | Control | Default |
|---|---|---|
| Enable local Full Line completion suggestions | checkbox | on |
| Languages (Java, Kotlin, …) | checkbox list | on |
| Synchronize suggestions with the completion popup? / show suggestions in debounce mode? | checkboxes (unverified) | ? |
| Info text: *Go to Inline Completion settings page…* links from Code Completion [bundle] | link | — |

### SET-ED-006 Code Folding — `editor.preferences.folding` (inferred) — App

| Option | Control | Default |
|---|---|---|
| Show code folding outline | checkbox + radio Always / On mouse hover [bundle] | on, Always |
| Show code folding arrows / Show bottom arrows | checkbox [bundle] | on / on |
| Fold by default: **General** — File header | checkbox | on |
| ↳ Imports | checkbox | on |
| ↳ Documentation comments | checkbox | off |
| ↳ Method bodies | checkbox | off |
| ↳ Custom folding regions | checkbox | off |
| **Java** — One-line methods | checkbox | on |
| ↳ "Closures" (anonymous classes implementing one method, before Java 8) | checkbox | on |
| ↳ Generic constructor and method parameters | checkbox | on |
| ↳ Simple property accessors | checkbox | off |
| ↳ End of line comments sequence | checkbox | off |
| ↳ Inner classes | checkbox | off |
| ↳ Anonymous classes | checkbox | off |
| ↳ Annotations | checkbox | off |
| ↳ I18n strings | checkbox | on |
| ↳ @SuppressWarnings | checkbox | on |
| ↳ Replace 'var' with inferred type | checkbox | off |
| ↳ Multiline comments | checkbox | off |
| **Kotlin**, **XML**, **Markdown**, **Groovy**, **Properties** … | per-language checkboxes | language defaults |

### SET-ED-007 Console — `Console` — App

| Option | Control | Default |
|---|---|---|
| Override console cycle buffer size (… KB) | checkbox + spinner | off, 1024 [warnings for large/unlimited in bundle] |
| Use soft wraps in console | checkbox | off |
| Default Encoding: | combobox (Favorites / More) | system default (UTF-8 on macOS/Linux) |
| Console commands history size: | spinner | 300 |
| Fold console lines that contain: | table (substrings) | e.g. `at java.base/jdk.internal.reflect`, `at java.lang.reflect.Method.invoke`, `at org.junit.platform…` (list inferred) |
| Exceptions: | table of substrings never folded [bundle] | empty |

### SET-ED-008 Editor Tabs — `editor.preferences.tabs` (inferred) — App

| Option | Control | Default |
|---|---|---|
| **Appearance** | | |
| Tab placement | combobox Top / Bottom / Left / Right / None | Top |
| Show tabs in: One row, and if tabs don't fit: | radio + sub-radio Scroll the tabs panel / Squeeze tabs | One row, Scroll |
| ↳ Multiple rows | radio | — |
| Show pinned tabs in a separate row | checkbox | off |
| Show file icon | checkbox | on |
| Show file extension | checkbox | on |
| Show directory for non-unique file names | checkbox | on |
| Mark modified (*) | checkbox | off (New UI shows a dot instead, inferred) |
| Show full path on mouse hover | checkbox | on |
| Use small font for labels | checkbox | off |
| Close button position: | combobox Right / Left / None (inferred values) | Right |
| **Opening Policy** | | |
| Enable preview tab | checkbox | off |
| Open declaration source in the same tab / other tab behaviour | see Advanced Settings (`editor.reuse.not.modified.tabs` false [adv]) | |
| **Tab Order** | | |
| Sort tabs alphabetically | checkbox | off |
| Open new tabs at the end | checkbox | on |
| **Closing Policy** | | |
| Tab limit: | spinner | 10 |
| When tabs exceed the limit: | radio Close unchanged / Close unused | Close unchanged |
| When the current tab is closed, activate: | radio Most recently opened tab / The tab on the left / The tab on the right | Most recently opened tab |

### SET-ED-009 Gutter Icons — `editor.preferences.gutterIcons` (inferred) — App

| Option | Control | Default |
|---|---|---|
| Show gutter icons | checkbox | on |
| Per-icon tree grouped by area (Common, Java, Kotlin, Gradle, Maven, Spring, …) | checkbox tree | all on |
| Common items (inferred list): Color preview [bundle], Image preview, Documentation comments in-place rendering [bundle], Breakpoint-related, Bookmarks | | on |
| Java items (inferred list): Implemented method, Implementing method, Overridden method, Overriding method, Recursive call, Run (main / test), Lambda, Sibling inheritor, Related problems | | on |

### SET-ED-010 Postfix Completion — `reference.settingsdialog.IDE.editor.postfix.templates` (inferred) — App

| Option | Control | Default |
|---|---|---|
| Enable postfix completion | checkbox [bundle] | on |
| ↳ Show postfix templates in completion autopopup | checkbox | on |
| ↳ Show postfix completions as command completions | checkbox [bundle] | on? |
| Expand templates with | combobox Tab / Enter / Space | Tab |
| Template tree by language (Java: `.arg .cast .castvar .else .field .for .fori .forr .format .if .inst .instanceof .iter .lambda .new .nn .not .notnull .null .opt .par .reqnonnull .return .serr .sout .souf .soutv .stream .switch .synchronized .throw .try .twr .var .while .yield` …; Kotlin, Groovy, …) | checkbox tree; + (custom, with Key:, Applicable expression types: [bundle]), −, Edit, Duplicate | all on |
| Description / Before: / After: preview | read-only | — |

### SET-ED-011 Smart Keys — `editor.preferences.smartKeys` (inferred) — App

| Option | Control | Default |
|---|---|---|
| Home moves caret to first non-whitespace character | checkbox | on |
| End on blank line moves caret to indent position | checkbox | on |
| Insert paired brackets (), [], {}, <> | checkbox | on |
| Insert pair quote | checkbox | on |
| Reformat block on typing '}' | checkbox | on |
| Use "CamelHumps" words | checkbox | off |
| ↳ Honor "CamelHumps" words settings when selecting on double click | checkbox | on |
| Surround selection on typing quote or brace | checkbox | on |
| Add multiple carets on double Ctrl with arrow keys (macOS: ⌥) | checkbox | on |
| Jump outside closing bracket/quote with Tab when typing | checkbox | off |
| **Enter**: Smart indent | checkbox | on |
| ↳ Insert pair '}' | checkbox | on |
| ↳ Close block comment | checkbox | on |
| ↳ Insert documentation comment stub | checkbox | on |
| Unindent on Backspace: | combobox Disabled / To nearest indent position / To proper indent position | To proper indent position |
| Reformat on paste: | combobox None / Indent block / Indent each line / Reformat block | Reformat block |
| **JavaDoc**: Automatically insert closing tag in JavaDoc | checkbox | on |
| **Kotlin**: Convert pasted Java code to Kotlin | checkbox | on |
| ↳ Don't show Java to Kotlin conversion dialog on paste | checkbox | off |
| **JSP**: Insert pair '%>' on Enter in JSP | checkbox | on |
| Subpage **HTML/CSS** (and JavaScript in commercial) | separate pages | — |

### SET-ED-012 Sticky Lines — `editor.stickyLines` — App

| Option | Control | Default |
|---|---|---|
| Show sticky lines while scrolling | checkbox | on |
| Maximum number of lines: | spinner | 5 |
| Languages: | checkbox list | all on |

### SET-ED-013 Code Editing — `preferences.editor.code.editing` — App

| Option | Control | Default |
|---|---|---|
| **Quick Documentation**: Show quick documentation on hover | checkbox [bundle] | on |
| ↳ Tooltip delay: … ms | spinner | 500 |
| **Editor Tooltips**: Tooltip delay | spinner | 500 |
| **Highlight on Caret Movement**: Matched brace | checkbox | on |
| ↳ Current scope | checkbox | off |
| ↳ Usages of element at caret | checkbox | on |
| **Error Highlighting**: Autoreparse delay (ms): | spinner | 300 |
| ↳ The 'Next Error' action goes through: | combobox The problems with the highest priority / All problems | The problems with the highest priority |
| **Refactorings**: Specify refactoring options: | radio In the editor / In modal dialogs | In the editor |
| ↳ Preselect current symbol name for Rename refactoring | checkbox | on |
| ↳ Show inline dialog for local variables | checkbox | on |
| **Formatting**: Show notification after reformat code action | checkbox | on |
| ↳ Show notification after optimize imports action | checkbox | on |
| **Code Assistance** (inferred group name) | | |

### SET-ED-014 Font — `editor.preferences.fonts.default` (inferred; parent id from settings-tree.md) — App

| Option | Control | Default |
|---|---|---|
| Font: | combobox | JetBrains Mono |
| ↳ Show only monospaced fonts | checkbox | on |
| Size: | spinner | 13 |
| Line height: | spinner | 1.2 [doc] |
| Enable ligatures | checkbox | off |
| Restore Defaults | link (only when changed) [doc] | — |
| **Typography Settings**: Main weight: / Bold weight: | combobox | Regular / Bold |
| Fallback font: | combobox | none |
| Character variants: | checkbox list of OpenType features (Slashed Zero, Contextual Ligatures, Stylistic Set N, Character Variant N, …) [bundle] | font defaults |
| Preview | editable sample text | — |

#### SET-ED-014a Console Font — `AppConsoleFontConfigurable$Provider` — App
| Option | Control | Default |
|---|---|---|
| Use console font instead of the default | checkbox | off |
| Font / Size / Line height / ligatures | as Font page | inherits editor font |

### SET-ED-015 Color Scheme — `reference.settingsdialog.IDE.editor.colors` — App

| Element | Behaviour |
|---|---|
| Scheme: | combobox (bundled schemes matching themes, plus custom); gear menu: *Duplicate…*, *Rename…*, *Restore Defaults*, *Delete…*, *Import Scheme…* (`.icls`, `.jar`, Eclipse XML), *Export* (IntelliJ IDEA color scheme (.icls) / Color scheme plugin .jar) [bundle]. |
| Change IDE Theme… | link [bundle] |
| Subpages | General, Language Defaults, Semantic highlighting (inferred placement), Code With Me?, Console Colors, Debugger, Diff & Merge, VCS (File Status), By Scope, Hierarchy?, Java, Kotlin, Groovy, Properties, XML, HTML, JSON, YAML, Markdown, RegExp, Gradle?, Shell Script, SQL, … plus *Color Scheme Font* and *Console Font*. |
| Element page layout | Tree of attribute keys; right panel checkboxes **Bold**, **Italic**, **Foreground**, **Background**, **Error stripe mark**, **Effects** (+ combobox Bordered / Underscored / Bold Underscored / Underwaved / Strikeout / Dotted Line) [bundle]; *Inherit values from:* link to parent key [bundle]; preview editor with click-to-select. |
| Semantic highlighting | Checkbox plus Color #1–#5 stops [bundle] (per language). |
| Color Scheme Font | *Use color scheme font instead of the default* + font fields. |

### Code Style pages

### SET-CS-001 Code Style — `preferences.sourceCode` — Proj* (scheme level selectable)

| Element | Control | Default |
|---|---|---|
| Scheme: | combobox: *Default* (IDE) / *Project* | Project for imported projects with `.idea/codeStyles/`, otherwise Default |
| Gear menu | *Copy to Project…*, *Copy to IDE…*, *Duplicate…*, *Rename…*, *Restore Defaults*, *Delete…*, *Export* (IntelliJ IDEA code style XML), *Import Scheme* (IntelliJ XML, Eclipse XML, Checkstyle) [bundle] | — |
| **General** tab: Line separator: | combobox System-Dependent / Unix and macOS (\n) / Windows (\r\n) / Classic Mac OS (\r) — "Applied to new files" [bundle] | System-Dependent |
| Hard wrap at: … columns | spinner | 120 |
| Wrap on typing | checkbox | off |
| Visual guides: | text ("Specify one guide (80) or several (80, 120)") [bundle] | empty |
| Indents Detection: Detect and use existing file indents for editing | checkbox [bundle] | on |
| **Formatter** tab: Do not format: | table of glob patterns / scope | empty |
| ↳ Do not optimize imports: | table of patterns | empty |
| ↳ Turn formatter on/off with markers in code comments | checkbox [bundle] + Off: / On: text | off; `@formatter:off` / `@formatter:on` |
| ↳ Enable regular expressions in formatter markers | checkbox | off |
| ↳ Keep simple blocks… (n/a) | | |
| ↳ Enable EditorConfig support | checkbox | on |
| Language subpages | Java, Kotlin, Groovy, Gradle (inherits Groovy/Kotlin DSL), XML, HTML, JSON, YAML, Properties, Markdown, Shell Script, SQL, Protocol Buffers, Other File Types ("Text files and unsupported file types") [bundle] | — |
| Per-language header | *Set from…* link (copy from another language or predefined style, e.g. Java → Google Java Style / Android); preview pane on the right that updates live. | — |

### SET-CS-010 Code Style › Java — `preferences.sourceCode.Java` (inferred) — Proj*

Tabs: **Tabs and Indents**, **Spaces**, **Wrapping and Braces**, **Blank Lines**, **JavaDoc**,
**Imports**, **Arrangement**, **Code Generation**. Key defaults below (IntelliJ default scheme).

#### Tabs and Indents
| Option | Control | Default |
|---|---|---|
| Use tab character | checkbox | off |
| ↳ Smart tabs | checkbox | off |
| Tab size: | spinner | 4 |
| Indent: | spinner | 4 |
| Continuation indent: | spinner | 8 |
| Keep indents on empty lines | checkbox | off |
| Label indent: | spinner | 0 |
| Absolute label indent | checkbox | off |
| Do not indent top level class members | checkbox | off |
| Use indents relative to expression start | checkbox | off |

#### Spaces (tree of checkboxes)
| Group | Items (default) |
|---|---|
| Before parentheses | Method declaration parentheses (off), Method call parentheses (off), 'if' / 'for' / 'while' / 'switch' / 'try' / 'catch' / 'synchronized' parentheses (on), Annotation parameters (off), Record header? (off) |
| Around operators | Assignment, Logical, Equality, Relational, Bitwise, Additive, Multiplicative, Shift (all on); Unary operators (off); Lambda arrow (on); Method reference double colon (off) |
| Before left brace | Class, Method, 'if', 'else', 'for', 'while', 'do', 'switch', 'try', 'catch', 'finally', 'synchronized' left brace (on); Array initializer left brace (off); Annotation array initializer left brace (off) |
| Before keywords | 'else', 'while', 'catch', 'finally' (on) |
| Within | Code braces, Brackets, Array initializer braces, Empty array initializer braces, Grouping / Method declaration / Empty method declaration / Method call / Empty method call / 'if' / 'for' / 'while' / 'switch' / 'try' / 'catch' / 'synchronized' / Type cast / Annotation parentheses, Angle brackets, Record header (all off) |
| In ternary operator (?:) | Before '?', After '?', Before ':', After ':' (on); Between '?' and ':' (off) |
| Type arguments | After comma (on); Before opening angle bracket (off) |
| Type parameters | Before opening angle bracket (off); Around type bounds (on) |
| Other | Before comma (off), After comma (on), Before 'for' semicolon (off), After 'for' semicolon (on), After type cast (on), Before colon in foreach (on), Inside one-line enum braces (off) |

#### Wrapping and Braces (selected)
| Option | Control | Default |
|---|---|---|
| Hard wrap at: / Wrap on typing | spinner / combobox | Default (120) / Default (no) |
| Visual guides | text | Default |
| Keep when reformatting: Line breaks / Comment at first column / Control statement in one line | checkboxes | on / on / on |
| ↳ Multiple expressions in one line / Simple blocks / Simple methods / Simple lambdas / Simple classes in one line | checkboxes | off |
| Ensure right margin is not exceeded | checkbox | off |
| Class / Method / Field annotations | combobox (Do not wrap / Wrap if long / Chop down if long / Wrap always) | Wrap always |
| Parameter / Local variable / Enum field annotations | combobox | Do not wrap |
| Braces placement: In class declaration / In method declaration / Other | combobox End of line / Next line if wrapped / Next line / Next line shifted / Next line each shifted | End of line |
| Extends/implements/permits list, Throws list | combobox | Do not wrap |
| Method declaration parameters | combobox + Align when multiline (on), New line after '(' (off), Place ')' on new line (off) | Do not wrap |
| Method call arguments | combobox + Align when multiline (off), Take priority over call chain wrapping (off) | Do not wrap |
| Chained method calls | combobox + Align when multiline (off), Wrap first call (off), Builder methods: text | Do not wrap |
| 'if()' statement | Force braces (Do not force / When multiline / Always), 'else' on new line (off), Special 'else if' treatment (on) | Do not force |
| 'for()', 'while()', 'do … while()' | Force braces | Do not force |
| 'switch' statement | Indent 'case' branches (on) | on |
| 'try' / 'try-with-resources' | 'catch' on new line (off), 'finally' on new line (off), resources wrap (Do not wrap) | |
| Binary / Assignment / Ternary expressions | combobox + Align when multiline + Operation sign on next line | Do not wrap; align off |
| Array initializer | combobox + Align when multiline (on) | Do not wrap |
| Lambda / Enum constants / Record components / Text blocks / Modifier list | wrapping comboboxes | Do not wrap / Do not wrap / Do not wrap |

#### Blank Lines
| Option | Default |
|---|---|
| Keep maximum blank lines: In declarations: / In code: / Before '}': / Between header and package: | 2 / 2 / 2 / 2 |
| Minimum blank lines: Before package statement: | 0 |
| After package statement: / Before imports: / After imports: | 1 / 1 / 1 |
| Around class: / After class header: / Before class end: / After anonymous class header: | 1 / 0 / 0 / 0 |
| Around field in interface: / Around field: | 0 / 0 |
| Around method in interface: / Around method: | 1 / 1 |
| Before method body: / Around initializer: | 0 / 1 |

#### JavaDoc
| Option | Default |
|---|---|
| Enable JavaDoc formatting | on |
| Alignment: Align parameter descriptions / Align thrown exception descriptions | on / on |
| Blank lines: After description / After parameter descriptions / After return tag | on / off / off |
| Invalid tags: Keep invalid tags / Keep empty @param tags / Keep empty @return tags / Keep empty @throws tags | on / on / on / on |
| Other: Enable leading asterisks / Use @throws rather than @exception / Wrap at right margin / Generate "<p>" on empty lines / Keep empty lines / Do not wrap one line comments / Preserve line feeds / Parameter descriptions on new line / Indent continuation lines | on / on / on / on / on / off / off / off / off |
| Markdown documentation comments (Java 23+): Prefer documentation line comments [bundle] | off |

#### Imports
| Option | Control | Default |
|---|---|---|
| Use single class import | checkbox | on |
| Use fully qualified class names | checkbox | off |
| Use fully qualified class names in JavaDoc | radio Always / If not already imported / Never (inferred values) | If not already imported? |
| Insert imports for inner classes | checkbox | off |
| Class count to use import with '*': | spinner | 5 |
| Names count to use static import with '*': | spinner | 3 |
| Packages to Use Import with '*': | table: Static, Package, With Subpackages | `java.awt.*`, `javax.swing.*` |
| Import Layout: | table (import static all other imports / blank line / import all other imports, module imports) | static all other, blank, all other |
| ↳ Layout static imports separately | checkbox | on |
| Module imports (Java 25): Insert module imports? (inferred) | checkbox | off |

#### Arrangement
| Option | Control |
|---|---|
| Grouping rules: Keep getters and setters together / Keep overridden methods together (order) / Keep dependent methods together (Depth-first / Breadth-first) | checkboxes + combos [bundle "Grouping rules:"] |
| Matching rules: | ordered rule list (Add rule, Add section rule, Remove, Edit, Move up/down) [bundle]; default order: static final fields, static fields, final fields, fields, initializers, constructors, methods, enums, interfaces, static classes, classes (inferred) |
| Force rearrange: | combobox Always / Never / Use the current mode (toggled in the Reformat Code dialog) [bundle] |
| Configure matching rules aliases | link [bundle] |

#### Code Generation
| Option | Control | Default |
|---|---|---|
| Naming: Field: / Static field: / Parameter: / Local variable: / Subclass: / Test class: | table Name prefix / Name suffix [bundle] | empty; Test class suffix `Test`; Subclass suffix `Impl` |
| Prefer longer names | checkbox | on |
| Final modifier: Make generated local variables final / Make generated parameters final | checkbox | off / off |
| Default visibility: (Escalate / As is / private / package-private / protected / public) | combobox | Escalate |
| Use 'var' for local variable declaration? (inferred) | | |
| Comment Code: Line comment at first column / Add a space at line comment start / Enforce on reformat / Block comment at first column / Add spaces around block comments | checkboxes | on / off / off / on / off |
| Override Method Signature: Insert @Override annotation / Use external annotations / Repeat synchronized modifier | checkboxes | on / off / on |
| Lambda Body: Use Class::isInstance and Class::cast when possible / Use Integer::sum, etc. when possible / Replace null-check with Objects::nonNull or Objects::isNull | checkboxes | on / on / on |
| Put generated annotations with target TYPE_USE between the modifiers and the type | checkbox [bundle] | on |
| Order of Members (Static fields, Instance fields, Constructors, Static methods, Instance methods, Static inner classes, Inner classes) | ordered list [bundle] | as listed |

### Remaining Editor pages

### SET-ED-016 Inspections — `Errors` — Proj*

| Element | Behaviour |
|---|---|
| Profile: | combobox: *Default* (IDE, stored in config) and *Project Default* (stored `.idea/inspectionProfiles/`) — default *Project Default* for new projects. Gear: Copy to Project / Copy to IDE, Duplicate, Rename, Description…, Restore Defaults, Export (IntelliJ IDEA inspection profile XML) [bundle], Import, Delete. |
| Search / filter | Text search; filter button: Show only enabled, Show only modified, New in 2026.x, by Severity, by Language. |
| Tree | Language → group → inspection, tri-state checkboxes; modified items highlighted (accent color); *Reset to default* per node. |
| Right pane | Description (HTML); **Scope** row (*Everywhere else* + added scopes with own severity); **Severity:** combobox Error / Warning / Weak Warning / Server Problem / Grammar Error / Typo / Consideration / No highlighting (fix only) + *Edit Severities…*; **Highlighting in editor:** combobox (Error, Warning, Weak Warning, Strikethrough, Not used symbol, Generic Server Error…, Text attributes); **Options** panel specific to the inspection. |
| Context menu | Add/Remove scope, Reset to default, Enable/Disable, Severity. |

### SET-ED-017 File and Code Templates — `fileTemplates` — Proj* (scheme Default / Project)

| Element | Behaviour |
|---|---|
| Scheme: | combobox Default / Project |
| Tabs | **Files**, **Includes** (e.g. *File Header*), **Code** (e.g. *New Method Body*, *Catch Statement Body*, *JUnit5 Test Method*), **Other** (framework templates) |
| Toolbar | + Create Template, + Create Child Template File, − Remove, Copy, Reset to Default |
| Template editor | Name:, Extension:, File name: (optional, supports variables), body (Velocity syntax, highlighted), *Reformat according to style* (on), *Enable Live Templates* (off), *Create child files* (off); Description pane listing predefined variables (`${PACKAGE_NAME}`, `${USER}`, `${DATE}`, `${NAME}` …). |

### SET-ED-018 File Encodings — `File.Encoding` — Proj

| Option | Control | Default |
|---|---|---|
| Global Encoding: | combobox | UTF-8 |
| Project Encoding: | combobox | UTF-8 |
| Path / Encoding table | table (per file/dir override) | empty |
| Default encoding for properties files: | combobox [bundle] | Default (ISO-8859-1? for Java <9 semantics) |
| ↳ Transparent native-to-ascii conversion | checkbox [bundle] | off |
| Create UTF-8 files: | combobox with NO BOM / with BOM / with BOM under Windows, with no BOM otherwise | with NO BOM |

### SET-ED-019 Live Templates — `editing.templates` — App (groups may be shared via settings)

| Element | Behaviour |
|---|---|
| By default expand with | combobox Tab / Enter / Space [bundle] — default Tab |
| Template tree | Groups (Java, Kotlin, Groovy, iterations, output, plain, surround, JUnit, Maven, XML, …) with checkboxes; + Live Template / Template Group…, − Remove, Copy, Restore deleted defaults, Change context (multi-select) |
| Template editor | Abbreviation:, Description:, Template text: (with `$VAR$`), *Edit Variables…* (table Name / Expression / Default value / Skip if defined), Options: *Expand with* (Default (Tab) / Tab / Enter / Space / None), *Reformat according to style* (off), *Use static import if possible* (off), *Shorten FQ names* (on); *Applicable in …* contexts with *Change* link (context tree: Java › Statement/Expression/Declaration/Comment/String/Smart type completion/…). |

### SET-ED-020 File Types — `preferences.fileTypes` — App (project-level overrides via *Override File Type*)

| Element | Behaviour |
|---|---|
| Project-level overrides (inferred) | Notice listing files overridden via the *Override File Type* action |
| Recognized File Types | list (with icons); +/−/edit (custom file type: name, description, syntax highlighting keywords, comments, braces) |
| File name patterns | table for selected type (+/−/edit; `*.java`, …); conflict dialog when a pattern belongs to another type |
| Hashbang patterns | table (e.g. `bash`) |
| Ignored Files and Folders | table/text; default `*.pyc;*.pyo;*.rbc;*.yarb;*~;.DS_Store;.git;.hg;.svn;CVS;__pycache__;_svn;vssver.scc;vssver2.scc;` (inferred) |
| Associate File Types with IntelliJ IDEA… | button (Windows/Linux) |

### SET-ED-021 Copyright — `copyright` — Proj

| Element | Behaviour |
|---|---|
| Default project copyright: | combobox (No copyright / profiles) — default No copyright |
| Scope table | Scope → Copyright profile; +/−/up/down |
| Subpage **Copyright Profiles** | profile list; Name, text (Velocity, `$today.year`, `$file.fileName`), *Keyword to detect copyright in comments:*, *Allow replacing copyright if old copyright contains:*, *Validate* button |
| Subpage **Formatting** (+ per language) | Comment type: Use block comment / Use line comment; Prefix each line; Relative location: Before other comments / After other comments; Add blank line after; Borders: Separator before/after, length, Add border; per-language *Use default settings* / *No copyright* |

### SET-ED-022 Inlay Hints — `inlay.hints` — Proj (most options application-wide)

| Element | Behaviour |
|---|---|
| Tree | Groups [bundle]: **Code vision**, **Types**, **Values**, **Parameter names**, **Annotations**, **Method chains**, **Lambdas**, **Code author**, **URL path**, **Other**; each group lists language-specific providers with checkboxes; selecting a provider shows description, preview editor and options. |
| Code vision (Java) | Usages, Inheritors, Related problems, Code author (VCS), JVM … each checkbox; position combobox (Above the declaration / Right / Default); *Default position for metrics*; *Show a Code Vision inlay when no results are available* [bundle]. Advanced: minimum usages `code.vision.java.minimal.usages` = 0 [adv]. |
| Parameter names (Java) | Show parameter hints (on); options: Non-literal arguments, Methods with same named numbered parameters, Builder-like methods (inferred names); *Exclude list…* dialog (patterns like `java.lang.Math.*`) [bundle]. |
| Method chains (Java) | Call chains with *Minimum number of unique types to show hints:* spinner (default 2, inferred) [bundle]. |
| Annotations (Java) | External annotations, Inferred annotations, *Shorten non-null and nullable annotations to !- and ?-suffix respectively* [bundle]. |
| Types | Kotlin property/local/function return types, Java implicit `var` types (off) |
| Values | Kotlin ranges, Java? |
| Lambdas | Kotlin lambda return / implicit receivers |
| Global toggle | *Toggle Inlay Hints Globally* action; disabled-language warning "Inlay hints for 'X' are disabled. Configure settings." [bundle] |

### SET-ED-023 Intentions — `preferences.intentionPowerPack` — App
Tree Language → category → intention with checkboxes; search field; right pane shows description plus *Before* / *After* samples. Disabling an intention also hides it from Alt+Enter.

### SET-ED-024 Language Injections — `IntelliLang.Configuration` — Proj (entries IDE or Project scope)

| Element | Behaviour |
|---|---|
| Table | Name, Language, Type (Java Parameter, Java Method Result?, XML Tag, XML Attribute, Generic Kotlin/Java/Groovy…), Scope (IDE / Project); checkboxes to enable; +/−/edit/copy/move to project/IDE, Import, Export. |
| Advanced section | Instrumentation: None / Assertions / Exceptions; *Language annotation class:* `org.intellij.lang.annotations.Language`; *Pattern annotation class:*; *Substitution annotation class:*; Analysis: Do not analyze / Conservative / Advanced (performs data flow). Defaults: Assertions?; Language annotation as listed. |

### SET-ED-025 Natural Languages — `proofread` — App

| Element | Behaviour |
|---|---|
| Languages list | English (USA) default; + adds language (downloads model); − removes |
| **Grammar and Style** `reference.settingsdialog.project.grazie` | Tabs *Scope* (Natural language text, Comments, String literals, Commit messages, per-language checkboxes) and *Rules* (rule tree per language with checkboxes, style categories). Default: text + comments + commit messages on; literals off (inferred). |
| **Spelling** `reference.settings.ide.settings.spelling` (Proj) | *Custom dictionaries:* table of `.dic` files/folders; *Accepted words* list; *Use single dictionary for saving words:* combobox project-level / application-level (default application-level, inferred). |

### SET-ED-026 Reader Mode — `editor.reader.mode` — Proj

| Option | Control | Default |
|---|---|---|
| Enable Reader mode | checkbox [bundle] — "Available for library and read-only files" | on |
| Show in Reader mode: Rendered documentation comments | checkbox | on |
| ↳ Font ligatures | checkbox | on |
| ↳ Increased line height | checkbox ("Increased by 1.2") | on |
| ↳ Code vision hints for usages, inheritors, and related problems | checkbox | on |
| ↳ Format code according to preferred style | checkbox | off |
| ↳ Use active scheme: X / Choose scheme: | radio + combobox | Use active scheme |
| Open files in Reader mode: (library files, read-only files, files from VCS) | radio/combobox (inferred) | Library and read-only files |

### SET-ED-027 TODO — `preferences.toDoOptions` — App

| Element | Behaviour |
|---|---|
| Patterns | table: Icon, Case sensitive, Pattern (regex); defaults `\btodo\b.*` and `\bfixme\b.*`, case-insensitive; per-pattern *Use color scheme TODO default colors* or custom text attributes |
| Filters | table: Name, Patterns (subset); used by the TODO tool window filter and Commit *Check TODO* |
| Search in multiline comments (inferred) | checkbox | on |

### SET-ED-029 … SET-ED-032 Other Editor pages
| ID | Page | Configurable id | Notes |
|---|---|---|---|
| SET-ED-029 | Emmet | `reference.idesettings.emmet` | HTML/CSS abbreviations; Enable Emmet, Expand abbreviation with Tab. |
| SET-ED-030 | GUI Designer [P] | `project.propGUI` | Generate GUI into: Binary class files / Java source code; Automatically copy form runtime classes; Default layout manager (GridLayoutManager); Default accessibility? |
| SET-ED-031 | Minimap | `com.intellij.minimap` | Show minimap (off), width, position left/right, file types. |
| SET-ED-032 | TextMate Bundles | `reference.settingsdialog.textmate.bundles` | Bundle table with checkboxes; +/−. |
| SET-ED-033 | Duplicates (commercial) | ? | Per-language anonymization settings for duplicate detection. |

Sources:
- https://www.jetbrains.com/help/idea/settings-editor-general.html
- https://www.jetbrains.com/help/idea/settings-auto-import.html
- https://www.jetbrains.com/help/idea/settings-editor-appearance.html
- https://www.jetbrains.com/help/idea/settings-code-completion.html
- https://www.jetbrains.com/help/idea/settings-code-folding.html
- https://www.jetbrains.com/help/idea/settings-console.html
- https://www.jetbrains.com/help/idea/settings-editor-tabs.html
- https://www.jetbrains.com/help/idea/settings-gutter-icons.html
- https://www.jetbrains.com/help/idea/settings-postfix-completion.html
- https://www.jetbrains.com/help/idea/settings-smart-keys.html
- https://www.jetbrains.com/help/idea/settings-editor-font.html
- https://www.jetbrains.com/help/idea/configuring-colors-and-fonts.html
- https://www.jetbrains.com/help/idea/settings-code-style.html and https://www.jetbrains.com/help/idea/code-style-java.html
- https://www.jetbrains.com/help/idea/settings-inspections.html
- https://www.jetbrains.com/help/idea/settings-file-and-code-templates.html
- https://www.jetbrains.com/help/idea/settings-file-encodings.html
- https://www.jetbrains.com/help/idea/settings-live-templates.html
- https://www.jetbrains.com/help/idea/settings-file-types.html
- https://www.jetbrains.com/help/idea/copyright.html
- https://www.jetbrains.com/help/idea/inlay-hints.html
- https://www.jetbrains.com/help/idea/settings-intentions.html
- https://www.jetbrains.com/help/idea/language-injections-settings.html
- https://www.jetbrains.com/help/idea/grammar.html, https://www.jetbrains.com/help/idea/spellchecking.html
- https://www.jetbrains.com/help/idea/reader-mode.html
- https://www.jetbrains.com/help/idea/settings-todo.html
- IntelliJ Community `ApplicationBundle.properties`, `JavaBundle.properties`, `CodeInsightBundle.properties`, `LangBundle.properties`, `IdeBundle.properties` (labels)

---

## 6. Plugins

### SET-PLG-001 Plugins — `preferences.pluginManager` — App

| Element | Behaviour |
|---|---|
| Tabs | **Marketplace** and **Installed** (Installed shows an update counter badge). |
| Search field | Per tab; supports filter tokens offered by a filter popup: `/tag:`, `/sortBy:` (downloads, name, rating, updated), `/vendor:`, `/organization:`, and on Installed `/enabled`, `/disabled`, `/bundled`, `/downloaded`, `/invalid`, `/updates`, `/outdated` (inferred list). |
| Marketplace list | Sections *Staff Picks*, *New and Updated*, *Top Downloads*, *Top Rated* when no query; each row: icon, name, vendor, downloads, rating, **Install** button. Paid plugins show price/trial tag. |
| Installed list | Groups *Downloaded*, *Bundled*, with *Update* buttons and enable checkbox per row; *Update All* link when updates exist. |
| Details pane | Name, vendor, version, tags; tabs *Overview*, *What's New*, *Reviews*, *Additional Info*; buttons **Install** / **Update** / **Enable** / **Disable** (split button: *Disable* / *Uninstall*), *Plugin homepage* link. Required dependencies listed; "Not loaded. The required plugin X is disabled." with *Enable required plugins* [bundle]. |
| Gear menu (⚙) | *Manage Plugin Repositories…*, *HTTP Proxy Settings…*, *Install Plugin from Disk…*, *Disable All Downloaded Plugins*, *Enable All Downloaded Plugins*, *Update All?* |
| Restart | Changes that need a restart show **Restart IDE** button in place of Install; on OK the IDE asks to restart now or later. Dynamic plugins load/unload without restart where possible. |
| Persistence | `disabled_plugins.txt` and plugin directory in config. |

Sources:
- https://www.jetbrains.com/help/idea/managing-plugins.html
- https://www.jetbrains.com/help/idea/settings-plugins.html
- IntelliJ Community `IdeBundle.properties` (labels)

---

## 7. Version Control

### SET-VCS-001 Version Control (root) — `project.propVCSSupport.Mappings` — Proj

| Option | Control | Default |
|---|---|---|
| Directory mappings | table Directory / VCS (`<Project>` root + detected roots); +/−/edit; unregistered roots notice with *Add root* | detected roots |
| Highlight directories that contain modified files in the Project tree | checkbox [bundle] | on |
| Highlight files changed in the last … days | checkbox + spinner [bundle] — "Modified files will be highlighted in the external stack trace and while debugging." | off, 1 |
| Limit history to … rows | checkbox + spinner [bundle] | off, 1000 |
| Filter Update Project information by scope | checkbox + scope combobox + *Manage Scopes* [bundle] | off |
| Restore workspace when switching branches | checkbox [bundle] | on |
| Show changed in last N days in "Changed" scope? | (merged with above) | — |

### SET-VCS-002 Commit — `project.propVCSSupport.CommitDialog` (inferred) — Proj

| Option | Control | Default |
|---|---|---|
| Use non-modal commit interface | checkbox — "Applies to projects under Git and Mercurial" [bundle]; inverse of advanced `git.non.modal.commit` false [adv] | on |
| **Commit message**: Clear initial commit message | checkbox [bundle] | off |
| Commit message inspections: | checklist with options [bundle]: Blank line between subject and body (on), Limit body line (on, 72), Limit subject line (on, 72), Spelling (on); per-inspection severity; *Show right margin* (Right margin: 72) [bundle] | as listed |
| **Commit Checks** (group "Git Checks"/"Advanced Git Checks" [bundle]) | | |
| Reformat code | checkbox | off |
| Rearrange code | checkbox | off |
| Optimize imports | checkbox | off |
| Analyze code (with profile, *Choose profile*) | checkbox [bundle] | on |
| Check TODO (filter, *Configure*) | checkbox [bundle] | off |
| Cleanup (with profile) | checkbox [bundle] | off |
| Update copyright | checkbox | off |
| Run Tests (choose run configuration) | checkbox + combobox | off |
| Run advanced checks after a commit is done | checkbox — "Failed checks will not prevent a commit" [bundle] | on |
| **Changelists**: Create changelists automatically | checkbox [bundle] | off |
| Show unversioned files in Commit tool window? (inferred) | checkbox | on |

### SET-VCS-003 Changelists — `project.propVCSSupport.ChangelistConflict` (inferred) — Proj

| Option | Control | Default |
|---|---|---|
| Create changelists automatically | checkbox [bundle] | off |
| Allow putting changes within one file into different changelists | checkbox [bundle] | on |
| When an empty changelist becomes inactive: | radio Ask / Delete / Do not delete (inferred values) [bundle label] | Ask |
| **Inactive Changelist**: Show dialog on attempt to edit file from inactive changelist | checkbox [bundle] | off |
| Highlight files from inactive changelists | checkbox [bundle] | off |
| **Conflicts**: Highlight files with changelist conflicts | checkbox [bundle] | on |
| Files with ignored conflicts: | list + Clear | empty |

### SET-VCS-004 Confirmation — `project.propVCSSupport.Confirmation` (inferred) — Proj

| Option | Control | Default |
|---|---|---|
| When files are created: | radio Ask / Add silently / Do not add [bundle] | Ask |
| ↳ Apply to files created outside IntelliJ IDEA | checkbox [bundle] | off |
| When files are deleted: | radio Ask / Remove silently / Do not remove [bundle] | Ask |
| Show options before: (Update, Status, Edit, …) | checkboxes per VCS operation [bundle "Show options before:"] | Update on (inferred) |
| Ask to unlock files set to read-only if you attempt to edit them | checkbox [bundle] | on? |
| Ask for confirmation to drop commits (Git) | checkbox [bundle] | on |

### SET-VCS-005 Directory Mappings — `project.propVCSSupport.DirectoryMappings` (inferred) — Proj
Same mapping table as the root page (Directory, VCS combobox); dialog *Add VCS Directory Mapping* with Project / Directory radio, VCS combobox, *Configure VCS* button.

### SET-VCS-006 File Status Colors — `reference.settingsdialog.IDE.editor.colors.File Status` (inferred) — App
| Element | Behaviour |
|---|---|
| Table | Status name (Modified, Added, Deleted, Unknown, Ignored, Merged with conflicts, Modified in not active changelist, Added in not active changelist, Changelist conflict [bundle], Hijacked, Switched, …) with color swatch; editing opens color picker *File Status Color:*; *Restore Default* button [bundle]; note "The color is set by the current UI Theme and cannot be overridden" when locked [bundle]. |

### SET-VCS-007 Issue Navigation — `project.propVCSSupport.IssueNavigation` (inferred) — Proj
| Element | Behaviour |
|---|---|
| Table | Columns *Issue* (regex) and *Link* (URL with `$0`, `$1` groups) [bundle]; + Add, Edit, Remove; *Add JIRA Pattern…* (asks JIRA installation URL) and *Add YouTrack Pattern…* [bundle]; empty text "No patterns configured". |

### SET-VCS-008 Log — `vcs.log` — Proj
| Option | Control | Default |
|---|---|---|
| Index repositories for faster search (per-root checkboxes) | checkbox list | on |
| Commit details / graph appearance options (inferred) | checkboxes | ? |

### SET-VCS-009 Shelf — `Shelf.Project.Settings` (inferred) — Proj
| Option | Control | Default |
|---|---|---|
| Shelve base revisions of files under distributed version control systems | checkbox [bundle] | on |
| ↳ (note) The base content of files larger than N K will not be stored [bundle] | label | — |
| Remove successfully applied files from the shelf | checkbox [bundle] | on |
| Delete already unshelved changelists: … days | checkbox + spinner (inferred) | ? |
| Change Shelves Location… | button → dialog with *Default location is* / custom path, *Move shelves to the new location* [bundle] | `.idea/shelf` |

### SET-VCS-010 Git — `vcs.Git` — Proj

| Option | Control | Default |
|---|---|---|
| Path to Git executable: | path + **Test** button; auto-detected label "Auto-detected: …" [bundle] | auto-detected |
| ↳ Set this path only for current project | checkbox | off |
| **Commit** group: Add the 'cherry-picked from <hash>' suffix when picking commits pushed to protected branches | checkbox [bundle] | off |
| Warn if CRLF line separators are about to be committed | checkbox [bundle] | on |
| Warn when committing in detached HEAD or during rebase | checkbox [bundle] | on |
| Warn when committing files larger than … MB | checkbox + spinner [bundle] | on, 50 |
| Warn when committing files with names that might cause issues on other systems | checkbox [bundle] | on |
| Sign commits with GPG key / *Configure GPG Key…* | status label + link → dialog (Root, GPG key table, *Sign commits with GPG key:* checkbox) [bundle] | not configured |
| Enable staging area | checkbox — "This will disable changelists support. Only for non-modal commit interface." [bundle] | off |
| **Push** group: Auto-update if push of the current branch was rejected | checkbox [bundle] | off |
| Show Push dialog for Commit and Push | checkbox [bundle] | on |
| ↳ Show Push dialog only when committing to protected branches | checkbox [bundle] | off |
| Protected branches: | text (patterns) [bundle] | `master, main` (inferred) |
| ↳ Load branch protection rules from GitHub | checkbox [bundle] | on |
| **Update** group: Update method: | combobox Merge / Rebase / Branch Default [bundle] | Merge |
| Clean working tree using: | radio Stash / Shelve [bundle] | Stash |
| Filter "Update Project" information by paths: | checkbox + scope [bundle] | off |
| Fetch remote changes automatically | checkbox [bundle] | off? |
| Check remote for incoming changes | checkbox (requires advanced option) [bundle] | on [adv `git.update.incoming.outgoing.info` true] |
| Fetch tags: | combobox Auto ("Follow git config") / Sync / Always / Never [bundle] | Auto |
| Use credential helper | checkbox [bundle] | off |
| **Stash**: Combine stashes and shelves in one tab | checkbox [bundle] | on |
| ↳ When "Show Diff" is called, compare stashed changes: | radio With the local version of a file / With the parent commit [bundle] | With the parent commit? |
| **Cherry-Pick**: On empty cherry-pick commit: | radio Skip commit / Create empty commit [bundle] | Skip commit? |
| Explicitly check for incoming commits on remotes? | see above | — |

### SET-VCS-011 GitHub — `GithubSettingsConfigurable` — Proj (accounts are App)
| Element | Behaviour |
|---|---|
| Accounts table | Account rows with avatar, login, server; *+* menu: *Log In via GitHub…* (browser OAuth), *Log In with Token…*, *Log In to GitHub Enterprise…*; − Remove; *Set as Default* (star). |
| Clone git repositories using ssh | checkbox — off |
| Connection timeout: … seconds | spinner — 5 |
| Advanced: GitHub host aliases `github.aliases` = `github.com` [adv] | | |

### SET-VCS-012 GitLab — `GitLabSettingsConfigurable` — Proj
Accounts table (Log In via GitLab / with Token, server URL); options for merge request review (inferred).

### SET-VCS-013 Mercurial / Subversion — `vcs.Mercurial`, `vcs.Subversion` (+ Network, Presentation, SSH) — Proj
Executable path, ignore whitespace, etc. Out of scope for the first workflow; options not listed.

Sources:
- https://www.jetbrains.com/help/idea/settings-version-control.html
- https://www.jetbrains.com/help/idea/settings-version-control-commit.html (Commit page)
- https://www.jetbrains.com/help/idea/settings-version-control-confirmation.html
- https://www.jetbrains.com/help/idea/settings-version-control-changelists.html
- https://www.jetbrains.com/help/idea/settings-version-control-issue-navigation.html
- https://www.jetbrains.com/help/idea/settings-version-control-shelf.html
- https://www.jetbrains.com/help/idea/settings-version-control-git.html
- https://www.jetbrains.com/help/idea/settings-version-control-github.html
- IntelliJ Community `VcsBundle.properties`, `GitBundle.properties` (labels)

---

## 8. Build, Execution, Deployment

### SET-BLD-001 Build Tools — `build.tools` — Proj

| Option | Control | Default |
|---|---|---|
| Sync project after changes in the build scripts: | checkbox + radio [bundle] | on |
| ↳ Any changes | radio | — |
| ↳ External changes — "Changes outside the IDE or from version control." | radio [bundle] | selected |
| (Floating *Sync Gradle Changes* button appears in the editor when the scripts change; hint "Auto-Sync Projects" [bundle]) | — | — |

Note: 2026.2 bundle wording uses **Sync** (earlier releases said *Reload project*).

### SET-BLD-002 Gradle — `reference.settingsdialog.project.gradle` — Proj

| Option | Control | Default |
|---|---|---|
| **General Settings** | | |
| Gradle user home: | path — "Override the default location where Gradle stores downloaded files…" [bundle] | `~/.gradle` (or `GRADLE_USER_HOME`) [doc] |
| Generate *.iml files for modules imported from Gradle | checkbox [bundle] | off |
| Enable parallel Gradle model fetching for Gradle 7.4+ | checkbox — "Incubating" warning [bundle] | off for new projects [doc] |
| **Gradle Projects** | | |
| Linked project selector | list of linked Gradle projects (left list when >1) | — |
| Download external annotations for dependencies | checkbox [bundle] | on |
| **Build and Run** | | |
| Build and run using: | combobox Gradle / IntelliJ IDEA [bundle] | Gradle [doc] |
| Run tests using: | combobox Gradle / IntelliJ IDEA / Choose per test [bundle] | Gradle [doc] |
| **Gradle** (*Gradle Source* group [bundle]) | | |
| Distribution: | combobox Wrapper / Wrapper task / Local installation [bundle] (help: "'gradle-wrapper.properties' file", "'wrapper' task in Gradle build script", "Specified location") | Wrapper [doc] |
| ↳ Location: | path (Local installation) — validation "Invalid Gradle SDK installation" [bundle] | — |
| ↳ Version: | combobox incl. *Auto-select* (new-project wizard / wrapper task) [bundle] | Auto-select |
| Gradle JVM: | JDK combobox (Project SDK, JAVA_HOME, registered JDKs, *Download JDK…*, *Add JDK…*) | Project SDK [doc] |
| Gradle JVM criteria: Version: / Vendor: | text / combobox Any vendor / Custom vendor (shown when daemon toolchain criteria are defined in `gradle/gradle-daemon-jvm.properties`) [bundle] | Undefined / Any vendor |
| Gradle VM options | moved to `gradle.properties`; page shows *Migrate* link if legacy value exists [bundle] | — |
| Generate {IDE} module per Gradle source set | checkbox, deprecated, shown only when previously changed [bundle] | on |
| **Advanced Settings** related | Download sources `gradle.download.sources` = true; Run using Gradle `gradle.run.using.gradle` = true; Attach scripts dependencies sources = false [adv] | |
| Offline mode | not on this page; toggle in Gradle tool window toolbar [doc] | off |

### SET-BLD-003 Maven — `MavenSettings` — Proj

| Option | Control | Default |
|---|---|---|
| Work offline | checkbox [bundle] | off |
| Use settings from .mvn/maven.config | checkbox [bundle] | on |
| Execute goals recursively | checkbox [bundle] | on |
| Print exception stack traces | checkbox [bundle] | off |
| Always update snapshots | checkbox [bundle] | off |
| Output level: | combobox Quiet / Info / Debug | Info |
| Checksum policy: | combobox No Global Policy / Fail / Warn | No Global Policy |
| Multiproject build fail policy: | combobox Default / Fail Fast / Fail At End / Never Fail | Default |
| Thread count (-T option) | text [bundle] | empty |
| Maven home path | combobox Bundled (Maven 3) / Use Maven wrapper / custom path | Use Maven wrapper if present, else Bundled (inferred) |
| User settings file | path + Override [bundle] | `~/.m2/settings.xml` |
| Local repository | path + Override [bundle] | `~/.m2/repository` |
| Show settings dialog for new Maven projects | checkbox [bundle] | off |

Subpages:

| ID | Page | Id | Options (defaults) |
|---|---|---|---|
| SET-BLD-004 | Importing | `reference.settings.project.maven.importing` | Keep source and test folders on project reload (off); Search for projects recursively? (on); Use Maven output directories (off); Phase to be used for folders update: (process-resources); Generated sources folders: Detect automatically / target/generated-sources / subdirectories / Don't detect (Detect automatically); Automatically download: Sources (off), Documentation (off), Annotations (on); Dependency types: (`jar, test-jar, maven-plugin, ejb, ejb-client, jboss-har, jboss-sar, war, ear, bundle`); VM options for importer: (`-Xmx768m`); JDK for importer: (Use Project JDK); Detect compiler automatically (on); Use M2E compatibility for Maven plugins? ; Exclude build directory (%PROJECT_ROOT%/target) (on); Create modules for aggregator projects (on); Enable fast import? |
| SET-BLD-005 | Ignored Files | `reference.settings.project.maven.ignored.files` | Checkbox list of `pom.xml` files; Ignored files patterns text. |
| SET-BLD-006 | Runner | `reference.settings.project.maven.runner` | Delegate IDE build/run actions to Maven (off); Run tests using: Maven/IntelliJ? ; VM Options:; JRE: (Use Project JDK); Environment variables; Properties: table; Skip Tests (off). |
| SET-BLD-007 | Running Tests | `reference.settings.project.maven.testRunning` | Pass to JUnit process: argLine (on), systemPropertyVariables (on), environmentVariables (on). |
| SET-BLD-008 | Repositories | `reference.settings.project.maven.repository.indices` | Indexed Maven Repositories table (URL, Type, Updated); Update button. |
| SET-BLD-009 | Archetype Catalogs | `reference.settings.project.maven.archetype.catalogs` | Catalog table (name, location); Add/Remove. |
| SET-BLD-009a | Gant | `reference.settingsdialog.project.gant` | Gant SDK home. |

### SET-BLD-010 Compiler — `project.propCompiler` — Proj

| Option | Control | Default |
|---|---|---|
| Resource patterns: | text | `!?*.java;!?*.form;!?*.class;!?*.groovy;!?*.scala;!?*.flex;!?*.kt;!?*.clj;!?*.aj` |
| Clear output directory on rebuild | checkbox | on |
| Add runtime assertions for notnull-annotated methods and parameters | checkbox + *Configure annotations…* | on |
| Automatically show first error in editor | checkbox | off |
| Display notification on build completion | checkbox | off |
| Build project automatically | checkbox ("only works while not running/debugging"; see advanced `compiler.automake.allow.when.app.running` false [adv]) | off |
| Compile independent modules in parallel | checkbox / combobox (Enable / Automatic) [doc] | Automatic? |
| Rebuild module on dependency change | checkbox | on |
| Build process heap size (Mbytes): / Shared heap size: | spinner | 700 (inferred; "auto" in some builds) |
| Shared VM options: | text | empty |
| User-local VM options (overrides Shared options): | text, stored in workspace.xml [doc] | empty |
| User-local heap size | spinner [doc] | empty |
| Advanced: Run compilation with lower priority (true), unified Java/Kotlin IC (true), detailed debug log (false) [adv] | | |

Subpages:

| ID | Page | Id | Options (defaults) |
|---|---|---|---|
| SET-BLD-011 | Annotation Processors | `reference.projectsettings.compiler.annotationProcessors` | Profile tree (*Default* + custom; modules assigned per profile; Gradle/Maven create profiles on sync); **Enable annotation processing** (off; Gradle-imported projects set per profile); Obtain processors from project classpath / Processor path: (classpath); Use --processor-module-path compiler option (off); Store generated sources relative to: Module output directory / Module content root (Module content root); Production sources directory: (`generated`); Test sources directory: (`generated_tests`); Annotation Processors table (FQ names); Annotation Processor options table (Option Name / Value). |
| SET-BLD-012 | Compiler Cache | `reference.projectsettings.compiler.compilercache` | Download and use shared compiled outputs (project-dependent; options not documented — open question). |
| SET-BLD-013 | Excludes | `reference.projectsettings.compiler.excludes` | Table of files/folders excluded from compilation, *Recursively* checkbox per entry. |
| SET-BLD-014 | Groovy Compiler | `Groovy compiler` | Path to configscript; Invoke dynamic support (off); Exclude from stub generation table. |
| SET-BLD-015 | Java Compiler | `reference.projectsettings.compiler.javacompiler` | Use compiler: Javac / Eclipse / Ajc? (Javac); Use '--release' option for cross-compilation (Java 9 and later) (on [doc]); Project bytecode version: (Same as language level); Per-module bytecode version table (Module, Target bytecode version); **Javac Options**: Use compiler from module target JDK when possible (on), Generate debugging info (on), Report use of deprecated features (off), Generate no warnings (off), Additional command line parameters: (text with Expand); Override compiler parameters per-module table (module, parameters); **Eclipse Options**: Path to ECJ batch compiler tool (empty = bundled), Proceed on errors (off). |
| SET-BLD-016 | Kotlin Compiler | `project.kotlinCompiler` | Language version (Latest stable), API version (Latest stable), Report compiler warnings (on), Additional command line parameters, Kotlin/JVM: Target JVM version (from SDK), Destination directory, Keep compiler process alive between invocations (on), Enable incremental compilation (on); Kotlin/JS section. Usually overridden by Gradle import. |
| SET-BLD-017 | RMI Compiler | `reference.projectsettings.compiler.rmicompiler` | Enable RMI stubs generation (off); Generate debugging info; Generate no warnings; Generate IIOP stubs; Additional command line parameters. |
| SET-BLD-018 | Validation | `project.validation` | Validate on build table (validator name, Enabled), Exclude from validation list. |

### SET-BLD-020 Coverage — `coverage` — Proj

| Option | Control | Default |
|---|---|---|
| When New Coverage Is Gathered | radio: Show options before applying coverage to the editor / Do not apply collected coverage / Replace active suites with the new one / Add to the active suites [bundle] | Show options before applying |
| Activate Coverage View | checkbox [bundle] | on |
| Show coverage in the project view | checkbox [bundle] | on |
| **Java Coverage**: Choose coverage runner: IntelliJ IDEA / JaCoCo | combobox | IntelliJ IDEA |
| ↳ Branch coverage / Tracing? | checkbox | on (2024+) |
| ↳ Track per test coverage | checkbox | off |
| ↳ Ignore implicitly declared default constructors | checkbox [bundle] | on |
| ↳ Exclude annotations (e.g. `*Generated*`) | table | `*Generated*` |
| ↳ Test folders coverage? | checkbox | off |

### SET-BLD-030 Debugger — `project.propDebugger` — App

| Option | Control | Default |
|---|---|---|
| Show debug window on breakpoint | checkbox [bundle] | on |
| Focus application on breakpoint | checkbox [bundle] | off |
| Hide debug window on process termination | checkbox [bundle] | off |
| Scroll execution point to center | checkbox [bundle] | off |
| Click line number to perform run to cursor | checkbox [bundle] | on |
| Remove breakpoint: | radio Click with left mouse button / Drag to the editor or click with middle mouse button [bundle] | Click with left mouse button |
| ↳ Confirm removal of conditional or logging breakpoints | checkbox [bundle] | on |
| Focus target process after resume (Windows) | checkbox [bundle] | off |
| **Java**: Transport: | radio Socket / Shared memory (Windows) [bundle] | Socket |
| ↳ Show alternative source switcher | checkbox [bundle] | on |
| ↳ Kill the debug process immediately | checkbox [bundle] | off |
| ↳ Attach memory agent | checkbox [bundle] | on |
| ↳ Start run configurations with the debug agent | checkbox [bundle] | off |
| ↳ Resume only the current thread | checkbox [bundle] | off |
| **Kotlin**: Attach coroutine agent | checkbox [doc] | on |
| Advanced: Maximum number of recent expressions = 50; Show inlay Run to Cursor popup = true [adv] | | |

Subpages:

| ID | Page | Id | Options (defaults) |
|---|---|---|---|
| SET-BLD-031 | Data Views | `debugger.dataViews` (inferred; display name in XDebuggerBundle) | Show values inline (on); Show value tooltip (on); Show value tooltip on code selection (on); Value tooltip delay (ms): (700); Enable auto expressions in Variables view (on); Sort values alphabetically (off). |
| SET-BLD-032 | Data Views › Java | `reference.idesettings.debugger.dataviews.java`? (inferred) | Autoscroll to new local variables (on); Predict condition values and exceptions based on data flow analysis (on); ↳ Gray out blocks of code that are predicted to be unreachable (on); **Show**: Declared type (off), Fully qualified names (off), Object id (on), Static fields (on), Static final fields (off), Synthetic fields (on), $val fields as local variables (on), Show hex value for primitives (off), Show type for strings (off) [labels bundle]; Hide null elements in arrays and collections (on); Auto populate Throwable object's stack trace (on); Enable alternative view for Collections classes (on); Enable 'toString()' object view: (on) with radio For all classes that override 'toString()' method / For classes from the list: (table). |
| SET-BLD-033 | Data Views › Kotlin | ? | Kotlin-specific renderers (e.g. disable coroutine agent info). |
| SET-BLD-034 | Java Type Renderers | `reference.idesettings.debugger.typerenderers` (inferred) | Renderer list (+/−/copy/up/down, enable checkbox); Renderer name:; Apply renderer to objects of type (fully qualified name):; When rendering a node: Use default renderer / Use following expression: (+ On-demand); Show type and object id; When expanding a node: Use default renderer / Use list of expressions: (table Name / Expression / On-demand); Test if a node can be expanded (optional):; Append default children [bundle]. |
| SET-BLD-035 | Stepping | `debugger.stepping` (inferred) | Common: Always do smart step into (on). Java: Skip synthetic methods (on); Skip constructors (off); Skip class loaders (on); Skip simple getters (off); Do not step into the classes (on) table: `com.sun.*`, `java.*`, `javax.*`, `jdk.internal.*`, `org.omg.*`, `sun.*`, `junit.*`, `org.junit.*`, `kotlin.*`?; Hide stack frames using stepping filters (on); Evaluate finally blocks on pop frame: Always / Never / Ask (Ask); Resume only current thread?; Show warning when stepping over breakpoints (inferred). Kotlin: Do not step into specific Kotlin classes (on), Do not step into Kotlin library classes. |
| SET-BLD-036 | HotSwap | `debugger.hotswap` (inferred) | Build project before reloading classes (on); Enable 'JVM will hang' warning (on); Reload classes after compilation: Always / Never / Ask (Ask); Suggest HotSwap in the editor when code is modified (on) [labels bundle]. |
| SET-BLD-037 | Async Stack Traces | `CaptureConfigurable` [P] | Table of capture points: Capture class name / method name / key expression, Insert class name / method name / key expression (+/−/duplicate/enable); Breakpoints based: …; Instrumenting agent (requires debugger restart) (on); Capture local variables (may greatly slow down the execution) (off); Throttle async stack traces on significant overhead (on); *Configure Annotations…* (Async Schedule / Async Execute annotation lists) [bundle]. |

### SET-BLD-040 Remote Jar Repositories — `RemoteRepositoriesConfigurable` — Proj
Table of repository URLs used for *From Maven…* library downloads: defaults `https://repo1.maven.org/maven2/` (Maven Central) and `https://repository.jboss.org/nexus/content/repositories/public/`; +/−/edit, *Reset to defaults*. Section "Bundled/Additional repositories" (inferred).

### SET-BLD-041 Run Targets — `Runtime.Targets.Configurable` — Proj
| Element | Behaviour |
|---|---|
| Target list | + Docker, Docker Compose?, SSH, WSL (Windows) targets; − / copy. |
| Target page | Name, connection (SSH config / Docker image / WSL distribution), *Project default target* selector (Local machine), *Introspect* button; **Languages** section with *Java* runtime (JDK home path, JDK version), *Maven* (home path, version), *Gradle* (home path, version) entries. |

### SET-BLD-042 Run Widget — `RunToolbarSettingsConfigurableProvider` — Proj
Controls run widget behaviour (e.g. how many configurations are pinned, show in toolbar). Options not documented — open question.

### SET-BLD-050 Other Build pages (not detailed)
| Page | Id | Note |
|---|---|---|
| Clouds | `RemoteServers` | Remote server/cloud deployment configs |
| Environment Dashboard | `ijent.dashboard` [P] | Remote/container environments overview |
| Java Profiler | `promo.profiler` | Commercial; profiler configurations (Async Profiler, Java Flight Recorder) |
| Kubernetes | `promo.kubernetes` | Commercial |
| Application Servers, Deployment, Docker, Required Plugins, Package Search | † commercial / plugin | out of scope |

Sources:
- https://www.jetbrains.com/help/idea/settings-build-tools.html (Build Tools page)
- https://www.jetbrains.com/help/idea/gradle-settings.html
- https://www.jetbrains.com/help/idea/maven-support.html and its *Maven settings* subpages
- https://www.jetbrains.com/help/idea/compiler.html
- https://www.jetbrains.com/help/idea/java-compiler.html
- https://www.jetbrains.com/help/idea/annotation-processors-support.html
- https://www.jetbrains.com/help/idea/settings-debugger.html and subpages (data views, stepping, HotSwap, async stack traces, type renderers)
- https://www.jetbrains.com/help/idea/settings-code-coverage.html
- https://www.jetbrains.com/help/idea/remote-jar-repositories.html
- https://www.jetbrains.com/help/idea/run-targets.html
- IntelliJ Community `GradleBundle.properties`, `ExternalSystemBundle.properties`, `MavenConfigurableBundle.properties`, `XDebuggerBundle.properties`, `JavaDebuggerBundle.properties`, `CoverageBundle.properties`, `JavaCoverageBundle.properties` (labels)

---

## 9. Languages & Frameworks

| ID | Page | Configurable id | Level | Options (defaults) |
|---|---|---|---|---|
| SET-LANG-001 | Kotlin | `preferences.language.Kotlin` | App | Kotlin plugin version label; Kotlin compiler/plugin mode notes (K2 is the only/ default mode in 2026.x — inferred); *Enable Kotlin autoconfiguration* and *Compiler reference index* live in Advanced Settings (`kotlin.enable.autoconfiguration` = true, `kotlin.compiler.ref.index` = true) [adv]. |
| SET-LANG-002 | Kotlin › Kotlin Scripting | `preferences.language.Kotlin.scripting` | Proj | Script definitions table (name, pattern/extension, enabled checkbox, order); *Load script dependencies automatically* (on, inferred). |
| SET-LANG-003 | JVM Logging | `jvm.logging` | Proj | Logger: combobox Unspecified / Java Util Logging / Log4j / Log4j2 / SLF4J / Apache Commons Logging (Unspecified = auto-detect); Variable name: (`LOG` / `log`, inferred) [bundle label]. |
| SET-LANG-004 | Lombok | `ProjectSettingsPage` | Proj | Enable Lombok plugin support / Enable lombok version warning / config file (`lombok.config`) handling (labels inferred). |
| SET-LANG-005 | JavaFX | `preferences.JavaFX` | App | Path to SceneBuilder: path. |
| SET-LANG-006 | Markdown | `Settings.Markdown` | Proj | Preview: Default layout (Editor and Preview), Preview layout (Vertical/Horizontal split), Sync preview scroll with editor (on); Markdown Extensions checkboxes (e.g. Mermaid, PlantUML); Custom CSS (Load from URI, Add CSS rules); Code style options; Hide floating toolbar and squash dashes in Advanced Settings [adv]. |
| SET-LANG-007 | Mermaid | `Settings.Mermaid` | App | Mermaid rendering options. |
| SET-LANG-008 | Schemas and DTDs | `preferences.externalResources` | Proj | External schemas table (URI → Location); Ignored schemas and DTDs; Default HTML language level. Subpages: Default XML Schemas (`xml.defaultSchemas`): Default XML Schema version 1.0/1.1; JSON Schema Mappings (`settings.json.schema`): mapping list (name, schema file/URL, schema version, file path patterns); Remote JSON Schemas (`settings.json.schema.catalog`): Allow downloading JSON schemas from remote sources (on), Use schemastore.org JSON Schema catalog (on), Always download the most recent version of schemas (on); XML Catalog (`xml.catalog`): catalog property file path. |
| SET-LANG-009 | Template Data Languages | `Template Data Languages` | Proj | Table Path → template data language; *Project Language* combobox. |
| SET-LANG-010 | XSLT, XSLT File Associations | `settings.xslt`, `xslt.associations` | App/Proj | XSLT processor options; associations table. |
| SET-LANG-011 | DotEnv | `DotEnv.SettingsForm` | App | Options for `.env` completion. |
| SET-LANG-020 | Commercial pages (not detailed): Spring, Spring Boot?, Jakarta EE / Java EE, JPA, SQL Dialects, SQL Resolution Scopes, JavaScript, TypeScript, Node.js, Style Sheets, Frameworks (Quarkus, Micronaut, Helidon), OpenAPI Specifications (`promo.swagger`), Kubernetes, Docker Compose | † | — | Out of scope for the Java + Gradle MVP; record ids when implemented. |

Sources:
- https://www.jetbrains.com/help/idea/settings-languages-and-frameworks.html (section index)
- https://www.jetbrains.com/help/idea/kotlin-scripting-settings.html (inferred URL; verify)
- https://www.jetbrains.com/help/idea/markdown.html
- https://www.jetbrains.com/help/idea/json.html
- [settings-tree.md](settings-tree.md)

---

## 10. Tools

### SET-TOOL-001 Actions on Save — `actions.on.save` — Proj

Table of actions, each row with a checkbox, a summary of its scope and *Configure…* links.
Actions run when a file is saved explicitly (`Ctrl+S` / Save All) and on autosave triggered by
frame deactivation/idle (inferred: since 2023.x they run on any save performed by the IDE; see
Open questions).

| Action | Sub-options | Default |
|---|---|---|
| Reformat code | file types: *All file types* / *Select file types* popup ("Files: X, N more") [bundle]; *Whole file* / *Changed lines* combobox [bundle]; *Configure scope…* (Code Style › Formatter "Do not format") | off |
| Optimize imports | *Configure scope…* | off |
| Rearrange code | *Configure arrangement rules…* [bundle] | off |
| Run code cleanup | profile: *Project Profile* / *Profile: X* [bundle] | off |
| Update copyright notice | *Configure copyright settings…* | off |
| Build project | — (only when Build project automatically is off; inferred) | off |
| Run eslint / Run Prettier / Upload to default server | commercial plugins | off |
| Links | *Configure autosave options…* (System Settings), *Configure keymap…* (Save shortcut) | — |

### SET-TOOL-002 Terminal — `terminal` — Proj (top section) + App

| Option | Control | Default |
|---|---|---|
| **Project Settings**: Start directory: | path | project root |
| Environment variables: | env editor (name/value table, *Include system environment variables*) | empty |
| **Application Settings**: Shell path: | combobox editable (detected shells: PowerShell, cmd.exe, Git Bash, WSL on Windows; `$SHELL` on Unix) | detected default |
| Default tab name: | text | Local [bundle] |
| Terminal engine: | combobox Reworked 2025 / Classic / Experimental 2024 (deprecated) [bundle] | Reworked 2025 |
| Use application title as tab name: | checkbox + combobox When command is running / Always [bundle] | ? |
| Audible bell | checkbox | on |
| Close session when it ends | checkbox | on |
| Mouse reporting | checkbox | on |
| Copy to clipboard on selection | checkbox | off |
| Paste on middle mouse button click | checkbox | on |
| Override IDE shortcuts | checkbox + *Configure terminal keybindings* (Keymap › Plugins › Terminal) [bundle] | on |
| Shell integration | checkbox | on |
| Highlight hyperlinks | checkbox | on |
| Use Option as Meta key | checkbox (macOS) | off |
| Cursor shape: | combobox Block / Underline / Vertical | Block |
| Run Commands using IDE | checkbox (smart command handling, Ctrl+Enter) [bundle] | on |
| Enable inline AI completion suggestions | checkbox (AI Assistant) [bundle] | off |
| Move focus to the Editor with: | combobox Escape / ? [bundle] | Escape |
| **Command Completion**: Show a completion popup as you type: | combobox Only for parameters / Always / Never [bundle] | Only for parameters? |
| ↳ Insert suggestion with: / Show completion popup with: | shortcut pickers [bundle] | Tab? / Ctrl+Space |
| **Font Settings**: Font: / Fallback: / Size: / Line height: / Column width: | comboboxes/spinners [bundle] | editor font, 1.0 line height, 1.0 column width |
| Enforce minimum contrast ratio: | checkbox + spinner [1–21] [bundle] | 4.5 [bundle] |
| Configure colors… | link → Color Scheme › Console Colors [bundle] | — |
| Prompt style: (2024 engine) / Show separators between executed commands | radio IDE single-line / IDE double-line / Shell prompt (PS1) [bundle] | ? |
| Advanced (scrollback 5000 lines, typeahead, encoding UTF-8, Shift+Enter sends Esc+CR) | see settings-tree.md [adv] | |

### SET-TOOL-003 External Tools — `preferences.externalTools` — App
| Element | Behaviour |
|---|---|
| Tree | Groups and tools with enable checkboxes; +/−/edit/copy/up/down. |
| Create/Edit Tool dialog | Name:, Group:, Description:, **Tool Settings**: Program:, Arguments:, Working directory: (with *Insert Macro…*); **Advanced Options**: Synchronize files after execution (on), Open console for tool output (on), ↳ Make console active on message in stdout (off) / stderr (off), Output filters: (text). Menu placement of tools is configured on Menus and Toolbars (inferred). |
| Macros | `$FilePath$`, `$ProjectFileDir$`, `$SelectedText$`, … via *Insert Macro…* dialog with preview. |

### SET-TOOL-004 Shared Indexes — id unknown † — App/Proj
| Option | Control | Default |
|---|---|---|
| JDKs: | radio Download automatically / Ask before download / Don't download, use local indexes | Download automatically |
| Maven libraries: | same radio | Ask before download |
| Project indexes (with `intellij.yaml`): | same radio (commercial / shared index server) | Ask before download? |
| Wait for shared indexes? | checkbox | ? |

### SET-TOOL-005 Startup Tasks — `preferences.startup.tasks` — Proj
Table of run configurations executed when the project opens; + adds existing run configuration (popup) or creates one; − remove; checkbox *Shared* per row? (stored in `.idea/workspace.xml` or shared file — inferred).

### SET-TOOL-006 Diff & Merge — `diff.base` — App
| Option | Control | Default |
|---|---|---|
| **Diff**: Context lines: | slider/spinner [bundle] | 4 |
| Go to the next file after reaching last change | checkbox [bundle] | on |
| (diff viewer toggles such as Ignore whitespaces, Highlight words, Collapse unchanged fragments are persisted from the viewer, not edited here) | — | — |
| **Merge**: Automatically apply non-conflicting changes | checkbox [bundle] | off |
| Automatically resolve conflicts in import statements | checkbox [bundle] | on |
| Highlight modified lines in gutter (inferred, merge editor) | checkbox [bundle] | on |
| Subpage **External Diff Tools** `diff.external` | *Enable external tools* checkbox (off) [bundle]; *Configure external tools:* tree (Tool group, Tool name, Program path, Argument pattern, Trust process exit code) with *Test Diff*, *Test Three-Side Diff*, *Test Merge* [bundle]; table *File Type / Diff Tool / Merge Tool* (Default → Built-in) [bundle]. | |

### SET-TOOL-007 … SET-TOOL-016 Other Tools pages
| ID | Page | Configurable id | Level | Options |
|---|---|---|---|---|
| SET-TOOL-007 | Tasks | `tasks` (+ Servers `tasks.servers`) | Proj | Changelist name format, feature branch name format, Save context on switching task, Commit message format, Update cache; Servers: issue tracker accounts (YouTrack, Jira, GitHub, …). |
| SET-TOOL-008 | Web Browsers and Preview | `reference.settings.ide.settings.web.browsers` | App | Browsers table (Name, Family, Path) [bundle]; Default Browser: System default / First listed / Custom path [bundle]; Show browser popup in the editor: For HTML files / For XML files [bundle]; Reload behavior (On Save / On Change / Disabled) [bundle]. |
| SET-TOOL-009 | Features Trainer | `FeaturesTrainerSettingsPanel` | App | Reset lessons progress, choose learning language. |
| SET-TOOL-010 | Features Suggester | `org.intellij.featuresSuggester.FeatureSuggesterConfigurable` | App | Checkbox list of feature suggestions. |
| SET-TOOL-011 | MCP Server | `com.intellij.mcpserver.settings` (+ Exposed Tools `.filter`) | App | Enable MCP Server, port/SSE URL, client auto-configure buttons (Claude Desktop, Cursor, VS Code…), run shell commands without confirmation (off), Exposed Tools checklist. |
| SET-TOOL-012 | LSP Servers | `lsp.servers` | Proj | Registered LSP server definitions. |
| SET-TOOL-013 | XPath Viewer | `xpath.settings` | App | Highlighting colors, scroll to first match. |
| SET-TOOL-014 | CSV Formats | `database.data.csv.formats` | App | CSV/TSV dialect table. |
| SET-TOOL-015 | Server Certificates | see SET-AB-010 | | |
| SET-TOOL-016 | Settings Repository | `ics` | App | Upstream URL, *Merge* / *Reset to My Settings* / *Overwrite Remote* buttons; read-only sources table. Deprecated in favour of Backup and Sync. |
| SET-TOOL-017 | Database, Space, Code With Me, HTTP Client, AI Assistant | † commercial/plugin | — | Out of scope. |

Sources:
- https://www.jetbrains.com/help/idea/actions-on-save.html
- https://www.jetbrains.com/help/idea/settings-tools-terminal.html
- https://www.jetbrains.com/help/idea/settings-tools-external-tools.html
- https://www.jetbrains.com/help/idea/shared-indexes.html
- https://www.jetbrains.com/help/idea/settings-tools-startup-tasks.html
- https://www.jetbrains.com/help/idea/settings-tools-diff-and-merge.html
- https://www.jetbrains.com/help/idea/settings-web-browsers.html
- IntelliJ Community `TerminalBundle.properties`, `CodeInsightBundle.properties`, `DiffBundle.properties`, `IdeBundle.properties` (labels)

---

## 11. Backup and Sync, Advanced Settings

### SET-BAK-001 Backup and Sync — `settings.sync` — App

| Element | Behaviour |
|---|---|
| Not signed in | *Log in with JetBrains Account* / *Enable Backup and Sync* button [doc]. |
| First enable dialog | Category checkboxes [bundle]: **UI settings** (theme, editor font, toolbar, notifications), **Keymaps**, **Code settings** (code style, file types, live templates), **Plugins**, **Tools** (version control, debugger), **System settings**; scope radio: sync with *IntelliJ IDEA instances only* / *all JetBrains IDEs* [doc]. Buttons *Push Settings to Account* (when account has data) / *Get Settings from Account*. |
| Enabled state | Status line (last synced time), category checkboxes editable on the page, sub-lists per category (e.g. UI settings › Editor font), *Disable Backup and Sync…* → confirmation with checkbox *Remove data from JB account and disable for all IDEs* [doc]. |
| Plugin rules | Installed-on-both: state syncs; enabled on one only: installed on other; disabled on one: not installed; uninstalled on one: disabled on other [doc]. |
| History | *Show Sync History* link (restore previous snapshot) — inferred. |

### SET-ADV-001 Advanced Settings — `advanced.settings` — App
Full option list with ids and defaults is generated: see
[settings-tree.md › Advanced Settings page](settings-tree.md#advanced-settings-page). Page
behaviour: grouped list (group headings), each row a checkbox / combobox / spinner with
trailing unit label; local search field *Search advanced settings*; *Show modified only*
checkbox; per-row *Reset Setting* affordance when modified [bundle]; "Nothing found" empty
state.

Sources:
- https://www.jetbrains.com/help/idea/sharing-your-ide-settings.html
- https://www.jetbrains.com/help/idea/advanced-settings.html
- IntelliJ Community `SettingsSyncBundle.properties`, `ApplicationBundle.properties` (labels)

---

## 12. Project Structure dialog

Opened with **File \| Project Structure…** (`Ctrl+Alt+Shift+S` / `⌘;`), from the main-toolbar
project widget menu, or *Open Module Settings* (`F4`) on a module in the Project view.
Modal dialog, title *Project Structure* [bundle], configurable id unknown (`project.settings.display.name`
is only the label key). For **File \| New Projects Setup \| Structure for New Projects…** the
same dialog edits the default project (Project and SDKs pages only, inferred).

### SET-PSD-000 Dialog behaviour

| Aspect | Behaviour |
|---|---|
| Layout | Left sidebar with sections **Project Settings** (Project, Modules, Libraries, Facets, Artifacts) and **Platform Settings** (SDKs, Global Libraries), then **Problems** (with error count) [bundle "Project Settings", "Platform Settings"]. Middle column: element list with +/−/copy toolbar (for Modules, Libraries, Facets, Artifacts, SDKs, Global Libraries). Right: editor for the selected element with a banner title (e.g. *Module 'app'*, *SDK '21'*, *General Settings for Project 'x'*) [bundle]. |
| Navigation | Back/Forward arrows at top; links from one page jump to another (e.g. library usages → module). |
| Buttons | OK, Cancel, Apply (enabled when modified). Unsaved-changes prompt when navigating from a modified page in some flows [bundle *Unsaved Changes*]. |
| Build-tool projects | For Gradle/Maven-imported modules a notice says changes will be overwritten on the next sync (inferred); module names and dependencies are effectively read-only. |
| Search | Typing in the element list filters (speed search). |

### SET-PSD-001 Project

| Field | Control | Default |
|---|---|---|
| Name: | text | project directory name |
| SDK: | combobox (*Add SDK* → Download JDK…, Add JDK from disk…, detected SDKs) + *Edit* | as created |
| Language level: | combobox: *SDK default (N – …)*, each Java version 8…25 with descriptive suffix ("21 – Record patterns, pattern matching for switch"), *N (Preview) – …*, *X – Experimental features*; unsupported preview levels shown red [doc] | SDK default |
| Compiler output: | path | `<project>/out` |

### SET-PSD-002 Modules

| Element | Behaviour |
|---|---|
| Module tree | Modules (and nested facets); + New Module / Import Module; − Remove; Copy; *Show/Hide Module Groups* [bundle]; *Flatten Modules* [bundle]. Unloaded modules listed "N Unloaded Modules" [bundle]. |
| Name: | text field above tabs |
| **Sources** tab | *Language level:* combobox (*Project default* [bundle]); content roots on the right (*Add Content Root* [bundle], remove ×); tree of the content root with *Mark as:* toolbar **Sources**, **Tests**, **Resources**, **Test Resources**, **Excluded** [bundle "Mark as:"]; root properties dialog: *Package prefix:*, *For generated sources* [bundle]; *Exclude files:* pattern field ("Use ; to separate name patterns, * for any number of symbols, ? for one") [bundle]. |
| **Paths** tab | *Compiler Output*: radio *Inherit project compile output path* / *Use module compile output path* with *Output path:* and *Test output path:* [bundle]; *Exclude output paths* checkbox (on) [bundle]; JavaDoc paths list; *External Annotations:* list [bundle]. |
| **Dependencies** tab | *Module SDK:* combobox (*Project SDK* [bundle] / specific) + Edit; table columns *Export* (checkbox), dependency name (with icon), *Scope* combobox **Compile / Test / Runtime / Provided**; + menu *JARs or Directories…*, *Library…*, *Module Dependency…*; − ; Edit; Move Up/Down; *Analyze*; *Dependencies storage format:* combobox IntelliJ IDEA (.iml) / Eclipse (.classpath) [bundle]. `<Module source>` and SDK rows are fixed. |
| Facet pages | Selecting a facet node under a module shows its settings (e.g. Spring, Web, Kotlin). |

### SET-PSD-003 Libraries (project libraries)

| Element | Behaviour |
|---|---|
| List | + *New Project Library*: Java / From Maven… (Maven coordinates search dialog with *Download to:*, *Transitive dependencies*, *Sources*, *JavaDocs*, *Annotations* checkboxes [bundle]) / Kotlin/JS?; −; Copy; *Remove All Unused Libraries* [bundle]. |
| Editor | Name:, roots tree (Classes, Sources, JavaDocs, Annotations, Native Library Locations [bundle]); toolbar *+* (attach files), *Specify Documentation URL*, *Exclude*, *Remove*; for Maven libraries *Change Version…* / *Change Coordinates…* [bundle]; Broken path highlighting with tooltip [bundle]. |
| Usages | *Add to Dependencies…* [bundle] (choose modules). |

### SET-PSD-004 Facets
List of all facets across modules grouped by type; + adds a facet to a chosen module; selecting a type shows *Defaults* [bundle] (framework detection settings: *Enable framework detection*, *Exclude from detection:* list [bundle]).

### SET-PSD-005 Artifacts

| Field | Control |
|---|---|
| List | + *JAR* (Empty / From modules with dependencies…), *Web Application: Exploded / Archive*, *JavaFX Application*, *Other*; −; *Add Copy of* [bundle] |
| Name: / Type: | text / combobox [bundle] |
| Output directory: | path [bundle "Output directory:"] |
| Include in project build | checkbox [bundle] (off) |
| Output Layout tab | tree of output (`<output root>`, archives, directories, module output, library files) with toolbar (create directory, archive, add copy of, remove, sort); right side *Available Elements* [bundle] with double-click to place; *Show content of elements* checkbox |
| Pre-processing / Post-processing tabs | *Run Ant target* checkbox + target chooser |
| Validation | Problems shown at the bottom with *Fix…* |

### SET-PSD-006 SDKs (Platform Settings)

| Element | Behaviour |
|---|---|
| List | + *Download JDK…* (vendor/version/location dialog), *Add JDK from disk…*, *Add Kotlin SDK*?, detected JDKs; −; Copy. |
| Name: | text |
| JDK home path: | path ("<type> home path:") [bundle] |
| Tabs | **Classpath**, **Sourcepath**, **Annotations**, **Documentation Paths** [bundle] each a root list with +/−/*Specify URL…* |

### SET-PSD-007 Global Libraries
Same editor as project libraries but stored at application level; *Level:* indicator [bundle]. Usable from any project.

### SET-PSD-008 Problems
List of configuration problems (missing SDK, broken library paths, unused libraries, invalid artifacts), each with severity icon and *Fix* drop-down (e.g. *Remove Invalid Roots* [bundle], *Remove Library*, *Add to Dependencies…*). Count shown in the sidebar node.

Sources:
- https://www.jetbrains.com/help/idea/project-settings-and-structure.html
- https://www.jetbrains.com/help/idea/creating-and-managing-modules.html
- https://www.jetbrains.com/help/idea/library.html
- https://www.jetbrains.com/help/idea/sdk.html
- https://www.jetbrains.com/help/idea/working-with-artifacts.html
- https://www.jetbrains.com/help/idea/facet-page.html
- IntelliJ Community `JavaUiBundle.properties`, `ProjectBundle.properties` (labels)

---

## Open questions

1. **Settings in a tab.** 2026.2 bundles contain "Settings tab" strings. Is there a user-visible
   option (Advanced Settings? registry?) that opens Settings as an editor tab instead of a
   dialog, and is it the default? Verify in a clean install.
2. **Modified-page marker and Revert All.** Confirm the exact visual treatment of modified tree
   nodes in Islands themes and where *Revert All* is surfaced.
3. **Project-level marker text.** Confirm wording (*For current project* vs *This page's settings
   apply to the current project*) and whether a marker also appears in the tree.
4. **Defaults** marked without `[doc]`/`[adv]`/`[bundle]` (the majority) are inferred and need
   verification, especially: Open project in (Ask vs New window), Main menu placement,
   Show tool window names, Editor tabs placement mode, Breadcrumbs placement, Reader Mode
   defaults, Compiler shared heap size, Gradle *Download external annotations*, Git *Fetch
   remote changes automatically*, Debugger *Remove breakpoint* mode.
5. **Configurable ids** marked "(inferred)" — in particular Editor › General subpages
   (Appearance, Code Completion, Code Folding, Editor Tabs, Gutter Icons, Postfix Completion,
   Smart Keys), Version Control subpages (Commit, Confirmation, Changelists, Directory
   Mappings, Issue Navigation, Shelf, File Status Colors), Debugger subpages, Font, Code Style
   › Java, Shared Indexes and the Project Structure dialog itself.
6. **Gutter group placement.** Are *Highlight modified lines in the gutter* and related options on
   Editor › General (bundle group *Gutter*) or on Version Control? Verify.
7. **Actions on Save trigger.** Do actions run on autosave (idle / frame deactivation) or only on
   explicit save in 2026.2?
8. **Code Completion Java/Kotlin sections**, Inline Completion page contents, Inlay Hints Java
   provider option names and default states, Postfix *Show postfix completions as command
   completions* default.
9. **Compiler Cache** and **Run Widget** pages: options are undocumented.
10. **Maven Importing** in 2026.2: whether "Enable fast import" and "Use M2E compatibility" are
    visible, and exact defaults of automatic downloads.
11. **Terminal** defaults for *Use application title as tab name*, *Prompt style*, completion
    shortcuts.
12. **Kotlin page** content after K1 removal (is there any K2 toggle left?).
13. **Process Elevation** and **Data Sharing** exact labels.
14. **Project Structure** behaviour for Gradle-synced modules: exact warning text and which fields
    are read-only.
15. **Menus and Toolbars** root list in 2026.2 New UI (Main Toolbar sections, run widget group).

## Coverage summary

- Settings pages with an ID: ~150 (including grouped rows for minor pages), of which ~95 have
  per-option tables; Project Structure: 9 entries (dialog + 8 nodes).
- Options listed: roughly 800 table rows describing individual controls or control groups (plus the generated Advanced Settings rows, referenced not repeated).
- Largest known gaps: Color Scheme element trees (per language), Inspections catalogue (by
  design a separate dataset), commercial-only pages, and the inferred defaults listed above.
