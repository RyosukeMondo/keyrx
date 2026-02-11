# KeyRx Windows Architecture

This document describes the architecture of the Windows platform implementation in KeyRx.

## Core Components

### Raw Input API Integration
Unlike the Linux implementation which uses `evdev`, the Windows implementation leverages the **Raw Input API** (`WM_INPUT`). This provides:
- **Per-device discrimination:** Distinguishing between multiple keyboards via their `hDevice` handles.
- **Low latency:** Direct access to hardware events before they are processed by the high-level Win32 input system.

### Message Pump
The Raw Input API requires a hidden "message-only window" to receive `WM_INPUT` messages.
- **`RawInputManager`:** Creates the message-only window and registers interest in keyboard raw input.
- **`wnd_proc`:** The window procedure where `WM_INPUT` (and `WM_INPUT_DEVICE_CHANGE`) messages are handled.
- **Concurrency:** The message pump runs on the main daemon thread. Panics within the callback are caught via `catch_unwind` to ensure daemon resilience.

### Device Mapping
Devices are tracked in a `DeviceMap`, which maps Win32 `hDevice` handles to `DeviceInfo` (path and serial number).
- **Serial Extraction:** The daemon extracts serial numbers from device paths (e.g., `\\?\HID#VID_...#...#{...}`) to allow stable configuration across reboots.

## Memory Safety & Resource Management

### RAII & Drop
Resources are managed using Rust's `Drop` trait:
- `RawInputManager` ensures the message window is destroyed and Raw Input unregistration is performed on drop.
- Thread safety is maintained by clearing `GWLP_USERDATA` before window destruction, preventing race conditions in `wnd_proc`.

### Synchronization
`Arc<RwLock>` is used for shared state (e.g., `DeviceMap`). All lock acquisitions use non-panicking patterns (graceful handling of lock poisoning) to prevent daemon crashes.

## Event Injection
Output is handled via the `SendInput` API.
- All injected events are marked as "synthetic" to prevent the daemon from re-processing its own output (though Raw Input typically doesn't receive `SendInput` events by default).

## Comparison with Linux
| Feature | Windows | Linux |
|---------|---------|-------|
| Input   | Raw Input API | evdev |
| Output  | SendInput | uinput |
| Loop    | Win32 Message Pump | `poll()` on FDs |
| Privacy | Non-exclusive (Passive) | Exclusive Grab |
