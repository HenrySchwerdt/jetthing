# Jetthing engineering guide

## Non-negotiables

- Keep engine crates independent of `egui`, `eframe`, `winit`, and OS UI APIs.
- Every meaningful user action is a typed command registered in `jetthing-command`.
- UI controls dispatch commands; they do not own filesystem or project policy.
- Prefer complete vertical slices with tests over empty crates or decorative controls.
- Never claim parity without reproducible evidence recorded in `docs/parity/`.
- Preserve clean-room boundaries: use public documentation, public source with compatible
  licensing, and observable behavior; do not copy JetBrains code, artwork, or branding.
- Avoid panics on user-controlled input. Return structured, actionable errors.

## Required checks

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p xtask -- parity --check
```

## Layering

`jetthing-app -> jetthing-ui -> jetthing-core -> {command, project, plugins}`.
Lower layers must not depend on higher layers. Language integrations belong behind
plugin/service traits, never inside the platform core.

