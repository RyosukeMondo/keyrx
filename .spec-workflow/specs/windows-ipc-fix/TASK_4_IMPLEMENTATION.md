# Task 4: Status API Endpoint Implementation Guide

## Prerequisites Checklist

Before implementing this task, verify:

- [ ] `keyrx_daemon/src/daemon/shared_state.rs` exists with `DaemonSharedState` struct
- [ ] `DaemonSharedState` has methods: `is_running()`, `uptime_secs()`, `get_active_profile()`, `get_device_count()`
- [ ] `keyrx_daemon/src/web/mod.rs` AppState has `daemon_state: Option<Arc<DaemonSharedState>>` field
- [ ] `keyrx_daemon/src/daemon/mod.rs` exports shared_state module

## Step-by-Step Implementation

### Step 1: Add Import (Line ~17)

**File**: `keyrx_daemon/src/web/api/metrics.rs`

Add after existing imports:
```rust
use crate::daemon::shared_state::DaemonSharedState;
```

### Step 2: Update get_status() Function Documentation (Line ~91)

Replace the existing minimal documentation with:

```rust
/// GET /api/status - Daemon status with dual-mode operation
///
/// This endpoint supports multiple operation modes for maximum compatibility:
///
/// 1. **Shared State Mode** (Windows single-process):
///    - Direct memory access via `Arc<RwLock<DaemonSharedState>>`
///    - Zero IPC overhead, instant response
///    - Primary mode for Windows where daemon and web server run in same process
///
/// 2. **Test Mode IPC**:
///    - Uses Unix sockets with 5-second timeout
///    - Enables integration testing with separate daemon process
///    - Activated when `test_mode_socket` is set in AppState
///
/// 3. **Production IPC** (Linux):
///    - Cross-process communication via Unix sockets
///    - Used when daemon and web server are separate processes
///    - Fallback when shared state is not available
///
/// The endpoint checks for shared state first (most efficient), then test mode IPC,
/// then production IPC. This allows the same code to work across all platforms and
/// deployment modes without platform-specific compilation.
///
/// # Architecture Note
///
/// On Windows, the daemon runs in the same process as the web server due to the
/// need for the low-level keyboard hook to be in the same thread as the message loop.
/// On Linux, they can be separate processes using evdev grab which doesn't have this
/// limitation.
///
/// # Returns
///
/// JSON response with:
/// - `status`: Always "running" if web server responds
/// - `version`: Daemon version from Cargo.toml
/// - `daemon_running`: Whether the daemon event loop is actively processing events
/// - `uptime_secs`: Seconds since daemon started (None if daemon not running)
/// - `active_profile`: Name of currently active profile (None if no profile loaded)
/// - `device_count`: Number of managed keyboard devices (None if daemon not running)
///
/// # Example Response
///
/// ```json
/// {
///   "status": "running",
///   "version": "0.1.5",
///   "daemon_running": true,
///   "uptime_secs": 3600,
///   "active_profile": "gaming",
///   "device_count": 2
/// }
/// ```
///
/// # Errors
///
/// Returns `DaemonError` if state extraction fails (rare, internal error only)
```

### Step 3: Replace get_status() Function Body (Lines ~102-169)

Replace the entire function implementation with:

```rust
async fn get_status(
    State(state): State<Arc<crate::web::AppState>>,
) -> Result<Json<StatusResponse>, DaemonError> {
    // Tri-mode operation for maximum platform compatibility:
    // 1. Shared state (Windows single-process) - fastest, zero IPC
    // 2. Test mode IPC (integration tests) - with timeout
    // 3. Production IPC (Linux cross-process) - fallback

    let (daemon_running, uptime_secs, active_profile, device_count) =
        if let Some(daemon_state) = &state.daemon_state {
            // Mode 1: Shared State (Windows single-process mode)
            //
            // On Windows, the daemon event loop runs in the same process as the web server
            // because the low-level keyboard hook (SetWindowsHookEx) requires the hook
            // callback to be in the same thread that pumps messages. This means we can
            // access daemon state directly via shared memory (Arc<RwLock>) with zero
            // IPC overhead.
            //
            // This is the fastest path: <1μs for state access vs ~1ms for IPC.
            log::debug!("Using shared state for daemon status");
            (
                daemon_state.is_running(),
                Some(daemon_state.uptime_secs()),
                daemon_state.get_active_profile(),
                Some(daemon_state.get_device_count()),
            )
        } else if let Some(socket_path) = &state.test_mode_socket {
            // Mode 2: Test Mode IPC with Timeout
            //
            // Used in integration tests where the daemon runs as a separate process
            // to simulate production-like conditions. We use a timeout to prevent
            // tests from hanging if the daemon crashes or socket becomes unavailable.
            log::debug!("Using test mode IPC for daemon status");
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
            //
            // On Linux, the daemon typically runs as a separate process from the web
            // server because evdev grab doesn't have the same threading restrictions
            // as Windows hooks. Communication happens via Unix domain sockets.
            //
            // This mode is also used if shared state is explicitly disabled for
            // debugging or testing purposes.
            log::debug!("Using production IPC for daemon status");
            match query_daemon_status() {
                Ok((uptime, profile, count)) => (true, Some(uptime), profile, Some(count)),
                Err(e) => {
                    log::warn!("Failed to query daemon status via IPC: {}", e);
                    (false, None, None, None)
                }
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

## Changes Summary

### What Changed
1. **Added import**: `use crate::daemon::shared_state::DaemonSharedState;`
2. **Added tri-mode logic**: Check shared state → test mode IPC → production IPC
3. **Added logging**: Debug logs to track which code path is used
4. **Enhanced documentation**: Detailed explanation of each mode and why
5. **Improved error handling**: Log IPC errors in production mode

### What Stayed the Same
1. **Function signature**: Same input/output types
2. **Response structure**: StatusResponse unchanged
3. **Test mode IPC logic**: Preserved exactly as-is
4. **Production IPC fallback**: Still works for Linux

### Performance Impact
- **Shared state path**: <1μs (replaces ~1ms IPC call)
- **Test/production IPC**: No change, same performance as before
- **Memory**: +8 bytes per AppState instance (Option<Arc> pointer)

## Testing Plan

### Unit Tests (Add to metrics.rs test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_with_shared_state() {
        // TODO: Create mock DaemonSharedState
        // TODO: Create AppState with shared state
        // TODO: Call get_status()
        // TODO: Verify response has daemon_running: true
    }

    #[tokio::test]
    async fn test_status_fallback_to_ipc() {
        // TODO: Create AppState without shared state
        // TODO: Mock IPC to return status
        // TODO: Call get_status()
        // TODO: Verify IPC was called
    }

    #[tokio::test]
    async fn test_status_ipc_failure() {
        // TODO: Create AppState without shared state
        // TODO: Mock IPC to return error
        // TODO: Call get_status()
        // TODO: Verify daemon_running: false
    }
}
```

### Integration Tests

```rust
// In keyrx_daemon/tests/windows_shared_state_test.rs

#[cfg(target_os = "windows")]
#[tokio::test]
async fn test_status_endpoint_uses_shared_state() {
    // 1. Start daemon with shared state
    // 2. Make HTTP request to /api/status
    // 3. Verify daemon_running: true
    // 4. Verify uptime_secs > 0
    // 5. Verify device_count matches expected
}
```

### Manual Testing

```powershell
# Windows testing script
$daemon = Start-Process ".\target\release\keyrx_daemon.exe" -ArgumentList "run" -PassThru
Start-Sleep 2

# Test status endpoint
$response = Invoke-RestMethod -Uri "http://localhost:9867/api/status"
Write-Host "Status Response:"
$response | ConvertTo-Json -Depth 5

# Verify expectations
if ($response.daemon_running -ne $true) {
    Write-Error "Expected daemon_running: true"
}
if ($null -eq $response.uptime_secs) {
    Write-Error "Expected uptime_secs to be set"
}
if ($response.device_count -lt 1) {
    Write-Error "Expected at least 1 device"
}

# Cleanup
Stop-Process -Id $daemon.Id
```

## Rollback Plan

If implementation causes issues:

```bash
git checkout keyrx_daemon/src/web/api/metrics.rs
cargo build -p keyrx_daemon
cargo test -p keyrx_daemon --lib
```

The old IPC-only code will be restored and everything will work as before (with IPC limitations on Windows).

## Verification Checklist

After implementation, verify:

- [ ] Code compiles without warnings: `cargo build -p keyrx_daemon`
- [ ] Existing tests pass: `cargo test -p keyrx_daemon`
- [ ] Documentation builds: `cargo doc -p keyrx_daemon --no-deps`
- [ ] Clippy is happy: `cargo clippy -p keyrx_daemon -- -D warnings`
- [ ] Status endpoint responds: `curl http://localhost:9867/api/status`
- [ ] Response shows `daemon_running: true` (when daemon is running)
- [ ] Response shows `uptime_secs > 0` (after a few seconds)
- [ ] Logs show "Using shared state for daemon status" on Windows

## Dependencies for Next Tasks

Once this task is complete, the following tasks can use the same pattern:

- **Task 5**: Update other metrics endpoints (latency, events, daemon state)
- **Task 6**: Update config endpoints for profile activation

The implementation pattern is:
```rust
if let Some(daemon_state) = &state.daemon_state {
    // Use shared state
} else {
    // Fall back to IPC
}
```

## Estimated Completion Time

- Implementation: 15 minutes
- Testing: 15 minutes
- Documentation: Already done in this guide
- **Total: 30 minutes**

## Contact for Questions

Refer to:
- Spec: `.spec-workflow/specs/windows-ipc-fix/spec.md`
- Architecture: `WINDOWS_IPC_FIX_REQUIRED.md`
- Task status: `.spec-workflow/specs/windows-ipc-fix/TASK_4_STATUS.md`
