# Android Studio parity

Reference host inventory (2026-10-10): macOS 26.2 arm64; Android Studio 2026.1,
build `AI-261.25134.95.2612.15914620`; Android SDK, emulator, and ADB 37.0.1 present.
The installed application was inspected read-only and is not a runtime dependency.

| ID | Feature | Status | Commands | Tests | Known limitations |
|---|---|---|---|---|---|
| AND-001 | Detect Android SDK | Implemented | `android.sdk.detect`, `android.sdk.manager`, `android.sdk.setLocation` | `jetthing-android` discovery-order/inventory/properties tests, core command tests | IDE setting › `local.properties` `sdk.dir` › `ANDROID_HOME` › `ANDROID_SDK_ROOT` › per-OS default; installed packages from `package.xml`/`source.properties`; SDK page lists installed packages only (no remote repository, install or update); `evidence/android-sdk.md` |
| AND-002 | Detect Gradle Android project | Partial | `android.project.detect` | Studio-template (Kotlin DSL + version catalog) and legacy Groovy fixtures | modules from `settings.gradle(.kts)`, AGP plugin ids or catalog aliases, AGP version, namespace/applicationId/SDK levels when literal, manifest launcher activity; Gradle scripts are not evaluated (no sync model, flavors or variants); `evidence/android-sdk.md` |
| AND-003 | Kotlin/Java editing | Partial | `language.complete` | registry mapping tests, ignored real jdtls test | Java diagnostics and completion through jdtls (`evidence/lsp/09-java-completion-jdtls.png`); Kotlin through kotlin-lsp/kotlin-language-server when installed (not yet verified); Android-specific completion and highlighting pending |
| AND-004 | Device listing | Implemented | `android.devices.list`, `android.devices.track`, `android.devices.select`, `toolwindow.activate.deviceManager` | fake-adb `track-devices` framing/getprop tests, selection and missing-adb tests | live `adb track-devices -l` with per-device properties, toolbar device selector, Device Manager tool window on a new right anchor; Wi-Fi pairing and device details pending; `evidence/android-devices.md` |
| AND-005 | Emulator launch | Partial | `android.avd.list`, `android.emulator.start`, `android.emulator.stop` | AVD ini parsing and fake-emulator early-exit tests, start/stop command tests | AVDs from the AVD home, `emulator -avd` launch with early-exit error reporting, stop via `adb emu kill`; no create/edit/wipe wizard; real boot unverified here (no KVM); `evidence/android-devices.md` |
| AND-006 | Gradle build | Partial | `build.project` | core Android build/run command test | Build runs `assembleDebug` through the wrapper with the task tree in the Build window; Run is disabled with an explanation instead of `gradlew run`; variant selection pending; `evidence/build-run.md` |
| AND-007 | APK install | Missing | `android.apk.install` | — | M4 |
| AND-008 | Logcat | Implemented | `android.logcat.start`, `android.logcat.filter`, `android.logcat.clear`, `android.logcat.pause`, `android.logcat.softWrap`, `android.logcat.stop`, `toolwindow.activate.logcat` | threadtime parser, filter-language, fake-adb streaming, pause/clear/capacity command tests | `adb logcat -v threadtime` stream with level colours, `package:mine`/`tag:`/`message:`/`level:` with `=:`, `~:` and `-` forms, match case, pause, clear, soft wrap, scroll to end, 100k-line ring; saved filters, `age:`/`is:` keys and multiple tabs pending; `evidence/android-logcat.md` |
| AND-009 | Debugging | Missing | `debug.start` | — | DAP/JDWP design pending |
| AND-010 | Resource navigation | Missing | — | — | semantic platform pending |
| AND-011 | Android project view | Implemented | `android.projectView.select`, `android.projectView.tree` | view-model golden test (manifests, merged java/kotlin source sets, resource qualifier groups, Gradle Scripts labels), core command test | Android ▾ / Project ▾ switcher, default for Android projects, double-click opens files; no generated sources, class icons or New › Android context actions; `evidence/android-project-view.md` |

