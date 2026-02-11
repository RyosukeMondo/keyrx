# Task 4: Update Status API Endpoint - Implementation Plan

## Status: BLOCKED - Waiting for Dependencies

### Dependencies Required
- ❌ **Task 1**: DaemonSharedState struct must be created in `keyrx_daemon/src/daemon/shared_state.rs`
- ❌ **Task 2**: AppState must be updated with `daemon_state: Option<Arc<DaemonSharedState>>` field

### Current State
- Checked `keyrx_daemon/src/daemon/shared_state.rs` - **does not exist**
- Checked `keyrx_daemon/src/web/mod.rs` - **AppState does not have daemon_state field**
- Current `get_status()` function only uses IPC (no shared state support)

### Implementation Ready

Once dependencies are met, the following changes need to be made to `keyrx_daemon/src/web/api/metrics.rs`:

## Changes to `get_status()` Function

### Location
File: `keyrx_daemon/src/web/api/metrics.rs`
Function: `async fn get_status()` (lines 102-169)

### Required Modifications

1. **Add import for DaemonSharedState** (at top of file):
```rust
use crate::daemon::shared_state::DaemonSharedState;
```

2. **Replace the function body** to check for shared state first:

```rust
async fn get_status(
    State(state): State<Arc<crate::web::AppState>>,
) -> Result<Json<StatusResponse>, DaemonError> {
    // Dual-mode operation: prefer shared state (Windows single-process),
    // fallback to IPC (Linux or cross-process)

    let (daemon_running, uptime_secs, active_profile, device_count) =
        if let Some(daemon_state) = &state.daemon_state {
            // Mode 1: Shared state (Windows single-process mode)
            // Direct access to daemon state via Arc<RwLock>, no IPC overhead
            (
                daemon_state.is_running(),
                Some(daemon_state.uptime_secs()),
                daemon_state.get_active_profile(),
                Some(daemon_state.get_device_count()),
            )
        } else if let Some(socket_path) = &state.test_mode_socket {
            // Mode 2: Test mode IPC with timeout
            use crate::ipc::{unix_socket::UnixSocketIpc, DaemonIpc, IpcRequest, IpcResponse};
            use std::time::Duration;

            let socket_path = socket_path.clone();
            let result = tokio::time::timeout(Duration::from_secs(5), async move {
                tokio::task::spawn_blocking(move || {
                    let mut ipc = UnixSocketIpc::new(socket_path);
                    ipc.send_request(&IpcRequest::GetStatus)
                })
                .await
            })
            .await;

            match result {
                Ok(Ok(Ok(IpcResponse::Status {
                    running,
                    uptime_secs: uptime,
                    active_profile: profile,
                    device_count: count,
                }))) => (running, Some(uptime), profile, Some(count)),
                Ok(Ok(Err(e))) => {
                    log::warn!("IPC error querying daemon status: {}", e);
                    (false, None, None, None)
                }
                Ok(Err(e)) => {
                    log::warn!("Failed to join IPC task: {}", e);
                    (false, None, None, None)
                }
                Err(_) => {
                    log::warn!("IPC timeout querying daemon status");
                    (false, None, None, None)
                }
                _ => (false, None, None, None),
            }
        } else {
            // Mode 3: Production IPC (Linux or cross-process mode)
            match query_daemon_status() {
                Ok((uptime, profile, count)) => (true, Some(uptime), profile, Some(count)),
                Err(_) => (false, None, None, None),
            }
        };

    Ok(Json(StatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_running,
        uptime_secs,
        active_profile,
        device_count,
    }))
}
```

### Key Features of the Implementation

1. **Triple-mode operation**:
   - **Shared State**: Direct memory access (Windows single-process)
   - **Test Mode IPC**: With timeout for testing
   - **Production IPC**: Fallback for Linux or cross-process

2. **Ordering matters**: Checks shared state first for best performance

3. **Thread-safe**: All shared state access is through Arc<RwLock>

4. **Documentation**: Clear comments explain each mode

5. **Backward compatible**: Existing IPC code paths still work

### Testing Strategy

Once implemented, test with:

```powershell
# 1. Start daemon
.\target\release\keyrx_daemon.exe run

# 2. Query status (should show daemon_running: true)
curl http://localhost:9867/api/status

# Expected response:
{
  "status": "running",
  "version": "0.1.5",
  "daemon_running": true,
  "uptime_secs": 123,
  "active_profile": "default",
  "device_count": 2
}
```

## Function Documentation Update

Add this documentation to the function:

```rust
/// GET /api/status - Daemon status with dual-mode operation
///
/// This endpoint supports multiple operation modes:
/// 1. **Shared State Mode** (Windows single-process): Direct memory access via Arc<RwLock>
/// 2. **Test Mode**: IPC with timeout for integration testing
/// 3. **Production IPC** (Linux): Cross-process communication via Unix sockets
///
/// The endpoint checks for shared state first (most efficient), then falls back to IPC
/// if shared state is not available. This allows the same code to work on both
/// Windows (single-process) and Linux (daemon + web server as separate processes).
///
/// # Returns
///
/// JSON response with daemon status including:
/// - `daemon_running`: Whether the daemon event loop is active
/// - `uptime_secs`: Seconds since daemon started (if available)
/// - `active_profile`: Name of currently active profile (if any)
/// - `device_count`: Number of managed keyboard devices
```

## Next Steps

1. Wait for Task 1 to create `DaemonSharedState` with these methods:
   - `is_running() -> bool`
   - `uptime_secs() -> u64`
   - `get_active_profile() -> Option<String>`
   - `get_device_count() -> usize`

2. Wait for Task 2 to update `AppState` with:
   - `daemon_state: Option<Arc<DaemonSharedState>>` field

3. Once dependencies are met, apply the changes above

4. Run tests to verify all code paths work

## Estimated Time
- Implementation: 15 minutes (once dependencies are ready)
- Testing: 15 minutes
- Total: 30 minutes

## References
- Spec: `.spec-workflow/specs/windows-ipc-fix/spec.md`
- Current implementation: `keyrx_daemon/src/web/api/metrics.rs` lines 102-169
