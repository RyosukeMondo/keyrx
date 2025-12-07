# Profile & Mapping Architecture Refinement

## Objective
Refactor the application architecture (Core + UI) to clearly distinguish between **Hardware Wiring** (Physical Keys -> Virtual Grid) and **Logical Mapping** (Virtual Keys -> Logical Actions), introducing a reusable **Virtual Layout** layer that supports both **Matrix** (Advanced) and **Semantic** (Beginner) modes.

## 1. Terminology & Renaming
We will perform a global refactor to align class names with their specific responsibilities.

| Current Name | **New Name** | Responsibility | Flow |
| :--- | :--- | :--- | :--- |
| (New) | **`VirtualLayout`** | Defines the virtual key set (Grid or Named). | `Rows`x`Cols` OR `Named Keys` (e.g., "KEY_A") |
| `DeviceProfile` | **`HardwareProfile`** | Defines **Hardware Wiring**. | `Physical Key` (Scancode) -> `Virtual Key` (ID) |
| `Profile` | **`Keymap`** | Defines **Logical Mapping**. | `Virtual Key` (ID) -> `Logical Action` (Keycode/Macro) |

**Rationale:**
- **`VirtualLayout`**: Decouples the layout definition from the device.
    - **Matrix Mode**: IDs are coordinates like `r0c0`, `r0c1`. (Great for hand-wired/custom matrix devices).
    - **Semantic Mode**: IDs are meaningful names like `KEY_A`, `KEY_ENTER`. (Great for standard keyboards/beginners).
- **`HardwareProfile`**: Explicitly describes the wiring of physical hardware sensors to the Virtual Layout.
- **`Keymap`**: Standard terminology for the logical layer.

## 2. Architecture Layers (Data Flow)

The system transforms input in 3 stages:

1.  **Physical Layer** (Hardware)
    *   Input: `Scancode` (e.g., `0x04` for 'A')
    *   *Mapped via `HardwareProfile`*
2.  **Virtual Layer** (Abstraction)
    *   Intermediate: `Virtual Key ID`
    *   **Advanced**: `r2c1` (Matrix Position)
    *   **Beginner**: `KEY_A` (Semantic Name)
    *   Defined by `VirtualLayout`
    *   *Mapped via `Keymap`*
3.  **Logical Layer** (Host)
    *   Output: `Action` (e.g., `Key A`, `Macro 1`, `Launch App`)

### Beginner Workflow (Semantic)
*   **Virtual Layout**: User selects "Standard ANSI 104".
*   **Hardware Profile**: Auto-wired! Physical Scancode `0x04` ('A') maps to Virtual `KEY_A`.
*   **Keymap**: User maps Virtual `KEY_A` -> `Action`.
*   *Benefit*: Users don't see "Row/Col". They just see their keys.

### Advanced Workflow (Matrix)
*   **Virtual Layout**: User defines "4x4 Stream Deck".
*   **Hardware Profile**: User wires a "500yen NumPad" (Physical) to the "4x4 Stream Deck" (Virtual).
    *   Physical Numpad '7' -> Virtual `r0c0`.
*   **Keymap**: User maps Virtual `r0c0` -> `OBS Scene Switch`.
*   *Benefit*: Complete freedom to remap any hardware to any virtual concept.

## 3. UI Structure (4 Tabs)

### Tab 1: Devices
*   **Purpose**: Manage hardware connections and status.

### Tab 2: Layouts
*   **Purpose**: Create and edit reusable Virtual Layouts.
*   **Features**:
    *   **Presets**: "Standard ANSI", "ISO", "Numpad", "Stream Deck".
    *   **Custom**: Define Rows x Cols (Matrix) or Custom Named Keys.

### Tab 3: Profiles (Wiring)
*   **Purpose**: Wire a specific Device to a Virtual Layout.
*   **UI Layout**:
    *   **Top**: Typical Key Layout (Physical ANSI/ISO Preset).
    *   **Bottom**: Virtual Layout Palette.
        *   If Matrix: Grid of `R/C` buttons.
        *   If Semantic: Grid of `KEY_A` buttons (or Visual Representation).
*   **Presets**: Use **Keyboard Layout Editor (KLE) JSON** for the Top "Typical Layout" visualization.

### Tab 4: Mapping (Logic)
*   **Purpose**: Map a Virtual Layout to Logical Actions.
*   **UI Layout**:
    *   **Top**: Virtual Layout Visualizer.
    *   **Bottom**: Logical Action Palette.

## 4. Implementation Plan

### Phase 1: Core Refactor (Rust)
1.  Rename `DeviceProfile` struct to `HardwareProfile`.
2.  Rename `Profile` struct to `Keymap`.
3.  Introduce `VirtualLayout` struct (supporting both Index/Coordinate and String IDs).
4.  Update FFI bridges.

### Phase 2: UI Model Refactor (Dart)
1.  Rename Dart models (`HardwareProfile`, `Keymap`).
2.  Update services (`DeviceProfileService` -> `HardwareProfileService`, `ProfileRegistryService` -> `KeymapService`).

### Phase 3: UI Page Implementation
1.  **Layouts Page**: New page for `VirtualLayout` CRUD.
2.  **Profiles Page**: Update `DeviceWiringPage` to work with `HardwareProfile`.
3.  **Mapping Page**: Update to work with `Keymap`.
