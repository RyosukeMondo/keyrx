# Resume Plan — Finish 1.1.0 & Fix Outstanding Gaps

**Branch:** `main` (solo repo; the original `resume/finish-1.1.0` branch was merged)
**Created:** 2026-06-21 (resumed after development pause)
**Scope:** Everything from the resume assessment EXCEPT re-enabling CI
(GitHub billing is currently out, so CI cannot run — explicitly deferred).

This file is the **single source of truth** for the loop. Each loop iteration
reads it, works the ONE task marked `[NEXT]`, then updates this file and commits.

---

## Loop Protocol (run every iteration)

1. **Read this file.** Find the single task marked `[NEXT]`.
2. **Do that task only.** Read just the files it names — do not load unrelated context.
3. **Verify** with the task's acceptance check. NOTE: the test profile is
   `optimized + debuginfo`, so a cold `cargo test` took **365m**. Therefore:
   - Per slice, verify with `cargo check -p keyrx_daemon --tests` (fast; deps are
     cached after the first build, only the daemon crate recompiles).
   - Run the full `cargo test -p keyrx_daemon <module>` only at PHASE boundaries
     (end of C, D, etc.), not every slice.
   - Possible speedup (not yet applied; needs care — perf/latency tests may rely on
     optimization): lower `[profile.test]`/`[profile.dev]` opt-level in root Cargo.toml.
   Never run the full workspace build unless a task explicitly says to.
4. **Update this file:** change the finished task `[NEXT]`→`[x]`, write a one-line
   result note under it, and promote the next pending `[ ]` task to `[NEXT]`.
5. **Commit** both the code change and this file together:
   `git add -A && git commit` with message `resume(<id>): <summary>`.
6. **Schedule the next iteration.** If no `[ ]` or `[NEXT]` tasks remain (all `[x]`
   or `[blocked]`), STOP the loop and report a final summary instead.

**Rules:** Follow repo CLAUDE.md (max 500 lines/file, 50 lines/fn, no unwrap/expect
in daemon core, tests for new code). If a task is bigger than one iteration, split
it in place (add sub-tasks) rather than doing a giant turn. If a task is blocked,
mark it `[blocked]` with the reason and move the `[NEXT]` marker to the next task.

---

## Phase A — Release hygiene (1.1.0)

- [x] **A1 — Commit staged installer/release work.**
  Files already modified in working tree: `keyrx_daemon/build.rs` (version-sync
  fail-hard → auto-fix), `keyrx-installer.iss` (`shellexec` on launch entry),
  `scripts/build_installer.ps1` (UI-before-binaries ordering + helper hoist),
  `keyrx_ui/package.json` 1.0.0→1.1.0, plus `Cargo.lock`/`package-lock.json`.
  Action: review the diffs are coherent, then commit them as the 1.1.0 release-build
  fixes. Do NOT run a full installer build (heavy); just verify the diffs make sense.
  *Accept:* working tree clean of those 6 files; commit present.
  → DONE: diffs verified coherent; committed. Working tree clean.

## Phase B — Reconcile stale status docs

- [x] **B1 — Fix `.spec-workflow/SPEC_STATUS_SUMMARY.md`.**
  It lists bug-remediation-sweep 0/67, production-readiness-remediation 9/14,
  windows-quality-improvements 7/14 — all now fully complete in their tasks.md.
  Action: recount each referenced spec's tasks.md and update the summary numbers/dates.
  *Accept:* summary matches actual `[x]` counts.
  → DONE: recounted (bug-remediation 36/36, prod-readiness 26/26, windows-quality
    25/25, installer 17/17); rewrote summary to mark all four COMPLETE.

- [x] **B2 — Fix `.spec-workflow/specs/production-readiness-remediation/STATUS.md`.**
  Still says "BLOCKED — WebSocket Integration Test Infrastructure Required" though
  tasks.md is 26/26. Action: update to reflect completion; note WebSocket fix landed
  in commit d8694064.
  *Accept:* no false BLOCKED state.
  → DONE: status line → COMPLETE (26/26); added RESOLUTION banner crediting
    commit d8694064; historical analysis retained as record.

- [x] **B3 — Fix Production Quality Gates table in `.claude/CLAUDE.md`.**
  Claims 962 backend tests (repo has ~2,300+) and stale frontend numbers.
  Action: recount Rust `#[test]`/`#[tokio::test]` and TS test files, update the table,
  remove the now-resolved "Will become strict after WebSocket fixes" footnote or
  update its wording. Note CI enforcement is currently paused (billing).
  *Accept:* table reflects reality.
  → DONE: table now shows ~2,331 Rust test fns + ~1,874 TS cases/92 files; footnote
    replaced with "CI enforcement PAUSED (billing)" note; counts dated 2026-06-22.

## Phase C — Daemon functional gaps

> **DECISION (2026-06-22):** User chose **FULL LIVE PLUMBING + full autonomy, no
> more questions** ("all approved, fully implement, get deeper as needed"). So C1–C5
> are implemented for real: expose live runtime `ExtendedState`, run a production
> daemon IPC server with a live-state handler, surface real latency + events to
> CLI/web/Monitor. See memory `resume-loop-full-autonomy`.
>
> **FINDING that drove this:** `IpcCommandHandler` was constructed ONLY in test mode;
> test mode does no keyboard capture so it had no live data. CLI (`keyrx metrics`)
> and web API (`/api/metrics/latency|events`, `/api/daemon/state`) call these IPC
> commands over `/tmp/keyrx-daemon.sock`, but production didn't serve a live handler,
> and the 255-bit `ExtendedState` wasn't exposed anywhere shareable.
>
> Plumbing sub-slices below replace the old C1–C5. Do them in order.

- [x] **C1 — Design + telemetry source.** Read the event path (`processor/mod.rs`,
  `daemon/event_loop.rs`, `daemon/event_broadcaster.rs`, where `ExtendedState` lives
  and how latency is recorded). Then add a shared `DaemonTelemetry` source (live
  `ExtendedState` snapshot + `Arc<LatencyRecorder>` + an event ring buffer) that the
  processor/event loop updates on the hot path with minimal overhead. Document the
  concrete design as a comment block and refine the remaining C slices from it.
  *Accept:* telemetry source type compiles with unit tests; design noted in plan.
  → DONE: added `daemon/telemetry.rs` — `DaemonTelemetry` (state snapshot + latest
    `LatencySnapshot` + bounded events ring) and `TelemetryState` which OWNS the
    255-bit packing convention (0..128 modifiers / 128..192 locks / 192..255 layers)
    that was duplicated in `web/api/metrics.rs`. Poison-tolerant locks. 10 unit tests
    pass. Registered in `daemon/mod.rs`. Wiring into event loop = C3; web reuse of
    `TelemetryState` parsing also deferred to C3.

- [x] **C2 — Handler holds telemetry + implement read methods.** Extended
  `IpcCommandHandler` with `Option<Arc<DaemonTelemetry>>` (new `with_telemetry` ctor;
  `new` keeps `None`). Implemented `GetState`/`GetLatencyMetrics`/`GetEventsTail` to
  read telemetry, returning well-formed empty/zero defaults when `None` (test mode).
  This ABSORBS the former C3/C4/C5 handler logic. Updated/added unit tests (default-
  empty + telemetry-backed paths). *Accept:* lib compiles; tests added.
  → DONE: lib compiles clean (errors found were pre-existing rot in `tests/`, see
    below). Also fixed 2 pre-existing broken integration test files
    (`e2e_key_blocking_test.rs` missing `Sequence` arm; `websocket_infrastructure_test.rs`
    `DaemonEvent::Error.data`→`payload`). Per-slice verify changed to
    `cargo check -p keyrx_daemon --lib` (`--tests` drags in pre-existing broken targets).

**C3 — Integration: populate telemetry + run production IPC server.** Split into 3:

- [x] **C3a — Event-loop population.** `Daemon` now owns `Arc<DaemonTelemetry>`
  (created in `new()`, exposed via `telemetry()`). `run_event_loop` (Linux) and
  `process_one_event` (Windows) gained a `telemetry: Option<&DaemonTelemetry>` param
  (mirrors `latency_recorder`): they push event descriptions to the ring and update
  the packed state snapshot when a mapping triggers (helpers `build_telemetry_state`
  / `event_description` in event_loop.rs). All call sites updated (2 in mod.rs, 2 in
  windows_remap_pipeline_test.rs). *Accept:* lib compiles (✓ 56s, clean).

- [x] **C3b — Latency snapshot → telemetry + runner wiring.** Add a `telemetry`
  param to `start_latency_broadcast_task` so it also calls `telemetry.update_latency`
  with each computed snapshot. In `platform_runners/{linux,windows}.rs`, pass
  `daemon.telemetry()` to the broadcast task. Update the 2 test call sites (None).
  *Accept:* lib + relevant tests compile; latency flows to telemetry.
  → DONE: broadcast task feeds telemetry; both runners pass `daemon.telemetry()`;
    2 test sites + ignore doc example updated. `cargo check --lib` clean. CAVEAT:
    `linux.rs` is `cfg(linux)` so NOT compiled on this Windows host — edit was
    mirrored 1:1 from `windows.rs`; verify on Linux/CI before release.

- [x] **C3c-1 — Web live state (both platforms).** Found latency+events ALREADY
  flow to the web via `DaemonQueryService` (`get_latency_snapshot`/`get_event_log`);
  only `/api/daemon/state` was a gap (returned empty). Added `with_telemetry` +
  `get_state()` to `DaemonQueryService`; both runners now call `.with_telemetry(
  daemon.telemetry())`. Rewrote `get_daemon_state` to build from `TelemetryState`
  (SSOT) for both the query path and the IPC fallback — collapsed ~50 lines of
  duplicated 255-bit parsing into one helper. Added 2 query-service tests.
  *Accept:* `cargo check --lib` clean. → DONE. Web state works on Windows AND Linux
  via the query service (no Unix socket needed for the web path).

- [x] **C3c-2 — Linux production IPC server (CLI path).** The web no longer needs
  IPC, but the `keyrx metrics latency|events` CLI connects to `DEFAULT_SOCKET_PATH`.
  In the Linux production runner, spawn an IPC server there using
  `IpcCommandHandler::with_telemetry(daemon.telemetry())` (mirror the test-mode
  IPC server setup in `run_test_mode`). NOTE: `cfg(linux)` — NOT compilable on the
  Windows host; mirror carefully and flag for Linux/CI verification. *Accept:* code
  written + mirrored; verify on Linux later.
  → DONE (UNVERIFIED on this host): added `start_production_ipc_server(telemetry)`
    helper in linux.rs (best-effort: logs+skips on failure), called before the daemon
    run loop; captured `telemetry_for_ipc` before the daemon move. The handler logic
    (`with_telemetry` + `handle`) is cross-platform unit-tested (C2); only the Linux
    glue is unverified. **⚠ MUST `cargo build` on Linux before release** (see risk note).

- [x] **C5 — `ClearEvents` IPC + web clear-events.** Add `IpcRequest::ClearEvents`,
  handle it via `telemetry.clear_events()`, and implement the web clear-events stub
  (`web/api/metrics.rs:300`) against it. *Accept:* working clear; test.
  → DONE in 9f966b30 (C3c-3). Superseded 2026-09-27 by the single read model: IPC
  and REST now clear the SAME telemetry ring (see Phase L).

- [x] **C5b — CLI events follow mode** (`cli/metrics.rs:40,69`). Replace the
  `Err("not implemented")` with a poll loop that repeatedly calls `GetEventsTail` and
  prints new events until interrupted. *Accept:* follow streams; test or manual note.
  → DONE (bd4e869f, 732bfb69): `--follow` polls with a timestamp cursor; found and fixed the IPC server answering one request per connection (EPIPE for reused clients); IPC now carries LatencyStats/KeyEventData.
- [x] **C6 — Real profile counts** (`src/web/api/profiles.rs:169,170`).
  device count and key-mapping count hardcoded to 0. Compute device count per profile
  and parse Rhai config to count mappings. *Accept:* non-zero correct counts; test.
  → DONE (0e9a8031): device/key/layer counts from the compiled .krx (layer_count was a `layer(` text heuristic, always 1).
- [x] **C7 — Parse error line number** (`src/web/api/profiles.rs:563`).
  Hardcoded line 1. Parse actual line from compiler error. *Accept:* correct line; test.
  → DONE (cc615a50, 22825a8c): CompilationError::location() parses "(line N, position M)"; POST /api/profiles/validate {config} added (the UI called it, it did not exist); errors carry column + length like the UI expects.

## Phase D — UI functional gaps

- [x] **D1 — Device rename API + UI.** (2026-09-27: backend rename is
  `PUT /api/devices/:id/name`; the UI device-*scope* chain was dead — scope was
  removed from the backend in Task 27 — and was deleted in 6ff105a4.)
  Backend endpoint missing; UI TODOs at `DevicesPage.tsx:304,328,338,447`; tests
  skipped at `DevicesPage.test.tsx:1292,1381`. Action: add backend rename endpoint,
  wire UI, un-skip tests. *Accept:* rename works end-to-end; tests pass.
  → DONE (5170ae1f): the backend route existed but 404ed for never-registered devices; ensure_registered on first web edit; rename returns the DeviceEntry (pinned by a contract fixture); UI schemas/mocks aligned; 2 rename workflow tests un-skipped.

- [x] **D2 — KeyConfigModal mapping editing** (`KeyConfigModal.tsx:200`).
  TODO stub. Implement edit flow. *Accept:* editing works; test.
  → DONE by removal (29bee7d5): KeyConfigModal was never rendered - the live editor KeyConfigPanel already does edit/clear; the modal and its MappingConfigForm family were deleted (misgrouped: the deletions landed in 35eb7389).

- [x] **D3 — metricsStore live layer** (`stores/metricsStore.ts:93`).
  Hardcodes `'Base'`. Source current layer from daemon state (depends on C1).
  *Accept:* shows real active layer; test.
  → DONE (f05751b5): daemon derives the active layer from the block's when-modifiers; labels are hex like the DSL (were decimal); UI event records carry the live layer. Verified live.

## Phase E — Architecture remediation decision

- [x] **E1 — Decide fate of architecture-remediation / architecture-completion specs.**
  `architecture-remediation/tasks.md` (52 open, 0 started) and
  `architecture-completion/tasks.md` (30 open) overlap heavily with the COMPLETED
  `comprehensive-architecture-refactoring` spec. Action: audit overlap; for each open
  task, mark it (a) already-done-elsewhere, (b) genuinely-still-needed, or (c) obsolete.
  Write the verdict into those tasks.md files. If genuinely-needed work remains, add
  concrete slices to Phase F below. *Accept:* clear documented verdict; no ambiguity.
  → DONE (2026-09-27): verified all 82 tasks against code (4 parallel research passes,
    no full builds). architecture-remediation: 35 done-elsewhere / 10 still-needed /
    7 obsolete. architecture-completion: 21 done-elsewhere / 5 still-needed / 4 obsolete.
    Most of both specs is stale — superseded by comprehensive-architecture-refactoring
    plus the C/G/L work since. Genuinely-needed items promoted to Phase F below.

## Phase F — Genuinely-needed architecture work (populated by E1)

- [x] **F1 — Split `config/profile_manager.rs` (712 lines) into a ProfileRepository +
  orchestrator; stop swallowing lock-poison errors.** It's the only file-size-gate
  violation this repo controls directly (the other two are `platform_runners/windows.rs`,
  delegated to the Windows session per G3/G5, and F6 below) and it still has 9
  `.expect("...poisoned")`/`.unwrap()` calls in production code (e.g. lines 672, 710,
  741, 856, 896, 929, 970, 1002) that don't propagate errors to the caller.
  `config/profile_compiler.rs` already shows the pattern (compilation logic extracted,
  `profile_manager.rs::compile_and_reload` delegates to it) — do the same for file I/O
  (scan_profiles/create/list/load_active_profile/clear_active_profile_file).
  *Accept:* `profile_manager.rs` ≤500 code lines; new `profile_repository.rs` owns the
  I/O; 0 unwrap/expect outside `#[cfg(test)]`; `scripts/verify/file-sizes.sh --update`
  drops it from the baseline.
  → DONE (da9c7966 split, 95ae1830): split into mod/crud/activation/persistence/types; poisoned-lock expects replaced by recovery; 0 unwrap/expect outside tests.
- [x] **F2 — Finish or delete the orphaned e2e test-harness split.**
  `keyrx_daemon/tests/harness/{mod,harness,error,config,assertions}.rs` and
  `tests/virtual/{mod,basic,complex,layers,passthrough,advanced_output,advanced_sequences}.rs`
  exist with real content (commit `90834f9d`) but are referenced by zero `mod`
  declarations anywhere in the repo. The files that actually compile are still the
  3386-line `tests/e2e_harness.rs` (pulled in via `mod e2e_harness;` from
  `virtual_e2e_test.rs:33`, `tap_hold_e2e_test.rs`, `e2e_windows_basic_test.rs`,
  `e2e_windows_bugs_test.rs`) and the 1904-line `tests/virtual_e2e_test.rs`. Someone
  split the code once and never rewired the callers or deleted the old files — pick a
  side. *Accept:* either (a) rewire the 4 dependent test files to `mod harness;`/
  `mod virtual;` and delete both monoliths, verified by
  `cargo check -p keyrx_daemon --tests`, or (b) delete the orphaned `tests/harness/`
  and `tests/virtual/` trees. No dead duplicate code either way.
  → DONE (74cead68): deleted the orphaned split; the monoliths had changed after it (85c2dbbe), so the copies were stale.

- [x] **F3 — Wire `EnvProvider`/`FileSystem` traits into `ProfileManager`/`ConfigService`,
  or delete them.** `keyrx_daemon/src/traits/{env.rs,filesystem.rs}` define real
  traits + Real/Mock impls but have zero consumers outside their own module; ~20 raw
  `std::env::var`/`fs::` call sites remain scattered in `profile_manager.rs`/
  `config_service.rs`. *Accept:* `ProfileManager`/`ConfigService` take an injected
  `EnvProvider`/`FileSystem` (preferred — makes them mockable in tests), or the traits
  are removed if injection isn't worth it here; either way
  `grep -rl 'EnvProvider\|FileSystem' keyrx_daemon/src` shows real consumers or the
  traits are gone.
  → DONE (b8aa2064): traits removed (only their own test used them); DI stays via constructor params.

- [x] **F4 — Adopt the frontend logger; remove raw `console.*` calls.**
  `keyrx_ui/src/utils/logger.ts` exists but 96 raw `console.(log|error|warn|info)`
  calls remain in non-test UI source (e.g. `ConfigurationPanel.tsx`,
  `RhaiSyncEngine.tsx`, `metricsStore.ts`, `useSimulation.ts`) — the logger was built
  but never adopted. *Accept:* `grep -rn "console\.\(log\|error\|warn\|info\)"
  keyrx_ui/src --include=*.ts --include=*.tsx | grep -v test` returns 0 (or only
  intentionally-excepted files); messages go through `logger.ts`'s structured format.
  → DONE (273dd679): ~78 calls in 24 files go through utils/logger.ts; only GlobalDebugPanel (intentional console capture) and JSDoc examples remain. Also vite drop_console:true was stripping logger.warn/error from production builds.

- [x] **F5 — Shrink the stale file-size baseline.**
  `scripts/verify/file-size-baseline.list` still lists 15 files; 9 of them
  (`cli/error.rs`, `cli/profiles.rs`, `daemon/error.rs` [now `error.rs`],
  `platform/linux/keycode_map.rs`, `platform/windows/rawinput.rs`,
  `web/api/diagnostics.rs`, `web/api/profiles.rs`, plus 2 UI files) were already
  split in recent commits and no longer exist at those paths, so the baseline is
  actively hiding how close the gate is to fully green. *Accept:* run
  `scripts/verify/file-sizes.sh --update` after F1 and F6 land; the baseline shrinks
  to only genuinely-outstanding files (`platform_runners/windows.rs`, pending G3).
  → DONE (57bede41): baseline shrunk to `platform_runners/windows.rs` only (G3).
- [x] **F6 — Split `config/simulation_engine.rs` (529 lines).** The one file-size-gate
  violation with no open task anywhere tracking it (unlike `profile_manager.rs` → F1,
  and `platform_runners/windows.rs` → delegated G3/G5). *Accept:*
  `simulation_engine.rs` ≤500 code lines, logic extracted into a focused sibling
  module, `scripts/verify/file-sizes.sh` passes without it in the baseline.
  → DONE (da9c7966): split into mod/types/scenarios/engine/tests.
- [x] **F-backlog — lower-impact or process items, do opportunistically:** remove
  `eprintln!` outside `cli/` (6 files: `config/layout_manager.rs`, `web/ws_rpc.rs`,
  `platform/linux/{mod,input_capture,output_injection}.rs`, `main.rs`); add
  architecture diagrams (doc-only, `.claude/CLAUDE.md` already covers the DI/SSOT
  patterns in prose); reconcile `.claude/CLAUDE.md`'s "4 fail" backend test count and
  the UI's "2 fail" (`DevicesPage`) against whether G4/G2's fixes actually cover them
  (eprintln! part DONE f9053d21; DevicesPage fixed a45c56a8)
  — re-run `cargo test --workspace` and `npm test` once, update the table, and only
  then treat architecture-completion 3.5.1–3.5.3 as closed.
  → DONE: eprintln! cleanup (f9053d21); counts reconciled - full suite 2024 pass / 0 fail / 91 ignored, UI 1844 / 0 / 23, lint 0 (7c0705cb fixed the last stale test). Architecture diagrams skipped (no doc requested).

---

## Phase L — Linux bring-up & read-model redesign (2026-09-27, Linux host)

First session on a Linux host. The Linux build had never compiled (C3 edits were
mirrored blind from Windows). Done, each verified live on a running daemon:

- [x] **L1 — Linux release pipeline from a fresh setup** (06856e2c): rhai
  `wasm-bindgen` feature (WASM link), `LC_ALL=C sort` in the WASM source hash
  (ja_JP locale made every build "STALE WASM"), `typeshare-cli` in `make setup`.
- [x] **L2 — Linux compile fixes** (d156ded0).
- [x] **L3 — Single read model** (62894263): `DaemonQueryService` (over
  `DaemonSharedState` + `DaemonTelemetry`) is the ONE source for IPC, REST, MCP and
  the WS latency feed. Fixed: IPC status hardcoded zeros; two latency aggregators
  stealing samples; two event rings; web→IPC silent fallbacks; per-transport
  activation side effects.
- [x] **L4 — One reload flag** (b93d6b13): `DaemonSharedState` shares the SIGHUP
  `ReloadState` flag; removed `ProfileService`'s self-SIGHUP (killed test binaries
  and `--test-mode` daemons).
- [x] **L5 — Stale socket = "daemon not running"** (ace4ddcd).
- [x] **L6 — Linux capture never blocks on an idle keyboard** (6eec2624): the
  event loop sat in `read()` on the first grabbed device, swallowing all other
  keyboards (incl. the physical one). Non-blocking fds + `poll()`, no lost batch
  events, `PlatformError::NoInput`.

Verification now: `cargo clippy --target x86_64-pc-windows-gnu` type-checks the
Windows code from Linux, but Windows is NOT run. Linux is the verified platform.

## Phase G — Next (ordered by impact)

- [x] **G1 — Linux: activating a profile must actually change remapping.**
  → DONE (e58f05bf..da39d296). `daemon/live_config.rs` is the one resolver:
    `run` follows the active profile, `run --config FILE` pins FILE at startup,
    a runtime activation switches profile, other reloads re-read a pinned file
    or re-resolve `.active`. Transports *request* an activation; the daemon
    publishes active_profile/config_path only after swapping the event loop's
    RemappingState (failed load = old mappings + old status). Linux loop and
    Windows message loop share `Daemon`'s reload path. Also fixed on the same
    path: ProfileService never attached to the daemon (config saves never
    reloaded); CLI `profiles activate` never told a running daemon (now IPC);
    rename of the active profile not persisted to `.active`; `load_config`
    leaked the file on every load; SIGTERM killed the daemon outright (no
    graceful shutdown, stale socket). Units/.desktop run `keyrx_daemon run`.
    Proof: `tests/live_profile_switch_test.rs` (real event loop, virtual kbd +
    captured output; fails when the swap is removed) and a live run of the
    installed binary: F23→F24, REST activate → F22, CLI activate → F24,
    SIGHUP after `.active` edit → F22, `--config` startup → F24.
- [x] **G2 — UI↔API metrics contract.**
  → DONE (be4b2673). Worse than listed: latency had 4 shapes (REST `min_us`,
    WS `min`, WS-RPC zeros, MCP `minUs`), WS-RPC events read the macro
    recorder, the Monitor page never called REST. Now `web/events.rs`
    typeshare types (LatencyStats/KeyEventData/DaemonState) are the wire
    format on WS push, REST, WS-RPC and MCP; telemetry ring holds KeyEventData;
    IPC renders `summary()`. Contract: `tests/api_contract_test.rs` pins real
    responses → `keyrx_ui/src/test/contract/*.json`, `src/types/contract.check.ts`
    fails `npm run type-check` on drift, MSW mocks serve the fixtures. Monitor
    page seeds history/state from REST. Verified live (headless Chromium
    screenshot of /monitor with injected keys). UI suite 1916 pass / 2 fail
    (pre-existing DevicesPage, same at HEAD).
- [x] **G3 — Windows production IPC server** → request
  `docs/requests/G3-windows-production-ipc.md` (Windows session implements;
  code shipped as bundle `D:\users\ryosu\bundles\keyrx-main.bundle`). (`keyrx_daemon status|metrics` cannot
  work on Windows; `DEFAULT_SOCKET_PATH` is a Unix path). Use `ipc::server::spawn`.
  → DONE (PR #1 f928a51f, merged 01b3f1df; 26bc4ac8 test wording). IpcEndpoint
    (socket file / named pipe), one spawn helper, IpcClient; windows.rs split.
    Verified: Linux clippy + windows-gnu clippy, full suite 2032/0/91, live
    Linux IPC; on Windows the cross-built lib tests (31 ipc:: incl. named-pipe
    round trip + second-server refused) and CLI<->pipe<->REST in test mode.
    NOT verified: production `run` with keyboard hooks on the Windows desktop
    (needs the interactive session; the host was too memory-starved to build).
- [x] **G4 — Test hygiene.** → DONE (02b0747c, c3589cd2, 3979abba). Root causes:
    `/api/devices` opened+closed every evdev node (close waits an RCU grace
    period, ~15 ms × 32) → sysfs enumeration, 440 ms → <1 ms, works without the
    input group. Config-dir: 9 callers bypassed `get_config_dir` (web layouts/
    diagnostics/config ignored KEYRX_CONFIG_DIR) → one resolver, pure + tested
    without env mutation (fixed the HOME race). compile_time `> 0` and script
    cwd assumptions fixed.
- [x] **G6 — Config-editing sprawl.** → DONE (c808cc18, 6ff105a4, a45c56a8).
    REST /api/config* was a second copy of ConfigService with an IPC self-call;
    neither copy compiled or reloaded. Now REST → ConfigService →
    ProfileService::set_profile_config (compile + reload if loaded). Found the
    wider class: UI called routes that don't exist (global layout, config
    client, device scope) → REST global-layout added, dead UI code removed,
    `tests/api_route_contract_test.rs` guards it. DevicesPage "Saved" feedback
    restored (the 2 long-failing UI tests).
    Still open: UI `validateConfig` POSTs `/api/profiles/validate` (server only
    has `/profiles/:name/validate`; the GET-based route check can't see it
    because `/profiles/:name` matches) → add after the profiles.rs split (G5).
- [x] **G7 — Profile swap with keys held.** Swapping RemappingState while a
  remapped key is down can leave its output pressed (release maps differently).
  Release held outputs before the swap.
  → DONE (b37e1a66): HeldOutputs platform decorator releases held outputs before any config swap (both platforms) and on shutdown; live e2e holds CapsLock across a switch.
- [x] **G8 — Only the first `device_start` block is applied** (warned at load);
  the Linux platform grabs `*` regardless of config patterns.
  → DONE (39758ae1, 9b0377e3): process_event never passed the device, so when_device never matched; one device-pattern rule in keyrx_core (case-insensitive, any identity) used by device_start and when_device; RemappingState routes each device to its first matching block with per-device state. Live e2e with two virtual keyboards. Limitation: Linux still grabs all keyboards and re-injects unmatched ones unchanged.
- [x] **G9 — UI lint debt:** `npm run lint` has 14 errors in untouched files
  (e.g. `MonitorPage.tsx` refs-during-render). EventRecord `layer` is always
  `'Base'` (see D3). `scripts/verify/{ssot,contracts}.sh` are stubs (exit 0).
  → DONE (21e1efd3): lint 0 errors / 0 warnings; fixed a real bug (the paused Monitor log silently un-froze); test-naming rule matched "spec" inside "special"/"Inspector".

- [x] **G5 — Oversize files** (gate made real in dd1d1638; 14 files being
  split; `windows.rs` handed to the G3 request) (>500 code lines): `web/api/diagnostics.rs`,
  `platform_runners/windows.rs`, `web/api/profiles.rs`.
  → DONE except `platform_runners/windows.rs` (in the G3 request): the gate was a stub (exit 0) - made real with a shrink-only baseline (dd1d1638, 57bede41); 14 files split by 6 parallel agents, each verified and merged (stale worktree bases re-applied by hand).

## Phase H — Linux daily-use overhaul (2026-09-28)

Goal: the user starts using keyrx on Linux. Walk every feature as a user,
make each one analyzable/debuggable, speed up iteration, fix real bugs.

- [x] **H1 — One real simulation engine.** → DONE (worktree
  agent-a6d9fa63682606c32, 9 commits c3a9748a..e9cd6c82). Routing/remapping
  moved into `keyrx_core::runtime::remapper::Remapper` (`process()` +
  `tick()`); one deterministic virtual-time driver
  `keyrx_core::simulate::run()` on top. The daemon event loop, CLI
  `simulate`/`test`, REST `/api/simulator`, WS-RPC `simulate` (was a
  placeholder that echoed input as "output"), and the WASM browser
  simulator all drive this same engine now - `SimulationEngine::new` also
  genuinely validates the loaded `.krx` instead of ignoring its bytes.
  Built-in scenarios no longer claim a config-specific PASS/FAIL (there is
  none to know without the profile); they check the one invariant that
  holds for any config, `simulate::stuck_keys` (nothing left held at the
  run's end). Found and fixed along the way: (1) custom modifier/lock
  state was accidentally per-device instead of the DSL-manual-documented
  shared-across-devices model (since 9b0377e3) - now one
  `SharedModifierState` behind `Arc<spin::Mutex<_>>` per `Remapper`,
  tap-hold/press-tracking staying per-device; (2) `simulate`/`test`
  without `--profile` silently guessed a hardcoded "default" instead of
  the daemon's actual active profile; (3) `record` wrote a format
  `simulate --events-file` couldn't read at all. Real CLI repro verified
  before/after (04-dual-function-keys.rhai, CapsLock-hold+C): before
  `C↓ C↑ Control↓ Control↑`, after `LCtrl↓ C↓ C↑ LCtrl↑`. Gates: `cargo
  clippy --workspace -- -D warnings` / `cargo test --workspace --lib`
  clean for every touched file (695 passed; the only 10 failures are the
  pre-existing uinput-needs-`input`-group set plus one dist-stub-only
  static_files test, both documented, neither touched by this work);
  `npm run build:wasm` + the useWasm/useSimulation vitest suites +
  `type-check` green with zero UI changes needed. Deferred (separate,
  pre-existing, not touched here): `keyrx_core/src/runtime/lookup.rs` is
  1 line over the file-size gate (main commit 38302865, not in the
  baseline list).
- [x] **H2 — UI walkthrough on a live Linux daemon** (Config, Devices,
  Monitor, Simulator tab); ConfigPage test failures from the handover.
- [x] **H3 — Linux ops & debuggability**: grab only devices a block matches
  (G8 limitation), permission diagnostics, install/unit flow, help text.
- [x] **H4 — Iteration speed**: measure test/build loops, cut the slow ones.
  → DONE: full suite 84 s serial → ~20 s with `cargo nextest run --workspace`
  (installed); a keyrx_core edit no longer breaks dev/check/test builds
  (stale WASM only fails release, eaf83f95); tests no longer collide with a
  running daemon (per-user socket, bf4b3a7d) or flake on live device counts;
  `RUST_LOG` honoured, `--debug` limited to keyrx crates (f3711451).
- [x] **H5 — Engine output semantics** → DONE: (a) 15d97bdb HeldOutputs
  refcount; (b) b3660c37 QMK permissive hold with buffered replay. (after H1; verified with a scratch
  harness on the real core):
  (a) a `with_shift`/`with_ctrl` mapping pressed while the user physically
  holds that modifier releases it: real LShift held + `/`→with_shift(2) + H
  → `LShift↓ LShift↓ 2↓ 2↑ LShift↑ H↓` (H comes out lowercase). Fix: the
  output stream is refcounted per key (one uinput device → global), press
  emitted for the first holder, release for the last.
  (b) tap-hold is "hold on other key press", not permissive hold as the docs
  say: rollover `B↓ E↓ B↑ E↑` with user_layout's `tap_hold(B, Enter, MD_00)`
  yields the MD_00 layer's E (Num2) instead of Enter,E. Implement QMK
  permissive hold (buffer the interrupting press; hold only if it is
  released while the tap-hold key is still down; timeout still → hold).
- [x] **H7 — Delete the test-only engine.** → DONE: `keyrx_daemon::processor`
  (`EventProcessor`, its `logging`/`test_utils`/`tests_coverage`) deleted, plus
  the dead `keyrx_daemon/src/logging.rs` (JSON tracing setup nothing called;
  `tracing`/`tracing-subscriber` deps dropped) and the unused
  `keyrx_ui/src/components/config/DiagnosticsPanel.tsx`. `processor_test.rs`,
  `integration_test.rs`, `multi_device_integration_test.rs` deleted - most of
  their assertions were already covered by `keyrx_core/tests/runtime_event_test.rs`
  and `keyrx_core/src/runtime/lookup_tests.rs` (which exercise the same
  `process_event`/`KeyLookup` those suites drove); the genuine gaps ported onto
  the real engine: a lock-gated-mapping persistence test
  (`runtime_event_test.rs`) and a two-`device_start`-block routing test through
  ONE production `Remapper` with interleaved events
  (`runtime/remapper_tests.rs::two_devices_routed_by_one_remapper_stay_independent_when_interleaved`
  - the old suites gave false confidence here via two separate
  `EventProcessor`s, which proves nothing about routing).
  `story_acceptance_test.rs` (Rhai→compile→engine pipeline, hot-reload) kept
  and rewritten onto `SimulationEngine`/`keyrx_core::simulate::run`. No bugs
  found by the ported tests; all gates green (`cargo nextest run --workspace`
  2008/2008, UI 1846/1869 pass/23 skipped).
- [x] **H8 — Device status tells the truth.** `services/device_service.rs:65`
  reports `active: true` for every enumerated keyboard (even in test mode);
  the Devices page Enable/Disable toggle only writes localStorage; the
  shared device_count is not refreshed on hotplug; live uinput tests panic
  instead of skipping without the input group.
- [x] **H6 — Linux capture drops what it cannot map**: EV_KEY codes without
  a `KeyCode` (brightness, mic-mute, vendor keys) and non-key events on a
  grabbed node (EV_REL of a keyboard with a pointer) are silently eaten.
  Forward them raw to the output device.

**Phase H result (2026-09-28):** all H items done. Also found and fixed on
the way: web origin guard (any web page could read the keystroke WS stream
and switch profiles - 5fc9600c); daemon grabbing its own output device via
hotplug (8eec867d); REST device list/edits bypassing DeviceService
(be9b834d, 4c2017a8); per-user IPC socket (bf4b3a7d); dead layer
mappings + more-specific-layer precedence (38302865). Verified live with
the release binary against a virtual keyboard (tap, hold, rollover,
timeout, held Shift, pass-through, IPC/REST/doctor). Gates: nextest
2021/2021, UI unit 1858/0, clippy linux+windows, fmt, file sizes.
Open, small: the config page lists all 257 layers instead of the used
ones; UI integration suite (`npm run test:integration`) has ~16
pre-existing failures (websocket-msw, tests needing a daemon on 13030).

## Handover (2026-09-28, end of Linux session)

State: every phase above is `[x]`. `main` pushed to GitHub (PR #1 merged) and to
the `windows` remote. Open items, none blocking:

- **Windows desktop live check (user).** G3 was verified with cross-built test
  exes and test-mode CLI↔pipe↔REST, not with a production `run` on the
  interactive desktop (real keys through the hook). Steps 1–4 are in
  `docs/requests/G3-windows-production-ipc.md`.
- **ConfigPage UI tests:** resolved. `ConfigPage.integration.test.tsx` and
  `ConfigPage.a11y.test.tsx` pass (2026-10-01: unit 1901 pass, a11y 11 pass,
  integration 85 pass; only `config-editor` and `rpc-communication` fail, and
  only because they need a live daemon on port 13030).
- **Installed Linux binary** (`~/.local/bin/keyrx_daemon` and `keyrx_compiler`):
  rebuilt and reinstalled 2026-10-01 with the round-2 merge (previous kept as
  `*.pre-k.bak`).
- **Branch `g3-review`** (local, Linux) is the PR #1 review checkout and can be
  deleted.

## Status Log
- 2026-06-21: Plan created. A1 marked [NEXT].
- 2026-06-22: A1 done (installer/release fixes committed). B1 [NEXT].
- 2026-06-22: Phase B done (B1/B2/B3 — stale docs reconciled). C1 [NEXT].
- 2026-06-22: C1 investigation revealed IPC telemetry is test-mode-only with no
  live data source. Asked user → chose FULL LIVE PLUMBING + full autonomy (no more
  questions). Phase C rewritten into plumbing slices C1–C5b. C1 (design) [NEXT].
  Autonomy recorded in memory `resume-loop-full-autonomy`.
- 2026-06-22: C1 DONE (daemon/telemetry.rs, 10 tests pass). Discovered cold
  `cargo test` = 365m (optimized test profile) → switched per-slice verify to
  `cargo check --tests`, full tests at phase boundaries only. C2 [NEXT].
- 2026-06-22: C2 DONE (IpcCommandHandler telemetry field + 3 read methods, absorbs
  former C3/C4/C5 handler logic). `--tests` revealed PRE-EXISTING compile rot in
  integration tests (suite doesn't currently build, contradicting "962/962 pass");
  fixed 2 files, more may exist — sweep at Phase-C boundary. Per-slice verify → `--lib`.
  C3 (event-loop + prod IPC integration) [NEXT].
- 2026-06-22: C3 split into C3a/C3b/C3c. C3a DONE — Daemon owns Arc<DaemonTelemetry>,
  event loop (both platforms) populates state+events; lib compiles clean (56s). C3b
  (latency task + runner wiring) [NEXT].
- 2026-06-22: C3b DONE — latency snapshot feeds telemetry; runners pass
  daemon.telemetry(); lib check clean. NOTE: linux.rs not compiled on Windows host
  (mirrored from windows.rs). C3c (prod IPC server + Windows web) [NEXT].
- 2026-06-22: C3c-1 DONE — web /api/daemon/state serves live state via
  DaemonQueryService.get_state() (telemetry-backed) on both platforms; SSOT cleanup
  of 255-bit parsing. lib check clean. C3c-2 (Linux CLI IPC server) [NEXT].
- 2026-06-22: C3c-2 DONE but UNVERIFIED (cfg-linux, can't compile on Windows) —
  start_production_ipc_server helper mirrors run_test_mode. C3 COMPLETE. Added
  cross-platform verification risk note. C5 (ClearEvents + web clear) [NEXT].
- 2026-09-27: First Linux-host session. Linux build fixed + verified; Phase L
  (L1–L6) done, see above. Live parity check: IPC == REST for status, latency,
  events. Full suite on Linux: 2005 pass / 5 fail (all G4, pre-existing) / 91
  ignored; needs `input` group for uinput tests. G1 [NEXT].
- 2026-09-27: G1 DONE — profile activation swaps live remapping on Linux (and
  Windows via the same Daemon path); proven by e2e test + live run. Full suite
  (input group): 2020 pass / 4 fail (all G4) / 91 ignored. G2 [NEXT].
- 2026-09-27: G2 DONE — one wire format for latency/events/state across WS,
  REST, WS-RPC, MCP, pinned by cross-language contract fixtures. G3 [NEXT].
- 2026-09-27: E1 DONE — audited all 82 open tasks in architecture-remediation/
  architecture-completion against current code; most is stale (superseded by
  comprehensive-architecture-refactoring + the C/G/L work). 56 done-elsewhere,
  15 still-needed, 11 obsolete. Verdicts written into both tasks.md. Real gaps
  promoted to Phase F (F1-F6 + backlog): profile_manager.rs split, orphaned
  e2e-harness split, dead EnvProvider/FileSystem traits, unadopted UI logger,
  stale file-size baseline, simulation_engine.rs split.
- 2026-09-27: G3 request written for the Windows session (code synced by
  push-to-checkout over ssh). G4-G8, C5b, C6, C7, D3, E1, F1, F5, F6 DONE;
  G5 done except windows.rs (G3). G9 (UI lint) running. D1 [NEXT].
- 2026-09-28: Phases D, E, F, G complete except G3 (Windows session, see
  docs/requests/G3-windows-production-ipc.md). Nothing is [NEXT] on Linux.
- 2026-09-28: G3 merged (PR #1). All phases done; only the Windows desktop live
  check (real keys through the hook) remains for the user.
- 2026-09-28: Wrap-up. UI use-case guide and ConfigPage work committed; main
  pushed to origin and windows. See Handover.
- 2026-10-01: Merged the three agent branches (DSL/core TapHoldKey + parser
  lints, daemon SYN_DROPPED/hot reload/CLI, UI a11y/UX) and gated them
  together: fmt, clippy (workspace, --all-targets, windows-gnu) clean;
  nextest 2088 pass / 70 skipped; file sizes OK; UI type-check, lint, unit
  1901 pass, a11y 11 pass, integration 85 pass (config-editor and
  rpc-communication need a live daemon on 13030 and fail without one).
  Fixed: all-targets clippy debt in tests/benches, the UI WebSocket URL
  split (`constants.WS_BASE_URL` vs `env.getWsUrl()`), the MSW websocket and
  FFI integration tests, and `scripts/build.sh` building only the daemon (the
  installed `keyrx_compiler` was stale and rejected `tap_hold(..., "VK_LCtrl")`).
  Release build installed and the real service restarted (doctor green, one
  device grabbed, IPC == REST). Scratch-instance live regression 14/14: Caps
  tap=Esc / hold=real LCtrl, home-row mod, hot reload (and a bad edit keeps the
  old config), SYN_DROPPED flood recovery, emergency chord.
- 2026-10-01 (evening): Round-2 merge (OneShot=8 / TapHoldKeyTimeoutOnly=9,
  deterministic .krx, parser diagnostics; unique `keyrx-out-<pid>` output,
  `config_error`, 5 ms min key-down, one-handed emergency stop, in-place CLI
  edits; UI round-trip-safe save, undo, Japanese) gated together: fmt, clippy
  (workspace all-targets + windows-gnu), nextest 2185 pass / 70 skipped, doc
  tests, file sizes, UI type-check/lint/unit 2018 pass/a11y 14 pass. Merge
  breakage fixed at the root: `validate_krx_format` checked `KRX\0` while the
  compiler writes `KRX\n` (now uses `KRX_MAGIC`, tested on a real compiled
  file); `/api/profiles/validate` lost line/column when the compiler reworded
  its errors (position is now data in `CompilationError::Syntax`); the UI
  Rhai parser surfaced `fn` bodies as phantom mappings (examples/08); a
  doctest and two lint nits. Release build installed, real service restarted
  (doctor green, only `USB Keyboard` grabbed, output `keyrx-out-<pid>`, old
  .krx still loads). Scratch live regression 67/67 checks (one harness read after the
  chord-stopped daemon's device vanished, not a product failure): caps,
  home-row, one_shot, tap_hold_timeout_only, min key-down (5.07 ms vs 0.01 ms
  with 0), hot reload good/bad + config_error, broken active profile grabs
  nothing, SYN_DROPPED flood, both emergency stops, example 08, all templates.
