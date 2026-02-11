# Windows IPC Fix - Shared State Implementation

## Overview
Replace broken Unix socket IPC with shared state architecture for Windows single-process daemon.

## Goal
Enable daemon-to-web-server communication on Windows by sharing state via Arc<RwLock> instead of IPC.

## Success Criteria
- [ ] DaemonSharedState struct created and documented
- [ ] AppState updated to include optional daemon_state
- [ ] Windows runner wires daemon state into AppState
- [ ] Status API endpoint uses shared state (shows daemon_running: true)
- [ ] Metrics API endpoint uses shared state (shows events)
- [ ] Profile activation triggers daemon reload
- [ ] All existing tests pass
- [ ] Key remapping works on Windows

## Architecture

```rust
// Shared between main thread (daemon) and web server thread
pub struct DaemonSharedState {
    running: Arc<AtomicBool>,           // Already exists in Daemon
    active_profile: Arc<RwLock<Option<String>>>,
    config_path: Arc<RwLock<PathBuf>>,
    device_count: Arc<AtomicUsize>,
    start_time: Instant,
}
```

## Tasks

### Task 1: Create DaemonSharedState (CRITICAL PATH)
**File**: `keyrx_daemon/src/daemon/shared_state.rs`
**Agent**: Backend specialist
**Dependencies**: None
**Estimated**: 30 min

Create new module with:
- DaemonSharedState struct with all fields
- Constructor `new()` that takes Daemon references
- Getter methods for each field
- Thread-safe access patterns
- Full documentation with examples
- Unit tests

### Task 2: Update AppState (CRITICAL PATH)
**File**: `keyrx_daemon/src/web/mod.rs`
**Agent**: API specialist
**Dependencies**: Task 1
**Estimated**: 20 min

Modify AppState:
- Add `daemon_state: Option<Arc<DaemonSharedState>>`
- Update `new()` to accept optional daemon_state
- Update `from_container()` to accept optional daemon_state
- Add helper method `has_daemon_state()`
- Update tests to handle new field
- Update all test fixtures

### Task 3: Wire Shared State in Windows Runner (CRITICAL PATH)
**File**: `keyrx_daemon/src/daemon/platform_runners/windows.rs`
**Agent**: Platform specialist
**Dependencies**: Task 1, Task 2
**Estimated**: 40 min

Modify windows.rs run_daemon():
- After creating Daemon, create DaemonSharedState
- Extract profile name from config_path
- Pass daemon_state to AppState creation
- Update test mode to also use shared state
- Add comments explaining the architecture
- Test compilation on Windows

### Task 4: Update Status API Endpoint (PARALLEL)
**File**: `keyrx_daemon/src/web/api/metrics.rs`
**Agent**: API specialist
**Dependencies**: Task 2
**Estimated**: 30 min

Modify get_status():
- Check if daemon_state exists in AppState
- If yes, read directly from shared state
- If no, fall back to IPC (for Linux)
- Add #[cfg(target_os = "windows")] hints
- Update function documentation
- Test both code paths

### Task 5: Update Metrics API Endpoints (PARALLEL)
**File**: `keyrx_daemon/src/web/api/metrics.rs`
**Agent**: API specialist
**Dependencies**: Task 2
**Estimated**: 30 min

Update these endpoints:
- get_latency_stats() - use shared state
- get_event_log() - use shared state
- get_daemon_state() - use shared state
- query_daemon_status() - add fallback logic
Update all to prefer shared state over IPC

### Task 6: Update Config API Endpoints (PARALLEL)
**File**: `keyrx_daemon/src/web/api/config.rs`
**Agent**: API specialist
**Dependencies**: Task 2
**Estimated**: 20 min

Update endpoints that query daemon:
- Profile activation should trigger reload
- Add shared state checks
- Fallback to IPC if not available

### Task 7: Add Profile Reload Trigger (CRITICAL PATH)
**File**: `keyrx_daemon/src/daemon/mod.rs`
**Agent**: Backend specialist
**Dependencies**: Task 1
**Estimated**: 30 min

Add to Daemon:
- `reload_from_shared_state()` method
- Hook into shared state profile changes
- Signal handler for reload trigger
- Test reload functionality

### Task 8: Integration Testing (FINAL)
**Agent**: Test specialist
**Dependencies**: All above
**Estimated**: 45 min

Test scenarios:
- Start daemon, verify status shows running
- Activate profile, verify daemon reloads
- Press keys, verify remapping works
- Check metrics show events
- Verify web UI displays correct status
- Run full test suite
- Document any issues found

## Files to Modify

1. `keyrx_daemon/src/daemon/mod.rs` - Add shared_state module
2. `keyrx_daemon/src/daemon/shared_state.rs` - NEW file
3. `keyrx_daemon/src/web/mod.rs` - Update AppState
4. `keyrx_daemon/src/daemon/platform_runners/windows.rs` - Wire shared state
5. `keyrx_daemon/src/web/api/metrics.rs` - Use shared state
6. `keyrx_daemon/src/web/api/config.rs` - Use shared state

## Critical Path
Task 1 → Task 2 → Task 3 → Task 7 → Task 8

## Parallel Execution
Tasks 4, 5, 6 can run in parallel after Task 2 completes

## Testing Strategy

### Unit Tests
- DaemonSharedState creation and access
- AppState with/without daemon_state
- Each API endpoint fallback logic

### Integration Tests
- Full daemon startup with shared state
- Profile activation → reload cycle
- Event metrics collection
- Cross-thread state access

### Manual Testing
1. Build: `cargo build --release -p keyrx_daemon`
2. Kill old daemon: `taskkill /F /IM keyrx_daemon.exe`
3. Start new: `.\target\release\keyrx_daemon.exe run`
4. Test status: `curl http://localhost:9867/api/status`
5. Activate profile: `curl -X POST http://localhost:9867/api/profiles/default/activate`
6. Press remapped keys
7. Check metrics: `curl http://localhost:9867/api/metrics/events`

## Rollback Plan

If implementation fails:
1. Revert all changes: `git checkout .`
2. Daemon still works in degraded mode (web server only)
3. Document issues in WINDOWS_IPC_FIX_REQUIRED.md
4. Consider alternative: Named Pipes implementation

## Acceptance Criteria

✅ Code compiles without warnings
✅ All existing tests pass
✅ Status API shows `daemon_running: true`
✅ Metrics API shows keyboard events
✅ Profile activation reloads daemon
✅ Key remapping works
✅ Web UI displays correct daemon status
✅ Documentation updated

## Estimated Total Time
Sequential: ~4.5 hours
Parallel (5 agents): ~1.5 hours

## Dependencies
- Rust 1.70+
- Windows 10+ (target platform)
- Running daemon for testing

## References
- `WINDOWS_IPC_FIX_REQUIRED.md` - Architecture details
- `keyrx_daemon/src/daemon/mod.rs` - Daemon implementation
- `keyrx_daemon/src/web/mod.rs` - AppState implementation
- `keyrx_daemon/src/platform/windows/` - Windows platform code
