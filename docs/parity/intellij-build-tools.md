# IntelliJ IDEA parity: Gradle and Maven

Gradle and Maven project import, synchronization, tool windows, task/goal execution, and
dependency management.

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
| IJ-BLD-001 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Detect Gradle project (settings.gradle[.kts], build.gradle[.kts]) | Partial | 5 | `build.project` | — | detected only when a `gradlew` wrapper exists; builds via `gradlew build`; no model import |
| IJ-BLD-002 | [work-with-gradle-projects](https://www.jetbrains.com/help/idea/work-with-gradle-projects.html) | Import Gradle project model (modules, source sets, dependencies) | Missing | 5 | — | — | not started |
| IJ-BLD-003 | [work-with-gradle-projects](https://www.jetbrains.com/help/idea/work-with-gradle-projects.html) | Reload Gradle project / auto-reload on script changes | Missing | 3 | — | — | not started |
| IJ-BLD-004 | [work-with-gradle-tasks](https://www.jetbrains.com/help/idea/work-with-gradle-tasks.html) | Gradle tool window: task tree and run task | Missing | 3 | — | — | not started |
| IJ-BLD-005 | [run-debug-gradle](https://www.jetbrains.com/help/idea/run-debug-gradle.html) | Gradle run/debug configurations | Missing | 3 | — | — | not started |
| IJ-BLD-006 | [gradle-jvm-selection](https://www.jetbrains.com/help/idea/gradle-jvm-selection.html) | Gradle JVM selection | Missing | 2 | — | — | not started |
| IJ-BLD-007 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Delegate build and run to Gradle vs IDE builder | Missing | 2 | — | — | not started |
| IJ-BLD-008 | [work-with-tests-in-gradle](https://www.jetbrains.com/help/idea/work-with-tests-in-gradle.html) | Run tests through Gradle with test tree | Missing | 3 | — | — | not started |
| IJ-BLD-009 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Gradle wrapper support (gradlew) | Partial | 3 | `build.project` | — | `gradlew build` is used; projects without a wrapper are not detected |
| IJ-BLD-010 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Offline mode and build scans | Missing | 1 | — | — | not started |
| IJ-BLD-011 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Kotlin DSL / Groovy DSL editing assistance | Missing | 3 | — | — | not started |
| IJ-BLD-012 | [migrate-gradle-build-scripts-to-kotlin-dsl](https://www.jetbrains.com/help/idea/migrate-gradle-build-scripts-to-kotlin-dsl.html) | Version catalogs (libs.versions.toml) support | Missing | 2 | — | — | not started |
| IJ-BLD-013 | [work-with-gradle-dependency-diagram](https://www.jetbrains.com/help/idea/work-with-gradle-dependency-diagram.html) | [Ultimate] Gradle dependency diagram | Missing | 1 | — | — | not started |
| IJ-BLD-014 | [gradle](https://www.jetbrains.com/help/idea/gradle.html) | Gradle composite builds and buildSrc | Missing | 1 | — | — | not started |
| IJ-BLD-020 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Detect Maven project (pom.xml) | Partial | 5 | `build.project` | — | detects Maven for building; no model import |
| IJ-BLD-021 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Import Maven project model (modules, dependencies, profiles) | Missing | 5 | — | — | not started |
| IJ-BLD-022 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Reload Maven project / auto-reload | Missing | 3 | — | — | not started |
| IJ-BLD-023 | [work-with-maven-goals](https://www.jetbrains.com/help/idea/work-with-maven-goals.html) | Maven tool window: lifecycle and plugin goals | Missing | 3 | — | — | not started |
| IJ-BLD-024 | [run-debug-configuration-maven](https://www.jetbrains.com/help/idea/run-debug-configuration-maven.html) | Maven run/debug configurations | Missing | 2 | — | — | not started |
| IJ-BLD-025 | [work-with-maven-profiles](https://www.jetbrains.com/help/idea/work-with-maven-profiles.html) | Maven profiles toggling | Missing | 1 | — | — | not started |
| IJ-BLD-026 | [work-with-maven-dependencies](https://www.jetbrains.com/help/idea/work-with-maven-dependencies.html) | pom.xml completion, dependency search, version updates | Missing | 3 | — | — | not started |
| IJ-BLD-027 | [delegate-build-and-run-actions-to-maven](https://www.jetbrains.com/help/idea/delegate-build-and-run-actions-to-maven.html) | Delegate build/run to Maven | Missing | 1 | — | — | not started |
| IJ-BLD-028 | [work-with-tests-in-maven](https://www.jetbrains.com/help/idea/work-with-tests-in-maven.html) | Run tests via Maven Surefire with test tree | Missing | 2 | — | — | not started |
| IJ-BLD-029 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Maven wrapper (mvnw) support | Partial | 2 | `build.project` | — | wrapper preferred when present |
| IJ-BLD-030 | [convert-a-regular-project-into-a-maven-project](https://www.jetbrains.com/help/idea/convert-a-regular-project-into-a-maven-project.html) | Add framework support / convert project to Maven or Gradle | Missing | 1 | — | — | not started |
| IJ-BLD-031 | [maven-support](https://www.jetbrains.com/help/idea/maven-support.html) | Download sources and documentation for dependencies | Missing | 1 | — | — | not started |
| IJ-BLD-040 | [work-with-maven-dependencies](https://www.jetbrains.com/help/idea/work-with-maven-dependencies.html) | Dependency Analyzer (conflicts, scopes, usages) | Missing | 2 | — | — | not started |
| IJ-BLD-041 | [package-search](https://www.jetbrains.com/help/idea/package-search.html) | Dependency search and add from editor | Missing | 1 | — | — | not started |
| IJ-BLD-042 | [ant](https://www.jetbrains.com/help/idea/ant.html) | Ant build files | Missing | 1 | — | — | not started |
| IJ-BLD-043 | [sbt-support](https://www.jetbrains.com/help/idea/sbt-support.html) | sbt (Scala plugin) | Missing | 1 | — | — | not started |
| IJ-BLD-044 | [build-overview](https://www.jetbrains.com/help/idea/build-overview.html) | Build output with clickable compiler errors | Missing | 3 | — | — | not started |
