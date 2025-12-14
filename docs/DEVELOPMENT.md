# Development Workflow

This is the single entrypoint for getting productive quickly on KeyRx. It stitches together the Dart/Flutter workflow with the Rust core so you can move fast without skipping quality gates.

## TL;DR daily loop
- One-time setup: `just setup` (installs toolchains, coverage tools, git hooks) then `just ui-setup` (pub get + codegen).
- Run the app with real core: `just ui`. Run with mock FFI: `just dev-mock` (`MOCK_FFI=true` flag flows to the bridge provider).
- Tight loop: `just ui-format && just ui-analyze && just ui-metrics && just ui-test` or simply `just ui-quality`.
- Coverage + HTML report: `just ui-coverage` → open `ui/coverage/html/index.html`.
- Regenerate Dart bindings while editing Rust FFI: `just dev-watch` (debounced watch on `core/src/ffi/*.rs`).

## Prerequisites
- Rust toolchain (`rustup`, `clippy`, `rustfmt`), Flutter SDK on PATH, and `dart` available.
- `just` installed locally.
- Optional accelerators: `sccache` (Rust builds), `watchexec-cli` (installed by `just setup`), `lcov`/`genhtml` (coverage).

## Environment setup
- `just setup` installs dependencies, coverage tools, and git hooks. Hooks mirror our quality gates (format, analyze, fast tests, coverage threshold) so expect commits to fail if the gates fail.
- `just ui-setup` runs `flutter pub get` plus `build_runner` to generate Riverpod code and any other generated artifacts.
- Use `just check` before PRs to run Rust + Flutter gates together.

## Running the app
- Real bridge: `just ui` (launches on Linux device by default; set `DEVICE` for others).
- Mock bridge: `just dev-mock` to use `MockKeyrxBridge` without Rust; ideal for UI-only work.
- Hot reload works normally; keep `MOCK_FFI=true` for UI flows that do not require native calls.

## Quality gates (Flutter)
- Format: `just ui-format` (no writes; fails if drift).
- Analyze: `just ui-analyze` (`--fatal-infos` to keep warnings at zero).
- Metrics: `just ui-metrics` (enforces 500 lines/file, 50 lines/function).
- Tests: `just ui-test` (honors `CONCURRENCY`, default 4).
- Goldens: `just ui-golden-verify` / `just ui-golden-update`.
- Coverage: `just ui-coverage` (lcov + HTML) or fast gate via `ui/scripts/check_coverage.sh` (`COVERAGE_THRESHOLD` default 60%, target 80%).

## Rust ↔ Dart FFI workflow
- Watch & regenerate bindings: `just dev-watch` (500ms debounce; overrides via `DEBOUNCE_MS=750` or `WATCH_COMMAND="just gen-dart-bindings --verbose"`).
- Manual regeneration: `just gen-dart-bindings`; verify drift with `just verify-bindings`.
- Rust only loop: `cargo watch -x run` or `just dev`.
- Keep FFI edits small to stay under the sub-second feedback goal; avoid `cargo clean` unless absolutely required to preserve incremental gains.

## Troubleshooting pointers
- State management: see `docs/STATE_MANAGEMENT.md` for Riverpod patterns and bridging rules.
- Testing & coverage: see `docs/TESTING.md` for harness usage, golden flows, and coverage gates.
- FFI debugging: see `docs/FFI_DEBUGGING.md` for tracing and error formatting.
- Performance & memory: see `docs/PROFILING.md` for DevTools, benchmarks, and leak checks.
- Logging: see `docs/LOGGING.md` for structured JSON logging conventions.
- Build speed: see `docs/BUILD_OPTIMIZATION.md` for Rust and Flutter acceleration tips.
