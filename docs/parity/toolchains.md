# Toolchain provisioning parity

Setting up what a language needs: JDKs/SDKs, language servers, build tools, debuggers and
formatters (issue #75). Reference: IntelliJ IDEA *Download JDK…*, SDK auto-detection and the
Gradle JVM selection. Evidence: `evidence/toolchains.md`.

| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
| TOOL-001 | IntelliJ Platform | Managed toolchain installs (Download JDK-style) | Partial | 5 | `toolchain.install`, `toolchain.cancel`, `toolchain.jobs`, `toolchain.uninstall` | fake download server tests: resolution, checksum mismatch, Range resume, failed extraction leaves nothing, uninstall, background service; command round-trip tests | SHA-256 verified (inline or checksum file), resumable, atomic rename from `.staging/`; tar.gz/tar.xz/zip/gz/raw; status-bar progress with cancel; no real catalog entries yet (JDK, rust-analyzer, Android SDK next), no signature (GPG/minisign) checks; `evidence/toolchains.md` |
| TOOL-002 | IntelliJ Platform | Detect installed SDKs/tools before downloading | Partial | 5 | `toolchain.locate`, `toolchain.missing`, `toolchain.list`, `toolchain.usePath` | PATH/home-variable/configured/pinned-managed precedence tests | configured path → pinned managed → `JAVA_HOME`-style variable → PATH → managed; versions of PATH binaries are not probed yet; `evidence/toolchains.md` |
| TOOL-003 | IntelliJ Platform | Project toolchain pinning | Partial | 3 | `toolchain.pins` | version-file parsing tests | `.jetthing/toolchains.toml`, `rust-toolchain(.toml)`, `.bazelversion`, Gradle `JavaLanguageVersion.of`/`jvmToolchain`, `gradle-daemon-jvm.properties`, `.sdkmanrc`, `.nvmrc`/`.node-version`, `.python-version`, `go.mod`; Maven toolchains and Gradle version catalogs pending |
| TOOL-004 | IntelliJ Platform | Proxy-aware, offline-tolerant downloads | Partial | 3 | — | NO_PROXY matching and offline/HTTP error tests | `HTTPS_PROXY`/`HTTP_PROXY`/`ALL_PROXY` with `NO_PROXY` (hosts, suffixes, IPv4 CIDR), system trust store; offline error names the host and suggests proxy settings or an existing installation; proxy UI in Settings pending |
