# Remap Behavior Diagnosis Report

## User-Reported Issue

**Symptom**: When pressing 'a' key with default profile (user_layout.rhai) activated, output is 'wa' (and possibly arrow '<-')

**Expected**: Single remapped output
**Actual**: Multiple characters including:
- 'w' (hardware key code leaking through?)
- 'a' (remapped character)
- arrow '<-' (possibly from layer MD_00)

## Configuration Analysis

From `/api/config` endpoint, the active profile has:

### Base Mappings
```
map("VK_W", "VK_A");                          // W -> A
tap_hold("VK_A", "VK_Tab", "MD_09", 200);    // A tap=Tab, hold=MD_09
```

### Active Layers
- MD_00 (32 mappings)
- MD_01 (30 mappings)
- MD_02 (30 mappings)
- MD_03 (30 mappings)
- MD_04 (15 mappings)
- MD_05 (10 mappings)
- MD_06 (15 mappings)
- MD_08 (4 mappings)
- MD_10 (32 mappings) - Shift layer

## Potential Root Causes

### 1. Hardware Key Code Leakage
**Issue**: Original hardware key code (VK_W) is being sent to OS BEFORE remapping to VK_A

**Evidence**:
- User sees 'w' in output (the original key)
- Then 'a' appears (the remapped key)

**Root Cause**: Windows keyboard hook may not be suppressing the original key event before remapping

**Location**: `keyrx_daemon/src/platform/windows/mod.rs` - keyboard hook callback

**Fix Required**: Ensure `LLKHF_INJECTED` flag is NOT set on original events, and hook returns 1 to suppress

### 2. Layer Contamination
**Issue**: Multiple layers are active simultaneously when they shouldn't be

**Evidence**:
- User sees arrow '<-' which suggests layer MD_00 is active
- Layer should only be active when holding specific keys

**Root Cause**: Layer activation/deactivation state not being properly managed

**Location**: `keyrx_core/src/runtime/state.rs` - DeviceState modifier management

**Fix Required**: Verify layers are properly deactivated when modifier keys are released

### 3. Tap-Hold Double Processing
**Issue**: Both tap action and hold action are firing, or tap-hold is not suppressing original key

**Evidence**:
- tap_hold("VK_A", "VK_Tab", "MD_09", 200) may be outputting both A and Tab
- Threshold timing (200ms) may not be working correctly

**Root Cause**: Tap-hold state machine may not be properly suppressing original key until decision is made

**Location**: `keyrx_core/src/runtime/tap_hold/processor.rs`

**Fix Required**: Ensure original key is buffered and not sent until tap/hold decision is made

### 4. Chained Remapping Contamination
**Issue**: When W -> A and A -> Tab are chained, multiple outputs occur

**Evidence**:
- map("VK_W", "VK_A") then tap_hold("VK_A", "VK_Tab", "MD_09", 200)
- This should produce: W press → A (remapped) → Tab (tap-hold resolved)
- Instead: W, A, Tab all appear

**Root Cause**: Remapped keys are processed multiple times through the remap engine

**Location**: `keyrx_core/src/runtime/` - event processing loop

**Fix Required**: Mark remapped events to prevent re-processing

## Diagnostic Tests Created

Created `keyrx_daemon/tests/layer_contamination_test.rs` with:

1. **test_simple_a_key_remapping**: Tests A→B without layers or tap-hold
2. **test_a_key_with_tap_hold**: Tests tap-hold in isolation
3. **test_complex_profile_with_conflicting_mappings**: Tests W→A plus A tap-hold (user's scenario)
4. **test_layer_state_isolation**: Tests layer activation/deactivation

**Status**: Tests fail due to profile compilation issues (need .krx files)

## Core Test Results

Ran core tap-hold tests: **118/118 PASSED ✅**

This confirms the tap-hold state machine logic is correct in isolation.
The issue is likely in the integration between:
- Windows keyboard hook (suppression)
- Daemon event processing (chain handling)
- Layer state management (activation/deactivation)

## Recommended Investigation Steps

### Step 1: Check Windows Hook Suppression
```rust
// In keyrx_daemon/src/platform/windows/mod.rs
// Verify keyboard hook callback returns 1 to suppress original event
unsafe extern "system" fn keyboard_hook_callback(...) -> LRESULT {
    // Check: Is original event being suppressed?
    // Check: Are injected events marked with LLKHF_INJECTED?
    // Check: Is return value 1 (suppress) or 0 (pass-through)?
}
```

### Step 2: Add Event Tracing
Enable detailed logging to see exact event flow:
```
Original Event: VK_W, Press
Remapped To: VK_A, Press
Tap-Hold Decision: Tap → VK_Tab
Layer Activated: MD_09
Output Event: VK_Tab, Press
```

### Step 3: Check Layer State Management
```rust
// In keyrx_core/src/runtime/state.rs
// Add debug logging for modifier set/clear operations
pub fn set_modifier(&mut self, id: u8) {
    log::debug!("Activating layer/modifier: {}", id);
    // Check: Is this being called correctly?
    // Check: Is clear_modifier called on key release?
}
```

### Step 4: Test with Simplified Profile
Create minimal test profile:
```rhai
device_start("*");
  map("VK_A", "VK_B");  // ONLY this, no tap-hold, no layers
device_end();
```

If this still shows double output (A + B), the issue is in basic remapping.
If this works, add tap-hold:
```rhai
device_start("*");
  tap_hold("VK_A", "VK_Tab", "MD_00", 200);
device_end();
```

### Step 5: Check Real-Time Metrics
```bash
# While daemon is running and processing real keyboard input
curl http://localhost:9867/api/metrics/events?count=100

# Look for patterns:
# - Are original events appearing in metrics?
# - Are multiple remapped events appearing for single input?
# - Are layer activation/deactivation events correct?
```

## Quick Test Commands

```powershell
# 1. Activate simple profile
curl -X POST http://localhost:9867/api/profiles/default/activate

# 2. Clear metrics
curl -X DELETE http://localhost:9867/api/metrics/events

# 3. Type a single 'a' key on physical keyboard

# 4. Check metrics
curl http://localhost:9867/api/metrics/events?count=100

# Expected: 2 events (press, release) with REMAPPED output
# Bug: Multiple press events or original key showing up
```

## Next Steps

1. **Immediate**: Run simplified profile test to isolate issue
2. **Short-term**: Add detailed event tracing/logging
3. **Medium-term**: Fix Windows hook suppression if that's the issue
4. **Long-term**: Add integration tests that verify real keyboard input → output flow

## Status

⚠️ **CRITICAL BUG**: Remap engine producing multiple outputs for single input
📊 **Core Logic**: Working correctly (118/118 tests pass)
🔍 **Investigation**: Need real-time tracing of Windows hook → daemon → output flow
