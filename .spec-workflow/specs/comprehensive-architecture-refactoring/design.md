# Design Document

## 1. Architecture Overview

### 1.1 Current Architecture Issues

The keyrx2 codebase suffers from several architectural anti-patterns:

1. **Monolithic Files**: 23 files exceed 500-line limit, with tap_hold.rs at 3614 lines
2. **Global State**: 6 static variables with interior mutability prevent testing and create thread-safety concerns
3. **SOLID Violations**: Modules have multiple responsibilities, hard-coded dispatch prevents extension
4. **Missing Abstraction**: No service layer between transport (CLI/Web) and business logic
5. **Tight Coupling**: Platform code directly imported rather than abstracted via traits

### 1.2 Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Transport Layer                          │
│  ┌──────────────┐              ┌──────────────┐            │
│  │   CLI        │              │   Web API    │            │
│  │  (commands)  │              │  (endpoints) │            │
│  └──────┬───────┘              └──────┬───────┘            │
│         │                              │                     │
└─────────┼──────────────────────────────┼─────────────────────┘
          │                              │
          └──────────┬───────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    Service Layer (NEW)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ProfileService│  │ConfigService │  │DeviceService │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    Domain Layer                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ProfileManager│  │ConfigManager │  │DeviceManager │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    Platform Layer (Abstracted)               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │             Platform Trait (NEW)                      │   │
│  ├──────────────────────────────────────────────────────┤   │
│  │  trait InputCapture { fn capture() -> KeyEvent }     │   │
│  │  trait OutputInjection { fn inject(KeyEvent) }       │   │
│  │  trait SystemTray { fn show_menu(...) }             │   │
│  └───────────────────┬──────────────────────────────────┘   │
│                      │                                       │
│       ┌──────────────┴──────────────┐                       │
│       ▼                              ▼                       │
│  ┌─────────────┐              ┌─────────────┐               │
│  │   Linux     │              │   Windows   │               │
│  │ (evdev,     │              │ (rawinput,  │               │
│  │  uinput)    │              │  SendInput) │               │
│  └─────────────┘              └─────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 Key Design Decisions

**Decision 1: Service Layer Pattern**
- **Rationale**: CLI and Web API duplicate business logic; service layer provides single source of truth
- **Trade-offs**: Adds one layer of indirection but dramatically improves testability
- **Alternatives Considered**: Keep duplication, use macro for code sharing (rejected as too brittle)

**Decision 2: Trait-Based Platform Abstraction**
- **Rationale**: Current platform code is tightly coupled; traits enable testing and future platform support
- **Trade-offs**: Slight runtime cost (dynamic dispatch) but worth it for testability
- **Alternatives Considered**: Conditional compilation only (rejected as untestable)

**Decision 3: Dependency Injection via Constructor**
- **Rationale**: Constructor injection is explicit, compile-time checked, and testable
- **Trade-offs**: More verbose than globals but dramatically improves testability
- **Alternatives Considered**: Service locator (rejected as implicit dependencies)

**Decision 4: File Size as Quality Gate**
- **Rationale**: Files >500 lines violate SRP; enforcing limit forces better design
- **Trade-offs**: Requires more files but each file has clear purpose
- **Alternatives Considered**: Higher limit like 1000 lines (rejected as still too large)

## 2. Module Design

### 2.1 File Size Refactoring Plans

#### Priority 1: CRITICAL Files (>1000 lines)

**File:** `keyrx_core/src/runtime/tap_hold.rs` (3614 lines → <500 lines)

**Extraction Plan:**
```
tap_hold/
├── mod.rs                    (~150 lines) - Public API, re-exports
├── state_machine.rs          (~800 lines) - Core state transitions
├── event_processor.rs        (~600 lines) - Event processing logic
├── timeout_handler.rs        (~300 lines) - Timeout management
├── testing/
│   ├── mod.rs               (~100 lines)
│   ├── scenarios.rs         (~800 lines) - Test scenarios
│   └── assertions.rs        (~400 lines) - Test assertions
└── types.rs                 (~400 lines) - TapHoldState, events
```

**Strategy:**
1. Extract test code to `testing/` submodule first (no logic changes)
2. Extract types to separate module
3. Split state machine into focused processor modules
4. Update mod.rs with re-exports to maintain public API

---

**File:** `keyrx_daemon/tests/e2e_harness.rs` (3523 lines → <500 lines)

**Extraction Plan:**
```
test_utils/e2e/
├── mod.rs                   (~100 lines) - Re-exports
├── harness_base.rs          (~400 lines) - Base harness trait
├── harness_linux.rs         (~800 lines) - Linux-specific harness
├── harness_windows.rs       (~800 lines) - Windows-specific harness
├── device_simulation.rs     (~500 lines) - Virtual device creation
├── event_injection.rs       (~400 lines) - Event injection utilities
└── assertions.rs            (~400 lines) - Assertion helpers
```

**Strategy:**
1. Extract platform-specific code to separate modules
2. Create trait for common harness interface
3. Move utilities to focused modules

---

**File:** `keyrx_compiler/tests/parser_function_tests.rs` (2864 lines → <500 lines)

**Extraction Plan:**
```
parser_tests/
├── mod.rs                   (~100 lines) - Shared test utilities
├── maps_tests.rs            (~600 lines) - Map function tests
├── taps_tests.rs            (~500 lines) - Tap/hold function tests
├── modifiers_tests.rs       (~500 lines) - Modifier tests
├── macros_tests.rs          (~500 lines) - Macro tests
├── layers_tests.rs          (~400 lines) - Layer tests
└── validation_tests.rs      (~200 lines) - Validation tests
```

**Strategy:**
1. Group tests by feature category
2. Extract shared test fixtures to mod.rs
3. Each test module focuses on one parser feature

---

**File:** `keyrx_daemon/src/platform/linux/mod.rs` (1952 lines → <500 lines)

**Extraction Plan:**
```
platform/linux/
├── mod.rs                   (~200 lines) - Platform trait impl, coordination
├── input_capture.rs         (~600 lines) - evdev device handling
├── output_injection.rs      (~500 lines) - uinput output
├── device_discovery.rs      (~400 lines) - Device enumeration
├── tray.rs                  (~200 lines) - System tray integration (existing)
└── keycode_map.rs           (~872 lines) - Keep as-is (data-driven)
```

**Strategy:**
1. Extract input/output responsibilities to focused modules
2. Keep mod.rs as orchestration layer
3. Implement Platform trait in mod.rs, delegate to submodules

---

#### Priority 2: HIGH Files (750-1000 lines)

**File:** `keyrx_daemon/src/web/api.rs` (1206 lines → <500 lines)

**Extraction Plan:**
```
web/
├── mod.rs                   (~100 lines) - Server setup, router config
├── api/
│   ├── mod.rs              (~50 lines) - Re-exports
│   ├── devices.rs          (~200 lines) - Device endpoints
│   ├── profiles.rs         (~200 lines) - Profile endpoints
│   ├── config.rs           (~200 lines) - Config endpoints
│   ├── macros.rs           (~150 lines) - Macro recorder endpoints
│   └── metrics.rs          (~100 lines) - Metrics/health endpoints
├── error.rs                (~150 lines) - ApiError type, conversions
└── middleware.rs           (~150 lines) - Logging, CORS, etc.
```

**Strategy:**
1. Extract error types first (other modules depend on it)
2. Split endpoints by domain (devices, profiles, config, macros)
3. Extract middleware to separate module
4. Keep mod.rs minimal (routing configuration only)

---

**File:** `keyrx_daemon/src/cli/config.rs` (914 lines → <500 lines)

**Extraction Plan:**
```
cli/
├── config/
│   ├── mod.rs              (~150 lines) - Command dispatch
│   ├── map_handlers.rs     (~200 lines) - map_key, map_macro, etc.
│   ├── tap_handlers.rs     (~200 lines) - set_tap_hold, etc.
│   ├── layer_handlers.rs   (~200 lines) - Layer commands
│   └── validation.rs       (~150 lines) - Config validation
└── common.rs               (existing)   - Output utilities
```

**Strategy:**
1. Group handlers by category (maps, taps, layers)
2. Extract validation logic to separate module
3. Use common.rs for all output formatting

---

### 2.2 Global State Elimination

#### Problem 1: MACRO_RECORDER Singleton

**Current Code** (web/api.rs:16):
```rust
static MACRO_RECORDER: OnceLock<MacroRecorder> = OnceLock::new();

fn get_macro_recorder() -> &'static MacroRecorder {
    MACRO_RECORDER.get_or_init(|| MacroRecorder::new())
}
```

**Issues:**
- Not injectable for testing
- Difficult to mock
- Hidden dependency

**Solution: Dependency Injection via AppState**
```rust
// web/mod.rs
pub struct AppState {
    macro_recorder: Arc<MacroRecorder>,
    device_manager: Arc<dyn DeviceManager>,
    config_service: Arc<ConfigService>,
}

// web/api/macros.rs
async fn start_recording(
    State(state): State<Arc<AppState>>
) -> Result<Json<RecordingState>, ApiError> {
    state.macro_recorder.start_recording().await?;
    Ok(Json(RecordingState { recording: true }))
}

// Testing
#[cfg(test)]
mod tests {
    use super::*;

    struct MockMacroRecorder;
    impl MacroRecorder for MockMacroRecorder { /* ... */ }

    #[test]
    fn test_start_recording() {
        let state = Arc::new(AppState {
            macro_recorder: Arc::new(MockMacroRecorder),
            // ... other mocks
        });
        // Test with mock
    }
}
```

**Migration Steps:**
1. Create AppState struct with Arc<MacroRecorder>
2. Pass AppState to router via `with_state()`
3. Update all endpoints to extract State
4. Remove static MACRO_RECORDER
5. Update tests to inject mock

---

#### Problem 2: Windows Global State

**Current Code** (platform/windows/rawinput.rs):
```rust
static BRIDGE_CONTEXT: RwLock<Option<BridgeContext>> = RwLock::new(None);
static BRIDGE_HOOK: RwLock<Option<isize>> = RwLock::new(None);
```

**Issues:**
- Thread synchronization overhead on every access
- Not testable
- Prevents multiple instances

**Solution: Move to WindowsPlatform Struct**
```rust
pub struct WindowsPlatform {
    bridge_context: Arc<Mutex<Option<BridgeContext>>>,
    bridge_hook: Arc<Mutex<Option<isize>>>,
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
        *hook = Some(install_hook()?);

        Ok(())
    }
}

// Testing
#[cfg(test)]
mod tests {
    #[test]
    fn test_platform_initialization() {
        let platform = WindowsPlatform::new().unwrap();
        platform.initialize().unwrap();
        // Assertions
    }
}
```

**Migration Steps:**
1. Create WindowsPlatform struct
2. Move static variables to struct fields
3. Update all access sites to use `self.bridge_context`
4. Remove static variables
5. Add unit tests

---

#### Problem 3: Test Utility Global State

**Current Code** (test_utils/output_capture.rs):
```rust
static SENDER: RwLock<Option<Sender<KeyEvent>>> = RwLock::new(None);

pub fn initialize_output_capture() -> Receiver<KeyEvent> {
    let (tx, rx) = crossbeam_channel::unbounded();
    *SENDER.write().unwrap() = Some(tx);
    rx
}
```

**Solution: Return Capture Handle**
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
fn test_with_output_capture() {
    let (capture, receiver) = OutputCapture::new();

    // Test code
    capture.inject_event(KeyEvent { /* ... */ });

    let event = receiver.recv().unwrap();
    assert_eq!(event, /* expected */);
}
```

**Migration Steps:**
1. Create OutputCapture struct
2. Update tests to use returned handle
3. Remove static SENDER
4. Update all test code

---

### 2.3 Service Layer Design

**Purpose**: Provide single source of truth for business logic, shared between CLI and Web API

#### ProfileService

```rust
pub struct ProfileService {
    profile_manager: Arc<ProfileManager>,
    config_manager: Arc<ConfigManager>,
}

impl ProfileService {
    pub fn new(
        profile_manager: Arc<ProfileManager>,
        config_manager: Arc<ConfigManager>,
    ) -> Self {
        Self { profile_manager, config_manager }
    }

    pub async fn list_profiles(&self) -> Result<Vec<ProfileInfo>> {
        self.profile_manager.list_profiles()
    }

    pub async fn activate_profile(&self, name: &str) -> Result<()> {
        let profile = self.profile_manager.load_profile(name)?;
        self.config_manager.apply_config(&profile.config)?;
        self.profile_manager.set_active(name)?;
        Ok(())
    }

    pub async fn create_profile(&self, name: &str) -> Result<ProfileInfo> {
        self.profile_manager.create_profile(name)
    }

    // ... more methods
}
```

#### CLI Integration

```rust
// cli/profiles.rs
async fn handle_activate_profile(
    service: &ProfileService,
    name: &str,
    json: bool,
) -> Result<()> {
    service.activate_profile(name).await?;
    output_success(&format!("Profile '{}' activated", name), json)?;
    Ok(())
}
```

#### Web API Integration

```rust
// web/api/profiles.rs
async fn activate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ActivationResult>, ApiError> {
    state.profile_service.activate_profile(&name).await?;
    Ok(Json(ActivationResult { success: true }))
}
```

**Benefits:**
- CLI and Web API share same logic
- Service is testable in isolation
- Business logic not duplicated

---

### 2.4 Platform Abstraction via Traits

**Current Problem:**
```rust
// Tight coupling to concrete types
#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxPlatform;

#[cfg(target_os = "windows")]
use crate::platform::windows::WindowsPlatform;
```

**Solution: Trait Abstraction**

```rust
// platform/mod.rs
pub trait Platform: Send + Sync {
    /// Initialize the platform layer
    fn initialize(&mut self) -> Result<()>;

    /// Capture next input event (blocking)
    fn capture_input(&mut self) -> Result<KeyEvent>;

    /// Inject output event
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;

    /// Get list of available devices
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;

    /// Cleanup and shutdown
    fn shutdown(&mut self) -> Result<()>;
}

// Linux implementation
pub struct LinuxPlatform {
    input_capture: Box<dyn InputCapture>,
    output_injection: Box<dyn OutputInjection>,
}

impl Platform for LinuxPlatform {
    fn initialize(&mut self) -> Result<()> {
        self.input_capture.initialize()?;
        self.output_injection.initialize()?;
        Ok(())
    }

    fn capture_input(&mut self) -> Result<KeyEvent> {
        self.input_capture.capture()
    }

    fn inject_output(&mut self, event: KeyEvent) -> Result<()> {
        self.output_injection.inject(event)
    }

    // ... implement other methods
}

// Factory function
pub fn create_platform() -> Result<Box<dyn Platform>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(LinuxPlatform::new()?))
    }

    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsPlatform::new()?))
    }
}

// Daemon uses trait
pub struct Daemon {
    platform: Box<dyn Platform>,
    config: Config,
}

impl Daemon {
    pub fn new(platform: Box<dyn Platform>, config: Config) -> Self {
        Self { platform, config }
    }

    pub fn run(&mut self) -> Result<()> {
        self.platform.initialize()?;

        loop {
            let event = self.platform.capture_input()?;
            let action = self.process_event(event)?;
            self.platform.inject_output(action)?;
        }
    }
}

// Testing
#[cfg(test)]
mod tests {
    struct MockPlatform;

    impl Platform for MockPlatform {
        fn capture_input(&mut self) -> Result<KeyEvent> {
            Ok(KeyEvent::test_event())
        }

        fn inject_output(&mut self, _event: KeyEvent) -> Result<()> {
            Ok(())
        }

        // ... implement other methods
    }

    #[test]
    fn test_daemon_with_mock_platform() {
        let platform = Box::new(MockPlatform);
        let daemon = Daemon::new(platform, Config::default());
        // Test daemon logic without real platform dependencies
    }
}
```

**Benefits:**
- Daemon testable without platform dependencies
- Easy to add new platforms (macOS, BSD)
- Platform-specific code isolated
- Runtime platform selection possible

---

## 3. Error Handling Strategy

### 3.1 Error Type Hierarchy

```rust
// errors/mod.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyrxError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("IPC error: {0}")]
    Ipc(#[from] IpcError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
}

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Platform not supported")]
    Unsupported,
}

// ... more error types
```

### 3.2 Error Propagation Pattern

**Replace unwrap/expect with ?:**

```rust
// Before (production code)
let config = load_config().expect("Failed to load config");

// After
let config = load_config()
    .context("Failed to load configuration from default path")?;
```

**Use map_err for context:**

```rust
// Before
self.events.lock().map_err(|e| format!("Lock failed: {}", e))?;

// After
self.events.lock()
    .map_err(|_| KeyrxError::Platform(PlatformError::LockPoisoned))?;
```

### 3.3 Error Handling in Tests

**Test code can use unwrap/expect:**
```rust
#[test]
fn test_profile_activation() {
    let service = ProfileService::new(/* ... */);
    let result = service.activate_profile("test").unwrap();
    assert_eq!(result.active, true);
}
```

**But production code cannot:**
```rust
pub fn activate_profile(&self, name: &str) -> Result<()> {
    let profile = self.load_profile(name)?;  // ✅ Use ?
    // NOT: let profile = self.load_profile(name).unwrap();  // ❌
    Ok(())
}
```

---

## 4. Testing Strategy

### 4.1 Unit Test Coverage Targets

| Component | Target | Strategy |
|-----------|--------|----------|
| keyrx_core | ≥90% | Critical path, all public APIs tested |
| keyrx_daemon | ≥80% | Service layer 100%, platform code ≥70% |
| keyrx_compiler | ≥80% | Parser logic 100%, error handling ≥70% |

### 4.2 Test Organization

**Unit Tests** (in same file as implementation):
```rust
// profile_service.rs
pub struct ProfileService { /* ... */ }

impl ProfileService { /* ... */ }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activate_profile_success() {
        let service = create_test_service();
        let result = service.activate_profile("test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_activate_profile_not_found() {
        let service = create_test_service();
        let result = service.activate_profile("nonexistent");
        assert!(matches!(result, Err(ConfigError::ProfileNotFound(_))));
    }
}
```

**Integration Tests** (in tests/ directory):
```rust
// tests/profile_service_integration.rs
use keyrx_daemon::services::ProfileService;

#[test]
fn test_profile_lifecycle() {
    // Create, activate, delete profile
}
```

### 4.3 Mock Strategy

**Use traits for mockability:**
```rust
#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockPlatform {
        pub events: Vec<KeyEvent>,
        pub event_index: usize,
    }

    impl Platform for MockPlatform {
        fn capture_input(&mut self) -> Result<KeyEvent> {
            if self.event_index < self.events.len() {
                let event = self.events[self.event_index].clone();
                self.event_index += 1;
                Ok(event)
            } else {
                Err(PlatformError::DeviceNotFound("Mock device".into()).into())
            }
        }

        fn inject_output(&mut self, _event: KeyEvent) -> Result<()> {
            Ok(())
        }
    }
}
```

### 4.4 CheckBytes Fuzzing

```rust
// fuzz/fuzz_targets/fuzz_deserialize_with_checkbytes.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use keyrx_core::runtime::CompiledConfig;
use rkyv::CheckBytes;

fuzz_target!(|data: &[u8]| {
    // Should not panic on malformed input
    let _ = rkyv::check_archived_root::<CompiledConfig>(data);
});
```

**Run fuzzing:**
```bash
cargo fuzz run fuzz_deserialize_with_checkbytes -- -max_total_time=3600
```

---

## 5. Migration Strategy

### 5.1 Phase Breakdown

#### Phase 1: Critical Foundation (Week 1-2)
**Goals:**
- Eliminate global state blockers
- Implement CheckBytes for security
- Create service layer base

**Tasks:**
1. Implement CheckBytes for all rkyv types
2. Remove MACRO_RECORDER global (inject via AppState)
3. Remove Windows BRIDGE_CONTEXT/BRIDGE_HOOK (move to struct)
4. Remove test utility global SENDER (return capture handle)
5. Create ProfileService, ConfigService, DeviceService base

#### Phase 2: File Size Compliance (Week 3-5)
**Goals:**
- All files ≤500 lines
- Module boundaries clear
- Tests split logically

**Tasks:**
6. Split tap_hold.rs (3614 → <500)
7. Split e2e_harness.rs (3523 → <500)
8. Split parser_function_tests.rs (2864 → <500)
9. Split linux/mod.rs (1952 → <500)
10. Split web/api.rs (1206 → <500)
11. Split daemon/mod.rs (1591 → <500)
12. Split remaining 17 files in 500-1000 range

#### Phase 3: Architecture Refactoring (Week 6-8)
**Goals:**
- Platform abstraction via traits
- Service layer fully functional
- SOLID principles enforced

**Tasks:**
13. Create Platform trait
14. Implement LinuxPlatform using trait
15. Implement WindowsPlatform using trait
16. Update Daemon to use trait abstraction
17. Wire CLI to use services
18. Wire Web API to use services
19. Extract error types to hierarchy

#### Phase 4: Quality & Documentation (Week 9-10)
**Goals:**
- Test coverage ≥80%
- All public APIs documented
- Zero clippy warnings

**Tasks:**
20. Add unit tests for platform code (target 70%)
21. Add service layer tests (target 100%)
22. Add integration tests for E2E scenarios
23. Document all public APIs with rustdoc
24. Add architecture decision records (ADRs)
25. Run cargo doc, fix all warnings

#### Phase 5: Validation (Week 11)
**Goals:**
- All quality gates pass
- No regressions

**Tasks:**
26. Run file size verification
27. Run test coverage analysis
28. Run clippy with deny warnings
29. Run integration test suite
30. Performance regression testing

### 5.2 Testing Protocol

**Before each task:**
1. Run full test suite: `cargo test --workspace`
2. Ensure all tests pass
3. Record baseline metrics (coverage, build time)

**After each task:**
1. Run full test suite again
2. Ensure no new test failures
3. Run clippy: `cargo clippy --workspace -- -D warnings`
4. Run fmt check: `cargo fmt --check`
5. Verify metrics unchanged or improved

### 5.3 Rollback Strategy

**If a task introduces regressions:**
1. Revert the commit
2. Analyze what went wrong
3. Create smaller subtasks
4. Re-attempt with better tests

**Quality gates (must pass before merge):**
- All tests pass
- Clippy 0 warnings
- Coverage ≥baseline (no decrease)
- Build time ≤baseline + 5%

---

## 6. Code Quality Gates

### 6.1 Pre-Commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run clippy
cargo clippy --workspace -- -D warnings || exit 1

# Run rustfmt check
cargo fmt --check || exit 1

# Run tests
cargo test --workspace || exit 1

# Check file sizes
scripts/verify_file_sizes.sh || exit 1
```

### 6.2 CI Pipeline

```yaml
# .github/workflows/ci.yml
quality-gates:
  - name: Clippy
    run: cargo clippy --workspace -- -D warnings

  - name: Rustfmt
    run: cargo fmt --check

  - name: Tests
    run: cargo test --workspace

  - name: Coverage
    run: |
      cargo tarpaulin --workspace --out Xml
      # Fail if coverage < 80%

  - name: File Size Compliance
    run: scripts/verify_file_sizes.sh
```

### 6.3 File Size Verification Script

```bash
#!/bin/bash
# scripts/verify_file_sizes.sh

MAX_LINES=500

# Find all .rs files, count lines excluding comments/blanks
find keyrx_core keyrx_daemon keyrx_compiler -name "*.rs" | while read file; do
    lines=$(tokei "$file" | awk '/Rust/ {print $5}')
    if [ "$lines" -gt "$MAX_LINES" ]; then
        echo "❌ $file: $lines lines (max $MAX_LINES)"
        exit 1
    fi
done

echo "✅ All files comply with $MAX_LINES line limit"
```

---

## 7. Dependency Management

### 7.1 Existing Dependencies (No Changes)

**Core dependencies:**
- rkyv 0.7 - Binary serialization (keep)
- serde 1.0 - JSON serialization (keep)
- thiserror 1.0 - Error handling (keep)

**Platform dependencies:**
- evdev 0.12 (Linux) - Keep
- windows-sys 0.48 (Windows) - Keep

**Web dependencies:**
- axum 0.6 - Keep
- tokio 1.0 - Keep

### 7.2 Optional New Dependencies

**Consider adding (if helpful):**
- anyhow - For application errors (alternative to custom error types)
- tracing - Structured logging (better than log crate)

**Not adding:**
- No new UI frameworks
- No new serialization formats
- No additional web frameworks

---

## 8. Performance Considerations

### 8.1 Expected Performance Impact

**File splits:** No impact (compile-time only)

**Trait dispatch:** <1% overhead on event processing (acceptable)

**Arc wrapping:** Minimal memory overhead (~16 bytes per Arc)

### 8.2 Performance Benchmarks

```rust
// benches/platform_dispatch.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_direct_call(c: &mut Criterion) {
    let platform = LinuxPlatform::new().unwrap();
    c.bench_function("direct_call", |b| {
        b.iter(|| platform.capture_input())
    });
}

fn bench_trait_dispatch(c: &mut Criterion) {
    let platform: Box<dyn Platform> = Box::new(LinuxPlatform::new().unwrap());
    c.bench_function("trait_dispatch", |b| {
        b.iter(|| platform.capture_input())
    });
}

criterion_group!(benches, bench_direct_call, bench_trait_dispatch);
criterion_main!(benches);
```

**Acceptance:** Trait dispatch ≤5% slower than direct call

---

## 9. Documentation Standards

### 9.1 Module-Level Documentation

```rust
//! Platform abstraction layer.
//!
//! This module provides trait-based abstractions for platform-specific
//! input/output operations, enabling cross-platform support and testability.
//!
//! # Architecture
//!
//! ```text
//! Platform Trait (abstract)
//!     ↓
//!  ┌──────────────┐
//!  ↓              ↓
//! LinuxPlatform  WindowsPlatform
//! ```
//!
//! # Examples
//!
//! ```
//! use keyrx_daemon::platform::create_platform;
//!
//! let mut platform = create_platform()?;
//! platform.initialize()?;
//! let event = platform.capture_input()?;
//! ```
```

### 9.2 Function Documentation

```rust
/// Activates a profile by name.
///
/// This method loads the profile configuration, applies it to the
/// daemon, and updates the active profile marker.
///
/// # Arguments
///
/// * `name` - The profile name to activate
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Profile does not exist
/// - Configuration is invalid
/// - Daemon cannot apply config
///
/// # Examples
///
/// ```
/// # use keyrx_daemon::services::ProfileService;
/// let service = ProfileService::new(/* ... */);
/// service.activate_profile("gaming")?;
/// ```
///
/// # Errors
///
/// Returns [`ConfigError::ProfileNotFound`] if the profile does not exist.
pub async fn activate_profile(&self, name: &str) -> Result<()> {
    // ...
}
```

---

## 10. Risk Mitigation

### 10.1 High-Risk Areas

**Risk:** Breaking platform-specific code during refactoring
**Mitigation:**
- Extensive integration tests before changes
- Platform-specific CI runners (Linux, Windows)
- Gradual migration with feature flags

**Risk:** Performance regression from trait dispatch
**Mitigation:**
- Benchmark before/after
- Profile with perf/flamegraph
- Acceptance criteria: <5% overhead

**Risk:** Test coverage reveals existing bugs
**Mitigation:**
- Fix bugs as separate tasks
- Prioritize by severity
- Don't block refactoring on bug fixes

### 10.2 Rollback Plan

**If major issues arise:**
1. Revert to last known good commit
2. Analyze root cause
3. Create isolated reproducer
4. Fix in isolation
5. Re-attempt with additional tests

---

This design provides a clear path from current technical debt to clean, testable, maintainable architecture following SOLID principles and best practices.
