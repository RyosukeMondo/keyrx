# Design Document

## Overview

This design isolates all unsafe code into minimal safety wrapper modules with comprehensive SAFETY documentation. Each unsafe operation gets a dedicated wrapper type that encapsulates the invariants. The core innovation is the `SafeHook` type that manages Windows hook lifecycle and the `SafeDevice` type for Linux evdev operations.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Minimize unsafe blocks**: Each unsafe block has one purpose
- **SAFETY comments**: Every unsafe block documents invariants
- **Error Handling**: Structured errors for all platform operations

### Project Structure (structure.md)
- Windows safety: `core/src/drivers/windows/safety/`
- Linux safety: `core/src/drivers/linux/safety/`
- Shared patterns: `core/src/drivers/common/`

## Code Reuse Analysis

### Existing Components to Leverage
- **windows-rs**: Already provides some safe wrappers
- **evdev crate**: Has safe APIs to use where possible
- **anyhow::Result**: For error propagation

### Integration Points
- **WindowsInputSource**: Uses SafeHook internally
- **LinuxInputSource**: Uses SafeDevice internally
- **Engine**: Receives errors from drivers

## Architecture

```mermaid
graph TD
    subgraph "Driver Layer"
        WIS[WindowsInputSource] --> SH[SafeHook]
        LIS[LinuxInputSource] --> SD[SafeDevice]
    end

    subgraph "Windows Safety"
        SH --> HW[HookWrapper]
        SH --> TL[ThreadLocalState]
        HW --> API[Windows API]
    end

    subgraph "Linux Safety"
        SD --> EV[EvdevWrapper]
        SD --> UI[UinputWrapper]
        EV --> DEV[/dev/input/*]
        UI --> UDEV[/dev/uinput]
    end

    subgraph "Common"
        WIS --> ER[DriverError]
        LIS --> ER
        ER --> REC[Recovery]
    end
```

### Modular Design Principles
- **Single File Responsibility**: Each safety wrapper in its own file
- **Component Isolation**: Wrappers don't know about each other
- **Minimal Unsafe**: Only necessary operations are unsafe
- **SAFETY Documentation**: Every unsafe block is documented

## Components and Interfaces

### Component 1: SafeHook (Windows)

- **Purpose:** Safe wrapper for Windows keyboard hooks
- **Interfaces:**
  ```rust
  /// Safe wrapper for Windows low-level keyboard hook.
  ///
  /// # Safety Invariants
  /// - Hook handle is valid while SafeHook exists
  /// - Callback never panics (caught internally)
  /// - Hook is unset on Drop
  pub struct SafeHook {
      handle: HHOOK,
      callback_id: usize,
  }

  impl SafeHook {
      /// Install a keyboard hook.
      ///
      /// # Errors
      /// Returns error if SetWindowsHookEx fails.
      pub fn install(callback: HookCallback) -> Result<Self, DriverError>;

      /// Check if hook is still valid.
      pub fn is_valid(&self) -> bool;
  }

  impl Drop for SafeHook {
      fn drop(&mut self) {
          // SAFETY: handle is valid, UnhookWindowsHookEx is thread-safe
          unsafe { UnhookWindowsHookEx(self.handle) };
      }
  }
  ```
- **Dependencies:** windows-rs
- **Reuses:** Windows API patterns

### Component 2: ThreadLocalState (Windows)

- **Purpose:** Safe thread-local storage for hook context
- **Interfaces:**
  ```rust
  /// Thread-local state for routing hook events.
  ///
  /// # Safety Invariants
  /// - State is only accessed from hook callback thread
  /// - Initialization is atomic
  /// - Access never panics
  pub struct ThreadLocalState {
      sender: Option<Sender<KeyEvent>>,
  }

  impl ThreadLocalState {
      /// Get the thread-local state, initializing if needed.
      pub fn get() -> &'static Self;

      /// Set the event sender.
      pub fn set_sender(&self, sender: Sender<KeyEvent>);

      /// Send an event through the channel.
      pub fn send(&self, event: KeyEvent) -> Result<(), DriverError>;
  }
  ```
- **Dependencies:** std::cell, crossbeam
- **Reuses:** Thread-local patterns

### Component 3: HookCallback (Windows)

- **Purpose:** Panic-safe hook callback wrapper
- **Interfaces:**
  ```rust
  /// Panic-catching wrapper for hook callback.
  ///
  /// # Safety
  /// - Catches all panics to prevent UB
  /// - Logs panic info for debugging
  /// - Returns valid LRESULT even on panic
  pub struct HookCallback {
      inner: Box<dyn Fn(KeyEvent) -> HookAction + Send>,
  }

  impl HookCallback {
      pub fn new<F>(f: F) -> Self
      where
          F: Fn(KeyEvent) -> HookAction + Send + 'static;

      /// Invoke callback with panic catching.
      pub fn invoke(&self, event: KeyEvent) -> HookAction {
          std::panic::catch_unwind(|| (self.inner)(event))
              .unwrap_or_else(|panic| {
                  log::error!("Hook callback panicked: {:?}", panic);
                  HookAction::PassThrough
              })
      }
  }
  ```
- **Dependencies:** std::panic
- **Reuses:** Panic catching pattern

### Component 4: SafeDevice (Linux)

- **Purpose:** Safe wrapper for evdev device operations
- **Interfaces:**
  ```rust
  /// Safe wrapper for Linux evdev device.
  ///
  /// # Safety Invariants
  /// - Device file descriptor is valid while SafeDevice exists
  /// - Exclusive grab is held
  /// - Device is released on Drop
  pub struct SafeDevice {
      device: evdev::Device,
      path: PathBuf,
      grabbed: bool,
  }

  impl SafeDevice {
      /// Open and grab a device.
      ///
      /// # Errors
      /// - DeviceNotFound if path doesn't exist
      /// - PermissionDenied if no read access
      /// - GrabFailed if exclusive access denied
      pub fn open(path: impl AsRef<Path>) -> Result<Self, DriverError>;

      /// Read next event with timeout.
      pub fn read_event(&mut self, timeout: Duration) -> Result<Option<InputEvent>, DriverError>;

      /// Check if device is still connected.
      pub fn is_connected(&self) -> bool;
  }

  impl Drop for SafeDevice {
      fn drop(&mut self) {
          if self.grabbed {
              let _ = self.device.ungrab();
          }
      }
  }
  ```
- **Dependencies:** evdev crate
- **Reuses:** evdev safe APIs

### Component 5: SafeUinput (Linux)

- **Purpose:** Safe wrapper for uinput virtual device
- **Interfaces:**
  ```rust
  /// Safe wrapper for Linux uinput virtual device.
  ///
  /// # Safety Invariants
  /// - Virtual device is created on construction
  /// - Device is destroyed on Drop
  /// - Events are validated before injection
  pub struct SafeUinput {
      device: UInputDevice,
  }

  impl SafeUinput {
      /// Create a virtual keyboard device.
      pub fn create_keyboard(name: &str) -> Result<Self, DriverError>;

      /// Inject a key event.
      pub fn inject(&mut self, event: KeyEvent) -> Result<(), DriverError>;

      /// Sync events (required after injection).
      pub fn sync(&mut self) -> Result<(), DriverError>;
  }
  ```
- **Dependencies:** uinput crate
- **Reuses:** uinput safe APIs

### Component 6: DriverError

- **Purpose:** Unified driver errors with recovery hints
- **Interfaces:**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum DriverError {
      #[error("Device not found: {path}")]
      DeviceNotFound { path: PathBuf },

      #[error("Permission denied: {resource} (hint: {hint})")]
      PermissionDenied { resource: String, hint: String },

      #[error("Device disconnected: {device}")]
      DeviceDisconnected { device: String },

      #[error("Hook installation failed: {code}")]
      HookFailed { code: u32 },

      #[error("Grab failed: {reason}")]
      GrabFailed { reason: String },

      #[error("Temporary error (retryable): {message}")]
      Temporary { message: String, retry_after: Duration },

      #[error("Platform error: {0}")]
      Platform(#[from] std::io::Error),
  }

  impl DriverError {
      pub fn is_retryable(&self) -> bool;
      pub fn suggested_action(&self) -> &'static str;
  }
  ```
- **Dependencies:** thiserror
- **Reuses:** Error patterns

## Data Models

### KeyEvent
```rust
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub timestamp: u64,
    pub scan_code: u16,
}
```

### HookAction
```rust
#[derive(Debug, Clone, Copy)]
pub enum HookAction {
    Block,
    PassThrough,
    Replace(KeyCode),
}
```

## Error Handling

### Error Scenarios

1. **Hook installation fails**
   - **Handling:** Return `DriverError::HookFailed` with Windows error code
   - **User Impact:** Clear message about what failed

2. **Device disconnection**
   - **Handling:** Return `DriverError::DeviceDisconnected`, engine reconnects
   - **User Impact:** Brief interruption, auto-recovery

3. **Permission denied on Linux**
   - **Handling:** Return error with hint about input group membership
   - **User Impact:** Actionable fix suggestion

4. **Panic in hook callback**
   - **Handling:** Catch, log, return PassThrough
   - **User Impact:** Key works normally, bug is logged

## Testing Strategy

### Unit Testing
- Test error type construction
- Test recovery suggestions
- Test is_retryable logic

### Integration Testing
- Test hook lifecycle (install/uninstall)
- Test device open/close
- Test error handling paths

### Platform Testing
- Test on Windows 10/11
- Test on Linux with various kernels
- Test device disconnection scenarios
