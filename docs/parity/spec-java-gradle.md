# Spec features not covered by the catalog: java-gradle.md

Rows for feature ids in [`docs/spec/features/java-gradle.md`](../spec/features/java-gradle.md)
that no IntelliJ catalog row links in its `Spec` column. `cargo run -p xtask -- parity
--sync-spec` appends a `Missing` row here for every new uncovered id; edit statuses and
evidence in place. Weights follow the spec priority: P0 = 5, P1 = 3, P2 = 1. Rows tagged
`[Ultimate]` are paid features in the spec. Keep the table last in this file.

| ID | Reference | Feature | Status | Weight | Spec | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|---|
| GRADLE-003 | [java-gradle.md](../spec/features/java-gradle.md) | Trust project prompt | Missing | 5 | `GRADLE-003` | — | — | not started (P0) |
| GRADLE-006 | [java-gradle.md](../spec/features/java-gradle.md) | Link Gradle Project | Missing | 3 | `GRADLE-006` | — | — | not started (P1) |
| GRADLE-007 | [java-gradle.md](../spec/features/java-gradle.md) | Unlink Gradle Project | Missing | 3 | `GRADLE-007` | — | — | not started (P1) |
| GRADLE-008 | [java-gradle.md](../spec/features/java-gradle.md) | Ignore / Unignore Gradle project | Missing | 1 | `GRADLE-008` | — | — | not started (P2) |
| GRADLE-009 | [java-gradle.md](../spec/features/java-gradle.md) | Orphan modules dialog | Missing | 1 | `GRADLE-009` | — | — | not started (P2) |
| GRADLE-011 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle Project Data To Import | Missing | 1 | `GRADLE-011` | — | — | not started (P2) |
| GRADLE-025 | [java-gradle.md](../spec/features/java-gradle.md) | Sync progress and errors in Build tool window | Missing | 5 | `GRADLE-025` | — | — | not started (P0) |
| GRADLE-026 | [java-gradle.md](../spec/features/java-gradle.md) | Sync failure notifications with fixes | Missing | 3 | `GRADLE-026` | — | — | not started (P1) |
| GRADLE-027 | [java-gradle.md](../spec/features/java-gradle.md) | Refresh dependencies | Missing | 1 | `GRADLE-027` | — | — | not started (P2) |
| GRADLE-029 | [java-gradle.md](../spec/features/java-gradle.md) | Parallel model fetch | Missing | 1 | `GRADLE-029` | — | — | not started (P2) |
| GRADLE-030 | [java-gradle.md](../spec/features/java-gradle.md) | Execute task before/after sync | Missing | 1 | `GRADLE-030` | — | — | not started (P2) |
| GRADLE-031 | [java-gradle.md](../spec/features/java-gradle.md) | Declarative Gradle sync (experimental) | Missing | 1 | `GRADLE-031` | — | — | not started (P2) |
| GRADLE-049 | [java-gradle.md](../spec/features/java-gradle.md) | Included plugin builds (`pluginManagement { includeBuild }`) | Missing | 3 | `GRADLE-049` | — | — | not started (P1) |
| GRADLE-051 | [java-gradle.md](../spec/features/java-gradle.md) | Module SDK from Java toolchain | Missing | 3 | `GRADLE-051` | — | — | not started (P1) |
| GRADLE-052 | [java-gradle.md](../spec/features/java-gradle.md) | Compiler args / encoding from build | Missing | 3 | `GRADLE-052` | — | — | not started (P1) |
| GRADLE-054 | [java-gradle.md](../spec/features/java-gradle.md) | IDE settings via `idea-ext` plugin | Missing | 1 | `GRADLE-054` | — | — | not started (P2) |
| GRADLE-055 | [java-gradle.md](../spec/features/java-gradle.md) | Generate `.iml` files option | Missing | 1 | `GRADLE-055` | — | — | not started (P2) |
| GRADLE-056 | [java-gradle.md](../spec/features/java-gradle.md) | Manual changes are transient | Missing | 3 | `GRADLE-056` | — | — | not started (P1) |
| GRADLE-057 | [java-gradle.md](../spec/features/java-gradle.md) | External annotations download | Missing | 1 | `GRADLE-057` | — | — | not started (P2) |
| GRADLE-062 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle/JDK compatibility check | Missing | 5 | `GRADLE-062` | — | — | not started (P0) |
| GRADLE-063 | [java-gradle.md](../spec/features/java-gradle.md) | Daemon JVM criteria (daemon toolchain) | Missing | 1 | `GRADLE-063` | — | — | not started (P2) |
| GRADLE-065 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle user home | Missing | 1 | `GRADLE-065` | — | — | not started (P2) |
| GRADLE-067 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle VM options migration | Missing | 1 | `GRADLE-067` | — | — | not started (P2) |
| GRADLE-068 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle Daemons dialog | Missing | 1 | `GRADLE-068` | — | — | not started (P2) |
| GRADLE-076 | [java-gradle.md](../spec/features/java-gradle.md) | IDE proxy propagation | Missing | 1 | `GRADLE-076` | — | — | not started (P2) |
| GRADLE-083 | [java-gradle.md](../spec/features/java-gradle.md) | Execute Gradle Task (Run Anything) | Missing | 5 | `GRADLE-083` | — | — | not started (P0) |
| GRADLE-084 | [java-gradle.md](../spec/features/java-gradle.md) | Task activation (before/after build, rebuild, sync, run) | Missing | 1 | `GRADLE-084` | — | — | not started (P2) |
| GRADLE-085 | [java-gradle.md](../spec/features/java-gradle.md) | Assign shortcut to task | Missing | 1 | `GRADLE-085` | — | — | not started (P2) |
| GRADLE-086 | [java-gradle.md](../spec/features/java-gradle.md) | Open Gradle Config | Missing | 3 | `GRADLE-086` | — | — | not started (P1) |
| GRADLE-087 | [java-gradle.md](../spec/features/java-gradle.md) | View options | Missing | 3 | `GRADLE-087` | — | — | not started (P1) |
| GRADLE-092 | [java-gradle.md](../spec/features/java-gradle.md) | Build Tools settings shortcut | Missing | 3 | `GRADLE-092` | — | — | not started (P1) |
| GRADLE-093 | [java-gradle.md](../spec/features/java-gradle.md) | Run Configurations node | Missing | 3 | `GRADLE-093` | — | — | not started (P1) |
| GRADLE-104 | [java-gradle.md](../spec/features/java-gradle.md) | HotSwap with Gradle delegation | Missing | 3 | `GRADLE-104` | — | — | not started (P1) |
| GRADLE-111 | [java-gradle.md](../spec/features/java-gradle.md) | Task and argument completion | Missing | 3 | `GRADLE-111` | — | — | not started (P1) |
| GRADLE-112 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle console output | Partial | 5 | `GRADLE-112` | `build.project`, `run.project` | `streams_real_process_output_and_exit_code`, `detects_gradle_build_files_and_prefers_existing_wrapper` | Gradle build/run output streamed to the Build tool window; no dedicated Gradle console |
| GRADLE-115 | [java-gradle.md](../spec/features/java-gradle.md) | Rerun / stop | Partial | 5 | `GRADLE-115` | `run.cancel` | `cancels_running_process` | stop a running Gradle task; rerun pending |
| GRADLE-116 | [java-gradle.md](../spec/features/java-gradle.md) | Continuous build | Missing | 1 | `GRADLE-116` | — | — | not started (P2) |
| GRADLE-117 | [java-gradle.md](../spec/features/java-gradle.md) | Environment / gradle.properties handoff | Missing | 3 | `GRADLE-117` | — | — | not started (P1) |
| GRADLE-121 | [java-gradle.md](../spec/features/java-gradle.md) | Test filter generation | Missing | 5 | `GRADLE-121` | — | — | not started (P0) |
| GRADLE-122 | [java-gradle.md](../spec/features/java-gradle.md) | Rerun failed tests via Gradle | Missing | 3 | `GRADLE-122` | — | — | not started (P1) |
| GRADLE-123 | [java-gradle.md](../spec/features/java-gradle.md) | Debug tests via Gradle | Missing | 5 | `GRADLE-123` | — | — | not started (P0) |
| GRADLE-124 | [java-gradle.md](../spec/features/java-gradle.md) | Up-to-date test rerun | Missing | 3 | `GRADLE-124` | — | — | not started (P1) |
| GRADLE-133 | [java-gradle.md](../spec/features/java-gradle.md) | Completion in dependencies/plugins blocks | Missing | 3 | `GRADLE-133` | — | — | not started (P1) |
| GRADLE-135 | [java-gradle.md](../spec/features/java-gradle.md) | Navigation in build scripts | Missing | 3 | `GRADLE-135` | — | — | not started (P1) |
| GRADLE-136 | [java-gradle.md](../spec/features/java-gradle.md) | Quick documentation for Gradle DSL | Missing | 1 | `GRADLE-136` | — | — | not started (P2) |
| GRADLE-137 | [java-gradle.md](../spec/features/java-gradle.md) | Gutter run icons in build scripts | Missing | 3 | `GRADLE-137` | — | — | not started (P1) |
| GRADLE-138 | [java-gradle.md](../spec/features/java-gradle.md) | `gradle.properties` support | Missing | 1 | `GRADLE-138` | — | — | not started (P2) |
| GRADLE-139 | [java-gradle.md](../spec/features/java-gradle.md) | Settings script support | Missing | 3 | `GRADLE-139` | — | — | not started (P1) |
| GRADLE-148 | [java-gradle.md](../spec/features/java-gradle.md) | Unused version catalog entry inspection | Missing | 1 | `GRADLE-148` | — | — | not started (P2) |
| GRADLE-149 | [java-gradle.md](../spec/features/java-gradle.md) | Rename catalog entries | Missing | 1 | `GRADLE-149` | — | — | not started (P2) |
| GRADLE-160 | [java-gradle.md](../spec/features/java-gradle.md) | Configuration avoidance (`ConfigurationAvoidance`) | Missing | 1 | `GRADLE-160` | — | — | not started (P2) |
| GRADLE-161 | [java-gradle.md](../spec/features/java-gradle.md) | Unrecognized dependency notation (`DependencyNotationArgument`) | Missing | 1 | `GRADLE-161` | — | — | not started (P2) |
| GRADLE-162 | [java-gradle.md](../spec/features/java-gradle.md) | Possibly misplaced call to Gradle method (`ForeignDelegate`) | Missing | 1 | `GRADLE-162` | — | — | not started (P2) |
| GRADLE-163 | [java-gradle.md](../spec/features/java-gradle.md) | Plugin DSL structure (`IncorrectPluginDslStructure`) | Missing | 3 | `GRADLE-163` | — | — | not started (P1) |
| GRADLE-164 | [java-gradle.md](../spec/features/java-gradle.md) | Deprecated configurations (`DeprecatedConfigurations`) | Missing | 3 | `GRADLE-164` | — | — | not started (P1) |
| GRADLE-165 | [java-gradle.md](../spec/features/java-gradle.md) | Redundant Kotlin standard library dependency (`RedundantKotlinStdLibDependency`) | Missing | 1 | `GRADLE-165` | — | — | not started (P2) |
| GRADLE-166 | [java-gradle.md](../spec/features/java-gradle.md) | Named-arguments dependency notation (`AvoidDependencyNamedArgumentsNotation`) | Missing | 1 | `GRADLE-166` | — | — | not started (P2) |
| GRADLE-167 | [java-gradle.md](../spec/features/java-gradle.md) | Legacy `apply` plugin notation (`AvoidApplyPluginMethod`) | Missing | 1 | `GRADLE-167` | — | — | not started (P2) |
| GRADLE-168 | [java-gradle.md](../spec/features/java-gradle.md) | Duplicate dependencies (`AvoidDuplicateDependencies`) | Missing | 1 | `GRADLE-168` | — | — | not started (P2) |
| GRADLE-169 | [java-gradle.md](../spec/features/java-gradle.md) | Duplicate repositories (`AvoidDuplicateRepositories`) | Missing | 1 | `GRADLE-169` | — | — | not started (P2) |
| GRADLE-170 | [java-gradle.md](../spec/features/java-gradle.md) | Describe custom tasks (`TaskMissingDescription`) | Missing | 1 | `GRADLE-170` | — | — | not started (P2) |
| GRADLE-171 | [java-gradle.md](../spec/features/java-gradle.md) | Repositories in build scripts (`AvoidRepositoriesInBuildGradle`) | Missing | 1 | `GRADLE-171` | — | — | not started (P2) |
| GRADLE-172 | [java-gradle.md](../spec/features/java-gradle.md) | Latest Gradle minor version (`LatestMinorVersion`) | Missing | 1 | `GRADLE-172` | — | — | not started (P2) |
| GRADLE-173 | [java-gradle.md](../spec/features/java-gradle.md) | Multiple repository URLs (`MultipleRepositoryUrls`) | Missing | 1 | `GRADLE-173` | — | — | not started (P2) |
| GRADLE-174 | [java-gradle.md](../spec/features/java-gradle.md) | JCenter repository (`JCenterRepository`) | Missing | 1 | `GRADLE-174` | — | — | not started (P2) |
| GRADLE-175 | [java-gradle.md](../spec/features/java-gradle.md) | Bintray publishing plugin (`BintrayPublishingPlugin`) | Missing | 1 | `GRADLE-175` | — | — | not started (P2) |
| GRADLE-176 | [java-gradle.md](../spec/features/java-gradle.md) | Unused version catalog entry | Missing | 1 | `GRADLE-176` | — | — | not started (P2) |
| GRADLE-181 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle user home | Missing | 1 | `GRADLE-181` | — | — | not started (P2) |
| GRADLE-182 | [java-gradle.md](../spec/features/java-gradle.md) | Generate *.iml files for modules imported from Gradle | Missing | 1 | `GRADLE-182` | — | — | not started (P2) |
| GRADLE-183 | [java-gradle.md](../spec/features/java-gradle.md) | Enable parallel Gradle model fetching for Gradle 7.4+ | Missing | 1 | `GRADLE-183` | — | — | not started (P2) |
| GRADLE-184 | [java-gradle.md](../spec/features/java-gradle.md) | Download external annotations for dependencies | Missing | 1 | `GRADLE-184` | — | — | not started (P2) |
| GRADLE-190 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle JVM criteria | Missing | 1 | `GRADLE-190` | — | — | not started (P2) |
| GRADLE-191 | [java-gradle.md](../spec/features/java-gradle.md) | "Use these settings for future projects" | Missing | 1 | `GRADLE-191` | — | — | not started (P2) |
| GRADLE-192 | [java-gradle.md](../spec/features/java-gradle.md) | Generate module per source set | Missing | 1 | `GRADLE-192` | — | — | not started (P2) |
| GRADLE-193 | [java-gradle.md](../spec/features/java-gradle.md) | Gradle VM options | Missing | 1 | `GRADLE-193` | — | — | not started (P2) |
| JAVA-JDK-003 | [java-gradle.md](../spec/features/java-gradle.md) | Module SDK | Missing | 5 | `JAVA-JDK-003` | — | — | not started (P0) |
| JAVA-JDK-007 | [java-gradle.md](../spec/features/java-gradle.md) | Missing SDK banner | Missing | 5 | `JAVA-JDK-007` | — | — | not started (P0) |
| JAVA-JDK-010 | [java-gradle.md](../spec/features/java-gradle.md) | Language-level quick-fixes | Missing | 5 | `JAVA-JDK-010` | — | — | not started (P0) |
| JAVA-JDK-011 | [java-gradle.md](../spec/features/java-gradle.md) | Preview features handling | Missing | 3 | `JAVA-JDK-011` | — | — | not started (P1) |
| JAVA-JDK-013 | [java-gradle.md](../spec/features/java-gradle.md) | Compiler output paths | Missing | 3 | `JAVA-JDK-013` | — | — | not started (P1) |
| JAVA-JDK-015 | [java-gradle.md](../spec/features/java-gradle.md) | Problems page | Missing | 1 | `JAVA-JDK-015` | — | — | not started (P2) |
| JAVA-EDIT-005 | [java-gradle.md](../spec/features/java-gradle.md) | Highlighting level per file | Missing | 3 | `JAVA-EDIT-005` | — | — | not started (P1) |
| JAVA-EDIT-027 | [java-gradle.md](../spec/features/java-gradle.md) | Member/name suggestions | Missing | 5 | `JAVA-EDIT-027` | — | — | not started (P0) |
| JAVA-EDIT-031 | [java-gradle.md](../spec/features/java-gradle.md) | Lookup sorting and filtering options | Missing | 1 | `JAVA-EDIT-031` | — | — | not started (P2) |
| JAVA-EDIT-033 | [java-gradle.md](../spec/features/java-gradle.md) | Exception, annotation, keyword-aware completion | Missing | 3 | `JAVA-EDIT-033` | — | — | not started (P1) |
| JAVA-EDIT-034 | [java-gradle.md](../spec/features/java-gradle.md) | Completion in string literals | Missing | 1 | `JAVA-EDIT-034` | — | — | not started (P2) |
| JAVA-LT-001 | [java-gradle.md](../spec/features/java-gradle.md) | `psvm` / `main` | Missing | 5 | `JAVA-LT-001` | — | — | not started (P0) |
| JAVA-LT-002 | [java-gradle.md](../spec/features/java-gradle.md) | `sout` | Missing | 5 | `JAVA-LT-002` | — | — | not started (P0) |
| JAVA-LT-003 | [java-gradle.md](../spec/features/java-gradle.md) | `soutv` | Missing | 5 | `JAVA-LT-003` | — | — | not started (P0) |
| JAVA-LT-004 | [java-gradle.md](../spec/features/java-gradle.md) | `soutm` | Missing | 3 | `JAVA-LT-004` | — | — | not started (P1) |
| JAVA-LT-005 | [java-gradle.md](../spec/features/java-gradle.md) | `soutp` | Missing | 3 | `JAVA-LT-005` | — | — | not started (P1) |
| JAVA-LT-006 | [java-gradle.md](../spec/features/java-gradle.md) | `souf` | Missing | 3 | `JAVA-LT-006` | — | — | not started (P1) |
| JAVA-LT-007 | [java-gradle.md](../spec/features/java-gradle.md) | `serr` | Missing | 3 | `JAVA-LT-007` | — | — | not started (P1) |
| JAVA-LT-008 | [java-gradle.md](../spec/features/java-gradle.md) | `soutc` / `serrc` | Missing | 1 | `JAVA-LT-008` | — | — | not started (P2) |
| JAVA-LT-009 | [java-gradle.md](../spec/features/java-gradle.md) | `iop` / `ior` | Missing | 1 | `JAVA-LT-009` | — | — | not started (P2) |
| JAVA-LT-010 | [java-gradle.md](../spec/features/java-gradle.md) | `fori` | Missing | 5 | `JAVA-LT-010` | — | — | not started (P0) |
| JAVA-LT-011 | [java-gradle.md](../spec/features/java-gradle.md) | `iter` | Missing | 5 | `JAVA-LT-011` | — | — | not started (P0) |
| JAVA-LT-012 | [java-gradle.md](../spec/features/java-gradle.md) | `itar` | Missing | 3 | `JAVA-LT-012` | — | — | not started (P1) |
| JAVA-LT-013 | [java-gradle.md](../spec/features/java-gradle.md) | `ritar` | Missing | 1 | `JAVA-LT-013` | — | — | not started (P2) |
| JAVA-LT-014 | [java-gradle.md](../spec/features/java-gradle.md) | `itli` | Missing | 3 | `JAVA-LT-014` | — | — | not started (P1) |
| JAVA-LT-015 | [java-gradle.md](../spec/features/java-gradle.md) | `itco` | Missing | 1 | `JAVA-LT-015` | — | — | not started (P2) |
| JAVA-LT-016 | [java-gradle.md](../spec/features/java-gradle.md) | `itit` | Missing | 1 | `JAVA-LT-016` | — | — | not started (P2) |
| JAVA-LT-017 | [java-gradle.md](../spec/features/java-gradle.md) | `iten` | Missing | 1 | `JAVA-LT-017` | — | — | not started (P2) |
| JAVA-LT-018 | [java-gradle.md](../spec/features/java-gradle.md) | `itm` | Missing | 3 | `JAVA-LT-018` | — | — | not started (P1) |
| JAVA-LT-019 | [java-gradle.md](../spec/features/java-gradle.md) | `ittok` | Missing | 1 | `JAVA-LT-019` | — | — | not started (P2) |
| JAVA-LT-020 | [java-gradle.md](../spec/features/java-gradle.md) | `ifn` | Missing | 3 | `JAVA-LT-020` | — | — | not started (P1) |
| JAVA-LT-021 | [java-gradle.md](../spec/features/java-gradle.md) | `inn` | Missing | 3 | `JAVA-LT-021` | — | — | not started (P1) |
| JAVA-LT-022 | [java-gradle.md](../spec/features/java-gradle.md) | `inst` | Missing | 3 | `JAVA-LT-022` | — | — | not started (P1) |
| JAVA-LT-023 | [java-gradle.md](../spec/features/java-gradle.md) | `lst` | Missing | 1 | `JAVA-LT-023` | — | — | not started (P2) |
| JAVA-LT-024 | [java-gradle.md](../spec/features/java-gradle.md) | `mn` / `mx` | Missing | 1 | `JAVA-LT-024` | — | — | not started (P2) |
| JAVA-LT-025 | [java-gradle.md](../spec/features/java-gradle.md) | `toar` | Missing | 1 | `JAVA-LT-025` | — | — | not started (P2) |
| JAVA-LT-026 | [java-gradle.md](../spec/features/java-gradle.md) | `lazy` | Missing | 1 | `JAVA-LT-026` | — | — | not started (P2) |
| JAVA-LT-027 | [java-gradle.md](../spec/features/java-gradle.md) | `geti` | Missing | 1 | `JAVA-LT-027` | — | — | not started (P2) |
| JAVA-LT-028 | [java-gradle.md](../spec/features/java-gradle.md) | `psf`, `prsf`, `psfi`, `psfs` | Missing | 3 | `JAVA-LT-028` | — | — | not started (P1) |
| JAVA-LT-029 | [java-gradle.md](../spec/features/java-gradle.md) | `St` | Missing | 1 | `JAVA-LT-029` | — | — | not started (P2) |
| JAVA-LT-030 | [java-gradle.md](../spec/features/java-gradle.md) | `thr` | Missing | 3 | `JAVA-LT-030` | — | — | not started (P1) |
| JAVA-LT-031 | [java-gradle.md](../spec/features/java-gradle.md) | `else-if` | Missing | 1 | `JAVA-LT-031` | — | — | not started (P2) |
| JAVA-LT-032 | [java-gradle.md](../spec/features/java-gradle.md) | `sts` (+ `stss`, `stsf` preview-level variants) | Missing | 1 | `JAVA-LT-032` | — | — | not started (P2) |
| JAVA-LT-033 | [java-gradle.md](../spec/features/java-gradle.md) | Surround templates `C`, `RL`, `WL`, `I` | Missing | 1 | `JAVA-LT-033` | — | — | not started (P2) |
| JAVA-LT-034 | [java-gradle.md](../spec/features/java-gradle.md) | Platform-wide templates usable in Java | Missing | 1 | `JAVA-LT-034` | — | — | not started (P2) |
| JAVA-PFX-001 | [java-gradle.md](../spec/features/java-gradle.md) | `.var` | Missing | 5 | `JAVA-PFX-001` | — | — | not started (P0) |
| JAVA-PFX-002 | [java-gradle.md](../spec/features/java-gradle.md) | `.if` | Missing | 5 | `JAVA-PFX-002` | — | — | not started (P0) |
| JAVA-PFX-003 | [java-gradle.md](../spec/features/java-gradle.md) | `.not` | Missing | 3 | `JAVA-PFX-003` | — | — | not started (P1) |
| JAVA-PFX-004 | [java-gradle.md](../spec/features/java-gradle.md) | `.for` / `.iter` | Missing | 5 | `JAVA-PFX-004` | — | — | not started (P0) |
| JAVA-PFX-005 | [java-gradle.md](../spec/features/java-gradle.md) | `.fori` / `.forr` | Missing | 3 | `JAVA-PFX-005` | — | — | not started (P1) |
| JAVA-PFX-006 | [java-gradle.md](../spec/features/java-gradle.md) | `.null` / `.nn` (`.notnull`) | Missing | 3 | `JAVA-PFX-006` | — | — | not started (P1) |
| JAVA-PFX-007 | [java-gradle.md](../spec/features/java-gradle.md) | `.return` | Missing | 3 | `JAVA-PFX-007` | — | — | not started (P1) |
| JAVA-PFX-008 | [java-gradle.md](../spec/features/java-gradle.md) | `.sout` / `.serr` / `.soutv` / `.souf` | Missing | 5 | `JAVA-PFX-008` | — | — | not started (P0) |
| JAVA-PFX-009 | [java-gradle.md](../spec/features/java-gradle.md) | `.cast` / `.castvar` | Missing | 3 | `JAVA-PFX-009` | — | — | not started (P1) |
| JAVA-PFX-010 | [java-gradle.md](../spec/features/java-gradle.md) | `.inst` / `.instanceof` | Missing | 3 | `JAVA-PFX-010` | — | — | not started (P1) |
| JAVA-PFX-011 | [java-gradle.md](../spec/features/java-gradle.md) | `.field` | Missing | 3 | `JAVA-PFX-011` | — | — | not started (P1) |
| JAVA-PFX-012 | [java-gradle.md](../spec/features/java-gradle.md) | `.par` | Missing | 1 | `JAVA-PFX-012` | — | — | not started (P2) |
| JAVA-PFX-013 | [java-gradle.md](../spec/features/java-gradle.md) | `.new` | Missing | 1 | `JAVA-PFX-013` | — | — | not started (P2) |
| JAVA-PFX-014 | [java-gradle.md](../spec/features/java-gradle.md) | `.arg` | Missing | 1 | `JAVA-PFX-014` | — | — | not started (P2) |
| JAVA-PFX-015 | [java-gradle.md](../spec/features/java-gradle.md) | `.lambda` | Missing | 1 | `JAVA-PFX-015` | — | — | not started (P2) |
| JAVA-PFX-016 | [java-gradle.md](../spec/features/java-gradle.md) | `.opt` | Missing | 1 | `JAVA-PFX-016` | — | — | not started (P2) |
| JAVA-PFX-017 | [java-gradle.md](../spec/features/java-gradle.md) | `.stream` | Missing | 1 | `JAVA-PFX-017` | — | — | not started (P2) |
| JAVA-PFX-018 | [java-gradle.md](../spec/features/java-gradle.md) | `.switch` | Missing | 3 | `JAVA-PFX-018` | — | — | not started (P1) |
| JAVA-PFX-019 | [java-gradle.md](../spec/features/java-gradle.md) | `.synchronized` | Missing | 1 | `JAVA-PFX-019` | — | — | not started (P2) |
| JAVA-PFX-020 | [java-gradle.md](../spec/features/java-gradle.md) | `.throw` | Missing | 3 | `JAVA-PFX-020` | — | — | not started (P1) |
| JAVA-PFX-021 | [java-gradle.md](../spec/features/java-gradle.md) | `.try` / `.twr` | Missing | 3 | `JAVA-PFX-021` | — | — | not started (P1) |
| JAVA-PFX-022 | [java-gradle.md](../spec/features/java-gradle.md) | `.while` / `.else` | Missing | 1 | `JAVA-PFX-022` | — | — | not started (P2) |
| JAVA-PFX-023 | [java-gradle.md](../spec/features/java-gradle.md) | `.format` | Missing | 1 | `JAVA-PFX-023` | — | — | not started (P2) |
| JAVA-PFX-024 | [java-gradle.md](../spec/features/java-gradle.md) | `.reqnonnull` | Missing | 1 | `JAVA-PFX-024` | — | — | not started (P2) |
| JAVA-PFX-025 | [java-gradle.md](../spec/features/java-gradle.md) | `.assert` | Missing | 1 | `JAVA-PFX-025` | — | — | not started (P2) |
| JAVA-PFX-026 | [java-gradle.md](../spec/features/java-gradle.md) | `.serr` / `.soutv` variants and `.optional` aliases (inferred; exact key list to be verified, see Open questions) | Missing | 1 | `JAVA-PFX-026` | — | — | not started (P2) |
| JAVA-EDIT-041 | [java-gradle.md](../spec/features/java-gradle.md) | Overload switching | Missing | 1 | `JAVA-EDIT-041` | — | — | not started (P2) |
| JAVA-EDIT-043 | [java-gradle.md](../spec/features/java-gradle.md) | External documentation | Missing | 1 | `JAVA-EDIT-043` | — | — | not started (P2) |
| JAVA-EDIT-046 | [java-gradle.md](../spec/features/java-gradle.md) | Context info | Missing | 1 | `JAVA-EDIT-046` | — | — | not started (P2) |
| JAVA-EDIT-051 | [java-gradle.md](../spec/features/java-gradle.md) | Implicit type hints for `var` | Missing | 1 | `JAVA-EDIT-051` | — | — | not started (P2) |
| JAVA-EDIT-052 | [java-gradle.md](../spec/features/java-gradle.md) | Method chain type hints | Missing | 1 | `JAVA-EDIT-052` | — | — | not started (P2) |
| JAVA-EDIT-053 | [java-gradle.md](../spec/features/java-gradle.md) | Annotation hints | Missing | 1 | `JAVA-EDIT-053` | — | — | not started (P2) |
| JAVA-EDIT-055 | [java-gradle.md](../spec/features/java-gradle.md) | Code vision: inheritors / implementations | Missing | 3 | `JAVA-EDIT-055` | — | — | not started (P1) |
| JAVA-EDIT-056 | [java-gradle.md](../spec/features/java-gradle.md) | Code vision: related problems | Missing | 1 | `JAVA-EDIT-056` | — | — | not started (P2) |
| JAVA-EDIT-057 | [java-gradle.md](../spec/features/java-gradle.md) | Code vision: VCS author | Missing | 1 | `JAVA-EDIT-057` | — | — | not started (P2) |
| JAVA-EDIT-058 | [java-gradle.md](../spec/features/java-gradle.md) | Code vision: JVM language/test status hints | Missing | 1 | `JAVA-EDIT-058` | — | — | not started (P2) |
| JAVA-EDIT-059 | [java-gradle.md](../spec/features/java-gradle.md) | Value hints for `switch`/`yield` & lambda return (inferred) | Missing | 1 | `JAVA-EDIT-059` | — | — | not started (P2) |
| JAVA-EDIT-062 | [java-gradle.md](../spec/features/java-gradle.md) | Closure-style folding of anonymous classes / lambdas | Missing | 1 | `JAVA-EDIT-062` | — | — | not started (P2) |
| JAVA-EDIT-063 | [java-gradle.md](../spec/features/java-gradle.md) | Getter/setter one-liners folding | Missing | 1 | `JAVA-EDIT-063` | — | — | not started (P2) |
| JAVA-EDIT-064 | [java-gradle.md](../spec/features/java-gradle.md) | Generic constructor parameters, `final var` and i18n string folding | Missing | 1 | `JAVA-EDIT-064` | — | — | not started (P2) |
| JAVA-EDIT-070 | [java-gradle.md](../spec/features/java-gradle.md) | Import popup / quick-fix | Missing | 5 | `JAVA-EDIT-070` | — | — | not started (P0) |
| JAVA-EDIT-073 | [java-gradle.md](../spec/features/java-gradle.md) | Optimize imports on the fly | Missing | 3 | `JAVA-EDIT-073` | — | — | not started (P1) |
| JAVA-EDIT-075 | [java-gradle.md](../spec/features/java-gradle.md) | Exclude from auto-import | Missing | 3 | `JAVA-EDIT-075` | — | — | not started (P1) |
| JAVA-EDIT-076 | [java-gradle.md](../spec/features/java-gradle.md) | Static import intentions | Missing | 3 | `JAVA-EDIT-076` | — | — | not started (P1) |
| JAVA-EDIT-079 | [java-gradle.md](../spec/features/java-gradle.md) | Module imports (`import module`) | Missing | 1 | `JAVA-EDIT-079` | — | — | not started (P2) |
| JAVA-FMT-002 | [java-gradle.md](../spec/features/java-gradle.md) | Reformat File dialog | Missing | 3 | `JAVA-FMT-002` | — | — | not started (P1) |
| JAVA-FMT-009 | [java-gradle.md](../spec/features/java-gradle.md) | Code style: JavaDoc | Missing | 1 | `JAVA-FMT-009` | — | — | not started (P2) |
| JAVA-FMT-010 | [java-gradle.md](../spec/features/java-gradle.md) | Code style: Imports | Missing | 3 | `JAVA-FMT-010` | — | — | not started (P1) |
| JAVA-FMT-011 | [java-gradle.md](../spec/features/java-gradle.md) | Code style: Arrangement | Missing | 1 | `JAVA-FMT-011` | — | — | not started (P2) |
| JAVA-FMT-012 | [java-gradle.md](../spec/features/java-gradle.md) | Code style: Code Generation | Missing | 3 | `JAVA-FMT-012` | — | — | not started (P1) |
| JAVA-FMT-013 | [java-gradle.md](../spec/features/java-gradle.md) | Code style: Wrapping of records, switch, text blocks, lambdas | Missing | 1 | `JAVA-FMT-013` | — | — | not started (P2) |
| JAVA-FMT-016 | [java-gradle.md](../spec/features/java-gradle.md) | Formatter markers | Missing | 1 | `JAVA-FMT-016` | — | — | not started (P2) |
| JAVA-FMT-024 | [java-gradle.md](../spec/features/java-gradle.md) | Unwrap/Remove | Missing | 3 | `JAVA-FMT-024` | — | — | not started (P1) |
| JAVA-FMT-027 | [java-gradle.md](../spec/features/java-gradle.md) | Code Cleanup | Missing | 1 | `JAVA-FMT-027` | — | — | not started (P2) |
| JAVA-EDIT-084 | [java-gradle.md](../spec/features/java-gradle.md) | Markdown documentation comments (`///`) | Missing | 1 | `JAVA-EDIT-084` | — | — | not started (P2) |
| JAVA-EDIT-085 | [java-gradle.md](../spec/features/java-gradle.md) | Javadoc inspections | Missing | 1 | `JAVA-EDIT-085` | — | — | not started (P2) |
| JAVA-EDIT-087 | [java-gradle.md](../spec/features/java-gradle.md) | Javadoc snippets (`{@snippet}`) | Missing | 1 | `JAVA-EDIT-087` | — | — | not started (P2) |
| JAVA-LANG-004 | [java-gradle.md](../spec/features/java-gradle.md) | Switch expressions & pattern switch | Missing | 5 | `JAVA-LANG-004` | — | — | not started (P0) |
| JAVA-LANG-005 | [java-gradle.md](../spec/features/java-gradle.md) | Record patterns | Missing | 3 | `JAVA-LANG-005` | — | — | not started (P1) |
| JAVA-LANG-007 | [java-gradle.md](../spec/features/java-gradle.md) | `var` local type inference | Missing | 5 | `JAVA-LANG-007` | — | — | not started (P0) |
| JAVA-LANG-008 | [java-gradle.md](../spec/features/java-gradle.md) | Unnamed variables/patterns (`_`) | Missing | 3 | `JAVA-LANG-008` | — | — | not started (P1) |
| JAVA-LANG-009 | [java-gradle.md](../spec/features/java-gradle.md) | Implicitly declared classes & instance main | Missing | 3 | `JAVA-LANG-009` | — | — | not started (P1) |
| JAVA-LANG-010 | [java-gradle.md](../spec/features/java-gradle.md) | Primitive patterns, flexible constructor bodies, module imports, scoped values, structured concurrency, stable/lazy values | Missing | 3 | `JAVA-LANG-010` | — | — | not started (P1) |
| JAVA-LANG-011 | [java-gradle.md](../spec/features/java-gradle.md) | Lambdas and method references | Missing | 5 | `JAVA-LANG-011` | — | — | not started (P0) |
| JAVA-LANG-012 | [java-gradle.md](../spec/features/java-gradle.md) | Streams | Missing | 3 | `JAVA-LANG-012` | — | — | not started (P1) |
| JAVA-LANG-013 | [java-gradle.md](../spec/features/java-gradle.md) | Generics | Missing | 5 | `JAVA-LANG-013` | — | — | not started (P0) |
| JAVA-LANG-016 | [java-gradle.md](../spec/features/java-gradle.md) | String templates | Missing | 1 | `JAVA-LANG-016` | — | — | not started (P2) |
| JAVA-LANG-017 | [java-gradle.md](../spec/features/java-gradle.md) | Data-flow analysis | Missing | 5 | `JAVA-LANG-017` | — | — | not started (P0) |
| JAVA-LOMBOK-003 | [java-gradle.md](../spec/features/java-gradle.md) | Lombok inspections | Missing | 1 | `JAVA-LOMBOK-003` | — | — | not started (P2) |
| JAVA-LOMBOK-004 | [java-gradle.md](../spec/features/java-gradle.md) | Delombok | Missing | 1 | `JAVA-LOMBOK-004` | — | — | not started (P2) |
| JAVA-LOMBOK-005 | [java-gradle.md](../spec/features/java-gradle.md) | Lombokify intentions | Missing | 1 | `JAVA-LOMBOK-005` | — | — | not started (P2) |
| JAVA-LOMBOK-006 | [java-gradle.md](../spec/features/java-gradle.md) | Annotation processing prompt | Missing | 3 | `JAVA-LOMBOK-006` | — | — | not started (P1) |
| JAVA-LOMBOK-007 | [java-gradle.md](../spec/features/java-gradle.md) | Generated members in Structure view | Missing | 1 | `JAVA-LOMBOK-007` | — | — | not started (P2) |
| JAVA-NAV-006 | [java-gradle.md](../spec/features/java-gradle.md) | Go to Text | Missing | 3 | `JAVA-NAV-006` | — | — | not started (P1) |
| JAVA-NAV-043 | [java-gradle.md](../spec/features/java-gradle.md) | Jump to source from tool windows | Partial | 5 | `JAVA-NAV-043` | `editor.openFile` | `project_search_command_streams_results_into_state` | see PLATFORM-NAV-031 |
| JAVA-NAV-046 | [java-gradle.md](../spec/features/java-gradle.md) | Analyze Stack Trace or Thread Dump | Missing | 1 | `JAVA-NAV-046` | — | — | not started (P2) |
| JAVA-INSP-008 | [java-gradle.md](../spec/features/java-gradle.md) | Batch fix in file | Missing | 3 | `JAVA-INSP-008` | — | — | not started (P1) |
| JAVA-INSP-012 | [java-gradle.md](../spec/features/java-gradle.md) | Inspect code with editor settings | Missing | 1 | `JAVA-INSP-012` | — | — | not started (P2) |
| JAVA-INSP-013 | [java-gradle.md](../spec/features/java-gradle.md) | Global inspections | Missing | 1 | `JAVA-INSP-013` | — | — | not started (P2) |
| JAVA-INSP-015 | [java-gradle.md](../spec/features/java-gradle.md) | Code Cleanup | Missing | 1 | `JAVA-INSP-015` | — | — | not started (P2) |
| JAVA-INSP-016 | [java-gradle.md](../spec/features/java-gradle.md) | Silent code cleanup on save / commit | Missing | 1 | `JAVA-INSP-016` | — | — | not started (P2) |
| JAVA-INSP-017 | [java-gradle.md](../spec/features/java-gradle.md) | Command-line inspector | Missing | 1 | `JAVA-INSP-017` | — | — | not started (P2) |
| JAVA-INSP-018 | [java-gradle.md](../spec/features/java-gradle.md) | Inspection descriptions | Missing | 3 | `JAVA-INSP-018` | — | — | not started (P1) |
| JAVA-INSP-031 | [java-gradle.md](../spec/features/java-gradle.md) | Language-level gating | Missing | 5 | `JAVA-INSP-031` | — | — | not started (P0) |
| JAVA-INSP-032 | [java-gradle.md](../spec/features/java-gradle.md) | Test-source awareness | Missing | 3 | `JAVA-INSP-032` | — | — | not started (P1) |
| JAVA-INSP-102 | [java-gradle.md](../spec/features/java-gradle.md) | `UNUSED_IMPORT` | Missing | 5 | `JAVA-INSP-102` | — | — | not started (P0) |
| JAVA-INSP-104 | [java-gradle.md](../spec/features/java-gradle.md) | `ConstantValue` | Missing | 5 | `JAVA-INSP-104` | — | — | not started (P0) |
| JAVA-INSP-105 | [java-gradle.md](../spec/features/java-gradle.md) | `UnusedAssignment` | Missing | 5 | `JAVA-INSP-105` | — | — | not started (P0) |
| JAVA-INSP-106 | [java-gradle.md](../spec/features/java-gradle.md) | `WrongPackageStatement` | Missing | 5 | `JAVA-INSP-106` | — | — | not started (P0) |
| JAVA-INSP-107 | [java-gradle.md](../spec/features/java-gradle.md) | `UNCHECKED_WARNING` | Missing | 5 | `JAVA-INSP-107` | — | — | not started (P0) |
| JAVA-INSP-108 | [java-gradle.md](../spec/features/java-gradle.md) | `RedundantCast` | Missing | 3 | `JAVA-INSP-108` | — | — | not started (P1) |
| JAVA-INSP-109 | [java-gradle.md](../spec/features/java-gradle.md) | `StringEquality` | Missing | 5 | `JAVA-INSP-109` | — | — | not started (P0) |
| JAVA-INSP-110 | [java-gradle.md](../spec/features/java-gradle.md) | `NumberEquality` | Missing | 3 | `JAVA-INSP-110` | — | — | not started (P1) |
| JAVA-INSP-111 | [java-gradle.md](../spec/features/java-gradle.md) | `EqualsBetweenInconvertibleTypes` | Missing | 3 | `JAVA-INSP-111` | — | — | not started (P1) |
| JAVA-INSP-112 | [java-gradle.md](../spec/features/java-gradle.md) | `IgnoreResultOfCall` | Missing | 3 | `JAVA-INSP-112` | — | — | not started (P1) |
| JAVA-INSP-113 | [java-gradle.md](../spec/features/java-gradle.md) | `OptionalGetWithoutIsPresent` | Missing | 3 | `JAVA-INSP-113` | — | — | not started (P1) |
| JAVA-INSP-114 | [java-gradle.md](../spec/features/java-gradle.md) | `MismatchedCollectionQueryUpdate` | Missing | 3 | `JAVA-INSP-114` | — | — | not started (P1) |
| JAVA-INSP-115 | [java-gradle.md](../spec/features/java-gradle.md) | `InfiniteRecursion` | Missing | 3 | `JAVA-INSP-115` | — | — | not started (P1) |
| JAVA-INSP-116 | [java-gradle.md](../spec/features/java-gradle.md) | `MalformedFormatString` | Missing | 3 | `JAVA-INSP-116` | — | — | not started (P1) |
| JAVA-INSP-117 | [java-gradle.md](../spec/features/java-gradle.md) | `SuspiciousMethodCalls` | Missing | 3 | `JAVA-INSP-117` | — | — | not started (P1) |
| JAVA-INSP-118 | [java-gradle.md](../spec/features/java-gradle.md) | `ArrayEquals` | Missing | 3 | `JAVA-INSP-118` | — | — | not started (P1) |
| JAVA-INSP-119 | [java-gradle.md](../spec/features/java-gradle.md) | `ImplicitArrayToString` | Missing | 3 | `JAVA-INSP-119` | — | — | not started (P1) |
| JAVA-INSP-120 | [java-gradle.md](../spec/features/java-gradle.md) | `ThrowableNotThrown` | Missing | 3 | `JAVA-INSP-120` | — | — | not started (P1) |
| JAVA-INSP-121 | [java-gradle.md](../spec/features/java-gradle.md) | `MagicConstant` | Missing | 1 | `JAVA-INSP-121` | — | — | not started (P2) |
| JAVA-INSP-122 | [java-gradle.md](../spec/features/java-gradle.md) | `Contract` | Missing | 1 | `JAVA-INSP-122` | — | — | not started (P2) |
| JAVA-INSP-124 | [java-gradle.md](../spec/features/java-gradle.md) | `NotNullFieldNotInitialized` | Missing | 1 | `JAVA-INSP-124` | — | — | not started (P2) |
| JAVA-INSP-125 | [java-gradle.md](../spec/features/java-gradle.md) | `PointlessBooleanExpression` | Missing | 3 | `JAVA-INSP-125` | — | — | not started (P1) |
| JAVA-INSP-126 | [java-gradle.md](../spec/features/java-gradle.md) | `SimplifiableConditionalExpression` | Missing | 3 | `JAVA-INSP-126` | — | — | not started (P1) |
| JAVA-INSP-127 | [java-gradle.md](../spec/features/java-gradle.md) | `TrivialIf` | Missing | 3 | `JAVA-INSP-127` | — | — | not started (P1) |
| JAVA-INSP-128 | [java-gradle.md](../spec/features/java-gradle.md) | `LoopStatementsThatDontLoop` | Missing | 1 | `JAVA-INSP-128` | — | — | not started (P2) |
| JAVA-INSP-129 | [java-gradle.md](../spec/features/java-gradle.md) | `InfiniteLoopStatement` | Missing | 1 | `JAVA-INSP-129` | — | — | not started (P2) |
| JAVA-INSP-130 | [java-gradle.md](../spec/features/java-gradle.md) | `DuplicateCondition` | Missing | 1 | `JAVA-INSP-130` | — | — | not started (P2) |
| JAVA-INSP-131 | [java-gradle.md](../spec/features/java-gradle.md) | `UnnecessaryLocalVariable` | Missing | 3 | `JAVA-INSP-131` | — | — | not started (P1) |
| JAVA-INSP-132 | [java-gradle.md](../spec/features/java-gradle.md) | `CatchMayIgnoreException` | Missing | 3 | `JAVA-INSP-132` | — | — | not started (P1) |
| JAVA-INSP-133 | [java-gradle.md](../spec/features/java-gradle.md) | `ReturnFromFinallyBlock` | Missing | 1 | `JAVA-INSP-133` | — | — | not started (P2) |
| JAVA-INSP-134 | [java-gradle.md](../spec/features/java-gradle.md) | `AutoCloseableResource` | Missing | 3 | `JAVA-INSP-134` | — | — | not started (P1) |
| JAVA-INSP-135 | [java-gradle.md](../spec/features/java-gradle.md) | `Convert2Lambda` | Missing | 3 | `JAVA-INSP-135` | — | — | not started (P1) |
| JAVA-INSP-136 | [java-gradle.md](../spec/features/java-gradle.md) | `Convert2MethodRef` | Missing | 3 | `JAVA-INSP-136` | — | — | not started (P1) |
| JAVA-INSP-137 | [java-gradle.md](../spec/features/java-gradle.md) | `Convert2Diamond` | Missing | 3 | `JAVA-INSP-137` | — | — | not started (P1) |
| JAVA-INSP-138 | [java-gradle.md](../spec/features/java-gradle.md) | `TryFinallyCanBeTryWithResources` | Missing | 1 | `JAVA-INSP-138` | — | — | not started (P2) |
| JAVA-INSP-139 | [java-gradle.md](../spec/features/java-gradle.md) | `ForCanBeForeach` | Missing | 1 | `JAVA-INSP-139` | — | — | not started (P2) |
| JAVA-INSP-140 | [java-gradle.md](../spec/features/java-gradle.md) | `RawUseOfParameterizedType` | Missing | 3 | `JAVA-INSP-140` | — | — | not started (P1) |
| JAVA-INSP-141 | [java-gradle.md](../spec/features/java-gradle.md) | `UnnecessaryBoxing` / `UnnecessaryUnboxing` | Missing | 1 | `JAVA-INSP-141` | — | — | not started (P2) |
| JAVA-INSP-142 | [java-gradle.md](../spec/features/java-gradle.md) | `IfCanBeSwitch` | Missing | 1 | `JAVA-INSP-142` | — | — | not started (P2) |
| JAVA-INSP-143 | [java-gradle.md](../spec/features/java-gradle.md) | `SimplifyStreamApiCallChains` | Missing | 1 | `JAVA-INSP-143` | — | — | not started (P2) |
| JAVA-INSP-144 | [java-gradle.md](../spec/features/java-gradle.md) | `RedundantThrows` | Missing | 1 | `JAVA-INSP-144` | — | — | not started (P2) |
| JAVA-INSP-145 | [java-gradle.md](../spec/features/java-gradle.md) | `SameParameterValue` | Missing | 1 | `JAVA-INSP-145` | — | — | not started (P2) |
| JAVA-INSP-146 | [java-gradle.md](../spec/features/java-gradle.md) | `UnusedReturnValue` | Missing | 1 | `JAVA-INSP-146` | — | — | not started (P2) |
| JAVA-INSP-147 | [java-gradle.md](../spec/features/java-gradle.md) | `AccessStaticViaInstance` | Missing | 1 | `JAVA-INSP-147` | — | — | not started (P2) |
| JAVA-INSP-148 | [java-gradle.md](../spec/features/java-gradle.md) | `FieldCanBeLocal` | Missing | 1 | `JAVA-INSP-148` | — | — | not started (P2) |
| JAVA-INSP-149 | [java-gradle.md](../spec/features/java-gradle.md) | `RedundantMethodOverride` | Missing | 1 | `JAVA-INSP-149` | — | — | not started (P2) |
| JAVA-INSP-150 | [java-gradle.md](../spec/features/java-gradle.md) | `NumericOverflow` | Missing | 1 | `JAVA-INSP-150` | — | — | not started (P2) |
| JAVA-INSP-151 | [java-gradle.md](../spec/features/java-gradle.md) | `IntegerDivisionInFloatingPointContext` | Missing | 1 | `JAVA-INSP-151` | — | — | not started (P2) |
| JAVA-INSP-152 | [java-gradle.md](../spec/features/java-gradle.md) | `DivideByZero` | Missing | 1 | `JAVA-INSP-152` | — | — | not started (P2) |
| JAVA-INSP-153 | [java-gradle.md](../spec/features/java-gradle.md) | `JUnitMalformedDeclaration` | Missing | 3 | `JAVA-INSP-153` | — | — | not started (P1) |
| JAVA-INSP-154 | [java-gradle.md](../spec/features/java-gradle.md) | `JUnit5AssertionsConverter` | Missing | 1 | `JAVA-INSP-154` | — | — | not started (P2) |
| JAVA-INSP-155 | [java-gradle.md](../spec/features/java-gradle.md) | `SerialVersionUIDNotStaticFinal` | Missing | 1 | `JAVA-INSP-155` | — | — | not started (P2) |
| JAVA-INSP-156 | [java-gradle.md](../spec/features/java-gradle.md) | `PreviewFeature` | Missing | 3 | `JAVA-INSP-156` | — | — | not started (P1) |
| JAVA-INSP-157 | [java-gradle.md](../spec/features/java-gradle.md) | `JavaModuleDefinition` | Missing | 1 | `JAVA-INSP-157` | — | — | not started (P2) |
| JAVA-INSP-158 | [java-gradle.md](../spec/features/java-gradle.md) | `InconsistentTextBlockIndent` | Missing | 1 | `JAVA-INSP-158` | — | — | not started (P2) |
| JAVA-INSP-159 | [java-gradle.md](../spec/features/java-gradle.md) | `UnnecessaryReturn` / `UnnecessaryContinue` / `UnnecessaryBreak` | Missing | 1 | `JAVA-INSP-159` | — | — | not started (P2) |
| JAVA-INSP-160 | [java-gradle.md](../spec/features/java-gradle.md) | `RedundantStreamOptionalCall` | Missing | 1 | `JAVA-INSP-160` | — | — | not started (P2) |
| JAVA-INSP-161 | [java-gradle.md](../spec/features/java-gradle.md) | `MissingSerialAnnotation` | Missing | 1 | `JAVA-INSP-161` | — | — | not started (P2) |
| JAVA-INSP-162 | [java-gradle.md](../spec/features/java-gradle.md) | `ReplaceAssignmentWithOperatorAssignment` | Missing | 1 | `JAVA-INSP-162` | — | — | not started (P2) |
| JAVA-INSP-202 | [java-gradle.md](../spec/features/java-gradle.md) | Import class / static member (see JAVA-EDIT-070) | Missing | 5 | `JAVA-INSP-202` | — | — | not started (P0) |
| JAVA-INSP-205 | [java-gradle.md](../spec/features/java-gradle.md) | Implement methods / make class abstract (for missing abstract implementations) | Missing | 5 | `JAVA-INSP-205` | — | — | not started (P0) |
| JAVA-INSP-206 | [java-gradle.md](../spec/features/java-gradle.md) | Add missing return / initialize variable / remove unreachable code | Missing | 3 | `JAVA-INSP-206` | — | — | not started (P1) |
| JAVA-INSP-207 | [java-gradle.md](../spec/features/java-gradle.md) | Invert 'if' condition; Split into declaration and assignment; Join declaration and assignment; Split 'if'; Merge nested 'if' | Missing | 3 | `JAVA-INSP-207` | — | — | not started (P1) |
| JAVA-INSP-208 | [java-gradle.md](../spec/features/java-gradle.md) | Flip comparison / Flip binary expression / Replace with `equals`, De Morgan conversions | Missing | 1 | `JAVA-INSP-208` | — | — | not started (P2) |
| JAVA-INSP-209 | [java-gradle.md](../spec/features/java-gradle.md) | Convert to switch expression / enhanced switch / create missing switch branches | Missing | 3 | `JAVA-INSP-209` | — | — | not started (P1) |
| JAVA-INSP-210 | [java-gradle.md](../spec/features/java-gradle.md) | Introduce local variable / Extract method (intention forms) | Missing | 3 | `JAVA-INSP-210` | — | — | not started (P1) |
| JAVA-INSP-211 | [java-gradle.md](../spec/features/java-gradle.md) | Bind constructor parameters to fields / Create field for parameter / Assign parameter to field | Missing | 3 | `JAVA-INSP-211` | — | — | not started (P1) |
| JAVA-INSP-212 | [java-gradle.md](../spec/features/java-gradle.md) | Replace with for-each / stream ↔ loop conversions (`Replace with forEach`, `Replace Stream API chain with loop`) | Missing | 1 | `JAVA-INSP-212` | — | — | not started (P2) |
| JAVA-INSP-213 | [java-gradle.md](../spec/features/java-gradle.md) | Annotate as @NotNull/@Nullable, Add `@Override`, Annotate deprecated, Make external annotation explicit | Missing | 1 | `JAVA-INSP-213` | — | — | not started (P2) |
| JAVA-INSP-214 | [java-gradle.md](../spec/features/java-gradle.md) | Replace concatenation with text block / `String.format` / `MessageFormat` / `StringBuilder` | Missing | 1 | `JAVA-INSP-214` | — | — | not started (P2) |
| JAVA-INSP-215 | [java-gradle.md](../spec/features/java-gradle.md) | Create test (from class) / Create missing tests | Missing | 3 | `JAVA-INSP-215` | — | — | not started (P1) |
| JAVA-INSP-216 | [java-gradle.md](../spec/features/java-gradle.md) | Create subclass / Implement interface / Seal class / Unimplement interface | Missing | 1 | `JAVA-INSP-216` | — | — | not started (P2) |
| JAVA-INSP-217 | [java-gradle.md](../spec/features/java-gradle.md) | Encapsulate field / Replace constructor with factory or builder (intention entry points of refactorings) | Missing | 1 | `JAVA-INSP-217` | — | — | not started (P2) |
| JAVA-INSP-218 | [java-gradle.md](../spec/features/java-gradle.md) | Add Javadoc / Fix doc comment | Missing | 1 | `JAVA-INSP-218` | — | — | not started (P2) |
| JAVA-INSP-219 | [java-gradle.md](../spec/features/java-gradle.md) | Convert compact ↔ canonical record constructor; Convert class ↔ record | Missing | 1 | `JAVA-INSP-219` | — | — | not started (P2) |
| JAVA-INSP-220 | [java-gradle.md](../spec/features/java-gradle.md) | Set language level / Add library to classpath / Add dependency (for Gradle modules may edit build script — inferred) | Missing | 5 | `JAVA-INSP-220` | — | — | not started (P0) |
| JAVA-INSP-221 | [java-gradle.md](../spec/features/java-gradle.md) | Iterate over iterable; Unroll loop; Collapse into loop; Move into if branches; Extract if condition | Missing | 1 | `JAVA-INSP-221` | — | — | not started (P2) |
| JAVA-INSP-222 | [java-gradle.md](../spec/features/java-gradle.md) | Inject language or reference; Edit regex fragment / Check RegExp | Missing | 1 | `JAVA-INSP-222` | — | — | not started (P2) |
| JAVA-INSP-223 | [java-gradle.md](../spec/features/java-gradle.md) | Change serialVersionUID; Sort content (enum constants, array literal); Wrap with unmodifiable collection | Missing | 1 | `JAVA-INSP-223` | — | — | not started (P2) |
| JAVA-REF-011 | [java-gradle.md](../spec/features/java-gradle.md) | Introduce Functional Variable | Missing | 1 | `JAVA-REF-011` | — | — | not started (P2) |
| JAVA-REF-039 | [java-gradle.md](../spec/features/java-gradle.md) | Replace Temp with Query | Missing | 1 | `JAVA-REF-039` | — | — | not started (P2) |
| JAVA-REF-041 | [java-gradle.md](../spec/features/java-gradle.md) | Extract Module | Missing | 1 | `JAVA-REF-041` | — | — | not started (P2) |
| JAVA-REF-043 | [java-gradle.md](../spec/features/java-gradle.md) | Delombok / Lombok refactorings | Missing | 1 | `JAVA-REF-043` | — | — | not started (P2) |
| JAVA-REF-044 | [java-gradle.md](../spec/features/java-gradle.md) | I18nize hard-coded string | Missing | 1 | `JAVA-REF-044` | — | — | not started (P2) |
| JAVA-REF-048 | [java-gradle.md](../spec/features/java-gradle.md) | Drag-and-drop move | Missing | 3 | `JAVA-REF-048` | — | — | not started (P1) |
| JAVA-REF-049 | [java-gradle.md](../spec/features/java-gradle.md) | Paste-time refactoring hints | Missing | 1 | `JAVA-REF-049` | — | — | not started (P2) |
| JAVA-GEN-010 | [java-gradle.md](../spec/features/java-gradle.md) | Delegate Methods | Missing | 1 | `JAVA-GEN-010` | — | — | not started (P2) |
| JAVA-GEN-011 | [java-gradle.md](../spec/features/java-gradle.md) | Super Method Call | Missing | 1 | `JAVA-GEN-011` | — | — | not started (P2) |
| JAVA-GEN-013 | [java-gradle.md](../spec/features/java-gradle.md) | Test Method / SetUp / TearDown / BeforeClass / AfterClass / Parameters Method (in test classes) | Missing | 3 | `JAVA-GEN-013` | — | — | not started (P1) |
| JAVA-GEN-014 | [java-gradle.md](../spec/features/java-gradle.md) | Logger | Missing | 1 | `JAVA-GEN-014` | — | — | not started (P2) |
| JAVA-GEN-015 | [java-gradle.md](../spec/features/java-gradle.md) | Copyright | Missing | 1 | `JAVA-GEN-015` | — | — | not started (P2) |
| JAVA-GEN-016 | [java-gradle.md](../spec/features/java-gradle.md) | Generate module-info descriptors | Missing | 1 | `JAVA-GEN-016` | — | — | not started (P2) |
| JAVA-GEN-017 | [java-gradle.md](../spec/features/java-gradle.md) | Javadoc generation intention | Missing | 1 | `JAVA-GEN-017` | — | — | not started (P2) |
| JAVA-GEN-018 | [java-gradle.md](../spec/features/java-gradle.md) | serialVersionUID | Missing | 1 | `JAVA-GEN-018` | — | — | not started (P2) |
| JAVA-GEN-022 | [java-gradle.md](../spec/features/java-gradle.md) | Lombok-aware generation | Missing | 1 | `JAVA-GEN-022` | — | — | not started (P2) |
| JAVA-GEN-023 | [java-gradle.md](../spec/features/java-gradle.md) | Generate code from usage | Missing | 5 | `JAVA-GEN-023` | — | — | not started (P0) |
| JAVA-RUN-021 | [java-gradle.md](../spec/features/java-gradle.md) | Run configuration from Gradle `application` plugin | Partial | 5 | `JAVA-RUN-021` | `run.project` | `detects_gradle_build_files_and_prefers_existing_wrapper` | Run executes `gradle run` / `./gradlew run` for Gradle projects; no run configuration is created |
| JAVA-TEST-007 | [java-gradle.md](../spec/features/java-gradle.md) | Recent tests popup | Missing | 1 | `JAVA-TEST-007` | — | — | not started (P2) |
| JAVA-TEST-011 | [java-gradle.md](../spec/features/java-gradle.md) | Parameterized and dynamic tests | Missing | 3 | `JAVA-TEST-011` | — | — | not started (P1) |
| JAVA-TEST-012 | [java-gradle.md](../spec/features/java-gradle.md) | JUnit/TestNG inspections & intentions | Missing | 3 | `JAVA-TEST-012` | — | — | not started (P1) |
| JAVA-TEST-014 | [java-gradle.md](../spec/features/java-gradle.md) | Debug tests | Missing | 5 | `JAVA-TEST-014` | — | — | not started (P0) |
| JAVA-TEST-016 | [java-gradle.md](../spec/features/java-gradle.md) | Test location from Gradle output | Missing | 5 | `JAVA-TEST-016` | — | — | not started (P0) |
| JAVA-DBG-025 | [java-gradle.md](../spec/features/java-gradle.md) | Resume / Pause | Missing | 5 | `JAVA-DBG-025` | — | — | not started (P0) |
| JAVA-DBG-026 | [java-gradle.md](../spec/features/java-gradle.md) | Show Execution Point | Missing | 5 | `JAVA-DBG-026` | — | — | not started (P0) |
| JAVA-DBG-027 | [java-gradle.md](../spec/features/java-gradle.md) | Step out of code block (inferred) | Missing | 1 | `JAVA-DBG-027` | — | — | not started (P2) |
| JAVA-DBG-032 | [java-gradle.md](../spec/features/java-gradle.md) | Stepping filters / "Do not step into" | Missing | 3 | `JAVA-DBG-032` | — | — | not started (P1) |
| JAVA-DBG-038 | [java-gradle.md](../spec/features/java-gradle.md) | Exception analysis | Missing | 1 | `JAVA-DBG-038` | — | — | not started (P2) |
| JAVA-DBG-040 | [java-gradle.md](../spec/features/java-gradle.md) | Breakpoints in libraries / decompiled code | Missing | 3 | `JAVA-DBG-040` | — | — | not started (P1) |
| JAVA-DBG-041 | [java-gradle.md](../spec/features/java-gradle.md) | Virtual threads support | Missing | 1 | `JAVA-DBG-041` | — | — | not started (P2) |
| JAVA-DBG-042 | [java-gradle.md](../spec/features/java-gradle.md) | Debug Gradle tasks / scripts | Missing | 3 | `JAVA-DBG-042` | — | — | not started (P1) |
| JAVA-COV-004 | [java-gradle.md](../spec/features/java-gradle.md) | Coverage suites | Missing | 1 | `JAVA-COV-004` | — | — | not started (P2) |
| JAVA-PROF-005 | [java-gradle.md](../spec/features/java-gradle.md) | Thread dumps | Missing | 1 | `JAVA-PROF-005` | — | — | not started (P2) |
| JAVA-PROF-006 | [java-gradle.md](../spec/features/java-gradle.md) | [Ultimate] Profiler inlay/gutter hints | Missing | 1 | `JAVA-PROF-006` | — | — | not started (P2) |
| JAVA-BUILD-002 | [java-gradle.md](../spec/features/java-gradle.md) | Build Module | Missing | 3 | `JAVA-BUILD-002` | — | — | not started (P1) |
| JAVA-BUILD-009 | [java-gradle.md](../spec/features/java-gradle.md) | Excludes from compilation | Missing | 1 | `JAVA-BUILD-009` | — | — | not started (P2) |
| JAVA-BUILD-010 | [java-gradle.md](../spec/features/java-gradle.md) | RMI / validation | Missing | 1 | `JAVA-BUILD-010` | — | — | not started (P2) |
| JAVA-BUILD-012 | [java-gradle.md](../spec/features/java-gradle.md) | Build on save / automatically | Missing | 1 | `JAVA-BUILD-012` | — | — | not started (P2) |
| JAVA-BUILD-014 | [java-gradle.md](../spec/features/java-gradle.md) | Build progress in status bar | Missing | 5 | `JAVA-BUILD-014` | — | — | not started (P0) |
| JAVA-BUILD-015 | [java-gradle.md](../spec/features/java-gradle.md) | Debug Build Process | Missing | 1 | `JAVA-BUILD-015` | — | — | not started (P2) |
| JAVA-BUILD-016 | [java-gradle.md](../spec/features/java-gradle.md) | Build notifications | Missing | 3 | `JAVA-BUILD-016` | — | — | not started (P1) |
| JAVA-VCS-001 | [java-gradle.md](../spec/features/java-gradle.md) | Refactoring-aware VCS moves | Missing | 3 | `JAVA-VCS-001` | — | — | not started (P1) |
