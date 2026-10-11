# IntelliJ IDEA parity: version control

Git integration (Git menu, Commit and Git tool windows, diff and merge viewers) plus
hosting integrations. Other VCSs are tracked at low weight.

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
| IJ-VCS-001 | [set-up-a-git-repository](https://www.jetbrains.com/help/idea/set-up-a-git-repository.html) | Detect Git repositories and roots | Implemented | 5 | `git.refresh` | real repository integration test | single root; nested roots and submodules pending |
| IJ-VCS-002 | [set-up-a-git-repository](https://www.jetbrains.com/help/idea/set-up-a-git-repository.html) | Clone repository (Get from VCS) | Missing | 3 | — | — | not started |
| IJ-VCS-003 | [set-up-a-git-repository](https://www.jetbrains.com/help/idea/set-up-a-git-repository.html) | Create Git repository (VCS > Enable Version Control) | Missing | 1 | — | — | not started |
| IJ-VCS-004 | [adding-files-to-version-control](https://www.jetbrains.com/help/idea/adding-files-to-version-control.html) | Add files / unversioned files handling | Partial | 2 | `git.stage` | real repository integration test | staging adds files; no unversioned-files group or add-on-create prompt Jetthing/jdtls-generated files are excluded from the unversioned list (#147). |
| IJ-VCS-005 | [ignoring-files](https://www.jetbrains.com/help/idea/ignoring-files.html) | .gitignore support and ignored-file display | Missing | 2 | — | — | not started Not .gitignore support; IDE artefacts are filtered in-process (#147). |
| IJ-VCS-006 | [main-version-control-shortcuts](https://www.jetbrains.com/help/idea/main-version-control-shortcuts.html) | VCS Operations Popup (Alt+`) | Missing | 2 | — | — | not started |
| IJ-VCS-010 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Local changes view with file status | Partial | 5 | `git.refresh` | real repository integration test | status list; no tree grouping or diff preview |
| IJ-VCS-011 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Stage / unstage files | Implemented | 5 | `git.stage`, `git.unstage` | real repository integration test | whole files only |
| IJ-VCS-012 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Partial commits (stage hunks/lines via gutter checkboxes) | Missing | 3 | — | — | not started |
| IJ-VCS-013 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Commit with message (Ctrl+K) | Missing | 5 | — | — | not started |
| IJ-VCS-014 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Amend commit | Missing | 3 | — | — | not started |
| IJ-VCS-015 | [commit-dialog](https://www.jetbrains.com/help/idea/commit-dialog.html) | Commit checks (reformat, inspections, tests before commit) | Missing | 1 | — | — | not started |
| IJ-VCS-016 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Commit message history and templates | Missing | 1 | — | — | not started |
| IJ-VCS-017 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Commit and Push (Ctrl+Alt+K) | Missing | 3 | — | — | not started |
| IJ-VCS-018 | [set-up-GPG-commit-signing](https://www.jetbrains.com/help/idea/set-up-GPG-commit-signing.html) | GPG commit signing | Missing | 1 | — | — | not started |
| IJ-VCS-019 | [managing-changelists](https://www.jetbrains.com/help/idea/managing-changelists.html) | Changelists | Missing | 2 | — | — | not started |
| IJ-VCS-020 | [shelving-and-unshelving-changes](https://www.jetbrains.com/help/idea/shelving-and-unshelving-changes.html) | Shelve / unshelve changes | Missing | 2 | — | — | not started |
| IJ-VCS-021 | [shelving-and-unshelving-changes](https://www.jetbrains.com/help/idea/shelving-and-unshelving-changes.html) | Git stash / unstash | Missing | 2 | — | — | not started |
| IJ-VCS-022 | [undo-changes](https://www.jetbrains.com/help/idea/undo-changes.html) | Rollback local changes (Ctrl+Alt+Z) | Missing | 3 | — | — | not started |
| IJ-VCS-023 | [using-patches](https://www.jetbrains.com/help/idea/using-patches.html) | Create / apply patch | Missing | 1 | — | — | not started |
| IJ-VCS-030 | [sync-with-a-remote-repository](https://www.jetbrains.com/help/idea/sync-with-a-remote-repository.html) | Fetch | Missing | 3 | — | — | not started |
| IJ-VCS-031 | [sync-with-a-remote-repository](https://www.jetbrains.com/help/idea/sync-with-a-remote-repository.html) | Pull / Update Project (Ctrl+T) with merge or rebase | Missing | 5 | — | — | not started |
| IJ-VCS-032 | [commit-and-push-changes](https://www.jetbrains.com/help/idea/commit-and-push-changes.html) | Push dialog (Ctrl+Shift+K) with commit preview, force push | Missing | 5 | — | — | not started |
| IJ-VCS-033 | [sync-with-a-remote-repository](https://www.jetbrains.com/help/idea/sync-with-a-remote-repository.html) | Manage remotes | Missing | 1 | — | — | not started |
| IJ-VCS-034 | [sync-with-a-remote-repository](https://www.jetbrains.com/help/idea/sync-with-a-remote-repository.html) | Credentials / SSH agent handling | Missing | 2 | — | — | not started |
| IJ-VCS-040 | [manage-branches](https://www.jetbrains.com/help/idea/manage-branches.html) | Branches popup: current branch display | Partial | 3 | `git.refresh` | real repository integration test | current branch name shown; no popup |
| IJ-VCS-041 | [manage-branches](https://www.jetbrains.com/help/idea/manage-branches.html) | Create / checkout / rename / delete branch | Missing | 5 | — | — | not started |
| IJ-VCS-042 | [manage-branches](https://www.jetbrains.com/help/idea/manage-branches.html) | Compare branches, show diff with working tree | Missing | 2 | — | — | not started |
| IJ-VCS-043 | [apply-changes-from-one-branch-to-another](https://www.jetbrains.com/help/idea/apply-changes-from-one-branch-to-another.html) | Merge branch into current | Missing | 3 | — | — | not started |
| IJ-VCS-044 | [apply-changes-from-one-branch-to-another](https://www.jetbrains.com/help/idea/apply-changes-from-one-branch-to-another.html) | Rebase (including interactive rebase dialog) | Missing | 3 | — | — | not started |
| IJ-VCS-045 | [apply-changes-from-one-branch-to-another](https://www.jetbrains.com/help/idea/apply-changes-from-one-branch-to-another.html) | Cherry-pick commits and individual changes | Missing | 2 | — | — | not started |
| IJ-VCS-046 | [use-tags-to-mark-specific-commits](https://www.jetbrains.com/help/idea/use-tags-to-mark-specific-commits.html) | Tags: create, push, checkout | Missing | 1 | — | — | not started |
| IJ-VCS-047 | [use-git-worktrees](https://www.jetbrains.com/help/idea/use-git-worktrees.html) | Git worktrees | Missing | 1 | — | — | not started |
| IJ-VCS-048 | [edit-project-history](https://www.jetbrains.com/help/idea/edit-project-history.html) | Edit history: squash, fixup, reword, drop, undo commit | Missing | 2 | — | — | not started |
| IJ-VCS-050 | [comparing-file-versions](https://www.jetbrains.com/help/idea/comparing-file-versions.html) | Diff viewer: side-by-side and unified | Missing | 5 | — | — | not started |
| IJ-VCS-051 | [comparing-file-versions](https://www.jetbrains.com/help/idea/comparing-file-versions.html) | Diff viewer: next/previous difference, word-level highlighting | Missing | 2 | — | — | not started |
| IJ-VCS-052 | [comparing-file-versions](https://www.jetbrains.com/help/idea/comparing-file-versions.html) | Compare two files / with clipboard / folders | Missing | 2 | — | — | not started |
| IJ-VCS-053 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | Git log with graph, filters (branch, user, date, path), search | Missing | 3 | — | — | not started |
| IJ-VCS-054 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | Commit details: changed files and diff | Missing | 3 | — | — | not started |
| IJ-VCS-055 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | File history (Show History) | Missing | 3 | — | — | not started |
| IJ-VCS-056 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | Annotate with Git Blame | Missing | 3 | — | — | not started |
| IJ-VCS-057 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | Show history for selection | Missing | 1 | — | — | not started |
| IJ-VCS-058 | [investigate-changes](https://www.jetbrains.com/help/idea/investigate-changes.html) | Gutter change markers with rollback/show diff popup | Missing | 3 | — | — | not started |
| IJ-VCS-059 | [resolve-conflicts](https://www.jetbrains.com/help/idea/resolve-conflicts.html) | Three-way merge tool with conflict resolution | Missing | 5 | — | — | not started |
| IJ-VCS-060 | [resolve-conflicts](https://www.jetbrains.com/help/idea/resolve-conflicts.html) | Resolve simple conflicts automatically / accept yours / theirs | Missing | 2 | — | — | not started |
| IJ-VCS-061 | [local-history](https://www.jetbrains.com/help/idea/local-history.html) | Local history independent of VCS | Missing | 2 | — | — | see IJ-ED-028 |
| IJ-VCS-070 | [github](https://www.jetbrains.com/help/idea/github.html) | GitHub account login | Missing | 2 | — | — | not started |
| IJ-VCS-071 | [create-and-merge-github-pull-requests](https://www.jetbrains.com/help/idea/create-and-merge-github-pull-requests.html) | Create GitHub pull requests | Missing | 2 | — | — | not started |
| IJ-VCS-072 | [work-with-github-pull-requests](https://www.jetbrains.com/help/idea/work-with-github-pull-requests.html) | Review GitHub pull requests (comments, timeline, diff) | Missing | 2 | — | — | not started |
| IJ-VCS-073 | [share-code-with-gists](https://www.jetbrains.com/help/idea/share-code-with-gists.html) | Share code as gist | Missing | 1 | — | — | not started |
| IJ-VCS-074 | [github-actions](https://www.jetbrains.com/help/idea/github-actions.html) | GitHub Actions workflow support | Missing | 1 | — | — | not started |
| IJ-VCS-075 | [gitlab](https://www.jetbrains.com/help/idea/gitlab.html) | GitLab merge requests | Missing | 1 | — | — | not started |
| IJ-VCS-076 | [gitlab-ci-cd](https://www.jetbrains.com/help/idea/gitlab-ci-cd.html) | GitLab CI/CD support | Missing | 1 | — | — | not started |
| IJ-VCS-077 | [handling-issues](https://www.jetbrains.com/help/idea/handling-issues.html) | Issue tracker links in commit messages | Missing | 1 | — | — | not started |
| IJ-VCS-078 | [fork-github-projects](https://www.jetbrains.com/help/idea/fork-github-projects.html) | Fork / share project on GitHub | Missing | 1 | — | — | not started |
| IJ-VCS-080 | [using-mercurial-integration](https://www.jetbrains.com/help/idea/using-mercurial-integration.html) | Mercurial integration | Missing | 1 | — | — | not started |
| IJ-VCS-081 | [using-subversion-integration](https://www.jetbrains.com/help/idea/using-subversion-integration.html) | Subversion integration | Missing | 1 | — | — | not started |
| IJ-VCS-082 | [using-perforce-integration](https://www.jetbrains.com/help/idea/using-perforce-integration.html) | Perforce integration | Missing | 1 | — | — | not started |
