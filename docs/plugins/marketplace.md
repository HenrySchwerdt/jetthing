# JetBrains Marketplace

The **Marketplace** tab of the Plugins page (File ▸ Plugins…) browses
[plugins.jetbrains.com](https://plugins.jetbrains.com) through its public, unauthenticated REST API
(`jetthing-marketplace`):

| Endpoint | Used for |
|---|---|
| `GET /api/searchPlugins?search=…&max=…` (`orderBy=downloads` for an empty query) | result list |
| `GET /api/plugins/{id}` | details pane |
| `GET /api/plugins/{id}/updates?size=1` | latest version on install |
| `GET /files/{file}` | download; only when you click **Install**, capped at 64 MiB |

`JETTHING_MARKETPLACE_URL` overrides the base URL (for mirrors).

## What can be installed

Most Marketplace plugins are JVM code written against the IntelliJ Platform (PSI, VFS, Swing,
extension points). Jetthing cannot run them; see [intellij-compat.md](intellij-compat.md).
On **Install**, Jetthing downloads the plugin, reads `META-INF/plugin.xml` from its jars and
checks for compiled classes:

| Plugin content | Result |
|---|---|
| `themeProvider` → `*.theme.json` (plus the `editorScheme` it references) | converted: UI theme (+ color scheme) |
| `bundledColorScheme` → scheme XML / `.icls` | converted: editor color scheme |
| `bundledKeymap` | converted: keymap |
| `defaultLiveTemplates` | converted: live templates |
| `*.tmLanguage` / `*.tmLanguage.json` resources | converted: TextMate grammar |
| any `.class` file, native library, `<actions>`, listeners, components or code extension points (`toolWindow`, `applicationService`, `platform.lsp.serverSupportProvider`, …) | **"Requires IntelliJ Platform (JVM) — not supported"**, with the classes and extension points that need code |

A plugin whose language support comes from a language server (LSP) still implements that through
JVM code. Jetthing reports it as unsupported and suggests configuring the same server in its own
language server settings.

Conversion produces a declarative Jetthing plugin (`kind = "declarative"`) that is installed
through the normal `.jtplugin` path into `<config>/plugins/<xmlId>/`:

```toml
id = "com.markskelton.one-dark-theme"
name = "One Dark Theme"
version = "6.2.5"
kind = "declarative"
capabilities = ["jetbrains-marketplace", "theme", "color-scheme"]
homepage = "https://github.com/one-dark/jetbrains-one-dark-theme"

[[assets]]
kind = "theme"
path = "assets/themes/one_dark.theme.json"
name = "One Dark"

[[assets]]
kind = "color-scheme"
path = "assets/schemes/one_dark.xml"
name = "One Dark"
```

The description keeps the Marketplace link and the plugin's license URL. Converted plugins can be
disabled and uninstalled like any other plugin. Each plugin keeps its own license; Jetthing only
reformats files the author published.

## Using the assets

`PluginRegistry::assets(AssetKind::Theme | ColorScheme | Keymap | LiveTemplates | TextmateBundle)`
lists the asset files of active plugins with their display names. The theme, color scheme and
keymap pickers build on this list.

## Not yet

- Applying a converted theme or color scheme to the workbench and editor. This needs the theme
  engine and color scheme switching; the assets are installed and listed now.
- Update checks for converted plugins, paid plugins, and custom plugin repositories.
- Plugin icons and screenshots: the list shows generated initials instead of Marketplace artwork.
