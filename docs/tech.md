# KeyRx Technical Architecture

**Version:** 2.0 - Revolutionary Mapping Architecture
**Last Updated:** 2025-12-06

---

## 1. System Architecture Overview

KeyRx implements a revolutionary device-profile decoupling architecture that transforms input management from simple key remapping to a professional-grade configuration system.

### 1.1. Core Architectural Principles

**Separation of Concerns:**
- **Device Layer:** Physical hardware identification and I/O
- **Profile Layer:** Logical behavior definitions
- **Runtime Layer:** Active device-profile bindings
- **UI Layer:** User interaction and visualization

**Event-Driven Design:**
- Asynchronous input processing using Tokio
- Event sourcing for replay debugging
- Non-blocking multi-device coordination

**Platform Abstraction:**
- OS-specific drivers implement generic traits
- Core logic remains platform-agnostic
- Conditional compilation for platform features

---

## 2. The Great Decoupling: Data Model Architecture

### 2.1. Device Identity System

**Device Identity Tracking**

Devices are uniquely identified using a three-part composite key:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// USB Vendor ID (16-bit)
    pub vendor_id: u16,

    /// USB Product ID (16-bit)
    pub product_id: u16,

    /// Device serial number (hardware or synthetic)
    /// - Hardware: From USB iSerial descriptor
    /// - Synthetic: Generated from port topology + VID:PID hash
    pub serial_number: String,

    /// User-assigned semantic label (optional)
    /// Example: "Work Stream Deck", "Gaming Keyboard"
    pub user_label: Option<String>,
}
```

**Serial Number Acquisition Strategy:**

| Platform | Primary Method | Fallback |
|----------|----------------|----------|
| **Windows** | Parse PnP Device Interface Path (`\\?\HID#...#<InstanceID>#...`) | Port topology hash |
| **Linux** | `EVIOCGUNIQ` ioctl via evdev | udev `ID_SERIAL`, then `phys` hash |

**Implementation Location:** `core/src/discovery/identity.rs` (NEW)

### 2.2. Device Registry (Runtime State)

**Purpose:** Track connected devices and their current state.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    /// Unique device identity
    pub identity: DeviceIdentity,

    /// Is remapping currently enabled for this device?
    pub is_remapping_enabled: bool,

    /// Currently active profile ID (None = passthrough)
    pub active_profile_id: Option<ProfileId>,

    /// Connection timestamp
    pub connected_at: DateTime<Utc>,

    /// Device state
    pub state: DeviceRuntimeState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRuntimeState {
    /// Device is actively processing input
    Active,

    /// Device is connected but remapping disabled (passthrough)
    Passthrough,

    /// Device connection failed or profile load error
    Failed { error_code: u32 },
}

/// Device Registry maintains runtime state of all connected devices
pub struct DeviceRegistry {
    /// Map of DeviceIdentity -> DeviceState
    devices: HashMap<DeviceIdentity, DeviceState>,

    /// Event channel for device state changes
    event_tx: mpsc::UnboundedSender<DeviceEvent>,
}
```

**Responsibilities:**
- Track connected devices by identity
- Manage per-device remap enable/disable state
- Bind devices to active profiles
- Emit events on device connect/disconnect/state change

**Persistence:** In-memory only (rebuilt on application start from device detection).

**Implementation Location:** `core/src/registry/device.rs` (NEW)

### 2.3. Profile Registry (Persistent Configuration)

**Purpose:** Store and manage profile definitions independent of devices.

```rust
pub type ProfileId = String; // UUID or user-friendly name

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique profile identifier
    pub id: ProfileId,

    /// User-visible name
    pub name: String,

    /// Layout definition (determines device compatibility)
    pub layout_type: LayoutType,

    /// Key mappings: (Row, Col) -> Action
    pub mappings: HashMap<PhysicalPosition, KeyAction>,

    /// Creation and modification timestamps
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,

    /// Optional metadata
    pub description: Option<String>,
    pub tags: Vec<String>,

    /// Rhai script source (for advanced users)
    pub script_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    /// Standard keyboard layout (ANSI, ISO, JIS)
    Standard(StandardLayout),

    /// Custom matrix layout (rows × columns)
    Matrix { rows: u8, cols: u8 },

    /// Split keyboard (left and right halves)
    Split {
        left: Box<LayoutType>,
        right: Box<LayoutType>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalPosition {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAction {
    /// Single key output
    Key(KeyCode),

    /// Chord (e.g., Ctrl+C)
    Chord { modifiers: Vec<KeyCode>, key: KeyCode },

    /// Rhai script execution
    Script(String),

    /// Block (suppress key)
    Block,

    /// Passthrough (no remapping)
    Pass,
}

/// Profile Registry manages persistent profile storage
pub struct ProfileRegistry {
    /// Map of ProfileId -> Profile
    profiles: HashMap<ProfileId, Profile>,

    /// Storage backend (filesystem JSON)
    storage: Box<dyn ProfileStorage>,
}
```

**Responsibilities:**
- CRUD operations for profiles
- Profile validation (layout compatibility checks)
- Profile search and filtering
- Import/export profiles

**Persistence:** Filesystem JSON storage in `$XDG_CONFIG_HOME/keyrx/profiles/`
- One file per profile: `<profile_id>.json`
- Atomic writes using temp file + rename

**Implementation Location:** `core/src/registry/profile.rs` (NEW)

### 2.4. Device Definitions (Hardware Specifications)

**Purpose:** Describe physical properties of hardware to enable layout awareness.

**Format:** TOML (static data, not executable logic)

```toml
# Example: Stream Deck MK.2
# File: device_definitions/elgato/stream-deck-mk2.toml

name = "Stream Deck MK.2"
vendor_id = 0x0fd9
product_id = 0x0080
manufacturer = "Elgato"

[layout]
type = "Matrix"
rows = 3
cols = 5

# Physical -> Logical mapping
# Maps HID Usage ID or Scan Code to (Row, Col)
[matrix_map]
0x01 = [0, 0]
0x02 = [0, 1]
0x03 = [0, 2]
# ... (15 total keys)

[visual]
# Optional visual rendering hints
key_width = 72
key_height = 72
key_spacing = 8
```

**Standard Keyboard Example:**
```toml
# File: device_definitions/standard/ansi-104.toml

name = "Generic ANSI 104-Key Keyboard"
layout_type = "Standard"
standard_layout = "ANSI"

[layout]
type = "Standard"
rows = 6
cols_per_row = [15, 15, 15, 13, 12, 8]

# For standard keyboards, we use scancode -> (row, col) mapping
[scancode_map]
0x1E = [2, 1]  # A key
0x30 = [3, 2]  # B key
# ... (full scancode table)
```

**Device Definition Loader:**
```rust
pub struct DeviceDefinition {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub layout: LayoutDefinition,
    pub matrix_map: HashMap<u16, PhysicalPosition>, // HID usage/scancode -> (row, col)
}

pub struct DeviceDefinitionLibrary {
    definitions: HashMap<(u16, u16), DeviceDefinition>, // (VID, PID) -> Definition
}

impl DeviceDefinitionLibrary {
    /// Load all .toml files from device_definitions/ directory
    pub fn load_from_directory(path: &Path) -> Result<Self>;

    /// Find definition for a device
    pub fn find_definition(&self, vid: u16, pid: u16) -> Option<&DeviceDefinition>;
}
```

**Implementation Location:**
- `core/src/definitions/` (loader)
- `device_definitions/` (TOML files in project root)

**Why TOML vs Rhai:**
- TOML: Static data (safe for community sharing, instant parsing)
- Rhai: Dynamic logic (profiles, actions, complex behaviors)

---

## 3. Multi-Stage Input Processing Pipeline

### 3.1. Event Flow Architecture

```
┌───────────────────────────────────────────────────────────────┐
│  OS Input Event                                               │
│  (Keyboard/Device driver)                                     │
└─────────────────────────┬─────────────────────────────────────┘
                          │
                          ▼
┌───────────────────────────────────────────────────────────────┐
│  Platform Driver (Windows Raw Input / Linux evdev)            │
│  - Capture raw event                                          │
│  - Extract device handle                                      │
│  - Read scancode/usage + modifiers                           │
└─────────────────────────┬─────────────────────────────────────┘
                          │
                          ▼
┌───────────────────────────────────────────────────────────────┐
│  Device Resolution                                            │
│  - Device Handle -> DeviceIdentity (VID:PID:Serial)          │
│  - Lookup in DeviceRegistry                                   │
│  - Check is_remapping_enabled                                │
└─────────────────────────┬─────────────────────────────────────┘
                          │
                  ┌───────┴────────┐
                  │                │
                  ▼                ▼
        ┌─────────────┐   ┌──────────────┐
        │ Passthrough │   │ Active       │
        │ (Disabled)  │   │ (Enabled)    │
        └──────┬──────┘   └──────┬───────┘
               │                 │
               │                 ▼
               │      ┌───────────────────────────────┐
               │      │ Load Active Profile           │
               │      │ - Get active_profile_id       │
               │      │ - Load Profile from Registry  │
               │      └────────────┬──────────────────┘
               │                   │
               │                   ▼
               │      ┌───────────────────────────────┐
               │      │ Coordinate Translation        │
               │      │ - Load Device Definition      │
               │      │ - ScanCode -> (Row, Col)      │
               │      └────────────┬──────────────────┘
               │                   │
               │                   ▼
               │      ┌───────────────────────────────┐
               │      │ Action Resolution             │
               │      │ - Lookup (Row, Col) in Profile│
               │      │ - Get KeyAction               │
               │      └────────────┬──────────────────┘
               │                   │
               │                   ▼
               │      ┌───────────────────────────────┐
               │      │ Execution                     │
               │      │ - Execute KeyAction           │
               │      │ - Run Rhai script if needed   │
               │      └────────────┬──────────────────┘
               │                   │
               └───────────┬───────┘
                           │
                           ▼
              ┌────────────────────────────┐
              │ Output Injection           │
              │ - Synthesize OS event      │
              │ - Inject via uinput/SendInput │
              └────────────────────────────┘
```

### 3.2. Implementation: Multi-Stage Lookup

**Stage 1: Device Resolution** (`core/src/engine/device_resolver.rs`)
```rust
pub struct DeviceResolver {
    registry: Arc<RwLock<DeviceRegistry>>,
}

impl DeviceResolver {
    pub async fn resolve(&self, device_handle: RawDeviceHandle)
        -> Result<Option<DeviceState>>
    {
        // Platform-specific: extract identity from handle
        let identity = self.extract_identity(device_handle)?;

        // Lookup in registry
        let registry = self.registry.read().await;
        Ok(registry.get_device_state(&identity).cloned())
    }
}
```

**Stage 2: Profile Resolution** (`core/src/engine/profile_resolver.rs`)
```rust
pub struct ProfileResolver {
    profile_registry: Arc<RwLock<ProfileRegistry>>,
}

impl ProfileResolver {
    pub async fn resolve(&self, profile_id: &ProfileId)
        -> Result<Option<Arc<Profile>>>
    {
        let registry = self.profile_registry.read().await;
        Ok(registry.get_profile(profile_id).map(Arc::new))
    }
}
```

**Stage 3: Coordinate Translation** (`core/src/engine/coordinate_translator.rs`)
```rust
pub struct CoordinateTranslator {
    definitions: Arc<DeviceDefinitionLibrary>,
}

impl CoordinateTranslator {
    pub fn translate(&self,
        device_identity: &DeviceIdentity,
        scancode: u16)
        -> Result<PhysicalPosition>
    {
        let definition = self.definitions
            .find_definition(device_identity.vendor_id, device_identity.product_id)
            .ok_or(Error::NoDeviceDefinition)?;

        definition.matrix_map
            .get(&scancode)
            .copied()
            .ok_or(Error::UnmappedScancode(scancode))
    }
}
```

**Stage 4: Action Resolution** (`core/src/engine/action_resolver.rs`)
```rust
pub struct ActionResolver;

impl ActionResolver {
    pub fn resolve(&self,
        profile: &Profile,
        position: PhysicalPosition)
        -> Option<&KeyAction>
    {
        profile.mappings.get(&position)
    }
}
```

**Stage 5: Execution** (`core/src/engine/executor.rs`)
```rust
pub struct ActionExecutor {
    script_engine: RhaiEngine,
}

impl ActionExecutor {
    pub async fn execute(&self, action: &KeyAction) -> Result<Vec<OutputEvent>> {
        match action {
            KeyAction::Key(code) => {
                Ok(vec![OutputEvent::KeyPress(*code)])
            }
            KeyAction::Chord { modifiers, key } => {
                let mut events = vec![];
                for m in modifiers {
                    events.push(OutputEvent::KeyPress(*m));
                }
                events.push(OutputEvent::KeyPress(*key));
                for m in modifiers.iter().rev() {
                    events.push(OutputEvent::KeyRelease(*m));
                }
                Ok(events)
            }
            KeyAction::Script(script) => {
                self.script_engine.execute(script).await
            }
            KeyAction::Block => Ok(vec![]),
            KeyAction::Pass => Err(Error::PassthroughRequested),
        }
    }
}
```

---

## 4. Platform-Specific Serial Number Acquisition

### 4.1. Windows Implementation

**Raw Input API Device Name Format:**
```
\\?\HID#VID_vvvv&PID_pppp&MI_ii#<InstanceID>#{<ClassGUID>}
```

**Serial Extraction Strategy:**

```rust
// File: core/src/drivers/windows/serial.rs

use windows::Win32::Devices::HumanInterfaceDevices::*;
use windows::Win32::Storage::FileSystem::*;

pub fn extract_serial_number(device_path: &str) -> Result<String> {
    // Parse the device path to extract InstanceID
    let instance_id = parse_instance_id_from_path(device_path)?;

    // Attempt to read iSerial descriptor via HID API
    if let Ok(serial) = read_iserial_descriptor(device_path) {
        if !serial.is_empty() {
            return Ok(serial);
        }
    }

    // Fallback: Use InstanceID as serial
    // If device has iSerial, Windows uses it as InstanceID
    // If not, Windows generates port-based ID (acceptable for our use case)
    Ok(instance_id)
}

fn parse_instance_id_from_path(path: &str) -> Result<String> {
    // Example: \\?\HID#VID_046D&PID_C52B#7&2a8c1b3d&0&0000#{...}
    //                                        ^^^^^^^^^^^^^^^^ (InstanceID)

    let parts: Vec<&str> = path.split('#').collect();
    if parts.len() < 3 {
        return Err(Error::InvalidDevicePath);
    }

    Ok(parts[2].to_string())
}

fn read_iserial_descriptor(device_path: &str) -> Result<String> {
    unsafe {
        let handle = CreateFileW(
            device_path,
            FILE_GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            None,
        )?;

        // Query HID string descriptor index 3 (iSerial)
        let mut buffer = [0u16; 256];
        let success = HidD_GetSerialNumberString(
            handle,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() * 2,
        );

        CloseHandle(handle);

        if success.as_bool() {
            Ok(String::from_utf16_lossy(&buffer))
        } else {
            Err(Error::NoSerialDescriptor)
        }
    }
}
```

**Trade-off: Port-Bound Devices**
- Devices without iSerial: InstanceID changes when moved to different USB port
- User Warning: "This device doesn't have a serial number. Configuration is bound to USB port."
- UX Impact: Moving device to new port = new device entry in registry

### 4.2. Linux Implementation

**evdev Unique Name Acquisition:**

```rust
// File: core/src/drivers/linux/serial.rs

use evdev::{Device, InputId};
use std::path::Path;

pub fn extract_serial_number(device_path: &Path) -> Result<String> {
    let mut device = Device::open(device_path)?;

    // Primary method: EVIOCGUNIQ ioctl (unique_name)
    if let Some(unique) = device.unique_name() {
        if !unique.is_empty() {
            return Ok(unique.to_string());
        }
    }

    // Secondary method: udev properties
    if let Ok(serial) = read_udev_serial(device_path) {
        return Ok(serial);
    }

    // Fallback: Generate synthetic ID from phys path
    let phys = device.physical_path()
        .unwrap_or("unknown")
        .to_string();

    let input_id = device.input_id();
    let synthetic = format!(
        "synthetic_{:04x}{:04x}_{}",
        input_id.vendor(),
        input_id.product(),
        hash_phys_path(&phys)
    );

    Ok(synthetic)
}

fn read_udev_serial(device_path: &Path) -> Result<String> {
    // Read from /sys/class/input/eventX/device/id/serial or ID_SERIAL
    let sys_path = format!("/sys/class/input/{}/device/",
        device_path.file_name().unwrap().to_str().unwrap());

    // Try ID_SERIAL_SHORT first
    if let Ok(serial) = std::fs::read_to_string(format!("{}serial", sys_path)) {
        return Ok(serial.trim().to_string());
    }

    Err(Error::NoUdevSerial)
}

fn hash_phys_path(phys: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    phys.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
```

**Port-Bound Fallback:**
- `phys` path example: `usb-0000:00:14.0-3/input0`
- Port change = different `phys` = different synthetic serial
- Same trade-off as Windows for generic hardware

---

## 5. FFI Architecture for Flutter Integration

### 5.1. Domain-Based FFI Modules

KeyRx uses a modular FFI architecture where each domain exposes specific functionality:

**Existing Domains:**
- `device.rs` - Device listing, profile access
- `engine.rs` - Engine control
- `script.rs` - Script management
- `validation.rs` - Config validation
- `diagnostics.rs` - Debugging

**New Domains for Revolutionary Mapping:**

**Device Registry FFI** (`core/src/ffi/domains/device_registry.rs`)
```rust
#[no_mangle]
pub extern "C" fn krx_device_registry_list_devices() -> *mut c_char {
    // Returns JSON: [{ identity: {...}, state: {...}, active_profile_id: "..." }, ...]
}

#[no_mangle]
pub extern "C" fn krx_device_registry_set_remap_enabled(
    vendor_id: u16,
    product_id: u16,
    serial: *const c_char,
    enabled: bool,
) -> *mut c_char {
    // Returns "ok" or "error:reason"
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
    // Sets device user label
}
```

**Profile Registry FFI** (`core/src/ffi/domains/profile_registry.rs`)
```rust
#[no_mangle]
pub extern "C" fn krx_profile_registry_list_profiles() -> *mut c_char {
    // Returns JSON: [{ id: "...", name: "...", layout_type: {...}, ... }, ...]
}

#[no_mangle]
pub extern "C" fn krx_profile_registry_get_profile(
    profile_id: *const c_char,
) -> *mut c_char {
    // Returns full profile JSON
}

#[no_mangle]
pub extern "C" fn krx_profile_registry_save_profile(
    profile_json: *const c_char,
) -> *mut c_char {
    // Creates or updates profile
}

#[no_mangle]
pub extern "C" fn krx_profile_registry_delete_profile(
    profile_id: *const c_char,
) -> *mut c_char {
    // Deletes profile
}

#[no_mangle]
pub extern "C" fn krx_profile_registry_find_compatible_profiles(
    layout_type_json: *const c_char,
) -> *mut c_char {
    // Returns profiles matching layout type
}
```

**Device Definitions FFI** (`core/src/ffi/domains/device_definitions.rs`)
```rust
#[no_mangle]
pub extern "C" fn krx_definitions_list_all() -> *mut c_char {
    // Returns all loaded device definitions
}

#[no_mangle]
pub extern "C" fn krx_definitions_get_for_device(
    vendor_id: u16,
    product_id: u16,
) -> *mut c_char {
    // Returns device definition JSON or null
}
```

### 5.2. Flutter FFI Bindings

**Dart Bindings** (`ui/lib/ffi/device_registry_ffi.dart`)
```dart
import 'dart:ffi';
import 'package:ffi/ffi.dart';

class DeviceRegistryFFI {
  final DynamicLibrary _lib;

  late final _listDevices = _lib.lookupFunction<
    Pointer<Utf8> Function(),
    Pointer<Utf8> Function()
  >('krx_device_registry_list_devices');

  late final _setRemapEnabled = _lib.lookupFunction<
    Pointer<Utf8> Function(Uint16, Uint16, Pointer<Utf8>, Bool),
    Pointer<Utf8> Function(int, int, Pointer<Utf8>, bool)
  >('krx_device_registry_set_remap_enabled');

  // ...

  Future<List<DeviceState>> listDevices() async {
    final resultPtr = _listDevices();
    final result = resultPtr.toDartString();
    calloc.free(resultPtr);

    final json = jsonDecode(result);
    return (json as List).map((e) => DeviceState.fromJson(e)).toList();
  }

  Future<void> setRemapEnabled(
    DeviceIdentity identity,
    bool enabled
  ) async {
    final serialPtr = identity.serialNumber.toNativeUtf8();
    final resultPtr = _setRemapEnabled(
      identity.vendorId,
      identity.productId,
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
}
```

---

## 6. Testing Strategy

### 6.1. Serial Number Extraction Tests

**Unit Tests** (`core/src/drivers/windows/serial_tests.rs`)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instance_id_with_serial() {
        let path = r"\\?\HID#VID_046D&PID_C52B#ABC123456#{...}";
        assert_eq!(parse_instance_id_from_path(path).unwrap(), "ABC123456");
    }

    #[test]
    fn test_parse_instance_id_port_based() {
        let path = r"\\?\HID#VID_1234&PID_5678#7&2a8c1b3d&0&0000#{...}";
        assert_eq!(
            parse_instance_id_from_path(path).unwrap(),
            "7&2a8c1b3d&0&0000"
        );
    }
}
```

**Integration Tests** (`core/tests/device_identity_tests.rs`)
```rust
#[tokio::test]
async fn test_multi_device_distinction() {
    let registry = DeviceRegistry::new();

    // Simulate two identical devices with different serials
    let device1 = DeviceIdentity {
        vendor_id: 0x0fd9,
        product_id: 0x0080,
        serial_number: "ABC123".to_string(),
        user_label: Some("Work Deck".to_string()),
    };

    let device2 = DeviceIdentity {
        vendor_id: 0x0fd9,
        product_id: 0x0080,
        serial_number: "XYZ789".to_string(),
        user_label: Some("Stream Deck".to_string()),
    };

    registry.register_device(device1.clone(), DeviceState { ... }).await;
    registry.register_device(device2.clone(), DeviceState { ... }).await;

    // Verify distinct tracking
    assert_eq!(registry.device_count().await, 2);

    // Assign different profiles
    registry.assign_profile(&device1, "profile-work".to_string()).await.unwrap();
    registry.assign_profile(&device2, "profile-stream".to_string()).await.unwrap();

    // Verify isolation
    let state1 = registry.get_device_state(&device1).await.unwrap();
    let state2 = registry.get_device_state(&device2).await.unwrap();

    assert_eq!(state1.active_profile_id, Some("profile-work".to_string()));
    assert_eq!(state2.active_profile_id, Some("profile-stream".to_string()));
}
```

### 6.2. Profile-Device Decoupling Tests

```rust
#[tokio::test]
async fn test_profile_portability() {
    let profile_registry = ProfileRegistry::new();
    let device_registry = DeviceRegistry::new();

    // Create a profile
    let profile = Profile {
        id: "gaming-fps".to_string(),
        name: "Gaming FPS Profile".to_string(),
        layout_type: LayoutType::Standard(StandardLayout::ANSI),
        mappings: HashMap::from([
            (PhysicalPosition { row: 2, col: 1 }, KeyAction::Key(KeyCode::E)), // WASD setup
        ]),
        // ...
    };

    profile_registry.save_profile(profile.clone()).await.unwrap();

    // Assign to first device
    let device1 = DeviceIdentity { /* ... */ };
    device_registry.assign_profile(&device1, "gaming-fps".to_string()).await.unwrap();

    // Disconnect device1, connect device2 (different hardware, same layout)
    device_registry.unregister_device(&device1).await;

    let device2 = DeviceIdentity { /* different serial */ };
    device_registry.register_device(device2.clone(), /* ... */).await;

    // Assign same profile to device2
    device_registry.assign_profile(&device2, "gaming-fps".to_string()).await.unwrap();

    // Verify profile works on both devices
    let profile_loaded = profile_registry.get_profile("gaming-fps").await.unwrap();
    assert_eq!(profile_loaded.name, "Gaming FPS Profile");
}
```

### 6.3. Latency Benchmarks

**Criterion Benchmarks** (`core/benches/mapping_latency.rs`)
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_full_pipeline(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("device_resolution", |b| {
        b.iter(|| {
            runtime.block_on(async {
                // Measure time from raw event to DeviceState lookup
            });
        });
    });

    c.bench_function("profile_lookup", |b| {
        b.iter(|| {
            runtime.block_on(async {
                // Measure profile load + coordinate translation
            });
        });
    });

    c.bench_function("action_execution", |b| {
        b.iter(|| {
            // Measure action resolution + output generation
        });
    });
}

criterion_group!(benches, bench_full_pipeline);
criterion_main!(benches);
```

**Performance SLO:**
- Device resolution: <50μs (p99)
- Profile lookup: <100μs (p99)
- Coordinate translation: <20μs (p99)
- Action execution: <100μs (p99)
- **Total pipeline: <1ms (p99)**

---

## 7. Migration Path from Current Architecture

### 7.1. Backward Compatibility Strategy

**Goal:** Migrate existing users without losing configurations.

**Migration Steps:**

1. **Profile Migration:**
   - Read existing `{vid}_{pid}.json` files
   - Convert to new Profile format
   - Assign UUID as profile_id
   - Set layout_type based on `rows` and `cols_per_row`

2. **Device Registry Initialization:**
   - On first run with new system, detect all connected devices
   - For each device, attempt serial extraction
   - If serial unavailable, generate synthetic ID
   - Create DeviceState with is_remapping_enabled = false (safe default)

3. **Profile Assignment:**
   - Match old `{vid}_{pid}.json` to migrated profile
   - Auto-assign profile to detected device with matching VID:PID
   - Prompt user to enable remapping (don't auto-enable)

**Migration Script** (`core/src/migration/v1_to_v2.rs`)
```rust
pub struct MigrationV1ToV2 {
    old_config_dir: PathBuf,
    profile_registry: Arc<RwLock<ProfileRegistry>>,
}

impl MigrationV1ToV2 {
    pub async fn migrate(&self) -> Result<MigrationReport> {
        let old_profiles = self.scan_old_profiles()?;

        let mut report = MigrationReport::default();

        for old_profile in old_profiles {
            let new_profile = self.convert_profile(old_profile)?;
            self.profile_registry.write().await.save_profile(new_profile).await?;
            report.profiles_migrated += 1;
        }

        Ok(report)
    }

    fn convert_profile(&self, old: OldDeviceProfile) -> Result<Profile> {
        Ok(Profile {
            id: Uuid::new_v4().to_string(),
            name: old.name.unwrap_or_else(||
                format!("{:04X}:{:04X}", old.vendor_id, old.product_id)),
            layout_type: LayoutType::Matrix {
                rows: old.rows,
                cols: old.cols_per_row.iter().max().copied().unwrap_or(0),
            },
            mappings: old.keymap.into_iter()
                .filter_map(|(sc, pk)| {
                    Some((PhysicalPosition { row: pk.row, col: pk.col },
                         KeyAction::Key(pk.keycode?)))
                })
                .collect(),
            created_at: old.discovered_at,
            modified_at: Utc::now(),
            description: None,
            tags: vec![],
            script_source: None,
        })
    }
}
```

### 7.2. Feature Flags for Gradual Rollout

```toml
# Cargo.toml
[features]
default = ["windows-driver", "linux-driver"]
revolutionary-mapping = ["serial-tracking", "device-registry", "profile-registry"]
serial-tracking = []
device-registry = []
profile-registry = []
```

**Conditional Compilation:**
```rust
#[cfg(feature = "revolutionary-mapping")]
pub mod registry;

#[cfg(not(feature = "revolutionary-mapping"))]
pub mod legacy_discovery;
```

---

## 8. Data Model Summary

| Component | Storage | Format | Lifetime |
|-----------|---------|--------|----------|
| **DeviceRegistry** | In-memory | HashMap | Per-session |
| **ProfileRegistry** | Filesystem | JSON (one file per profile) | Persistent |
| **DeviceDefinitions** | Filesystem | TOML (read-only) | Immutable |
| **Device-Profile Bindings** | Persisted preferences | JSON (device_bindings.json) | Persistent |
| **User Labels** | Device bindings file | JSON | Persistent |

**File Structure:**
```
$XDG_CONFIG_HOME/keyrx/
├── profiles/
│   ├── gaming-fps-profile.json
│   ├── work-vim-profile.json
│   └── stream-deck-obs.json
├── device_bindings.json  # Maps DeviceIdentity -> active_profile_id + user_label
└── device_definitions/   # Shipped with application
    ├── standard/
    │   ├── ansi-104.toml
    │   └── iso-105.toml
    └── elgato/
        ├── stream-deck-mk2.toml
        └── stream-deck-xl.toml
```

**device_bindings.json Example:**
```json
{
  "bindings": [
    {
      "device": {
        "vendor_id": 4057,
        "product_id": 128,
        "serial_number": "ABC123456",
        "user_label": "Work Stream Deck"
      },
      "active_profile_id": "stream-deck-obs",
      "is_remapping_enabled": true,
      "last_connected": "2025-12-06T10:30:00Z"
    },
    {
      "device": {
        "vendor_id": 4057,
        "product_id": 128,
        "serial_number": "XYZ789012",
        "user_label": "Streaming Deck"
      },
      "active_profile_id": "twitch-bot-controls",
      "is_remapping_enabled": false,
      "last_connected": "2025-12-05T22:15:00Z"
    }
  ]
}
```

---

## 9. Security Considerations

### 9.1. Rhai Sandbox Enforcement

All Rhai scripts execute in sandboxed environment:
- No filesystem access
- No network access
- No process spawning
- Max execution time: 100ms per key event
- Max recursion depth: 64

### 9.2. Profile Validation

Before loading profiles:
- JSON schema validation
- Layout compatibility check
- Scancode range validation
- Cycle detection in mappings

### 9.3. FFI Safety

All FFI functions:
- Validate pointer arguments (null checks)
- Use panic guards (`std::panic::catch_unwind`)
- Return errors as strings (no panics across FFI boundary)
- Free allocated memory correctly

---

## 10. Observability & Debugging

### 10.1. Structured Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(registry))]
pub async fn assign_profile(
    registry: &DeviceRegistry,
    device: &DeviceIdentity,
    profile_id: ProfileId,
) -> Result<()> {
    info!(
        vendor_id = device.vendor_id,
        product_id = device.product_id,
        serial = %device.serial_number,
        profile_id = %profile_id,
        "Assigning profile to device"
    );

    // ... implementation

    Ok(())
}
```

### 10.2. Metrics

Prometheus-compatible metrics:
- `keyrx_devices_connected{vendor_id, product_id}` - Gauge
- `keyrx_profile_swaps_total` - Counter
- `keyrx_input_latency_seconds{stage}` - Histogram
- `keyrx_remap_enabled_devices` - Gauge

### 10.3. Tracing (OpenTelemetry)

Distributed tracing for input pipeline:
- Span per input event
- Stage annotations (resolution, translation, execution)
- Latency breakdown per stage

---

## 11. Error Handling

### 11.1. Error Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Device not found: {0:?}")]
    DeviceNotFound(DeviceIdentity),

    #[error("Profile not found: {0}")]
    ProfileNotFound(ProfileId),

    #[error("No device definition for VID:{0:04X} PID:{1:04X}")]
    NoDeviceDefinition(u16, u16),

    #[error("Layout incompatible: device requires {required:?}, profile has {actual:?}")]
    LayoutIncompatible {
        required: LayoutType,
        actual: LayoutType,
    },

    #[error("Serial extraction failed: {0}")]
    SerialExtractionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 11.2. Fail-Safe Behaviors

- Profile load failure → Device enters Passthrough mode
- Device definition missing → Use generic ANSI layout
- Serial extraction failure → Generate synthetic ID + warn user
- FFI panic → Catch, log, return error string

---

## 12. Performance Optimizations

### 12.1. Caching Strategy

- **Device Definitions:** Loaded once on startup, stored in Arc for shared read access
- **Active Profiles:** Cached in Arc<Profile> per device (avoid JSON parsing per event)
- **Coordinate Translation:** Build HashMap<Scancode, (Row, Col)> on profile load

### 12.2. Lock Contention Reduction

- Use `RwLock` for registries (many readers, few writers)
- Per-device locks instead of global locks where possible
- Lock-free data structures for hot paths (consider `dashmap` for registries)

### 12.3. Async I/O

- Profile saves use `tokio::fs::write` (non-blocking)
- Device detection uses async drivers (tokio-udev for Linux)

---

## 13. Implementation Checklist

**Phase 1: Data Structures**
- [ ] Define `DeviceIdentity` struct
- [ ] Define `DeviceState` struct
- [ ] Define `Profile` struct
- [ ] Define `LayoutType` enum
- [ ] Define `PhysicalPosition` struct
- [ ] Define `KeyAction` enum

**Phase 2: Serial Extraction**
- [ ] Implement Windows serial extraction
- [ ] Implement Linux serial extraction
- [ ] Add synthetic ID fallback
- [ ] Unit tests for parsing logic
- [ ] Integration tests with mock devices

**Phase 3: Registries**
- [ ] Implement `DeviceRegistry`
- [ ] Implement `ProfileRegistry`
- [ ] Implement `DeviceDefinitionLibrary`
- [ ] Add persistence for device bindings
- [ ] Add persistence for profiles

**Phase 4: Pipeline Integration**
- [ ] Implement `DeviceResolver`
- [ ] Implement `ProfileResolver`
- [ ] Implement `CoordinateTranslator`
- [ ] Implement `ActionResolver`
- [ ] Integrate into existing engine

**Phase 5: FFI Layer**
- [ ] Add device registry FFI functions
- [ ] Add profile registry FFI functions
- [ ] Add device definitions FFI functions
- [ ] Dart bindings for new FFI

**Phase 6: Migration**
- [ ] Write migration script
- [ ] Test migration with sample data
- [ ] Add migration UI prompts

**Phase 7: Testing**
- [ ] Unit tests for all modules
- [ ] Integration tests for multi-device
- [ ] Latency benchmarks
- [ ] End-to-end tests

---

This technical architecture provides the foundation for implementing the revolutionary mapping vision while maintaining performance, safety, and user experience standards.
