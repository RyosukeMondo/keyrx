# Request G3 — Windows production IPC server

**For:** the Windows session (`D:\users\ryosu\repos\keyrx`)
**From:** Linux session, 2026-09-27. Plan: `.spec-workflow/RESUME_PLAN.md` → G3
**Deliverable:** commits on `main`, plus a filled-in **Result** section at the end
of this file. Don't push unless the user says to.

## 0. Code is already synced

The Linux session fast-forwarded this checkout to its `main` over ssh
(`git log --oneline -3` should show this request's commit). Future Linux
commits arrive the same way: the Linux side has a `windows` remote, and this
repo has `receive.denyCurrentBranch=updateInstead`. Keep the working tree clean
between tasks, or the push is refused.

Your uncommitted CLAUDE.md edits (a slimming pass over `.claude/CLAUDE.md`,
`CLAUDE.md` and `scripts/CLAUDE.md`) conflicted with the updated
`.claude/CLAUDE.md`, so they were NOT merged. They are in
`stash@{0}` ("pre-linux-sync WIP: CLAUDE.md slimming"). Re-apply them by hand
if still wanted, and keep the Linux Host Notes lines added since then.

Read `.claude/CLAUDE.md` → "Linux Host Notes" (the read-model and live-config
rules apply to Windows too) and RESUME_PLAN Phases L and G.

## 1. Problem

`keyrx_daemon status | state | metrics latency | metrics events` and
`profiles activate` (which notifies a running daemon) talk to the daemon over
IPC. On Windows:

1. **No server.** Only `platform_runners/linux.rs` calls
   `start_production_ipc_server`; `platform_runners/windows.rs` never does.
2. **The client can't find a named pipe.** `ipc::DEFAULT_SOCKET_PATH` is
   `/tmp/keyrx-daemon.sock`, and `UnixSocketIpc::connect` first checks
   `socket_path.exists()`. A named pipe is not a file, so on Windows the check
   always fails and the CLI always reports "daemon not running".
3. **Server setup is Unix-only.** `IpcServer::start` removes a stale socket file
   and chmods it to 0600. Neither step applies to a pipe.

The handler side is already cross-platform and tested. `IpcCommandHandler` over
the single `DaemonQueryService` is the same read model REST, MCP and WS use.
Only the transport endpoint and the runner wiring are missing.

## 2. Required design (one mechanism, not a Windows copy)

- **One endpoint type.** Replace the `DEFAULT_SOCKET_PATH` string with a small
  type in `ipc/mod.rs`, e.g. `IpcEndpoint`, owning the platform difference:
  - Unix: `/tmp/keyrx-daemon.sock` (a file: remove it if stale, chmod 0600,
    remove it on shutdown)
  - Windows: a named pipe via the `interprocess` 1.2 namespaced name
    (`@keyrx-daemon` → `\\.\pipe\keyrx-daemon`). No file operations.
  - `IpcEndpoint::default_for_platform()` is used by the server spawn, every
    CLI command and `cli/profiles.rs::notify_running_daemon`. Tests pass
    explicit endpoints; the existing Windows test already uses the
    `@Local\keyrx-test-<ts>` style.
- **"Not running" detection.**
  - Unix keeps the current behaviour: file missing = `SocketNotFound`, and a
    file that refuses connections = `StaleSocket`.
  - On Windows, a connect error of `NotFound` (pipe absent) must map to
    `SocketNotFound`. There is no stale case.
  - Keep the CLI message identical on both platforms: "daemon not running".
- **One spawn helper for both runners.** Move `start_production_ipc_server`
  and `remove_ipc_socket` out of `platform_runners/linux.rs` into a shared
  place (e.g. `platform_runners/mod.rs` or `ipc/server.rs`). Call the helper
  from `windows.rs` right after `daemon_query` is built, with the same
  `daemon_query` the web `AppState` uses. Do not build a second one. Remove the
  endpoint on exit, which is a no-op for pipes.
- **Rename** `UnixSocketIpc` → `IpcClient` and `ipc/unix_socket.rs` →
  `ipc/client.rs`, since it is not Unix-specific any more. Update the module
  docs, which still say "Unix socket".
- **Single instance.** If another daemon already owns the pipe, log a warning
  and keep running without IPC, the same best-effort policy Linux has. Do not
  kill the other instance from here; `ensure_single_instance` already exists.
- **Pipe security.** The default named-pipe DACL gives Everyone read-only
  access, so other users can't send requests. That's acceptable. Note it in a
  comment, and don't widen it.
- **Don't touch** `web/api/config.rs::query_active_profile`. It calls the
  daemon over IPC from inside the daemon, which violates the one-read-model
  rule, and the Linux session is removing it (G6).

## 2b. While in `windows.rs`: bring it under the size limit (G5)

`daemon/platform_runners/windows.rs` has 537 code lines against a 500 limit.
`scripts/verify/file-sizes.sh` is a real gate now, with a shrink-only baseline
in `scripts/verify/file-size-baseline.list`. Moving the IPC helper out already
helps. Split by responsibility (message loop, tray handling, single-instance and
PID file, test mode) until the file is under 500. Then run
`bash scripts/verify/file-sizes.sh --update` (from Git Bash) and commit the
shrunk baseline.

## 3. Acceptance

Automated, on Windows:
- `cargo test -p keyrx_daemon ipc::` passes, including a named-pipe round trip
  (server spawn → `IpcClient` GetStatus, GetEventsTail, ClearEvents) and "no
  server → `SocketNotFound`".
- `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` and
  `bash scripts/verify/file-sizes.sh` pass.
- `cargo test --workspace`: report pass/fail/ignored counts, and list any
  failure that isn't in the known-flaky G4 set.

Live, on Windows, with the built daemon (`keyrx_daemon run`, no `--config`,
so it follows the active profile):
1. `keyrx_daemon status` shows Running: Yes, and the same active profile and
   device count as `curl http://127.0.0.1:9867/api/status`.
2. Type a few keys. `keyrx_daemon metrics events` shows them as
   `press X -> Y`, the same events as `GET /api/metrics/events` (a
   `KeyEventData[]`, oldest first). `keyrx_daemon metrics latency` matches the
   numbers from `GET /api/metrics/latency`.
3. `keyrx_daemon profiles activate <other>` prints "Daemon: running daemon
   switched", and a key mapped differently in `<other>` now behaves the new
   way.
4. Stop the daemon. `keyrx_daemon status` must print "daemon not running",
   not an I/O error.

Linux must keep working. Run `cargo clippy -p keyrx_daemon --target
x86_64-unknown-linux-gnu -- -D warnings` if the toolchain is available;
otherwise say so and the Linux session will verify after merging.

## 4. Report back (fill in below)

## Result

- Commits: on branch `windows` (not `main`, at the user's request), one commit
  "feat(ipc): Windows production IPC over a named pipe (G3, G5)". PR open against `main`.
- Tests (pass/fail/ignored): **NOT RUN.** The Windows host ran out of memory
  compiling `keyrx_daemon` (clippy killed mid-build) and no build has finished,
  so the code is uncompiled. `cargo fmt` done; `scripts/verify/file-sizes.sh`
  passes and the baseline is now empty (0 files over 500).
- Live check 1–4: not run (no build; KeyRx not installed on this host).
- Implementation:
  - `ipc/endpoint.rs`: `IpcEndpoint { SocketFile(PathBuf) | NamedPipe(String) }`
    with `default_for_platform()`, `test_for_process(pid)`, `parse/from_cli`
    (CLI `--socket` is a pipe name on Windows). Owns stale-file removal,
    chmod 0600, removal on exit (no-ops for pipes) and connect-error mapping:
    `NotFound` -> `SocketNotFound` on both, `ConnectionRefused` -> `StaleSocket`
    for socket files only. `DEFAULT_SOCKET_PATH` removed.
  - `SocketNotFound` now prints "Daemon not running: no IPC endpoint at ...
    (error code 3005)" on both platforms.
  - `UnixSocketIpc` -> `IpcClient`, `ipc/unix_socket.rs` -> `ipc/client.rs`;
    any failure now drops the connection. Its tests run on Windows too.
  - `platform_runners/mod.rs`: shared `start_production_ipc_server` /
    `remove_production_ipc_endpoint`; both runners call them with the web
    `AppState`'s `daemon_query`. Second instance on the pipe: bind fails,
    warning, daemon runs without IPC.
  - G5: `windows.rs` -> `windows/{mod,message_loop,instance,shell,test_mode}.rs`
    (largest 191 code lines). Test mode uses `IpcEndpoint::test_for_process`.
  - New tests: named-pipe round trip via `server::spawn` (GetStatus,
    GetEventsTail, ClearEvents), no server -> `SocketNotFound`, second server
    on the same pipe fails (Windows), endpoint unit tests.
- Deviations / open issues:
  - Endpoint type lives in `ipc/endpoint.rs`, re-exported from `ipc/mod.rs`.
  - Linux side please: `cargo clippy --workspace -- -D warnings`,
    `cargo clippy -p keyrx_daemon --target x86_64-pc-windows-gnu -- -D warnings`,
    `cargo test -p keyrx_daemon ipc::`. The Windows named-pipe tests and live
    checks still need a Windows run with enough memory (try `-j 2`).
  - `ipc::client` `test_timeout_handling` kept Unix-only: the client has no
    real read timeout (pre-existing).

## Follow-up (2026-10-01): shared-engine changes to verify on Windows

Linux-side round 2 touched code the Windows daemon shares. Nothing here was
run on Windows (the windows-gnu clippy and `--no-run` test build are clean).
When you next sync, please check:

- **New `BaseKeyMapping` variants 7/8/9** in the shared engine (explicit rkyv
  discriminants): 7 = `TapHoldKey` (`tap_hold`/`hold_only` can hold a real key
  such as `VK_LCtrl`), 8 = `OneShot` (`one_shot(key, "VK_LShift")`),
  9 = `TapHoldKeyTimeoutOnly`. The `.krx` is now byte-deterministic
  (`SOURCE_DATE_EPOCH` or 0 for the timestamp). Run
  `cargo test -p keyrx_core` and check the Windows hook still produces correct
  Shift/Ctrl for a home-row mod and a one-shot Shift.
- **Min key-down decorator** (`platform/min_key_down/`, default 5 ms, flag
  `--min-key-down-ms`, env `KEYRX_MIN_KEY_DOWN_MS`, `settings.json`): it is
  applied in `daemon/mod.rs` (`MinKeyDown::new(platform, ..)`) around whatever
  `Platform` the daemon was given. Confirm the Windows runner builds its
  daemon through that path (a tap should keep the key down >= 5 ms; `0`
  disables) and that `SendInput` pacing still behaves.
- **Output device naming:** Linux names its uinput device `keyrx-out-<pid>`;
  `OutputDeviceInfo` (name + path) is now in `status` on IPC and REST. On
  Windows there is no virtual device, so `output_device` must be absent
  (`skip_serializing_if`); check `keyrx_daemon status` prints no output line.
  `DeviceInfo` also gained `is_virtual`/`is_keyrx_output`; REST
  `/api/devices?include_virtual=` filters on them.
- **`config_error`** in status (failed hot reload / broken active profile) is
  set through the shared `DaemonSharedState`; the Windows runner should surface
  it the same way (`status`, `doctor`).
- The one-handed emergency stop (`--emergency-hold-ms`, hold Escape alone) was
  implemented in `platform/linux/emergency_stop.rs` and `platform/emergency.rs`;
  check whether `platform/emergency.rs` is meant to be shared with the Windows
  hook and that the Windows chord behaviour is unchanged.


## Addendum (2026-10-03): web/unit changes that touch Windows

- **Static files** (`web/static_files.rs`): a missing file path (anything with
  an extension, e.g. a stale hashed `assets/*.js` chunk after an upgrade) is a
  real `404`, not `index.html`/200; only extension-less SPA routes fall back to
  `index.html`. The UI reloads once when a lazy chunk is gone. Check the
  Windows-embedded UI behaves the same and the tray's "Open Web UI" still works.
- **Import endpoint**: `POST /api/profiles/import` (`web/api/profiles/`) backs
  the UI's "Load layout from file" (`.krx` or `.rhai`; the new
  profile is not activated). It is platform independent; just confirm a profile imported
  on Windows lands in the Windows profile dir and appears in `profiles list`.
- **systemd unit** (`keyrx_daemon/systemd/keyrx-user.service`) is Linux only:
  `StartLimitIntervalSec=0`, `KillMode=process`, `RestartSteps`/
  `RestartMaxDelaySec`. Nothing to port; the Windows equivalent is the
  scheduled task / service restart policy, which should likewise never give up.
