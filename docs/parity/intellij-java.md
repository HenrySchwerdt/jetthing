# IntelliJ IDEA parity: Java and JVM languages

Java-specific language intelligence, project model, SDKs, compilation, and JVM language
support (Kotlin, Groovy). Generic editor behavior is tracked in the editor area.

Sources: IntelliJ IDEA public documentation (<https://www.jetbrains.com/help/idea/>), the predefined Windows
keymap reference (<https://www.jetbrains.com/help/idea/reference-keymap-win-default.html>) and the unified-distribution FAQ (<https://lp.jetbrains.com/intellij-idea-unified-faq/>), which lists
the features that need an Ultimate subscription. Rows were written from public
documentation and observable behavior only; no JetBrains code, artwork, or branding
is reused.

Rows tagged `[Ultimate]` require an IntelliJ IDEA Ultimate subscription. Untagged rows are
available in the free distribution. `Weight` (1-5) is the relative importance used by the
weighted dashboard: 5 = core daily workflow, 3 = significant, 2 = moderate, 1 = minor.
Statuses describe Jetthing, conservatively: nothing in this catalog is `Verified`.

| ID | Reference | Feature | Status | Weight | Spec | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|---|
| IJ-JAVA-001 | [creating-and-running-your-first-java-application](https://www.jetbrains.com/help/idea/creating-and-running-your-first-java-application.html) | Create Java project (New Project wizard, build system choice) | Partial | 3 | `GRADLE-004` | `project.create` | `create_project_command_generates_and_opens_without_overwriting`, `templates_and_collisions_are_safe` | Java Gradle project (Kotlin or Groovy DSL) with sample code; needs JDK 21 and Gradle on PATH; Maven and JDK download pending |
| IJ-JAVA-002 | [sdk](https://www.jetbrains.com/help/idea/sdk.html) | JDK detection, download, and selection | Missing | 5 | `JAVA-JDK-002`, `JAVA-JDK-004`, `JAVA-JDK-005`, `JAVA-JDK-006` | — | — | not started |
| IJ-JAVA-003 | [supported-java-versions](https://www.jetbrains.com/help/idea/supported-java-versions.html) | Language level per project and module | Missing | 3 | `JAVA-JDK-008`, `JAVA-JDK-009`, `GRADLE-050` | — | — | not started |
| IJ-JAVA-004 | [configure-modules](https://www.jetbrains.com/help/idea/configure-modules.html) | Modules with source, test, resource, generated roots | Missing | 3 | `PLATFORM-PROJ-014` | — | — | not started |
| IJ-JAVA-005 | [library](https://www.jetbrains.com/help/idea/library.html) | Libraries and dependency scopes | Missing | 3 | `PLATFORM-PROJ-016`, `JAVA-JDK-014` | — | — | not started |
| IJ-JAVA-006 | [configure-sdk-documentation](https://www.jetbrains.com/help/idea/configure-sdk-documentation.html) | Attach JDK sources and documentation | Missing | 1 | `JAVA-JDK-012` | — | — | not started |
| IJ-JAVA-010 | [working-with-source-code](https://www.jetbrains.com/help/idea/working-with-source-code.html) | Java syntax highlighting and parsing | Implemented | 5 | `JAVA-EDIT-001` | editor edit pipeline | `java_tokens_resolve_to_islands_dark_colors`, `every_bundled_highlight_query_compiles` | tree-sitter-java highlighting and parse errors; full reparse per edit |
| IJ-JAVA-011 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Java semantic error highlighting (resolve, types) | Partial | 5 | `JAVA-EDIT-003` | `language.complete` | `completion_is_enabled_for_java_files_and_explains_a_missing_server`; ignored real jdtls test | semantic errors from jdtls when installed; no built-in Java analysis |
| IJ-JAVA-012 | [auto-completing-code](https://www.jetbrains.com/help/idea/auto-completing-code.html) | Java code completion | Partial | 5 | `JAVA-EDIT-020` | `language.complete`, `language.completion` | `completion_is_enabled_for_java_files_and_explains_a_missing_server`; ignored real jdtls test | completion from jdtls when installed (`evidence/lsp/09-java-completion-jdtls.png`) |
| IJ-JAVA-013 | [navigating-through-the-source-code](https://www.jetbrains.com/help/idea/navigating-through-the-source-code.html) | Java navigation (declaration, implementations, super) | Missing | 5 | `JAVA-NAV-008`, `JAVA-NAV-009`, `JAVA-NAV-011` | — | — | not started |
| IJ-JAVA-014 | [code-inspection](https://www.jetbrains.com/help/idea/code-inspection.html) | Java inspections (nullability, unused, probable bugs) | Missing | 3 | `JAVA-INSP-101`, `JAVA-INSP-103` | — | — | not started |
| IJ-JAVA-015 | [nullable-and-notnull-annotations](https://www.jetbrains.com/help/idea/nullable-and-notnull-annotations.html) | @Nullable/@NotNull inference and checks | Missing | 2 | `JAVA-LANG-015`, `JAVA-LANG-018`, `JAVA-INSP-123` | — | — | not started |
| IJ-JAVA-016 | [intention-actions](https://www.jetbrains.com/help/idea/intention-actions.html) | Java quick-fixes and intentions | Missing | 3 | `JAVA-INSP-201`, `JAVA-INSP-203`, `JAVA-INSP-204` | — | — | not started |
| IJ-JAVA-017 | [generating-code](https://www.jetbrains.com/help/idea/generating-code.html) | Java code generation (constructor, getters/setters, equals/hashCode, toString, records) | Missing | 3 | `JAVA-GEN-002`, `JAVA-GEN-003`, `JAVA-GEN-004`, `JAVA-GEN-005`, `JAVA-GEN-006`, `JAVA-GEN-007` | — | — | not started |
| IJ-JAVA-018 | [refactoring-source-code](https://www.jetbrains.com/help/idea/refactoring-source-code.html) | Java refactorings engine | Missing | 5 | — | — | — | see intellij-refactoring.md |
| IJ-JAVA-019 | [javadocs](https://www.jetbrains.com/help/idea/javadocs.html) | Javadoc: generation stubs, rendering, completion inside docs | Missing | 1 | `JAVA-EDIT-080`, `JAVA-EDIT-081`, `JAVA-EDIT-082` | — | — | not started |
| IJ-JAVA-020 | [decompiler](https://www.jetbrains.com/help/idea/decompiler.html) | Java bytecode decompiler for library classes | Missing | 2 | `JAVA-EDIT-008` | — | — | not started |
| IJ-JAVA-021 | [decompiler](https://www.jetbrains.com/help/idea/decompiler.html) | Show bytecode | Missing | 1 | — | — | — | not started |
| IJ-JAVA-022 | [6317-lombok](https://plugins.jetbrains.com/plugin/6317-lombok) | Lombok support | Missing | 2 | `JAVA-LOMBOK-001`, `JAVA-LOMBOK-002` | — | — | not started |
| IJ-JAVA-023 | [annotation-processors-support](https://www.jetbrains.com/help/idea/annotation-processors-support.html) | Annotation processing in the build | Missing | 2 | `JAVA-BUILD-008`, `GRADLE-053` | — | — | not started |
| IJ-JAVA-024 | [supported-java-versions](https://www.jetbrains.com/help/idea/supported-java-versions.html) | Java Platform Module System (module-info) support | Missing | 1 | `JAVA-LANG-014`, `GRADLE-058` | — | — | not started |
| IJ-JAVA-025 | [jshell-console](https://www.jetbrains.com/help/idea/jshell-console.html) | JShell console | Missing | 1 | — | — | — | not started |
| IJ-JAVA-026 | [supported-java-versions](https://www.jetbrains.com/help/idea/supported-java-versions.html) | Modern Java: records, sealed types, pattern matching, text blocks | Missing | 2 | `JAVA-LANG-001`, `JAVA-LANG-002`, `JAVA-LANG-003`, `JAVA-LANG-006`, `JAVA-JDK-017` | — | — | not started |
| IJ-JAVA-030 | [compiling-applications](https://www.jetbrains.com/help/idea/compiling-applications.html) | Build Project (Ctrl+F9) incremental compilation | Partial | 5 | `JAVA-BUILD-001` | `build.project` | real process/output/cancellation tests | delegates to detected Gradle/Maven build; no IDE compiler |
| IJ-JAVA-031 | [compiling-applications](https://www.jetbrains.com/help/idea/compiling-applications.html) | Rebuild Project / Recompile file | Missing | 2 | `JAVA-BUILD-003`, `JAVA-BUILD-004` | — | — | not started |
| IJ-JAVA-032 | [compiling-applications](https://www.jetbrains.com/help/idea/compiling-applications.html) | Build output errors mapped to editor locations | Missing | 3 | `JAVA-BUILD-013` | — | — | not started |
| IJ-JAVA-033 | [build-overview](https://www.jetbrains.com/help/idea/build-overview.html) | Build tool window with build tree | Missing | 3 | `JAVA-BUILD-005` | — | — | not started |
| IJ-JAVA-034 | [compiling-applications](https://www.jetbrains.com/help/idea/compiling-applications.html#package_into_jar) | Build artifacts (JAR) | Missing | 1 | `JAVA-BUILD-011` | — | — | not started |
| IJ-JAVA-040 | [get-started-with-kotlin](https://www.jetbrains.com/help/idea/get-started-with-kotlin.html) | Kotlin language support | Partial | 5 | — | `language.complete` | `maps_files_to_servers_and_language_ids`, `every_bundled_highlight_query_compiles` | Kotlin highlighting; completion/diagnostics via kotlin-lsp or kotlin-language-server when installed (not verified) |
| IJ-JAVA-041 | [get-started-with-kotlin](https://www.jetbrains.com/help/idea/get-started-with-kotlin.html) | Java-Kotlin interop and J2K conversion | Missing | 1 | — | — | — | not started |
| IJ-JAVA-042 | [interactive-groovy-console](https://www.jetbrains.com/help/idea/interactive-groovy-console.html) | Groovy language support and console | Missing | 1 | — | — | — | not started |
| IJ-JAVA-043 | [discover-intellij-idea-for-scala](https://www.jetbrains.com/help/idea/discover-intellij-idea-for-scala.html) | Scala (plugin) | Missing | 1 | — | — | — | not started |
| IJ-JAVA-044 | [ktor](https://www.jetbrains.com/help/idea/ktor.html) | Ktor framework support | Missing | 1 | — | — | — | not started |
