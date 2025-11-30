# Design Document: platform-drivers

## Overview

This design implements real keyboard capture and injection for Linux (evdev/uinput) and Windows (WH_KEYBOARD_LL/SendInput). The drivers replace the current stubs while maintaining the existing `InputSource` trait interface, enabling seamless integration with the Engine.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Trait Abstraction**: Drivers implement `InputSource` trait
- **Tokio Async**: Driver threads communicate via async channels
- **Platform Drivers**: Compile-time selection via `#[cfg(target_os)]`
- **No Global State**: All driver state in struct instances

### Project Structure (structure.md)
- `core/src/drivers/linux.rs` - Linux evdev/uinput implementation
- `core/src/drivers/windows.rs` - Windows hook implementation
- `core/src/drivers/common.rs` - Shared utilities (device listing)

## Code Reuse Analysis

### Existing Components to Leverage
- **`InputSource` trait** (`traits/input_source.rs`): Already defined with poll_events, send_output, start, stop
- **`InputEvent`, `OutputAction`** (`engine/types.rs`): Event types already defined
- **`KeyCode`** (`engine/types.rs`): Key mapping with evdev/VK conversion stubs already present
- **`evdev_to_keycode`** (`drivers/linux.rs:156-269`): Mapping already implemented
- **`vk_to_keycode`** (`drivers/windows.rs:104-235`): Mapping already implemented
- **`keycode_to_vk`** (`drivers/windows.rs:240-367`): Reverse mapping exists

### Integration Points
- **Engine**: Uses `InputSource` trait, no changes needed
- **CLI run command**: Already creates driver based on platform
- **Benchmarks**: Already measure event processing latency

## Architecture

```mermaid
graph TD
    subgraph "Linux Driver"
        A[Physical Keyboard] -->|evdev| B[/dev/input/eventX]
        B -->|EVIOCGRAB| C[LinuxInput]
        C -->|channel| D[Async Event Queue]
        D --> E[Engine]
        E --> F[uinput Writer]
        F -->|/dev/uinput| G[Virtual Keyboard]
        G --> H[Applications]
    end

    subgraph "Windows Driver"
        I[Physical Keyboard] --> J[WH_KEYBOARD_LL]
        J -->|callback| K[Hook Thread]
        K -->|channel| L[Async Event Queue]
        L --> M[Engine]
        M --> N[SendInput]
        N --> O[Applications]
    end
```

### Thread Model

```
Linux:
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   evdev Reader  │────▶│  Async Channel  │────▶│     Engine      │
│   (blocking)    │     │   (bounded)     │     │   (async)       │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │  uinput Writer  │
                                                │   (blocking)    │
                                                └─────────────────┘

Windows:
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Message Pump   │────▶│  Async Channel  │────▶│     Engine      │
│  + Hook Callback│     │   (bounded)     │     │   (async)       │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │    SendInput    │
                                                │   (same thread) │
                                                └─────────────────┘
```

## Components and Interfaces

### Component 1: Linux evdev Reader
- **Purpose**: Capture keyboard events from /dev/input/eventX
- **Files**: `core/src/drivers/linux.rs`
- **Thread**: Dedicated blocking thread (evdev read is blocking)
- **Interface**:
  ```rust
  struct EvdevReader {
      device: evdev::Device,
      tx: Sender<InputEvent>,
      running: Arc<AtomicBool>,
  }

  impl EvdevReader {
      fn spawn(device_path: &Path, tx: Sender<InputEvent>) -> JoinHandle<()>;
      fn grab(&mut self) -> Result<()>;  // EVIOCGRAB
      fn ungrab(&mut self) -> Result<()>;
  }
  ```
- **Dependencies**: evdev crate, crossbeam-channel

### Component 2: Linux uinput Writer
- **Purpose**: Inject remapped keys via virtual keyboard
- **Files**: `core/src/drivers/linux.rs`
- **Interface**:
  ```rust
  struct UinputWriter {
      device: evdev::UInputDevice,
  }

  impl UinputWriter {
      fn new(name: &str) -> Result<Self>;
      fn emit(&mut self, key: KeyCode, pressed: bool) -> Result<()>;
      fn sync(&mut self) -> Result<()>;
  }
  ```
- **Dependencies**: evdev crate (uinput feature)

### Component 3: LinuxInput (InputSource)
- **Purpose**: Coordinate evdev reader and uinput writer
- **Files**: `core/src/drivers/linux.rs`
- **Interface**:
  ```rust
  pub struct LinuxInput {
      reader_handle: Option<JoinHandle<()>>,
      writer: UinputWriter,
      rx: Receiver<InputEvent>,
      running: Arc<AtomicBool>,
      device_path: PathBuf,
  }

  impl LinuxInput {
      pub fn new(device_path: Option<PathBuf>) -> Result<Self>;
      pub fn list_devices() -> Result<Vec<DeviceInfo>>;
  }

  impl InputSource for LinuxInput { ... }
  ```

### Component 4: Windows Hook Manager
- **Purpose**: Install and manage WH_KEYBOARD_LL hook
- **Files**: `core/src/drivers/windows.rs`
- **Thread**: Message pump thread required for hook callbacks
- **Interface**:
  ```rust
  struct HookManager {
      hook_handle: HHOOK,
      tx: Sender<InputEvent>,
      running: Arc<AtomicBool>,
  }

  impl HookManager {
      fn install(tx: Sender<InputEvent>) -> Result<Self>;
      fn uninstall(&mut self) -> Result<()>;
      fn run_message_loop(&self);  // Blocking
  }
  ```
- **Dependencies**: windows-rs crate

### Component 5: Windows SendInput Injector
- **Purpose**: Inject remapped keys via SendInput API
- **Files**: `core/src/drivers/windows.rs`
- **Interface**:
  ```rust
  struct SendInputInjector;

  impl SendInputInjector {
      fn inject_key(key: KeyCode, pressed: bool) -> Result<()>;
      fn inject_sequence(keys: &[(KeyCode, bool)]) -> Result<()>;
  }
  ```
- **Dependencies**: windows-rs crate

### Component 6: WindowsInput (InputSource)
- **Purpose**: Coordinate hook and injection
- **Files**: `core/src/drivers/windows.rs`
- **Interface**:
  ```rust
  pub struct WindowsInput {
      hook_thread: Option<JoinHandle<()>>,
      rx: Receiver<InputEvent>,
      running: Arc<AtomicBool>,
  }

  impl WindowsInput {
      pub fn new() -> Result<Self>;
  }

  impl InputSource for WindowsInput { ... }
  ```

### Component 7: Device Listing
- **Purpose**: List available keyboard devices
- **Files**: `core/src/drivers/common.rs`, CLI command
- **Interface**:
  ```rust
  pub struct DeviceInfo {
      pub path: PathBuf,      // Linux: /dev/input/event3
      pub name: String,       // "AT Translated Set 2 keyboard"
      pub vendor_id: u16,
      pub product_id: u16,
      pub is_keyboard: bool,
  }

  pub fn list_keyboards() -> Result<Vec<DeviceInfo>>;
  ```

## Data Models

### DeviceInfo
```rust
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    /// Device path (Linux: /dev/input/eventX, Windows: device instance ID)
    pub path: PathBuf,
    /// Human-readable device name
    pub name: String,
    /// USB Vendor ID
    pub vendor_id: u16,
    /// USB Product ID
    pub product_id: u16,
    /// Whether this is a keyboard device
    pub is_keyboard: bool,
}
```

### LatencyStats
```rust
#[derive(Debug, Clone, Serialize)]
pub struct LatencyStats {
    pub capture_us: u64,    // Time from hardware to driver
    pub process_us: u64,    // Time in script runtime
    pub inject_us: u64,     // Time to output key
    pub total_us: u64,      // End-to-end latency
    pub sample_count: usize,
}
```

## Error Handling

### Linux-Specific Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum LinuxDriverError {
    #[error("Device not found: {path}")]
    DeviceNotFound { path: PathBuf },

    #[error("Permission denied accessing {path}. Run: sudo usermod -aG input $USER")]
    PermissionDenied { path: PathBuf },

    #[error("Failed to grab keyboard: {0}")]
    GrabFailed(#[source] std::io::Error),

    #[error("Failed to create uinput device: {0}")]
    UinputFailed(#[source] std::io::Error),

    #[error("evdev error: {0}")]
    Evdev(#[from] evdev::Error),
}
```

### Windows-Specific Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum WindowsDriverError {
    #[error("Failed to install keyboard hook: {0}")]
    HookInstallFailed(windows::core::Error),

    #[error("SendInput failed: error code {0}")]
    SendInputFailed(u32),

    #[error("Message pump thread panicked")]
    MessagePumpPanic,

    #[error("Hook callback timeout exceeded")]
    HookTimeout,
}
```

### Recovery Strategies

1. **Initialization Failure**: Return error, keyboard continues normally
2. **Runtime Grab Loss**: Attempt re-grab once, then shutdown gracefully
3. **Injection Failure**: Log error, skip this key, continue
4. **Thread Panic**: Set running=false, cleanup in Drop impl

## Testing Strategy

### Unit Testing
- Key code conversion: evdev ↔ KeyCode ↔ VK roundtrips
- Event serialization and channel passing
- Error type construction and messages

### Integration Testing
- Mock evdev device for Linux (requires /dev/uinput access)
- Virtual keyboard injection and verification
- Start/stop lifecycle without real keyboard

### Manual Testing Checklist
- [ ] Single key remap (CapsLock → Escape)
- [ ] Key blocking (Insert → nothing)
- [ ] Modifier combinations (Ctrl+C)
- [ ] Rapid key repeat
- [ ] Multiple keyboards (if available)
- [ ] Hot-unplug during operation
- [ ] Ctrl+C graceful shutdown
- [ ] Kill -9 keyboard recovery

### Performance Testing
- Latency measurement under load (100 keys/sec)
- Memory usage over 1 hour operation
- CPU usage during idle and typing

## File Changes Summary

| File | Action | Purpose |
|------|--------|---------|
| `core/src/drivers/linux.rs` | Rewrite | Full evdev/uinput implementation |
| `core/src/drivers/windows.rs` | Rewrite | Full WH_KEYBOARD_LL implementation |
| `core/src/drivers/common.rs` | Create | Shared device listing logic |
| `core/src/drivers/mod.rs` | Modify | Export new types, re-export platform driver |
| `core/src/cli/commands/devices.rs` | Create | `keyrx devices` command |
| `core/src/cli/commands/mod.rs` | Modify | Export DevicesCommand |
| `core/src/bin/keyrx.rs` | Modify | Add devices subcommand |
| `core/src/error.rs` | Modify | Add driver error variants |
| `core/Cargo.toml` | Modify | Add evdev uinput feature |
| `core/tests/driver_test.rs` | Create | Driver integration tests |

## Platform-Specific Notes

### Linux
- Requires `input` group membership or udev rules
- evdev grab prevents other applications from seeing keys
- uinput virtual device appears as new keyboard
- Wayland compatible (evdev is kernel-level)

### Windows
- Hook runs in context of installing thread
- Requires message pump for callbacks
- SendInput subject to UIPI (User Interface Privilege Isolation)
- May trigger antivirus warnings (keyboard hook)

## Security Considerations

- Driver code has no network access
- No file system access beyond /dev/input and /dev/uinput
- Rhai sandbox prevents script access to driver internals
- Keyboard state not persisted to disk
