# Language servers and completion: evidence

Reference scene: `09-completion.png` (IntelliJ IDEA Community 2025.2.6.3, java-gradle sample, and
Android Studio 2026.2.1) from `docs/parity/evidence/reference/`. Before this work Jetthing showed
"command is disabled: language.complete" for Java and Kotlin files
(`jetthing/java-gradle/09-completion-absent.png`).

## Environment

- Linux, Xvfb 1600x1000, `JETTHING_RENDERER=glow JETTHING_NO_UPDATE_CHECK=1`.
- rust-analyzer 1.97.0 (rustup component), Eclipse JDT LS 1.40.0 (`jdtls` on PATH), OpenJDK 21.
- `/home/user/samples/java-gradle`: `settings.gradle.kts`, `build.gradle.kts` (application plugin,
  Java 21 toolchain) and `src/main/java/com/example/App.java` as in the reference scene.

## Scenes

| File | Steps | Result |
|---|---|---|
| `status-jdtls-import-progress.png` | Open the project, open `App.java` | jdtls starts lazily on the first Java file. The status bar shows its `$/progress` report ("jdtls: Initialize Workspace 69% (Importing root project …)") with a spinner. |
| `09-java-completion-jdtls.png` | After the import, type `app.` on line 10 and press Ctrl+Space | The popup lists the jdtls members (`clone`, `equals`, …, `greeting`, …) with their signatures. jdtls diagnostics are also shown (squiggle under `app.`, red dot on Problems). |
| `kotlin-server-missing-notice.png` | Open a `.kt` file without a Kotlin server installed | Balloon "Kotlin language server not found" with the install hint, "Copy Install Hint" and "Restart Language Servers". The status bar shows "kotlin: not installed". |

## Still different from the reference

- Rows are sorted by label, not by the server's `sortText`/relevance (IntelliJ puts `greeting`
  first), and there is no filtering as you type, kind icon, bold matched characters, return type
  column or tips footer. This is the next slice (completion popup rework).
- The popup does not open on its own after `.`; Ctrl+Space is needed.
- No documentation side panel, parameter info or snippets yet.

## Reproduce

```bash
cargo test -p jetthing-lsp                                         # fake-server integration tests
cargo test -p jetthing-lsp --test real_servers -- --ignored        # rust-analyzer + jdtls, if installed
```
