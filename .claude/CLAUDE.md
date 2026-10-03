# KeyRx Development Guide

## Linux Host Notes
- System packages: `sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev libudev-dev libevdev-dev pkg-config`
- Cargo tools beyond `make setup` defaults: `wasm-pack`, `typeshare-cli` (UI prebuild), target `wasm32-unknown-unknown`
- Device access: `scripts/install.sh` adds you to `input`/`uinput` (needs re-login). Until then, run uinput tests as
  `sudo setpriv --reuid=$(id -u) --regid=$(id -g) --groups=$(id -G|tr ' ' ,),$(getent group input|cut -d: -f3) env PATH=$PATH HOME=$HOME cargo test ...`
- Type-check Windows code from Linux: `cargo clippy -p keyrx_daemon --target x86_64-pc-windows-gnu -- -D warnings` (checks only; not run)
- Run Windows tests without building on Windows: `cargo test -p keyrx_daemon --lib --target x86_64-pc-windows-gnu --no-run` (mingw linker installed), `scp` the printed `.exe` to `windows:` and run it there (e.g. `lib-tests.exe ipc::`). The Windows box is often too memory-starved to build `keyrx_daemon`
- The daemon's read side is ONE `DaemonQueryService` shared by IPC, REST, MCP and WS — add new status/metrics there, never per transport
- Which config is live is decided ONLY in `daemon/live_config.rs`: `run` follows the active profile, `run --config FILE` pins FILE at startup, a runtime activation (`DaemonSharedState::request_activation`, via `ProfileService`/IPC) switches profile; status is published only after the swap
- In-process e2e against the real event loop: `LinuxPlatform::scoped(<test kbd name>, <unique output name>)` — see `tests/live_profile_switch_test.rs`
- IPC endpoint = `ipc::IpcEndpoint` (per-user Unix socket `$XDG_RUNTIME_DIR/keyrx-daemon.sock`, fallback `/tmp/keyrx-daemon-<uid>.sock` / Windows named pipe `\\.\pipe\keyrx-daemon`); tests that need "no daemon at the default endpoint" set `XDG_RUNTIME_DIR` to a temp dir; client is `ipc::IpcClient`; both runners use `platform_runners::start_production_ipc_server`. Test mode listens on `keyrx-test-<pid>`
- Device patterns (`device_start`, `when_device`) have ONE rule: `keyrx_core::runtime::device_pattern` (glob, case-insensitive, matched against id/name/serial/path). Each device is routed to its first matching block (`daemon::remapping_state`)
- `scripts/verify/file-sizes.sh` is the real 500-line gate (run by `make verify`); `scripts/verify/file-size-baseline.list` may only shrink
- Monitoring wire types (latency, key events, state) are the typeshare structs in `keyrx_daemon/src/web/events.rs` for EVERY transport; if a REST response changes, regenerate `UPDATE_CONTRACT_FIXTURES=1 cargo test -p keyrx_daemon --test api_contract_test` and `npm run typeshare`, then fix the UI until `npm run type-check` passes

## Quality Gates

**CI enforcement is currently PAUSED** (GitHub billing out → `ci.yml` is
`ci.yml.disabled`). Gates are verified by running locally, not on push/PR.
Fast full suite: `cargo nextest run --workspace` (under the setpriv wrapper above).

```bash
make verify && scripts/fix_doc_tests.sh
cd keyrx_ui && npm test && npm run test:coverage && npm run test:a11y
```

## SSOT

- `.krx` binary is THE config source
- `ExtendedState` is THE state representation
- No duplicate formats (JSON/TOML)

## Critical Constraints (read before changing config/parser/build code)

### rkyv Binary Format — Enum Discriminants
All rkyv-serialized enums (`BaseKeyMapping`, `KeyMapping`, `Condition`, `ConditionItem`) use
`#[repr(u8)]` with **explicit discriminant values** (`= 0`, `= 1`, etc.). This makes variant
ordering in source code irrelevant to binary format. New variants get the next unused ID.
Enforced by `test_base_key_mapping_discriminant_stability`.

### Dual Parser — keyrx_core + keyrx_compiler
Both crates have a Rhai DSL parser (keyrx_core uses `spin::Mutex`/no_std, keyrx_compiler uses `std::Mutex`).
**Mapping creation logic is shared** via `keyrx_core::parser::builders` (SSOT). When adding a new DSL function:
1. Add builder in `keyrx_core/src/parser/builders.rs`
2. Add thin Rhai wrapper in BOTH `keyrx_core/src/parser/functions/` AND `keyrx_compiler/src/parser/functions/`
3. Add discriminant entry in `test_base_key_mapping_discriminant_stability`

### Build Order — Staleness Enforcement
`build.rs` **fails** a release build if WASM or UI dist is stale (dev/check/test builds only warn). Use `make build` for full pipeline.
Bypass for Rust-only dev: `KEYRX_SKIP_FRONTEND_CHECK=1 cargo build`

### Version SSOT
`Cargo.toml [workspace.package] version` is the single source. UI version injected at Vite build time
via `vite.config.ts` (reads Cargo.toml directly). No intermediate generated files.

## Troubleshooting

- Windows VM (vagrant): see `vagrant/windows/README.md`.
- WASM build/loading problems: the `wasm-troubleshooting` skill (`.claude/skills/wasm-troubleshooting/`).

## Shared Utilities

### Frontend (TypeScript/React)

**Time Formatting** (`src/utils/timeFormatting.ts`):
- `formatTimestampMs(micros)` - "1.23s"
- `formatTimestampRelative(timestamp)` - "1 hour ago"

**Key Code Mapping** (`src/utils/keyCodeMapping.ts`):
- `keyCodeToLabel(code)` - "A", "Enter"
- `parseKeyCode(label)` - 65

**Test Utilities** (`tests/testUtils.tsx`):
- `renderWithProviders(ui, options)` - Wrap with providers
- `createMockStore(state)` - Mock Zustand store

### Backend (Rust)

**CLI Common** (`keyrx_daemon/src/cli/common.rs`):
- `output_success(data, json)` - Format success
- `output_error(message, code, json)` - Format errors

### Dependency Injection

**API Context** (`src/contexts/ApiContext.tsx`):
```typescript
const { apiBaseUrl, wsBaseUrl } = useApi();
```

**ConfigStorage** (`src/services/ConfigStorage.ts`):
- `LocalStorageImpl` - Production
- `MockStorageImpl` - Testing

## References

- **Script Docs**: `scripts/CLAUDE.md`
- **Steering Docs**: `.spec-workflow/specs/ai-dev-foundation/`
- **Project Structure**: `.spec-workflow/steering/structure.md`
- **CI/CD**: `.github/workflows/`
- **Production Readiness**: `.spec-workflow/specs/production-readiness-remediation/PRODUCTION_READINESS_REPORT.md`
- **Rust Guidelines**: https://rust-lang.github.io/api-guidelines/
