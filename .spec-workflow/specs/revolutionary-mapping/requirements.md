# Requirements Document - Revolutionary Mapping

## Introduction

The Revolutionary Mapping feature transforms KeyRx from a simple key remapper into a professional-grade Input Management System by decoupling physical devices from logical profiles. This paradigm shift solves the fundamental scalability problem of managing multiple identical devices (e.g., two Stream Decks with different purposes) and enables instant, purpose-specific configuration swapping.

**Purpose:** Enable users to:
- Uniquely identify and manage multiple identical input devices independently
- Create portable, hardware-independent profiles that can be assigned to any compatible device
- Toggle remapping per device without affecting other devices
- Swap device behavior instantly by changing profiles
- Support custom device layouts beyond standard keyboards (5×5 macro pads, split keyboards, Stream Decks)

**Value:** This feature positions KeyRx as the only input management tool that treats devices as individuals and profiles as portable configurations, solving pain points that competitors cannot address.

## Alignment with Product Vision

This feature directly supports KeyRx's product vision:

**"Logic > Configuration"**: Profiles remain scriptable via Rhai while adding device-profile decoupling for true flexibility.

**"True Blank Canvas"**: Extends the existing row-col physical abstraction to support any device layout, not just keyboards.

**"Progressive Complexity"**: Simple users can use pre-made profiles; power users can create complex, portable configurations.

**"CLI First, GUI Later"**: All device and profile management operations will be CLI-accessible before UI implementation.

**New Product Capability**: Transforms KeyRx from a "keyboard remapper" into an "input management system" - a category upgrade that differentiates us from all competitors.

## Requirements

### Requirement 1: Unique Device Identity

**User Story:** As a power user with two identical Stream Decks, I want each device to be uniquely identified by its serial number, so that I can configure them independently without conflicts.

#### Acceptance Criteria

1. WHEN a device connects to the system THEN KeyRx SHALL extract a unique identifier comprising (vendor_id, product_id, serial_number)
2. IF a device has a hardware serial number (USB iSerial descriptor) THEN KeyRx SHALL use it as the unique identifier
3. IF a device lacks a hardware serial number THEN KeyRx SHALL generate a stable synthetic identifier based on USB port topology and VID:PID hash
4. WHEN two devices with identical VID:PID but different serials connect THEN KeyRx SHALL track them as separate entities in the device registry
5. WHEN a user assigns a custom label to a device THEN KeyRx SHALL persist this label across sessions and associate it with the device identity

---

### Requirement 2: Device Registry (Runtime State Management)

**User Story:** As a user managing multiple devices, I want to see all connected devices with their current state, so that I can understand which devices are active and what profiles they're using.

#### Acceptance Criteria

1. WHEN a device connects THEN KeyRx SHALL register it in the device registry with initial state (identity, connected_at timestamp, state: Passthrough)
2. WHEN a device disconnects THEN KeyRx SHALL remove it from the active device registry
3. WHEN querying device state THEN KeyRx SHALL return: device identity, is_remapping_enabled flag, active_profile_id (if assigned), runtime state (Active/Passthrough/Failed)
4. WHEN a device state changes (remap toggle, profile assignment) THEN KeyRx SHALL emit a DeviceEvent for UI synchronization
5. IF no devices are connected THEN the device registry SHALL return an empty list without errors

---

### Requirement 3: Profile Registry (Persistent Configuration Storage)

**User Story:** As a user, I want to create and manage profiles independently of any connected device, so that I can design configurations before the hardware arrives or swap profiles between devices.

#### Acceptance Criteria

1. WHEN creating a profile THEN KeyRx SHALL store: profile_id (UUID), name, layout_type (Matrix/Standard/Split), mappings (row,col → KeyAction), created_at, modified_at timestamps
2. WHEN saving a profile THEN KeyRx SHALL persist it to `$XDG_CONFIG_HOME/keyrx/profiles/{profile_id}.json` using atomic write (temp file + rename)
3. WHEN loading profiles on startup THEN KeyRx SHALL read all `.json` files from the profiles directory and build an in-memory registry
4. WHEN deleting a profile THEN KeyRx SHALL remove it from the filesystem and emit a ProfileDeleted event
5. WHEN searching for compatible profiles for a device THEN KeyRx SHALL return only profiles whose layout_type matches the device's layout definition
6. IF a profile file is corrupted THEN KeyRx SHALL log an error, skip that profile, and continue loading other profiles
7. WHEN updating a profile THEN KeyRx SHALL update the modified_at timestamp automatically

---

### Requirement 4: Per-Device Remap Control

**User Story:** As a user with 5 input devices, I want to enable/disable remapping for each device individually, so that I can temporarily use a device in passthrough mode without affecting others.

#### Acceptance Criteria

1. WHEN a device is registered THEN it SHALL default to is_remapping_enabled = false (safe default, requires user opt-in)
2. WHEN a user toggles remapping for a device THEN KeyRx SHALL update the is_remapping_enabled flag and change runtime state (Active ↔ Passthrough)
3. IF is_remapping_enabled is false THEN input from that device SHALL pass through without any processing (bypass all profile mappings)
4. IF is_remapping_enabled is true AND active_profile_id is set THEN input SHALL be processed through the assigned profile
5. IF is_remapping_enabled is true BUT active_profile_id is null THEN the device SHALL remain in Passthrough state
6. WHEN toggling remap state THEN the change SHALL take effect immediately (< 100ms) without requiring restart

---

### Requirement 5: Profile-to-Device Assignment

**User Story:** As a content creator, I want to assign my "OBS Controls" profile to one Stream Deck and my "Photoshop Shortcuts" profile to another identical Stream Deck, so that each device serves a different purpose.

#### Acceptance Criteria

1. WHEN a user assigns a profile to a device THEN KeyRx SHALL validate layout compatibility (device layout matches profile layout_type)
2. IF layouts are incompatible THEN KeyRx SHALL return an error with details (e.g., "Device has 3×5 layout but profile requires 5×5")
3. WHEN a profile is assigned THEN KeyRx SHALL update device_state.active_profile_id and persist the binding to `device_bindings.json`
4. WHEN a device reconnects THEN KeyRx SHALL automatically reload the previously assigned profile from device_bindings.json
5. WHEN unassigning a profile THEN KeyRx SHALL set active_profile_id to null and set device to Passthrough state
6. IF a referenced profile is deleted THEN devices using it SHALL transition to Passthrough state and log a warning

---

### Requirement 6: Device-Profile Bindings Persistence

**User Story:** As a user, I want my profile assignments and device labels to persist across application restarts, so that I don't have to reconfigure my devices every time I restart KeyRx.

#### Acceptance Criteria

1. WHEN a profile is assigned to a device THEN KeyRx SHALL write the binding to `.spec-workflow/keyrx/device_bindings.json` using atomic write
2. WHEN KeyRx starts THEN it SHALL load device_bindings.json and apply bindings to any connected devices
3. IF a device in bindings file is not connected THEN KeyRx SHALL retain the binding for future connection
4. WHEN a user sets a device label THEN it SHALL be stored in device_bindings.json under the device identity
5. IF device_bindings.json is missing THEN KeyRx SHALL create it with an empty bindings array
6. IF device_bindings.json is corrupted THEN KeyRx SHALL create a backup and start with empty bindings, logging an error

---

### Requirement 7: Device Definition Library (Layout Awareness)

**User Story:** As a macro pad owner, I want KeyRx to understand my device's 5×5 grid layout, so that the visual editor displays the correct button arrangement instead of a generic keyboard.

#### Acceptance Criteria

1. WHEN KeyRx starts THEN it SHALL load all device definitions from `device_definitions/**/*.toml` recursively
2. WHEN a device connects THEN KeyRx SHALL look up its definition using (vendor_id, product_id) as the key
3. IF a device definition exists THEN KeyRx SHALL use it to translate scancodes to (row, col) positions
4. IF no device definition exists THEN KeyRx SHALL fall back to a generic ANSI keyboard layout
5. WHEN a device definition defines a matrix_map THEN it SHALL map each scancode/HID usage ID to a (row, col) tuple
6. IF a scancode is not in the matrix_map THEN KeyRx SHALL log a warning and ignore that key press
7. WHEN loading device definitions THEN KeyRx SHALL validate: vendor_id/product_id are non-zero, rows/cols are non-zero, matrix_map contains valid positions

---

### Requirement 8: Serial Number Extraction (Windows)

**User Story:** As a Windows user with two identical keyboards, I want KeyRx to distinguish them using their serial numbers, so that I can assign different profiles to each.

#### Acceptance Criteria

1. WHEN a device connects on Windows THEN KeyRx SHALL extract the device path from Raw Input API (format: `\\?\HID#VID_vvvv&PID_pppp&MI_ii#<InstanceID>#{ClassGUID}`)
2. WHEN parsing the device path THEN KeyRx SHALL extract the InstanceID segment (between second and third `#` delimiters)
3. IF HidD_GetSerialNumberString succeeds THEN KeyRx SHALL use the returned serial number string
4. IF HidD_GetSerialNumberString fails THEN KeyRx SHALL use the InstanceID as the serial (which may be port-based)
5. IF the InstanceID is port-based (e.g., `7&3a2b4c5&0&0000`) THEN KeyRx SHALL warn the user that the device configuration is port-dependent
6. WHEN a port-bound device is moved to a different USB port THEN KeyRx SHALL treat it as a new device (separate entry in device registry)

---

### Requirement 9: Serial Number Extraction (Linux)

**User Story:** As a Linux user, I want KeyRx to extract unique device identifiers using the evdev interface, so that I can manage multiple identical devices independently.

#### Acceptance Criteria

1. WHEN a device connects on Linux THEN KeyRx SHALL open the evdev device file (`/dev/input/eventX`)
2. WHEN querying for unique identifier THEN KeyRx SHALL use the EVIOCGUNIQ ioctl (`device.unique_name()`)
3. IF EVIOCGUNIQ returns a non-empty string THEN KeyRx SHALL use it as the serial number
4. IF EVIOCGUNIQ returns empty THEN KeyRx SHALL read udev properties (`ID_SERIAL` or `ID_SERIAL_SHORT` from `/sys/class/input/eventX/device/`)
5. IF udev properties are unavailable THEN KeyRx SHALL generate a synthetic serial using: `synthetic_{vid:04x}{pid:04x}_{phys_path_hash:016x}`
6. WHEN generating a synthetic serial THEN KeyRx SHALL hash the `phys` path (USB port topology) to create a stable identifier
7. IF a synthetic serial is used THEN KeyRx SHALL warn the user that the device configuration is port-dependent

---

### Requirement 10: Input Processing Pipeline Integration

**User Story:** As a power user, I want sub-millisecond latency when using profiles, so that KeyRx feels invisible during use.

#### Acceptance Criteria

1. WHEN an input event arrives THEN KeyRx SHALL resolve the device identity from the OS handle in < 50μs (p99)
2. WHEN device identity is resolved THEN KeyRx SHALL load the device state from the registry in < 10μs (cached lookup)
3. IF is_remapping_enabled is false THEN KeyRx SHALL bypass all processing and output the event as-is (< 10μs passthrough latency)
4. IF is_remapping_enabled is true THEN KeyRx SHALL load the assigned profile from cache in < 100μs (p99)
5. WHEN translating scancodes to (row, col) THEN KeyRx SHALL use a cached translation map in < 20μs (p99)
6. WHEN resolving (row, col) to KeyAction THEN KeyRx SHALL perform HashMap lookup in < 10μs (p99)
7. WHEN the full pipeline completes THEN total latency SHALL be < 1ms (p99) from input event to output injection
8. IF any pipeline stage fails THEN KeyRx SHALL fall back to passthrough mode for that event and log an error

---

### Requirement 11: Navigation & UX - Hardware-First Philosophy

**User Story:** As a new user, I want to set up my devices before creating mappings, so that the workflow feels natural (connect hardware → configure behavior).

#### Acceptance Criteria

1. WHEN the user opens KeyRx UI THEN the default landing page SHALL be the Devices tab
2. WHEN viewing the navigation sidebar THEN the Devices icon SHALL appear above the Editor icon
3. WHEN no devices are connected THEN the Devices tab SHALL display an empty state message: "No devices connected. Connect a device to get started."
4. WHEN one or more devices are connected THEN the Devices tab SHALL display a list of DeviceCard components (one per device)
5. WHEN a user first connects a device THEN KeyRx SHALL prompt: "Would you like to label this device?" with a text input field
6. IF the user provides a label THEN it SHALL be stored and displayed instead of the generic device name

---

### Requirement 12: Devices Tab UI - Per-Device Controls

**User Story:** As a user managing multiple devices, I want to see all my devices in one view with controls for each, so that I can quickly toggle remapping and change profiles.

#### Acceptance Criteria

1. WHEN viewing the Devices tab THEN each device SHALL be displayed in a DeviceCard widget showing: user label (or fallback name), VID:PID:Serial, profile selector dropdown, remap toggle switch
2. WHEN clicking the remap toggle THEN KeyRx SHALL call the device registry API to set is_remapping_enabled and update the UI state immediately
3. WHEN selecting a profile from the dropdown THEN KeyRx SHALL validate layout compatibility and display an error if incompatible
4. IF profile assignment succeeds THEN the dropdown SHALL show the selected profile name
5. WHEN clicking "Edit Label" THEN a dialog SHALL appear with a text field pre-filled with the current label
6. WHEN saving a new label THEN the DeviceCard SHALL update to show the new label immediately
7. WHEN clicking "Manage Profiles" THEN the UI SHALL navigate to the device-specific profiles page
8. WHEN the remap toggle is OFF THEN the toggle SHALL display "OFF" in red/gray and the profile selector SHALL be disabled (grayed out)
9. WHEN the remap toggle is ON THEN the toggle SHALL display "ON" in green and the profile selector SHALL be enabled

---

### Requirement 13: Visual Editor - Dynamic Layout Rendering

**User Story:** As a Stream Deck owner, I want the visual editor to show a 3×5 grid of buttons (matching my hardware), so that I can visually map each button to actions.

#### Acceptance Criteria

1. WHEN opening the Visual Editor THEN a profile selector dropdown SHALL appear at the top
2. WHEN selecting a profile THEN the editor SHALL read the profile's layout_type (Matrix, Standard, or Split)
3. IF layout_type is Matrix{rows: 3, cols: 5} THEN the editor SHALL render a 3×5 grid of clickable buttons
4. IF layout_type is Standard(ANSI) THEN the editor SHALL render a full ANSI 104-key keyboard layout
5. WHEN rendering a layout THEN each physical position SHALL display its current mapping (if any)
6. IF a position is unmapped THEN it SHALL display as an empty button with a dashed border
7. WHEN a user clicks a physical position THEN it SHALL highlight and prompt: "Now select an output key"
8. WHEN a user selects an output key from the soft keyboard palette THEN the mapping SHALL be created and the button SHALL update to show the output key label

---

### Requirement 14: Soft Keyboard Palette

**User Story:** As a user creating mappings, I want to see all available output keys in a searchable palette, so that I can easily find and assign keys without typing keycode names.

#### Acceptance Criteria

1. WHEN the Visual Editor is open THEN a soft keyboard palette SHALL be displayed on the right side of the screen
2. WHEN rendering the palette THEN it SHALL display all KeyCode enum variants as clickable buttons
3. WHEN typing in the search box THEN the palette SHALL filter keys to show only those matching the search query (case-insensitive)
4. WHEN clicking a key in the palette THEN it SHALL become the selected output key (highlighted)
5. IF a physical position is already selected THEN clicking a palette key SHALL create the mapping immediately
6. WHEN a mapping is created THEN the palette SHALL clear its selection and return to the full key list
7. WHEN the palette is empty (no keys match search) THEN it SHALL display: "No keys found"

---

### Requirement 15: Profile Migration (Backward Compatibility)

**User Story:** As an existing KeyRx user, I want my old device profiles to be automatically migrated to the new system, so that I don't lose my configurations when upgrading.

#### Acceptance Criteria

1. WHEN KeyRx starts with the new system for the first time THEN it SHALL detect old profiles in `~/.config/keyrx/devices/{vid}_{pid}.json` format
2. IF old profiles exist THEN KeyRx SHALL display a migration prompt: "Migrate old profiles to the new system?"
3. WHEN the user accepts migration THEN KeyRx SHALL convert each old DeviceProfile to a new Profile (UUID ID, layout_type inferred from rows/cols_per_row, mappings from keymap)
4. WHEN migration completes THEN KeyRx SHALL create a backup of old profiles in `~/.config/keyrx/devices_backup/` before deletion
5. IF any connected device matches the VID:PID of a migrated profile THEN KeyRx SHALL auto-assign the migrated profile to that device
6. WHEN migration completes THEN KeyRx SHALL display a summary: "Migrated X profiles. Y devices auto-assigned."
7. IF migration fails for a profile THEN KeyRx SHALL log the error and continue with other profiles (partial migration allowed)

---

### Requirement 16: FFI Layer - Device Registry Exposure

**User Story:** As a Flutter UI developer, I want to access device registry functions via FFI, so that I can build the Devices tab UI without duplicating logic.

#### Acceptance Criteria

1. WHEN calling `krx_device_registry_list_devices()` THEN it SHALL return a JSON array of DeviceState objects
2. WHEN calling `krx_device_registry_set_remap_enabled(vid, pid, serial, enabled)` THEN it SHALL update the device state and return "ok" or "error:message"
3. WHEN calling `krx_device_registry_assign_profile(vid, pid, serial, profile_id)` THEN it SHALL validate layout compatibility and assign the profile
4. WHEN calling `krx_device_registry_set_user_label(vid, pid, serial, label)` THEN it SHALL update the user label and persist to device_bindings.json
5. IF an FFI function encounters a panic THEN it SHALL catch the panic and return "error:panic occurred" (never crash across FFI boundary)
6. IF an FFI function receives null pointers THEN it SHALL return "error:null pointer" without dereferencing
7. WHEN FFI functions return strings THEN the caller SHALL be responsible for freeing the allocated C string memory

---

### Requirement 17: FFI Layer - Profile Registry Exposure

**User Story:** As a UI developer, I want to manage profiles via FFI, so that I can create, list, and delete profiles from the Visual Editor.

#### Acceptance Criteria

1. WHEN calling `krx_profile_registry_list_profiles()` THEN it SHALL return a JSON array of all profiles (id, name, layout_type, created_at, modified_at)
2. WHEN calling `krx_profile_registry_get_profile(profile_id)` THEN it SHALL return the full profile JSON including mappings
3. WHEN calling `krx_profile_registry_save_profile(profile_json)` THEN it SHALL parse, validate, and save the profile
4. WHEN calling `krx_profile_registry_delete_profile(profile_id)` THEN it SHALL remove the profile file and return "ok" or "error:in use by device"
5. WHEN calling `krx_profile_registry_find_compatible_profiles(layout_json)` THEN it SHALL return only profiles matching the provided layout_type
6. IF profile_json is invalid JSON THEN the function SHALL return "error:invalid JSON: {details}"
7. IF profile validation fails THEN the function SHALL return "error:validation failed: {details}"

---

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each module (DeviceRegistry, ProfileRegistry, DeviceIdentity, CoordinateTranslator) SHALL have a single, well-defined purpose
- **Modular Design**: Device identity extraction SHALL be platform-specific modules (`identity/windows.rs`, `identity/linux.rs`) implementing a common interface
- **Dependency Management**: Device registry SHALL NOT depend on profile storage details; profile registry SHALL NOT depend on device detection
- **Clear Interfaces**: FFI boundary SHALL use JSON for complex types and simple C types for primitives, with panic guards on all `extern "C"` functions
- **File Size Limits**: No single Rust file SHALL exceed 500 lines (excluding comments and blank lines); complex modules SHALL be split into submodules
- **Function Size Limits**: No function SHALL exceed 50 lines; complex logic SHALL be extracted into helper functions
- **Test Coverage**: All registry modules SHALL have ≥ 90% test coverage; FFI functions SHALL have panic safety tests

### Performance

- **Input Latency**: Total pipeline latency (device resolution → profile lookup → coordinate translation → action execution) SHALL be < 1ms (p99)
- **Device Resolution**: Resolving device handle to DeviceIdentity SHALL take < 50μs (p99)
- **Profile Lookup**: Loading a profile from cache SHALL take < 100μs (p99); cold load from disk < 10ms
- **Coordinate Translation**: Scancode to (row, col) translation SHALL take < 20μs (p99) using cached translation maps
- **Passthrough Mode**: When remapping is disabled, event passthrough SHALL add < 10μs overhead
- **Startup Time**: Loading all profiles and device definitions SHALL complete within 500ms on systems with < 100 profiles
- **Memory Usage**: Device registry SHALL use < 1KB per device; profile cache SHALL use < 100KB per cached profile

### Security

- **Sandboxing**: Rhai scripts in profiles SHALL NOT have access to filesystem, network, or process spawning
- **FFI Safety**: All FFI functions SHALL validate pointer arguments (null checks) and use panic guards (`std::panic::catch_unwind`)
- **Input Validation**: Profile JSON SHALL be validated against schema before loading; invalid profiles SHALL be rejected with clear error messages
- **File Permissions**: Profile files SHALL be readable/writable only by the user (0600 permissions on Unix)
- **Port-Bound Warning**: Users SHALL be warned when a device uses a synthetic (port-bound) serial number
- **Injection Prevention**: User-provided labels and profile names SHALL be sanitized before use in file paths or logs

### Reliability

- **Crash Recovery**: If a profile fails to load, KeyRx SHALL log the error and continue with other profiles (partial failure allowed)
- **Atomic Writes**: Profile saves and device_bindings.json updates SHALL use atomic write (temp file + rename) to prevent corruption
- **Backup Creation**: Migration SHALL create backups before modifying old profiles
- **Graceful Degradation**: If device definition is missing, fall back to generic ANSI layout; if profile is missing, use passthrough mode
- **Error Logging**: All errors SHALL be logged with structured context (device identity, profile ID, file path, error details)
- **Panic Prevention**: No panic SHALL occur during normal operation; all potential panic points SHALL use `Result<T, Error>` and proper error handling

### Usability

- **Onboarding**: First-time users SHALL see a clear empty state on the Devices tab prompting them to connect a device
- **Feedback**: All state changes (remap toggle, profile assignment) SHALL provide immediate visual feedback (< 100ms UI update)
- **Error Messages**: All error messages SHALL be user-friendly (e.g., "Device not found. Try reconnecting it." instead of "DeviceNotFoundError")
- **Tooltips**: All UI controls (toggle, dropdown, buttons) SHALL have descriptive tooltips
- **Keyboard Navigation**: All UI elements SHALL be keyboard-accessible (tab navigation, enter to activate)
- **Accessibility**: Text SHALL have sufficient contrast (WCAG AA compliant); icons SHALL have alt text

### Maintainability

- **Documentation**: All public APIs SHALL have rustdoc comments with examples; all FFI functions SHALL document memory ownership
- **Code Comments**: Complex algorithms (e.g., synthetic ID generation) SHALL have inline comments explaining the logic
- **Migration Path**: Old data structures SHALL be clearly marked as deprecated with migration instructions
- **Feature Flags**: The revolutionary mapping system SHALL be behind a feature flag for gradual rollout
- **Logging**: All critical paths SHALL have structured logging with appropriate log levels (trace, debug, info, warn, error)
- **Metrics**: Key operations (device registration, profile assignment) SHALL emit metrics for monitoring

