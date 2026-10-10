# IntelliJ IDEA parity: Settings dialog

Pages of the Settings dialog (Ctrl+Alt+S) and their observable effects. A row is only
Partial once Jetthing persists and applies the setting; hard-coded defaults do not count.

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
| IJ-SET-001 | [settings-preferences-dialog](https://www.jetbrains.com/help/idea/settings-preferences-dialog.html) | Settings dialog with category tree and search | Missing | 5 | — | — | not started |
| IJ-SET-002 | [configuring-project-and-ide-settings](https://www.jetbrains.com/help/idea/configuring-project-and-ide-settings.html) | IDE-level vs project-level settings scopes and persistence | Missing | 5 | — | — | not started |
| IJ-SET-003 | [sharing-your-ide-settings](https://www.jetbrains.com/help/idea/sharing-your-ide-settings.html) | Settings Sync / export / import | Missing | 2 | — | — | not started |
| IJ-SET-004 | [settings-preferences-dialog](https://www.jetbrains.com/help/idea/settings-preferences-dialog.html) | Reset to defaults / modified-setting markers | Missing | 1 | — | — | not started |
| IJ-SET-010 | [user-interface-themes](https://www.jetbrains.com/help/idea/user-interface-themes.html) | Appearance: theme selection | Missing | 3 | — | — | not started |
| IJ-SET-011 | [appearance-and-behavior](https://www.jetbrains.com/help/idea/appearance-and-behavior.html) | Appearance: UI font and size, compact mode | Missing | 2 | — | — | not started |
| IJ-SET-012 | [appearance-and-behavior](https://www.jetbrains.com/help/idea/appearance-and-behavior.html) | Appearance: tool window options (show stripes, widescreen layout) | Missing | 1 | — | — | not started |
| IJ-SET-013 | [new-ui](https://www.jetbrains.com/help/idea/new-ui.html) | New UI options (compact mode, project gradients, toolbar layout) | Missing | 1 | — | — | not started |
| IJ-SET-014 | [customize-actions-menus-and-toolbars](https://www.jetbrains.com/help/idea/customize-actions-menus-and-toolbars.html) | Menus and Toolbars customization | Missing | 1 | — | — | not started |
| IJ-SET-015 | [system-settings](https://www.jetbrains.com/help/idea/system-settings.html) | System Settings: startup, project opening, autosave, confirmations | Missing | 2 | — | — | not started |
| IJ-SET-016 | [settings-http-proxy](https://www.jetbrains.com/help/idea/settings-http-proxy.html) | System Settings: HTTP proxy | Missing | 1 | — | — | not started |
| IJ-SET-017 | [system-settings](https://www.jetbrains.com/help/idea/system-settings.html) | System Settings: passwords storage | Missing | 1 | — | — | not started |
| IJ-SET-018 | [notifications](https://www.jetbrains.com/help/idea/notifications.html) | Notifications settings | Missing | 1 | — | — | not started |
| IJ-SET-019 | [settings-file-colors](https://www.jetbrains.com/help/idea/settings-file-colors.html) | File Colors and Scopes | Missing | 1 | — | — | not started |
| IJ-SET-020 | [path-variables](https://www.jetbrains.com/help/idea/path-variables.html) | Path Variables | Missing | 1 | — | — | not started |
| IJ-SET-021 | [language-and-region](https://www.jetbrains.com/help/idea/language-and-region.html) | Language and Region (UI language) | Missing | 1 | — | — | not started |
| IJ-SET-030 | [configuring-keyboard-and-mouse-shortcuts](https://www.jetbrains.com/help/idea/configuring-keyboard-and-mouse-shortcuts.html) | Keymap: predefined keymaps (Windows, macOS, Linux variants) | Partial | 5 | — | command registry unit tests | commands carry a single default shortcut label; no keymap switching |
| IJ-SET-031 | [configuring-keyboard-and-mouse-shortcuts](https://www.jetbrains.com/help/idea/configuring-keyboard-and-mouse-shortcuts.html) | Keymap: add / remove keyboard and mouse shortcuts, conflict detection | Missing | 3 | — | — | not started |
| IJ-SET-032 | [configuring-keyboard-and-mouse-shortcuts](https://www.jetbrains.com/help/idea/configuring-keyboard-and-mouse-shortcuts.html) | Keymap: find action by shortcut | Missing | 1 | — | — | not started |
| IJ-SET-040 | [settings-editor-general](https://www.jetbrains.com/help/idea/settings-editor-general.html) | Editor > General (caret, scrolling, soft wraps, strip trailing spaces) | Missing | 3 | — | — | not started |
| IJ-SET-041 | [settings-editor-appearance](https://www.jetbrains.com/help/idea/settings-editor-appearance.html) | Editor > General > Appearance (line numbers, whitespace, indent guides) | Missing | 2 | — | — | not started |
| IJ-SET-042 | [settings-editor-tabs](https://www.jetbrains.com/help/idea/settings-editor-tabs.html) | Editor > General > Editor Tabs (placement, limits, closing policy) | Missing | 2 | — | — | not started |
| IJ-SET-043 | [settings-code-completion](https://www.jetbrains.com/help/idea/settings-code-completion.html) | Editor > General > Code Completion options | Missing | 2 | — | — | not started |
| IJ-SET-044 | [working-with-source-code](https://www.jetbrains.com/help/idea/working-with-source-code.html#code_folding) | Editor > General > Code Folding defaults | Missing | 1 | — | — | not started |
| IJ-SET-045 | [settings-postfix-completion](https://www.jetbrains.com/help/idea/settings-postfix-completion.html) | Editor > General > Postfix Completion templates | Missing | 1 | — | — | not started |
| IJ-SET-046 | [sticky-lines](https://www.jetbrains.com/help/idea/sticky-lines.html) | Editor > General > Sticky Lines | Missing | 1 | — | — | not started |
| IJ-SET-047 | [configuring-colors-and-fonts](https://www.jetbrains.com/help/idea/configuring-colors-and-fonts.html) | Editor > Font (family, size, ligatures, line height) | Missing | 3 | — | — | not started |
| IJ-SET-048 | [configuring-colors-and-fonts](https://www.jetbrains.com/help/idea/configuring-colors-and-fonts.html) | Editor > Color Scheme (per-language token colors) | Missing | 3 | — | — | one built-in token palette |
| IJ-SET-049 | [configuring-code-style](https://www.jetbrains.com/help/idea/configuring-code-style.html) | Editor > Code Style (per-language indent, wrapping, blank lines) | Missing | 3 | — | — | not started |
| IJ-SET-050 | [editorconfig](https://www.jetbrains.com/help/idea/editorconfig.html) | EditorConfig support | Missing | 3 | — | — | not started |
| IJ-SET-051 | [configuring-inspection-severities](https://www.jetbrains.com/help/idea/configuring-inspection-severities.html) | Editor > Inspections (profiles, severities, scopes) | Missing | 3 | — | — | not started |
| IJ-SET-052 | [using-file-and-code-templates](https://www.jetbrains.com/help/idea/using-file-and-code-templates.html) | Editor > File and Code Templates | Missing | 2 | — | — | not started |
| IJ-SET-053 | [encoding](https://www.jetbrains.com/help/idea/encoding.html) | Editor > File Encodings | Missing | 1 | — | — | not started |
| IJ-SET-054 | [using-live-templates](https://www.jetbrains.com/help/idea/using-live-templates.html) | Editor > Live Templates | Missing | 2 | — | — | not started |
| IJ-SET-055 | [file-type-associations](https://www.jetbrains.com/help/idea/file-type-associations.html) | Editor > File Types | Missing | 2 | — | — | not started |
| IJ-SET-056 | [inlay-hints](https://www.jetbrains.com/help/idea/inlay-hints.html) | Editor > Inlay Hints | Missing | 2 | — | — | not started |
| IJ-SET-057 | [settings-auto-import](https://www.jetbrains.com/help/idea/settings-auto-import.html) | Editor > General > Auto Import | Missing | 2 | — | — | not started |
| IJ-SET-058 | [spellchecking](https://www.jetbrains.com/help/idea/spellchecking.html) | Editor > Natural Languages > Spelling | Missing | 1 | — | — | not started |
| IJ-SET-059 | [intention-actions](https://www.jetbrains.com/help/idea/intention-actions.html) | Editor > Intentions (enable/disable) | Missing | 1 | — | — | not started |
| IJ-SET-060 | [using-todo](https://www.jetbrains.com/help/idea/using-todo.html) | Editor > TODO patterns and filters | Missing | 1 | — | — | not started |
| IJ-SET-061 | [reader-mode](https://www.jetbrains.com/help/idea/reader-mode.html) | Editor > Reader Mode | Missing | 1 | — | — | not started |
| IJ-SET-070 | [managing-plugins](https://www.jetbrains.com/help/idea/managing-plugins.html) | Plugins: installed plugin list and enable/disable | Partial | 3 | `plugin.list`, `plugin.enable`, `plugin.disable`, `plugin.setSetting` | registry/store and host enable/disable tests | interim File ▸ Plugins… dialog lists plugins with enable checkboxes, details, permissions and plugin settings; changes apply live and persist per user; not yet the scene-24 Settings page; `evidence/plugins/NOTES.md` |
| IJ-SET-071 | [managing-plugins](https://www.jetbrains.com/help/idea/managing-plugins.html) | Plugins: Marketplace browse and install | Missing | 3 | — | — | not started |
| IJ-SET-072 | [managing-plugins](https://www.jetbrains.com/help/idea/managing-plugins.html) | Plugins: install from disk, update, custom repositories | Missing | 2 | — | — | not started |
| IJ-SET-080 | [settings-version-control](https://www.jetbrains.com/help/idea/settings-version-control.html) | Version Control settings root | Missing | 1 | — | — | not started |
| IJ-SET-081 | [settings-version-control-directory-mappings](https://www.jetbrains.com/help/idea/settings-version-control-directory-mappings.html) | Directory Mappings (VCS roots) | Partial | 2 | `git.refresh` | real repository integration test | single auto-detected Git root; not configurable |
| IJ-SET-082 | [settings-version-control-git](https://www.jetbrains.com/help/idea/settings-version-control-git.html) | Git settings (executable path, update method, push options) | Missing | 2 | — | — | not started |
| IJ-SET-083 | [settings-version-control-github](https://www.jetbrains.com/help/idea/settings-version-control-github.html) | GitHub accounts | Missing | 2 | — | — | not started |
| IJ-SET-084 | [settings-version-control-gitlab](https://www.jetbrains.com/help/idea/settings-version-control-gitlab.html) | GitLab accounts | Missing | 1 | — | — | not started |
| IJ-SET-085 | [commit-dialog](https://www.jetbrains.com/help/idea/commit-dialog.html) | Commit settings (checks before commit, non-modal commit) | Missing | 1 | — | — | not started |
| IJ-SET-086 | [settings-version-control-confirmation](https://www.jetbrains.com/help/idea/settings-version-control-confirmation.html) | Confirmation settings (add files, delete files) | Missing | 1 | — | — | not started |
| IJ-SET-087 | [settings-version-control-log](https://www.jetbrains.com/help/idea/settings-version-control-log.html) | Log settings | Missing | 1 | — | — | not started |
| IJ-SET-088 | [settings-version-control-changelist-conflicts](https://www.jetbrains.com/help/idea/settings-version-control-changelist-conflicts.html) | Changelists settings | Missing | 1 | — | — | not started |
| IJ-SET-090 | [configure-compiler-settings](https://www.jetbrains.com/help/idea/configure-compiler-settings.html) | Build > Compiler settings | Missing | 2 | — | — | not started |
| IJ-SET-091 | [annotation-processors-support](https://www.jetbrains.com/help/idea/annotation-processors-support.html) | Build > Compiler > Annotation Processors | Missing | 2 | — | — | not started |
| IJ-SET-092 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Build Tools > Gradle settings (JVM, build and run using, offline) | Missing | 3 | — | — | not started |
| IJ-SET-093 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Build Tools > Maven settings (home, settings.xml, runner, importing) | Missing | 3 | — | — | not started |
| IJ-SET-094 | [settings-debugger](https://www.jetbrains.com/help/idea/settings-debugger.html) | Debugger settings (stepping filters, data views, HotSwap) | Missing | 2 | — | — | not started |
| IJ-SET-095 | [run-targets](https://www.jetbrains.com/help/idea/run-targets.html) | Run targets (SSH, Docker, WSL) | Missing | 1 | — | — | not started |
| IJ-SET-096 | [run-tool-window](https://www.jetbrains.com/help/idea/run-tool-window.html) | Console settings (fold patterns, cycle buffer) | Missing | 1 | — | — | not started |
| IJ-SET-097 | [coverage-settings](https://www.jetbrains.com/help/idea/coverage-settings.html) | Coverage settings | Missing | 1 | — | — | not started |
| IJ-SET-098 | [deploying-applications](https://www.jetbrains.com/help/idea/deploying-applications.html) | [Ultimate] Deployment (SFTP/FTP servers, mappings) | Missing | 1 | — | — | not started |
| IJ-SET-099 | [docker](https://www.jetbrains.com/help/idea/docker.html) | Docker connection settings | Missing | 1 | — | — | not started |
| IJ-SET-100 | [settings-languages-and-frameworks](https://www.jetbrains.com/help/idea/settings-languages-and-frameworks.html) | Languages & Frameworks settings root | Missing | 1 | — | — | not started |
| IJ-SET-101 | [terminal-emulator](https://www.jetbrains.com/help/idea/terminal-emulator.html) | Tools > Terminal settings (shell path, environment, start directory) | Missing | 2 | — | — | shell comes from environment; not configurable |
| IJ-SET-102 | [settings-tools-external-tools](https://www.jetbrains.com/help/idea/settings-tools-external-tools.html) | Tools > External Tools | Missing | 1 | — | — | not started |
| IJ-SET-103 | [settings-tools-server-certificates](https://www.jetbrains.com/help/idea/settings-tools-server-certificates.html) | Tools > Server Certificates | Missing | 1 | — | — | not started |
| IJ-SET-104 | [settings-tools-startup-tasks](https://www.jetbrains.com/help/idea/settings-tools-startup-tasks.html) | Tools > Startup Tasks | Missing | 1 | — | — | not started |
| IJ-SET-105 | [saving-and-reverting-changes](https://www.jetbrains.com/help/idea/saving-and-reverting-changes.html) | Tools > Actions on Save (reformat, optimize imports, rearrange) | Missing | 3 | — | — | not started |
| IJ-SET-106 | [intellij-idea-unified-faq](https://lp.jetbrains.com/intellij-idea-unified-faq/) | Language server settings (servers per language) | Missing | 2 | — | — | rust-analyzer is auto-detected; no settings UI |
| IJ-SET-107 | [sdk](https://www.jetbrains.com/help/idea/sdk.html) | Project Structure > SDKs | Missing | 3 | — | — | not started |
| IJ-SET-108 | [configure-modules](https://www.jetbrains.com/help/idea/configure-modules.html) | Project Structure > Modules (sources, resources, tests, excluded) | Missing | 3 | — | — | not started |
| IJ-SET-109 | [library](https://www.jetbrains.com/help/idea/library.html) | Project Structure > Libraries | Missing | 2 | — | — | not started |
| IJ-SET-110 | [artifacts](https://www.jetbrains.com/help/idea/artifacts.html) | Project Structure > Artifacts | Missing | 1 | — | — | not started |
| IJ-SET-111 | [facet-page](https://www.jetbrains.com/help/idea/facet-page.html) | Project Structure > Facets | Missing | 1 | — | — | not started |
| IJ-SET-112 | [switching-boot-jdk](https://www.jetbrains.com/help/idea/switching-boot-jdk.html) | Choose Boot Java Runtime for the IDE | Missing | 1 | — | — | not started |
| IJ-SET-113 | [increasing-memory-heap](https://www.jetbrains.com/help/idea/increasing-memory-heap.html) | Change IDE memory settings | Missing | 1 | — | — | not started |
| IJ-SET-114 | [settings-scopes](https://www.jetbrains.com/help/idea/settings-scopes.html) | Scopes (named file sets for search, inspections, colors) | Missing | 2 | — | — | not started |
| IJ-SET-115 | [configuring-project-and-ide-settings](https://www.jetbrains.com/help/idea/configuring-project-and-ide-settings.html) | Advanced Settings page | Missing | 1 | — | — | not started |
