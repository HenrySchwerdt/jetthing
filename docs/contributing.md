# Contributing

## Commit and PR titles

Releases are versioned automatically from commit messages, so every commit on `main`
(for squash merges, the PR title) must follow [Conventional Commits](https://www.conventionalcommits.org):

```
<type>[!]: <short summary>
```

| Type | Use for | Release effect |
|---|---|---|
| `feat` | user-visible feature | patch while `0.0.x`, minor from `0.1.0` |
| `fix` | bug fix | patch |
| `perf` | performance improvement | patch |
| `refactor`, `docs`, `test`, `chore`, `ci` | no behaviour change | none |

Add `!` after the type (`feat!: ...`) or a `BREAKING CHANGE:` footer for breaking changes.
See [release.md](release.md#automatic-versioning) for how the release PR is built.
Before committing, run the checks listed in `AGENTS.md`.
