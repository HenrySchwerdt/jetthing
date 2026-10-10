# Android Studio parity

Reference host inventory (2026-10-10): macOS 26.2 arm64; Android Studio 2026.1,
build `AI-261.25134.95.2612.15914620`; Android SDK, emulator, and ADB 37.0.1 present.
The installed application was inspected read-only and is not a runtime dependency.

| ID | Feature | Status | Commands | Tests | Known limitations |
|---|---|---|---|---|---|
| AND-001 | Detect Android SDK | Planned | `android.sdk.detect` | — | host path observed only |
| AND-002 | Detect Gradle Android project | Partial | `build.detect` | `jetthing-gradle` plugin-detection tests | static detection of `com.android.application`/`library` (ids and version-catalog aliases) per included module; exposed as `GradleBuild::android_application_modules()`; no variants/SDK model yet |
| AND-003 | Kotlin/Java editing | Partial | `language.complete` | registry mapping tests, ignored real jdtls test | Java diagnostics and completion through jdtls (`evidence/lsp/09-java-completion-jdtls.png`); Kotlin through kotlin-lsp/kotlin-language-server when installed (not yet verified); Android-specific completion and highlighting pending |
| AND-004 | Device listing | Missing | `android.devices.list` | — | ADB available, adapter pending |
| AND-005 | Emulator launch | Missing | `android.emulator.start` | — | emulator available, adapter pending |
| AND-006 | Gradle build | Partial | `build.project` | core Android build/run command test | Build runs `assembleDebug` through the wrapper with the task tree in the Build window; Run is disabled with an explanation instead of `gradlew run`; variant selection pending; `evidence/build-run.md` |
| AND-007 | APK install | Missing | `android.apk.install` | — | M4 |
| AND-008 | Logcat | Missing | `android.logcat.start` | — | streaming tool window pending |
| AND-009 | Debugging | Missing | `debug.start` | — | DAP/JDWP design pending |
| AND-010 | Resource navigation | Missing | — | — | semantic platform pending |

