# Design Document: CLI-First Configuration Management

## Overview

This design implements profile management, device naming, keyboard layout presets, and configuration tooling for KeyRx2 using a **pure CLI architecture**. All functionality is accessible via CLI commands with JSON output for automated testing.

**Key Architectural Principle**: Every feature must work autonomously via CLI. A separate spec (`web-ui-configuration-editor`) will add optional web UI after CLI v1.0.

## Steering Document Alignment

### Technical Standards (tech.md)
- **CLI First**: "Prioritize CLI interfaces, add GUI later" - explicit design guideline
- **JSON Structured Logging**: All outputs machine-parseable for AI agents
- **Zero Manual Testing**: CLI automation enables deterministic verification
- **SSOT**: `.krx` binary files remain single source of runtime truth

### Project Structure (structure.md)
- CLI commands in `keyrx_daemon/src/cli/` (new module)
- Shared business logic in `keyrx_daemon/src/config/` (profile manager, device registry)
- Web API in `keyrx_daemon/src/web/` (thin wrappers around business logic)
- React UI in `keyrx_ui/` (Phase 6 only, optional)

## Code Reuse Analysis

### Existing Components to Leverage
- **`keyrx_daemon/src/main.rs`**: CLI argument parsing with clap (add new subcommands)
- **`keyrx_daemon/src/config_loader.rs`**: .krx loading (extend for hot-reload)
- **`keyrx_daemon/src/device_manager/mod.rs`**: Device enumeration (add naming/registry)
- **`keyrx_daemon/src/web/mod.rs`**: Axum server (add REST endpoints)
- **`keyrx_compiler/src/main.rs`**: Rhai compilation (expose as library API)

### New Components Required
- `DeviceRegistry`: Persistent device name/scope storage (`devices.json`)
- `ProfileManager`: Profile CRUD and hot-reload logic
- `LayoutManager`: KLE JSON layout storage and retrieval
- `RhaiGenerator`: Convert CLI commands to Rhai code modifications
- `SimulationEngine`: WASM-based deterministic event replay

### Integration Points
- Extend `main.rs` with new subcommands (devices, profiles, config, layers, simulate)
- Add business logic modules (zero web dependency)
- Web API endpoints call business logic (no duplication)

---

## Dependencies

### Required Crates (Cargo.toml)

**Core Dependencies**:
```toml
[dependencies]
# Existing dependencies
keyrx_core = { path = "../keyrx_core" }
keyrx_compiler = { path = "../keyrx_compiler" }

# New dependencies for this feature
clap = { version = "4.5", features = ["derive", "cargo"] }  # CLI parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
interprocess = { version = "1.2", features = ["tokio_support"] }  # IPC
tempfile = "3.8"  # Test isolation
hdrhistogram = "7.5"  # Latency metrics
```

**Optional Dependencies** (web UI):
```toml
[dependencies]
axum = { version = "0.7", optional = true }
tokio = { version = "1.35", features = ["full"], optional = true }
tower = { version = "0.4", optional = true }
tower-http = { version = "0.5", features = ["fs", "cors"], optional = true }

[features]
default = []
web = ["axum", "tokio", "tower", "tower-http"]
```

**Dev Dependencies**:
```toml
[dev-dependencies]
proptest = "1.4"  # Property-based testing
assert_cmd = "2.0"  # CLI testing
predicates = "3.0"  # Assertion helpers
```

### Version Constraints Rationale

- **clap 4.5**: Latest stable, derive macros for ergonomic CLI
- **serde 1.0**: Industry standard, stable API
- **interprocess 1.2**: Cross-platform IPC with tokio support
- **tempfile 3.8**: Automatic cleanup, battle-tested
- **hdrhistogram 7.5**: HdrHistogram for accurate latency percentiles
- **axum 0.7**: Modern async web framework (optional)
- **proptest 1.4**: Property-based testing for simulation determinism

### System Dependencies

**Linux**:
- No additional system dependencies (uses Unix sockets)

**Windows**:
- Named pipes API (via `windows-sys` crate, already present)

**Build Tools**:
- Rust 1.70+ (existing requirement)
- No C/C++ compilers required (pure Rust)

---

## Architecture

### CLI-First Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ User Interface Layer (both coexist independently)           │
│  ┌──────────────────────┐      ┌────────────────────────┐  │
│  │  CLI Commands        │      │  Web UI (Optional)     │  │
│  │  keyrx devices list  │      │  React Components      │  │
│  │  keyrx profiles *    │      │  → /api/* endpoints    │  │
│  └────────┬─────────────┘      └───────────┬────────────┘  │
│           │                                 │               │
│           │  Both call same business logic │               │
│           └─────────┬───────────────────────┘               │
└─────────────────────┼─────────────────────────────────────┘
                      │
┌─────────────────────▼─────────────────────────────────────┐
│ Business Logic Layer (CLI-agnostic)                       │
│  ┌──────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ ProfileMgr   │  │ DeviceReg   │  │ LayoutMgr       │  │
│  │ - create()   │  │ - rename()  │  │ - import_kle()  │  │
│  │ - activate() │  │ - set_scope │  │ - get_layout()  │  │
│  │ - delete()   │  │ - persist() │  │ - validate()    │  │
│  └──────────────┘  └─────────────┘  └─────────────────┘  │
│  ┌──────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ RhaiGen      │  │ ConfigMgr   │  │ Simulator       │  │
│  │ - set_key()  │  │ - compile() │  │ - replay()      │  │
│  │ - add_layer()│  │ - validate()│  │ - test_scenario │  │
│  └──────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────┬─────────────────────────────────────┘
                      │
┌─────────────────────▼─────────────────────────────────────┐
│ Persistence Layer                                          │
│  ~/.config/keyrx/                                         │
│  ├── devices.json        (DeviceRegistry)                 │
│  ├── profiles/*.rhai     (Rhai source)                    │
│  ├── profiles/*.krx      (Compiled binaries)              │
│  └── layouts/*.json      (KLE keyboard layouts)           │
└────────────────────────────────────────────────────────────┘
```

**Key Design Points**:
1. **Business logic has ZERO dependency on CLI or web frameworks**
2. **CLI commands call business logic → return JSON**
3. **Web endpoints call same business logic → return JSON**
4. **No code duplication between CLI and web**

## Components and Interfaces

### Component 1: DeviceRegistry
- **Purpose**: Persist device names, scopes, layout assignments
- **Interfaces**:
  ```rust
  pub struct DeviceRegistry {
      devices: HashMap<String, DeviceEntry>,
      path: PathBuf,
  }

  pub struct DeviceEntry {
      pub id: String,
      pub name: String,
      pub serial: Option<String>,
      pub scope: DeviceScope,  // DeviceSpecific | Global
      pub layout: Option<String>,
      pub last_seen: u64,
  }

  impl DeviceRegistry {
      pub fn load(path: &Path) -> Result<Self>;
      pub fn save(&self) -> Result<()>;
      pub fn rename(&mut self, id: &str, name: &str) -> Result<()>;
      pub fn set_scope(&mut self, id: &str, scope: DeviceScope) -> Result<()>;
      pub fn set_layout(&mut self, id: &str, layout: &str) -> Result<()>;
      pub fn forget(&mut self, id: &str) -> Result<DeviceEntry>;
      pub fn list(&self) -> Vec<&DeviceEntry>;
  }
  ```
- **Dependencies**: serde_json for persistence
- **Reuses**: None (new component)

---

### Component 2: ProfileManager
- **Purpose**: Create, activate, delete profiles with hot-reload and concurrency safety
- **Interfaces**:
  ```rust
  use std::sync::{Arc, RwLock};

  pub struct ProfileManager {
      config_dir: PathBuf,
      active_profile: Arc<RwLock<Option<String>>>,  // Concurrent access safe
      profiles: HashMap<String, ProfileMetadata>,
      activation_lock: Arc<Mutex<()>>,  // Serialize activations
  }

  pub struct ProfileMetadata {
      pub name: String,
      pub rhai_path: PathBuf,
      pub krx_path: PathBuf,
      pub modified_at: SystemTime,
      pub layer_count: usize,
  }

  impl ProfileManager {
      pub fn new(config_dir: PathBuf) -> Result<Self>;
      pub fn scan_profiles(&mut self) -> Result<()>;
      pub fn create(&mut self, name: &str, template: ProfileTemplate) -> Result<ProfileMetadata>;
      pub fn activate(&mut self, name: &str) -> Result<ActivationResult>;  // Acquires activation_lock
      pub fn delete(&mut self, name: &str) -> Result<()>;
      pub fn duplicate(&mut self, src: &str, dest: &str) -> Result<ProfileMetadata>;
      pub fn export(&self, name: &str, dest: &Path) -> Result<()>;
      pub fn import(&mut self, src: &Path, name: &str) -> Result<ProfileMetadata>;
      pub fn list(&self) -> Vec<&ProfileMetadata>;
  }

  pub struct ActivationResult {
      pub compile_time_ms: u64,
      pub reload_time_ms: u64,
      pub success: bool,
      pub error: Option<String>,
  }
  ```
- **Concurrency Strategy**:
  - `active_profile` uses `Arc<RwLock<>>` for concurrent reads, exclusive writes
  - `activation_lock` serializes concurrent `activate()` calls (second waits for first)
  - Config swap uses `Arc::swap` for atomic updates (no race conditions)
- **Dependencies**: keyrx_compiler (for Rhai compilation), keyrx_core (for .krx loading)
- **Reuses**: Existing compiler and config loader

---

### Component 3: RhaiGenerator
- **Purpose**: Programmatically modify Rhai source (CLI → code generation)
- **Interfaces**:
  ```rust
  pub struct RhaiGenerator {
      source: String,
      ast: RhaiAst,  // Parsed representation
  }

  impl RhaiGenerator {
      pub fn load(path: &Path) -> Result<Self>;
      pub fn set_key_mapping(&mut self, layer: &str, key: &str, action: KeyAction) -> Result<()>;
      pub fn delete_key_mapping(&mut self, layer: &str, key: &str) -> Result<()>;
      pub fn add_layer(&mut self, layer_id: &str, name: &str, mode: LayerMode) -> Result<()>;
      pub fn rename_layer(&mut self, layer_id: &str, new_name: &str) -> Result<()>;
      pub fn delete_layer(&mut self, layer_id: &str) -> Result<()>;
      pub fn save(&self, path: &Path) -> Result<()>;
      pub fn to_string(&self) -> String;
  }

  pub enum KeyAction {
      SimpleRemap { output: String },
      TapHold { tap: String, hold: String, threshold_ms: u16 },
      Macro { sequence: Vec<MacroStep> },
      Conditional { condition: Condition, then_action: Box<KeyAction>, else_action: Box<KeyAction> },
  }
  ```
- **Dependencies**: Rhai parser for AST manipulation
- **Reuses**: None (new component)

**Design Decision**: Instead of string concatenation, parse Rhai into AST, modify structurally, regenerate code. This prevents syntax errors.

---

### Component 4: LayoutManager
- **Purpose**: Manage keyboard layout presets (KLE JSON format)
- **Interfaces**:
  ```rust
  pub struct LayoutManager {
      layouts_dir: PathBuf,
      layouts: HashMap<String, KeyboardLayout>,
  }

  pub struct KeyboardLayout {
      pub name: String,
      pub kle_json: serde_json::Value,
      pub source: LayoutSource,  // Builtin | Custom
  }

  impl LayoutManager {
      pub fn new(layouts_dir: PathBuf) -> Result<Self>;
      pub fn list(&self) -> Vec<&str>;
      pub fn get(&self, name: &str) -> Option<&KeyboardLayout>;
      pub fn import(&mut self, path: &Path, name: &str) -> Result<KeyboardLayout>;
      pub fn delete(&mut self, name: &str) -> Result<()>;
      pub fn validate_kle(json: &serde_json::Value) -> Result<()>;
  }
  ```
- **Dependencies**: serde_json for KLE parsing
- **Reuses**: None (new component)

**Builtin Layouts**: Embedded in binary (include_str!) for ANSI 104, ISO 105, JIS 109, HHKB, numpad

---

### Component 5: SimulationEngine
- **Purpose**: Deterministic event replay for testing
- **Interfaces**:
  ```rust
  pub struct SimulationEngine {
      krx_data: Vec<u8>,
      core: keyrx_core::EventProcessor,
      clock: VirtualClock,
  }

  pub struct EventSequence {
      pub events: Vec<SimulatedEvent>,
      pub seed: u64,
  }

  pub struct SimulatedEvent {
      pub device_id: Option<String>,
      pub timestamp_us: u64,
      pub key: String,
      pub event_type: EventType,  // Press, Release
  }

  impl SimulationEngine {
      pub fn new(krx_path: &Path) -> Result<Self>;
      pub fn replay(&mut self, sequence: &EventSequence) -> Result<Vec<OutputEvent>>;
      pub fn run_scenario(&mut self, scenario: BuiltinScenario) -> Result<ScenarioResult>;
  }

  pub enum BuiltinScenario {
      TapHoldUnderThreshold,
      TapHoldOverThreshold,
      PermissiveHold,
      CrossDeviceModifiers,
      // ... more scenarios
  }
  ```
- **Dependencies**: keyrx_core (simulation), serde for event file parsing
- **Reuses**: Existing keyrx_core EventProcessor

**Testing Strategy**: All scenarios stored as JSON files in `keyrx_daemon/tests/scenarios/`. CLI can replay them deterministically.

---

### Component 6: DaemonIPC
- **Purpose**: Inter-process communication for CLI to query running daemon
- **Interfaces**:
  ```rust
  pub trait DaemonIpc {
      fn send_request(&mut self, req: IpcRequest) -> Result<IpcResponse>;
      fn receive_response(&mut self, timeout_ms: u64) -> Result<IpcResponse>;
  }

  pub struct UnixSocketIpc {
      socket_path: PathBuf,  // /tmp/keyrx-daemon.sock
      stream: Option<UnixStream>,
  }

  impl DaemonIpc for UnixSocketIpc {
      fn send_request(&mut self, req: IpcRequest) -> Result<IpcResponse>;
      fn receive_response(&mut self, timeout_ms: u64) -> Result<IpcResponse>;
  }

  #[derive(Serialize, Deserialize)]
  pub enum IpcRequest {
      GetStatus,
      GetState,  // Modifier/lock state
      GetLatencyMetrics,
      GetEventsTail { count: usize },
  }

  #[derive(Serialize, Deserialize)]
  pub enum IpcResponse {
      Status { running: bool, uptime_seconds: u64, active_profile: Option<String>, device_count: usize },
      State { modifiers: Vec<u8>, locks: Vec<u8>, active_layers: Vec<String> },
      Latency { min_us: u64, avg_us: u64, max_us: u64, p95_us: u64, p99_us: u64 },
      Events { events: Vec<EventRecord> },
      Error { code: u32, message: String },
  }
  ```
- **Socket Location**: `/tmp/keyrx-daemon.sock` (Linux), `\\.\pipe\keyrx-daemon` (Windows)
- **Protocol**: JSON-based request/response over Unix socket
- **Timeout**: 5 seconds default, configurable
- **Error Handling**:
  - Socket not found → "Daemon not running"
  - Connection refused → "Daemon not accepting connections"
  - Timeout → "Daemon not responding"
- **Dependencies**: `interprocess` crate (version 1.2+)
- **Reuses**: None (new component)

---

## CLI Command Structure

All CLI commands follow this pattern:
```
keyrx <noun> <verb> [args...] [--json] [--quiet]
```

### Subcommands Organization

```
keyrx_daemon/src/cli/
├── mod.rs              # Command routing, shared --json flag
├── devices.rs          # keyrx devices {list|rename|set-scope|forget|set-layout}
├── profiles.rs         # keyrx profiles {list|create|activate|delete|duplicate|export|import}
├── config.rs           # keyrx config {set-key|get-key|delete-key|validate|show|diff}
├── layers.rs           # keyrx layers {list|create|rename|delete|show}
├── layouts.rs          # keyrx layouts {list|show|import|delete}
├── simulate.rs         # keyrx simulate <profile> --events <...>
├── test.rs             # keyrx test <profile> --scenario <name>
├── status.rs           # keyrx status
├── state.rs            # keyrx state inspect
├── metrics.rs          # keyrx metrics {latency|events}
└── web.rs              # keyrx web {start|stop}
```

**Each module exports**:
```rust
pub fn execute(args: &Args) -> Result<Output>;

pub struct Output {
    pub json: serde_json::Value,
    pub text: Option<String>,  // Human-readable summary
    pub exit_code: i32,
}
```

**main.rs orchestrates**:
```rust
let output = match args.command {
    Command::Devices(d) => cli::devices::execute(d)?,
    Command::Profiles(p) => cli::profiles::execute(p)?,
    // ...
};

if args.json {
    println!("{}", serde_json::to_string_pretty(&output.json)?);
} else if let Some(text) = output.text {
    println!("{}", text);
}

std::process::exit(output.exit_code);
```

---

## Data Models

### devices.json Format
```json
{
  "version": 1,
  "devices": {
    "USB\\VID_1234&PID_5678\\SERIAL_ABC": {
      "id": "USB\\VID_1234&PID_5678\\SERIAL_ABC",
      "name": "Left Numpad",
      "serial": "SERIAL_ABC",
      "scope": "device-specific",
      "layout": "numpad",
      "last_seen": 1735459200
    }
  }
}
```

### Profile Metadata (in-memory only, scanned from filesystem)
```json
{
  "name": "gaming",
  "rhai_path": "/home/user/.config/keyrx/profiles/gaming.rhai",
  "krx_path": "/home/user/.config/keyrx/profiles/gaming.krx",
  "modified_at": 1735459200,
  "layer_count": 5,
  "is_active": false
}
```

### Event Sequence File (for simulation)
```json
{
  "seed": 42,
  "events": [
    {"timestamp_us": 0, "device_id": "USB123", "key": "CapsLock", "type": "press"},
    {"timestamp_us": 50000, "device_id": "USB123", "key": "CapsLock", "type": "release"},
    {"timestamp_us": 100000, "device_id": "USB123", "key": "A", "type": "press"}
  ]
}
```

---

## Testing Strategy

### Unit Testing
- **DeviceRegistry**: Load/save, rename, forget, corrupted JSON handling
- **ProfileManager**: Create, activate, hot-reload, compilation failure rollback
- **RhaiGenerator**: AST modification, code generation correctness
- **SimulationEngine**: Event replay determinism, built-in scenarios

```rust
#[test]
fn test_device_registry_persist() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("devices.json");

    let mut registry = DeviceRegistry::new(&path).unwrap();
    registry.rename("USB123", "My Keyboard").unwrap();
    registry.save().unwrap();

    let loaded = DeviceRegistry::load(&path).unwrap();
    assert_eq!(loaded.devices.get("USB123").unwrap().name, "My Keyboard");
}
```

### Integration Testing (CLI)
- **End-to-End via CLI**: All integration tests run CLI commands and parse JSON output

```rust
#[test]
fn test_profile_create_and_activate() {
    // Run CLI: keyrx profiles create test-profile
    let output = run_cli(&["profiles", "create", "test-profile", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], true);

    // Run CLI: keyrx profiles activate test-profile
    let output = run_cli(&["profiles", "activate", "test-profile", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], true);
    assert!(json["compile_time_ms"].as_u64().unwrap() < 1000);
}

fn run_cli(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_keyrx"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}
```

### Property-Based Testing
- **Simulation determinism**: Same event sequence + seed = identical output

```rust
proptest! {
    #[test]
    fn simulation_is_deterministic(
        events in prop::collection::vec(event_strategy(), 0..100),
        seed in any::<u64>(),
    ) {
        let engine1 = SimulationEngine::new(TEST_KRX_PATH).unwrap();
        let engine2 = SimulationEngine::new(TEST_KRX_PATH).unwrap();

        let seq = EventSequence { events: events.clone(), seed };
        let result1 = engine1.replay(&seq).unwrap();
        let result2 = engine2.replay(&seq).unwrap();

        prop_assert_eq!(result1, result2);
    }
}
```

### Test Isolation Strategy

**Problem**: Tests that modify shared state (devices.json, profiles/, daemon state) cannot run in parallel.

**Solution**: Multi-level isolation

```rust
// 1. Unit tests: Pure business logic, no filesystem
#[test]
fn test_profile_metadata_parsing() {
    let meta = ProfileMetadata::new("test", "/tmp/test.rhai");
    assert_eq!(meta.name, "test");
}

// 2. Integration tests: Isolated temp directories
#[test]
fn test_device_registry_crud() {
    let temp = tempdir().unwrap();  // Each test gets unique tmpdir
    let registry_path = temp.path().join("devices.json");

    let mut registry = DeviceRegistry::new(&registry_path).unwrap();
    registry.rename("USB123", "My Keyboard").unwrap();

    // No interference from other tests
}

// 3. Daemon state tests: Sequential execution
#[cfg(test)]
mod daemon_tests {
    // Tests that interact with running daemon must run sequentially
    // Run with: cargo test daemon_tests -- --test-threads=1

    #[test]
    fn test_activate_profile_hot_reload() {
        // Start test daemon
        // Activate profile
        // Verify state changed
    }
}
```

**Test Execution Strategy**:
```bash
# Fast parallel unit tests (no filesystem)
cargo test --lib

# Isolated integration tests (tempdir-based, parallel safe)
cargo test --test integration -- --test-threads=8

# Sequential daemon tests (shared daemon state)
cargo test --test daemon -- --test-threads=1
```

**Tempdir Usage Pattern**:
```rust
use tempfile::tempdir;

#[test]
fn test_profile_activation() {
    let temp = tempdir().unwrap();
    let config_dir = temp.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    let mut pm = ProfileManager::new(config_dir.clone()).unwrap();
    // All file I/O confined to temp directory
    // Automatic cleanup on test exit
}
```

**Mock Strategy for FFI**:
- No FFI in this feature (all pure Rust)
- If future features add FFI, use trait abstraction for mocking

---

### Autonomous E2E Testing
- **GitHub Actions**: Run CLI commands in CI, verify JSON output

```yaml
- name: Test profile management
  run: |
    keyrx profiles create test --json | jq -e '.success == true'
    keyrx profiles list --json | jq -e '.profiles | length > 0'
    keyrx profiles activate test --json | jq -e '.compile_time_ms < 1000'
```

---

## Error Handling

### Error Categories

1. **User Input Errors** (exit code 1)
   - Profile not found
   - Invalid key name
   - Invalid JSON syntax

2. **Compilation Errors** (exit code 1)
   - Rhai syntax errors
   - Undefined modifiers
   - Invalid key codes

3. **System Errors** (exit code 1)
   - File I/O errors
   - Permission denied
   - Daemon not running

4. **Warnings** (exit code 2)
   - Threshold > 1000ms
   - Deprecated syntax

---

### Error Code Enumeration

All errors include a numeric code for programmatic handling:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // User Input Errors (1000-1999)
    ProfileNotFound = 1001,
    DeviceNotFound = 1002,
    LayerNotFound = 1003,
    LayoutNotFound = 1004,
    InvalidKeyName = 1005,
    InvalidProfileName = 1006,
    InvalidDeviceName = 1007,
    InvalidLayerName = 1008,
    InvalidJsonSyntax = 1009,
    ProfileNameTooLong = 1010,
    DeviceNameTooLong = 1011,
    LayerNameTooLong = 1012,
    InvalidCharacters = 1013,
    ProfileLimitExceeded = 1014,
    LayerLimitExceeded = 1015,
    BaseLayerProtected = 1016,

    // Compilation Errors (2000-2999)
    RhaiSyntaxError = 2001,
    UndefinedModifier = 2002,
    InvalidKeyCode = 2003,
    CompilationTimeout = 2004,
    CircularDependency = 2005,
    DuplicateLayerId = 2006,
    InvalidThreshold = 2007,
    MacroTooLarge = 2008,

    // System Errors (3000-3999)
    FileIoError = 3001,
    PermissionDenied = 3002,
    DaemonNotRunning = 3003,
    DaemonNotResponding = 3004,
    IpcSocketNotFound = 3005,
    IpcConnectionRefused = 3006,
    IpcTimeout = 3007,
    ConfigDirectoryNotFound = 3008,
    DiskSpaceExhausted = 3009,
    RegistryCorrupted = 3010,
    ProfileCorrupted = 3011,
    HotReloadFailed = 3012,
    DaemonCrashed = 3013,

    // Warnings (4000-4999)
    ThresholdHigh = 4001,
    TooManyLayers = 4002,
    DeprecatedSyntax = 4003,
    LargeProfile = 4004,
    SlowCompilation = 4005,
    UnusedLayer = 4006,
    ConflictingMapping = 4007,
}

impl ErrorCode {
    pub fn exit_code(&self) -> i32 {
        match self {
            ErrorCode::ThresholdHigh
            | ErrorCode::TooManyLayers
            | ErrorCode::DeprecatedSyntax
            | ErrorCode::LargeProfile
            | ErrorCode::SlowCompilation
            | ErrorCode::UnusedLayer
            | ErrorCode::ConflictingMapping => 2, // Warnings
            _ => 1, // Errors
        }
    }

    pub fn category(&self) -> &'static str {
        let code = *self as u32;
        match code {
            1000..=1999 => "User Input Error",
            2000..=2999 => "Compilation Error",
            3000..=3999 => "System Error",
            4000..=4999 => "Warning",
            _ => "Unknown",
        }
    }
}
```

**Usage in Error Responses**:
```rust
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub hint: Option<String>,
    pub details: serde_json::Value,
}

impl AppError {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "success": false,
            "error": {
                "code": self.code as u32,
                "category": self.code.category(),
                "message": self.message,
                "hint": self.hint,
                "details": self.details,
            }
        })
    }
}
```

**Example Error JSON**:
```json
{
  "success": false,
  "error": {
    "code": 1001,
    "category": "User Input Error",
    "message": "Profile 'gaming' not found",
    "hint": "Run 'keyrx profiles list' to see available profiles",
    "details": {
      "profile_name": "gaming",
      "available_profiles": ["default", "work"]
    }
  }
}
```

---

### Error Output Format (JSON)

```json
{
  "success": false,
  "error": {
    "code": "PROFILE_NOT_FOUND",
    "message": "Profile 'gaming' not found.",
    "hint": "Run 'keyrx profiles list' to see available profiles.",
    "details": {
      "profile_name": "gaming",
      "available_profiles": ["default", "work"]
    }
  }
}
```

**Text Output** (when --json not used):
```
Error: Profile 'gaming' not found.

Hint: Run 'keyrx profiles list' to see available profiles.

Available profiles:
  - default
  - work
```

---

## Implementation Philosophy

**Pure CLI v1.0**: This spec delivers a complete, production-ready CLI tool with zero web dependencies.

**Next Steps**: After v1.0 CLI is shipped, the `web-ui-configuration-editor` spec will add optional web-based visual configuration as v1.1.
