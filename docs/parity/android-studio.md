# Android Studio parity

Reference host inventory (2026-10-10): macOS 26.2 arm64; Android Studio 2026.1,
build `AI-261.25134.95.2612.15914620`; Android SDK, emulator, and ADB 37.0.1 present.
The installed application was inspected read-only and is not a runtime dependency.

| ID | Feature | Status | Commands | Tests | Known limitations |
|---|---|---|---|---|---|
| AND-001 | Detect Android SDK | Planned | `android.sdk.detect` | — | host path observed only |
| AND-002 | Detect Gradle Android project | Missing | — | — | M4 |
| AND-003 | Kotlin/Java editing | Missing | — | — | editor is currently plain text |
| AND-004 | Device listing | Missing | `android.devices.list` | — | ADB available, adapter pending |
| AND-005 | Emulator launch | Missing | `android.emulator.start` | — | emulator available, adapter pending |
| AND-006 | Gradle build | Missing | `build.run` | — | universal runner pending |
| AND-007 | APK install | Missing | `android.apk.install` | — | M4 |
| AND-008 | Logcat | Missing | `android.logcat.start` | — | streaming tool window pending |
| AND-009 | Debugging | Missing | `debug.start` | — | DAP/JDWP design pending |
| AND-010 | Resource navigation | Missing | — | — | semantic platform pending |

