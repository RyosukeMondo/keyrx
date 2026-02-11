# Daemon Production Readiness - Windows Release

**Status**: CRITICAL - Production blockers identified
**Priority**: P0 (Must fix before Windows release)
**Estimated Effort**: 5-8 days
**Target Release**: Pre-Windows installer build

## Executive Summary

Production validation of keyrx daemon for Windows release reveals **critical production blockers** in error handling, resource management, and service lifecycle. The daemon is not currently production-ready for Windows service deployment.

### Risk Assessment: **HIGH**

- **Crash Risk**: HIGH - 46 unwrap/expect calls in Windows platform code
- **Resource Leak Risk**: MEDIUM - Thread/mutex cleanup incomplete
- **Service Integration**: MISSING - No Windows service lifecycle support
- **Observability**: LOW - Missing health checks, metrics endpoints incomplete
- **Recovery**: LOW - Missing automatic crash recovery mechanisms

## Critical Findings

### 🔴 P0: Production Blockers

1. **Unsafe Error Handling** (46 occurrences in Windows platform)
   - `unwrap()` and `expect()` calls that will crash the daemon
   - Missing fallback paths for platform errors
   - Impact: Daemon crashes instead of degrading gracefully

2. **Missing Windows Service Support**
   - No service lifecycle management (install/uninstall/start/stop)
   - No Windows Event Log integration
   - No service control handler (SCM integration)
   - Impact: Cannot run as Windows service, manual start only

3. **Resource Leak Risks**
   - Thread spawning without cleanup tracking (11 locations)
   - Mutex poisoning handling incomplete
   - Missing Drop implementations for critical resources
   - Impact: Memory/handle leaks on error paths

4. **Missing Health Checks**
   - No `/health` endpoint for monitoring
   - No readiness/liveness probes
   - No dependency health verification (web server, hooks)
   - Impact: Cannot detect daemon failures, no automated restart

### ⚠️  P1: Production Concerns

5. **Incomplete Crash Recovery**
   - PID file management incomplete (Windows only)
   - No automatic restart on crash
   - Missing state persistence for recovery
   - Impact: Manual intervention required after crashes

6. **Observability Gaps**
   - Metrics endpoint exists but incomplete
   - No structured logging (JSON) for production
   - Missing performance counters
   - Impact: Difficult to diagnose production issues

7. **Graceful Shutdown Issues**
   - Signal handling incomplete on Windows
   - Message loop integration fragile
   - Resource cleanup order not guaranteed
   - Impact: May leave orphaned resources on exit

## Production Readiness Scorecard

| Category | Score | Status | Blockers |
|----------|-------|--------|----------|
| Error Handling | 40% | 🔴 FAIL | 46 unwrap/expect in platform code |
| Resource Management | 55% | ⚠️  WARN | Thread cleanup, mutex recovery |
| Windows Service | 0% | 🔴 FAIL | No SCM integration |
| Health Checks | 20% | 🔴 FAIL | No /health endpoint |
| Metrics | 60% | ⚠️  WARN | Incomplete implementation |
| Crash Recovery | 30% | 🔴 FAIL | No auto-restart |
| Graceful Shutdown | 65% | ⚠️  WARN | Cleanup order issues |
| Logging | 50% | ⚠️  WARN | No structured logging |
| **OVERALL** | **45%** | 🔴 **NOT READY** | 4 P0 blockers |

## Detailed Analysis

### 1. Error Handling Completeness

**Current State**: UNSAFE
- Windows platform code: 46 unwrap/expect calls
- Raw Input: 14 unwrap/expect
- Key Blocker: 9 unwrap/expect
- Device Map: 2 unwrap/expect
- Virtual Keyboard: 3 unwrap/expect

**Production Impact**:
```rust
// CURRENT (CRASHES DAEMON):
let state = PLATFORM_STATE.lock().unwrap(); // Line 40 in platform_state.rs

// PRODUCTION-SAFE (RETURNS ERROR):
let state = recover_lock_with_context(&PLATFORM_STATE, "context")?;
```

**Required Changes**:
- Replace all unwrap/expect with proper error propagation
- Add fallback paths for non-critical failures
- Implement degraded operation modes

### 2. Windows Service Support

**Current State**: MISSING
- No service installation code
- No SCM (Service Control Manager) integration
- No service control handler
- No Windows Event Log integration

**Required Implementation**:
```rust
// New module: src/platform/windows/service.rs
pub struct WindowsService {
    service_status_handle: SERVICE_STATUS_HANDLE,
    stop_event: HANDLE,
}

impl WindowsService {
    pub fn install() -> Result<()>
    pub fn uninstall() -> Result<()>
    pub fn service_main() -> Result<()>
    pub fn handle_control(control: u32) -> Result<()>
}
```

**Event Log Integration**:
- Startup/shutdown events
- Error conditions
- Configuration changes
- Performance warnings

### 3. Resource Leak Prevention

**Thread Spawning (11 locations)**:
```rust
// CURRENT (NO TRACKING):
std::thread::spawn(|| { /* work */ });

// PRODUCTION-SAFE (TRACKED):
pub struct ThreadPool {
    handles: Vec<JoinHandle<()>>,
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Join all threads on shutdown
    }
}
```

**Mutex Poisoning**:
- Only partially handled via `recover_lock_with_context`
- Missing in some critical paths
- No cleanup on poisoned state

**Required Changes**:
- Thread pool with lifecycle management
- Complete mutex recovery coverage
- Drop implementations for all platform resources

### 4. Health Check Endpoint

**Current State**: MISSING
**Required Endpoint**: `GET /health`

**Response Format**:
```json
{
  "status": "healthy|degraded|unhealthy",
  "timestamp": "2026-01-30T12:00:00Z",
  "uptime": 3600,
  "version": "0.1.5",
  "components": {
    "web_server": "healthy",
    "platform_hooks": "healthy",
    "config_loaded": "healthy",
    "remapping_active": "healthy"
  },
  "metrics": {
    "events_processed": 12345,
    "current_latency_p95": 45,
    "error_count_5m": 0
  }
}
```

**Health Check Logic**:
- Web server responding: YES/NO
- Platform hooks installed: YES/NO
- Config loaded: YES/NO
- Remapping state valid: YES/NO
- No recent crashes: Check last 5 minutes

### 5. Crash Recovery

**PID File Management (Windows)**:
```rust
// Current: Only basic PID file write
fn ensure_single_instance(config_dir: &Path) -> bool {
    // Kills old instance but no recovery
}

// Needed: Crash detection + recovery
pub struct CrashRecovery {
    pid_file: PathBuf,
    crash_log: PathBuf,
    last_crash: Option<SystemTime>,
}

impl CrashRecovery {
    pub fn check_crash(&self) -> Result<CrashInfo>
    pub fn record_clean_shutdown(&self) -> Result<()>
    pub fn attempt_recovery(&self) -> Result<()>
}
```

**Auto-Restart**:
- Windows service restart policy
- Backoff strategy (exponential)
- Max restart attempts (3-5)
- Crash loop detection

### 6. Graceful Shutdown

**Current Issues**:
1. Message loop shutdown not guaranteed
2. Resource cleanup order undefined
3. Web server may not drain connections

**Required Shutdown Sequence**:
```rust
pub fn shutdown(&mut self) {
    // 1. Stop accepting new requests
    self.web_server.stop_accepting();

    // 2. Drain active WebSocket connections (max 5s)
    self.web_server.drain_connections(Duration::from_secs(5));

    // 3. Stop event loop
    self.running.store(false, Ordering::SeqCst);

    // 4. Unhook platform (release keyboard)
    self.platform.shutdown()?;

    // 5. Cleanup PID file
    cleanup_pid_file(&self.config_dir);

    // 6. Final log flush
    log::info!("Shutdown complete");
}
```

### 7. Observability

**Metrics Endpoint** (`GET /api/metrics`):
- Exists but incomplete
- Missing: error rates, event types, mapping distribution
- No Prometheus format support

**Logging**:
- Current: Text format only
- Needed: JSON structured logging for production

```json
{
  "timestamp": "2026-01-30T12:00:00.123Z",
  "level": "ERROR",
  "service": "keyrx_daemon",
  "event": "platform_hook_failed",
  "context": {
    "error": "Hook installation failed",
    "admin_rights": false,
    "retry_count": 3
  }
}
```

**Performance Counters** (Windows specific):
- CPU usage per thread
- Memory working set
- Handle count
- Event processing rate

## Acceptance Criteria

### Must Have (P0) - Before Release
- [ ] Zero unwrap/expect calls in production code paths
- [ ] Windows service lifecycle (install/uninstall/start/stop)
- [ ] Health check endpoint (`/health`)
- [ ] Automatic crash recovery with backoff
- [ ] Thread pool with lifecycle tracking
- [ ] Complete mutex recovery coverage

### Should Have (P1) - Post-Release Acceptable
- [ ] Structured JSON logging
- [ ] Prometheus metrics endpoint
- [ ] Performance counters
- [ ] Crash dump collection
- [ ] State persistence for recovery

### Testing Requirements
- [ ] Service installation E2E test
- [ ] Crash recovery simulation
- [ ] Resource leak detection (valgrind equivalent)
- [ ] Load test (1000 events/sec for 1 hour)
- [ ] Graceful shutdown under load

## Dependencies

- **Blocking**: Windows installer cannot proceed until P0 items resolved
- **Related Specs**: None (new initiative)
- **External**: Windows SDK for service APIs

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Week 1 | 3 days | Error handling refactor, unwrap elimination |
| Week 1-2 | 2 days | Windows service implementation |
| Week 2 | 2 days | Health checks, crash recovery |
| Week 2 | 1 day | Testing, validation |

**Total**: 8 days (conservative estimate)

## Risks

1. **Windows Service Complexity**: SCM integration may reveal additional issues
2. **Backwards Compatibility**: Service changes may affect existing deployments
3. **Testing Coverage**: Need Windows-specific E2E infrastructure
4. **Performance Impact**: Health checks and metrics may add latency

## Success Metrics

- **Crash Rate**: < 0.01% (1 crash per 10,000 events)
- **Recovery Time**: < 5 seconds (auto-restart)
- **Resource Leaks**: Zero leaks in 24-hour soak test
- **Health Check Response**: < 100ms
- **Service Reliability**: 99.9% uptime in production

## Next Steps

1. Review findings with team
2. Approve production readiness plan
3. Prioritize P0 fixes
4. Create detailed implementation tasks
5. Setup Windows E2E test infrastructure
