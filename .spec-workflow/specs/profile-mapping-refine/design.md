# Design Document

## Overview

This design implements a 3-stage input processing pipeline that decouples physical hardware from logical configuration using an intermediate "Virtual Layout" layer. This refactoring aligns the system with the "True Blank Canvas" and "Progressive Complexity" product pillars.

## Steering Document Alignment

### Technical Standards (tech.md)
- Adheres to the 3-Layer Hybrid Architecture (Rust Core -> FFI -> Flutter UI).
- Implements the "Dual-Mode Key Identification" concept via `VirtualLayout` types (Matrix vs Semantic).
- Uses the specified dependency injection pattern for `HardwareProfileService` and `KeymapService`.

### Project Structure (structure.md)
- Introduces new directories in `~/.config/keyrx/`: `layouts/`, `hardware/`, `keymaps/`.
- Updates `ui/lib/pages/` with `layouts.dart`, `wiring.dart`, `mapping.dart`.
- Renames core structs in `core/src/config/`.

## Architecture

### The 3-Stage Pipeline

```mermaid
graph TD
    A[Physical Input (Scancode)] -->|HardwareProfile| B[Virtual Key (ID)]
    B -->|Keymap| C[Logical Action]

    subgraph "Stage 1: Hardware Wiring"
    A
    end

    subgraph "Stage 2: Virtual Layout"
    B
    end

    subgraph "Stage 3: Logical Mapping"
    C
    end
```

### Data Flow
1.  **Driver Layer**: Receives OS/Hardware event (scancode).
2.  **Engine Layer**:
    *   Looks up `HardwareProfile` for current device.
    *   Translates `Scancode` -> `VirtualKeyID` (e.g., `0x04` -> `"KEY_A"`).
    *   Looks up active `Keymap` for the `VirtualLayout`.
    *   Translates `VirtualKeyID` -> `Action` (e.g., `"KEY_A"` -> `Macro::Paste`).
3.  **Injector Layer**: Executes the `Action`.

### Multi-Device Concurrency
The Engine maintains a runtime registry of connected devices. Each device instance (unique by VID/PID/Serial) can have **multiple active assignments**.
For each physical device, the user can configure a list of active **Profile Slots**. Each slot contains:
1.  **Hardware Profile**: Defines which subset of physical keys (scancodes) are mapped to a Virtual Layout.
2.  **Keymap**: Defines the logical behavior for that Virtual Layout.
3.  **Active State**: Toggle to enable/disable this specific slot.

This allows a single physical device (e.g., a large keyboard) to be logically split.
**Priority & Conflict Resolution**:
- Slots are processed in **top-down order**.
- If multiple active slots map the same physical scancode, the **highest priority (topmost)** slot consumes the event.
- This ensures predictable behavior when layouts overlap.

## Components and Interfaces

### Rust Core Components

#### `VirtualLayoutRegistry`
- **Purpose**: Manages available virtual layouts.
- **Interfaces**: `get_layout(id)`, `list_layouts()`, `create_layout()`.

#### `HardwareProfileManager` (Renamed from DeviceProfile)
- **Purpose**: Manages hardware wiring configs.
- **Interfaces**: `get_profile(vendor, product)`, `save_profile()`.

#### `KeymapEngine` (Renamed from Profile)
- **Purpose**: Handles logical mapping state.
- **Interfaces**: `remap(virtual_key, action)`, `get_action(virtual_key)`.

#### `RuntimeConfig` (New)
- **Purpose**: Manages active state and assignments for connected devices.
- **Interfaces**: `get_device_slots(device_id)`, `add_slot(device_id)`, `remove_slot(device_id)`, `set_slot_active(slot_id, bool)`, `update_slot(slot_id, profile_id, keymap_id)`, `reorder_slot(device_id, old_index, new_index)`.

### Flutter UI Components

#### `LayoutEditor` (New)
- **Purpose**: UI for creating custom grids or semantic layouts.
- **Location**: `ui/lib/pages/layouts.dart`.

#### `WiringEditor` (Refactored DeviceWiringPage)
- **Purpose**: UI for pressing physical keys to assign them to the Virtual Layout.
- **Location**: `ui/lib/pages/wiring.dart`.
- **Interaction**: Listens to "Raw Input" from FFI to detect pressed scancodes, highlights corresponding Virtual Key.

#### `MappingEditor` (Refactored VisualEditorPage)
- **Purpose**: UI for assigning actions to Virtual Keys.
- **Location**: `ui/lib/pages/mapping.dart`.
- **Interaction**: Purely logical; does not require physical device connection to edit.

## Current UI State

### General Layout (Left Navigation Rail)
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │  DEVICES                                         │
│  Profiles          │  [Refresh]                                       │
│  Mapping           │  ┌────────────────────────────────────────────┐  │
│  Run               │  │  Logitech K270                             │  │
│  Metrics           │  │  ID: 046d:c52b                             │  │
│  Calibration       │  │  Profile: [ Standard ANSI ]                │  │
│  Debugger          │  └────────────────────────────────────────────┘  │
│  Console           │                                                  │
│  Timing            │                                                  │
└────────────────────┴──────────────────────────────────────────────────┘
```

## Proposed UI Wireframes

### Navigation Structure
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │                                                  │
│  Layouts           │  [ Main Content Area ]                           │
│  Wiring            │                                                  │
│  Mapping           │                                                  │
│  ----------------  │                                                  │
│  Run               │                                                  │
│  Metrics           │                                                  │
│  Calibration       │                                                  │
│  Debugger          │                                                  │
│  Console           │                                                  │
└────────────────────┴──────────────────────────────────────────────────┘
```

### Page 1: Devices
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │ DEVICES                                          │
│  Layouts           │                                                  │
│  Wiring            │ Connected Devices                                │
│  Mapping           │ ┌────────────────────────────────────────────┐   │
│                    │ │  Stream Deck XL (ID: 0fd9:0060)            │   │
│                    │ │  [ + Add Profile Slot ]                    │   │
│                    │ │                                            │   │
│                    │ │  1. [▲][▼] [X] Active                      │   │
│                    │ │     Wiring: [ Left Half (4x4)           ]  │   │
│                    │ │     Keymap: [ Photoshop Tools           ]  │   │
│                    │ │     [Edit Wiring] [Edit Keymap] [Remove]   │   │
│                    │ │                                            │   │
│                    │ │  2. [▲][▼] [ ] Active                      │   │
│                    │ │     Wiring: [ Right Half (4x4)          ]  │   │
│                    │ │     Keymap: [ Streaming Controls        ]  │   │
│                    │ │     ⚠️ Conflict: Overlaps with Slot 1       │   │
│                    │ │     [Edit Wiring] [Edit Keymap] [Remove]   │   │
│                    │ └────────────────────────────────────────────┘   │
└────────────────────┴──────────────────────────────────────────────────┘
```

### Page 2: Layouts (New)
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │ LAYOUTS                                          │
│  Layouts           │                                                  │
│  Wiring            │ My Layouts           │ Editing: Stream Deck 4x4  │
│  Mapping           │ [ + New Layout ]     │ Type: Matrix              │
│                    │                      │                           │
│                    │ • Standard ANSI      │ ┌───┐ ┌───┐ ┌───┐ ┌───┐   │
│                    │ • ISO 105            │ │0,0│ │0,1│ │0,2│ │0,3│   │
│                    │ • Stream Deck 4x4    │ └───┘ └───┘ └───┘ └───┘   │
└────────────────────┴──────────────────────────────────────────────────┘
```

### Page 3: Wiring
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │ WIRING (Physical -> Virtual)                     │
│  Layouts           │                                                  │
│  Wiring            │ Target Profile: Stream Deck 4x4 (Default)        │
│  Mapping           │ Virtual Layout: Stream Deck 4x4                  │
│                    │                                                  │
│                    │ 1. SOURCE: Physical Scancodes (Standard Layout)  │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐ [ Search Scancode... ]   │
│                    │ │ESC│ │F1 │ │F2 │ │F3 │                          │
│                    │ └───┘ └───┘ └───┘ └───┘                          │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐                          │
│                    │ │ Q │ │ W │ │ E │ │ R │  <-- Click to Select     │
│                    │ └───┘ └───┘ └───┘ └───┘      (Scancode 0x14)     │
│                    │                                                  │
│                    │ 2. TARGET: Virtual Layout (Stream Deck 4x4)      │
│                    │    Mode: Matrix (Row/Col)                        │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐                          │
│                    │ │0,0│ │0,1│ │0,2│ │0,3│  <-- Click to Assign     │
│                    │ └───┘ └───┘ └───┘ └───┘      0x14 -> R0C3        │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐                          │
│                    │ │1,0│ │1,1│ │1,2│ │1,3│                          │
│                    │ └───┘ └───┘ └───┘ └───┘                          │
└────────────────────┴──────────────────────────────────────────────────┘
```

### Page 4: Mapping
```
┌────────────────────┬──────────────────────────────────────────────────┐
│  Devices           │ MAPPING (Virtual -> Action)                      │
│  Layouts           │                                                  │
│  Wiring            │ Target Keymap: Photoshop Tools (Left)            │
│  Mapping           │ Virtual Layout: Stream Deck 4x4                  │
│                    │                                                  │
│                    │ 1. SOURCE: Virtual Layout (Stream Deck 4x4)      │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐ [ Layer: Default ]       │
│                    │ │0,0│ │0,1│ │0,2│ │0,3│                          │
│                    │ └───┘ └───┘ └───┘ └───┘                          │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐                          │
│                    │ │1,0│ │1,1│ │1,2│ │1,3│ <-- Selected (R1C3)      │
│                    │ └───┘ └───┘ └───┘ └───┘                          │
│                    │                                                  │
│                    │ 2. ACTION PALETTE: Standard Keys & Macros        │
│                    │ [ Basic ] [ Media ] [ Layers ] [ Macros ]        │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐ [ Search Action... ]     │
│                    │ │ A │ │ B │ │ C │ │ D │                          │
│                    │ └───┘ └───┘ └───┘ └───┘                          │
│                    │ ┌───┐ ┌───┐ ┌───┐ ┌───┐                          │
│                    │ │ 1 │ │ 2 │ │ 3 │ │ 4 │ <-- Click to Map         │
│                    │ └───┘ └───┘ └───┘ └───┘     R1C3 = Output '4'    │
└────────────────────┴──────────────────────────────────────────────────┘
```

## Data Models

### VirtualLayout
```rust
pub struct VirtualLayout {
    pub id: String,
    pub name: String,
    pub layout_type: LayoutType, // Matrix | Semantic
    pub keys: Vec<VirtualKeyDef>,
}

pub struct VirtualKeyDef {
    pub id: String,       // "r0c0" or "KEY_A"
    pub label: String,    // "Row 0 Col 0" or "A"
    pub position: (f32, f32), // Visual coordinates for UI
    pub size: (f32, f32),     // Visual size
}
```

### HardwareProfile
```rust
pub struct HardwareProfile {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub virtual_layout_id: String,
    pub wiring: HashMap<u16, String>, // Scancode -> VirtualKeyID
}
```

### Keymap
```rust
pub struct Keymap {
    pub id: String,
    pub name: String,
    pub virtual_layout_id: String,
    pub layers: Vec<Layer>, // Rhai script or JSON model
}
```

## Migration Strategy

### Breaking Changes
- Old `DeviceProfile` JSONs will be incompatible.
- Old `Profile` concepts need to be migrated.
- **Strategy**:
    1.  On startup, detect old profiles.
    2.  Run a migration utility to convert them to "Semantic Mode" `HardwareProfiles` + `Keymaps`.
    3.  Archive old files to `~/.config/keyrx/legacy_backup/`.

## Testing Strategy

### Unit Testing
- Test pipeline transformation: `Scancode` -> `VirtualKey` -> `Action`.
- Test serialization/deserialization of new structs.

### Integration Testing
- Verify FFI bridge correctly exposes the 3 stages to Dart.
- Verify `WiringEditor` receives raw scancodes via FFI stream.
