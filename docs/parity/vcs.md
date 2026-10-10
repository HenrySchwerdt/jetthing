# Version control (Git) parity

Rows follow the `PLATFORM-VCS-*` ids of `docs/spec/features/platform.md`. Reference
scenes: 12 Commit, 13 Diff, 14 Git log (`docs/parity/evidence/reference/`). Jetthing
captures live in `docs/parity/evidence/git/` (see `evidence/git/README.md`).

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| PLATFORM-VCS-003 | IntelliJ Platform | Commit tool window | Partial | `toolwindow.activate.commit`, `git.commit`, `git.showDiff` | `jetthing-git` commit tests, `vcs::tests` | changes/unversioned groups with checkboxes, message, commit of selected paths; double-click opens the diff; refreshes automatically after external changes (2 s background `git status --no-optional-locks`, plus on window focus); no changelists, amend, commit checks or per-hunk inclusion; `evidence/git/13b-diff-external-change-words.png` |
| PLATFORM-VCS-004 | IntelliJ Platform | Commit and Push | Missing | — | — | — |
| PLATFORM-VCS-005 | IntelliJ Platform | Amend commit | Missing | — | — | — |
| PLATFORM-VCS-010 | IntelliJ Platform | Unversioned files | Partial | `git.stage`, `git.commit` | `jetthing-git` status tests | Unversioned Files group, unchecked by default; files created outside the IDE appear in the group and in the project tree without a manual refresh (`evidence/git/12c-tree-autorefresh.png`); no ignored-files node or Add to .gitignore |
| PLATFORM-VCS-011 | IntelliJ Platform | Rollback local changes | Partial | `vcs.rollbackChange` (⌥⌘Z / Ctrl+Alt+Z) | `line_changes::tests::navigates_and_rolls_back_hunks_with_undo` | rolls back the changed lines at the caret or a clicked gutter marker as one undoable edit (`evidence/git/gutter-rollback.png`); no rollback of whole files from the Commit window yet |
| PLATFORM-VCS-012 | IntelliJ Platform | Gutter change markers | Partial | `vcs.nextChange`, `vcs.previousChange`, `vcs.rollbackChange`, `vcs.showChangeDiff` | `line_changes::tests`, `change_popup::tests` | green/blue bars and a grey deletion wedge between line numbers and text, live while typing, against HEAD (untracked files have none); click opens a popup with previous/next, Rollback, Show Diff and the HEAD text of the changed lines (`evidence/git/gutter-change-markers.png`, `gutter-change-popup.png`); no inline word diff in the popup, Copy, or error-stripe marks |
| PLATFORM-NAV-034 | IntelliJ Platform | VCS change navigation in editor | Implemented | `vcs.nextChange` (⌃⌥⇧↓), `vcs.previousChange` (⌃⌥⇧↑) | `line_changes::tests::navigates_and_rolls_back_hunks_with_undo` | moves the caret to the next/previous changed hunk; stops at the ends |
| PLATFORM-VCS-014 | IntelliJ Platform | Git stash | Missing | — | — | — |
| PLATFORM-VCS-020 | IntelliJ Platform | Diff viewer | Partial | `git.showDiff`, `diff.setViewer`, `diff.activate`, `diff.close` | `diff::tests` (hunks, word ranges, scroll mapping), `diffs_working_files_against_head`, `vcs::tests::shows_diff_and_navigates_changes`, `diff_ui::tests` | HEAD vs working copy (unsaved editor text included) as an editor tab; side-by-side with connectors and both line-number columns, or unified; word-level highlights in modified hunks; syntax colours wherever the language layer highlights the file; error-stripe markers; synchronized scrolling; updates live when the file changes. Read-only; no whitespace modes, collapse-unchanged, apply/revert arrows or per-revision compare. Matches scene 13 layout (`evidence/git/13-diff-side-by-side.png`, `13c-diff-unified.png`) |
| PLATFORM-VCS-021 | IntelliJ Platform | Diff navigation | Partial | `diff.nextChange` (F7), `diff.previousChange` (⇧F7) | `vcs::tests::shows_diff_and_navigates_changes` | next/previous difference within the file, toolbar arrows; does not continue to the next file |
| PLATFORM-VCS-024 | IntelliJ Platform | Compare with revision/branch | Missing | — | — | only HEAD vs working copy via `git.showDiff` |
| PLATFORM-VCS-030 | IntelliJ Platform | Annotate / blame | Missing | — | — | — |
| PLATFORM-VCS-031 | IntelliJ Platform | File history | Missing | — | — | — |
| PLATFORM-VCS-032 | IntelliJ Platform | Git log tool window | Missing | — | — | scene 14 |
| PLATFORM-VCS-035 | IntelliJ Platform | Branches popup / VCS widget | Missing | — | — | toolbar widget shows the branch; popup has only Commit and Refresh |
| PLATFORM-VCS-041 | IntelliJ Platform | Fetch | Missing | — | — | — |
| PLATFORM-VCS-042 | IntelliJ Platform | Pull | Missing | — | — | — |
| PLATFORM-VCS-043 | IntelliJ Platform | Push dialog | Missing | — | — | — |

Statuses are conservative: “Partial” rows list what is still missing.
