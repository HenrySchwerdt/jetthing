# Main menu (scene 07)

Reference: `reference/intellij-community/java-gradle/07-main-menu.png`. Jetthing:
`jetthing/menus/07-file-menu.png`, `07-edit-menu.png`, `07-code-menu.png` (Linux, Xvfb
1600×1000, XWin keymap).

## Steps

1. Click the hamburger. As in the New UI, the titles File, Edit, View, Navigate, Code,
   Refactor, Build, Run, Tools, Git, Window and Help expand inline in the main toolbar, in
   place of the project and VCS widgets.
2. Click File → `07-file-menu.png`. Hovering another title switches menus →
   `07-code-menu.png`, `07-edit-menu.png`.
3. Esc, a click outside, or running an item collapses the bar back to the hamburger.

## Matches the reference

- The File menu has the reference order and separators: New ▸, Open…, Recent Projects ▸,
  Close Project | Settings… (Ctrl+Alt+S), Project Structure… (Ctrl+Alt+Shift+S),
  File Properties ▸ | Local History ▸ | Save All (Ctrl+S), Reload All from Disk
  (Ctrl+Alt+Y), Invalidate Caches… | Manage IDE Settings ▸ … | Export ▸, Print… |
  Power Save Mode | Exit.
- The Code menu, which was an empty popup before (`jetthing/NOTES.md`, 07), now lists
  the reference structure. Code Completion ▸, Comment with Line Comment and Move Line
  Up/Down are enabled.
- Shortcuts come from the active keymap.

## Still differs

- No menu icons, no underlined mnemonics.
- The reference build shows "Repair IDE" and a "New Projects Setup" submenu. The spec
  data (build 262) inlines Settings for New Projects… and the run-configuration
  templates instead.
- Entries Jetthing does not implement are shown disabled with the tooltip "Not available
  in Jetthing yet". The reference hides some of them at runtime. Only the five File
  entries hidden in the reference screenshot are hidden.
- Jetthing-only commands (Refresh Plugins, Run, Stop, Refresh Git Status, terminal
  actions, update actions) are appended after a separator in their menu, because the
  reference has no matching action or registers it at runtime (Run).
