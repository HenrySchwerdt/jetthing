# Git evidence

Captured on Linux under Xvfb (1600×1000, `JETTHING_RENDERER=glow`,
`JETTHING_NO_UPDATE_CHECK=1`) with a `java-gradle` sample repository whose
`App.java` matches the reference scene: one commit, then the line
`// Greets each build tool by name` inserted before the `for` loop.

| File | Steps | Reference scene |
|---|---|---|
| `13-diff-side-by-side.png` | Alt+0, double-click `App.java` in the Commit window | 13 Diff |
| `13b-diff-external-change-words.png` | With the diff open, run `sed -i 's/"Hello, "/"Hi there, "/' App.java` and `echo new > NOTES.txt` in another process; wait ~2 s, no clicks | 12 Commit / 13 Diff |
| `13c-diff-unified.png` | F7, then choose “Unified viewer” in the toolbar | 13 Diff (unified mode) |
| `12c-tree-autorefresh.png` | Change `settings.gradle` outside the IDE, Alt+1 | issue #47 bug 3 |

Compared with reference scene 13 (IntelliJ IDEA, `java-gradle`):

- Same structure: editor tab for the diff, toolbar with previous/next arrows and a
  viewer dropdown, “N difference(s)” on the right, base revision hash and path
  above the left pane, “Current version” above the right pane, two line-number
  columns with the connector between them, green inserted block, error-stripe mark.
- Both panes use the editor's tree-sitter highlighting (Java here).
- Differences: no whitespace/highlighting dropdowns, settings gear or help, no include-in-commit
  checkbox in the gutter, the tab is titled `App.java (Diff)` rather than
  `Commit`, and the diff is read-only.

Background refresh: `git --no-optional-locks status` every 2 s on a worker
thread and immediately when the window regains focus, so it never takes
`index.lock` away from the user's own git commands.
