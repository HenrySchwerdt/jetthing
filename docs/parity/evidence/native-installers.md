# Native installers (PLAT-024)

How to reproduce: `cargo run -p xtask -- dist --installer` on each OS (see
`docs/release.md`), or run the **Release** workflow on any branch.

## Linux `.deb` (local, Ubuntu container, x86_64)

```text
$ cargo run -p xtask -- dist --installer
dist/jetthing-0.0.1-linux-x86_64.tar.gz
dist/jetthing_0.0.1_amd64.deb
$ sha256sum -c dist/*.sha256
jetthing-0.0.1-linux-x86_64.tar.gz: OK
jetthing_0.0.1_amd64.deb: OK
$ sudo dpkg -i dist/jetthing_0.0.1_amd64.deb
Setting up jetthing (0.0.1) ...
Processing triggers for hicolor-icon-theme (0.17-2) ...
$ readlink -f /usr/bin/jetthing
/usr/lib/jetthing/jetthing
$ jetthing-cli commands | head -1
[
$ sudo dpkg -r jetthing && ls /usr/lib/jetthing /usr/bin/jetthing
ls: cannot access '/usr/lib/jetthing': No such file or directory
ls: cannot access '/usr/bin/jetthing': No such file or directory
```

## CI (GitHub-hosted runners)

The `Release` workflow builds every archive and installer, verifies the checksums and
smoke-tests each installer:

- **ubuntu-22.04**: `.deb` install, CLI run, then uninstall leaves nothing behind.
- **windows-latest**: silent MSI install. The workflow checks Program Files, the
  install marker, the Start Menu shortcut, the CLI and the Add/Remove Programs entry.
  A reinstall must still show a single product (MajorUpgrade), and uninstall must
  leave nothing behind.
- **macos-latest**, arm64 and x86_64: the workflow mounts the DMG and checks the
  Applications link, `plutil -lint`, `codesign --verify --deep --strict`, `lipo`
  architectures and a CLI run on the native architecture.

The run and its artifact list are linked from the pull request that introduced the
installers (#53).
