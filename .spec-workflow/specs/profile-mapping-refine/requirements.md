# Requirements Document

## Introduction

Refactor the application architecture (Core + UI) to clearly distinguish between **Hardware Wiring** (Physical Keys -> Virtual Grid) and **Logical Mapping** (Virtual Keys -> Logical Actions). This introduces a reusable **Virtual Layout** layer that supports both **Matrix** (Advanced) and **Semantic** (Beginner) modes, decoupling physical devices from logical configurations.

## Alignment with Product Vision

This feature directly supports the **"True Blank Canvas"** foundational pillar by removing OS assumptions and allowing true hardware-level abstraction. It also enables **"Progressive Complexity"** by offering both Semantic Mode (simple for beginners) and Matrix Mode (powerful for advanced users).

## Requirements

### Requirement 1: Core Architecture Refactor

**User Story:** As a developer, I need the core data structures to reflect the 3-stage pipeline (Physical -> Virtual -> Logical) so that I can implement layout-agnostic features.

#### Acceptance Criteria

1.  **Renaming**: `DeviceProfile` SHALL be renamed/refactored to `HardwareProfile`.
2.  **Renaming**: `Profile` SHALL be renamed/refactored to `Keymap`.
3.  **New Struct**: A `VirtualLayout` struct SHALL be introduced to define the available virtual keys.
4.  **Pipeline**: The event processing pipeline SHALL transform `Scancode` -> `VirtualKey` -> `Action`.
5.  **Storage**: Configuration files SHALL be organized into `layouts/`, `hardware/`, and `keymaps/`.

### Requirement 2: Virtual Layout Support

**User Story:** As a user, I want to define a "Virtual Layout" (e.g., "4x4 Grid" or "Standard ANSI") so that I can reuse my logical keymaps across different physical devices.

#### Acceptance Criteria

1.  **Layout Types**: The system SHALL support "Matrix" (Row/Col based) and "Semantic" (Name based) layout types.
2.  **Definition**: Users (or defaults) CAN define the geometry and key identifiers of a layout.
3.  **Reuse**: Multiple Hardware Profiles CAN point to the same Virtual Layout.
4.  **Reuse**: Multiple Keymaps CAN point to the same Virtual Layout.

### Requirement 3: Hardware Wiring (Hardware Profile)

**User Story:** As a user, I want to "wire" my physical keyboard switches to a Virtual Layout so that the system knows which physical key corresponds to which virtual position.

#### Acceptance Criteria

1.  **Discovery**: When a new device is detected, the system SHALL prompt to create a `HardwareProfile`.
2.  **Wiring Process**: Users CAN interactively press physical keys to assign them to Virtual Keys.
3.  **Auto-Wire**: For standard layouts, the system SHOULD offer an "Auto-Wire" capability based on standard scancodes.
4.  **Multi-Wiring**: A single physical device (identified by Serial/VID/PID) CAN have multiple `HardwareProfiles` associated with it (e.g., mapping keys 1-4 to a "Macro Pad" layout and keys 5-8 to a "Numpad" layout).

### Requirement 4: Logical Mapping (Keymap)

**User Story:** As a user, I want to map my Virtual Keys to logical actions (e.g., "A", "Ctrl+C", "Macro") so that I can customize my input behavior.

#### Acceptance Criteria

1.  **Abstraction**: The mapping UI SHALL operate on `VirtualKey` IDs, not physical scancodes.
2.  **Independence**: Keymaps SHALL be independent of the physical device connected, relying only on the `VirtualLayout`.

### Requirement 5: UI Overhaul (4 Tabs)

**User Story:** As a user, I want a clear separation of concerns in the UI so that I understand the difference between connecting a device, defining a layout, wiring it, and mapping it.

#### Acceptance Criteria

1.  **Tab Structure**: The main UI SHALL consist of 4 main tabs:
    *   **Devices**: Connection status and management.
    *   **Layouts**: Creation/Editing of Virtual Layouts.
    *   **Wiring**: Wiring Physical Keys to Virtual Layouts.
    *   **Mapping**: Mapping Virtual Keys to Actions.
2.  **Visualizers**:
    *   **Wiring Tab**: Visualizes physical layout vs virtual grid.
    *   **Mapping Tab**: Visualizes virtual layout with assigned actions.

### Requirement 6: Multi-Device Support

**User Story:** As a user, I want to use multiple devices simultaneously (e.g., two macro pads), each with its own logical keymap, so that I can expand my input capabilities.

#### Acceptance Criteria

1.  **Concurrency**: The system SHALL support input from multiple active devices simultaneously.
2.  **Configuration**: Each connected device instance SHALL be independently assignable to:
    *   A **Hardware Profile** (Wiring).
    *   A **Keymap** (Logical Mapping).
3.  **Activation**: Users CAN toggle specific devices "ON" or "OFF" (active/inactive) in the Devices tab.
4.  **Distinct Mapping**: Two identical physical devices CAN be mapped to different Virtual Layouts or different Keymaps.
5.  **Multi-Profile Per Device**: Users CAN assign multiple Hardware Profiles to a single physical device (e.g., splitting a keyboard into two logical pads), and toggle each profile independently.

### Requirement 7: Conflict Resolution & Priority

**User Story:** As a user, when I have overlapping active profiles (e.g., two profiles mapping the same physical key), I want to control which one takes precedence so that the system behaves predictably.

#### Acceptance Criteria

1.  **Priority Order**: The system SHALL respect a user-defined priority order for active Profile Slots.
2.  **Resolution**: If multiple active slots map the same physical scancode, the slot with the HIGHER priority SHALL consume the event.
3.  **Reordering**: Users CAN reorder Profile Slots (move up/down) in the UI to change their priority.
4.  **Conflict Detection**: The UI SHOULD warn the user if two active slots on the same device have overlapping physical key assignments.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Separation of Concerns**: Core logic must be strictly separated from UI via FFI.
- **Backwards Compatibility**: Migration path for existing profiles (if any) or clear reset strategy.

### Usability
- **Terminology**: New terms (Hardware Profile, Keymap, Virtual Layout) must be consistently used in the UI.
- **Onboarding**: The "Device Discovery Flow" must guide the user through the 3 stages without overwhelming them.
