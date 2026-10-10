# Architecture

Jetthing is a native Rust workspace with a headless engine and replaceable UI shell.
The initial shell uses eframe/egui with wgpu. This choice provides a portable native
window, GPU rendering, IME/input handling, accessibility work in progress upstream,
and fast delivery of the first usable slice. It is not an architectural dependency:
no crate below `jetthing-ui` imports UI types.

## Layers

- `jetthing-command`: typed command metadata, JSON parameters/results, discovery,
  validation, execution, and structured errors.
- `jetthing-project`: safe project-root model, bounded file tree, UTF-8 document IO.
- `jetthing-text`: revisioned rope storage, Unicode-safe transactional edits, and history.
- `jetthing-language`: language-neutral analysis output with the first Tree-sitter Rust provider.
- `jetthing-terminal`: native PTY process lifecycle, asynchronous output, input, and resizing.
- `jetthing-search`: cancellable generation-based background text search with streamed,
  bounded results and binary/large-file exclusion.
- `jetthing-git`: repository/branch/status modeling and argument-safe Git operations;
  it delegates repository semantics to the installed Git executable.
- `jetthing-plugins`: versioned TOML manifests, capability declarations, discovery,
  and validation. Executable/Wasm loading is deliberately not claimed yet.
- `jetthing-core`: application state and built-in command handlers.
- `jetthing-ui`: native shell, project tree, highlighted editor, command palette,
  resizable terminal tool window, and status surface.
- `jetthing-app`: process startup and native window.
- `jetthing-cli`: headless command discovery and execution.

## Command contract

Commands have stable IDs, human labels, JSON schemas expressed as documentation,
capability requirements, an enabled predicate, and an execution handler. GUI and CLI
use the same registry. Cancellation/progress and MCP/control transports are next-step
extensions of this contract.

## Long-term direction

The semantic platform will layer VFS snapshots, incremental syntax, symbol/stub
indexes, dependency-aware invalidation, and prioritized cancellable jobs. LSP is one
provider behind a richer language-service interface, not the semantic architecture.
Untrusted plugins will use the WebAssembly Component Model with explicit capabilities;
native plugins will require an explicit trusted tier and process isolation.

## Reference findings

PhotoCraft validates the engine-first command-registry approach and generated parity
reports. IntelliJ's VFS/PSI/stub indexes motivate stable virtual identities and compact
externally-visible symbol indexes. Zed demonstrates GPU-first native UI; Lapce validates
rope-based text storage and language-service isolation. Jetthing reimplements behavior
cleanly and does not reuse protected assets or branding.
