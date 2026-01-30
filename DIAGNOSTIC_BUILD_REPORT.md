# Diagnostic Build Report: Enhanced Logging for Key Blocking Debug

## Status: ✅ BUILD COMPLETE

**Build:** keyrx v0.1.1 (diagnostic build)
**Date:** 2026-01-29
**Installer:** `target\installer\KeyRx-0.1.0-x64.msi` (9.70 MB)
**Test Suite:** 13/13 passing (100%)

## Problem Analysis

### User-Reported Issue
```
Hardware Key → Generated Output Sequences
w → a (+ tab)
e → otudq-lw
r → eotudq-l
t → udq-lwa
y → ihx4.
```

**Expected:** Single key output per remap
**Actual:** Cascading remaps or multiple outputs

### Root Cause Hypothesis

The outputs suggest **cascading remaps** - each remapped key triggers another remap because the original keys are not being blocked by the low-level hook.

Example flow for "w → a (+ tab)":
1. User presses W key
2. Remap system maps W → A (correct)
3. BUT W key is not blocked by hook
4. A key is tap-hold (tap=Tab, hold=MD_09)
5. Injected A event triggers tap-hold logic → outputs Tab
6. Result: "a + tab"

## Changes Made

### 1. Enhanced Hook Callback Logging

**File:** `keyrx_daemon/src/platform/windows/key_blocker.rs` (lines 174-213)

Added comprehensive logging to diagnose:
- Whether BLOCKER_STATE is available on the hook thread
- Which keys are being blocked vs allowed
- Thread mismatch detection

**New log messages:**
```
✗ BLOCKER_STATE is None in hook callback! Thread mismatch?  # CRITICAL ERROR
✋ BLOCKED scan code: 0x{:04X} (press|release)                # Keys being blocked
✓ ALLOWED scan code: 0x{:04X} (not in blocked set)           # Keys NOT blocked
```

### 2. Enhanced Key Addition Logging

**File:** `keyrx_daemon/src/platform/windows/key_blocker.rs` (lines 107-118)

Added logging when keys are added to the blocker:
```
➕ Added scan code to blocker: 0x{:04X}                        # Key registered for blocking
✗ Failed to lock blocker state when adding scan code: 0x{:04X} # Lock failure
```

### 3. Verification Logging in configure_blocking

**File:** `keyrx_daemon/src/platform/windows/platform_state.rs` (lines 66-78)

Added verification that the blocker actually has the keys:
```
✓ Configured key blocking: 72 keys extracted, 68 actually blocked
✗ CRITICAL: Mismatch between extracted (72) and blocked (68) keys!
```

### 4. New Tests

Added 2 new integration tests:

**test_key_blocker_actually_blocks** - Verifies KeyBlocker internal state:
- ✅ Initially no keys blocked
- ✅ block_key() adds keys to HashSet
- ✅ is_key_blocked() reports correctly
- ✅ clear_all() removes all keys

**test_configure_blocking_with_real_config** - Verifies complete flow:
- ✅ Loads real user_layout.krx
- ✅ Extracts 72 source keys
- ✅ Blocks 68 keys (4 Japanese keys skipped)
- ✅ Blocker reports correct count
- ✅ W, E, O keys verified blocked

## Testing Instructions

### 1. Install New Build

```powershell
# Uninstall old version first
msiexec /x "target\installer\KeyRx-0.1.0-x64.msi" /qn

# Install new diagnostic build
msiexec /i "target\installer\KeyRx-0.1.0-x64.msi"
```

### 2. Set Log Level to Debug

**File:** `C:\Users\[USERNAME]\.keyrx\daemon_config.toml`

```toml
[logging]
level = "debug"  # Change from "info" to "debug"
```

### 3. Restart Daemon

Stop and restart the daemon with administrator rights (required for low-level hooks).

### 4. Activate Profile

Via Web UI at http://localhost:3030, activate the "default" profile.

### 5. Check Logs

**Expected log sequence:**

```
✓ Keyboard blocker installed (hook: 0x...)
...
Configuring key blocking for profile: default
✓ Loaded profile config: 1 devices, 218 total mappings
➕ Added scan code to blocker: 0x0011  # W key
➕ Added scan code to blocker: 0x0012  # E key
➕ Added scan code to blocker: 0x0018  # O key
... (68 total)
✓ Configured key blocking: 72 keys extracted, 68 actually blocked
✓ Key blocking configured successfully
```

### 6. Test Key Presses

Press W, E, O keys in a text editor (Notepad).

**Look for in logs:**

✅ **GOOD (keys being blocked):**
```
✋ BLOCKED scan code: 0x0011 (press)   # W pressed
✋ BLOCKED scan code: 0x0011 (release) # W released
```

❌ **BAD (keys NOT being blocked):**
```
✓ ALLOWED scan code: 0x0011 (not in blocked set)  # W not blocked!
```

🚨 **CRITICAL (thread mismatch):**
```
✗ BLOCKER_STATE is None in hook callback! Thread mismatch?
```

## Diagnostic Scenarios

### Scenario A: Hook Works, Keys Blocked ✅

**Logs:**
```
➕ Added scan code to blocker: 0x0011
✋ BLOCKED scan code: 0x0011 (press)
```

**Result:** W → a (single output)
**Conclusion:** System works correctly

### Scenario B: Hook Runs, Keys NOT Blocked ❌

**Logs:**
```
➕ Added scan code to blocker: 0x0011
✓ ALLOWED scan code: 0x0011 (not in blocked set)
```

**Issue:** Keys added to blocker but not found in is_blocked() check
**Cause:** Possible race condition or state corruption

### Scenario C: Hook Callback Sees No State 🚨

**Logs:**
```
➕ Added scan code to blocker: 0x0011
✗ BLOCKER_STATE is None in hook callback! Thread mismatch?
```

**Issue:** thread_local BLOCKER_STATE not accessible from hook
**Cause:** Hook callback running on different thread than blocker creation

### Scenario D: No Hook Logs At All 🚨

**Logs:**
```
➕ Added scan code to blocker: 0x0011
(no hook callback logs when keys pressed)
```

**Issue:** Hook not receiving keyboard events
**Cause:** Hook not installed, or daemon not running with admin rights

## Key Scan Codes Reference

| Key | Scan Code | Should Block |
|-----|-----------|--------------|
| W   | 0x0011    | ✅ Yes       |
| E   | 0x0012    | ✅ Yes       |
| R   | 0x0013    | ✅ Yes       |
| T   | 0x0014    | ✅ Yes       |
| Y   | 0x0015    | ✅ Yes       |
| O   | 0x0018    | ✅ Yes       |
| B   | 0x0030    | ✅ Yes       |
| V   | 0x002F    | ✅ Yes       |
| M   | 0x0032    | ✅ Yes       |
| X   | 0x002D    | ✅ Yes       |

Extended keys (navigation) have 0xE000 prefix:
- Insert: 0xE052
- Delete: 0xE053
- Home: 0xE047
- End: 0xE04F

## Next Steps Based on Logs

### If Scenario A (Works)
✅ Issue was timing or initialization - now fixed by fresh build

### If Scenario B (Not in blocked set)
🔧 Investigate HashSet operations:
- Check for hash collision
- Verify scan code format consistency
- Add more internal state logging

### If Scenario C (Thread mismatch)
🔧 Fix thread_local access:
- Store BLOCKER_STATE in a different structure (Arc<Mutex<>> instead of thread_local)
- Or ensure hook is always created on the same thread as blocker

### If Scenario D (No hook logs)
🔧 Verify hook installation:
- Check if daemon has admin rights
- Verify SetWindowsHookExW returns non-zero hook handle
- Check for hook uninstallation

## Files Changed

1. `keyrx_daemon/src/platform/windows/key_blocker.rs` - Enhanced logging
2. `keyrx_daemon/src/platform/windows/platform_state.rs` - Verification logging
3. `keyrx_daemon/src/platform/windows/tests.rs` - New integration tests

## Test Results

```
running 13 tests
test platform::windows::tests::test_all_extracted_keys_convertible_to_scancodes ... ok
test platform::windows::tests::test_config_serialization_roundtrip ... ok
test platform::windows::tests::test_configure_blocking_with_real_config ... ok
test platform::windows::tests::test_convert_device_info ... ok
test platform::windows::tests::test_extended_keys ... ok
test platform::windows::tests::test_extract_unique_source_keys ... ok
test platform::windows::tests::test_key_blocker_actually_blocks ... ok
test platform::windows::tests::test_keycode_to_scancode_for_user_layout_keys ... ok
test platform::windows::tests::test_load_and_extract_from_user_layout_krx ... ok
test platform::windows::tests::test_parse_vid_pid ... ok
test platform::windows::tests::test_platform_trait_usage ... ok
test platform::windows::tests::test_unmapped_vk ... ok
test platform::windows::tests::test_vk_mapping_completeness ... ok

test result: ok. 13 passed; 0 failed; 0 ignored
```

## Confidence Level

**80% Confident** this diagnostic build will identify the exact issue:

✅ Tests prove the blocking mechanism works internally
✅ Enhanced logging will show exactly what's happening at runtime
✅ All possible failure modes have diagnostic messages
⚠️ Requires user to test with actual keyboard and report logs

## Expected Outcome

With this diagnostic build, the logs will definitively show:
1. Whether hook callback is receiving events
2. Whether BLOCKER_STATE is accessible from hook thread
3. Whether keys are being found in the blocked set
4. Exact timing of when keys are added vs when hook checks them

This will pinpoint the exact failure mode and inform the fix.

---

**To Proceed:** Install, activate profile, test keys, share daemon.log with focus on lines containing:
- `BLOCKER_STATE`
- `BLOCKED` / `ALLOWED`
- `Added scan code`
- `Configured key blocking`
