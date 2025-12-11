# Architecture: Wiring & Device Profiles (Rust to Flutter)

This document describes the architecture of the "Wiring" system in KeyRx, explaining how physical devices are mapped to profiles and how that state is synchronized between the Rust Core and the Flutter UI.

## Core Concept: The "Two Truths"

The central complexity in the KeyRx wiring architecture is the separation between **Runtime State** (Ephemeral) and **Persistent Configuration** (Durable).

### 1. Persistent Configuration (`RuntimeConfig`)
*   **What it is**: The user's desired setup. "I want my Logitech G502 to use the 'Gaming' keymap."
*   **Storage**: Saved to disk (`runtime.json`).
*   **Structure**:
    *   **DeviceSlots**: A list of configured devices (identified by VID:PID or Serial).
    *   **ProfileSlots**: An ordered list of profiles (Wiring + Keymap) for each device.
*   **Role**: Defines *potential* behavior. It exists even if the device is unplugged.

### 2. Runtime State (`DeviceRegistry`)
*   **What it is**: The current physical reality. "A Logitech G502 is plugged in right now."
*   **Storage**: In-memory (RAM), lost on restart.
*   **Structure**:
    *   **DeviceState**: Connection status, global "Remap Enabled" toggle, timestamps.
*   **Role**: Defines *actual* capability. It only exists when hardware is present.

---

## Architecture Diagram

The following block diagram illustrates how the Flutter UI interacts with these two layers via the FFI Bridge.

```mermaid
graph TD
    subgraph "Flutter UI"
        UI_D[Devices Page]
        UI_RC[Run Controls / Dashboard]
    end

    subgraph "Services (Dart)"
        S_RT[RuntimeService]
        S_REG[DeviceRegistryService]
    end

    subgraph "FFI Bridge"
        FFI_RT[runtime_get_config]
        FFI_DEV[device_registry_list]
    end

    subgraph "Rust Core"
        subgraph "Engine"
            PL[Mapping Pipeline]
        end

        subgraph "Storage"
            RC[RuntimeConfig]
        end

        subgraph "Memory"
            DR[DeviceRegistry]
        end
    end

    %% Flows
    UI_D -->|Reads/Writes| S_RT
    UI_RC -->|Reads| S_RT
    UI_RC -->|Reads| S_REG

    S_RT <-->|JSON| FFI_RT
    S_REG <--|JSON| FFI_DEV

    FFI_RT <-->|Access| RC
    FFI_DEV <-->|Access| DR

    %% Engine Interaction
    DR -.->|Notify Connected| PL
    RC -.->|Configures| PL

    note[Merged View: Run Controls combines<br/>Registry (Is it here?) + Config (What does it do?)]
    UI_RC -.-> note
```

---

## Data Flow & State Verification

### 1. The "No Profile" Problem (and Solution)
Historically, `DeviceRegistry` had a simple `profile_id` field. This became obsolete with the introduction of **Multi-Slot Profiles**, where a single device can have multiple active profiles stacked by priority.

**The Fix**: Use a "Merged View" approach.

*   **Flutter UI (`RunControlsPage`)** explicitly fetches **both**:
    1.  `DeviceState` from `DeviceRegistryService` (via Stream).
    2.  `RuntimeConfig` from `RuntimeService` (via Future).

*   **Logic**:
    ```dart
    bool isConnected = deviceState.connected;
    bool remapGloballyEnabled = deviceState.remapEnabled;

    // Find matching config for this physical device
    var config = runtimeConfig.devices.find(d =>
        d.vid == device.vid &&
        d.pid == device.pid &&
        (d.serial == null || d.serial == device.serial)
    );

    // Count active slots
    int activeSlots = config?.slots.where(s => s.active).length ?? 0;

    String status = (remapGloballyEnabled && activeSlots > 0)
        ? "Active ($activeSlots slots)"
        : "No Profile";
    ```

### 2. The Engine Pipeline
In the Rust Core, the `MappingPipeline` performs this same merge continuously for every keystroke:

1.  **Event In**: USB Event from `evdev`.
2.  **Identify**: Extract VID/PID/Serial.
3.  **Check Registry**: Is `remap_enabled` true in `DeviceRegistry`? If No -> Passthrough.
4.  **Check Config**: Look up `RuntimeConfig` for this identity.
5.  **Iterate Slots**: Go through `DeviceSlots` in priority order.
6.  **Execute**: First active slot that handles the key wins.

## Key Takeaways

1.  **Separation of Concerns**: The *Registry* knows about **Hardware**. The *Config* knows about **Software** (Logic).
2.  **Loose Coupling**: A configuration for a device can exist before you ever buy the device.
3.  **Strict Serial Matching**: Configuration can target a specific serial number (for distinguishing two identical keyboards) or generic VID:PID (for any keyboard of that model). `DeviceState` *always* has the specific serial number.
