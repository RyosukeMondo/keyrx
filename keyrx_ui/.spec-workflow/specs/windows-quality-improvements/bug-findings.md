# Windows Quality Audit - Bug Findings Report

This report documents the findings of the Phase 1 audit for Windows Quality Improvements.

## Summary Statistics

| Severity | Count |
| :--- | :--- |
| **CRITICAL** | 1 |
| **HIGH** | 3 |
| **MEDIUM** | 2 |
| **LOW** | 2 |
| **Total** | **8** |

---

## CRITICAL Priority

### WIN-BUG #1: Use-After-Free in `wnd_proc` during destruction
- **File:** [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** `wnd_proc` (Line 206) vs `Drop` (Line 182)
- **Root Cause:** `RawInputManager::drop` frees the `RawInputContext` before destroying the window. If `wnd_proc` is concurrently executing (e.g., on a different thread or due to pending messages), it dereferences a freed pointer.
- **User Impact:** Random daemon crashes, especially on shutdown or device reconfiguration.
- **Reproduction:** Call `RawInputManager::drop` while high-frequency input is occurring.

---

## HIGH Priority

### WIN-BUG #2: RwLock Poisoning Cascade
- **File:** [device_map.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/device_map.rs), [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** All `.unwrap()` calls on `RwLock` read/write access.
- **Root Cause:** Use of `.unwrap()` on lock acquisition. If a thread panics while holding a lock, all subsequent accesses will panic.
- **User Impact:** Total daemon hang or crash if a single thread panics.
- **Reproduction:** Deliberately panic in a thread holding the `device_map` lock.

### WIN-BUG #3: Unbounded Memory Allocation in `GetRawInputData`
- **File:** [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** Lines 214-230
- **Root Cause:** `close_size` is used to allocate a `Vec` without bounds checking. A malicious or buggy driver could trigger an OOM.
- **User Impact:** Potential out-of-memory crash.
- **Reproduction:** Simulate a `WM_INPUT` message with a very large `cbSize`.

### WIN-BUG #4: Missing Panic Recovery in Message Loop
- **File:** [main.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/main.rs)
- **Location:** Lines 187-220
- **Root Cause:** The Windows message loop lacks a `catch_unwind`. Any panic in `wnd_proc` or event processing terminates the main thread.
- **User Impact:** Sudden exit without recovery or state cleanup.
- **Reproduction:** Trigger a panic inside `wnd_proc`.

---

## MEDIUM Priority

### WIN-BUG #5: Silent Failures in Device Hotplug
- **File:** [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** Line 251 (`let _ = add_device()`)
- **Root Cause:** New device arrival errors are entirely ignored.
- **User Impact:** Newly plugged keyboards may fail to remap silently.
- **Reproduction:** Connect a device that fails `GetRawInputDeviceInfoW`.

### WIN-BUG #6: Race Condition in Device Removal
- **File:** [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** `WM_INPUT_DEVICE_CHANGE` vs `WM_INPUT`
- **Root Cause:** A device might be removed from the `device_map` while `WM_INPUT` is still processing its last few events.
- **User Impact:** "Unknown device" logs or potential panics.
- **Reproduction:** Rapidly plug and unplug a device while typing.

---

## LOW Priority

### WIN-BUG #7: Layout-Dependent Scancode Mapping
- **File:** [keycode.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/keycode.rs)
- **Location:** `scancode_to_keycode` (Lines 161-176)
- **Root Cause:** Reliance on `MapVirtualKeyW` which depends on the active thread's keyboard layout instead of a fixed hardware mapping table.
- **User Impact:** Incorrect remapping if the system layout changes.
- **Reproduction:** Switch system layout from US to Dvorak and observe scancode mapping changes.

### WIN-BUG #8: Missing Raw Input Unregistration
- **File:** [rawinput.rs](file:///c:/Users/ryosu/repos/keyrx/keyrx_daemon/src/platform/windows/rawinput.rs)
- **Location:** `Drop` implementation
- **Root Cause:** `RawInputManager` does not call `RegisterRawInputDevices` with `RIDEV_REMOVE` on drop.
- **User Impact:** Potential resource leak in the OS window manager if the process lives long.
- **Reproduction:** Repeatedly create and destroy `RawInputManager` instances.
