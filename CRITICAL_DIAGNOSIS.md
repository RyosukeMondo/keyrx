# CRITICAL DIAGNOSIS: Key Blocking Failure

## Status: 🚨 CRITICAL - Getting Worse

**Date:** 2026-01-29
**Version:** 0.1.2
**Issue:** Key blocking hook completely non-functional, causing cascading remaps

## User Report - Worsening Behavior

```
Input → Output
w     → a          (expected, but also outputs tab)
e     → otud-lwa   (cascading garbage - WORSE)
r     → eotu-lwa   (cascading garbage - WORSE)
t     → udq-lwa    (cascading garbage - WORSE)
y     → ihx4.      (cascading garbage - WORSE)
u     → dq-lwa     (cascading garbage - WORSE)
i     → hx4.       (cascading garbage - WORSE)
o     → tudq-lwa   (cascading garbage - WORSE)
p, n  → keep repeating
```

**Analysis:** ZERO keys are being blocked. Each key press triggers:
1. Original key reaches system (not blocked)
2. Remap system detects it and injects mapped key
3. Mapped key ALSO reaches system (also not blocked if it's remapped)
4. Creates cascade: e → o → t → u → d → q → ...

## Root Cause Hypothesis

### The thread_local Problem

**File:** `keyrx_daemon/src/platform/windows/key_blocker.rs:32-35`

```rust
thread_local! {
    static BLOCKER_STATE: std::cell::RefCell<Option<Arc<Mutex<KeyBlockerState>>>> =
        std::cell::RefCell::new(None);
}
```

**CRITICAL BUG:** The Windows hook callback runs on a **different thread** than where the blocker was created!

### Windows Hook Thread Behavior

From Microsoft docs:
> "Low-level hooks are called in the context of the thread that installed the hook."

**But**: SetWindowsHookExW with `WH_KEYBOARD_LL` runs on the **message loop thread**, which may be:
- A different thread than where `KeyBlocker::new()` was called
- A Windows system thread
- Not the thread that set `BLOCKER_STATE` thread_local

### Evidence

1. ✅ **Tests pass** - All 13 tests pass because they run on single thread
2. ❌ **Runtime fails** - Hook callback can't access `BLOCKER_STATE` thread_local
3. 🔍 **Enhanced logging** added but user needs to check if:
   ```
   ✗ BLOCKER_STATE is None in hook callback! Thread mismatch?
   ```
   appears in logs

## Immediate Fix Required

### Option 1: Replace thread_local with static Mutex (Recommended)

```rust
// Replace thread_local with a static Arc<Mutex<>>
static BLOCKER_STATE: OnceLock<Arc<Mutex<KeyBlockerState>>> = OnceLock::new();

impl KeyBlocker {
    pub fn new() -> Result<Self, String> {
        let state = Arc::new(Mutex::new(KeyBlockerState::new()));

        // Initialize static state (only once)
        let _ = BLOCKER_STATE.set(state.clone());

        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                GetModuleHandleW(ptr::null()),
                0,
            )
        };

        Ok(Self { hook, state })
    }
}

unsafe extern "system" fn keyboard_hook_proc(...) -> LRESULT {
    // Access static state instead of thread_local
    let should_block = if let Some(state_arc) = BLOCKER_STATE.get() {
        if let Ok(state) = state_arc.lock() {
            state.is_blocked(full_scan_code)
        } else {
            false
        }
    } else {
        false
    };

    if should_block {
        return 1; // Block
    }
    CallNextHookEx(...)
}
```

### Option 2: Use SendInput with LLKHF_INJECTED flag detection

Detect injected events and don't remap them:

```rust
unsafe extern "system" fn keyboard_hook_proc(...) -> LRESULT {
    let kbd = *(l_param as *const KBDLLHOOKSTRUCT);

    // LLKHF_INJECTED = 0x10
    if (kbd.flags & 0x10) != 0 {
        // This is an injected event, don't remap it
        return CallNextHookEx(...);
    }

    // Normal event handling...
}
```

## Next Steps

### 1. Check Logs IMMEDIATELY

**File:** `C:\Users\[USERNAME]\.keyrx\daemon.log`

**Look for:**
```
✗ BLOCKER_STATE is None in hook callback! Thread mismatch?
```

If this appears, it CONFIRMS the thread_local issue.

### 2. Run E2E Tests

```bash
cargo test --test e2e_key_blocking -- --nocapture --ignored
```

These tests verify actual runtime behavior with virtual keyboard.

### 3. Apply Fix

Based on log evidence, apply Option 1 (static Mutex) or Option 2 (injected flag).

## Files to Check

1. **Daemon log:** `C:\Users\[USERNAME]\.keyrx\daemon.log`
2. **Config:** `C:\Users\[USERNAME]\.keyrx\daemon_config.toml`
   ```toml
   [logging]
   level = "debug"  # SET THIS TO SEE DIAGNOSTICS
   ```

## Test Suite Status

| Test Type | Count | Status | Catches Bug? |
|-----------|-------|--------|--------------|
| Unit tests | 13 | ✅ Pass | ❌ No (single thread) |
| Integration | 2 | ✅ Pass | ❌ No (single thread) |
| E2E | 5 | ⚠️ Ignored | ✅ Yes (requires admin) |

**Problem:** Unit tests don't catch multi-threaded hook issues!

## Priority Actions

1. **USER:** Set `level = "debug"` in config, restart, share logs
2. **DEV:** Implement static Mutex fix for thread safety
3. **DEV:** Add E2E tests that run in CI with virtual keyboard
4. **DEV:** Add hook callback diagnostics to detect thread mismatch

## References

- Microsoft: [SetWindowsHookExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- Microsoft: [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644985(v=vs.85))
- Rust: [OnceLock](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)

---

**Bottom Line:** The hook is installed but can't access the blocked keys list due to thread_local isolation. Fix: Use static storage that's accessible from any thread.
