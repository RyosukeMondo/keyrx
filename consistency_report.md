# Consistency Report: Rust Core vs Flutter UI

This report details the findings from a systematic consistency check between the Rust Core (FFI & Daemon) and the Flutter UI (FFI Bridge & IPC Client).

## Architecture Overview

The system uses a dual-layered approach:
1.  **FFI (Foreign Function Interface):** Used for in-process (Embedded) mode.
    *   **Rust:** `core/src/ffi/` (Exports)
    *   **Dart:** `ui/lib/ffi/` (Bridge)
    *   **Contracts:** `core/src/ffi/contracts/*.json`
2.  **IPC (Inter-Process Communication):** Used for Daemon mode (`keyrx daemon`).
    *   **Rust:** `core/src/daemon/` (JSON-RPC over Unix Socket)
    *   **Dart:** `ui/lib/services/daemon_service.dart` (Client)
    *   **Protocol:** `core/src/daemon/protocol.rs` vs `ui/lib/models/daemon_protocol.dart`

## Fixed Inconsistencies

The following discrepancies were identified and immediately fixed:

### 1. Missing `DecisionEvent` in Dart (IPC)
*   **Issue:** The Rust daemon broadcasts `event.decision` (showing how the engine remapped a key), but the Dart client had no `DecisionEvent` class and no handler for this notification. This prevented the UI from visualizing remapping logic in real-time.
*   **Fix:** Added `DecisionEvent` class and `DaemonEvent.decision` variant to `ui/lib/models/daemon_protocol.dart`.
*   **Fix:** Added `DaemonNotifications.eventDecision` constant and wired it in `DaemonEvent.fromNotification`.

### 2. Incorrect FFI Contract Signature
*   **Issue:** `core/src/ffi/contracts/engine.ffi-contract.json` specified that `start_loop` takes a `profile_id` parameter. However, both the Rust implementation (`keyrx_engine_start_loop` in `core/src/ffi/exports_engine.rs`) and the Dart bridge (`ui/lib/ffi/bridge.dart`) implement it as taking **no arguments**.
*   **Fix:** Updated `engine.ffi-contract.json` to remove the `profile_id` parameter, aligning the specification with the actual implementation.

## Open Bugs / Feature Gaps

The following issues require further attention:

### 1. Daemon Discovery Not Implemented
*   **Severity:** High
*   **Issue:** The Flutter UI sends `discovery.start` via JSON-RPC when scanning for devices in Daemon mode. The Rust Daemon (`core/src/daemon/ipc.rs`) **does not handle this method**.
*   **Impact:** Device scanning in Daemon mode will fail silently or timeout.
*   **Recommendation:** Implement `discovery.start` and `discovery.stop` handlers in `core/src/daemon/ipc.rs`, leveraging the `drivers` module to list devices.

### 2. Hardcoded Modifiers in Input Events
*   **Severity:** Medium
*   **Issue:** In `core/src/daemon/ipc.rs`, the `event_to_notification` function hardcodes the `modifiers` field to an empty list (`[]`) for `InputEvent`.
*   **Impact:** The UI will never show active modifiers (Ctrl, Shift, etc.) in the event log when running in Daemon mode.
*   **Recommendation:** Populate the `modifiers` field correctly from the engine's state or the raw input event data.

### 3. FFI Discovery "Legacy" Status
*   **Severity:** Low (Clarification needed)
*   **Issue:** `ui/lib/ffi/bridge_discovery.dart` marks methods as "Legacy" and points to `DeviceRegistryService`. The FFI contract `discovery.ffi-contract.json` exists but might be stale.
*   **Recommendation:** Audit the `discovery.ffi-contract.json` and ensure the `DeviceRegistryService` in Dart is correctly wiring up to the `DeviceRegistry` in Rust (via `core/src/ffi/contracts/device_registry.ffi-contract.json`).

## Conclusion

The primary "consistency" breaks were in the IPC protocol (Daemon) and the FFI Contract definitions. The IPC layer fixes (DecisionEvent) should immediately enable better visualization features in the UI. The Discovery gap is a functional bug that needs a dedicated implementation task.
