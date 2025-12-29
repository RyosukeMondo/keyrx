# Tasks Document: Comprehensive Configuration Management & UI

## Phase 0: Environment Setup

- [x] 0. Verify dependencies and create IPC infrastructure
  - Files: `keyrx_daemon/Cargo.toml`, `keyrx_daemon/src/ipc/mod.rs` (new), `keyrx_daemon/src/ipc/unix_socket.rs` (new)
  - Add dependencies to Cargo.toml:
    - `interprocess = { version = "1.2", features = ["tokio_support"] }`
    - `tempfile = "3.8"`
    - `hdrhistogram = "7.5"`
    - `assert_cmd = "2.0"` (dev-dependency)
    - `predicates = "3.0"` (dev-dependency)
    - `proptest = "1.4"` (dev-dependency)
  - Create UnixSocketIpc implementation with send_request/receive_response methods
  - Define IpcRequest/IpcResponse enums matching design.md
  - Add unit tests: connect, send request, receive response, timeout handling
  - Purpose: Establish IPC mechanism for CLI-daemon communication
  - _Leverage: interprocess crate_
  - _Requirements: Design section "Component 6: DaemonIPC"_
  - _Prompt: Role: Rust systems developer with IPC expertise | Task: Set up project dependencies and create IPC infrastructure in keyrx_daemon/src/ipc/ with Unix socket implementation |

    **Dependencies to add**:
    - interprocess 1.2 (IPC)
    - tempfile 3.8 (test isolation)
    - hdrhistogram 7.5 (latency metrics)
    - assert_cmd 2.0, predicates 3.0, proptest 1.4 (testing)

    **IPC Implementation**:
    - Create UnixSocketIpc struct with socket_path field (/tmp/keyrx-daemon.sock)
    - Implement DaemonIpc trait with send_request and receive_response methods
    - Define IpcRequest enum: GetStatus, GetState, GetLatencyMetrics, GetEventsTail
    - Define IpcResponse enum: Status, State, Latency, Events, Error
    - JSON serialization/deserialization for messages
    - 5-second default timeout, configurable

    **Error Handling**:
    - Socket not found → error code 3005
    - Connection refused → error code 3006
    - Timeout → error code 3007
    - Deserialize errors → error code 1009

  | Restrictions: Use serde_json for message serialization, non-blocking reads with timeout, gracefully handle daemon offline, file ≤300 lines
  | Success: ✅ Dependencies added and compile, ✅ UnixSocketIpc connects to test socket, ✅ Request/response roundtrip works, ✅ Timeout handling verified, ✅ Unit tests pass
  | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (document IpcRequest/IpcResponse enum variants), then mark [x] when done

---

## Phase 1: Core Business Logic

- [x] 1. Create DeviceRegistry component
  - File: `keyrx_daemon/src/config/device_registry.rs` (new file)
  - Purpose: Persistent device metadata storage with atomic writes and input validation
  - _Leverage: None (new component)_
  - _Requirements: 1.1-1.7 (Device Detection and Naming)_
  - _Prompt: Role: Rust developer with file I/O and persistence expertise | Task: Create DeviceRegistry in keyrx_daemon/src/config/device_registry.rs with atomic JSON persistence and comprehensive validation |

    **Exact Struct Definitions**:
    ```rust
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DeviceEntry {
        pub id: String,           // Device ID (max 256 chars)
        pub name: String,         // User-friendly name (max 64 chars)
        pub serial: Option<String>, // Serial number if available
        pub scope: DeviceScope,
        pub layout: Option<String>, // Layout name (max 32 chars)
        pub last_seen: u64,       // Unix timestamp (seconds)
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum DeviceScope {
        DeviceSpecific,
        Global,
    }

    pub struct DeviceRegistry {
        devices: HashMap<String, DeviceEntry>,
        path: PathBuf,
    }
    ```

    **Methods to Implement**:
    ```rust
    impl DeviceRegistry {
        // Load from disk, returns error if file exists but is corrupted
        pub fn load(path: &Path) -> Result<Self, RegistryError>;

        // Create new empty registry
        pub fn new(path: PathBuf) -> Self;

        // Atomic save: write to .tmp, then rename
        pub fn save(&self) -> Result<(), RegistryError>;

        // Rename device (validates name length ≤64, valid chars)
        pub fn rename(&mut self, id: &str, name: &str) -> Result<(), RegistryError>;

        // Set scope (DeviceSpecific or Global)
        pub fn set_scope(&mut self, id: &str, scope: DeviceScope) -> Result<(), RegistryError>;

        // Assign layout (validates layout exists)
        pub fn set_layout(&mut self, id: &str, layout: &str) -> Result<(), RegistryError>;

        // Remove device from registry
        pub fn forget(&mut self, id: &str) -> Result<DeviceEntry, RegistryError>;

        // List all devices
        pub fn list(&self) -> Vec<&DeviceEntry>;

        // Get device by ID
        pub fn get(&self, id: &str) -> Option<&DeviceEntry>;

        // Update last_seen timestamp
        pub fn update_last_seen(&mut self, id: &str) -> Result<(), RegistryError>;
    }
    ```

    **Error Handling**:
    ```rust
    #[derive(Debug)]
    pub enum RegistryError {
        DeviceNotFound(String),
        InvalidName(String),       // Name too long or invalid chars
        InvalidDeviceId(String),   // ID too long (>256)
        IoError(std::io::Error),
        SerdeError(serde_json::Error),
        RegistryCorrupted(String), // JSON parse failed
    }
    ```

    **Input Validation**:
    - Device name: ≤64 chars, alphanumeric + space/dash/underscore only
    - Device ID: ≤256 chars
    - Layout name: ≤32 chars

    **Atomic Write Implementation**:
    ```rust
    pub fn save(&self) -> Result<(), RegistryError> {
        let tmp_path = self.path.with_extension("tmp");
        // 1. Write to .tmp file
        let json = serde_json::to_string_pretty(&self.devices)?;
        std::fs::write(&tmp_path, json)?;
        // 2. Atomic rename
        std::fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
    ```

  | Restrictions: File ≤500 lines, all methods ≤50 lines, use serde_json for serialization, atomic writes prevent corruption, validate all inputs before mutation, return Result types with descriptive errors
  | Success: ✅ Load/save roundtrip preserves data, ✅ Atomic writes verified (interruption test), ✅ Corrupted JSON returns RegistryCorrupted error with hint, ✅ Input validation rejects invalid names/IDs, ✅ Unit tests cover all methods and error paths, ✅ Code coverage ≥90%
  | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (document DeviceEntry struct, all methods, error enum variants), then mark [x] when done

- [x] 2. Create ProfileManager component
  - File: `keyrx_daemon/src/config/profile_manager.rs` (new file)
  - Purpose: Profile CRUD with hot-reload and concurrency-safe activation
  - _Leverage: keyrx_compiler (Rhai compilation), keyrx_core (config loading)_
  - _Requirements: 2.1-2.8 (Profile Management), Error scenarios 14-16_
  - _Prompt: Role: Rust developer with concurrency and hot-reload expertise | Task: Create ProfileManager in keyrx_daemon/src/config/profile_manager.rs with thread-safe hot-reload and atomic config swaps |

    **Exact Struct Definitions**:
    ```rust
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock, Mutex};
    use std::time::SystemTime;

    pub struct ProfileManager {
        config_dir: PathBuf,
        active_profile: Arc<RwLock<Option<String>>>,  // Concurrent reads
        profiles: HashMap<String, ProfileMetadata>,
        activation_lock: Arc<Mutex<()>>,  // Serialize activations
    }

    pub struct ProfileMetadata {
        pub name: String,           // Profile name (max 32 chars)
        pub rhai_path: PathBuf,     // Source .rhai file
        pub krx_path: PathBuf,      // Compiled .krx file
        pub modified_at: SystemTime,
        pub layer_count: usize,
    }

    pub enum ProfileTemplate {
        Blank,           // Empty config with just base layer
        QmkLayers,       // QMK-style layer system example
    }

    pub struct ActivationResult {
        pub compile_time_ms: u64,
        pub reload_time_ms: u64,
        pub success: bool,
        pub error: Option<String>,
    }
    ```

    **Methods to Implement**:
    ```rust
    impl ProfileManager {
        // Create with config directory
        pub fn new(config_dir: PathBuf) -> Result<Self, ProfileError>;

        // Scan profiles/ directory for .rhai files
        pub fn scan_profiles(&mut self) -> Result<(), ProfileError>;

        // Create new profile from template
        pub fn create(&mut self, name: &str, template: ProfileTemplate)
            -> Result<ProfileMetadata, ProfileError>;

        // Hot-reload profile (compile + atomic swap)
        // CRITICAL: Acquires activation_lock, serializes concurrent calls
        pub fn activate(&mut self, name: &str) -> Result<ActivationResult, ProfileError>;

        // Delete profile (both .rhai and .krx)
        pub fn delete(&mut self, name: &str) -> Result<(), ProfileError>;

        // Duplicate profile
        pub fn duplicate(&mut self, src: &str, dest: &str)
            -> Result<ProfileMetadata, ProfileError>;

        // Export profile to file
        pub fn export(&self, name: &str, dest: &Path) -> Result<(), ProfileError>;

        // Import profile from file
        pub fn import(&mut self, src: &Path, name: &str)
            -> Result<ProfileMetadata, ProfileError>;

        // List all profiles
        pub fn list(&self) -> Vec<&ProfileMetadata>;

        // Get currently active profile name
        pub fn get_active(&self) -> Option<String>;
    }
    ```

    **Hot-Reload Implementation**:
    ```rust
    pub fn activate(&mut self, name: &str) -> Result<ActivationResult, ProfileError> {
        // 1. Acquire activation lock (serialize concurrent activations)
        let _lock = self.activation_lock.lock().unwrap();

        let start = Instant::now();

        // 2. Compile .rhai → .krx
        let profile = self.profiles.get(name).ok_or(ProfileError::NotFound)?;
        let compile_start = Instant::now();

        // Timeout after 30 seconds (requirement error scenario 14)
        let result = timeout(Duration::from_secs(30), async {
            keyrx_compiler::compile(&profile.rhai_path, &profile.krx_path)
        }).await?;

        let compile_time = compile_start.elapsed();

        // 3. Load .krx into memory
        let krx_data = std::fs::read(&profile.krx_path)?;
        let new_config = Arc::new(krx_data);

        // 4. Atomic swap (no race conditions)
        let reload_start = Instant::now();
        // ... Arc::swap logic here ...
        let reload_time = reload_start.elapsed();

        // 5. Update active_profile
        *self.active_profile.write().unwrap() = Some(name.to_string());

        Ok(ActivationResult {
            compile_time_ms: compile_time.as_millis() as u64,
            reload_time_ms: reload_time.as_millis() as u64,
            success: true,
            error: None,
        })
    }
    ```

    **Error Handling**:
    ```rust
    pub enum ProfileError {
        NotFound(String),             // Error code 1001
        InvalidName(String),          // Error code 1006
        CompilationFailed(String),    // Error code 2001
        CompilationTimeout,           // Error code 2004
        IoError(std::io::Error),      // Error code 3001
        PermissionDenied,             // Error code 3002
        ProfileLimitExceeded,         // Error code 1014 (>100)
        DiskSpaceExhausted,           // Error code 3009
    }
    ```

    **Input Validation**:
    - Profile name: ≤32 chars, alphanumeric + dash/underscore only
    - Profile count limit: ≤100 profiles
    - Compilation timeout: 30 seconds

    **Concurrency Safety**:
    - activation_lock ensures second concurrent activate() waits
    - active_profile RwLock allows concurrent reads, exclusive writes
    - Arc::swap provides atomic config updates

  | Restrictions: File ≤500 lines, activate() ≤80 lines, use Arc<RwLock<>> for active_profile, Mutex for activation_lock, timeout compilation at 30s, rollback on failure (keep previous config), validate profile names, exit code 1 on error
  | Success: ✅ Profile activation <100ms (excluding compile time), ✅ Compilation timeout kills process and returns error, ✅ Concurrent activate() calls serialize (second waits), ✅ Rollback on compilation failure preserves previous profile, ✅ No daemon restart required, ✅ Unit tests cover all methods, ✅ Concurrency test verifies no race conditions, ✅ Coverage ≥90%
  | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (document ProfileManager struct, ProfileMetadata, all methods, concurrency strategy), then mark [x] when done

- [x] 3. Create RhaiGenerator for programmatic code modification
  - File: `keyrx_daemon/src/config/rhai_generator.rs` (new file)
  - Parse Rhai source into AST (use rhai::Engine)
  - Implement set_key_mapping (modify AST, regenerate code)
  - Implement add_layer, rename_layer, delete_layer
  - Add unit tests: generate tap-hold, macro, verify syntax correctness
  - Purpose: CLI → Rhai code generation
  - _Leverage: rhai crate (AST parsing)_
  - _Requirements: 3.1-3.7_
  - _Prompt: Role: Rust developer with compiler/AST expertise | Task: Create RhaiGenerator in keyrx_daemon/src/config/rhai_generator.rs using Rhai AST manipulation | Restrictions: Parse AST not string concat, regenerate syntactically valid code, preserve comments | Success: Generated Rhai compiles successfully, AST modifications preserve structure, tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 4. Create LayoutManager for KLE JSON handling
  - File: `keyrx_daemon/src/config/layout_manager.rs` (new file)
  - Embed builtin layouts (ansi_104, iso_105, jis_109, hhkb, numpad) using include_str!
  - Implement import with KLE JSON validation
  - Implement list, get, delete
  - Add unit tests: validate KLE format, import custom layout
  - Purpose: Keyboard layout preset management
  - _Leverage: serde_json for KLE parsing_
  - _Requirements: 6.1-6.5_
  - _Prompt: Role: Rust developer with embedded data expertise | Task: Create LayoutManager in keyrx_daemon/src/config/layout_manager.rs with builtin KLE layouts | Restrictions: Embed layouts in binary, validate KLE schema, reject invalid JSON | Success: All builtin layouts parse correctly, custom imports validated, tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 5. Create SimulationEngine for deterministic testing
  - File: `keyrx_daemon/src/config/simulation_engine.rs` (new file)
  - Load .krx config and initialize keyrx_core EventProcessor
  - Implement replay with VirtualClock (deterministic timing)
  - Implement built-in scenarios (tap-hold, permissive-hold, cross-device)
  - Add unit tests: same input+seed = identical output
  - Purpose: CLI-based autonomous testing
  - _Leverage: keyrx_core (EventProcessor, VirtualClock)_
  - _Requirements: 7.1-7.6_
  - _Prompt: Role: Rust developer with testing expertise | Task: Create SimulationEngine in keyrx_daemon/src/config/simulation_engine.rs for deterministic replay | Restrictions: Use VirtualClock for timing, seed-based determinism, support multi-device events | Success: 100% deterministic (proptest verified), scenarios JSON-serializable, tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

## Phase 2: CLI Commands (Devices & Profiles)

- [x] 6. Implement `keyrx devices` command
  - File: `keyrx_daemon/src/cli/devices.rs` (new file)
  - Implement subcommands: list, rename, set-scope, forget, set-layout
  - JSON output with `--json` flag
  - Integrate with DeviceRegistry
  - Add CLI integration tests (run command, parse JSON output)
  - Purpose: Device management via CLI
  - _Leverage: clap for arg parsing, DeviceRegistry_
  - _Requirements: 1.1-1.7_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx devices` command in keyrx_daemon/src/cli/devices.rs | Restrictions: Use clap, support --json flag, exit codes (0=success, 1=error), helpful error messages | Success: All subcommands work, JSON parseable, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 7. Implement `keyrx profiles` command
  - File: `keyrx_daemon/src/cli/profiles.rs` (new file)
  - Implement subcommands: list, create, activate, delete, duplicate, export, import
  - JSON output with compilation timing (compile_time_ms, reload_time_ms)
  - Integrate with ProfileManager
  - Add CLI integration tests
  - Purpose: Profile management via CLI
  - _Leverage: clap, ProfileManager_
  - _Requirements: 2.1-2.8_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx profiles` command in keyrx_daemon/src/cli/profiles.rs | Restrictions: Hot-reload on activate, JSON timing output, confirm flag for destructive ops | Success: Profile switching <100ms, clear error messages, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

## Phase 3: CLI Commands (Configuration & Layers)

- [x] 8. Implement `keyrx config` command
  - File: `keyrx_daemon/src/cli/config.rs` (new file)
  - Implement set-key (simple, tap-hold, macro), get-key, delete-key
  - Implement validate (dry-run compilation)
  - Implement show (KRX metadata), diff (compare profiles)
  - Integrate with RhaiGenerator
  - Add CLI integration tests
  - Purpose: Key mapping configuration via CLI
  - _Leverage: clap, RhaiGenerator, ProfileManager_
  - _Requirements: 3.1-3.7_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx config` command in keyrx_daemon/src/cli/config.rs | Restrictions: Auto-recompile on set-key, validate without applying, detailed compile errors | Success: Mappings applied correctly, errors show line numbers, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 9. Implement `keyrx layers` command
  - File: `keyrx_daemon/src/cli/layers.rs` (new file)
  - Implement list, create, rename, delete, show
  - Integrate with RhaiGenerator (AST modification for layers)
  - Add CLI integration tests
  - Purpose: Layer management via CLI
  - _Leverage: clap, RhaiGenerator_
  - _Requirements: 4.1-4.5_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx layers` command in keyrx_daemon/src/cli/layers.rs | Restrictions: Confirm flag for delete, list all keys in layer for show, integrate with Rhai AST | Success: Layer CRUD works, show displays mappings, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 10. Implement `keyrx layouts` command
  - File: `keyrx_daemon/src/cli/layouts.rs` (new file)
  - Implement list, show, import, delete
  - Output KLE JSON for show
  - Integrate with LayoutManager
  - Add CLI integration tests
  - Purpose: Layout management via CLI
  - _Leverage: clap, LayoutManager_
  - _Requirements: 6.1-6.5_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx layouts` command in keyrx_daemon/src/cli/layouts.rs | Restrictions: Show outputs valid KLE JSON, import validates schema, helpful validation errors | Success: Layouts import/export correctly, validation catches errors, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

## Phase 4: CLI Commands (Simulation & Monitoring)

- [x] 11. Implement `keyrx simulate` command
  - File: `keyrx_daemon/src/cli/simulate.rs` (new file)
  - Support inline events: `--events "press:A,wait:50,release:A"`
  - Support event files: `--events-file scenario.json`
  - Support seed for determinism: `--seed 42`
  - Integrate with SimulationEngine
  - Add CLI integration tests (determinism verification)
  - Purpose: Deterministic simulation via CLI
  - _Leverage: clap, SimulationEngine_
  - _Requirements: 7.1-7.6_
  - _Prompt: Role: Rust CLI developer with testing expertise | Task: Implement `keyrx simulate` command in keyrx_daemon/src/cli/simulate.rs | Restrictions: Parse event DSL, JSON output with input/output, seed-based determinism | Success: Same seed = identical output, event file replay works, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 12. Implement `keyrx test` command
  - File: `keyrx_daemon/src/cli/test.rs` (new file)
  - Implement built-in scenarios (tap-hold-under-threshold, etc.)
  - Support `--scenario all` to run all scenarios
  - JSON output with pass/fail counts
  - Integrate with SimulationEngine
  - Add CLI integration tests
  - Purpose: Autonomous testing via CLI
  - _Leverage: clap, SimulationEngine_
  - _Requirements: 7.6_
  - _Prompt: Role: Rust CLI developer with QA expertise | Task: Implement `keyrx test` command in keyrx_daemon/src/cli/test.rs | Restrictions: Built-in scenarios as JSON files, detailed failure output, exit code 1 if any fail | Success: All scenarios pass for valid configs, failures detailed, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 13. Implement `keyrx status` command
  - File: `keyrx_daemon/src/cli/status.rs` (new file)
  - Query daemon via IPC or shared memory
  - Output: running, uptime, active_profile, device_count
  - JSON output
  - Add CLI integration tests
  - Purpose: Daemon status inspection via CLI
  - _Leverage: clap, daemon IPC mechanism_
  - _Requirements: 5.1_
  - _Prompt: Role: Rust CLI developer with IPC expertise | Task: Implement `keyrx status` command in keyrx_daemon/src/cli/status.rs | Restrictions: Non-blocking query, handle daemon not running gracefully, JSON output | Success: Status accurate, fails gracefully when daemon offline, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [x] 14. Implement `keyrx state inspect` command
  - File: `keyrx_daemon/src/cli/state.rs` (new file)
  - Query current modifier/lock state from daemon
  - Output 255-bit state as JSON array
  - Add CLI integration tests
  - Purpose: Runtime state inspection via CLI
  - _Leverage: clap, daemon IPC_
  - _Requirements: 5.4_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx state inspect` command in keyrx_daemon/src/cli/state.rs | Restrictions: Query daemon IPC, JSON array output, handle daemon offline | Success: State accurate during active remapping, graceful offline handling, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [ ] 15. Implement `keyrx metrics` command
  - File: `keyrx_daemon/src/cli/metrics.rs` (new file)
  - Subcommand `latency`: Output min, avg, max, p95, p99
  - Subcommand `events`: Tail last N events
  - Integrate with daemon metrics collection (HdrHistogram)
  - Add CLI integration tests
  - Purpose: Performance monitoring via CLI
  - _Leverage: clap, daemon IPC_
  - _Requirements: 5.5, 5.6_
  - _Prompt: Role: Rust CLI developer | Task: Implement `keyrx metrics` command in keyrx_daemon/src/cli/metrics.rs | Restrictions: JSON output, events support --follow flag, latency uses HdrHistogram | Success: Latency stats accurate, events tail works, integration tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

## Phase 5: Integration & Testing

- [ ] 16. Integration test suite for CLI
  - File: `keyrx_daemon/tests/cli_integration.rs` (new file)
  - Test all commands with JSON parsing
  - Test error scenarios (profile not found, invalid JSON, etc.)
  - Test deterministic simulation (seed-based replay)
  - Property-based tests for simulation
  - Purpose: Verify CLI autonomous operation
  - _Leverage: All CLI modules_
  - _Requirements: All_
  - _Prompt: Role: QA engineer with Rust testing expertise | Task: Create comprehensive CLI integration tests in keyrx_daemon/tests/cli_integration.rs | Restrictions: Test via compiled binary not library calls, parse JSON output, verify exit codes | Success: 100+ test cases pass, all commands verified, edge cases covered | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [ ] 17. Add GitHub Actions CI for CLI tests
  - File: `.github/workflows/cli-tests.yml` (new file)
  - Run `keyrx profiles create test --json | jq -e '.success == true'`
  - Run `keyrx simulate default --events ...` and verify determinism
  - Run `keyrx test default --scenario all --json` and verify pass
  - Test on Ubuntu and Windows
  - Purpose: Autonomous CI verification
  - _Leverage: GitHub Actions, jq for JSON parsing_
  - _Requirements: All_
  - _Prompt: Role: DevOps engineer | Task: Create CLI test workflow in .github/workflows/cli-tests.yml | Restrictions: Test on Linux and Windows matrix, use jq for JSON assertions, fail on any command error | Success: CI passes for all CLI commands, determinism verified, runs on push/PR | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

- [ ] 18. Create test scenario library
  - File: `keyrx_daemon/tests/scenarios/*.json` (multiple files)
  - Scenario: tap-hold-under-threshold.json
  - Scenario: tap-hold-over-threshold.json
  - Scenario: permissive-hold.json
  - Scenario: cross-device-modifiers.json
  - Scenario: macro-sequence.json
  - Purpose: Reusable test cases for simulation
  - _Leverage: SimulationEngine event format_
  - _Requirements: 7.1-7.6_
  - _Prompt: Role: QA engineer | Task: Create test scenario JSON files in keyrx_daemon/tests/scenarios/ | Restrictions: Event format must match EventSequence, include seed for determinism, cover edge cases | Success: All scenarios replay successfully, cover major features, JSON validates | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts_

## Phase 6: Validation & Verification

- [ ] 19. Performance validation and benchmarking
  - Files: `keyrx_daemon/benches/profile_activation.rs`, `keyrx_daemon/benches/ipc_latency.rs`
  - Purpose: Verify performance targets from requirements
  - _Leverage: criterion crate for benchmarking_
  - _Requirements: 2.7 (activation <100ms), 5.5 (latency metrics), IPC performance_
  - _Prompt: Role: Performance engineer with Rust benchmarking expertise | Task: Create performance benchmarks and validation tests to verify all performance requirements |

    **Benchmarks to Create**:
    ```rust
    // benches/profile_activation.rs
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_profile_activation(c: &mut Criterion) {
        c.bench_function("profile_activation", |b| {
            b.iter(|| {
                // Measure ProfileManager::activate() excluding compilation
                // Target: <100ms for hot-reload
            });
        });
    }

    // benches/ipc_latency.rs
    fn bench_ipc_roundtrip(c: &mut Criterion) {
        c.bench_function("ipc_status_query", |b| {
            b.iter(|| {
                // Measure UnixSocketIpc send_request + receive_response
                // Target: <10ms for status queries
            });
        });
    }
    ```

    **Performance Targets to Verify**:
    - Profile activation (hot-reload): <100ms (excluding compilation)
    - IPC status query: <10ms roundtrip
    - Device registry save: <50ms (atomic write)
    - Simulation replay: Deterministic (measure variance = 0 for same seed)

    **Validation Script**:
    ```bash
    # Run benchmarks
    cargo bench --bench profile_activation
    cargo bench --bench ipc_latency

    # Extract results
    cat target/criterion/profile_activation/*/estimates.json | \
      jq '.mean.point_estimate / 1000000' # Convert to ms

    # Fail if targets exceeded
    [ $(jq '.mean.point_estimate / 1000000' < results.json) -lt 100 ] || exit 1
    ```

  | Restrictions: Use criterion for benchmarks, measure excluding I/O where specified, run 1000+ iterations for statistical significance, CI runs benchmarks and fails on regression
  | Success: ✅ Profile activation <100ms verified, ✅ IPC latency <10ms verified, ✅ All performance targets met, ✅ Benchmarks run in CI, ✅ Regression detection configured
  | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (document benchmark results, performance metrics), then mark [x] when done

- [ ] 20. Quality validation (clippy, coverage, file sizes)
  - Files: All implementation files
  - Purpose: Verify code quality gates before spec completion
  - _Leverage: cargo clippy, cargo-llvm-cov, custom scripts_
  - _Requirements: Code quality standards from CLAUDE.md_
  - _Prompt: Role: QA lead with code quality expertise | Task: Run comprehensive quality checks and verify all quality gates pass |

    **Quality Gates**:
    ```bash
    # 1. Clippy (zero warnings)
    cargo clippy --workspace -- -D warnings
    # Exit code must be 0

    # 2. Code coverage (≥80% overall, ≥90% for business logic)
    cargo llvm-cov --workspace --json --output-path coverage.json
    jq '.data[0].totals.lines.percent' coverage.json
    # Must be ≥ 80.0

    # 3. File size limits
    find keyrx_daemon/src -name '*.rs' -exec wc -l {} + | \
      awk '{if ($1 > 500) {print $2 " exceeds 500 lines (" $1 ")"; exit 1}}'

    # 4. Function size limits (use cargo-geiger or custom script)
    # Verify no function exceeds 50 lines

    # 5. Documentation coverage
    cargo doc --no-deps
    # All pub items must have doc comments
    ```

    **Specific Checks**:
    - Clippy warnings: 0 (use -D warnings)
    - Overall coverage: ≥80%
    - Business logic coverage: ≥90% (DeviceRegistry, ProfileManager, etc.)
    - File sizes: ≤500 lines (excluding comments/blank)
    - Function sizes: ≤50 lines
    - Public API documentation: 100% (all pub items)

    **Coverage by Module**:
    ```
    keyrx_daemon/src/config/device_registry.rs: ≥90%
    keyrx_daemon/src/config/profile_manager.rs: ≥90%
    keyrx_daemon/src/ipc/unix_socket.rs: ≥85%
    keyrx_daemon/src/cli/*.rs: ≥80%
    keyrx_daemon/src/web/api.rs: ≥75%
    ```

  | Restrictions: All checks must pass (exit code 0), use CI-compatible tools, generate reports in JSON format, fail fast on first violation
  | Success: ✅ Zero clippy warnings, ✅ Coverage ≥80% overall and ≥90% for business logic, ✅ All files ≤500 lines, ✅ All functions ≤50 lines, ✅ All pub items documented, ✅ Quality report generated
  | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (coverage percentages, file sizes, quality metrics), then mark [x] when done

- [ ] 21. Implementation logging and knowledge base
  - Files: `.spec-workflow/specs/web-ui-ux-comprehensive/implementation-log.json`
  - Purpose: Create searchable knowledge base for future AI agents
  - _Leverage: spec-workflow log-implementation tool_
  - _Requirements: All completed tasks_
  - _Prompt: Role: Documentation engineer | Task: Use log-implementation tool to comprehensively document all implemented artifacts for future discoverability |

    **CRITICAL**: This task uses the `mcp spec-workflow log-implementation` tool for EVERY completed task.

    **Artifacts to Document**:

    For each task, call log-implementation with:
    ```json
    {
      "specName": "web-ui-ux-comprehensive",
      "taskId": "1",
      "summary": "Implemented DeviceRegistry with atomic writes and validation",
      "filesModified": [],
      "filesCreated": ["keyrx_daemon/src/config/device_registry.rs"],
      "statistics": {
        "linesAdded": 350,
        "linesRemoved": 0
      },
      "artifacts": {
        "classes": [{
          "name": "DeviceRegistry",
          "purpose": "Persistent device metadata storage with atomic writes",
          "location": "keyrx_daemon/src/config/device_registry.rs",
          "methods": ["load", "save", "rename", "set_scope", "set_layout", "forget", "list", "get"],
          "isExported": true
        }],
        "functions": [{
          "name": "validate_device_name",
          "purpose": "Validate device name length and characters",
          "location": "keyrx_daemon/src/config/device_registry.rs:25",
          "signature": "fn validate_device_name(name: &str) -> Result<(), RegistryError>",
          "isExported": false
        }]
      }
    }
    ```

    **For CLI Commands** (Tasks 6-15):
    - Document as apiEndpoints (even though they're CLI, they're endpoints for automation)
    ```json
    "apiEndpoints": [{
      "method": "CLI",
      "path": "keyrx devices list",
      "purpose": "List all detected devices with metadata",
      "requestFormat": "--json flag optional",
      "responseFormat": "{ devices: DeviceEntry[] }",
      "location": "keyrx_daemon/src/cli/devices.rs:45"
    }]
    ```

    **For Business Logic**:
    - Document classes (ProfileManager, DeviceRegistry, etc.)
    - Document key functions with signatures
    - Document error enums with all variants

    **For IPC**:
    - Document IpcRequest/IpcResponse enums as apiEndpoints
    - Document integration (CLI → IPC → Daemon data flow)

  | Restrictions: Use log-implementation tool for ALL tasks, include complete artifact data (not summaries), document all public APIs, record exact file locations with line numbers, include data flow diagrams in integrations
  | Success: ✅ All 21 tasks logged with detailed artifacts, ✅ Implementation log searchable via grep, ✅ Future agents can discover "How do I list devices?" → finds Task 6 artifacts, ✅ Knowledge base complete
  | Instructions: For each completed task (0-20), call log-implementation tool with comprehensive artifact data, then mark [x] when all logging complete

---

## Summary Statistics

**Total Tasks**: 21 (CLI-first focus, web UI moved to separate spec)
**Estimated Effort**: 50-65 hours (2-2.5 weeks full-time)

**By Phase**:
- Phase 0 (Environment Setup): 1 task, ~3 hours
- Phase 1 (Business Logic): 5 tasks, ~15 hours
- Phase 2 (CLI Devices/Profiles): 2 tasks, ~8 hours
- Phase 3 (CLI Config/Layers): 3 tasks, ~12 hours
- Phase 4 (CLI Simulation/Monitoring): 5 tasks, ~15 hours
- Phase 5 (Integration/Testing): 3 tasks, ~10 hours
- Phase 6 (Validation & Verification): 3 tasks, ~5 hours

**Milestones**:
- ✅ Phase 0 complete → Dependencies installed, IPC infrastructure ready
- ✅ Phase 1 complete → Business logic testable via unit tests
- ✅ Phase 2-4 complete → CLI fully functional, autonomous testing enabled
- ✅ Phase 5 complete → CI/CD verified, production-ready CLI
- ✅ Phase 6 complete → Performance validated, quality gates passed, implementation logged, **READY FOR v1.0 RELEASE**

**Critical Path**: All phases (Phases 0-6) - Pure CLI implementation
**Next Steps**: After v1.0 CLI release, implement web-ui-configuration-editor spec for v1.1

**Testing Philosophy**: Every feature has CLI test before any UI is built. Web UI tested via API integration tests that verify same logic as CLI.
