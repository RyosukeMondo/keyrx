# Integration Issues Checklist

**Generated**: 2026-02-02
**For**: Task teams 1-5
**Purpose**: Critical issues discovered during integration testing

---

## 🔴 CRITICAL ISSUES (Must Fix Before Deployment)

### Issue #1: Daemon State Not Initialized
**Owner**: Task 1 (Daemon State Management)
**Priority**: P0
**Impact**: All metrics APIs fail, status shows incorrect state

**Problem**:
```json
// Current behavior:
{
    "daemon_running": false,  // ❌ Should be true
    "uptime_secs": null,      // ❌ Should show uptime
    "device_count": null      // ❌ Should show count
}
```

**Files to Fix**:
- `keyrx_daemon/src/daemon/platform_runners/windows.rs:320`
- `keyrx_daemon/src/daemon/mod.rs`
- `keyrx_daemon/src/daemon/shared_state.rs`

**Action Required**:
1. In `run_event_loop()`, call `shared_state.set_running(true)`
2. Set device count after device discovery
3. Initialize uptime tracking
4. Test that status API returns correct values

**Verification**:
```bash
curl http://localhost:9867/api/status
# Should return daemon_running: true
```

---

### Issue #2: Profile Activation False Success
**Owner**: Task 2 (Profile Management)
**Priority**: P0
**Impact**: Users think profile is active but it's not

**Problem**:
```bash
# Returns success:
{"success": true, "compile_time_ms": 0, "reload_time_ms": 0}

# But profile is NOT active:
curl /api/profiles/active
{"active_profile": null}  # ❌
```

**Files to Fix**:
- `keyrx_daemon/src/services/profile_service.rs`
- `keyrx_daemon/src/web/handlers/profile.rs`

**Action Required**:
1. Actually compile the profile configuration
2. Load the .krx binary into RemappingState
3. Call `shared_state.set_active_profile(Some(name))`
4. Return actual compile/reload times
5. Add error handling for compilation failures

**Verification**:
```bash
curl -X POST /api/profiles/default/activate
curl /api/profiles/active
# Should return: {"active_profile": "default"}
```

---

### Issue #3: Metrics API Fails
**Owner**: Task 3 (Metrics Collection)
**Priority**: P0
**Impact**: Cannot monitor daemon performance

**Problem**:
```json
{
    "error": {
        "code": "DAEMON_NOT_RUNNING",
        "message": "Daemon is not running"
    }
}
```

**Root Cause**: Cascading failure from Issue #1 (daemon state not initialized)

**Action Required**:
1. **Wait for Issue #1 to be fixed**
2. Then verify metrics endpoints work
3. Test event recording
4. Test latency statistics

**Verification**:
```bash
curl http://localhost:9867/api/metrics/events
# Should return event array, not error
```

---

## 🟡 HIGH PRIORITY ISSUES

### Issue #4: Latency Stats Serialization Bug
**Owner**: Task 3 (Metrics Collection)
**Priority**: P1
**Impact**: Metrics API returns null count field

**Problem**:
```rust
// Test failure at metrics.rs:305
assertion `left == right` failed
  left: Null   // ❌ count field is null
 right: 10
```

**Files to Fix**:
- `keyrx_daemon/src/web/handlers/metrics.rs`
- `keyrx_daemon/src/daemon/metrics.rs`

**Action Required**:
1. Check serialization attributes on `LatencyStats::count`
2. Remove `#[serde(skip_serializing_if)]` if present
3. Ensure count always serializes, even when zero
4. Re-run test: `cargo test test_latency_stats_serialization`

---

### Issue #5: Static Mut Reference Warning
**Owner**: Task 4 (Platform Integration)
**Priority**: P1
**Impact**: Potential undefined behavior

**Problem**:
```rust
// platform/windows/platform_state.rs:39
unsafe { PLATFORM_STATE.clone() }  // ⚠️ UB risk
```

**Action Required**:
1. Replace mutable static with `OnceLock<Arc<Mutex<T>>>`
2. Remove unsafe code
3. Ensure thread safety
4. Re-run clippy: `cargo clippy --workspace -- -D warnings`

**Code Example**:
```rust
use std::sync::OnceLock;

static PLATFORM_STATE: OnceLock<Arc<Mutex<PlatformState>>> = OnceLock::new();

pub fn get_platform_state() -> Arc<Mutex<PlatformState>> {
    PLATFORM_STATE.get_or_init(|| {
        Arc::new(Mutex::new(PlatformState::new()))
    }).clone()
}
```

---

## 🟢 LOW PRIORITY ISSUES

### Issue #6: Unused Code Warnings
**Owner**: All teams
**Priority**: P2
**Impact**: Code cleanliness

**Warnings**:
1. `platform_runners/windows.rs:320` - unused `container` parameter
2. `auth/rate_limit.rs:26` - unused `last_attempt` field
3. `web/api/diagnostics.rs:246` - unused `format_bytes()` function

**Action Required**:
```bash
cargo fix --lib -p keyrx_daemon
cargo fmt
cargo clippy --workspace -- -D warnings
```

---

## Test Results Summary

| Category | Status | Count |
|----------|--------|-------|
| ✅ Passed | PASS | 641/650 (98.6%) |
| ❌ Failed | FAIL | 1/650 (0.15%) |
| ⏸️ Ignored | SKIP | 8/650 (1.2%) |
| ⏱️ Duration | - | 2.08s |

**Failed Test**: `test_latency_stats_serialization` (Issue #4)

**Ignored Tests**: 8 tests requiring MockPlatform implementation

---

## Integration Test Commands

### Quick Verification:
```bash
# 1. Build
cargo build --release -p keyrx_daemon

# 2. Start daemon (admin required)
.\target\release\keyrx_daemon.exe run

# 3. Test status
curl http://localhost:9867/api/status

# 4. Test profile activation
curl -X POST http://localhost:9867/api/profiles/default/activate
curl http://localhost:9867/api/profiles/active

# 5. Test metrics
curl http://localhost:9867/api/metrics/events
```

### Full Test Suite:
```bash
cargo test -p keyrx_daemon --lib
```

---

## Success Criteria

### Before marking tasks complete, verify:

#### ✅ Task 1 (State Management):
- [ ] `daemon_running` returns `true` after startup
- [ ] `uptime_secs` returns non-null value
- [ ] `device_count` returns detected device count
- [ ] Status API shows correct state

#### ✅ Task 2 (Profile Management):
- [ ] Profile activation compiles config
- [ ] Active profile name is stored in SharedState
- [ ] Active profile API returns correct name
- [ ] Compile/reload times are non-zero

#### ✅ Task 3 (Metrics):
- [ ] Metrics API doesn't return "daemon not running"
- [ ] Event log populates with key events
- [ ] Latency stats serialize correctly
- [ ] Test `test_latency_stats_serialization` passes

#### ✅ Task 4 (Platform):
- [ ] No unsafe static mut references
- [ ] Platform state uses safe Rust patterns
- [ ] Clippy passes with `-D warnings`

#### ✅ Task 5 (Error Handling):
- [ ] Errors propagate correctly
- [ ] User-friendly error messages
- [ ] No silent failures
- [ ] Proper HTTP status codes

---

## Dependency Graph

```
Issue #1 (Daemon State)
    ↓ blocks
Issue #3 (Metrics API)

Issue #2 (Profile Activation)
    ↓ required for
End-to-End Key Remapping Tests

Issue #4 (Latency Serialization)
    ↓ required for
Full Metrics API Functionality
```

**Critical Path**: Fix Issue #1 → Fix Issue #2 → Re-test everything

---

## Full Report

See complete analysis in: `docs/integration-test-report.md`

---

**Status**: 🔴 **BLOCKING** - Do not deploy until critical issues resolved

**Next Review**: After Issues #1 and #2 are fixed
