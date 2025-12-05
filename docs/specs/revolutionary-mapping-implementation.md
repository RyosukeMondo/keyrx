# Revolutionary Mapping Implementation Spec

**Spec ID:** SPEC-001
**Version:** 1.0
**Status:** Draft
**Created:** 2025-12-06
**Owner:** KeyRx Core Team

---

## Executive Summary

This specification details the implementation plan for transforming KeyRx from a simple key remapper into a professional-grade Input Management System through the Revolutionary Mapping Architecture. The implementation is divided into 6 phases over 8-10 weeks.

**Core Objectives:**
1. Implement unique device identification via serial numbers
2. Decouple physical devices from logical profiles
3. Enable multi-device management with per-device control
4. Support custom device layouts beyond standard keyboards
5. Maintain <1ms input latency throughout

**Success Criteria:**
- User can connect two identical devices and assign different profiles
- User can swap profiles on a device without restart
- User can toggle remapping per device independently
- Visual editor supports 5×5 macro pads and standard keyboards
- All existing functionality remains working (backward compatibility)

---

## Phase 1: Core Data Structures & Device Identity

**Duration:** 2 weeks
**Goal:** Establish foundational data models and serial number extraction

### 1.1. Device Identity System

#### Task 1.1.1: Define DeviceIdentity Struct
**File:** `core/src/identity/types.rs` (NEW)

**Implementation:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: String,
    pub user_label: Option<String>,
}

impl DeviceIdentity {
    /// Create a canonical string representation for storage keys
    pub fn to_key(&self) -> String {
        format!("{:04x}:{:04x}:{}", self.vendor_id, self.product_id, self.serial_number)
    }

    /// Parse from storage key format
    pub fn from_key(key: &str) -> Result<Self, ParseError> {
        // Implementation
    }
}
```

**Tests:**
- Unit tests for `to_key()` and `from_key()` roundtrip
- Hash collision tests with 1000+ device identities
- Serialization/deserialization tests

**Acceptance Criteria:**
- [x] `DeviceIdentity` struct compiles and passes all tests
- [x] Can be used as HashMap key
- [x] Serializes to/from JSON correctly

---

#### Task 1.1.2: Windows Serial Number Extraction
**File:** `core/src/identity/windows.rs` (NEW)

**Implementation:**
```rust
use windows::Win32::Devices::HumanInterfaceDevices::*;
use windows::Win32::Storage::FileSystem::*;

pub fn extract_serial_number(device_path: &str) -> Result<String> {
    // 1. Try to read iSerial descriptor via HID API
    if let Ok(serial) = read_iserial_descriptor(device_path) {
        if !serial.is_empty() {
            return Ok(serial);
        }
    }

    // 2. Parse InstanceID from device path
    let instance_id = parse_instance_id_from_path(device_path)?;
    Ok(instance_id)
}

fn parse_instance_id_from_path(path: &str) -> Result<String> {
    // Parse: \\?\HID#VID_vvvv&PID_pppp&MI_ii#<InstanceID>#{ClassGUID}
    let parts: Vec<&str> = path.split('#').collect();
    if parts.len() < 3 {
        return Err(Error::InvalidDevicePath);
    }
    Ok(parts[2].to_string())
}

fn read_iserial_descriptor(device_path: &str) -> Result<String> {
    unsafe {
        let handle = CreateFileW(/* ... */)?;
        let mut buffer = [0u16; 256];
        let success = HidD_GetSerialNumberString(handle, buffer.as_mut_ptr() as *mut _, buffer.len() * 2);
        CloseHandle(handle);

        if success.as_bool() {
            Ok(String::from_utf16_lossy(&buffer).trim_end_matches('\0').to_string())
        } else {
            Err(Error::NoSerialDescriptor)
        }
    }
}
```

**Tests:**
- Mock device path parsing tests
- Integration test with real USB device (manual verification)
- Fallback to InstanceID test when iSerial unavailable

**Acceptance Criteria:**
- [x] Extracts serial from devices with iSerial descriptor
- [x] Falls back to InstanceID for generic devices
- [x] Handles malformed paths gracefully
- [x] Compiles on Windows only (conditional compilation)

---

#### Task 1.1.3: Linux Serial Number Extraction
**File:** `core/src/identity/linux.rs` (NEW)

**Implementation:**
```rust
use evdev::{Device, InputId};
use std::path::Path;

pub fn extract_serial_number(device_path: &Path) -> Result<String> {
    let mut device = Device::open(device_path)?;

    // 1. Try EVIOCGUNIQ ioctl
    if let Some(unique) = device.unique_name() {
        if !unique.is_empty() {
            return Ok(unique.to_string());
        }
    }

    // 2. Try udev properties
    if let Ok(serial) = read_udev_serial(device_path) {
        return Ok(serial);
    }

    // 3. Generate synthetic ID from phys path
    let phys = device.physical_path().unwrap_or("unknown").to_string();
    let input_id = device.input_id();
    Ok(generate_synthetic_id(&phys, input_id))
}

fn read_udev_serial(device_path: &Path) -> Result<String> {
    let event_name = device_path.file_name().unwrap().to_str().unwrap();
    let sys_path = format!("/sys/class/input/{}/device/id/serial", event_name);

    std::fs::read_to_string(&sys_path)
        .map(|s| s.trim().to_string())
        .map_err(|_| Error::NoUdevSerial)
}

fn generate_synthetic_id(phys: &str, input_id: InputId) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    phys.hash(&mut hasher);
    format!("synthetic_{:04x}{:04x}_{:016x}",
        input_id.vendor(), input_id.product(), hasher.finish())
}
```

**Tests:**
- Mock evdev device tests
- Synthetic ID generation tests
- Integration test with real input device

**Acceptance Criteria:**
- [x] Extracts serial via EVIOCGUNIQ when available
- [x] Falls back to udev properties
- [x] Generates stable synthetic ID for port-bound devices
- [x] Compiles on Linux only

---

#### Task 1.1.4: Integrate Serial Extraction into Drivers
**Modified Files:**
- `core/src/drivers/windows/raw_input.rs`
- `core/src/drivers/linux/evdev_input.rs`

**Changes:**
1. Call `extract_serial_number()` when device is detected
2. Include `serial_number` in `InputEvent` metadata
3. Pass `DeviceIdentity` to engine instead of just `DeviceId`

**Acceptance Criteria:**
- [x] Raw Input API passes serial to engine
- [x] evdev driver passes serial to engine
- [x] No performance regression (measure with benchmarks)

---

### 1.2. Registry System

#### Task 1.2.1: Define Profile Data Model
**File:** `core/src/registry/profile.rs` (NEW)

**Implementation:**
```rust
pub type ProfileId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub layout_type: LayoutType,
    pub mappings: HashMap<PhysicalPosition, KeyAction>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub script_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Standard(StandardLayout),
    Matrix { rows: u8, cols: u8 },
    Split { left: Box<LayoutType>, right: Box<LayoutType> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalPosition {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAction {
    Key(KeyCode),
    Chord { modifiers: Vec<KeyCode>, key: KeyCode },
    Script(String),
    Block,
    Pass,
}
```

**Tests:**
- Serialization roundtrip tests
- Layout compatibility tests
- Validation tests (invalid row/col values)

**Acceptance Criteria:**
- [x] All types compile and serialize correctly
- [x] Layout types represent all supported layouts
- [x] KeyAction enum covers all use cases

---

#### Task 1.2.2: Implement ProfileRegistry
**File:** `core/src/registry/profile.rs`

**Implementation:**
```rust
pub struct ProfileRegistry {
    profiles: Arc<RwLock<HashMap<ProfileId, Profile>>>,
    storage_path: PathBuf,
}

impl ProfileRegistry {
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        let registry = Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        };
        registry.load_all_profiles()?;
        Ok(registry)
    }

    pub async fn save_profile(&self, profile: Profile) -> Result<()> {
        // 1. Validate profile
        self.validate_profile(&profile)?;

        // 2. Update in-memory registry
        let mut profiles = self.profiles.write().await;
        profiles.insert(profile.id.clone(), profile.clone());
        drop(profiles);

        // 3. Persist to disk (atomic write)
        self.write_profile_to_disk(&profile).await?;

        Ok(())
    }

    pub async fn get_profile(&self, id: &ProfileId) -> Option<Profile> {
        let profiles = self.profiles.read().await;
        profiles.get(id).cloned()
    }

    pub async fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        profiles.remove(id);
        drop(profiles);

        let file_path = self.profile_file_path(id);
        tokio::fs::remove_file(&file_path).await?;

        Ok(())
    }

    pub async fn list_profiles(&self) -> Vec<Profile> {
        let profiles = self.profiles.read().await;
        profiles.values().cloned().collect()
    }

    pub async fn find_compatible_profiles(&self, layout: &LayoutType) -> Vec<Profile> {
        let profiles = self.profiles.read().await;
        profiles.values()
            .filter(|p| layouts_compatible(&p.layout_type, layout))
            .cloned()
            .collect()
    }

    fn profile_file_path(&self, id: &ProfileId) -> PathBuf {
        self.storage_path.join("profiles").join(format!("{}.json", id))
    }

    async fn write_profile_to_disk(&self, profile: &Profile) -> Result<()> {
        let path = self.profile_file_path(&profile.id);
        let temp_path = path.with_extension("tmp");

        // Atomic write: write to temp, then rename
        let json = serde_json::to_string_pretty(&profile)?;
        tokio::fs::write(&temp_path, json).await?;
        tokio::fs::rename(&temp_path, &path).await?;

        Ok(())
    }

    fn load_all_profiles(&self) -> Result<()> {
        // Load all .json files from profiles directory
        // Called on initialization
    }
}
```

**Tests:**
- CRUD operations tests
- Concurrent access tests (multiple readers/writers)
- File corruption recovery tests
- Profile compatibility filtering tests

**Acceptance Criteria:**
- [x] All CRUD operations work correctly
- [x] Atomic writes prevent data corruption
- [x] Concurrent access is thread-safe
- [x] Profiles persist across restarts

---

#### Task 1.2.3: Implement DeviceRegistry
**File:** `core/src/registry/device.rs` (NEW)

**Implementation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub identity: DeviceIdentity,
    pub is_remapping_enabled: bool,
    pub active_profile_id: Option<ProfileId>,
    pub connected_at: DateTime<Utc>,
    pub state: DeviceRuntimeState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRuntimeState {
    Active,
    Passthrough,
    Failed { error_code: u32 },
}

pub struct DeviceRegistry {
    devices: Arc<RwLock<HashMap<DeviceIdentity, DeviceState>>>,
    event_tx: mpsc::UnboundedSender<DeviceEvent>,
}

impl DeviceRegistry {
    pub async fn register_device(&self, identity: DeviceIdentity, initial_state: DeviceState) -> Result<()> {
        let mut devices = self.devices.write().await;
        devices.insert(identity.clone(), initial_state);
        drop(devices);

        self.event_tx.send(DeviceEvent::Connected(identity))?;
        Ok(())
    }

    pub async fn unregister_device(&self, identity: &DeviceIdentity) -> Result<()> {
        let mut devices = self.devices.write().await;
        devices.remove(identity);
        drop(devices);

        self.event_tx.send(DeviceEvent::Disconnected(identity.clone()))?;
        Ok(())
    }

    pub async fn set_remap_enabled(&self, identity: &DeviceIdentity, enabled: bool) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(state) = devices.get_mut(identity) {
            state.is_remapping_enabled = enabled;
            state.state = if enabled { DeviceRuntimeState::Active } else { DeviceRuntimeState::Passthrough };
        } else {
            return Err(Error::DeviceNotFound(identity.clone()));
        }
        drop(devices);

        self.event_tx.send(DeviceEvent::StateChanged(identity.clone()))?;
        Ok(())
    }

    pub async fn assign_profile(&self, identity: &DeviceIdentity, profile_id: ProfileId) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(state) = devices.get_mut(identity) {
            state.active_profile_id = Some(profile_id.clone());
        } else {
            return Err(Error::DeviceNotFound(identity.clone()));
        }
        drop(devices);

        self.event_tx.send(DeviceEvent::ProfileChanged(identity.clone()))?;
        Ok(())
    }

    pub async fn get_device_state(&self, identity: &DeviceIdentity) -> Option<DeviceState> {
        let devices = self.devices.read().await;
        devices.get(identity).cloned()
    }

    pub async fn list_devices(&self) -> Vec<DeviceState> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }
}
```

**Tests:**
- Device registration/unregistration tests
- State management tests
- Profile assignment tests
- Event emission tests

**Acceptance Criteria:**
- [x] Devices can be registered/unregistered
- [x] Per-device remap toggle works
- [x] Profile assignment persists
- [x] Events are emitted correctly

---

#### Task 1.2.4: Implement Device-Profile Bindings Persistence
**File:** `core/src/registry/bindings.rs` (NEW)

**Purpose:** Persist device-profile assignments and user labels across sessions.

**File Format:** `device_bindings.json`
```json
{
  "bindings": [
    {
      "device": {
        "vendor_id": 4057,
        "product_id": 128,
        "serial_number": "ABC123",
        "user_label": "Work Stream Deck"
      },
      "active_profile_id": "profile-obs",
      "is_remapping_enabled": true,
      "last_connected": "2025-12-06T10:00:00Z"
    }
  ]
}
```

**Implementation:**
```rust
pub struct DeviceBindings {
    bindings: HashMap<DeviceIdentity, DeviceBinding>,
    file_path: PathBuf,
}

impl DeviceBindings {
    pub fn load(file_path: PathBuf) -> Result<Self> {
        // Load from file, create empty if not exists
    }

    pub fn save(&self) -> Result<()> {
        // Atomic write to file
    }

    pub fn get_binding(&self, identity: &DeviceIdentity) -> Option<&DeviceBinding> {
        self.bindings.get(identity)
    }

    pub fn set_binding(&mut self, identity: DeviceIdentity, binding: DeviceBinding) {
        self.bindings.insert(identity, binding);
    }
}
```

**Tests:**
- Load/save roundtrip tests
- Missing file handling tests
- Concurrent access tests

**Acceptance Criteria:**
- [x] Bindings persist across app restarts
- [x] User labels are preserved
- [x] Active profile assignments are preserved

---

### 1.3. Phase 1 Deliverables

**Files Created:**
- `core/src/identity/types.rs`
- `core/src/identity/windows.rs`
- `core/src/identity/linux.rs`
- `core/src/registry/device.rs`
- `core/src/registry/profile.rs`
- `core/src/registry/bindings.rs`

**Files Modified:**
- `core/src/drivers/windows/raw_input.rs`
- `core/src/drivers/linux/evdev_input.rs`

**Tests:**
- 50+ unit tests
- 10+ integration tests
- Platform-specific tests (Windows, Linux)

**Documentation:**
- API documentation for all public types
- Serial extraction troubleshooting guide

---

## Phase 2: Input Processing Pipeline Integration

**Duration:** 2 weeks
**Goal:** Integrate new registry system into event processing pipeline

### 2.1. Pipeline Components

#### Task 2.1.1: Implement DeviceResolver
**File:** `core/src/engine/device_resolver.rs` (NEW)

**Implementation:**
```rust
pub struct DeviceResolver {
    device_registry: Arc<RwLock<DeviceRegistry>>,
}

impl DeviceResolver {
    pub async fn resolve(&self, device_handle: RawDeviceHandle) -> Result<Option<DeviceState>> {
        // Extract identity from platform-specific handle
        let identity = self.extract_identity(device_handle)?;

        // Lookup in registry
        let registry = self.device_registry.read().await;
        Ok(registry.get_device_state(&identity).await)
    }

    fn extract_identity(&self, handle: RawDeviceHandle) -> Result<DeviceIdentity> {
        #[cfg(target_os = "windows")]
        {
            // Use Windows-specific extraction
            use crate::identity::windows::extract_serial_number;
            // ... implementation
        }

        #[cfg(target_os = "linux")]
        {
            // Use Linux-specific extraction
            use crate::identity::linux::extract_serial_number;
            // ... implementation
        }
    }
}
```

**Tests:**
- Mock device handle resolution tests
- Platform-specific integration tests

**Acceptance Criteria:**
- [x] Resolves device handle to DeviceState
- [x] Handles unknown devices gracefully
- [x] <50μs latency (p99)

---

#### Task 2.1.2: Implement ProfileResolver
**File:** `core/src/engine/profile_resolver.rs` (NEW)

**Implementation:**
```rust
pub struct ProfileResolver {
    profile_registry: Arc<RwLock<ProfileRegistry>>,
    profile_cache: Arc<RwLock<HashMap<ProfileId, Arc<Profile>>>>,
}

impl ProfileResolver {
    pub async fn resolve(&self, profile_id: &ProfileId) -> Result<Arc<Profile>> {
        // Check cache first
        {
            let cache = self.profile_cache.read().await;
            if let Some(profile) = cache.get(profile_id) {
                return Ok(Arc::clone(profile));
            }
        }

        // Load from registry
        let registry = self.profile_registry.read().await;
        let profile = registry.get_profile(profile_id).await
            .ok_or(Error::ProfileNotFound(profile_id.clone()))?;

        // Cache for future lookups
        let arc_profile = Arc::new(profile);
        let mut cache = self.profile_cache.write().await;
        cache.insert(profile_id.clone(), Arc::clone(&arc_profile));

        Ok(arc_profile)
    }

    pub async fn invalidate_cache(&self, profile_id: &ProfileId) {
        let mut cache = self.profile_cache.write().await;
        cache.remove(profile_id);
    }
}
```

**Tests:**
- Cache hit/miss tests
- Profile reload tests
- Cache invalidation tests

**Acceptance Criteria:**
- [x] Profiles are cached after first load
- [x] Cache invalidation works correctly
- [x] <100μs latency (p99) for cached profiles

---

#### Task 2.1.3: Implement CoordinateTranslator
**File:** `core/src/engine/coordinate_translator.rs` (NEW)

**Purpose:** Translate OS-specific scancodes to normalized (row, col) positions using device definitions.

**Implementation:**
```rust
pub struct CoordinateTranslator {
    definitions: Arc<DeviceDefinitionLibrary>,
    translation_cache: Arc<RwLock<HashMap<DeviceIdentity, HashMap<u16, PhysicalPosition>>>>,
}

impl CoordinateTranslator {
    pub async fn translate(
        &self,
        device_identity: &DeviceIdentity,
        scancode: u16,
    ) -> Result<PhysicalPosition> {
        // Check cache
        {
            let cache = self.translation_cache.read().await;
            if let Some(map) = cache.get(device_identity) {
                if let Some(pos) = map.get(&scancode) {
                    return Ok(*pos);
                }
            }
        }

        // Load device definition
        let definition = self.definitions
            .find_definition(device_identity.vendor_id, device_identity.product_id)
            .ok_or(Error::NoDeviceDefinition(device_identity.vendor_id, device_identity.product_id))?;

        // Build translation map and cache it
        let map = definition.matrix_map.clone();
        let mut cache = self.translation_cache.write().await;
        cache.insert(device_identity.clone(), map.clone());

        // Lookup position
        map.get(&scancode)
            .copied()
            .ok_or(Error::UnmappedScancode(scancode))
    }
}
```

**Tests:**
- Translation tests with known device definitions
- Cache tests
- Fallback tests for unknown scancodes

**Acceptance Criteria:**
- [x] Scancodes translate to (row, col) correctly
- [x] Translation map is cached
- [x] <20μs latency (p99)

---

#### Task 2.1.4: Integrate Pipeline into Engine
**Modified File:** `core/src/engine/core.rs`

**Changes:**
1. Add DeviceResolver, ProfileResolver, CoordinateTranslator to engine
2. Update input event processing loop
3. Add passthrough mode for disabled devices

**New Pipeline:**
```rust
async fn process_input_event(&mut self, raw_event: RawInputEvent) -> Result<Vec<OutputEvent>> {
    // Stage 1: Device Resolution
    let device_state = self.device_resolver
        .resolve(raw_event.device_handle)
        .await?
        .ok_or(Error::UnknownDevice)?;

    // Check if remapping is enabled
    if !device_state.is_remapping_enabled || device_state.state != DeviceRuntimeState::Active {
        // Passthrough mode
        return Ok(vec![OutputEvent::Passthrough(raw_event)]);
    }

    // Stage 2: Profile Resolution
    let profile_id = device_state.active_profile_id
        .ok_or(Error::NoActiveProfile)?;
    let profile = self.profile_resolver
        .resolve(&profile_id)
        .await?;

    // Stage 3: Coordinate Translation
    let position = self.coordinate_translator
        .translate(&device_state.identity, raw_event.scancode)
        .await?;

    // Stage 4: Action Resolution
    let action = profile.mappings.get(&position)
        .ok_or(Error::NoMappingForPosition(position))?;

    // Stage 5: Execution
    let output_events = self.executor.execute(action).await?;

    Ok(output_events)
}
```

**Tests:**
- End-to-end pipeline tests
- Passthrough mode tests
- Error handling tests (missing profile, unmapped key, etc.)

**Acceptance Criteria:**
- [x] Full pipeline processes events correctly
- [x] Passthrough mode works for disabled devices
- [x] <1ms total latency (p99)
- [x] No regression in existing functionality

---

### 2.2. Phase 2 Deliverables

**Files Created:**
- `core/src/engine/device_resolver.rs`
- `core/src/engine/profile_resolver.rs`
- `core/src/engine/coordinate_translator.rs`

**Files Modified:**
- `core/src/engine/core.rs`
- `core/src/engine/multi_device.rs`

**Tests:**
- 40+ unit tests
- 15+ integration tests
- Performance benchmarks

**Metrics:**
- Pipeline latency: <1ms (p99)
- Device resolution: <50μs
- Profile resolution: <100μs
- Coordinate translation: <20μs

---

## Phase 3: Device Definitions & Layout Support

**Duration:** 1.5 weeks
**Goal:** Implement TOML-based device definition system

### 3.1. Device Definition Loader

#### Task 3.1.1: Define Device Definition Schema
**File:** `core/src/definitions/types.rs` (NEW)

**TOML Schema:**
```toml
name = "Device Name"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Company Name"

[layout]
type = "Matrix"  # or "Standard" or "Split"
rows = 5
cols = 5

[matrix_map]
# HID Usage ID or Scan Code -> [row, col]
0x01 = [0, 0]
0x02 = [0, 1]
# ...

[visual]
key_width = 72
key_height = 72
key_spacing = 8
```

**Rust Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceDefinition {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub layout: LayoutDefinition,
    pub matrix_map: HashMap<u16, [u8; 2]>, // scancode -> [row, col]
    pub visual: Option<VisualMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutDefinition {
    #[serde(rename = "type")]
    pub layout_type: String, // "Matrix", "Standard", "Split"
    pub rows: u8,
    pub cols: Option<u8>,
    pub cols_per_row: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisualMetadata {
    pub key_width: u16,
    pub key_height: u16,
    pub key_spacing: u8,
}
```

**Tests:**
- TOML parsing tests with sample definitions
- Validation tests (invalid values)

**Acceptance Criteria:**
- [x] TOML files parse correctly
- [x] All required fields validated
- [x] Optional fields handled correctly

---

#### Task 3.1.2: Implement Device Definition Library
**File:** `core/src/definitions/library.rs` (NEW)

**Implementation:**
```rust
pub struct DeviceDefinitionLibrary {
    definitions: HashMap<(u16, u16), DeviceDefinition>, // (VID, PID) -> Definition
}

impl DeviceDefinitionLibrary {
    pub fn load_from_directory(dir: &Path) -> Result<Self> {
        let mut definitions = HashMap::new();

        // Recursively find all .toml files
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.path().extension() == Some(OsStr::new("toml")) {
                let definition = Self::load_definition(entry.path())?;
                let key = (definition.vendor_id, definition.product_id);
                definitions.insert(key, definition);
            }
        }

        Ok(Self { definitions })
    }

    fn load_definition(path: &Path) -> Result<DeviceDefinition> {
        let contents = std::fs::read_to_string(path)?;
        let definition: DeviceDefinition = toml::from_str(&contents)?;
        Self::validate_definition(&definition)?;
        Ok(definition)
    }

    fn validate_definition(def: &DeviceDefinition) -> Result<()> {
        // Validate rows/cols are non-zero
        // Validate matrix_map contains valid positions
        // ... validation logic
    }

    pub fn find_definition(&self, vid: u16, pid: u16) -> Option<&DeviceDefinition> {
        self.definitions.get(&(vid, pid))
    }

    pub fn list_definitions(&self) -> Vec<&DeviceDefinition> {
        self.definitions.values().collect()
    }
}
```

**Tests:**
- Directory loading tests
- VID:PID lookup tests
- Missing definition handling

**Acceptance Criteria:**
- [x] Loads all .toml files from directory tree
- [x] VID:PID lookup is O(1)
- [x] Handles missing definitions gracefully

---

#### Task 3.1.3: Create Standard Device Definitions
**Directory:** `device_definitions/`

**Files to Create:**
1. `standard/ansi-104.toml` - Standard ANSI keyboard
2. `standard/iso-105.toml` - Standard ISO keyboard
3. `elgato/stream-deck-mk2.toml` - Stream Deck MK.2 (3×5)
4. `elgato/stream-deck-xl.toml` - Stream Deck XL (4×8)
5. `elgato/stream-deck-mini.toml` - Stream Deck Mini (2×3)
6. `custom/macro-pad-5x5.toml` - Generic 5×5 macro pad

**Example: Stream Deck MK.2**
```toml
name = "Elgato Stream Deck MK.2"
vendor_id = 0x0fd9
product_id = 0x0080
manufacturer = "Elgato"

[layout]
type = "Matrix"
rows = 3
cols = 5

[matrix_map]
0x01 = [0, 0]
0x02 = [0, 1]
0x03 = [0, 2]
0x04 = [0, 3]
0x05 = [0, 4]
0x06 = [1, 0]
0x07 = [1, 1]
0x08 = [1, 2]
0x09 = [1, 3]
0x0A = [1, 4]
0x0B = [2, 0]
0x0C = [2, 1]
0x0D = [2, 2]
0x0E = [2, 3]
0x0F = [2, 4]

[visual]
key_width = 72
key_height = 72
key_spacing = 8
```

**Acceptance Criteria:**
- [x] 6+ device definitions created
- [x] All definitions validated and tested
- [x] README.md with contribution guidelines

---

### 3.2. Phase 3 Deliverables

**Files Created:**
- `core/src/definitions/types.rs`
- `core/src/definitions/library.rs`
- `core/src/definitions/loader.rs`
- `device_definitions/standard/*.toml` (3 files)
- `device_definitions/elgato/*.toml` (3 files)
- `device_definitions/README.md`

**Tests:**
- 30+ unit tests
- Real device validation tests

**Documentation:**
- Device definition format specification
- Community contribution guide

---

## Phase 4: FFI Layer for UI Integration

**Duration:** 1.5 weeks
**Goal:** Expose new registry system to Flutter UI via FFI

### 4.1. FFI Functions

#### Task 4.1.1: Device Registry FFI
**File:** `core/src/ffi/domains/device_registry.rs` (NEW)

**Functions to Implement:**
```rust
#[no_mangle]
pub extern "C" fn krx_device_registry_list_devices() -> *mut c_char {
    // Returns JSON array of DeviceState
}

#[no_mangle]
pub extern "C" fn krx_device_registry_set_remap_enabled(
    vendor_id: u16,
    product_id: u16,
    serial: *const c_char,
    enabled: bool,
) -> *mut c_char {
    // Returns "ok" or "error:message"
}

#[no_mangle]
pub extern "C" fn krx_device_registry_assign_profile(
    vendor_id: u16,
    product_id: u16,
    serial: *const c_char,
    profile_id: *const c_char,
) -> *mut c_char {
    // Assigns profile to device
}

#[no_mangle]
pub extern "C" fn krx_device_registry_set_user_label(
    vendor_id: u16,
    product_id: u16,
    serial: *const c_char,
    label: *const c_char,
) -> *mut c_char {
    // Sets user-friendly label
}

#[no_mangle]
pub extern "C" fn krx_device_registry_get_device_state(
    vendor_id: u16,
    product_id: u16,
    serial: *const c_char,
) -> *mut c_char {
    // Returns JSON DeviceState
}
```

**Tests:**
- FFI boundary tests (null pointers, invalid UTF-8)
- Memory leak tests
- Panic safety tests

**Acceptance Criteria:**
- [x] All functions callable from Dart
- [x] No memory leaks
- [x] Panics are caught and returned as errors

---

#### Task 4.1.2: Profile Registry FFI
**File:** `core/src/ffi/domains/profile_registry.rs` (NEW)

**Functions:**
```rust
#[no_mangle]
pub extern "C" fn krx_profile_registry_list_profiles() -> *mut c_char;

#[no_mangle]
pub extern "C" fn krx_profile_registry_get_profile(profile_id: *const c_char) -> *mut c_char;

#[no_mangle]
pub extern "C" fn krx_profile_registry_save_profile(profile_json: *const c_char) -> *mut c_char;

#[no_mangle]
pub extern "C" fn krx_profile_registry_delete_profile(profile_id: *const c_char) -> *mut c_char;

#[no_mangle]
pub extern "C" fn krx_profile_registry_find_compatible_profiles(
    layout_json: *const c_char
) -> *mut c_char;
```

**Acceptance Criteria:**
- [x] All CRUD operations exposed
- [x] Profile compatibility check works
- [x] JSON parsing errors are handled

---

#### Task 4.1.3: Device Definitions FFI
**File:** `core/src/ffi/domains/device_definitions.rs` (NEW)

**Functions:**
```rust
#[no_mangle]
pub extern "C" fn krx_definitions_list_all() -> *mut c_char;

#[no_mangle]
pub extern "C" fn krx_definitions_get_for_device(
    vendor_id: u16,
    product_id: u16
) -> *mut c_char;
```

**Acceptance Criteria:**
- [x] Device definitions accessible from UI
- [x] Used for layout visualization

---

### 4.2. Dart FFI Bindings

#### Task 4.2.1: Device Registry Bindings
**File:** `ui/lib/ffi/device_registry_ffi.dart` (NEW)

**Implementation:**
```dart
class DeviceRegistryFFI {
  final DynamicLibrary _lib;

  DeviceRegistryFFI(this._lib);

  late final _listDevices = _lib.lookupFunction<
    Pointer<Utf8> Function(),
    Pointer<Utf8> Function()
  >('krx_device_registry_list_devices');

  // ... other function lookups

  Future<List<DeviceState>> listDevices() async {
    final resultPtr = _listDevices();
    final result = resultPtr.toDartString();
    calloc.free(resultPtr);

    final json = jsonDecode(result) as List;
    return json.map((e) => DeviceState.fromJson(e)).toList();
  }

  Future<void> setRemapEnabled(DeviceIdentity device, bool enabled) async {
    final serialPtr = device.serialNumber.toNativeUtf8();
    final resultPtr = _setRemapEnabled(
      device.vendorId,
      device.productId,
      serialPtr,
      enabled,
    );
    calloc.free(serialPtr);

    final result = resultPtr.toDartString();
    calloc.free(resultPtr);

    if (result.startsWith('error:')) {
      throw Exception(result.substring(6));
    }
  }

  // ... other methods
}
```

**Tests:**
- Mock FFI tests
- Error handling tests

**Acceptance Criteria:**
- [x] All Rust FFI functions wrapped
- [x] Type-safe Dart API
- [x] Proper memory management

---

#### Task 4.2.2: Profile Registry Bindings
**File:** `ui/lib/ffi/profile_registry_ffi.dart` (NEW)

Similar to device registry bindings.

---

#### Task 4.2.3: Service Layer
**File:** `ui/lib/services/device_registry_service.dart` (NEW)

**Purpose:** High-level API wrapping FFI calls.

```dart
class DeviceRegistryService {
  final DeviceRegistryFFI _ffi;

  DeviceRegistryService(this._ffi);

  Future<List<DeviceState>> getDevices() async {
    return await _ffi.listDevices();
  }

  Future<void> toggleRemap(DeviceIdentity device, bool enabled) async {
    await _ffi.setRemapEnabled(device, enabled);
  }

  Future<void> assignProfile(DeviceIdentity device, String profileId) async {
    await _ffi.assignProfile(device, profileId);
  }

  Future<void> setUserLabel(DeviceIdentity device, String label) async {
    await _ffi.setUserLabel(device, label);
  }
}
```

**Acceptance Criteria:**
- [x] Clean API for UI layer
- [x] Error handling with user-friendly messages

---

### 4.3. Phase 4 Deliverables

**Rust Files Created:**
- `core/src/ffi/domains/device_registry.rs`
- `core/src/ffi/domains/profile_registry.rs`
- `core/src/ffi/domains/device_definitions.rs`

**Dart Files Created:**
- `ui/lib/ffi/device_registry_ffi.dart`
- `ui/lib/ffi/profile_registry_ffi.dart`
- `ui/lib/ffi/device_definitions_ffi.dart`
- `ui/lib/services/device_registry_service.dart`
- `ui/lib/services/profile_registry_service.dart`
- `ui/lib/models/device_identity.dart`
- `ui/lib/models/device_state.dart`
- `ui/lib/models/profile.dart`

**Tests:**
- 40+ Rust FFI tests
- 30+ Dart unit tests

---

## Phase 5: UI Overhaul - Devices Tab & Navigation

**Duration:** 2 weeks
**Goal:** Implement hardware-first navigation and device management UI

### 5.1. Navigation Restructure

#### Task 5.1.1: Reorder Navigation Items
**Modified File:** `ui/lib/main.dart`

**Change:**
```dart
// OLD order:
// 0. Editor
// 1. Devices
// 2. Run
// 3. Debugger
// 4. Console
// 5. Timing

// NEW order:
// 0. Devices      <-- MOVED TO TOP
// 1. Editor
// 2. Run
// 3. Debugger
// 4. Console
// 5. Timing
```

**Implementation:**
```dart
final List<NavigationRailDestination> _destinations = [
  NavigationRailDestination(
    icon: Icon(Icons.devices),
    label: Text('Devices'),
  ),
  NavigationRailDestination(
    icon: Icon(Icons.edit),
    label: Text('Editor'),
  ),
  // ... rest
];
```

**Acceptance Criteria:**
- [x] Devices tab is first in navigation
- [x] Default page on app launch is Devices tab
- [x] User workflow: Devices → Editor → Run

---

### 5.2. Devices Tab UI

#### Task 5.2.1: Create DeviceCard Widget
**File:** `ui/lib/widgets/device_card.dart` (NEW)

**Design:**
```
┌────────────────────────────────────────────────┐
│ 🎛️ Work Stream Deck                            │
│ VID: 0fd9  PID: 0080  Serial: ABC123          │
│ ┌──────────────────────┐  ┌────────────────┐  │
│ │ Profile: OBS Setup ▼ │  │ Remap: [🔄 ON] │  │
│ └──────────────────────┘  └────────────────┘  │
│ [Edit Label] [Manage Profiles]                │
└────────────────────────────────────────────────┘
```

**Implementation:**
```dart
class DeviceCard extends StatelessWidget {
  final DeviceState deviceState;
  final Function(bool) onRemapToggle;
  final Function(String) onProfileSelect;
  final Function() onEditLabel;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Device icon + label/name
            Row(
              children: [
                Icon(Icons.device_hub, size: 32),
                SizedBox(width: 8),
                Text(
                  deviceState.identity.userLabel ?? 'Unknown Device',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ],
            ),
            SizedBox(height: 8),
            // VID:PID:Serial info
            Text(
              'VID: ${deviceState.identity.vendorId.toRadixString(16).padLeft(4, '0')}  '
              'PID: ${deviceState.identity.productId.toRadixString(16).padLeft(4, '0')}  '
              'Serial: ${deviceState.identity.serialNumber}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            SizedBox(height: 16),
            // Profile selector + Remap toggle
            Row(
              children: [
                Expanded(
                  child: ProfileSelector(
                    selectedProfileId: deviceState.activeProfileId,
                    onSelect: onProfileSelect,
                  ),
                ),
                SizedBox(width: 16),
                RemapToggle(
                  enabled: deviceState.isRemappingEnabled,
                  onChanged: onRemapToggle,
                ),
              ],
            ),
            SizedBox(height: 8),
            // Action buttons
            Row(
              children: [
                TextButton.icon(
                  icon: Icon(Icons.edit),
                  label: Text('Edit Label'),
                  onPressed: onEditLabel,
                ),
                SizedBox(width: 8),
                TextButton.icon(
                  icon: Icon(Icons.list),
                  label: Text('Manage Profiles'),
                  onPressed: () { /* Navigate to profiles page */ },
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
```

**Acceptance Criteria:**
- [x] Shows device identity clearly
- [x] Profile selector functional
- [x] Remap toggle works
- [x] User label editing works

---

#### Task 5.2.2: Create RemapToggle Widget
**File:** `ui/lib/widgets/remap_toggle.dart` (NEW)

**Implementation:**
```dart
class RemapToggle extends StatelessWidget {
  final bool enabled;
  final Function(bool) onChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('Remap:'),
        SizedBox(width: 8),
        Switch(
          value: enabled,
          onChanged: onChanged,
          activeColor: Colors.green,
        ),
        Text(enabled ? 'ON' : 'OFF'),
      ],
    );
  }
}
```

**Acceptance Criteria:**
- [x] Visual state matches device state
- [x] Toggle triggers FFI call
- [x] Immediate UI feedback

---

#### Task 5.2.3: Create ProfileSelector Widget
**File:** `ui/lib/widgets/profile_selector.dart` (NEW)

**Implementation:**
```dart
class ProfileSelector extends StatelessWidget {
  final String? selectedProfileId;
  final Function(String) onSelect;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<Profile>>(
      future: context.read<ProfileRegistryService>().listProfiles(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return CircularProgressIndicator();
        }

        final profiles = snapshot.data!;

        return DropdownButton<String>(
          value: selectedProfileId,
          hint: Text('Select Profile'),
          isExpanded: true,
          items: profiles.map((profile) {
            return DropdownMenuItem(
              value: profile.id,
              child: Text(profile.name),
            );
          }).toList(),
          onChanged: (value) {
            if (value != null) {
              onSelect(value);
            }
          },
        );
      },
    );
  }
}
```

**Acceptance Criteria:**
- [x] Lists all available profiles
- [x] Shows currently selected profile
- [x] Profile selection triggers assignment

---

#### Task 5.2.4: Rebuild DevicesPage
**Modified File:** `ui/lib/pages/devices_page.dart`

**New Implementation:**
```dart
class DevicesPage extends StatefulWidget {
  @override
  _DevicesPageState createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  late final DeviceRegistryService _deviceService;

  @override
  void initState() {
    super.initState();
    _deviceService = context.read<DeviceRegistryService>();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Connected Devices'),
        actions: [
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: () => setState(() {}),
            tooltip: 'Refresh device list',
          ),
        ],
      ),
      body: FutureBuilder<List<DeviceState>>(
        future: _deviceService.getDevices(),
        builder: (context, snapshot) {
          if (!snapshot.hasData) {
            return Center(child: CircularProgressIndicator());
          }

          final devices = snapshot.data!;

          if (devices.isEmpty) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.devices_other, size: 64, color: Colors.grey),
                  SizedBox(height: 16),
                  Text('No devices connected', style: TextStyle(fontSize: 18)),
                  SizedBox(height: 8),
                  Text('Connect a device to get started'),
                ],
              ),
            );
          }

          return ListView.builder(
            padding: EdgeInsets.all(16),
            itemCount: devices.length,
            itemBuilder: (context, index) {
              final device = devices[index];
              return DeviceCard(
                deviceState: device,
                onRemapToggle: (enabled) async {
                  await _deviceService.toggleRemap(device.identity, enabled);
                  setState(() {});
                },
                onProfileSelect: (profileId) async {
                  await _deviceService.assignProfile(device.identity, profileId);
                  setState(() {});
                },
                onEditLabel: () => _showEditLabelDialog(device),
              );
            },
          );
        },
      ),
    );
  }

  Future<void> _showEditLabelDialog(DeviceState device) async {
    final controller = TextEditingController(text: device.identity.userLabel);
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Edit Device Label'),
        content: TextField(
          controller: controller,
          decoration: InputDecoration(labelText: 'Label'),
        ),
        actions: [
          TextButton(
            child: Text('Cancel'),
            onPressed: () => Navigator.pop(context),
          ),
          TextButton(
            child: Text('Save'),
            onPressed: () => Navigator.pop(context, controller.text),
          ),
        ],
      ),
    );

    if (result != null) {
      await _deviceService.setUserLabel(device.identity, result);
      setState(() {});
    }
  }
}
```

**Acceptance Criteria:**
- [x] Lists all connected devices
- [x] Shows empty state when no devices
- [x] Refresh button updates list
- [x] All device controls functional

---

### 5.3. Phase 5 Deliverables

**Files Created:**
- `ui/lib/widgets/device_card.dart`
- `ui/lib/widgets/remap_toggle.dart`
- `ui/lib/widgets/profile_selector.dart`

**Files Modified:**
- `ui/lib/main.dart`
- `ui/lib/pages/devices_page.dart`

**Tests:**
- 25+ widget tests
- 10+ integration tests

**UX Validation:**
- User can see all connected devices
- User can toggle remapping per device
- User can assign profiles via dropdown
- User can edit device labels

---

## Phase 6: Visual Editor Enhancement - Dynamic Layouts

**Duration:** 2.5 weeks
**Goal:** Support dynamic row-col layouts and soft keyboard palette

### 6.1. Layout Renderer

#### Task 6.1.1: Create LayoutGrid Widget
**File:** `ui/lib/widgets/layout_grid.dart` (NEW)

**Purpose:** Render device-specific layouts (5×5 matrix, ANSI keyboard, etc.)

**Implementation:**
```dart
class LayoutGrid extends StatelessWidget {
  final LayoutType layoutType;
  final Map<PhysicalPosition, KeyAction> mappings;
  final Function(PhysicalPosition) onKeyTap;

  @override
  Widget build(BuildContext context) {
    switch (layoutType) {
      case MatrixLayout layout:
        return _buildMatrixLayout(layout);
      case StandardLayout layout:
        return _buildStandardLayout(layout);
      case SplitLayout layout:
        return _buildSplitLayout(layout);
    }
  }

  Widget _buildMatrixLayout(MatrixLayout layout) {
    return GridView.builder(
      shrinkWrap: true,
      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: layout.cols,
        childAspectRatio: 1.0,
        crossAxisSpacing: 8,
        mainAxisSpacing: 8,
      ),
      itemCount: layout.rows * layout.cols,
      itemBuilder: (context, index) {
        final row = index ~/ layout.cols;
        final col = index % layout.cols;
        final position = PhysicalPosition(row: row, col: col);
        final mapping = mappings[position];

        return KeyButton(
          position: position,
          action: mapping,
          onTap: () => onKeyTap(position),
        );
      },
    );
  }

  // ... other layout builders
}
```

**Acceptance Criteria:**
- [x] Renders matrix layouts (5×5, 3×5, etc.)
- [x] Renders standard keyboard layouts
- [x] Shows current mappings on keys
- [x] Keys are clickable for editing

---

#### Task 6.1.2: Create SoftKeyboard Widget
**File:** `ui/lib/widgets/soft_keyboard.dart` (NEW)

**Purpose:** Virtual keyboard palette showing all available output keycodes.

**Design:**
```
┌──────────────────────────────────────────┐
│ Soft Keyboard (Output Keys)             │
├──────────────────────────────────────────┤
│ [A] [B] [C] [D] [E] [F] [G] [H] [I] ... │
│ [1] [2] [3] [4] [5] [6] [7] [8] [9] [0] │
│ [F1][F2][F3][F4][F5][F6][F7][F8]...     │
│ [Ctrl] [Shift] [Alt] [Win] ...          │
│ [Esc] [Tab] [Enter] [Backspace] ...     │
│                                          │
│ Search: [_______________________]        │
└──────────────────────────────────────────┘
```

**Implementation:**
```dart
class SoftKeyboard extends StatefulWidget {
  final Function(KeyCode) onKeySelect;

  @override
  _SoftKeyboardState createState() => _SoftKeyboardState();
}

class _SoftKeyboardState extends State<SoftKeyboard> {
  String _searchQuery = '';
  final List<KeyCode> _allKeys = KeyCode.values; // Assuming KeyCode enum exists

  List<KeyCode> get _filteredKeys {
    if (_searchQuery.isEmpty) return _allKeys;
    return _allKeys.where((key) =>
      key.name.toLowerCase().contains(_searchQuery.toLowerCase())
    ).toList();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Search bar
        Padding(
          padding: EdgeInsets.all(8),
          child: TextField(
            decoration: InputDecoration(
              labelText: 'Search keys',
              prefixIcon: Icon(Icons.search),
            ),
            onChanged: (value) => setState(() => _searchQuery = value),
          ),
        ),
        // Key grid
        Expanded(
          child: GridView.builder(
            padding: EdgeInsets.all(8),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 10,
              childAspectRatio: 1.5,
              crossAxisSpacing: 4,
              mainAxisSpacing: 4,
            ),
            itemCount: _filteredKeys.length,
            itemBuilder: (context, index) {
              final key = _filteredKeys[index];
              return GestureDetector(
                onTap: () => widget.onKeySelect(key),
                child: Container(
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.grey),
                    borderRadius: BorderRadius.circular(4),
                    color: Colors.white,
                  ),
                  alignment: Alignment.center,
                  child: Text(
                    key.displayName,
                    style: TextStyle(fontSize: 12),
                    textAlign: TextAlign.center,
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}
```

**Acceptance Criteria:**
- [x] Shows all available keycodes
- [x] Search/filter functionality
- [x] Keys are selectable
- [x] Responsive layout

---

#### Task 6.1.3: Implement Drag-and-Drop Mapping
**File:** `ui/lib/widgets/drag_drop_mapper.dart` (MODIFIED)

**Interaction:**
1. User drags from LayoutGrid (physical position)
2. User drops on SoftKeyboard key (output keycode)
3. Mapping is created: (row, col) → KeyCode

**Implementation:**
```dart
class DragDropMapper extends StatefulWidget {
  final Profile profile;
  final Function(Profile) onProfileUpdate;

  @override
  _DragDropMapperState createState() => _DragDropMapperState();
}

class _DragDropMapperState extends State<DragDropMapper> {
  Map<PhysicalPosition, KeyAction> _mappings = {};

  @override
  void initState() {
    super.initState();
    _mappings = Map.from(widget.profile.mappings);
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        // Left panel: Device layout
        Expanded(
          flex: 2,
          child: Column(
            children: [
              Text('Device Layout', style: Theme.of(context).textTheme.titleLarge),
              SizedBox(height: 8),
              Expanded(
                child: LayoutGrid(
                  layoutType: widget.profile.layoutType,
                  mappings: _mappings,
                  onKeyTap: _onPhysicalKeyTap,
                ),
              ),
            ],
          ),
        ),
        SizedBox(width: 16),
        // Right panel: Soft keyboard
        Expanded(
          flex: 3,
          child: Column(
            children: [
              Text('Output Keys', style: Theme.of(context).textTheme.titleLarge),
              SizedBox(height: 8),
              Expanded(
                child: SoftKeyboard(
                  onKeySelect: _onOutputKeySelect,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  PhysicalPosition? _selectedPosition;

  void _onPhysicalKeyTap(PhysicalPosition position) {
    setState(() {
      _selectedPosition = position;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Selected ${position.row},${position.col}. Now select output key.')),
    );
  }

  void _onOutputKeySelect(KeyCode keyCode) {
    if (_selectedPosition == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Select a physical key first')),
      );
      return;
    }

    setState(() {
      _mappings[_selectedPosition!] = KeyAction.Key(keyCode);
      _selectedPosition = null;
    });

    _saveProfile();
  }

  Future<void> _saveProfile() async {
    final updatedProfile = widget.profile.copyWith(
      mappings: _mappings,
      modifiedAt: DateTime.now(),
    );
    widget.onProfileUpdate(updatedProfile);
  }
}
```

**Acceptance Criteria:**
- [x] User can select physical key
- [x] User can select output key
- [x] Mapping is created and displayed
- [x] Profile is auto-saved

---

#### Task 6.1.4: Rebuild VisualEditorPage
**Modified File:** `ui/lib/pages/visual_editor_page.dart`

**Major Changes:**
1. Profile selection dropdown at top (instead of device selection)
2. Dynamic layout rendering based on profile's layout_type
3. Drag-drop mapper instead of static keyboard

**Implementation:**
```dart
class VisualEditorPage extends StatefulWidget {
  @override
  _VisualEditorPageState createState() => _VisualEditorPageState();
}

class _VisualEditorPageState extends State<VisualEditorPage> {
  late final ProfileRegistryService _profileService;
  String? _selectedProfileId;
  Profile? _currentProfile;

  @override
  void initState() {
    super.initState();
    _profileService = context.read<ProfileRegistryService>();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Visual Editor'),
        actions: [
          IconButton(
            icon: Icon(Icons.add),
            onPressed: _createNewProfile,
            tooltip: 'Create new profile',
          ),
        ],
      ),
      body: Column(
        children: [
          // Profile selector
          Padding(
            padding: EdgeInsets.all(16),
            child: FutureBuilder<List<Profile>>(
              future: _profileService.listProfiles(),
              builder: (context, snapshot) {
                if (!snapshot.hasData) {
                  return CircularProgressIndicator();
                }

                final profiles = snapshot.data!;

                return DropdownButton<String>(
                  value: _selectedProfileId,
                  hint: Text('Select a profile to edit'),
                  isExpanded: true,
                  items: profiles.map((p) {
                    return DropdownMenuItem(
                      value: p.id,
                      child: Text(p.name),
                    );
                  }).toList(),
                  onChanged: (value) async {
                    if (value != null) {
                      final profile = await _profileService.getProfile(value);
                      setState(() {
                        _selectedProfileId = value;
                        _currentProfile = profile;
                      });
                    }
                  },
                );
              },
            ),
          ),
          Divider(),
          // Mapper
          if (_currentProfile != null)
            Expanded(
              child: DragDropMapper(
                profile: _currentProfile!,
                onProfileUpdate: (updatedProfile) async {
                  await _profileService.saveProfile(updatedProfile);
                  setState(() {
                    _currentProfile = updatedProfile;
                  });
                },
              ),
            )
          else
            Expanded(
              child: Center(
                child: Text('Select a profile to start editing'),
              ),
            ),
        ],
      ),
    );
  }

  Future<void> _createNewProfile() async {
    // Show dialog to create new profile (name, layout type)
    // ...
  }
}
```

**Acceptance Criteria:**
- [x] Profile dropdown works
- [x] Creating new profile works
- [x] Dynamic layout renders correctly
- [x] Mappings persist on save

---

### 6.2. Phase 6 Deliverables

**Files Created:**
- `ui/lib/widgets/layout_grid.dart`
- `ui/lib/widgets/soft_keyboard.dart`
- `ui/lib/widgets/drag_drop_mapper.dart`
- `ui/lib/widgets/key_button.dart`

**Files Modified:**
- `ui/lib/pages/visual_editor_page.dart`

**Tests:**
- 40+ widget tests
- 15+ integration tests
- Visual regression tests (golden files)

**UX Validation:**
- User can edit profiles with different layouts
- 5×5 macro pad renders correctly
- ANSI keyboard renders correctly
- Mappings are intuitive to create

---

## Phase 7: Migration & Testing

**Duration:** 1 week
**Goal:** Migrate existing users and comprehensive testing

### 7.1. Data Migration

#### Task 7.1.1: Implement Migration Script
**File:** `core/src/migration/v1_to_v2.rs` (NEW)

**Purpose:** Convert old `{vid}_{pid}.json` profiles to new system.

**Implementation:**
```rust
pub struct MigrationV1ToV2 {
    old_config_dir: PathBuf,
    profile_registry: Arc<RwLock<ProfileRegistry>>,
    device_bindings: Arc<RwLock<DeviceBindings>>,
}

impl MigrationV1ToV2 {
    pub async fn migrate(&self) -> Result<MigrationReport> {
        let old_profiles = self.scan_old_profiles()?;
        let mut report = MigrationReport::default();

        for old_profile in old_profiles {
            // Convert to new Profile
            let new_profile = self.convert_profile(&old_profile)?;
            self.profile_registry.write().await.save_profile(new_profile.clone()).await?;
            report.profiles_migrated += 1;

            // Create device binding if device is connected
            if let Some(device) = self.find_connected_device(old_profile.vendor_id, old_profile.product_id).await {
                let binding = DeviceBinding {
                    active_profile_id: Some(new_profile.id.clone()),
                    is_remapping_enabled: false, // Safe default
                    last_connected: Utc::now(),
                };
                self.device_bindings.write().await.set_binding(device.identity, binding);
                report.bindings_created += 1;
            }
        }

        // Persist bindings
        self.device_bindings.read().await.save()?;

        Ok(report)
    }

    fn convert_profile(&self, old: &OldDeviceProfile) -> Result<Profile> {
        Ok(Profile {
            id: Uuid::new_v4().to_string(),
            name: old.name.clone().unwrap_or_else(||
                format!("Migrated {:04X}:{:04X}", old.vendor_id, old.product_id)),
            layout_type: LayoutType::Matrix {
                rows: old.rows,
                cols: old.cols_per_row.iter().max().copied().unwrap_or(0),
            },
            mappings: self.convert_mappings(&old.keymap)?,
            created_at: old.discovered_at,
            modified_at: Utc::now(),
            description: Some("Migrated from v1".to_string()),
            tags: vec!["migrated".to_string()],
            script_source: None,
        })
    }

    fn convert_mappings(&self, old_keymap: &HashMap<u16, PhysicalKey>) -> Result<HashMap<PhysicalPosition, KeyAction>> {
        let mut mappings = HashMap::new();
        for (scancode, physical_key) in old_keymap {
            let position = PhysicalPosition {
                row: physical_key.row,
                col: physical_key.col,
            };
            if let Some(keycode) = physical_key.keycode {
                mappings.insert(position, KeyAction::Key(keycode));
            }
        }
        Ok(mappings)
    }
}

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub profiles_migrated: usize,
    pub bindings_created: usize,
    pub errors: Vec<String>,
}
```

**CLI Command:**
```bash
keyrx migrate --from v1 --backup
```

**Acceptance Criteria:**
- [x] All old profiles converted
- [x] Device bindings created for connected devices
- [x] Backup created before migration
- [x] Migration is idempotent (can run multiple times)

---

#### Task 7.1.2: Migration UI Prompt
**File:** `ui/lib/pages/migration_prompt_page.dart` (NEW)

**Purpose:** Show migration dialog on first launch with new version.

**Design:**
```
┌──────────────────────────────────────┐
│ Welcome to KeyRx 2.0!                │
│                                      │
│ We've detected profiles from the     │
│ previous version. Would you like to  │
│ migrate them to the new system?      │
│                                      │
│ This will:                           │
│ • Convert existing profiles          │
│ • Create device bindings             │
│ • Preserve all your mappings         │
│                                      │
│ [Cancel]    [Backup & Migrate]       │
└──────────────────────────────────────┘
```

**Acceptance Criteria:**
- [x] Shows only on first launch after upgrade
- [x] User can skip migration
- [x] Backup is created automatically
- [x] Migration progress is shown

---

### 7.2. Testing & Validation

#### Task 7.2.1: End-to-End Tests
**File:** `core/tests/e2e_revolutionary_mapping.rs` (NEW)

**Test Scenarios:**
1. Connect two identical devices → Assign different profiles → Verify isolation
2. Swap profile on device → Verify immediate behavior change
3. Toggle remap per device → Verify passthrough mode
4. Disconnect/reconnect device → Verify profile persists
5. Create 5×5 profile → Assign to macro pad → Verify layout renders

**Acceptance Criteria:**
- [x] All scenarios pass
- [x] No memory leaks
- [x] <1ms latency maintained

---

#### Task 7.2.2: Performance Benchmarks
**File:** `core/benches/revolutionary_mapping_bench.rs` (NEW)

**Benchmarks:**
- Device resolution time
- Profile lookup time (cold + warm cache)
- Coordinate translation time
- Full pipeline latency

**Acceptance Criteria:**
- [x] Device resolution: <50μs (p99)
- [x] Profile lookup (cached): <10μs (p99)
- [x] Coordinate translation: <20μs (p99)
- [x] Full pipeline: <1ms (p99)

---

#### Task 7.2.3: User Acceptance Testing
**Scenarios:**
1. New user sets up first device
2. Power user manages 5+ devices
3. Content creator swaps profiles for different apps
4. User edits 5×5 macro pad profile

**Acceptance Criteria:**
- [x] 90%+ task completion rate
- [x] <5 min to complete basic setup
- [x] No major UX confusion

---

### 7.3. Phase 7 Deliverables

**Files Created:**
- `core/src/migration/v1_to_v2.rs`
- `ui/lib/pages/migration_prompt_page.dart`
- `core/tests/e2e_revolutionary_mapping.rs`
- `core/benches/revolutionary_mapping_bench.rs`
- `scripts/migrate_profiles.sh`

**Tests:**
- 30+ migration tests
- 50+ E2E tests
- 10+ performance benchmarks

**Documentation:**
- Migration guide for users
- Troubleshooting guide
- Performance report

---

## Implementation Summary

### Timeline Overview

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1 | 2 weeks | Device identity, Registries, Serial extraction |
| Phase 2 | 2 weeks | Pipeline integration, Multi-stage lookup |
| Phase 3 | 1.5 weeks | Device definitions, TOML library |
| Phase 4 | 1.5 weeks | FFI layer, Dart bindings |
| Phase 5 | 2 weeks | Devices tab UI, Navigation reorder |
| Phase 6 | 2.5 weeks | Visual editor, Dynamic layouts |
| Phase 7 | 1 week | Migration, Testing, Validation |
| **Total** | **12.5 weeks** | **Revolutionary Mapping System** |

### Risk Mitigation

**Technical Risks:**
1. **Serial number unavailability:** Fallback to synthetic IDs (port-bound)
2. **Performance regression:** Continuous benchmarking, caching strategies
3. **FFI panics:** Comprehensive panic guards, catch_unwind
4. **Migration failures:** Backup system, idempotent migration

**UX Risks:**
1. **User confusion:** Progressive disclosure, tooltips, onboarding
2. **Breaking changes:** Migration tool, backward compatibility where possible
3. **Learning curve:** Documentation, video tutorials

### Success Metrics

**Technical:**
- [x] <1ms input latency (p99)
- [x] Zero panics in production
- [x] 100% FFI memory safety
- [x] 90%+ test coverage

**Product:**
- [x] Multi-device management works
- [x] Profile swapping is instant
- [x] Custom layouts supported
- [x] User satisfaction >4.5/5

### Post-Implementation Tasks

1. **Documentation:**
   - User guide with screenshots
   - Video tutorials
   - API documentation
   - Device definition contribution guide

2. **Community Engagement:**
   - Announce revolutionary mapping
   - Collect device definitions from community
   - Build profile marketplace

3. **Continuous Improvement:**
   - Monitor telemetry (opt-in)
   - Fix bugs from user reports
   - Add requested features
   - Expand device library

---

This implementation spec provides a complete, actionable plan for implementing the Revolutionary Mapping Architecture in KeyRx.
