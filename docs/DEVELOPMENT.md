# Development Workflow

## FFI Watch Mode
- Purpose: regenerate Dart bindings automatically when Rust FFI contracts change to keep Dart/Rust in sync.
- Install prerequisite: `cargo install watchexec-cli` (or via package manager). The `just setup` recipe installs it by default.
- Start watcher: `just dev-watch` (runs `scripts/watch_ffi.sh`).
- What it watches: `core/src/ffi/*.rs` (recursive).
- On change: runs `just gen-dart-bindings` with a 500ms debounce to avoid thrashing.
- Customization: set `DEBOUNCE_MS=750` or override the command with `WATCH_COMMAND="just gen-dart-bindings --verbose"` when invoking `just dev-watch`.
- Notes: keep Rust changes small to get sub-second regeneration; watcher clears output on each run for readability.
