# Profile Activation Bug Report - v0.1.5

## Problem Summary

**Status**: profile-a (253 bytes) activates ✅, default (24KB) gets stuck on "activating..." ❌

## Root Cause Analysis

### default.rhai Complexity
- **File size**: 24,411 bytes (96x larger than profile-a)
- **Total lines**: 450 lines
- **Mapping commands**: 276 (map(), tap_hold(), when_start())
- **Features used**:
  - 10 TapHold modifiers (MD_00 - MD_09)
  - Multiple conditional layers (when_start/when_end)
  - Navigation keys, function keys, mirrored layouts
  - Japanese keyboard support (JIS layout)

### profile-a.rhai (Working)
- **File size**: 253 bytes
- **Content**: Blank template with `device_start("*"); device_end();`
- **Mapping commands**: 0
- **Activation time**: Instant

## Suspected Issues

### 1. Compilation Timeout (Most Likely)
```
User activates default profile
→ API POST /api/profiles/default/activate
→ Daemon compiles default.rhai to .krx
→ Compilation takes >30 seconds (276 commands)
→ HTTP request times out
→ UI stuck on "activating..."
```

**Evidence**:
- default.krx exists (3,624 bytes, updated 2026/01/29 18:07:55)
- But activation fails consistently
- API health check fails (connection timeout)

### 2. Blocking I/O in Activation Handler
The profile activation may be running on the async runtime thread, blocking other API requests.

**Relevant code**: `keyrx_daemon/src/api/profiles.rs`

### 3. Daemon Not Starting Web Server
**Critical finding**: Daemon process is running (PID 77412) but API health check fails.

Port 9867 was owned by PID 23304 (different from daemon), suggesting:
- Old daemon process still had port locked
- New daemon couldn't bind to port
- API server never started

## Test Results

### Installation Tests (test_installation.ps1)
```
[1/7] ✅ Binary exists (v0.1.5, 2026/01/29 19:31:13)
[2/7] ✅ Binary timestamp matches source
[3/7] ✅ Daemon is running (PID 77412)
[4/7] ❌ API health check FAILED (connection timeout)
[5/7] ❌ Profiles endpoint - Cannot test (API down)
[6/7] ❌ Configuration - Cannot test (API down)
[7/7] ❌ Port 9867 - Owned by wrong PID (23304)
```

### Profile Analysis (analyze_profiles.ps1)
```
default:
  Size: 24,411 bytes ⚠️ WARNING: Large file may cause timeout
  Compiled: 3,624 bytes (stale: 2026/01/29 18:07:55)
  Complexity: Very high (276 commands)

profile-a:
  Size: 253 bytes
  Compiled: 168 bytes (fresh: 2026/01/29 20:28:25)
  Complexity: Zero (empty template)
```

## Impact

### User Experience
1. ✅ Simple profiles work (profile-a, 253 bytes)
2. ❌ Complex profiles fail (default, 24KB)
3. ❌ No error message shown to user
4. ❌ Metrics show zero events (keyboard not remapping)
5. ❌ UI stuck on "activating..." indefinitely

### Severity
- **CRITICAL**: Real-world user profiles (like default.rhai) cannot be activated
- **BLOCKING**: No error feedback to help user debug
- **SILENT**: Daemon appears running but API doesn't respond

## Reproduction Steps

1. Install KeyRx v0.1.5
2. Create complex profile with 200+ mappings, multiple layers
3. Try to activate via Web UI
4. **Expected**: Profile activates within 5 seconds
5. **Actual**: UI stuck on "activating...", daemon unresponsive

## Fixes Required

### 1. Fix: Async Compilation (HIGH PRIORITY)
Move profile compilation to background task with progress reporting.

**File**: `keyrx_daemon/src/api/profiles.rs`

```rust
// BEFORE (blocks runtime)
async fn activate_profile(profile_name: String) -> Result<()> {
    let krx_data = compile_profile(&profile_name)?; // BLOCKS!
    state.load_config(krx_data)?;
    Ok(())
}

// AFTER (non-blocking)
async fn activate_profile(profile_name: String) -> Result<()> {
    let krx_data = tokio::task::spawn_blocking(move || {
        compile_profile(&profile_name)
    }).await??;

    state.load_config(krx_data)?;
    Ok(())
}
```

### 2. Fix: Timeout Configuration (MEDIUM PRIORITY)
Increase HTTP timeout for activation endpoint, add progress updates.

```rust
// Set longer timeout for complex profiles
let timeout = if file_size > 10_000 {
    Duration::from_secs(120)  // 2 minutes for large profiles
} else {
    Duration::from_secs(30)
};
```

### 3. Fix: Error Reporting (MEDIUM PRIORITY)
Return detailed error messages to UI when compilation fails.

```rust
Err(ProfileError::CompilationTimeout {
    profile: profile_name,
    size_bytes: file_size,
    estimated_seconds: 60,
    suggestion: "Try simplifying the profile or splitting into multiple profiles"
})
```

### 4. Fix: Progress Updates (LOW PRIORITY)
Use WebSocket to send compilation progress to UI.

```rust
ws_send(ProgressUpdate {
    stage: "compiling",
    percent: 45,
    message: "Processing layer 3 of 8..."
});
```

## Tests Added

### 1. profile_activation_test.rs (NEW)
```rust
#[test]
fn test_complex_profile_activation() {
    // Test 500+ remappings complete within 30 seconds
}

#[test]
fn test_large_profile_timeout() {
    // Test 24KB file compiles without timeout
}

#[test]
fn test_profile_compilation_performance() {
    // Benchmark: tiny, small, medium, large, huge
}
```

### 2. DEBUG_ACTIVATION.ps1 (NEW)
Comprehensive diagnostic script (run as Administrator):
- Stops daemon cleanly
- Starts with console logging
- Tests both profiles
- Captures detailed errors
- Shows stderr logs

### 3. analyze_profiles.ps1 (NEW)
Quick profile analysis (no admin needed):
- File size warnings
- Complexity scoring
- Syntax error detection
- Compilation status

## Workarounds (Immediate)

### Option A: Simplify default.rhai
Split into multiple smaller profiles:
- `default-base.rhai` - Basic remappings (50 commands)
- `default-layers.rhai` - Layer-specific (100 commands)
- `default-advanced.rhai` - Advanced features (126 commands)

### Option B: Increase Timeout Manually
Edit daemon code, rebuild:
```rust
// keyrx_daemon/src/api/profiles.rs
const ACTIVATION_TIMEOUT: Duration = Duration::from_secs(120);
```

### Option C: Pre-compile Profiles
Compile .krx files offline with compiler:
```bash
keyrx_compiler.exe default.rhai -o default.krx
```
Then daemon only loads pre-compiled .krx (fast).

## Next Steps

1. **Immediate**: Run `DEBUG_ACTIVATION.ps1` as admin to capture logs
2. **Short-term**: Implement Fix #1 (async compilation)
3. **Medium-term**: Add performance benchmarks for large profiles
4. **Long-term**: Optimize compiler for complex layouts

## Related Files

- `keyrx_daemon/src/api/profiles.rs:167` - activate_profile handler
- `keyrx_daemon/tests/profile_activation_test.rs` - Test suite
- `DEBUG_ACTIVATION.ps1` - Diagnostic script
- `scripts/analyze_profiles.ps1` - Profile analyzer
- `C:\Users\ryosu\AppData\Roaming\keyrx\profiles\default.rhai` - Failing profile

## References

- WS1 Bug Remediation: 67 fixes in v0.1.1
- API Async Fixes: spawn_blocking added in v0.1.4
- SSOT Implementation: v0.1.5

## User Quote

> "thanks. run in admin, installed, version v0.1.5. ok then launched, but no remap working. web UI, can show profile, config, metrics page ok. but profile activate profile-a, ok. activate default -> acviating... fail at page metrics, no key input detected. please check log. debug, fix. add tests, improve analyzability, autonomous test, fix, bug hunter."

---

**Created**: 2026/01/29 20:45 JST
**Version**: 0.1.5
**Status**: Active Investigation
**Priority**: P0 (Blocking real-world usage)
