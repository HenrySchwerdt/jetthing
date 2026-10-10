# Releases, installers and updates

Jetthing ships two kinds of downloads for every release:

- **Archives** — portable directories the self-updater reads and replaces in place.
- **Native installers** — `.msi`, `.dmg` and `.deb`, for a normal install with menu
  entries and clean uninstall.

| Platform | Installer | Archive |
|---|---|---|
| Windows x86_64 | `jetthing-<version>-windows-x86_64.msi` | `jetthing-<version>-windows-x86_64.zip` |
| macOS Apple silicon | `jetthing-<version>-macos-aarch64.dmg` | `jetthing-<version>-macos-aarch64.tar.gz` |
| macOS Intel | `jetthing-<version>-macos-x86_64.dmg` | `jetthing-<version>-macos-x86_64.tar.gz` |
| Linux x86_64 | `jetthing_<version>_amd64.deb` | `jetthing-<version>-linux-x86_64.tar.gz` |

Every file has a `<file>.sha256` next to it in `sha256sum` format. Check a download with
`sha256sum -c <file>.sha256` (Linux), `shasum -a 256 -c <file>.sha256` (macOS) or
`Get-FileHash <file>` (PowerShell). The names are stable: the self-updater and these
docs depend on them.

## Install and uninstall

### Windows (MSI)

Double-click `jetthing-<version>-windows-x86_64.msi` and approve the administrator
prompt. It installs:

- `C:\Program Files\Jetthing\` with `jetthing.exe` and `jetthing-cli.exe`;
- a **Jetthing** Start Menu entry;
- the install folder on the system `PATH` (new terminals), so `jetthing .` and
  `jetthing-cli` work from a shell.

A newer MSI replaces an older one in place, because all versions share one upgrade
code. Installing an older MSI over a newer one is refused.

To install without prompts, for example in scripts, run
`msiexec /i jetthing-<version>-windows-x86_64.msi /qn` from an elevated shell.

To uninstall, use **Settings → Apps → Installed apps → Jetthing → Uninstall**, or run
`msiexec /x jetthing-<version>-windows-x86_64.msi /qn`. This removes the files, the
Start Menu entry and the `PATH` entry. Project data in `.jetthing/` folders inside your
projects is left alone.

**SmartScreen.** Unsigned MSIs show "Windows protected your PC". Click
**More info → Run anyway**. Signed builds (see [Signing](#optional-signing)) avoid
this once the certificate has a reputation.

### macOS (DMG)

Open `jetthing-<version>-macos-<arch>.dmg`. Use `aarch64` on Apple silicon and
`x86_64` on Intel. Then drag **Jetthing** onto **Applications**. The bundle
(`io.github.henryschwerdt.jetthing`) contains `jetthing` and `jetthing-cli` in
`Jetthing.app/Contents/MacOS/`. To use them from a terminal, run:

```bash
sudo ln -s /Applications/Jetthing.app/Contents/MacOS/jetthing /usr/local/bin/jetthing
```

To uninstall, move `/Applications/Jetthing.app` to the Bin.

**Gatekeeper.** Release builds are signed ad hoc but not notarized unless the signing
secrets below are configured. The first time you open one, macOS reports that it
"cannot verify" the app. Use either:

- **System Settings → Privacy & Security → Open Anyway** (macOS 15 and later), or
  right-click the app → **Open** on older versions.
- `xattr -dr com.apple.quarantine /Applications/Jetthing.app`

Apps launched from Finder do not inherit your shell's `PATH`. If Cargo, Gradle or Git
live in a shell-only location, start Jetthing from a terminal with
`open -a Jetthing` or the symlink above.

### Linux (.deb)

```bash
sudo apt install ./jetthing_<version>_amd64.deb   # or: sudo dpkg -i …
```

The package installs `/usr/lib/jetthing/`, adds `/usr/bin/jetthing` and
`/usr/bin/jetthing-cli` links, `jetthing.desktop` and hicolor icons. The `Depends:`
glibc version comes from the binary's symbol versions. Release builds run on Ubuntu
22.04, so they need glibc 2.35 or newer.

To uninstall, run `sudo apt remove jetthing` (or `sudo dpkg -r jetthing`).

### Archives (all platforms)

Extract the archive anywhere you can write and run `jetthing` (`jetthing.exe` on
Windows). On Linux, the archive also contains `jetthing.desktop` and `jetthing.png`. To
add a menu entry, copy `jetthing.desktop` to `~/.local/share/applications/` and
`jetthing.png` to `~/.local/share/icons/hicolor/512x512/apps/`, then set `Exec=` to
the absolute path of `jetthing`. To uninstall, delete the directory.

## Updates

**Help → Check for Updates…** (also run silently at startup unless
`JETTHING_NO_UPDATE_CHECK` is set) reads the latest GitHub release. **Update and
Restart** then works according to how Jetthing was installed.
`jetthing_update::InstallKind::detect` decides this from the running executable:

| Installed from | Detected by | What the update does |
|---|---|---|
| Archive | default | Downloads the archive, verifies its `.sha256` and replaces `jetthing` and `jetthing-cli` in place (the old binary is kept as `jetthing.old`). **Restart** relaunches. |
| MSI | `jetthing.install` containing `msi` next to the exe | Downloads and verifies the new `.msi`. **Quit and Install** saves the session, starts `msiexec /i <msi>` and quits. The MSI upgrades the installed version. |
| DMG | executable inside `*.app/Contents/MacOS/` | Downloads and verifies the new `.dmg`. **Quit and Install** opens it and quits; drag Jetthing into Applications and replace the old copy. |
| .deb | `jetthing.install` containing `deb` | Downloads and verifies the new `.deb`. **Quit and Install** opens it with `xdg-open` (your software installer) and quits. |
| `cargo run` | executable not named `jetthing` | Links to the release page only. |

Installer downloads go to `<temp>/jetthing-update/`. A release without a `.sha256` for
the needed file is refused. The installed-copy paths (Program Files, `/Applications`,
`/usr/lib`) are never written by the IDE itself.

| Variable | Effect |
|---|---|
| `JETTHING_NO_UPDATE_CHECK` | disable the startup check |
| `JETTHING_GITHUB_TOKEN` / `GITHUB_TOKEN` | token for a private release repository |
| `JETTHING_UPDATE_REPOSITORY` | `owner/repo` to update from (default `HenrySchwerdt/jetthing`) |

## Building locally

```bash
cargo run -p xtask -- dist                                  # archive for the host
cargo run -p xtask -- dist --installer                      # + native installer
cargo run -p xtask -- dist --installer --target x86_64-apple-darwin
```

The output goes to `dist/`. Each artifact path is printed on its own line.

| Installer | Needs |
|---|---|
| `.msi` | [WiX Toolset v5](https://wixtoolset.org) (`dotnet tool install --global wix --version 5.0.2`); the `WIX_EXE` variable overrides the `wix` path. Definition: `packaging/windows/jetthing.wxs`. |
| `.dmg` | Xcode command line tools (`codesign`, `hdiutil`). Signing uses `JETTHING_MACOS_SIGN_IDENTITY` when it is set, and ad hoc (`-`) otherwise. |
| `.deb` | `dpkg-deb` (and `objdump` for the glibc requirement). |

The app icon is original artwork. `assets/icon/jetthing.svg` is the source, and
`python3 assets/icon/render.py` (cairosvg and Pillow) regenerates the committed `.png`,
`.ico` and `.icns` files.

## Release workflow

`.github/workflows/release.yml` builds the archives and installers on
`ubuntu-22.04`, `windows-latest` and `macos-latest`. The macOS runner builds both
`aarch64` and `x86_64`, cross-compiling the latter. Each installer is then
smoke-tested:

- The `.deb` is installed and removed again.
- The MSI is installed, reinstalled to check the upgrade code, and uninstalled. The
  workflow checks the Start Menu entry, the install marker and that no files are left
  behind.
- The DMG is mounted, and the workflow checks the Info.plist, the code signature and
  the CLI.

The workflow runs on pull requests that touch packaging files (build and test only)
and can be started manually from the Actions tab.

Publishing GitHub release `v<version>` (`version` from `[workspace.package]` in
`Cargo.toml`):

- **Push to `main`**: the workflow publishes when the release is missing, is a draft,
  or lacks any of the expected files. If tag `v<version>` already exists on a
  different commit, it does not publish and logs a warning instead. Re-run the tag's
  workflow to repair the release, or bump the version.
- **Push of tag `v<version>`**: the workflow always publishes or repairs (re-uploads)
  the release. The tag must equal the workspace version, or the run fails.
- **Pull requests and other branches**: the workflow never publishes.

To ship a version, bump `[workspace.package] version` and merge it to `main`.

## Optional signing

All signing is optional. The workflow skips each signing step when its secrets are
unset, so forks and pull requests build unsigned. Store certificates as base64:
`base64 -i cert.p12 | pbcopy`.

| Secret | Used for |
|---|---|
| `MACOS_CERTIFICATE`, `MACOS_CERTIFICATE_PASSWORD` | Developer ID Application certificate (`.p12`), imported into a temporary keychain |
| `MACOS_SIGN_IDENTITY` | e.g. `Developer ID Application: Name (TEAMID)`; enables the hardened runtime and a timestamp |
| `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | `notarytool` notarization and stapling of the DMG (the checksum is regenerated after stapling) |
| `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` | Authenticode `.pfx` used by `signtool` to sign the MSI (the checksum is regenerated afterwards) |

Never commit certificates or passwords. The repository contains no signing material.
