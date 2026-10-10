# Jetthing

Jetthing is an experimental, clean-room, native Rust IDE platform. The current
milestone is a usable foundation: a project tree, editable tabs, file save,
command palette, typed command registry, and manifest-based plugin discovery.

```bash
cargo run -p jetthing-app -- .
cargo run -p jetthing-cli -- commands
cargo run -p xtask -- parity
```

See `docs/roadmap.md` for honest status and limitations.

<!-- parity:begin -->
**Parity progress:** 9.9% implemented (weighted), 0.0% verified, across 656 tracked
features in 13 areas. See [`docs/parity/dashboard.md`](docs/parity/dashboard.md).
<!-- parity:end -->
