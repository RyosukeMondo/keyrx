# Tasks: IPC Test Mode for E2E Testing

## Overview

Enable 5 failing IPC-dependent E2E tests by adding test mode with full IPC infrastructure.

**Total Tasks:** 8
**Estimated Time:** 1-2 days

---

- [x] 1. Add --test-mode CLI flag to daemon
  - File: keyrx_daemon/src/cli/run.rs
  - Add `test_mode: bool` field to `RunArgs` struct with #[clap(long)] attribute
  - Add validation to ensure test mode only enabled in debug builds
  - Purpose: Enable test mode via CLI flag for E2E testing
  - _Leverage: keyrx_daemon/src/cli/run.rs (existing RunArgs struct), keyrx_daemon/src/cli/mod.rs (CLI structure)_
  - _Requirements: REQ-3.1.1, REQ-4.3.1, REQ-4.3.3_
  - _Prompt: Role: CLI Developer with expertise in Rust clap library and command-line interfaces | Task: Add --test-mode CLI flag to RunArgs struct following REQ-3.1.1 and REQ-4.3.x, ensuring flag only works in debug builds for security | Restrictions: Must use clap attributes correctly, must validate test mode only in debug builds (#[cfg(debug_assertions)]), must not affect production builds | Success: --test-mode flag accepted in debug builds, rejected in release builds, clear error message if test mode unavailable_

- [ ] 2. Create IPC module structure
  - File: keyrx_daemon/src/ipc/mod.rs (new file)
  - Create new `ipc` module with IpcServer, IpcClient, and IpcCommand types
  - Define IPC message protocol (JSON-based commands and responses)
  - Purpose: Establish IPC infrastructure foundation
  - _Leverage: keyrx_daemon/src/daemon/mod.rs (daemon structure patterns), tokio::net::UnixListener_
  - _Requirements: REQ-3.2.1, REQ-3.2.2_
  - _Prompt: Role: Systems Programmer with expertise in IPC and Rust async programming | Task: Create IPC module with IpcServer, IpcClient, IpcCommand types following REQ-3.2.x, using Unix sockets and JSON protocol | Restrictions: Must use tokio UnixListener/UnixStream, must handle socket cleanup, must support async command handling | Success: IPC module compiles, socket creates successfully, clean shutdown removes socket file_

- [ ] 3. Implement IPC server for test mode
  - File: keyrx_daemon/src/ipc/server.rs (new file)
  - Implement IpcServer::new() to create Unix socket at /tmp/keyrx-test-{pid}.sock
  - Implement connection handling loop that accepts clients and spawns handlers
  - Purpose: Accept and process IPC commands from REST API handlers
  - _Leverage: tokio::net::UnixListener, keyrx_daemon/src/ipc/mod.rs (IPC types)_
  - _Requirements: REQ-3.2.1, REQ-3.2.2, REQ-4.1.2, REQ-4.3.2_
  - _Prompt: Role: Backend Developer with expertise in Rust tokio and Unix sockets | Task: Implement IPC server with Unix socket creation and connection handling following REQ-3.2.x and REQ-4.x, spawning async handlers for each connection | Restrictions: Must use unique socket path per daemon instance, must set socket permissions to 600, must cleanup socket on shutdown | Success: IPC server accepts connections, handles multiple clients concurrently, cleans up socket file on exit_

- [ ] 4. Implement profile activation via IPC
  - File: keyrx_daemon/src/ipc/commands.rs (new file)
  - Add IpcCommand::ActivateProfile variant with profile name
  - Implement command handler that activates profile via ProfileManager
  - Return success/failure response with profile name
  - Purpose: Enable profile activation through IPC in test mode
  - _Leverage: keyrx_daemon/src/config/profile_manager.rs (ProfileManager), keyrx_daemon/src/ipc/mod.rs_
  - _Requirements: REQ-3.2.3, REQ-3.3.1, REQ-4.1.1_
  - _Prompt: Role: Backend Developer with expertise in Rust async and state management | Task: Implement profile activation IPC command handler following REQ-3.2.3 and REQ-3.3.1, using ProfileManager to activate profiles with < 50ms latency | Restrictions: Must validate profile exists, must handle activation errors gracefully, must return clear success/failure response | Success: Profile activation works via IPC, response includes profile name, latency < 50ms_

- [ ] 5. Implement daemon status query via IPC
  - File: keyrx_daemon/src/ipc/commands.rs
  - Add IpcCommand::GetStatus variant
  - Implement command handler that queries daemon running state
  - Return status response with daemon_running field
  - Purpose: Enable daemon status queries through IPC in test mode
  - _Leverage: keyrx_daemon/src/daemon/mod.rs (daemon state), keyrx_daemon/src/ipc/mod.rs_
  - _Requirements: REQ-3.2.4, REQ-3.3.2, REQ-4.1.1_
  - _Prompt: Role: Backend Developer with expertise in Rust state management and IPC | Task: Implement daemon status IPC command handler following REQ-3.2.4 and REQ-3.3.2, querying daemon running state with < 50ms latency | Restrictions: Must return accurate daemon state, must handle concurrent queries, must include all required status fields | Success: Status query works via IPC, daemon_running field populated correctly, latency < 50ms_

- [ ] 6. Integrate IPC with REST API handlers
  - File: keyrx_daemon/src/web/api/profiles.rs, keyrx_daemon/src/web/api/metrics.rs
  - Update activate_profile handler to use IPC in test mode
  - Update get_status handler to query IPC for daemon_running field in test mode
  - Add IPC timeout handling (5 seconds)
  - Purpose: Connect REST API to IPC infrastructure in test mode
  - _Leverage: keyrx_daemon/src/ipc/mod.rs (IpcClient), keyrx_daemon/src/web/api/ (existing handlers)_
  - _Requirements: REQ-3.3.1, REQ-3.3.2, REQ-3.3.3, REQ-3.3.4_
  - _Prompt: Role: API Developer with expertise in Rust axum and async integration | Task: Update REST API handlers to use IPC in test mode following REQ-3.3.x, adding IPC client calls with timeout and error handling | Restrictions: Must check test mode flag, must handle IPC timeouts gracefully, must maintain production mode behavior, must provide clear error messages | Success: REST API calls use IPC in test mode, timeouts after 5 seconds, clear error messages, production mode unaffected_

- [ ] 7. Update daemon startup for test mode
  - File: keyrx_daemon/src/cli/run.rs, keyrx_daemon/src/daemon/mod.rs
  - Start IPC server when --test-mode flag provided
  - Skip keyboard capture initialization in test mode
  - Pass IPC client to AppState for REST API handlers
  - Purpose: Initialize daemon with IPC infrastructure in test mode
  - _Leverage: keyrx_daemon/src/ipc/server.rs (IpcServer), keyrx_daemon/src/daemon/mod.rs (daemon initialization)_
  - _Requirements: REQ-3.1.2, REQ-3.1.3, REQ-3.1.4, REQ-4.1.2_
  - _Prompt: Role: Systems Engineer with expertise in application initialization and Rust tokio | Task: Update daemon startup to initialize IPC server in test mode following REQ-3.1.x and REQ-4.1.2, skipping keyboard capture, startup < 2 seconds | Restrictions: Must start IPC server before REST API, must skip platform layer init in test mode, must cleanup IPC on shutdown | Success: Daemon starts with IPC in test mode, startup < 2 seconds, IPC server ready before API endpoints, clean shutdown_

- [ ] 8. Test IPC-dependent E2E tests
  - File: Run tests with --test-mode: scripts/automated-e2e-test.ts
  - Update test runner to pass --test-mode flag to daemon
  - Run all 5 IPC-dependent tests (status-001, integration-001, workflow-002, workflow-003, workflow-007)
  - Verify all tests pass with test mode enabled
  - Purpose: Validate IPC test mode fixes all failing tests
  - _Leverage: scripts/automated-e2e-test.ts, scripts/fixtures/daemon-fixture.ts_
  - _Requirements: All requirements_
  - _Prompt: Role: QA Engineer with expertise in E2E testing and test automation | Task: Execute all IPC-dependent tests with test mode enabled, verify all tests pass following all requirements, measuring IPC latency and reliability | Restrictions: Must run each test 10 times to verify no flakiness, must measure IPC latency, must verify daemon_running field correct | Success: All 5 IPC-dependent tests pass, zero flaky failures, IPC latency < 50ms, 100% E2E test pass rate (83/83)_

---

## Task Dependencies

```
1 (CLI flag) ──→ 2 (IPC module) ──→ 3 (IPC server)
                                      ↓
                                4 (Profile activation) ──┐
                                      ↓                   ├──→ 6 (REST API integration)
                                5 (Status query) ────────┘      ↓
                                                                 7 (Daemon startup)
                                                                 ↓
                                                                 8 (E2E tests)
```

## Success Criteria

- ✅ All 8 tasks complete
- ✅ All 5 IPC-dependent tests pass with --test-mode
- ✅ 100% E2E test pass rate (83/83)
- ✅ IPC latency < 50ms
- ✅ Test mode startup < 2 seconds
- ✅ Zero flaky test failures (10 consecutive runs)

## Verification Checklist

- [ ] Run `cargo build --release` - compiles without errors
- [ ] Run `target/release/keyrx_daemon run --test-mode` - starts successfully
- [ ] Check `/tmp/keyrx-test-*.sock` exists while daemon running
- [ ] Run `npx tsx scripts/automated-e2e-test.ts --daemon-path target/release/keyrx_daemon --test-mode` - all 83 tests pass
- [ ] Run tests 10 consecutive times - zero failures
- [ ] Measure IPC latency - < 50ms
- [ ] Verify socket cleanup - no orphaned socket files after shutdown
- [ ] Test release build - --test-mode flag rejected

## Notes

- **Estimated Time:** 1-2 days (moderate complexity, new IPC infrastructure)
- **Priority:** High (fixes 5 failing tests, improves test coverage to 100%)
- **Risk:** Medium (new IPC code, needs thorough testing)
- **Alternative:** Could skip IPC-dependent tests, but loses coverage
