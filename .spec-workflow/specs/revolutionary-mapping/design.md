# Design Document - Revolutionary Mapping

## Overview

The Revolutionary Mapping system introduces a fundamental architectural shift in KeyRx by decoupling physical device identity from logical configuration profiles. This design implements a three-layer architecture: **Identity Layer** (device uniqueness), **Registry Layer** (runtime state + persistent storage), and **Pipeline Layer** (multi-stage input processing).

The system allows users to manage multiple identical devices independently, swap profiles instantly, and support arbitrary device layouts beyond standard keyboards. The design maintains KeyRx's core performance requirement of <1ms input latency while adding new capabilities.

**Place in Overall System:** This feature extends the existing event processing pipeline by adding device resolution and profile resolution stages before the existing Rhai script execution. It integrates with the current FFI layer to expose new capabilities to the Flutter UI.

## Steering Document Alignment

### Technical Standards (tech.md)

**Event Sourcing:** The design maintains the existing event sourcing pattern where input events remain immutable and flow through the processing pipeline.

**No Global State:** All registries (DeviceRegistry, ProfileRegistry) are self-contained structs passed via Arc<RwLock<>> for thread-safe shared access.

**Modular Drivers:** Platform-specific serial number extraction is isolated in `identity/windows.rs` and `identity/linux.rs`, implementing a common interface.

**CLI First:** All registry operations (list devices, assign profile, toggle remap) will be CLI-accessible via `keyrx device ...` and `keyrx profile ...` commands before UI implementation.

**Scripting Contract:** Profiles continue to use Rhai scripting as defined in tech.md, with the addition of row-col physical positioning.

**FFI Architecture:** Follows the existing domain-based FFI pattern with new domains: `device_registry.rs`, `profile_registry.rs`, `device_definitions.rs`.

**Performance:** Adheres to the <1ms latency requirement through aggressive caching (profile cache, translation map cache) and lock-free reads where possible.

### Project Structure (structure.md)

**New Modules:**
- `core/src/identity/` - Device identity and serial extraction
- `core/src/registry/` - Device and profile registries
- `core/src/definitions/` - Device definition loader
- `core/src/engine/device_resolver.rs` - Pipeline stage 1
- `core/src/engine/profile_resolver.rs` - Pipeline stage 2
- `core/src/engine/coordinate_translator.rs` - Pipeline stage 3

**Modified Modules:**
- `core/src/drivers/windows/raw_input.rs` - Add serial extraction
- `core/src/drivers/linux/evdev_input.rs` - Add serial extraction
- `core/src/engine/core.rs` - Integrate new pipeline stages
- `core/src/ffi/domains/` - Add new FFI domains

**New Directories:**
- `device_definitions/` - TOML device specifications
- `.config/keyrx/profiles/` - User profile storage

**File Size:** All new files will adhere to the 500-line limit; complex modules will be split into submodules.

## Code Reuse Analysis

### Existing Components to Leverage

- **`core/src/config/paths.rs`**: Reuse existing config directory resolution for profiles and device_bindings.json storage location
- **`core/src/drivers/keycodes/`**: Reuse existing KeyCode enum and aliases for profile mappings
- **`core/src/scripting/engine.rs`**: Reuse Rhai engine for profile script execution
- **`core/src/discovery/types.rs`**: PhysicalKey struct concept extends to PhysicalPosition; existing row/col layout awareness provides foundation
- **`core/src/ffi/utils.rs`**: Reuse existing FFI panic guards, string conversion utilities, and error handling patterns
- **`ui/lib/ffi/`**: Extend existing FFI binding pattern for new registry domains
- **`ui/lib/services/`**: Follow existing service layer pattern for DeviceRegistryService and ProfileRegistryService

### Integration Points

- **Existing Engine:** New pipeline stages integrate before existing Rhai script execution; passthrough mode bypasses all existing processing
- **Existing FFI Layer:** New FFI domains follow the same panic-safe, JSON-based pattern as existing `device.rs`, `engine.rs` domains
- **Existing Discovery:** Old `discovery/` module will be deprecated but kept for migration; new system replaces its functionality
- **Existing UI Navigation:** Devices tab already exists; we'll enhance it and reorder navigation rather than creating from scratch
- **Existing Visual Editor:** Current editor provides drag-drop infrastructure; we'll extend it to support dynamic layouts

## Architecture

### High-Level System Architecture

```mermaid
graph TB
    subgraph "OS Layer"
        WIN[Windows Raw Input]
        LIN[Linux evdev]
    end

    subgraph "Identity Layer"
        WINSER[Windows Serial Extraction]
        LINSER[Linux Serial Extraction]
        DEVID[DeviceIdentity]
    end

    subgraph "Registry Layer"
        DEVREG[DeviceRegistry<br/>In-Memory]
        PROFR EG[ProfileRegistry<br/>Cached + Disk]
        DEFLIB[DeviceDefinitionLibrary<br/>Read-Only]
        BINDINGS[DeviceBindings<br/>Persistent]
    end

    subgraph "Pipeline Layer"
        DEVRES[DeviceResolver]
        PROFRES[ProfileResolver]
        COORDT[CoordinateTranslator]
        ACTRES[ActionResolver]
        EXEC[Executor]
    end

    subgraph "FFI Layer"
        DEVFFI[device_registry FFI]
        PROFFFI[profile_registry FFI]
        DEFFFI[definitions FFI]
    end

    subgraph "UI Layer"
        DEVPAGE[Devices Page]
        EDPAGE[Editor Page]
    end

    WIN --> WINSER
    LIN --> LINSER
    WINSER --> DEVID
    LINSER --> DEVID

    DEVID --> DEVREG
    DEVREG --> BINDINGS
    PROFR EG --> BINDINGS

    DEVRES --> DEVREG
    PROFRES --> PROFR EG
    COORDT --> DEFLIB

    DEVRES --> PROFRES
    PROFRES --> COORDT
    COORDT --> ACTRES
    ACTRES --> EXEC

    DEVREG --> DEVFFI
    PROFR EG --> PROFFFI
    DEFLIB --> DEFFFI

    DEVFFI --> DEVPAGE
    PROFFFI --> EDPAGE
    DEFFFI --> EDPAGE
```

### Modular Design Principles

- **Single File Responsibility**:
  - `identity/windows.rs` - Windows serial extraction only
  - `registry/device.rs` - Device runtime state only
  - `registry/profile.rs` - Profile storage only
  - `registry/bindings.rs` - Device-profile persistence only

- **Component Isolation**:
  - DeviceRegistry has no knowledge of Profile internal structure
  - ProfileRegistry has no knowledge of Device connectivity
  - CoordinateTranslator only knows about scancode → (row, col) mapping

- **Service Layer Separation**:
  - Identity layer: Extracts device uniqueness (no state management)
  - Registry layer: Manages state and storage (no I/O knowledge)
  - Pipeline layer: Processes events (no storage knowledge)

- **Utility Modularity**:
  - Serial extraction utilities separate from device registration
  - TOML parsing separate from definition lookup
  - JSON serialization separate from storage

### Data Flow Through System

```
Input Event → Device Handle → DeviceIdentity → DeviceState → Profile → (Row,Col) → KeyAction → Output
     ↓              ↓              ↓               ↓            ↓          ↓           ↓
   OS Driver    Platform    Identity Cache   Device Reg   Profile Cache  Def Lib   Action Map
```

## Components and Interfaces

### Component 1: DeviceIdentity (core/src/identity/types.rs)

- **Purpose:** Uniquely identify a physical device across sessions
- **Interfaces:**
  ```rust
  pub struct DeviceIdentity {
      pub vendor_id: u16,
      pub product_id: u16,
      pub serial_number: String,
      pub user_label: Option<String>,
  }

  impl DeviceIdentity {
      pub fn to_key(&self) -> String;  // "{vid:04x}:{pid:04x}:{serial}"
      pub fn from_key(key: &str) -> Result<Self>;
  }
  ```
- **Dependencies:** serde for serialization
- **Reuses:** None (new foundational component)

---

### Component 2: Windows Serial Extraction (core/src/identity/windows.rs)

- **Purpose:** Extract unique serial number from Windows device path
- **Interfaces:**
  ```rust
  pub fn extract_serial_number(device_path: &str) -> Result<String>;
  fn parse_instance_id_from_path(path: &str) -> Result<String>;
  fn read_iserial_descriptor(path: &str) -> Result<String>;
  ```
- **Dependencies:** windows-rs (HidD_GetSerialNumberString, CreateFileW)
- **Reuses:** None (platform-specific)

---

### Component 3: Linux Serial Extraction (core/src/identity/linux.rs)

- **Purpose:** Extract unique serial number from Linux evdev device
- **Interfaces:**
  ```rust
  pub fn extract_serial_number(device_path: &Path) -> Result<String>;
  fn read_udev_serial(device_path: &Path) -> Result<String>;
  fn generate_synthetic_id(phys: &str, input_id: InputId) -> String;
  ```
- **Dependencies:** evdev crate, std::fs for udev sysfs reading
- **Reuses:** Existing evdev integration from `drivers/linux/`

---

### Component 4: DeviceRegistry (core/src/registry/device.rs)

- **Purpose:** Track runtime state of all connected devices
- **Interfaces:**
  ```rust
  pub struct DeviceRegistry {
      devices: Arc<RwLock<HashMap<DeviceIdentity, DeviceState>>>,
      event_tx: mpsc::UnboundedSender<DeviceEvent>,
  }

  impl DeviceRegistry {
      pub fn new(event_tx: mpsc::UnboundedSender<DeviceEvent>) -> Self;
      pub async fn register_device(&self, identity: DeviceIdentity, state: DeviceState) -> Result<()>;
      pub async fn unregister_device(&self, identity: &DeviceIdentity) -> Result<()>;
      pub async fn set_remap_enabled(&self, identity: &DeviceIdentity, enabled: bool) -> Result<()>;
      pub async fn assign_profile(&self, identity: &DeviceIdentity, profile_id: ProfileId) -> Result<()>;
      pub async fn get_device_state(&self, identity: &DeviceIdentity) -> Option<DeviceState>;
      pub async fn list_devices(&self) -> Vec<DeviceState>;
  }
  ```
- **Dependencies:** tokio for async, serde for serialization
- **Reuses:** Existing event channel pattern from `engine/multi_device.rs`

---

### Component 5: ProfileRegistry (core/src/registry/profile.rs)

- **Purpose:** Manage persistent profile storage and in-memory cache
- **Interfaces:**
  ```rust
  pub struct ProfileRegistry {
      profiles: Arc<RwLock<HashMap<ProfileId, Profile>>>,
      storage_path: PathBuf,
  }

  impl ProfileRegistry {
      pub fn new(storage_path: PathBuf) -> Result<Self>;
      pub async fn save_profile(&self, profile: Profile) -> Result<()>;
      pub async fn get_profile(&self, id: &ProfileId) -> Option<Profile>;
      pub async fn delete_profile(&self, id: &ProfileId) -> Result<()>;
      pub async fn list_profiles(&self) -> Vec<Profile>;
      pub async fn find_compatible_profiles(&self, layout: &LayoutType) -> Vec<Profile>;

      fn load_all_profiles(&self) -> Result<()>;
      fn validate_profile(&self, profile: &Profile) -> Result<()>;
      async fn write_profile_to_disk(&self, profile: &Profile) -> Result<()>;
  }
  ```
- **Dependencies:** tokio::fs for async I/O, serde_json for serialization
- **Reuses:** Existing config paths from `config/paths.rs`

---

### Component 6: DeviceBindings (core/src/registry/bindings.rs)

- **Purpose:** Persist device-profile assignments and user labels
- **Interfaces:**
  ```rust
  pub struct DeviceBindings {
      bindings: HashMap<DeviceIdentity, DeviceBinding>,
      file_path: PathBuf,
  }

  impl DeviceBindings {
      pub fn load(file_path: PathBuf) -> Result<Self>;
      pub fn save(&self) -> Result<()>;
      pub fn get_binding(&self, identity: &DeviceIdentity) -> Option<&DeviceBinding>;
      pub fn set_binding(&mut self, identity: DeviceIdentity, binding: DeviceBinding);
      pub fn remove_binding(&mut self, identity: &DeviceIdentity);
  }
  ```
- **Dependencies:** serde_json for file I/O
- **Reuses:** Atomic write pattern from existing `discovery/storage.rs`

---

### Component 7: DeviceDefinitionLibrary (core/src/definitions/library.rs)

- **Purpose:** Load and lookup device layout definitions from TOML files
- **Interfaces:**
  ```rust
  pub struct DeviceDefinitionLibrary {
      definitions: HashMap<(u16, u16), DeviceDefinition>,  // (VID, PID) -> Definition
  }

  impl DeviceDefinitionLibrary {
      pub fn load_from_directory(dir: &Path) -> Result<Self>;
      pub fn find_definition(&self, vid: u16, pid: u16) -> Option<&DeviceDefinition>;
      pub fn list_definitions(&self) -> Vec<&DeviceDefinition>;

      fn load_definition(path: &Path) -> Result<DeviceDefinition>;
      fn validate_definition(def: &DeviceDefinition) -> Result<()>;
  }
  ```
- **Dependencies:** toml crate, walkdir for recursive file search
- **Reuses:** None (new)

---

### Component 8: DeviceResolver (core/src/engine/device_resolver.rs)

- **Purpose:** Resolve OS device handle to DeviceState
- **Interfaces:**
  ```rust
  pub struct DeviceResolver {
      device_registry: Arc<RwLock<DeviceRegistry>>,
  }

  impl DeviceResolver {
      pub fn new(registry: Arc<RwLock<DeviceRegistry>>) -> Self;
      pub async fn resolve(&self, device_handle: RawDeviceHandle) -> Result<Option<DeviceState>>;

      fn extract_identity(&self, handle: RawDeviceHandle) -> Result<DeviceIdentity>;
  }
  ```
- **Dependencies:** Platform-specific identity extraction modules
- **Reuses:** Existing RawDeviceHandle type from drivers

---

### Component 9: ProfileResolver (core/src/engine/profile_resolver.rs)

- **Purpose:** Load profiles from cache or registry
- **Interfaces:**
  ```rust
  pub struct ProfileResolver {
      profile_registry: Arc<RwLock<ProfileRegistry>>,
      profile_cache: Arc<RwLock<HashMap<ProfileId, Arc<Profile>>>>,
  }

  impl ProfileResolver {
      pub fn new(registry: Arc<RwLock<ProfileRegistry>>) -> Self;
      pub async fn resolve(&self, profile_id: &ProfileId) -> Result<Arc<Profile>>;
      pub async fn invalidate_cache(&self, profile_id: &ProfileId);
  }
  ```
- **Dependencies:** None
- **Reuses:** Standard Arc caching pattern

---

### Component 10: CoordinateTranslator (core/src/engine/coordinate_translator.rs)

- **Purpose:** Translate scancodes to (row, col) physical positions
- **Interfaces:**
  ```rust
  pub struct CoordinateTranslator {
      definitions: Arc<DeviceDefinitionLibrary>,
      translation_cache: Arc<RwLock<HashMap<DeviceIdentity, HashMap<u16, PhysicalPosition>>>>,
  }

  impl CoordinateTranslator {
      pub fn new(definitions: Arc<DeviceDefinitionLibrary>) -> Self;
      pub async fn translate(&self, device_identity: &DeviceIdentity, scancode: u16) -> Result<PhysicalPosition>;
  }
  ```
- **Dependencies:** DeviceDefinitionLibrary
- **Reuses:** Existing PhysicalPosition concept from discovery/types.rs

---

### Component 11: FFI - Device Registry (core/src/ffi/domains/device_registry.rs)

- **Purpose:** Expose device registry to Flutter UI
- **Interfaces:**
  ```rust
  #[no_mangle]
  pub extern "C" fn krx_device_registry_list_devices() -> *mut c_char;

  #[no_mangle]
  pub extern "C" fn krx_device_registry_set_remap_enabled(
      vendor_id: u16, product_id: u16, serial: *const c_char, enabled: bool
  ) -> *mut c_char;

  #[no_mangle]
  pub extern "C" fn krx_device_registry_assign_profile(
      vendor_id: u16, product_id: u16, serial: *const c_char, profile_id: *const c_char
  ) -> *mut c_char;

  #[no_mangle]
  pub extern "C" fn krx_device_registry_set_user_label(
      vendor_id: u16, product_id: u16, serial: *const c_char, label: *const c_char
  ) -> *mut c_char;
  ```
- **Dependencies:** libc, existing FFI utilities
- **Reuses:** Panic guard pattern from `ffi/utils.rs`

---

### Component 12: Flutter - DeviceRegistryService (ui/lib/services/device_registry_service.dart)

- **Purpose:** High-level Dart API wrapping FFI calls
- **Interfaces:**
  ```dart
  class DeviceRegistryService {
      Future<List<DeviceState>> getDevices();
      Future<void> toggleRemap(DeviceIdentity device, bool enabled);
      Future<void> assignProfile(DeviceIdentity device, String profileId);
      Future<void> setUserLabel(DeviceIdentity device, String label);
  }
  ```
- **Dependencies:** DeviceRegistryFFI (direct FFI bindings)
- **Reuses:** Existing service layer pattern from `services/device_service.dart`

---

### Component 13: Flutter - DeviceCard Widget (ui/lib/widgets/device_card.dart)

- **Purpose:** Display single device with controls
- **Interfaces:**
  ```dart
  class DeviceCard extends StatelessWidget {
      final DeviceState deviceState;
      final Function(bool) onRemapToggle;
      final Function(String) onProfileSelect;
      final VoidCallback onEditLabel;
  }
  ```
- **Dependencies:** Material widgets, ProfileSelector, RemapToggle
- **Reuses:** Existing Card widget patterns from UI

---

### Component 14: Flutter - LayoutGrid Widget (ui/lib/widgets/layout_grid.dart)

- **Purpose:** Dynamically render device layouts (matrix, standard, split)
- **Interfaces:**
  ```dart
  class LayoutGrid extends StatelessWidget {
      final LayoutType layoutType;
      final Map<PhysicalPosition, KeyAction> mappings;
      final Function(PhysicalPosition) onKeyTap;
  }
  ```
- **Dependencies:** None
- **Reuses:** Existing keyboard layout concept from `models/keyboard_layout.dart`, extending to support arbitrary grids

---

## Data Models

### DeviceIdentity
```rust
pub struct DeviceIdentity {
    pub vendor_id: u16,           // USB Vendor ID
    pub product_id: u16,          // USB Product ID
    pub serial_number: String,    // Hardware serial or synthetic ID
    pub user_label: Option<String>, // User-friendly name
}
```

### DeviceState
```rust
pub struct DeviceState {
    pub identity: DeviceIdentity,
    pub is_remapping_enabled: bool,
    pub active_profile_id: Option<ProfileId>,
    pub connected_at: DateTime<Utc>,
    pub state: DeviceRuntimeState,
}

pub enum DeviceRuntimeState {
    Active,
    Passthrough,
    Failed { error_code: u32 },
}
```

### Profile
```rust
pub struct Profile {
    pub id: ProfileId,             // UUID
    pub name: String,
    pub layout_type: LayoutType,
    pub mappings: HashMap<PhysicalPosition, KeyAction>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub script_source: Option<String>, // Optional Rhai script
}
```

### LayoutType
```rust
pub enum LayoutType {
    Standard(StandardLayout),      // ANSI, ISO, JIS
    Matrix { rows: u8, cols: u8 }, // Custom grid
    Split {                        // Split keyboard
        left: Box<LayoutType>,
        right: Box<LayoutType>,
    },
}

pub enum StandardLayout {
    ANSI,
    ISO,
    JIS,
}
```

### PhysicalPosition
```rust
pub struct PhysicalPosition {
    pub row: u8,
    pub col: u8,
}
```

### KeyAction
```rust
pub enum KeyAction {
    Key(KeyCode),                  // Single key output
    Chord {                        // Modifier + key (e.g., Ctrl+C)
        modifiers: Vec<KeyCode>,
        key: KeyCode,
    },
    Script(String),                // Rhai script execution
    Block,                         // Suppress key
    Pass,                          // Passthrough (no remap)
}
```

### DeviceBinding
```rust
pub struct DeviceBinding {
    pub active_profile_id: Option<ProfileId>,
    pub is_remapping_enabled: bool,
    pub last_connected: DateTime<Utc>,
}
```

### DeviceDefinition
```rust
pub struct DeviceDefinition {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub layout: LayoutDefinition,
    pub matrix_map: HashMap<u16, PhysicalPosition>, // scancode -> (row, col)
    pub visual: Option<VisualMetadata>,
}

pub struct LayoutDefinition {
    pub layout_type: String,       // "Matrix", "Standard", "Split"
    pub rows: u8,
    pub cols: Option<u8>,
    pub cols_per_row: Option<Vec<u8>>,
}

pub struct VisualMetadata {
    pub key_width: u16,
    pub key_height: u16,
    pub key_spacing: u8,
}
```

## Error Handling

### Error Scenarios

1. **Device Serial Extraction Fails**
   - **Handling:** Fall back to synthetic ID generation; log warning; set flag `is_port_bound = true`
   - **User Impact:** User sees warning: "This device doesn't have a serial number. Configuration is tied to USB port."

2. **Profile File Corrupted**
   - **Handling:** Skip loading that profile; log error with file path; continue with other profiles
   - **User Impact:** Profile doesn't appear in list; error shown in logs/console

3. **Profile Assignment - Layout Incompatible**
   - **Handling:** Return error from `assign_profile()` with details: "Device has 3×5 layout but profile requires 5×5"
   - **User Impact:** Dialog shows error; assignment is rejected; device remains in current state

4. **Device Registry - Device Not Found**
   - **Handling:** Return `Error::DeviceNotFound(identity)` from registry methods
   - **User Impact:** UI shows: "Device not connected. Please reconnect device."

5. **FFI Panic**
   - **Handling:** `catch_unwind()` in all extern "C" functions; return "error:panic occurred"
   - **User Impact:** Operation fails gracefully; UI shows generic error; app doesn't crash

6. **Device Definition Missing**
   - **Handling:** Fall back to generic ANSI layout; log info message
   - **User Impact:** Editor shows standard keyboard layout instead of custom layout

7. **Profile Load - Rhai Script Invalid**
   - **Handling:** Validate script on load; if invalid, set `script_source = None` and use mappings only
   - **User Impact:** Profile loads but without advanced scripting; user can fix script in editor

8. **Atomic Write Failure**
   - **Handling:** If temp file write succeeds but rename fails, retry once; if still fails, return error without corrupting original
   - **User Impact:** Save operation fails; user sees error; original profile preserved

9. **Migration - Old Profile Parse Error**
   - **Handling:** Log error; skip that profile; continue with others; include in migration report
   - **User Impact:** Migration summary shows: "Migrated 9/10 profiles. 1 failed (see logs)."

## Testing Strategy

### Unit Testing

**Core Modules:**
- `identity/windows.rs`: Test `parse_instance_id_from_path()` with various device path formats
- `identity/linux.rs`: Test `generate_synthetic_id()` with different phys paths; verify hash stability
- `registry/device.rs`: Test all CRUD operations; concurrent access tests
- `registry/profile.rs`: Test save/load roundtrip; validation; compatibility filtering
- `registry/bindings.rs`: Test load/save; missing file handling
- `definitions/library.rs`: Test TOML parsing; validation; VID:PID lookup

**Key Unit Tests:**
```rust
#[test]
fn test_device_identity_key_roundtrip() {
    let identity = DeviceIdentity { /* ... */ };
    let key = identity.to_key();
    let parsed = DeviceIdentity::from_key(&key).unwrap();
    assert_eq!(identity, parsed);
}

#[tokio::test]
async fn test_device_registry_concurrent_access() {
    // Spawn 10 tasks reading/writing device registry concurrently
    // Verify no data races or deadlocks
}

#[test]
fn test_profile_validation_rejects_invalid_layout() {
    let profile = Profile {
        layout_type: LayoutType::Matrix { rows: 0, cols: 5 }, // Invalid: 0 rows
        /* ... */
    };
    assert!(validate_profile(&profile).is_err());
}
```

### Integration Testing

**Cross-Module Tests:**
- Device registration → profile assignment → remap toggle → input processing (full flow)
- Profile save → app restart → profile reload → verify mappings intact
- Two identical devices → different profiles → verify isolation
- Device disconnect/reconnect → verify bindings persist
- Migration: old profile files → new system → verify conversion

**Key Integration Tests:**
```rust
#[tokio::test]
async fn test_multi_device_isolation() {
    let registry = DeviceRegistry::new(/* ... */);
    let profile_registry = ProfileRegistry::new(/* ... */);

    // Register two identical devices (same VID:PID, different serials)
    let device1 = DeviceIdentity { serial_number: "ABC123", /* ... */ };
    let device2 = DeviceIdentity { serial_number: "XYZ789", /* ... */ };

    // Assign different profiles
    registry.assign_profile(&device1, "profile-work".into()).await.unwrap();
    registry.assign_profile(&device2, "profile-gaming".into()).await.unwrap();

    // Verify isolation
    let state1 = registry.get_device_state(&device1).await.unwrap();
    let state2 = registry.get_device_state(&device2).await.unwrap();
    assert_eq!(state1.active_profile_id, Some("profile-work".into()));
    assert_eq!(state2.active_profile_id, Some("profile-gaming".into()));
}

#[tokio::test]
async fn test_profile_persistence() {
    let profile = Profile { /* ... */ };
    let registry = ProfileRegistry::new(temp_dir()).unwrap();

    // Save profile
    registry.save_profile(profile.clone()).await.unwrap();

    // Simulate restart: create new registry instance
    let new_registry = ProfileRegistry::new(temp_dir()).unwrap();

    // Verify profile loaded
    let loaded = new_registry.get_profile(&profile.id).await.unwrap();
    assert_eq!(loaded, profile);
}
```

### End-to-End Testing

**User Scenarios:**
1. **First-Time User Setup:**
   - Connect device → See empty state → Assign label → Assign profile → Toggle remap ON → Verify input remapped

2. **Multi-Device Power User:**
   - Connect 3 devices → Label each → Assign different profiles → Toggle remap individually → Verify each device works independently

3. **Profile Swapping:**
   - Connect device with profile A active → Swap to profile B → Verify behavior changes immediately (< 100ms)

4. **Migration:**
   - Start with old KeyRx data → Upgrade to new version → Accept migration prompt → Verify all old profiles converted → Verify devices auto-assigned

5. **Custom Layout:**
   - Connect 5×5 macro pad → Open editor → Verify 5×5 grid displayed → Create mappings → Verify mappings work

**E2E Test Implementation:**
```rust
#[tokio::test]
async fn test_e2e_profile_swap() {
    let mut engine = Engine::new_with_revolutionary_mapping(/* ... */);
    let device = simulate_device_connection(/* VID:PID:Serial */);

    // Assign profile A
    engine.assign_profile(&device.identity, "profile-vim").await.unwrap();
    engine.set_remap_enabled(&device.identity, true).await.unwrap();

    // Simulate key press (expecting Vim-style mapping)
    let input = simulate_key_press(&device, "H"); // Vim left
    let output = engine.process_input(input).await.unwrap();
    assert_eq!(output, KeyCode::Left);

    // Swap to profile B (gaming)
    engine.assign_profile(&device.identity, "profile-gaming").await.unwrap();

    // Same key, different output
    let input = simulate_key_press(&device, "H");
    let output = engine.process_input(input).await.unwrap();
    assert_eq!(output, KeyCode::H); // Passthrough in gaming profile
}
```

### Performance Testing

**Benchmarks (using Criterion):**
```rust
fn bench_device_resolution(c: &mut Criterion) {
    c.bench_function("device_resolution", |b| {
        b.iter(|| {
            // Measure time from raw handle to DeviceState
        });
    });
}

fn bench_profile_lookup(c: &mut Criterion) {
    c.bench_function("profile_lookup_cached", |b| {
        b.iter(|| {
            // Measure cached profile load
        });
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("full_pipeline", |b| {
        b.iter(|| {
            // Measure end-to-end: raw event → output
        });
    });
}
```

**Performance Targets:**
- Device resolution: <50μs (p99)
- Profile lookup (cached): <100μs (p99)
- Coordinate translation: <20μs (p99)
- Full pipeline: <1ms (p99)

### Fuzz Testing

**Fuzzing Targets:**
- Device path parsing (random strings)
- Profile JSON deserialization (malformed JSON)
- TOML parsing (malformed device definitions)
- Concurrent registry access (random operation sequences)

```rust
#[cfg(test)]
mod fuzz {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_device_path_parsing_never_panics(path in "\\PC.*") {
            let _ = parse_instance_id_from_path(&path); // Should return Err, not panic
        }

        #[test]
        fn test_profile_json_never_panics(json in ".*") {
            let _ = serde_json::from_str::<Profile>(&json); // Should return Err, not panic
        }
    }
}
```

## Security Considerations

1. **Sandboxing:** Rhai scripts in profiles continue to use existing sandbox (no FS/network access)
2. **FFI Boundary:** All extern "C" functions validate pointers and use panic guards
3. **File Permissions:** Profile files written with 0600 permissions (user-only read/write)
4. **Input Validation:** Profile names and device labels sanitized before use in file paths
5. **Port-Bound Warning:** Users explicitly warned when devices lack hardware serials

## Migration Path

**Old System:** `~/.config/keyrx/devices/{vid}_{pid}.json` per VID:PID
**New System:** `~/.config/keyrx/profiles/{uuid}.json` per profile + `device_bindings.json`

**Migration Process:**
1. Detect old files on first run with new system
2. Prompt user: "Migrate old profiles?"
3. Convert each old file to new Profile (generate UUID, infer layout from rows/cols)
4. For each connected device matching old VID:PID, create binding to migrated profile
5. Backup old files to `devices_backup/`
6. Write migration report

## Performance Optimizations

1. **Caching:**
   - Profile cache: Arc<Profile> shared across pipeline stages
   - Translation map cache: HashMap<scancode, (row, col)> per device
   - Device registry: In-memory HashMap for O(1) lookups

2. **Lock Contention:**
   - Use RwLock for registries (many readers, few writers)
   - Minimize lock hold time (clone data before releasing)
   - Consider lock-free structures (dashmap) for hot paths

3. **Async I/O:**
   - Use tokio::fs for non-blocking profile I/O
   - Batch profile loads on startup

4. **Lazy Loading:**
   - Device definitions loaded once on startup
   - Profiles loaded into cache on first access

