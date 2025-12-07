# Tasks Document

## Phase 1: Rust Core Refactor

- [x] 1. Define Core Structs
  - File: `core/src/config/models.rs` (create/update)
  - Define `VirtualLayout`, `HardwareProfile`, `Keymap`, `RuntimeConfig`, and `ProfileSlot` structs.
  - Implement serialization/deserialization (Serde).
  - _Requirements: 1.1, 1.2, 1.3, 6.2, 7.1_
  - _Prompt: Role: Rust Backend Developer | Task: Define `VirtualLayout`, `HardwareProfile`, and `Keymap` structs. Additionally, define `RuntimeConfig` and `ProfileSlot` to support 1:N device-to-profile mapping with priority. | Context: Replacing old `DeviceProfile` and `Profile`. | Success: Structs defined and compilable._

- [x] 2. Update Engine Pipeline
  - File: `core/src/engine/pipeline.rs` (or similar)
  - Refactor event processing to use the 3-stage pipeline with multi-device concurrency.
  - Implement priority resolution: Iterate through active slots for the device; first match consumes event.
  - _Requirements: 1.4, 6.1, 7.2_
  - _Prompt: Role: Rust System Engineer | Task: Refactor event loop. For each hardware event, look up the device's active `ProfileSlot`s. Iterate top-down (priority). Translate Scancode -> VirtualKey. If mapped, process via Keymap -> Action. | Context: Decoupling hardware from logic with priority support. | Success: Pipeline compiles and logically follows the priority stages._

- [x] 3. Update File Storage Logic
  - File: `core/src/config/manager.rs`
  - Update config manager to load/save from `layouts/`, `hardware/`, and `keymaps/` directories.
  - _Requirements: 1.5_
  - _Prompt: Role: Rust Developer | Task: Update configuration management to read/write JSON files in `~/.config/keyrx/layouts`, `hardware`, and `keymaps`. | Success: File I/O handles the new directory structure._

- [x] 4. Update FFI Exports
  - File: `core/src/ffi/exports.rs`
  - Expose new structs and management functions (including `RuntimeConfig` manipulation) to Dart via FFI.
  - _Requirements: Non-functional (Modularity)_
  - _Prompt: Role: Rust/FFI Engineer | Task: Update `extern "C"` functions. Expose `VirtualLayout`, `HardwareProfile`, `Keymap` CRUD. Expose `RuntimeConfig` methods: `add_slot`, `remove_slot`, `reorder_slot`, `set_active`. | Success: FFI functions match the new core API._

## Phase 2: Dart Model & Service Refactor

- [x] 5. Generate FFI Bindings
  - File: `ui/lib/ffi/generated/bindings.dart`
  - Run `flutter pub run ffi_gen` or manual update to reflect Rust FFI changes.
  - _Requirements: Non-functional_
  - _Prompt: Role: Flutter/Rust Bridge Engineer | Task: Regenerate or manually update Dart FFI bindings to match the new Rust exports. | Success: Dart code can call the new Rust functions._

- [ ] 6. Update Dart Models
  - File: `ui/lib/models/`
  - Create/Update Dart classes mirroring the Rust structs (`ProfileSlot`, `RuntimeConfig`, etc).
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Role: Flutter Developer | Task: Create `VirtualLayout`, `HardwareProfile`, `Keymap`, and `RuntimeConfig` data classes in Dart. | Success: Models match Rust structs._

- [ ] 7. Update Dart Services
  - File: `ui/lib/services/`
  - Update `HardwareService`, `LayoutService`, `KeymapService` to use the new FFI calls.
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Role: Flutter Developer | Task: Implement/Refactor services to manage the CRUD operations for the new models via FFI. | Success: Services correctly bridge UI and Core._

## Phase 3: UI Implementation

- [ ] 8. Implement Layouts Tab
  - File: `ui/lib/pages/layouts.dart`
  - Create UI for listing and creating/editing Virtual Layouts.
  - _Requirements: 2.1, 2.2, 5.1_
  - _Prompt: Role: Flutter UI Developer | Task: Create `LayoutsPage` with a list of layouts and an editor for defining keys (Matrix grid or named keys). | Success: User can create and save a Virtual Layout._

- [ ] 9. Implement Wiring Tab
  - File: `ui/lib/pages/wiring.dart`
  - Create UI for wiring Physical Keys to Virtual Layouts.
  - Interaction: QMK-style (Physical Source Top, Virtual Target Bottom).
  - _Requirements: 3.1, 3.2, 5.1_
  - _Prompt: Role: Flutter UI Developer | Task: Create `WiringPage`. Display Physical Keys (Source) on top, Virtual Layout (Target) on bottom. Click source -> Click target to assign. No physical key press required. | Success: User can wire a physical device to a layout._

- [ ] 10. Implement Mapping Tab
  - File: `ui/lib/pages/mapping.dart`
  - Update the existing editor to map Virtual Keys to Actions.
  - Interaction: QMK-style (Virtual Source Top, Action Palette Bottom).
  - _Requirements: 4.1, 4.2, 5.1_
  - _Prompt: Role: Flutter UI Developer | Task: Refactor `MappingPage`. Display Virtual Layout (Source) on top, Action Palette (Standard Keys/Macros) on bottom. Click source -> Click action to map. | Success: User can map virtual keys to actions._

- [ ] 11. Update Main Navigation
  - File: `ui/lib/main.dart`
  - Implement the 4-tab navigation structure (Devices, Layouts, Wiring, Mapping).
  - _Requirements: 5.1_
  - _Prompt: Role: Flutter UI Developer | Task: Update the main application scaffold to use a 4-item left navigation rail. | Success: Navigation works correctly._

- [ ] 12. Implement Devices Tab
  - File: `ui/lib/pages/devices.dart`
  - Create UI for managing connected devices and their Profile Slots.
  - Features: Add/Remove slots, Toggle Active, Reorder (Up/Down), Select Wiring/Keymap.
  - _Requirements: 6.3, 7.3, 7.4_
  - _Prompt: Role: Flutter UI Developer | Task: Create `DevicesPage`. List connected devices. For each device, allow adding/removing "Profile Slots". Each slot has an active toggle, Wiring selector, Keymap selector, and Reorder controls. | Success: User can configure 1:N profiles per device._
