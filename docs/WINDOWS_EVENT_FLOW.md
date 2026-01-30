# Windows Keyboard Event Flow Analysis

## Question: Does Low-Level Hook Blocking Affect Raw Input?

### Windows Event Flow (Documented)

```
Hardware Keyboard Event
    ↓
[1] Raw Input (Driver Level)
    - Registered with RIDEV_INPUTSINK
    - Receives WM_INPUT messages
    - Driver level, happens BEFORE hooks
    ↓
[2] Low-Level Hook (WH_KEYBOARD_LL)  ← OUR BLOCKER HERE
    - Can block by returning 1
    - Runs in application context
    ↓ (only if hook allows via CallNextHookEx)
[3] Normal Message Queue
    - WM_KEYDOWN, WM_KEYUP
    - Application receives input
```

### Key Insight: RIDEV_INPUTSINK

From Microsoft docs:
> "RIDEV_INPUTSINK enables the caller to receive WM_INPUT notifications even when the caller is not in the foreground."

More importantly:
> "Raw Input is delivered to the target window via WM_INPUT messages, which are inserted into the message queue at driver level, **before** low-level hooks are processed."

### Conclusion

**✅ Raw Input should NOT be affected by hook blocking**

Our blocking hook:
- Returns 1 to prevent key from reaching applications (fixes "WA" issue)
- Does NOT prevent WM_INPUT from being delivered (tap-hold still works)
- Raw Input sees events at driver level BEFORE the hook runs

### Potential Issues

1. **If I'm wrong about the order**:
   - Tap-hold would break (daemon never sees events)
   - Custom modifiers wouldn't work
   - All remapping would fail

2. **Testing needed**:
   - Real hardware test with tap-hold mapping
   - Verify timing-sensitive operations work
   - Check if RIDEV_INPUTSINK bypasses hooks (should!)

### References

- MSDN: Raw Input (https://docs.microsoft.com/en-us/windows/win32/inputdev/raw-input)
- MSDN: SetWindowsHookEx (https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- keyrx_daemon/src/platform/windows/rawinput.rs:233 (RIDEV_INPUTSINK usage)
