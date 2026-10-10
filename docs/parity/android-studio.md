# Android Studio parity

Reference host inventory (2026-10-10): macOS 26.2 arm64; Android Studio 2026.1,
build `AI-261.25134.95.2612.15914620`; Android SDK, emulator, and ADB 37.0.1 present.
The installed application was inspected read-only and is not a runtime dependency.

| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
| AND-001 | Android Studio | Detect Android SDK | Planned | 2 | `android.sdk.detect` | — | host path observed only |
| AND-002 | Android Studio | Detect Gradle Android project | Missing | 3 | — | — | M4 |
| AND-003 | Android Studio | Kotlin/Java editing | Missing | 5 | — | — | editor is currently plain text |
| AND-004 | Android Studio | Device listing | Missing | 3 | `android.devices.list` | — | ADB available, adapter pending |
| AND-005 | Android Studio | Emulator launch | Missing | 3 | `android.emulator.start` | — | emulator available, adapter pending |
| AND-006 | Android Studio | Gradle build | Missing | 3 | `build.run` | — | universal runner pending |
| AND-007 | Android Studio | APK install | Missing | 2 | `android.apk.install` | — | M4 |
| AND-008 | Android Studio | Logcat | Missing | 3 | `android.logcat.start` | — | streaming tool window pending |
| AND-009 | Android Studio | Debugging | Missing | 5 | `debug.start` | — | DAP/JDWP design pending |
| AND-010 | Android Studio | Resource navigation | Missing | 2 | — | — | semantic platform pending |

