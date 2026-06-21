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
3. **Verify** with the task's acceptance check (scoped tests/clippy on the touched
   crate only — e.g. `cargo test -p keyrx_daemon <module>`, `cargo clippy -p keyrx_daemon`).
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

- [NEXT] **C1 — Implement IPC `GetState`** (`keyrx_daemon/src/ipc/commands.rs:51`).
  Returns "not implemented yet". Wire it to the daemon's real state broadcast
  (see commit d8694064 "daemon state broadcasting"). *Accept:* returns real state;
  scoped test added; `cargo test -p keyrx_daemon ipc`.

- [ ] **C2 — Implement IPC `GetLatencyMetrics`** (`commands.rs:58`).
  Wire to the metrics source used by the web metrics API. *Accept:* real metrics; test.

- [ ] **C3 — Implement IPC `GetEventsTail`** (`commands.rs:65`).
  Return the recent event ring/tail. *Accept:* returns recent events; test.

- [ ] **C4 — Implement metrics follow/tail mode** (`src/cli/metrics.rs:40,69`).
  Currently `Err("not implemented")`. *Accept:* follow mode streams; test or manual note.

- [ ] **C5 — Implement metrics web endpoint stub** (`src/web/api/metrics.rs:300`).
  Returns "not implemented" response. *Accept:* returns real payload; test.

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
