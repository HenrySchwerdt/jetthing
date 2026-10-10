# Plugin system evidence (API 1)

Captured under Xvfb 1600×1000, `JETTHING_RENDERER=glow`. The build used
`cargo build -p jetthing-app --features example-plugins` with an empty
`JETTHING_CONFIG_DIR`.

| File | Shows |
|---|---|
| `hello-toolwindow-and-tab.png` | Example plugin: "H" button on the right stripe with the Hello tool window open, a plugin editor tab ("Hello") next to a file tab, the plugin notification balloon and the plugin status widget. "Open Hello Tab" ran from Find Action, where plugin commands are listed with category and shortcut. |
| `plugins-dialog.png` | File ▸ Plugins…: installed plugins with enable checkboxes, details (version, vendor, kind, status, source, permissions, contributions, standalone command) and the plugin's settings page rendered from its declarative fields. |
| `standalone-app.png` | `jetthing-app --app dev.jetthing.hello`: only the plugin's app view, with title bar, window controls, status text, status widget and notification. |

Also observed in the same session:

- Unchecking Hello removed its stripe button, status widget and commands at once, and
  wrote `{"disabled":["dev.jetthing.hello"]}` to `plugins.json`. Re-checking restored them.
- `jetthing-app --app no.such` printed
  `jetthing: unknown plugin `no.such`; plugins with an app view: dev.jetthing.hello`
  and exited with code 2.
- Alt-based chords could not be sent through xdotool in this Xvfb setup; the built-in
  Alt+1 failed the same way. Ctrl chords work. Keybinding mapping is covered by unit tests
  (`plugin_ui::tests`, `contributions::tests`).

Reference: the IntelliJ Plugins page (scene `24-plugins`) is the target for the next slice
(Settings ▸ Plugins with Marketplace/Installed tabs). This dialog is an interim
manager and does not claim parity with it.
