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

## 3. Acceptance

Automated, on Windows:
- `cargo test -p keyrx_daemon ipc::` passes, including a named-pipe round trip
  (server spawn → `IpcClient` GetStatus, GetEventsTail, ClearEvents) and "no
  server → `SocketNotFound`".
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
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

- Commits:
- Tests (pass/fail/ignored):
- Live check 1–4:
- Deviations / open issues:
