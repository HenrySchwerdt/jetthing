# Editor splits and the preview tab

Captured 2026-10-11 on Linux under Xvfb (1600x1000, `JETTHING_RENDERER=glow`, debug build),
driven through the control socket (`jetthing-cli ui`). Sample project: `src/App.java`,
`src/Util.java`, `README.md`, `build.gradle.kts`.

| File | Shows |
|---|---|
| `audit-v0.0.5/04-fixed-preview-tab.png` | single click on `App.java` in the Project tree opens an italic preview tab (AUD-034/IJ-ED-161); the next single click replaces it, a double click or an edit keeps it |
| `audit-v0.0.5/04-fixed-tab-menu-splits.png` | tab context menu with Split Right, Split Down, Move to Opposite Group, Change Splitter Orientation, Unsplit, Unsplit All, Keep Tab Open (disabled entries are shown dimmed) |
| `audit-v0.0.5/04-fixed-editor-splits.png` | three groups (`App.java` over `Util.java` on the left, `README.md` on the right), each with its own tab strip; the focused group (blue divider) is the one whose tab the status bar and title show |
| `audit-v0.0.5/07-fixed-window-editor-tabs.png` | Window > Editor Tabs lists Split Right / Split Down / Change Splitter Orientation / Unsplit / Unsplit All, wired through `ACTION_COMMANDS` |
| `audit-v0.0.5/19-fixed-open-in-right-split.png` | Project tree context menu with "Open in Right Split  Shift+Enter" |
| `audit-v0.0.5/05-fixed-search-everywhere-right-split.png` | Search Everywhere: typing `build.gr` and pressing Shift+Enter opened `build.gradle.kts` in the right group |

Behavior notes:

- A tab lives in exactly one group, so Split Right / Down *move* the tab into the new group
  (an empty group when it is the group's only tab). Showing one document in two groups is
  not implemented.
- Open in Right Split targets the group after the focused one, wrapping around, and
  creates a split to the right when the editor is not split.
- Dragging a tab onto the outer quarter of an editor group splits at that edge; dropping on
  another group moves the tab. The control socket has no drag primitive, so this path is
  covered by `editor.splitAt` / `editor.moveToGroup` tests in `jetthing-core` and the pure
  `jetthing_split::drop_edge` test.
- Splits are stored in `.jetthing/workspace.json` (`splits`) and restored with the tabs.
