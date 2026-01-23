# Design: IPC Test Mode for E2E Testing

## 1. Architecture

### Current Architecture (Production)
```
keyrx_daemon (run mode)
  ├─ Web Server (REST API + WebSocket)
  ├─ Platform Layer (evdev/Windows hooks)
  ├─ Config Manager
  └─ Profile Manager
```

### New Architecture (Test Mode)
```
keyrx_daemon --test-mode
  ├─ Web Server (REST API + WebSocket)
  ├─ IPC Socket (new)
  │   ├─ Profile activation commands
  │   └─ Daemon status queries
  ├─ Platform Layer (stub - no keyboard capture)
  ├─ Config Manager
  └─ Profile Manager (IPC-connected)
```

## 2. Implementation Design

### 2.1 CLI Flag Addition

```rust
// keyrx_daemon/src/cli/run.rs
#[derive(Parser)]
pub struct RunArgs {
    // ... existing fields ...

    /// Enable test mode (runs with IPC but without keyboard capture)
    #[clap(long)]
    pub test_mode: bool,
}
```

### 2.2 IPC Socket Creation

```rust
// keyrx_daemon/src/ipc/mod.rs (new module)
pub struct IpcServer {
    socket_path: PathBuf,
    listener: tokio::net::UnixListener,
}

impl IpcServer {
    pub async fn new() -> Result<Self> {
        let socket_path = Self::get_socket_path();
        let listener = UnixListener::bind(&socket_path)?;
        Ok(Self { socket_path, listener })
    }

    pub async fn handle_connections(&self, profile_mgr: Arc<ProfileManager>) {
        loop {
            let (stream, _) = self.listener.accept().await?;
            tokio::spawn(Self::handle_client(stream, profile_mgr.clone()));
        }
    }

    async fn handle_client(stream: UnixStream, profile_mgr: Arc<ProfileManager>) {
        // Read IPC command
        // Execute command (activate profile, query status)
        // Send response
    }
}
```

### 2.3 Profile Activation via IPC

```rust
// keyrx_daemon/src/web/api/profiles.rs
pub async fn activate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if cfg!(test_mode) {
        // Send IPC command to activate profile
        let response = state.ipc_client.send_command(
            IpcCommand::ActivateProfile { name: name.clone() }
        ).await?;

        Ok(Json(json!({
            "profile": name,
            "status": "activated"
        })))
    } else {
        // Production mode: direct activation
        state.profile_service.activate(&name).await?;
        Ok(Json(json!({"profile": name})))
    }
}
```

### 2.4 Daemon Status via IPC

```rust
// keyrx_daemon/src/web/api/metrics.rs
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatusResponse>, ApiError> {
    let daemon_running = if cfg!(test_mode) {
        // Query via IPC
        state.ipc_client.send_command(IpcCommand::GetDaemonStatus)
            .await?
            .daemon_running
    } else {
        // Production mode: use shared state
        state.daemon_running.load(Ordering::Relaxed)
    };

    Ok(Json(StatusResponse {
        status: "running",
        daemon_running,
        // ... other fields
    }))
}
```

## 3. IPC Protocol

### 3.1 Command Format (JSON)
```json
{
  "type": "activate_profile" | "get_status" | "query_device",
  "payload": {
    "profile_name": "Gaming"  // for activate_profile
  }
}
```

### 3.2 Response Format (JSON)
```json
{
  "success": true | false,
  "data": {
    "daemon_running": true,
    "active_profile": "Gaming"
  },
  "error": null | "error message"
}
```

## 4. Test Mode Differences

| Feature | Production Mode | Test Mode |
|---------|----------------|-----------|
| Keyboard Capture | ✅ Active | ❌ Stubbed |
| IPC Socket | ❌ None | ✅ Active |
| Profile Activation | Direct | Via IPC |
| Daemon Status | Shared state | Via IPC |
| Event Injection | ✅ Active | ✅ Active (simulator) |
| Web Server | ✅ Active | ✅ Active |

## 5. Testing Strategy

### 5.1 Unit Tests
- Test IPC message parsing
- Test IPC command handling
- Mock IPC socket for unit tests

### 5.2 Integration Tests
- Start daemon in test mode
- Send IPC commands
- Verify responses
- Measure latency

### 5.3 E2E Tests
- Run existing failing tests with --test-mode flag
- Verify all 5 tests pass
- Measure end-to-end performance

## 6. Performance Considerations

### 6.1 IPC Overhead
- Unix socket: ~0.1ms latency (negligible)
- JSON parsing: ~0.5ms (acceptable)
- Total overhead: < 1ms per API call

### 6.2 Test Mode Startup
- IPC socket creation: ~10ms
- No keyboard capture init: saves ~500ms
- Net result: Faster startup in test mode

## 7. Security Considerations

### 7.1 Test Mode Protection
```rust
#[cfg(debug_assertions)]
const TEST_MODE_ALLOWED: bool = true;

#[cfg(not(debug_assertions))]
const TEST_MODE_ALLOWED: bool = false;

pub fn validate_test_mode(args: &RunArgs) -> Result<()> {
    if args.test_mode && !TEST_MODE_ALLOWED {
        return Err("Test mode only available in debug builds");
    }
    Ok(())
}
```

### 7.2 IPC Socket Permissions
- Unix socket: chmod 600 (owner only)
- Socket path: /tmp/keyrx-test-{pid}.sock
- Cleanup on shutdown: remove socket file

## 8. Rollback Plan

If test mode causes issues:
1. Remove --test-mode flag
2. Mark 5 IPC-dependent tests as skipped
3. Document limitation in test README
4. No production impact (test-only feature)
