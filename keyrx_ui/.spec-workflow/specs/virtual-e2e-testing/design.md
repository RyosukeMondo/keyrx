# Design Document

## Overview

The Virtual E2E Testing framework enables fully automated end-to-end testing of keyrx without physical hardware. It leverages Linux's uinput subsystem to create virtual keyboards for both input injection and output capture, exercising the complete kernel evdev/uinput path that the production daemon uses.

The architecture creates a closed loop:
1. **VirtualKeyboard** (uinput) → generates events the daemon can read via evdev
2. **keyrx_daemon** → processes events through the real platform layer
3. **OutputCapture** (evdev) → reads events from daemon's uinput output

This design validates the exact same code path used in production, unlike mock-based tests which bypass the kernel interface.

## Steering Document Alignment

### Technical Standards (tech.md)

This implementation follows keyrx architectural patterns:

- **Trait-Based Abstraction**: VirtualKeyboard and OutputCapture can be extended for other platforms
- **Zero-Copy Where Possible**: Direct evdev event reading without intermediate buffers
- **Structured Logging**: JSON format for test diagnostics
- **Fail-Safe Defaults**: Cleanup on panic, timeout, or normal exit
- **Explicit Error Handling**: All I/O operations return Result<T, E>

### Project Structure (structure.md)

File organization follows keyrx workspace conventions:

```
keyrx_daemon/
├── src/
│   ├── test_utils/              # NEW: Virtual device test utilities
│   │   ├── mod.rs               # Module exports
│   │   ├── virtual_keyboard.rs  # VirtualKeyboard implementation
│   │   └── output_capture.rs    # OutputCapture implementation
│   └── platform/
│       └── linux/               # Existing Linux platform code
├── tests/
│   ├── e2e_harness.rs           # NEW: E2E test harness
│   ├── virtual_e2e_tests.rs     # NEW: Automated E2E test cases
│   └── e2e_tests.rs             # Existing hardware-based tests
└── Cargo.toml                   # Add dev-dependencies
```

## Code Reuse Analysis

### Existing Components to Leverage

- **keyrx_daemon::platform::linux::UinputOutput**: Reference for uinput device creation
  - VirtualKeyboard uses same uinput patterns for event injection
  - Keycode mapping functions (keycode_to_evdev) reused

- **keyrx_daemon::platform::linux::EvdevInput**: Reference for evdev reading
  - OutputCapture uses same evdev patterns for event capture
  - Keycode conversion (evdev_to_keycode) reused

- **keyrx_daemon::platform::linux::keycode_map**: Keycode conversion functions
  - `keycode_to_evdev()` for VirtualKeyboard injection
  - `evdev_to_keycode()` for OutputCapture reading

- **keyrx_daemon::daemon::Daemon**: Daemon lifecycle management
  - Used by E2EHarness to start/stop daemon process
  - Signal handling (SIGTERM) for graceful shutdown

- **keyrx_core::config**: Configuration types
  - DeviceConfig for pattern matching
  - ConfigRoot for test configurations

### Integration Points

- **uinput crate**: Virtual keyboard creation (same as UinputOutput)
  - Device registration with EV_KEY capabilities
  - Event injection with SYN_REPORT

- **evdev crate**: Event capture from daemon output
  - Device enumeration by name
  - Non-blocking event reading

- **std::process::Command**: Daemon process management
  - Spawn daemon as subprocess
  - Capture stdout/stderr for diagnostics
  - Send signals for shutdown

- **tempfile crate**: Temporary configuration files
  - Create .krx files for test scenarios
  - Automatic cleanup on test completion

## Architecture

The E2E test framework creates a virtual device loop that exercises the full daemon pipeline:

```mermaid
graph TD
    subgraph TestHarness["E2E Test Harness"]
        TH[Test Code]
        VK[VirtualKeyboard<br/>uinput device]
        OC[OutputCapture<br/>evdev reader]
    end

    subgraph Kernel["Linux Kernel"]
        UI1[/dev/uinput<br/>VirtualKeyboard]
        EV1[/dev/input/eventX<br/>Test Input]
        UI2[/dev/uinput<br/>Daemon Output]
        EV2[/dev/input/eventY<br/>keyrx Virtual Keyboard]
    end

    subgraph Daemon["keyrx_daemon Process"]
        EVD[EvdevInput<br/>grab eventX]
        PROC[EventProcessor<br/>KeyLookup + State]
        UIO[UinputOutput<br/>inject to eventY]
    end

    TH -->|1. inject events| VK
    VK -->|2. write| UI1
    UI1 -->|3. creates| EV1
    EV1 -->|4. grab & read| EVD
    EVD -->|5. KeyEvent| PROC
    PROC -->|6. remap| UIO
    UIO -->|7. write| UI2
    UI2 -->|8. creates| EV2
    EV2 -->|9. read| OC
    OC -->|10. verify| TH

    style TH fill:#e1f5ff
    style VK fill:#fff3cd
    style OC fill:#d4edda
    style EVD fill:#fff3cd
    style PROC fill:#f8d7da
    style UIO fill:#d4edda
```

### Modular Design Principles

- **Single File Responsibility**:
  - `virtual_keyboard.rs`: uinput device creation and event injection only
  - `output_capture.rs`: evdev device discovery and event reading only
  - `e2e_harness.rs`: Test orchestration only (no device I/O details)
  - `virtual_e2e_tests.rs`: Test cases only (use harness API)

- **Component Isolation**:
  - VirtualKeyboard has no knowledge of OutputCapture
  - OutputCapture has no knowledge of VirtualKeyboard
  - E2EHarness coordinates both without exposing internals
  - Test cases use harness API exclusively

- **Service Layer Separation**:
  - Device layer: VirtualKeyboard, OutputCapture (low-level I/O)
  - Orchestration layer: E2EHarness (lifecycle, coordination)
  - Test layer: Test functions (assertions, scenarios)

## Components and Interfaces

### Component 1: VirtualKeyboard (keyrx_daemon/src/test_utils/virtual_keyboard.rs)

**Purpose:** Create virtual input device and inject key events for testing

**Interfaces:**
```rust
pub struct VirtualKeyboard {
    device: uinput::Device,
    name: String,
    path: PathBuf,
}

impl VirtualKeyboard {
    /// Create a virtual keyboard with the given name
    /// Name is used for device identification in tests
    pub fn create(name: &str) -> Result<Self, VirtualDeviceError>;

    /// Inject a key event into the virtual device
    /// Writes EV_KEY + EV_SYN to uinput
    pub fn inject(&mut self, event: KeyEvent) -> Result<(), VirtualDeviceError>;

    /// Inject multiple events with optional delay between them
    pub fn inject_sequence(&mut self, events: &[KeyEvent], delay: Option<Duration>)
        -> Result<(), VirtualDeviceError>;

    /// Get the device path (e.g., /dev/input/event5)
    pub fn path(&self) -> &Path;

    /// Get the device name
    pub fn name(&self) -> &str;
}

impl Drop for VirtualKeyboard {
    /// Destroy the virtual device on drop
    fn drop(&mut self);
}
```

**Dependencies:**
- `uinput` crate for virtual device creation
- `keyrx_core::runtime::event::KeyEvent` for event type
- `keyrx_daemon::platform::linux::keycode_map::keycode_to_evdev` for key conversion

**Reuses:**
- UinputOutput patterns for device creation
- keycode_to_evdev for KeyCode → evdev conversion

**Design Decisions:**
- **Unique device names**: Include timestamp/random suffix for parallel tests
- **Full key capability**: Register all EV_KEY codes like UinputOutput
- **Sync after each event**: Ensure events are processed immediately
- **Drop cleanup**: Destroy device even on panic

### Component 2: OutputCapture (keyrx_daemon/src/test_utils/output_capture.rs)

**Purpose:** Find and read events from daemon's virtual keyboard output

**Interfaces:**
```rust
pub struct OutputCapture {
    device: evdev::Device,
    name: String,
}

impl OutputCapture {
    /// Find a device by name with timeout
    /// Polls /dev/input/event* until device appears or timeout
    pub fn find_by_name(name: &str, timeout: Duration) -> Result<Self, VirtualDeviceError>;

    /// Read next event with timeout
    /// Returns None if timeout expires
    pub fn next_event(&mut self, timeout: Duration) -> Result<Option<KeyEvent>, VirtualDeviceError>;

    /// Collect all events within timeout period
    /// Stops collecting when no events for `timeout` duration
    pub fn collect_events(&mut self, timeout: Duration) -> Vec<KeyEvent>;

    /// Drain any pending events (clear buffer)
    pub fn drain(&mut self);

    /// Assert that captured events match expected sequence
    /// Panics with detailed diff on mismatch
    pub fn assert_events(captured: &[KeyEvent], expected: &[KeyEvent]);
}
```

**Dependencies:**
- `evdev` crate for device reading
- `keyrx_core::runtime::event::KeyEvent` for event type
- `keyrx_daemon::platform::linux::keycode_map::evdev_to_keycode` for key conversion

**Reuses:**
- EvdevInput patterns for device reading
- evdev_to_keycode for evdev → KeyCode conversion
- Device enumeration from device_manager

**Design Decisions:**
- **Poll-based discovery**: Device may not exist immediately after daemon starts
- **Non-blocking reads**: Use timeout to avoid hanging tests
- **Filter key events only**: Ignore EV_SYN, EV_MSC, etc.
- **Detailed assertion**: Show expected vs actual on failure

### Component 3: E2EHarness (keyrx_daemon/tests/e2e_harness.rs)

**Purpose:** Orchestrate complete E2E test lifecycle

**Interfaces:**
```rust
pub struct E2EHarness {
    virtual_input: VirtualKeyboard,
    daemon_process: Child,
    output_capture: OutputCapture,
    config_path: PathBuf,
}

pub struct E2EConfig {
    pub mappings: Vec<KeyMapping>,
    pub device_pattern: String,
}

impl E2EHarness {
    /// Setup complete E2E test environment
    /// 1. Create VirtualKeyboard
    /// 2. Write config file matching virtual keyboard
    /// 3. Start daemon process
    /// 4. Wait for daemon to grab device
    /// 5. Find and open output device
    pub fn setup(config: E2EConfig) -> Result<Self, E2EError>;

    /// Inject events into virtual keyboard
    pub fn inject(&mut self, events: &[KeyEvent]) -> Result<(), E2EError>;

    /// Capture output events with timeout
    pub fn capture(&mut self, timeout: Duration) -> Vec<KeyEvent>;

    /// Inject and immediately capture, convenience method
    pub fn inject_and_capture(&mut self, events: &[KeyEvent], timeout: Duration)
        -> Result<Vec<KeyEvent>, E2EError>;

    /// Verify captured events match expected
    pub fn verify(&self, captured: &[KeyEvent], expected: &[KeyEvent]);

    /// Graceful teardown
    /// 1. Send SIGTERM to daemon
    /// 2. Wait for daemon to exit
    /// 3. Destroy virtual keyboard
    /// 4. Remove config file
    pub fn teardown(self) -> Result<(), E2EError>;
}

impl Drop for E2EHarness {
    /// Ensure cleanup even on panic
    fn drop(&mut self);
}
```

**Dependencies:**
- `VirtualKeyboard` for input injection
- `OutputCapture` for output verification
- `std::process::Command` for daemon subprocess
- `tempfile` for temporary config files
- `keyrx_compiler` for config serialization

**Reuses:**
- serialize_config from keyrx_compiler for .krx generation
- Signal handling patterns from daemon

**Design Decisions:**
- **Subprocess daemon**: Isolates test from daemon state
- **Automatic cleanup**: Drop impl ensures no orphaned resources
- **Timeout-based verification**: Handle async nature of event propagation
- **Config generation**: Create .krx on-the-fly for test scenarios

### Component 4: VirtualDeviceError (keyrx_daemon/src/test_utils/mod.rs)

**Purpose:** Error type for virtual device operations

**Interfaces:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum VirtualDeviceError {
    #[error("Permission denied: {0}. Try: sudo chmod 666 /dev/uinput")]
    PermissionDenied(String),

    #[error("Device not found: {0}")]
    NotFound(String),

    #[error("Timeout waiting for device: {0}")]
    Timeout(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device creation failed: {0}")]
    CreationFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum E2EError {
    #[error("Virtual device error: {0}")]
    VirtualDevice(#[from] VirtualDeviceError),

    #[error("Daemon failed to start: {0}")]
    DaemonStart(String),

    #[error("Daemon exited unexpectedly: {0}")]
    DaemonCrash(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Verification failed: expected {expected:?}, got {actual:?}")]
    VerificationFailed {
        expected: Vec<KeyEvent>,
        actual: Vec<KeyEvent>,
    },
}
```

## Data Models

### E2EConfig
```rust
/// Configuration for an E2E test scenario
pub struct E2EConfig {
    /// Key mappings to test
    pub mappings: Vec<KeyMapping>,
    /// Device pattern (typically matches virtual keyboard name)
    pub device_pattern: String,
}

impl E2EConfig {
    /// Create config for simple remap test
    pub fn simple_remap(from: KeyCode, to: KeyCode) -> Self;

    /// Create config for modifier test
    pub fn modifier(trigger: KeyCode, modifier_id: u8) -> Self;

    /// Create config for conditional test
    pub fn conditional(
        modifier_trigger: KeyCode,
        modifier_id: u8,
        when_active: Vec<(KeyCode, KeyCode)>,
    ) -> Self;

    /// Convert to ConfigRoot for serialization
    pub fn to_config_root(&self, device_name: &str) -> ConfigRoot;
}
```

### TestEvent Helper
```rust
/// Helper for creating test event sequences
pub struct TestEvents;

impl TestEvents {
    /// Create press event
    pub fn press(key: KeyCode) -> KeyEvent {
        KeyEvent::Press(key)
    }

    /// Create release event
    pub fn release(key: KeyCode) -> KeyEvent {
        KeyEvent::Release(key)
    }

    /// Create press+release sequence (tap)
    pub fn tap(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(key), KeyEvent::Release(key)]
    }

    /// Create sequence of taps
    pub fn type_keys(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter().flat_map(|k| Self::tap(*k)).collect()
    }
}
```

## Error Handling

### Error Scenarios

#### 1. /dev/uinput Not Accessible
**Scenario:** Permission denied when creating VirtualKeyboard

**Handling:**
```rust
match VirtualKeyboard::create("test-keyboard") {
    Ok(vk) => vk,
    Err(VirtualDeviceError::PermissionDenied(msg)) => {
        panic!(
            "Cannot create virtual keyboard: {}\n\n\
             To fix in CI, add these steps:\n\
             - run: sudo modprobe uinput\n\
             - run: sudo chmod 666 /dev/uinput\n\n\
             To fix locally:\n\
             - sudo usermod -aG input,uinput $USER\n\
             - Log out and back in",
            msg
        );
    }
    Err(e) => panic!("Virtual keyboard creation failed: {}", e),
}
```

**User Impact:** Clear CI setup instructions

#### 2. Daemon Fails to Start
**Scenario:** Daemon process exits immediately

**Handling:**
```rust
let mut child = Command::new(daemon_path)
    .args(["run", "--config", config_path])
    .stderr(Stdio::piped())
    .spawn()?;

// Wait briefly and check if still running
thread::sleep(Duration::from_millis(500));
match child.try_wait() {
    Ok(Some(status)) => {
        let stderr = read_stderr(&mut child);
        return Err(E2EError::DaemonStart(format!(
            "Daemon exited with {}: {}",
            status, stderr
        )));
    }
    Ok(None) => { /* Still running, good */ }
    Err(e) => return Err(E2EError::DaemonStart(e.to_string())),
}
```

**User Impact:** Daemon stderr captured for debugging

#### 3. Output Device Not Found
**Scenario:** Daemon's virtual keyboard doesn't appear

**Handling:**
```rust
match OutputCapture::find_by_name("keyrx Virtual Keyboard", Duration::from_secs(5)) {
    Ok(oc) => oc,
    Err(VirtualDeviceError::Timeout(msg)) => {
        // Check if daemon is still running
        if let Ok(Some(status)) = daemon.try_wait() {
            let stderr = read_stderr(&mut daemon);
            panic!(
                "Daemon exited before creating output device.\n\
                 Exit status: {}\n\
                 Stderr: {}",
                status, stderr
            );
        }
        panic!(
            "Timeout waiting for daemon output device: {}\n\
             Available devices: {:?}",
            msg,
            list_input_devices()
        );
    }
    Err(e) => panic!("Output capture failed: {}", e),
}
```

**User Impact:** Lists available devices for debugging

#### 4. Event Verification Failure
**Scenario:** Captured events don't match expected

**Handling:**
```rust
pub fn assert_events(captured: &[KeyEvent], expected: &[KeyEvent]) {
    if captured != expected {
        let mut diff = String::new();
        diff.push_str("Event verification failed:\n\n");
        diff.push_str("Expected:\n");
        for (i, e) in expected.iter().enumerate() {
            diff.push_str(&format!("  [{}] {:?}\n", i, e));
        }
        diff.push_str("\nActual:\n");
        for (i, e) in captured.iter().enumerate() {
            let marker = if i < expected.len() && expected[i] != *e {
                " <-- MISMATCH"
            } else if i >= expected.len() {
                " <-- EXTRA"
            } else {
                ""
            };
            diff.push_str(&format!("  [{}] {:?}{}\n", i, e, marker));
        }
        if captured.len() < expected.len() {
            diff.push_str(&format!("\nMissing {} events\n", expected.len() - captured.len()));
        }
        panic!("{}", diff);
    }
}
```

**User Impact:** Detailed diff showing exactly what failed

## Testing Strategy

### Unit Testing

**Approach:** Test VirtualKeyboard and OutputCapture in isolation

**Key Components to Test:**

1. **VirtualKeyboard Creation**
   - Test device appears in /dev/input/ after creation
   - Test device has correct name
   - Test device is destroyed on drop
   - Test error on permission denied

2. **VirtualKeyboard Injection**
   - Test press event generates correct evdev event
   - Test release event generates correct evdev event
   - Test sequence injection with delays
   - Test all KeyCode values map correctly

3. **OutputCapture Discovery**
   - Test finds device by exact name
   - Test timeout when device doesn't exist
   - Test finds newly created device within timeout

4. **OutputCapture Reading**
   - Test reads press events correctly
   - Test reads release events correctly
   - Test timeout returns None when no events
   - Test collect_events gathers multiple events

5. **E2EConfig Helpers**
   - Test simple_remap creates correct config
   - Test modifier creates correct config
   - Test conditional creates correct config
   - Test to_config_root generates valid structure

### Integration Testing

**Approach:** Test components working together without daemon

**Key Flows to Test:**

1. **Virtual Device Round-Trip**
   ```rust
   #[test]
   fn test_virtual_keyboard_readable() {
       let vk = VirtualKeyboard::create("test-vk").unwrap();
       let reader = evdev::Device::open(vk.path()).unwrap();

       vk.inject(KeyEvent::Press(KeyCode::A)).unwrap();

       // Read event from device
       let event = reader.next_event().unwrap();
       assert_eq!(event.code(), evdev::Key::KEY_A.code());
       assert_eq!(event.value(), 1);
   }
   ```

2. **Config Generation and Serialization**
   ```rust
   #[test]
   fn test_e2e_config_serialization() {
       let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
       let root = config.to_config_root("test-device");

       let bytes = serialize_config(&root).unwrap();
       let loaded = deserialize_config(&bytes).unwrap();

       assert_eq!(root, loaded);
   }
   ```

### End-to-End Testing

**Approach:** Full pipeline tests with daemon subprocess

**User Scenarios:**

1. **Simple Remap (A→B)**
   ```rust
   #[test]
   fn e2e_simple_remap() {
       let harness = E2EHarness::setup(E2EConfig::simple_remap(KeyCode::A, KeyCode::B)).unwrap();

       let output = harness.inject_and_capture(
           &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
           Duration::from_millis(100),
       ).unwrap();

       harness.verify(&output, &[
           KeyEvent::Press(KeyCode::B),
           KeyEvent::Release(KeyCode::B),
       ]);
   }
   ```

2. **Modifier State**
   ```rust
   #[test]
   fn e2e_modifier_state() {
       let harness = E2EHarness::setup(E2EConfig::conditional(
           KeyCode::CapsLock, 0x00,
           vec![(KeyCode::H, KeyCode::Left)],
       )).unwrap();

       // Without modifier: H passes through
       let output1 = harness.inject_and_capture(
           &TestEvents::tap(KeyCode::H),
           Duration::from_millis(100),
       ).unwrap();
       harness.verify(&output1, &TestEvents::tap(KeyCode::H));

       // With modifier: H becomes Left
       harness.inject(&[KeyEvent::Press(KeyCode::CapsLock)]).unwrap();
       let output2 = harness.inject_and_capture(
           &TestEvents::tap(KeyCode::H),
           Duration::from_millis(100),
       ).unwrap();
       harness.verify(&output2, &TestEvents::tap(KeyCode::Left));
   }
   ```

3. **Modified Output (Shift+Key)**
   ```rust
   #[test]
   fn e2e_modified_output() {
       let config = E2EConfig {
           mappings: vec![KeyMapping::modified_output(
               KeyCode::Num2, KeyCode::Num2, true, false, false, false
           )],
           device_pattern: "*".to_string(),
       };
       let harness = E2EHarness::setup(config).unwrap();

       let output = harness.inject_and_capture(
           &TestEvents::tap(KeyCode::Num2),
           Duration::from_millis(100),
       ).unwrap();

       // Should output Shift+2 sequence
       harness.verify(&output, &[
           KeyEvent::Press(KeyCode::LShift),
           KeyEvent::Press(KeyCode::Num2),
           KeyEvent::Release(KeyCode::Num2),
           KeyEvent::Release(KeyCode::LShift),
       ]);
   }
   ```

### Performance Testing

**Benchmarks:**

1. **Event Round-Trip Latency**
   - Measure inject → capture time
   - Target: <100ms for single event

2. **Harness Setup Time**
   - Measure E2EHarness::setup duration
   - Target: <2 seconds

3. **Parallel Test Isolation**
   - Run multiple E2E tests concurrently
   - Verify no cross-contamination
