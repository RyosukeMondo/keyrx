# AI Agent Guide for KeyRx

This guide shows how to apply `.keyrx/agent.md` in day-to-day work and how to validate that an AI agent can safely add FFI features end-to-end.

## Audience & Goals
- Audience: AI coding agents and humans collaborating with them.
- Goal: ship changes via Rust-first, contract-driven workflow without breaking FFI safety, realtime guarantees, or architecture rules.

## Before You Start
- Read `.keyrx/agent.md` (non-negotiable constraints).
- Open `.spec-workflow/specs/rust-first-ai-native-ffi/tasks.md`, set the active task to `[-]`, and scan existing Implementation Logs to avoid duplication.
- Load relevant steering docs when in doubt (`.spec-workflow/steering/`), plus `docs/ffi-architecture.md` for boundary patterns.

## Standard Workflow (Rust-first)
1. **Define contract in Rust**: add/adjust `#[ffi_export]` functions and `FfiMarshaler` structs in `core/src/ffi/**`. Rust is SSOT—do not edit Dart/generated code manually.
2. **Generate bindings**: run codegen (`just gen-dart-bindings` or equivalent), then `cargo fmt`, `cargo clippy -- -D warnings`.
3. **Implement & guard**: keep realtime threads lock-free and allocation-free; return `FfiResult<T>` with actionable `FfiError` codes; add `#[instrument(skip(...))]` where tracing is required.
4. **Tests**: add unit + property tests for FFI invariants (null/UTF-8 handling, panic capture, contract ↔ function parity). Target ≥80% coverage for tool code.
5. **Docs & logs**: update relevant docs if behavior changes. Use the spec log-implementation tool with artifacts, then mark the task `[x]`.

## How to Prompt an AI Agent (examples)
- *Schema-first add*: “Using `.keyrx/agent.md`, add a diagnostics ping FFI: update Rust contract with `#[ffi_export(domain=\"diagnostics\")]`, run codegen, wire generated Dart call, add tests for null/UTF-8/panic handling, keep realtime threads lock-free.”
- *Refactor*: “Split `core/src/ffi/exports_engine.rs` into ≤500-line modules following the module-boundary rules in `.keyrx/agent.md`; keep Rust as SSOT and rerun codegen.”

## Validation Checklist (per change)
- Contracts originate from Rust; no manual JSON edits.
- No blocking/locking on FFI or audio callbacks; hot paths stay allocation-free.
- Generated Dart is used as-is; no handwritten bindings.
- `cargo fmt` + `cargo clippy -- -D warnings` + relevant tests pass.
- Tracing uses `tracing`/`#[instrument]`; no `println!` in FFI exports.
- Files stay ≤500 lines (or are split with clear domains/features).

## Quick End-to-End Dry Run (smoke test for the directives)
Use this to verify an AI agent understands the rules before larger tasks:
1. Add `#[ffi_export(domain = "diagnostics", version = "1.0")] fn ping(...) -> FfiResult<PingResult>`.
2. Run codegen and ensure generated Dart exposes `ping`.
3. Add a test that:
   - returns `pong:{id}` on success,
   - rejects null/empty request IDs with `FfiError::invalid_input`,
   - captures panics at the boundary.
4. Confirm no locks/allocations on realtime paths and file stays <500 lines.
5. Run fmt + clippy + tests; log the implementation.

If any step fails, fix the directive or code before shipping.

## Common Pitfalls to Avoid
- Editing generated Dart or contract JSON directly.
- Adding `Mutex`/`RwLock` or blocking I/O in audio/FFI threads.
- Skipping property tests for FFI marshaling.
- Letting panics cross the FFI boundary or returning stringly errors.

## When in Doubt
Re-read `.keyrx/agent.md`, check `docs/ffi-architecture.md`, and prefer Rust-first changes with clear, testable contracts. Escalate if an architectural decision is unclear.
