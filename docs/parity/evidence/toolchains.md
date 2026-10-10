# Toolchain provisioning evidence

Issue #75. Reference behaviour: IntelliJ IDEA's *Download JDK…* (Project Structure → SDKs)
downloads a vendor JDK chosen from the foojay Disco API into `~/.jdks/<vendor>-<version>`,
shows progress in the status bar and detects JDKs that are already installed
(`JAVA_HOME`, `/usr/lib/jvm`, SDKMAN) before offering a download — see
`reference/intellij-community/NOTES.md` (project JDK auto-detected as
`Java 21.0.12 /usr/lib/jvm/java-1.21.0-openjdk-amd64`). Jetthing generalises this to every
component a language needs, under `<data dir>/jetthing/toolchains/<component>/<version>/`.

## Slice 1 — provisioning core (this evidence)

Everything below runs headless against a local fake HTTP server
(`crates/jetthing-toolchains/tests/support/mod.rs`: fixed bodies, `Range: bytes=n-` with
`206`/`416`, one-shot connection cuts, request log). Reproduce with:

```bash
cargo test -p jetthing-toolchains
cargo test -p jetthing-core toolchain
```

| Acceptance item (#75) | Test |
|---|---|
| Resolution (index, version prefix, channel, platform) | `resolves_downloads_verifies_installs_and_uninstalls`, `catalog::tests::resolves_versions_channels_and_platforms` |
| Checksum verification (inline digest and `sha256sum` file) | `resolves_downloads_verifies_installs_and_uninstalls` |
| Checksum mismatch → nothing installed, download deleted | `checksum_mismatch_installs_nothing_and_discards_the_download` |
| Resume: second attempt sends `Range: bytes=<cut>-` and continues from the cut | `interrupted_download_resumes_with_a_range_request` |
| Atomic install: a broken archive leaves no version dir and no staging leftovers | `failed_extraction_leaves_no_partial_install` |
| Uninstall; locate falls back to the remaining version | `resolves_downloads_verifies_installs_and_uninstalls` |
| PATH-first discovery; home variable; pinned managed version; configured path | `path_and_home_installs_win_unless_the_project_pins_a_managed_version` |
| Offline: clear error naming host and proxy settings; HTTP status errors | `unreachable_hosts_report_offline_and_missing_files_report_http_status` |
| Background install with progress events; duplicate requests join the job | `service_installs_in_the_background_and_reports_progress` |
| Archive safety: `../` entries rejected, tar.gz/tar.xz/zip/gz/raw layouts | `extract::tests::*` |
| Project pins from version files | `pinning::tests::*` |
| `NO_PROXY` hosts, suffixes, IPv4 CIDR | `download::tests::no_proxy_matches_hosts_suffixes_and_networks` |
| Commands: install (wait), locate, list, missing, uninstall, usePath, pins, errors | `toolchain_commands::tests::*` |

Results on Linux x86_64 (2026-10-10): 17 unit + 7 integration tests in
`jetthing-toolchains`, 3 command tests in `jetthing-core`, all passing.

CLI smoke run (no catalog entries ship in this slice, so nothing is installable yet):

```text
$ JETTHING_TOOLCHAINS_DIR=/tmp/tc jetthing-cli run toolchain.list
{"components":[],"platform":"linux-x86_64","root":"/tmp/tc","settingsError":null}
$ jetthing-cli run toolchain.install '{"component":"jdk"}'
invalid parameters: unknown toolchain component `jdk`
```

## Slice 3a — first catalog entries

Built-in providers (`crates/jetthing-toolchains/src/providers/`): `jdk` (foojay Disco API),
`jdtls` (download.eclipse.org milestones), `rust-analyzer`, `bazelisk` and `taplo` (GitHub
releases with asset digests). Provider tests: `cargo test -p jetthing-toolchains --test providers`.

Live run on a Linux x86_64 container, 2026-10-10. A system JDK 21 is present
(`JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64`), so `toolchain.list` correctly keeps
preferring it while the managed install exists alongside:

```text
$ export JETTHING_TOOLCHAINS_DIR=/tmp/tcroot
$ jetthing-cli run toolchain.install '{"component":"jdk","version":"21","wait":true}'
{"component":"jdk","job":1,"path":"/tmp/tcroot/jdk/21.0.12.1+1","version":"21.0.12.1+1"}   # 16.7 s
$ /tmp/tcroot/jdk/21.0.12.1+1/bin/java -version
openjdk version "21.0.12.1" 2026-08-18 LTS
$ cat /tmp/tcroot/jdk/21.0.12.1+1/.jetthing-install.json
{ "component": "jdk", "version": "21.0.12.1+1",
  "source_url": "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12.1%2B1/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12.1_1.tar.gz",
  "sha256": "ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94", ... }
$ jetthing-cli run toolchain.install '{"component":"jdtls","wait":true}'
{"component":"jdtls","job":1,"path":"/tmp/tcroot/jdtls/1.61.0","version":"1.61.0"}       # 5.9 s
$ jetthing-cli run toolchain.list
jdk:   located JAVA_HOME (/usr/lib/jvm/java-21-openjdk-amd64), installed ["21.0.12.1+1"]
jdtls: located managed (/tmp/tcroot/jdtls/1.61.0/bin/jdtls), installed ["1.61.0"], requires ["jdk"]
rust-analyzer: located PATH (/root/.cargo/bin/rust-analyzer)
```

(`toolchain.list` output abbreviated from JSON.) In this sandbox the GitHub REST API is
blocked by the network policy, so no live rust-analyzer/bazelisk/taplo install is recorded;
their release assets themselves download fine.

## Pending (next slices)

- More catalog entries (rustup toolchains, Android SDK, kotlin-lsp, Node, Python, Go, ...)
  and end-to-end runs on a clean container: open a Java Gradle project without a JDK →
  Set up → completion (needs the LSP resolver hook so jdtls starts from the managed install).
- "Missing toolchain" editor notification and Settings > Languages & Frameworks > Toolchains.
- Status-bar progress screenshot (the widget only appears while an install runs).
- Probing versions of `PATH` binaries; signature (GPG/minisign) verification.
