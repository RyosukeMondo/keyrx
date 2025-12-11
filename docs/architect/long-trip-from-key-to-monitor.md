# Architecture: Long Trip from Key to Monitor

This document traces the complete journey of a key press event, from the physical hardware interrupt to the pixel changing on your screen in the Flutter Dashboard.

## The Concept

The core philosophy of KeyRx is: **"The Engine matches Hardware to Software via Slots."**

*   **Hardware**: Physical Reality (Scancodes).
*   **Software**: Logic (Rhia Scripts/Keymaps).
*   **Slots**: The Bridge (Wiring).

## The Journey

```mermaid
sequenceDiagram
    participant HW as Physical Keyboard
    participant OS as Windows OS
    participant R_CORE as Rust Core (evdev)
    participant R_ENG as Rust Engine (Mapping)
    participant R_RT as Rust Runtime (Slots)
    participant FFI as FFI Bridge
    participant D_STREAM as Dart Stream
    participant UI as Flutter Monitor

    Note over HW, OS: 1. Physical Event
    HW->>OS: HID Report (Scancode 0x14 "Q")
    OS->>R_CORE: Intercepted via Hooks

    Note over R_CORE, R_ENG: 2. Core Processing
    R_CORE->>R_CORE: Normalize to Evdev Key 0x10 ("Q")
    R_CORE->>R_ENG: Ingest(DeviceID, KeyCode, Value)

    Note over R_ENG, R_RT: 3. The Decision (The "Trip")
    R_ENG->>R_RT: Query: "Who owns DeviceID?"
    R_RT-->>R_ENG: "Slot 1: Hardware 'Compact' + Keymap 'Gaming'"

    R_ENG->>R_ENG: Apply Hardware Profile (Scancode -> Logical Key)
    R_ENG->>R_ENG: Apply Keymap (Logical Key -> Mapped Output 'B')

    Note over R_ENG, FFI: 4. Reporting
    R_ENG->>FFI: Emit Event { type: "Key", code: "B", source: "Mapped" }

    Note over FFI, UI: 5. Visualization (The Return)
    FFI->>D_STREAM: Stream Update (JSON)
    D_STREAM->>UI: Decode & Render
    UI->>UI: Highlight "B" on Visualizer
```

## Detailed Flow Analysis

### 1. Physical Input (The Source)
*   **Mechanism**: Windows `SetWindowsHookEx` (LL) or `RawInput`.
*   **Identity**: The keystroke comes tagged with a Device Handle.
*   **Resolution**: Rust converts this Handle to a `DeviceInstanceId` (VID:PID:Serial).

### 2. The Slot Lookup (The "Wiring")
This is where the magic (or the bug) happens. The Engine asks the `RuntimeConfig`:
*   *Input*: `DeviceInstanceId` ("My Keyboard")
*   *Question*: "What active slots do I have?"
*   *Expected Answer*: "You have Slot #1 active."

**Critical Data Integrity**:
*   If `RuntimeConfig` has a malformed Slot ID, or if the `DeviceInstanceId` doesn't match exactly (e.g. Serial mismatch), the Engine returns **No Slots**.
*   **Fallback**: If No Slots found, the Engine defaults to **Passthrough** (Original Key).

### 3. The Transformation
If a Slot is found:
1.  **Hardware Profile**: Translates Physical Scancode (e.g., location 16) to Logical ID (e.g., "Left Shift").
2.  **Keymap/Script**: Translates Logical ID to Output (e.g., "Meta + C").

### 4. The Loopback (Monitor)
The data displayed in the "Monitor" tab is the **Output** of the engine.
*   It does *not* show raw input (unless passthrough occurred).
*   If you see "Q" instead of "B", it means the Engine performed "Passthrough".
*   **Why Passthrough?** Because Step 2 (Slot Lookup) likely failed.

## Failure Diagnosis Checklist

If you see Raw Input ("Q") instead of Mapped Input ("B"), one of these links is broken:

1.  **Identity Mismatch**: Rust sees `Serial: A`, Config has `Serial: B`.
2.  **Slot Inactive**: The slot exists but `active: false`.
3.  **Malformed Config**: The slot data looks valid to JSON but nonsense to logic (e.g., `keymap_id: "default"` when you meant `"gaming_layer"`).

### Case Study: The "Double-Encoded" Bug
In your current `runtime.json`, we see:
```json
"id": "{\"id\":\"...\",\"keymap_id\":\"real_map\"...}",
"keymap_id": "default"
```
The Engine reads `keymap_id` -> "default".
"Default" maps to Empty/Passthrough.
Result: **Q -> Q**.

The "Real" keymap ID ("real_map") is trapping inside the `id` string, invisible to the Engine.
