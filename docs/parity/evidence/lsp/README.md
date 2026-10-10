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

### Completion popup rework (slice 2)

| File | Steps | Result |
|---|---|---|
| `09-java-autopopup-after-dot.png` | Type `app.` (no Ctrl+Space) | The popup opens by itself on the server trigger character. `greeting(String name)  String` comes first, as in the reference: jdtls gives every member the same `sortText`, and the jdtls provider ranks members inherited from `java.lang.Object` after the class's own. The layout matches the reference rows: kind icon, label, grey parameters, right-aligned type, footer hint. |
| `09b-java-filter-prefix.png` | Type `g` | Filtered on the client to `greeting`, `getClass` with the matched `g` in bold. There is no new server request. |
| `09c-java-camel-hump-hC.png` | Replace with `hC` | Camel-hump match `hashCode` (`h` + `C` bold). |
| `09d-rust-autopopup-rust-analyzer.png` | Rust: `message.` in `/home/user/samples/rust-app` | rust-analyzer items with `labelDetails` types right-aligned. |
| `09e-rust-camel-hump-ps.png` | Type `ps` | `push_str` via the `p`…`_s` humps. |

Accepting with Enter inserts the label at the caret. Tab replaces the identifier after the caret
(covered by `jetthing-core` completion tests, including `additionalTextEdits` as one undo step).

## Still different from the reference

- Methods are inserted without `()`. Snippets with tab stops come next (slice 3).
- jdtls still creates `.project` and `bin/` in Gradle projects (the same as the VS Code Java
  extension). `.classpath` and `.settings` are kept out by passing
  `-Djava.import.generatesMetadataFilesAtProjectRoot=false`.
- No star/arrow relevance markers, documentation side panel (Ctrl+Q) or parameter info yet.
- The popup closes when nothing matches, like the reference. Explicit Ctrl+Space with no results
  reports "No suggestions" in the status bar instead of a popup message.

## Reproduce

```bash
cargo test -p jetthing-lsp                                         # fake-server integration tests
cargo test -p jetthing-lsp --test real_servers -- --ignored        # rust-analyzer + jdtls, if installed
```
