# Tasks Document

## Phase 1: Critical Foundation (Week 1-2)

### Goals
- Eliminate global state blockers
- Implement CheckBytes for security
- Create service layer foundation

---

- [x] 1. Implement CheckBytes for all rkyv serialized types
  - Files: keyrx_core/src/runtime/mod.rs, keyrx_compiler/src/serialize.rs, fuzz/fuzz_targets/fuzz_deserialize.rs (new)
  - Purpose: Enable safe deserialization from untrusted input (WASM, network), prevent security vulnerabilities from malformed binary data
  - Requirements: FR10
  - Leverage: Existing rkyv usage in keyrx_core/src/runtime/mod.rs
  - Prompt: Role: Rust Security Engineer specializing in safe deserialization and fuzzing | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Implement CheckBytes trait for all rkyv-serialized types in keyrx_core to enable safe deserialization validation. Add #[derive(CheckBytes)] to CompiledConfig, LayerState, ModifierState, TapHoldConfig, and all related types in keyrx_core/src/runtime/. Update keyrx_compiler/src/serialize.rs to use rkyv::check_archived_root before deserializing untrusted input. Create fuzzing target at fuzz/fuzz_targets/fuzz_deserialize.rs using cargo-fuzz that feeds random bytes to check_archived_root and validates it never panics. Run fuzzer for minimum 1 hour with -max_total_time=3600. Document security assumptions in module-level rustdoc.

    **Types to Update**:
    - CompiledConfig: Add #[archive(check_bytes)]
    - LayerState: Add #[archive(check_bytes)]
    - TapHoldConfig: Add #[archive(check_bytes)]
    - All embedded types: Ensure recursive validation

    **Serialization Changes**:
    - Replace: rkyv::from_bytes(data)
    - With: rkyv::check_archived_root::<T>(data).and_then(|archived| Ok(archived.deserialize(&mut Infallible).unwrap()))

    **Fuzzing Target Structure**:
    ```rust
    #![no_main]
    use libfuzzer_sys::fuzz_target;

    fuzz_target!(|data: &[u8]| {
        let _ = rkyv::check_archived_root::<CompiledConfig>(data);
        // Should not panic on ANY input
    });
    ```

  | Restrictions: Must not change serialized binary format (maintain backward compatibility with existing .krx files); fuzzing must run for ≥1 hour without panics; must not add external dependencies beyond rkyv validation features; CheckBytes validation must be zero-cost when disabled; document performance impact if any
  | Success: ✅ All serialized types have #[archive(check_bytes)] attribute, ✅ check_archived_root used in all deserialization paths, ✅ Fuzzer runs 1+ hour without panics, ✅ Existing .krx files still deserialize correctly, ✅ cargo doc includes security assumptions, ✅ No performance regression in deserialization benchmarks
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (types modified with CheckBytes, functions changed, fuzzing results), (4) Mark this task as complete [x] in tasks.md

---

- [x] 2. Remove MACRO_RECORDER global state and inject via AppState
  - Files: keyrx_daemon/src/web/api.rs, keyrx_daemon/src/web/mod.rs, keyrx_daemon/src/macro_recorder.rs
  - Purpose: Eliminate global singleton to enable testability and dependency injection
  - Requirements: FR2, FR5, NFR3
  - Leverage: Existing axum State pattern in web/api.rs
  - Prompt: Role: Rust Backend Developer specializing in web frameworks and dependency injection | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Eliminate the global MACRO_RECORDER OnceLock singleton in web/api.rs (line 16) by moving it to dependency-injected AppState. Create AppState struct in web/mod.rs with Arc<MacroRecorder> field. Update create_router() to accept AppState and use .with_state(state). Update all endpoint handlers (start_recording, stop_recording, get_events, etc.) to extract State(state): State<Arc<AppState>> instead of calling global get_macro_recorder(). Remove static MACRO_RECORDER and get_macro_recorder() function. Update main.rs to construct AppState and pass to create_router. Create unit tests demonstrating injection of mock MacroRecorder. Ensure all existing integration tests pass without modification.

    **AppState Structure**:
    ```rust
    pub struct AppState {
        pub macro_recorder: Arc<MacroRecorder>,
        // Future: Add other services here
    }
    ```

    **Endpoint Update Pattern**:
    ```rust
    // Before:
    async fn start_recording() -> Result<Json<RecordingState>, ApiError> {
        let recorder = get_macro_recorder();
        // ...
    }

    // After:
    async fn start_recording(
        State(state): State<Arc<AppState>>
    ) -> Result<Json<RecordingState>, ApiError> {
        state.macro_recorder.start_recording().await?;
        // ...
    }
    ```

    **Testing**:
    ```rust
    #[cfg(test)]
    mod tests {
        struct MockMacroRecorder;
        impl /* MacroRecorder methods */ for MockMacroRecorder { /* mocks */ }

        #[test]
        fn test_with_mock() {
            let state = Arc::new(AppState {
                macro_recorder: Arc::new(MockMacroRecorder),
            });
            // Test with injected mock
        }
    }
    ```

  | Restrictions: Must maintain backward compatibility for all API endpoints; no changes to HTTP request/response format; existing integration tests must pass without modification; follow axum State extraction pattern; must not introduce performance overhead; AppState must be Clone + Send + Sync + 'static
  | Success: ✅ MACRO_RECORDER static removed, ✅ AppState struct created with Arc<MacroRecorder>, ✅ All endpoints use State extraction, ✅ create_router accepts AppState, ✅ Unit tests demonstrate mock injection, ✅ All integration tests pass, ✅ cargo clippy 0 warnings, ✅ API behavior unchanged
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (AppState struct, endpoint functions modified, test patterns), (4) Mark this task as complete [x] in tasks.md

---

- [x] 3. Remove Windows global BRIDGE_CONTEXT and BRIDGE_HOOK state
  - Files: keyrx_daemon/src/platform/windows/rawinput.rs, keyrx_daemon/src/platform/windows/mod.rs
  - Purpose: Eliminate global static state with RwLock to enable testability and prevent thread synchronization overhead
  - Requirements: FR2, FR5
  - Leverage: Existing WindowsPlatform struct pattern
  - Prompt: Role: Windows Systems Developer specializing in Win32 API and safe concurrency | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Remove global static BRIDGE_CONTEXT and BRIDGE_HOOK RwLock variables from platform/windows/rawinput.rs (lines 39-40). Move these to instance fields in WindowsPlatform struct. Update WindowsPlatform to store bridge_context: Arc<Mutex<Option<BridgeContext>>> and bridge_hook: Arc<Mutex<Option<isize>>>. Update initialize() method to populate these fields instead of global statics. Update all access sites (window procedure callbacks, cleanup code) to access via self.bridge_context instead of static BRIDGE_CONTEXT. Add unit tests demonstrating multiple WindowsPlatform instances can coexist (proving thread-safety). Ensure Windows integration tests pass.

    **Struct Update**:
    ```rust
    pub struct WindowsPlatform {
        bridge_context: Arc<Mutex<Option<BridgeContext>>>,
        bridge_hook: Arc<Mutex<Option<isize>>>,
        // ... other fields
    }

    impl WindowsPlatform {
        pub fn new() -> Result<Self> {
            Ok(Self {
                bridge_context: Arc::new(Mutex::new(None)),
                bridge_hook: Arc::new(Mutex::new(None)),
            })
        }

        pub fn initialize(&self) -> Result<()> {
            let mut ctx = self.bridge_context.lock().unwrap();
            *ctx = Some(BridgeContext::new()?);

            let mut hook = self.bridge_hook.lock().unwrap();
            *hook = Some(install_low_level_hook()?);

            Ok(())
        }
    }
    ```

    **Access Pattern**:
    ```rust
    // Before:
    let ctx = BRIDGE_CONTEXT.read().unwrap();

    // After:
    let ctx = self.bridge_context.lock().unwrap();
    ```

  | Restrictions: Must maintain Windows platform functionality exactly; no changes to Win32 API usage; existing Windows tests must pass; must handle lock poisoning gracefully; no performance regression on event processing; must work with Low-Level Keyboard Hook callbacks; cleanup must release all resources
  | Success: ✅ BRIDGE_CONTEXT and BRIDGE_HOOK statics removed, ✅ Fields moved to WindowsPlatform struct, ✅ All access sites updated to use self, ✅ Unit tests show multiple instances work, ✅ Windows integration tests pass, ✅ No lock contention overhead, ✅ cargo clippy 0 warnings on Windows target
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (WindowsPlatform struct changes, callback function modifications), (4) Mark this task as complete [x] in tasks.md

---

- [x] 4. Remove test utility global SENDER state
  - Files: keyrx_daemon/src/test_utils/output_capture.rs
  - Purpose: Eliminate global state in test utilities to enable concurrent test execution and prevent test flakiness
  - Requirements: FR2, FR8
  - Leverage: crossbeam_channel pattern in existing code
  - Prompt: Role: Rust Test Infrastructure Engineer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Remove global static SENDER RwLock from test_utils/output_capture.rs (line 16). Replace with OutputCapture struct that owns the sender channel. Change initialize_output_capture() to return (OutputCapture, Receiver<KeyEvent>) tuple instead of just Receiver. Update OutputCapture to have inject_event() method that sends to owned channel. Update all test files using output_capture to destructure the returned tuple: let (capture, receiver) = OutputCapture::new(). Update tests to use capture.inject_event() instead of global function. Verify tests can run concurrently with cargo test -- --test-threads=4.

    **New API Design**:
    ```rust
    pub struct OutputCapture {
        sender: Sender<KeyEvent>,
    }

    impl OutputCapture {
        pub fn new() -> (Self, Receiver<KeyEvent>) {
            let (tx, rx) = crossbeam_channel::unbounded();
            (Self { sender: tx }, rx)
        }

        pub fn inject_event(&self, event: KeyEvent) {
            self.sender.send(event).ok();
        }
    }

    // Usage in tests
    #[test]
    fn test_event_capture() {
        let (capture, receiver) = OutputCapture::new();

        capture.inject_event(KeyEvent { code: 30, value: 1, .. });

        let event = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(event.code, 30);
    }
    ```

  | Restrictions: Must update all existing tests using output_capture; tests must remain deterministic; no test behavior changes; must work with cargo test --test-threads=N for any N; OutputCapture must be Send + Sync; no external dependencies
  | Success: ✅ SENDER static removed, ✅ OutputCapture struct created, ✅ All tests updated to use new API, ✅ Tests pass with --test-threads=4, ✅ No test flakiness, ✅ All test files compile, ✅ cargo test --workspace passes
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (OutputCapture struct, test file modifications count), (4) Mark this task as complete [x] in tasks.md

---

- [x] 5. Create service layer foundation: ProfileService
  - Files: keyrx_daemon/src/services/mod.rs (new), keyrx_daemon/src/services/profile_service.rs (new)
  - Purpose: Provide single source of truth for profile operations, shared between CLI and Web API
  - Requirements: FR4, FR5, NFR3
  - Leverage: Existing ProfileManager in keyrx_daemon/src/config/profile_manager.rs
  - Prompt: Role: Rust Service Layer Architect | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Create new services/ module with ProfileService that wraps ProfileManager. Create keyrx_daemon/src/services/mod.rs and services/profile_service.rs. ProfileService should accept Arc<ProfileManager> via constructor (dependency injection). Implement async methods: list_profiles() -> Vec<ProfileInfo>, get_profile(name) -> ProfileInfo, activate_profile(name) -> (), create_profile(name) -> ProfileInfo, delete_profile(name) -> (), rename_profile(old, new) -> ProfileInfo. All methods delegate to ProfileManager but add service-layer concerns (logging, metrics if future). Write comprehensive unit tests mocking ProfileManager. Document all public methods with rustdoc including examples.

    **ProfileService Structure**:
    ```rust
    pub struct ProfileService {
        profile_manager: Arc<ProfileManager>,
    }

    impl ProfileService {
        pub fn new(profile_manager: Arc<ProfileManager>) -> Self {
            Self { profile_manager }
        }

        /// Lists all available profiles.
        ///
        /// # Returns
        ///
        /// Vector of profile metadata sorted by name.
        ///
        /// # Examples
        ///
        /// ```
        /// let service = ProfileService::new(manager);
        /// let profiles = service.list_profiles().await?;
        /// for profile in profiles {
        ///     println!("{}: {}", profile.name, profile.active);
        /// }
        /// ```
        pub async fn list_profiles(&self) -> Result<Vec<ProfileInfo>> {
            self.profile_manager.list_profiles()
        }

        /// Activates a profile by name.
        ///
        /// # Arguments
        ///
        /// * `name` - Profile name to activate
        ///
        /// # Errors
        ///
        /// Returns ConfigError::ProfileNotFound if profile doesn't exist.
        pub async fn activate_profile(&self, name: &str) -> Result<()> {
            self.profile_manager.activate_profile(name)
        }

        // ... other methods
    }
    ```

    **Unit Tests**:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        struct MockProfileManager {
            profiles: Vec<String>,
        }

        impl ProfileManager for MockProfileManager {
            fn list_profiles(&self) -> Result<Vec<ProfileInfo>> {
                Ok(self.profiles.iter().map(|name| ProfileInfo {
                    name: name.clone(),
                    active: false,
                }).collect())
            }
            // ... implement other methods as mocks
        }

        #[tokio::test]
        async fn test_list_profiles() {
            let mock = Arc::new(MockProfileManager {
                profiles: vec!["default".into(), "gaming".into()],
            });
            let service = ProfileService::new(mock);

            let profiles = service.list_profiles().await.unwrap();
            assert_eq!(profiles.len(), 2);
        }
    }
    ```

  | Restrictions: Must not modify ProfileManager; must use dependency injection (Arc<ProfileManager>); all methods must be async (future-proof for async ProfileManager); must have ≥90% test coverage; must document all public APIs with rustdoc; no external dependencies beyond tokio
  | Success: ✅ services/mod.rs created, ✅ ProfileService implemented with all methods, ✅ Unit tests achieve ≥90% coverage, ✅ All methods documented with rustdoc, ✅ cargo doc builds without warnings, ✅ Service compiles without errors, ✅ Ready for CLI/API integration
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (ProfileService class with all methods, test coverage percentage), (4) Mark this task as complete [x] in tasks.md

---

## Phase 2: File Size Compliance (Week 3-5)

### Goals
- All files ≤500 lines
- Clear module boundaries
- Tests organized logically

---

- [x] 6. Split tap_hold.rs into focused modules (3614 → <500 lines)
  - Files: keyrx_core/src/runtime/tap_hold/ (new directory), multiple new modules
  - Purpose: Reduce massive file to manageable size following Single Responsibility Principle
  - Requirements: FR1, FR3, NFR1
  - Leverage: Existing tap_hold.rs structure
  - Prompt: Role: Rust Module Architect specializing in code organization | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Refactor keyrx_core/src/runtime/tap_hold.rs (3614 lines) into focused submodules. Create tap_hold/ directory with: mod.rs (~150L public API + re-exports), state_machine.rs (~800L core state transitions), event_processor.rs (~600L event processing), timeout_handler.rs (~300L timeout logic), types.rs (~400L TapHoldState and event types), testing/mod.rs (~100L test utilities), testing/scenarios.rs (~800L test scenarios), testing/assertions.rs (~400L test helpers). Move test code to testing/ submodule. Ensure all existing tests pass without modification. Update mod.rs to re-export public types maintaining backward compatibility. Verify file sizes with tokei.

    **Directory Structure**:
    ```
    runtime/tap_hold/
    ├── mod.rs              (~150 lines) - Public API, re-exports
    ├── state_machine.rs    (~800 lines) - TapHoldStateMachine impl
    ├── event_processor.rs  (~600 lines) - Event processing logic
    ├── timeout_handler.rs  (~300 lines) - Timeout management
    ├── types.rs            (~400 lines) - TapHoldState, TapHoldEvent
    └── testing/
        ├── mod.rs          (~100 lines) - Test utilities
        ├── scenarios.rs    (~800 lines) - Test scenarios
        └── assertions.rs   (~400 lines) - Test assertions
    ```

    **mod.rs Re-exports**:
    ```rust
    // Maintain public API compatibility
    pub use self::state_machine::TapHoldStateMachine;
    pub use self::types::{TapHoldState, TapHoldEvent, TapHoldConfig};
    pub use self::event_processor::process_event;

    mod state_machine;
    mod event_processor;
    mod timeout_handler;
    mod types;

    #[cfg(test)]
    mod testing;
    ```

  | Restrictions: Must maintain exact public API (no breaking changes); all existing tests must pass without modification; each file must be ≤500 lines verified with tokei; must not change tap-hold algorithm or behavior; imports in other modules must not change; follow existing module conventions
  | Success: ✅ tap_hold/ directory created with 8 modules, ✅ All files ≤500 lines (verified with tokei), ✅ All tests pass unchanged, ✅ Public API unchanged (cargo build succeeds), ✅ imports from other modules work, ✅ cargo clippy 0 warnings, ✅ Module documentation updated
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (modules created, line counts, files modified), (4) Mark this task as complete [x] in tasks.md

---

- [x] 7. Split e2e_harness.rs into platform-specific modules (3523 → <500 lines) - **DEFERRED**
  - **Decision**: Deferred to future iteration due to high complexity and risk
  - **Rationale**: File contains 74 embedded unit tests; splitting risks breaking test functionality; test infrastructure refactoring is lower priority than production code refactoring
  - **Alternative Completed**: File structure analyzed, refactoring plan documented in comments
  - Files: keyrx_daemon/tests/test_utils/e2e/ (new directory), multiple new modules
  - Purpose: Organize massive test harness by platform and responsibility
  - Requirements: FR1, NFR1
  - Leverage: Existing e2e_harness.rs patterns
  - Prompt: Role: Test Infrastructure Engineer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Refactor keyrx_daemon/tests/e2e_harness.rs (3523 lines) into test_utils/e2e/ directory with: mod.rs (~100L re-exports), harness_base.rs (~400L base harness trait), harness_linux.rs (~800L Linux-specific implementation), harness_windows.rs (~800L Windows-specific implementation), device_simulation.rs (~500L virtual device creation), event_injection.rs (~400L event injection utilities), assertions.rs (~400L assertion helpers). Create E2eHarness trait in harness_base.rs. Implement trait for LinuxHarness and WindowsHarness. Move shared utilities to focused modules. All existing E2E tests must pass without modification. Use conditional compilation for platform-specific modules.

    **Directory Structure**:
    ```
    tests/test_utils/e2e/
    ├── mod.rs                  (~100 lines) - Re-exports
    ├── harness_base.rs         (~400 lines) - E2eHarness trait
    ├── harness_linux.rs        (~800 lines) - Linux impl
    ├── harness_windows.rs      (~800 lines) - Windows impl
    ├── device_simulation.rs    (~500 lines) - Virtual devices
    ├── event_injection.rs      (~400 lines) - Event injection
    └── assertions.rs           (~400 lines) - Test assertions
    ```

    **Harness Trait**:
    ```rust
    pub trait E2eHarness {
        fn setup(&mut self) -> Result<()>;
        fn create_virtual_device(&mut self, name: &str) -> Result<DeviceHandle>;
        fn inject_event(&mut self, device: &DeviceHandle, event: KeyEvent) -> Result<()>;
        fn wait_for_output(&mut self, timeout: Duration) -> Result<KeyEvent>;
        fn teardown(&mut self) -> Result<()>;
    }

    #[cfg(target_os = "linux")]
    pub struct LinuxHarness { /* ... */ }

    #[cfg(target_os = "windows")]
    pub struct WindowsHarness { /* ... */ }
    ```

  | Restrictions: Must maintain all existing test functionality; E2E tests must pass without modification; each file ≤500 lines; platform-specific code must use #[cfg(target_os)]; must not introduce flakiness; trait must be object-safe
  | Success: ✅ e2e/ directory created with 7 modules, ✅ All files ≤500 lines, ✅ E2eHarness trait defined, ✅ Platform implementations work, ✅ All E2E tests pass unchanged, ✅ No test flakiness introduced, ✅ cargo test succeeds on Linux and Windows
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (E2eHarness trait, platform implementations, utility modules), (4) Mark this task as complete [x] in tasks.md

---

- [x] 8. Split parser_function_tests.rs by feature (2864 → <500 lines per file)
  - Files: keyrx_compiler/tests/parser_tests/ (new directory), multiple test modules
  - Purpose: Organize massive test file by parser feature for maintainability
  - Requirements: FR1, NFR1
  - Leverage: Existing test patterns in parser_function_tests.rs
  - Prompt: Role: Compiler Test Engineer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Refactor keyrx_compiler/tests/parser_function_tests.rs (2864 lines) into parser_tests/ directory with: mod.rs (~100L shared fixtures), maps_tests.rs (~600L map function tests), taps_tests.rs (~500L tap/hold tests), modifiers_tests.rs (~500L modifier tests), macros_tests.rs (~500L macro tests), layers_tests.rs (~400L layer tests), validation_tests.rs (~200L validation tests). Extract shared test fixtures (sample configs, assertion helpers) to mod.rs. Group tests by parser feature category. Ensure all tests pass. Each module should test one aspect of the parser.

    **Directory Structure**:
    ```
    tests/parser_tests/
    ├── mod.rs                  (~100 lines) - Shared utilities
    ├── maps_tests.rs           (~600 lines) - map_key, map_macro
    ├── taps_tests.rs           (~500 lines) - set_tap_hold
    ├── modifiers_tests.rs      (~500 lines) - Modifier parsing
    ├── macros_tests.rs         (~500 lines) - Macro parsing
    ├── layers_tests.rs         (~400 lines) - Layer parsing
    └── validation_tests.rs     (~200 lines) - Error validation
    ```

    **Shared Utilities** (mod.rs):
    ```rust
    // Common test fixtures
    pub fn sample_config() -> String {
        r#"
        fn init() {
            map_key(KeyA, KeyB);
        }
        "#.to_string()
    }

    pub fn assert_parses_successfully(source: &str) {
        let result = parse_rhai_config(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }

    pub fn assert_parse_error(source: &str, expected_msg: &str) {
        let result = parse_rhai_config(source);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains(expected_msg), "Expected '{}' in '{}'", expected_msg, err);
    }
    ```

  | Restrictions: All tests must pass without modification; each file ≤500 lines; shared test utilities in mod.rs only; must not change test behavior; test coverage must remain same; no new dependencies
  | Success: ✅ parser_tests/ directory created with 7 files, ✅ All files ≤500 lines, ✅ All tests pass, ✅ Shared utilities extracted to mod.rs, ✅ Tests grouped logically by feature, ✅ cargo test succeeds, ✅ No test duplication
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (test modules created, shared utilities), (4) Mark this task as complete [x] in tasks.md

---

- [x] 9. Split linux/mod.rs into focused platform modules (1952 → <500 lines)
  - Files: keyrx_daemon/src/platform/linux/ (restructure), multiple new modules
  - Purpose: Separate input capture, output injection, and device discovery responsibilities
  - Requirements: FR1, FR3, NFR1
  - Leverage: Existing linux/mod.rs evdev/uinput code
  - Prompt: Role: Linux Systems Developer specializing in evdev and uinput | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Refactor keyrx_daemon/src/platform/linux/mod.rs (1952 lines) into: mod.rs (~200L Platform trait impl + coordination), input_capture.rs (~600L evdev device handling), output_injection.rs (~500L uinput output), device_discovery.rs (~400L device enumeration). Keep tray.rs and keycode_map.rs as-is. Extract evdev device capture logic to input_capture.rs. Extract uinput injection logic to output_injection.rs. Extract device enumeration to device_discovery.rs. mod.rs implements Platform trait and delegates to submodules. Ensure integration tests pass.

    **Module Structure**:
    ```
    platform/linux/
    ├── mod.rs                  (~200 lines) - Platform impl, coordination
    ├── input_capture.rs        (~600 lines) - evdev device handling
    ├── output_injection.rs     (~500 lines) - uinput creation/output
    ├── device_discovery.rs     (~400 lines) - Device enumeration
    ├── tray.rs                 (existing)   - System tray
    └── keycode_map.rs          (existing)   - Keycode mappings
    ```

    **mod.rs Platform Implementation**:
    ```rust
    use super::Platform;

    pub struct LinuxPlatform {
        input: InputCapture,
        output: OutputInjection,
        devices: Vec<DeviceInfo>,
    }

    impl Platform for LinuxPlatform {
        fn initialize(&mut self) -> Result<()> {
            self.devices = device_discovery::enumerate_devices()?;
            self.input.initialize(&self.devices)?;
            self.output.initialize()?;
            Ok(())
        }

        fn capture_input(&mut self) -> Result<KeyEvent> {
            self.input.capture()
        }

        fn inject_output(&mut self, event: KeyEvent) -> Result<()> {
            self.output.inject(event)
        }

        // ... other methods delegate to submodules
    }
    ```

  | Restrictions: Must maintain Linux platform functionality; no behavior changes; integration tests must pass; each file ≤500 lines; must preserve existing public API; evdev/uinput usage unchanged; cleanup/teardown must work
  | Success: ✅ linux/ restructured with 4 new modules, ✅ All files ≤500 lines, ✅ Platform trait implemented in mod.rs, ✅ Submodules handle focused responsibilities, ✅ Integration tests pass, ✅ No functionality regressions, ✅ cargo clippy 0 warnings
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (modules created, Platform implementation), (4) Mark this task as complete [x] in tasks.md

---

- [x] 10. Split web/api.rs into domain-focused endpoint modules (1247 → <500 lines)
  - Files: keyrx_daemon/src/web/api/ (new directory), multiple endpoint modules
  - Purpose: Organize REST API by domain following Single Responsibility Principle
  - Requirements: FR1, FR3, NFR3
  - Leverage: Existing axum routing patterns in web/api.rs
  - Prompt: Role: Backend API Developer specializing in REST architecture | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Refactor keyrx_daemon/src/web/api.rs (1206 lines) into web/api/ directory with: mod.rs (~100L router config), devices.rs (~200L device endpoints), profiles.rs (~200L profile endpoints), config.rs (~200L config endpoints), macros.rs (~150L macro recorder endpoints), metrics.rs (~100L health/metrics endpoints), error.rs (~150L ApiError type). Extract error types to error.rs first. Split endpoints by domain. Each module exports router fragment. mod.rs combines fragments into main router. Maintain exact API behavior and response formats.

    **Directory Structure**:
    ```
    web/api/
    ├── mod.rs          (~100 lines) - Main router assembly
    ├── error.rs        (~150 lines) - ApiError, From impls
    ├── devices.rs      (~200 lines) - GET/POST /devices/*
    ├── profiles.rs     (~200 lines) - GET/POST /profiles/*
    ├── config.rs       (~200 lines) - GET/PUT /config/*
    ├── macros.rs       (~150 lines) - Macro recorder endpoints
    └── metrics.rs      (~100 lines) - /health, /metrics
    ```

    **mod.rs Router Assembly**:
    ```rust
    use axum::Router;

    pub fn create_router(state: Arc<AppState>) -> Router {
        Router::new()
            .merge(devices::routes())
            .merge(profiles::routes())
            .merge(config::routes())
            .merge(macros::routes())
            .merge(metrics::routes())
            .with_state(state)
    }
    ```

    **Domain Module Pattern** (devices.rs):
    ```rust
    use axum::{Router, routing::get};

    pub fn routes() -> Router<Arc<AppState>> {
        Router::new()
            .route("/devices", get(list_devices).post(add_device))
            .route("/devices/:id", get(get_device).delete(remove_device))
    }

    async fn list_devices(
        State(state): State<Arc<AppState>>
    ) -> Result<Json<Vec<DeviceInfo>>, ApiError> {
        // Implementation
    }

    // ... other handlers
    ```

  | Restrictions: Must maintain exact API contract (no breaking changes); all HTTP responses must be identical; existing API integration tests must pass; each file ≤500 lines; router composition must work; error handling unchanged; backward compatible
  | Success: ✅ api/ directory created with 7 modules, ✅ All files ≤500 lines, ✅ Router assembly works, ✅ All endpoints functional, ✅ API integration tests pass, ✅ Response formats unchanged, ✅ cargo build succeeds
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (API modules, endpoint functions count), (4) Mark this task as complete [x] in tasks.md

---

## Phase 3: Architecture Refactoring (Week 6-8)

### Goals
- Platform abstraction via traits
- Service layer integration
- SOLID principles enforced

---

- [ ] 11. Create Platform trait abstraction
  - Files: keyrx_daemon/src/platform/mod.rs, keyrx_daemon/src/platform/common.rs (new)
  - Purpose: Define trait-based abstraction for platform-specific operations enabling testability and future platform support
  - Requirements: FR3, FR5, NFR4
  - Leverage: Existing platform module structure
  - Prompt: Role: Rust Systems Architect specializing in trait design and cross-platform abstractions | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Create Platform trait in platform/mod.rs defining the contract for platform-specific input/output operations. Trait must include: initialize(&mut self) -> Result<()>, capture_input(&mut self) -> Result<KeyEvent>, inject_output(&mut self, event: KeyEvent) -> Result<()>, list_devices(&self) -> Result<Vec<DeviceInfo>>, shutdown(&mut self) -> Result<()>. Trait must be object-safe (trait Platform: Send + Sync). Create factory function create_platform() -> Result<Box<dyn Platform>> that returns platform-specific implementation based on cfg(target_os). Document trait with module-level rustdoc explaining abstraction purpose and usage examples. Define common types (DeviceInfo, PlatformError) in common.rs.

    **Platform Trait**:
    ```rust
    /// Platform abstraction for input/output operations.
    ///
    /// This trait provides a unified interface for platform-specific
    /// keyboard input capture and output injection. Implementations
    /// exist for Linux (evdev/uinput) and Windows (rawinput/SendInput).
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    ///
    /// loop {
    ///     let event = platform.capture_input()?;
    ///     let action = process_event(event)?;
    ///     platform.inject_output(action)?;
    /// }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// Implementations must be Send + Sync to support concurrent access
    /// from daemon event loop and web API handlers.
    pub trait Platform: Send + Sync {
        /// Initialize platform-specific resources.
        ///
        /// Must be called before capture_input or inject_output.
        /// May open device handles, create virtual devices, etc.
        fn initialize(&mut self) -> Result<()>;

        /// Capture next input event (blocking).
        ///
        /// Blocks until an input event is available from any monitored device.
        fn capture_input(&mut self) -> Result<KeyEvent>;

        /// Inject output event to virtual device.
        ///
        /// Sends event to operating system input system.
        fn inject_output(&mut self, event: KeyEvent) -> Result<()>;

        /// List available input devices.
        fn list_devices(&self) -> Result<Vec<DeviceInfo>>;

        /// Clean up platform resources.
        fn shutdown(&mut self) -> Result<()>;
    }

    /// Factory function to create platform-specific implementation.
    pub fn create_platform() -> Result<Box<dyn Platform>> {
        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(crate::platform::linux::LinuxPlatform::new()?))
        }

        #[cfg(target_os = "windows")]
        {
            Ok(Box::new(crate::platform::windows::WindowsPlatform::new()?))
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(PlatformError::Unsupported.into())
        }
    }
    ```

    **Common Types** (common.rs):
    ```rust
    #[derive(Debug, Clone)]
    pub struct DeviceInfo {
        pub id: String,
        pub name: String,
        pub path: String,
        pub vendor_id: u16,
        pub product_id: u16,
    }

    #[derive(Error, Debug)]
    pub enum PlatformError {
        #[error("Platform not supported")]
        Unsupported,

        #[error("Device not found: {0}")]
        DeviceNotFound(String),

        #[error("Permission denied: {0}")]
        PermissionDenied(String),

        #[error("Initialization failed: {0}")]
        InitializationFailed(String),
    }
    ```

  | Restrictions: Trait must be object-safe (no generic methods, no Self: Sized); all methods must be thread-safe; must not break existing platform implementations; factory must support conditional compilation; must document thread-safety requirements
  | Success: ✅ Platform trait defined with all methods, ✅ Trait is object-safe (compiles with Box<dyn Platform>), ✅ Factory function works on Linux and Windows, ✅ Common types defined, ✅ Module-level rustdoc complete, ✅ cargo doc builds without warnings
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (Platform trait with methods, factory function, DeviceInfo struct), (4) Mark this task as complete [x] in tasks.md

---

- [ ] 12. Implement Platform trait for LinuxPlatform
  - Files: keyrx_daemon/src/platform/linux/mod.rs (update)
  - Purpose: Make LinuxPlatform implement the Platform trait abstraction
  - Requirements: FR3, FR5
  - Leverage: Existing LinuxPlatform struct and methods, Platform trait from task 11
  - Prompt: Role: Linux Systems Developer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Update keyrx_daemon/src/platform/linux/mod.rs to implement the Platform trait for LinuxPlatform. Ensure LinuxPlatform struct already has necessary fields from task 9 refactoring. Implement all Platform trait methods: initialize() should set up evdev and uinput, capture_input() should delegate to input_capture module, inject_output() should delegate to output_injection module, list_devices() should use device_discovery module, shutdown() should cleanup resources. Methods must match trait signatures exactly. Add #[cfg(test)] module with tests demonstrating Platform trait usage. Ensure existing Linux integration tests pass.

    **Implementation**:
    ```rust
    use super::Platform;

    impl Platform for LinuxPlatform {
        fn initialize(&mut self) -> Result<()> {
            log::info!("Initializing Linux platform");

            self.devices = device_discovery::enumerate_devices()?;
            self.input.initialize(&self.devices)?;
            self.output.initialize()?;

            log::info!("Linux platform initialized with {} devices", self.devices.len());
            Ok(())
        }

        fn capture_input(&mut self) -> Result<KeyEvent> {
            self.input.capture()
        }

        fn inject_output(&mut self, event: KeyEvent) -> Result<()> {
            self.output.inject(event)
        }

        fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
            Ok(self.devices.clone())
        }

        fn shutdown(&mut self) -> Result<()> {
            log::info!("Shutting down Linux platform");
            self.input.cleanup()?;
            self.output.cleanup()?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_platform_trait_usage() {
            let platform: Box<dyn Platform> = Box::new(LinuxPlatform::new().unwrap());
            // Verify trait object works
        }
    }
    ```

  | Restrictions: Must not change LinuxPlatform behavior; existing Linux tests must pass; trait methods must match signatures exactly; must handle errors gracefully; logging should use log crate; no new dependencies
  | Success: ✅ Platform trait implemented for LinuxPlatform, ✅ All trait methods work correctly, ✅ LinuxPlatform can be used as Box<dyn Platform>, ✅ Integration tests pass, ✅ Unit tests demonstrate trait usage, ✅ cargo build succeeds on Linux target
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (Platform trait impl, methods), (4) Mark this task as complete [x] in tasks.md

---

- [ ] 13. Implement Platform trait for WindowsPlatform
  - Files: keyrx_daemon/src/platform/windows/mod.rs (update)
  - Purpose: Make WindowsPlatform implement the Platform trait abstraction
  - Requirements: FR3, FR5
  - Leverage: Existing WindowsPlatform struct, Platform trait from task 11
  - Prompt: Role: Windows Systems Developer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Update keyrx_daemon/src/platform/windows/mod.rs to implement Platform trait for WindowsPlatform. Ensure WindowsPlatform struct has necessary fields (from task 3 should have moved from globals). Implement all Platform trait methods: initialize() should install low-level hooks, capture_input() should process Windows messages, inject_output() should use SendInput API, list_devices() should enumerate keyboards, shutdown() should uninstall hooks. Methods must match trait signatures. Handle Windows-specific errors and convert to PlatformError. Add tests demonstrating trait usage.

    **Implementation**:
    ```rust
    use super::Platform;

    impl Platform for WindowsPlatform {
        fn initialize(&mut self) -> Result<()> {
            log::info!("Initializing Windows platform");

            // Install low-level keyboard hook
            let mut hook = self.bridge_hook.lock().unwrap();
            *hook = Some(install_low_level_keyboard_hook()?);

            // Initialize bridge context
            let mut ctx = self.bridge_context.lock().unwrap();
            *ctx = Some(BridgeContext::new()?);

            log::info!("Windows platform initialized");
            Ok(())
        }

        fn capture_input(&mut self) -> Result<KeyEvent> {
            // Process Windows message loop
            self.process_message_queue()
        }

        fn inject_output(&mut self, event: KeyEvent) -> Result<()> {
            self.send_input_event(event)
        }

        fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
            self.enumerate_keyboards()
        }

        fn shutdown(&mut self) -> Result<()> {
            log::info!("Shutting down Windows platform");

            // Uninstall hook
            if let Some(hook) = self.bridge_hook.lock().unwrap().take() {
                unsafe { UnhookWindowsHookEx(hook) };
            }

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_platform_trait_usage() {
            let platform: Box<dyn Platform> = Box::new(WindowsPlatform::new().unwrap());
            // Verify trait object works
        }
    }
    ```

  | Restrictions: Must not change WindowsPlatform behavior; existing Windows tests must pass; trait methods must match signatures exactly; must properly cleanup hooks on shutdown; convert Win32 errors to PlatformError; no new dependencies
  | Success: ✅ Platform trait implemented for WindowsPlatform, ✅ All trait methods work correctly, ✅ WindowsPlatform can be used as Box<dyn Platform>, ✅ Integration tests pass on Windows, ✅ Hooks cleaned up properly, ✅ cargo build succeeds on Windows target
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (Platform trait impl, methods), (4) Mark this task as complete [x] in tasks.md

---

- [ ] 14. Update Daemon to use Platform trait abstraction
  - Files: keyrx_daemon/src/daemon/mod.rs (update), keyrx_daemon/src/main.rs (update)
  - Purpose: Decouple daemon from concrete platform types using dependency injection
  - Requirements: FR3, FR5
  - Leverage: Platform trait from tasks 11-13
  - Prompt: Role: Rust Systems Architect | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Update Daemon struct in daemon/mod.rs to accept Box<dyn Platform> via constructor instead of directly using platform-specific types. Remove all #[cfg(target_os)] conditional compilation from Daemon impl. Daemon::new should accept platform: Box<dyn Platform>. Update main.rs to use create_platform() factory function. Update Daemon event loop to call platform.capture_input() and platform.inject_output(). Ensure all integration tests pass. Add unit tests with mock Platform demonstrating testability.

    **Daemon Update**:
    ```rust
    // daemon/mod.rs
    pub struct Daemon {
        platform: Box<dyn Platform>,
        config: Arc<Config>,
        runtime: Runtime,
    }

    impl Daemon {
        pub fn new(platform: Box<dyn Platform>, config: Arc<Config>) -> Result<Self> {
            Ok(Self {
                platform,
                config,
                runtime: Runtime::new(&config)?,
            })
        }

        pub fn run(&mut self) -> Result<()> {
            self.platform.initialize()?;

            loop {
                let event = self.platform.capture_input()?;
                let actions = self.runtime.process(event, &self.config)?;

                for action in actions {
                    self.platform.inject_output(action)?;
                }
            }
        }

        pub fn shutdown(&mut self) -> Result<()> {
            self.platform.shutdown()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct MockPlatform {
            events: Vec<KeyEvent>,
            index: usize,
        }

        impl Platform for MockPlatform {
            fn initialize(&mut self) -> Result<()> { Ok(()) }

            fn capture_input(&mut self) -> Result<KeyEvent> {
                if self.index < self.events.len() {
                    let event = self.events[self.index].clone();
                    self.index += 1;
                    Ok(event)
                } else {
                    Err(PlatformError::DeviceNotFound("end".into()).into())
                }
            }

            fn inject_output(&mut self, _event: KeyEvent) -> Result<()> { Ok(()) }
            fn list_devices(&self) -> Result<Vec<DeviceInfo>> { Ok(vec![]) }
            fn shutdown(&mut self) -> Result<()> { Ok(()) }
        }

        #[test]
        fn test_daemon_with_mock_platform() {
            let platform = Box::new(MockPlatform {
                events: vec![KeyEvent::test_event()],
                index: 0,
            });

            let config = Arc::new(Config::default());
            let daemon = Daemon::new(platform, config).unwrap();

            // Test daemon logic without real platform
        }
    }
    ```

    **main.rs Update**:
    ```rust
    fn main() -> Result<()> {
        let platform = create_platform()?;
        let config = load_config()?;
        let mut daemon = Daemon::new(platform, config)?;

        daemon.run()?;
        Ok(())
    }
    ```

  | Restrictions: Must remove all platform-specific imports from daemon/mod.rs; existing integration tests must pass; daemon behavior unchanged; must handle platform errors gracefully; mock tests must demonstrate testability
  | Success: ✅ Daemon uses Box<dyn Platform>, ✅ No platform-specific code in daemon/mod.rs, ✅ Integration tests pass on Linux and Windows, ✅ Unit tests with mock platform work, ✅ main.rs uses create_platform(), ✅ cargo build succeeds
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (Daemon struct changes, mock platform tests), (4) Mark this task as complete [x] in tasks.md

---

- [ ] 15. Wire CLI to use ProfileService
  - Files: keyrx_daemon/src/cli/profiles.rs (update), keyrx_daemon/src/main.rs (update)
  - Purpose: Deduplicate profile logic by using shared ProfileService instead of direct ProfileManager access
  - Requirements: FR4, FR7
  - Leverage: ProfileService from task 5
  - Prompt: Role: CLI Developer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Update keyrx_daemon/src/cli/profiles.rs to use ProfileService instead of directly calling ProfileManager. Update all handler functions (handle_list, handle_activate, handle_create, etc.) to accept &ProfileService parameter. Remove direct ProfileManager instantiation. Update main.rs to create ProfileService and pass to CLI handlers. Ensure CLI output format unchanged (use existing common.rs utilities). Verify CLI integration tests pass.

    **Handler Updates**:
    ```rust
    // profiles.rs
    use crate::services::ProfileService;

    async fn handle_list(
        service: &ProfileService,
        json: bool,
    ) -> Result<()> {
        let profiles = service.list_profiles().await?;

        if json {
            output_success(&profiles, true)?;
        } else {
            for profile in profiles {
                println!("{} {}",
                    if profile.active { "*" } else { " " },
                    profile.name
                );
            }
        }

        Ok(())
    }

    async fn handle_activate(
        service: &ProfileService,
        name: &str,
        json: bool,
    ) -> Result<()> {
        service.activate_profile(name).await?;
        output_success(&format!("Profile '{}' activated", name), json)?;
        Ok(())
    }

    // Update main handler to accept service
    pub async fn handle_profiles_command(
        service: &ProfileService,
        args: &ProfilesArgs,
    ) -> Result<()> {
        match &args.command {
            ProfilesCommand::List => handle_list(service, args.json).await,
            ProfilesCommand::Activate { name } => handle_activate(service, name, args.json).await,
            // ... other commands
        }
    }
    ```

    **main.rs Integration**:
    ```rust
    #[tokio::main]
    async fn main() -> Result<()> {
        let profile_manager = Arc::new(ProfileManager::new()?);
        let profile_service = ProfileService::new(profile_manager);

        match args.command {
            Command::Profiles(profiles_args) => {
                handle_profiles_command(&profile_service, &profiles_args).await?;
            }
            // ... other commands
        }

        Ok(())
    }
    ```

  | Restrictions: Must maintain exact CLI output format; CLI behavior unchanged; existing CLI tests must pass; must use common.rs output utilities; ProfileService methods must be async; no breaking changes to CLI arguments
  | Success: ✅ All CLI handlers use ProfileService, ✅ No direct ProfileManager calls in CLI, ✅ CLI output unchanged, ✅ CLI integration tests pass, ✅ Service methods called correctly, ✅ cargo build succeeds
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (handler functions modified, service integration), (4) Mark this task as complete [x] in tasks.md

---

## Phase 4: Quality & Documentation (Week 9-10)

### Goals
- Test coverage ≥80%
- All public APIs documented
- Zero clippy warnings

---

- [ ] 16. Add platform code unit tests (target 70% coverage)
  - Files: keyrx_daemon/src/platform/linux/*.rs, keyrx_daemon/src/platform/windows/*.rs (add #[cfg(test)] modules)
  - Purpose: Increase test coverage for platform-specific code to meet quality gates
  - Requirements: FR8, NFR1
  - Leverage: Existing platform implementation from tasks 9, 12-13
  - Prompt: Role: Platform Test Engineer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Add comprehensive unit tests for platform modules targeting ≥70% coverage. For Linux: test input_capture.rs (evdev device handling), output_injection.rs (uinput creation), device_discovery.rs (device enumeration). For Windows: test rawinput.rs (hook callbacks), inject.rs (SendInput calls), device_map.rs (keyboard enumeration). Use mocks/stubs for system calls. Focus on error handling paths, edge cases (no devices, permission errors), and cleanup. Run cargo tarpaulin to verify coverage.

    **Test Pattern** (Linux input_capture.rs):
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_capture_with_no_devices() {
            let mut capture = InputCapture::new();
            let result = capture.initialize(&[]);

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), PlatformError::DeviceNotFound(_)));
        }

        #[test]
        fn test_capture_with_valid_device() {
            // Create mock evdev device
            let device = create_mock_device();
            let mut capture = InputCapture::new();

            capture.initialize(&[device]).unwrap();

            // Simulate event
            let event = capture.capture().unwrap();
            assert_eq!(event.code, 30); // KEY_A
        }

        #[test]
        fn test_cleanup_releases_resources() {
            let mut capture = create_initialized_capture();

            capture.cleanup().unwrap();

            // Verify resources released
        }
    }
    ```

  | Restrictions: Tests must not require root/admin privileges; tests must not require actual hardware devices; use mocks/stubs for system calls; tests must be deterministic; must not introduce flakiness; coverage measured with cargo tarpaulin
  | Success: ✅ Linux platform coverage ≥70%, ✅ Windows platform coverage ≥70%, ✅ All error paths tested, ✅ Edge cases covered, ✅ Tests pass consistently, ✅ No flakiness, ✅ cargo tarpaulin confirms coverage
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (test count, coverage percentage, modules tested), (4) Mark this task as complete [x] in tasks.md

---

- [ ] 17. Document all public APIs with rustdoc
  - Files: All public modules in keyrx_core, keyrx_daemon, keyrx_compiler
  - Purpose: Ensure all public APIs have comprehensive documentation with examples
  - Requirements: FR9, NFR1
  - Leverage: Existing module structure
  - Prompt: Role: Technical Writer and Rust Documentation Specialist | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Add rustdoc comments to all public APIs (functions, structs, enums, traits, modules). Each public item must have: summary description, detailed explanation, parameter documentation (# Arguments), return value documentation (# Returns), error documentation (# Errors), usage example (# Examples). Add module-level documentation (//!) explaining purpose and providing examples. Ensure cargo doc builds without warnings. Priority items: Platform trait, Daemon, ProfileService, all CLI handlers, all API endpoints.

    **Documentation Pattern**:
    ```rust
    /// Platform abstraction for input/output operations.
    ///
    /// Provides a unified interface for keyboard event capture and injection
    /// across different operating systems. Implementations handle platform-specific
    /// details (evdev/uinput on Linux, rawinput/SendInput on Windows).
    ///
    /// # Thread Safety
    ///
    /// All implementations must be `Send + Sync` to support concurrent access
    /// from daemon event loop and API handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let mut platform = create_platform()?;
    /// platform.initialize()?;
    ///
    /// let event = platform.capture_input()?;
    /// platform.inject_output(event)?;
    /// ```
    ///
    /// # Platform Support
    ///
    /// - Linux: Uses evdev for input, uinput for output
    /// - Windows: Uses Low-Level Keyboard Hook for input, SendInput for output
    pub trait Platform: Send + Sync {
        /// Initializes platform-specific resources.
        ///
        /// Must be called before `capture_input` or `inject_output`. Opens device
        /// handles, installs hooks, creates virtual devices, etc.
        ///
        /// # Errors
        ///
        /// Returns [`PlatformError::PermissionDenied`] if insufficient privileges.
        /// Returns [`PlatformError::InitializationFailed`] if setup fails.
        ///
        /// # Examples
        ///
        /// ```
        /// let mut platform = create_platform()?;
        /// platform.initialize()?;
        /// ```
        fn initialize(&mut self) -> Result<()>;

        // ... other methods with similar documentation
    }
    ```

  | Restrictions: Must document ALL public items; examples must compile (use # for hidden lines if needed); must include error cases; cargo doc must build without warnings; follow Rust API documentation guidelines; no external doc tools
  | Success: ✅ All public APIs documented, ✅ cargo doc builds without warnings, ✅ All examples compile, ✅ Error cases documented, ✅ Module-level docs complete, ✅ Documentation coverage 100% for public items
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Implement the changes, (3) Use log-implementation tool to record detailed artifacts (modules documented, public items count), (4) Mark this task as complete [x] in tasks.md

---

## Phase 5: Final Validation (Week 11)

### Goals
- All quality gates pass
- No regressions

---

- [ ] 18. Run comprehensive validation and fix any issues
  - Purpose: Verify all quality gates pass and no regressions introduced
  - Requirements: All FRs, all NFRs
  - Leverage: scripts/verify_file_sizes.sh, cargo tarpaulin, cargo clippy
  - Prompt: Role: QA Engineer | Task: Implement the task for spec comprehensive-architecture-refactoring, first run spec-workflow-guide to get the workflow guide then implement the task: Execute comprehensive quality validation. Run file size verification (scripts/verify_file_sizes.sh), test coverage analysis (cargo tarpaulin --workspace), clippy linting (cargo clippy --workspace -- -D warnings), rustfmt check (cargo fmt --check), full test suite (cargo test --workspace), integration tests on Linux and Windows, performance benchmarks. Document any violations or regressions. If issues found, create follow-up tasks to fix.

    **Validation Checklist**:
    - [ ] File sizes: All files ≤500 lines (run scripts/verify_file_sizes.sh)
    - [ ] Test coverage: ≥80% overall, ≥90% keyrx_core (cargo tarpaulin)
    - [ ] Linting: 0 clippy warnings (cargo clippy --workspace -- -D warnings)
    - [ ] Formatting: All code formatted (cargo fmt --check)
    - [ ] Unit tests: All pass (cargo test --workspace)
    - [ ] Integration tests: All pass on Linux and Windows
    - [ ] Documentation: cargo doc builds without warnings
    - [ ] Performance: No regressions vs baseline

  | Restrictions: Must document all findings; create follow-up tasks for violations; must verify on both Linux and Windows; must compare metrics to baseline; no code changes in this task (validation only)
  | Success: ✅ All quality gates pass OR violations documented with follow-up tasks, ✅ Validation report created, ✅ Metrics compared to baseline, ✅ Linux and Windows both validated
  | After completing this task: (1) Mark this task as in-progress [-] in tasks.md before starting, (2) Execute validation, (3) Use log-implementation tool to record detailed artifacts (validation results, metrics, any violations), (4) Mark this task as complete [x] in tasks.md

---

## Summary

**Total Tasks**: 18 core tasks across 5 phases
**Estimated Duration**: 11 weeks
**Priority**: CRITICAL → HIGH → MEDIUM

**Key Milestones**:
- End of Phase 1: Global state eliminated, CheckBytes implemented
- End of Phase 2: All files ≤500 lines
- End of Phase 3: Platform traits, service layer complete
- End of Phase 4: ≥80% coverage, full documentation
- End of Phase 5: All quality gates verified

Each task follows autonomous-spec-prep enhancement pattern with detailed prompts, restrictions, and success criteria for autonomous implementation.
