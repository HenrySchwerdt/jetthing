# Plugin system evidence (API 1)

Captured under Xvfb 1600×1000, `JETTHING_RENDERER=glow`. The build used
`cargo build -p jetthing-app --features example-plugins` with an empty
`JETTHING_CONFIG_DIR`.

| File | Shows |
|---|---|
| `hello-toolwindow-and-tab.png` | Example plugin: "H" button on the right stripe with the Hello tool window open, a plugin editor tab ("Hello") next to a file tab, the plugin notification balloon and the plugin status widget. "Open Hello Tab" ran from Find Action, where plugin commands are listed with category and shortcut. |
| `24-plugins-installed.png` | File ▸ Plugins… (`plugin.manager.open`) right after ⚙ ▸ Install Plugin from Disk… installed `darcula-plus.jtplugin` (packed with `cargo xtask pack-plugin`): Marketplace / Installed tabs with count badge and gear; search field; groups "Downloaded (1 of 1 enabled)", "Project" and "Bundled" with Disable all links; rows with icon, bold name, version and vendor, checkbox; details with vendor, homepage link, Disable / Uninstall, version, Overview / Additional Info tabs. |
| `24-plugins-settings-tab.png` | Same page with the bundled Hello plugin selected: the Settings tab renders its declarative settings page (text and boolean fields). |
| `24-plugins-marketplace.png` | Marketplace tab: no marketplace is configured yet, so an empty state offers Install Plugin from Disk…. |
| `standalone-app.png` | `jetthing-app --app dev.jetthing.hello`: only the plugin's app view, with title bar, window controls, status text, status widget and notification. |

Also observed in the same session:

- Uninstall removed `<config>/plugins/com.example.darcula-plus/` and the entry; the status bar said
  "Uninstalled plugin com.example.darcula-plus".
- Unchecking Hello removed its stripe button, status widget and commands at once, and
  wrote `{"disabled":["dev.jetthing.hello"]}` to `plugins.json`. Re-checking restored them.
- `jetthing-app --app no.such` printed
  `jetthing: unknown plugin `no.such`; plugins with an app view: dev.jetthing.hello`
  and exited with code 2.
- Alt-based chords could not be sent through xdotool in this Xvfb setup; the built-in
  Alt+1 failed the same way. Ctrl chords work. Keybinding mapping is covered by unit tests
  (`plugin_ui::tests`, `contributions::tests`).

## Comparison with reference scene 24-plugins

Reference: `reference/intellij-community/java-gradle/24-plugins.png` (Marketplace tab) and
`reference/intellij-community/bazel/b07-plugin-installed.png` (Installed tab).

Matches: header "Plugins" with Marketplace / Installed tabs, an Installed count badge and a gear menu
(Install Plugin from Disk…); search hint "Type / to see options"; grouped list with grey group headers,
"Downloaded (x of y enabled)" and "Disable all" links; 64 px rows with icon, bold name, version/vendor
line and checkbox; details pane with name, vendor, homepage link, Enable/Disable button, version and
Overview / Additional Info tabs; Uninstall for downloaded plugins.

Differences:
- The page is a standalone window, not a page inside the Settings dialog tree. The Settings dialog
  (#55) has no custom-page hook yet, so the page moves there once one exists.
- Changes apply immediately: there is a Close button instead of OK / Cancel / Apply.
- The Marketplace tab has no catalog, so there are no Suggested / Staff Picks sections, downloads,
  ratings, What's New or Reviews tabs, and no screenshots carousel.
- Icons are generated initials (no plugin artwork), there is no "Bundled plugins updates" section,
  and Disable is a plain button rather than a split button.
- Uninstalling happens immediately; there is no "restart required" state, because nothing in the
  current plugin kinds needs a restart.
