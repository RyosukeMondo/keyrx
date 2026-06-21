# Resume Plan — Finish 1.1.0 & Fix Outstanding Gaps

**Branch:** `resume/finish-1.1.0`
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

- [NEXT] **C3b — Latency snapshot → telemetry + runner wiring.** Add a `telemetry`
  param to `start_latency_broadcast_task` so it also calls `telemetry.update_latency`
  with each computed snapshot. In `platform_runners/{linux,windows}.rs`, pass
  `daemon.telemetry()` to the broadcast task. Update the 2 test call sites (None).
  *Accept:* lib + relevant tests compile; latency flows to telemetry.

- [ ] **C3c — Production IPC server + Windows web wiring.** In the Linux production
  runner, spawn an IPC server on `DEFAULT_SOCKET_PATH` using
  `IpcCommandHandler::with_telemetry(daemon.telemetry())` so CLI/web IPC queries hit
  live data. On Windows (no Unix socket), expose `daemon.telemetry()` through the web
  `AppState` so `/api/daemon/state` + `/api/metrics/*` read it directly. *Accept:*
  prod daemon serves live state/latency/events end-to-end.

- [ ] **C5 — `ClearEvents` IPC + web clear-events.** Add `IpcRequest::ClearEvents`,
  handle it via `telemetry.clear_events()`, and implement the web clear-events stub
  (`web/api/metrics.rs:300`) against it. *Accept:* working clear; test.

- [ ] **C5b — CLI events follow mode** (`cli/metrics.rs:40,69`). Replace the
  `Err("not implemented")` with a poll loop that repeatedly calls `GetEventsTail` and
  prints new events until interrupted. *Accept:* follow streams; test or manual note.

- [ ] **C6 — Real profile counts** (`src/web/api/profiles.rs:169,170`).
  device count and key-mapping count hardcoded to 0. Compute device count per profile
  and parse Rhai config to count mappings. *Accept:* non-zero correct counts; test.

- [ ] **C7 — Parse error line number** (`src/web/api/profiles.rs:563`).
  Hardcoded line 1. Parse actual line from compiler error. *Accept:* correct line; test.

## Phase D — UI functional gaps

- [ ] **D1 — Device rename API + UI.**
  Backend endpoint missing; UI TODOs at `DevicesPage.tsx:304,328,338,447`; tests
  skipped at `DevicesPage.test.tsx:1292,1381`. Action: add backend rename endpoint,
  wire UI, un-skip tests. *Accept:* rename works end-to-end; tests pass.

- [ ] **D2 — KeyConfigModal mapping editing** (`KeyConfigModal.tsx:200`).
  TODO stub. Implement edit flow. *Accept:* editing works; test.

- [ ] **D3 — metricsStore live layer** (`stores/metricsStore.ts:93`).
  Hardcodes `'Base'`. Source current layer from daemon state (depends on C1).
  *Accept:* shows real active layer; test.

## Phase E — Architecture remediation decision

- [ ] **E1 — Decide fate of architecture-remediation / architecture-completion specs.**
  `architecture-remediation/tasks.md` (52 open, 0 started) and
  `architecture-completion/tasks.md` (30 open) overlap heavily with the COMPLETED
  `comprehensive-architecture-refactoring` spec. Action: audit overlap; for each open
  task, mark it (a) already-done-elsewhere, (b) genuinely-still-needed, or (c) obsolete.
  Write the verdict into those tasks.md files. If genuinely-needed work remains, add
  concrete slices to Phase F below. *Accept:* clear documented verdict; no ambiguity.

## Phase F — Genuinely-needed architecture work (populated by E1, if any)

- [ ] *(empty until E1 determines what, if anything, is real)*

---

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
