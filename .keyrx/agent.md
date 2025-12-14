# KeyRx AI Agent Directive

Purpose: a compact, unambiguous rulebook for AI coding agents working on KeyRx. Treat this as the single source of operational truth before touching code.

## Read This First (guardrails)
- SSOT = Rust. Contracts, types, and state live in Rust; Dart mirrors generated bindings only (FRB v2).
- Contract-driven: define/adjust Rust APIs first, run codegen, then implement UI. Never hand-edit generated Dart or manual JSON contracts.
- Panic safety: FFI boundaries must catch panics and return structured errors. Never let a panic cross FFI.
- Realtime threads: audio/FFI callback threads may not block or take locks (no `Mutex`/`RwLock`/allocations there). Use lock-free ring buffers (e.g., `rtrb`), atomics, preallocated data.
- No globals: use `FfiContext` handle-based state; avoid `OnceLock/Mutex` singletons.
- File size budget: target ≤500 lines per module; split by domain/responsibility when approaching 450–500 lines.

## Default Workflow (schema-first)
1) Open tasks: `.spec-workflow/specs/rust-first-ai-native-ffi/tasks.md`; mark the active task `[-]`.
2) Read spec docs + relevant Implementation Logs before coding to avoid duplication.
3) Schema first: change/extend Rust FFI surface (`core/src/ffi/**`, `api.rs`), add/adjust contracts (`#[ffi_export]`), keep Rust as SSOT.
4) Generate bindings: run just/cargo workflows (e.g., `just gen-dart-bindings`, `cargo fmt`, `cargo clippy -- -D warnings`).
5) Implement logic in Rust; consume via generated Dart only.
6) Tests: unit + property tests for FFI boundary; keep coverage ≥80% for tool code.
7) Log: run spec log-implementation tool with artifacts; flip task to `[x]`.

## Architecture Invariants
- Flutter Rust Bridge v2 for all bindings.
- Contract JSON derives from Rust macros (`#[ffi_export]` + `FfiMarshaler`), not hand-written files.
- Error handling: prefer `anyhow` in internal code; FFI returns `FfiResult<T>` with `FfiError` codes. No stringly-typed errors crossing the boundary.
- Threading: UI/Dart isolate stays non-blocking; Rust Tokio for async; dedicated audio/FFI threads must be lock-free and allocation-free on hot paths.
- Data movement: favor zero-copy (`Vec<T>` ↔ `TypedData`) and bounded buffers; avoid JSON for large/streaming payloads.
- Observability: use `tracing` with `#[instrument(skip(...))]`; no `println!` in FFI exports.

## Code Style & Patterns
- Imports/order: std -> third-party -> workspace crates; keep lint-free.
- Results not panics: return `FfiError::invalid_input/internal/not_found` etc. Wrap external fallible calls.
- Timeouts/backpressure: when streaming, enforce bounded queues and drop/notify rather than block.
- Naming: functions are verbs (`start_recording`), data structs are nouns; domain modules under `core/src/ffi/domains/{domain}.rs`.
- Config: linker tweaks go in `.cargo/config.toml`; just recipes orchestrate workflows.

## Testing Requirements
- Unit tests for every new helper; property tests for FFI invariants (round-trip encode/decode, contract ↔ function parity).
- FFI boundary tests: null/UTF-8 validation, panic capture, invalid handles.
- Performance checks: avoid allocations in hot paths; benchmark if touching streaming/FFI loops.
- Clippy + fmt are mandatory; run domain-specific tests (`cargo test -p core --lib`, `just ui:test` when Dart changes).

## Examples (good vs bad)
- Good: Rust-first contract with macro and structured errors
  ```rust
  #[ffi_export(domain = "engine", version = "1.0")]
  pub fn get_active_profile(ctx: &FfiContext) -> FfiResult<ProfileView> {
      let state = ctx.domain::<EngineState>()?.active_profile()?;
      Ok(ProfileView::from(state))
  }
  ```
- Bad: Manual JSON contract edits, `Mutex` in audio callback, or Dart invoking ungenerated methods.

## Minimal FFI Flow (Rust → Dart)
```rust
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct PingResult {
    pub message: String,
}

#[ffi_export(domain = "diagnostics", version = "1.0")]
pub fn ping(ctx: &FfiContext, request_id: &str) -> FfiResult<PingResult> {
    ctx.require_ready()?;
    Ok(PingResult { message: format!("pong:{request_id}") })
}
```
Generated Dart (call site):
```dart
final resp = await api.ping(requestId: 'ui-init');
print(resp.message); // "pong:ui-init"
```

## Module Boundaries
- Split when near 500 lines: move domain helpers into `{domain}/{feature}.rs` with `mod.rs` re-exports.
- Keep one responsibility per file; avoid mega-modules (e.g., legacy `exports_engine.rs`).

## Workflow Checklist (per change)
- [ ] Mark task `[-]` in tasks.md and read latest Implementation Logs.
- [ ] Touch Rust SSOT first; regenerate bindings.
- [ ] Run `cargo fmt`, `cargo clippy -- -D warnings`, relevant tests.
- [ ] Ensure no blocking/allocations on realtime threads.
- [ ] Update docs/user-facing guides as needed.
- [ ] Use log-implementation tool with artifacts; mark task `[x]`.

## Quick anti-patterns to avoid
- Manual contract JSON edits or Dart-side type changes without Rust source changes.
- Blocking calls (file I/O, network) on FFI/audio threads.
- Global mutable state; prefer `FfiContext`.
- Swallowing errors; always surface actionable codes/messages.

Keep this document in sync as the FFI spec evolves; update examples when macros or contracts change.
