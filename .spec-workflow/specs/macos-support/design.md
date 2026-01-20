# Design Document

## Overview

This design adds macOS platform support to keyrx by implementing the existing Platform trait using high-level Rust crates (rdev, enigo, tray-icon) to minimize Objective-C FFI complexity. The implementation achieves <1ms latency using CGEventTap for input capture and CGEventPost for output injection, with memory safety guaranteed through RAII patterns and minimal unsafe code (<5% of platform code).

The macOS platform will be isolated in `keyrx_daemon/src/platform/macos/`, requiring **zero changes** to keyrx_core, keyrx_compiler, or keyrx_ui. This preserves the four-crate architecture and maintains cross-platform configuration compatibility.

## Steering Document Alignment

### Technical Standards (tech.md)

**Language and Tooling**:
- Rust 1.70+ stable (line 16-20): macOS uses same Rust toolchain as Linux/Windows
- Multiple compilation targets (line 18-20): Adds `x86_64-apple-darwin` and `aarch64-apple-darwin`
- wasm32-unknown-unknown (line 20): No changes to WASM compilation (keyrx_core unmodified)

**Platform Abstraction** (tech.md line 318-334):
- Implements existing `Platform` trait (line 320-337)
- Uses conditional compilation: `#[cfg(target_os = "macos")]` (follows Windows/Linux pattern)
- Factory pattern: Adds macOS arm to `create_platform()` function

**Dependency Management** (tech.md line 26-77):
- New macOS-gated dependencies: `rdev`, `enigo`, `iokit-sys`, `objc2`, `accessibility-sys`
- Feature flags: `#[cfg(target_os = "macos")]` for platform-specific code
- No changes to core dependencies (rkyv, hashbrown, fixedbitset remain platform-agnostic)

**Four-Crate Architecture** (tech.md line 79-116):
- keyrx_core: **No changes** (no_std, OS-agnostic)
- keyrx_compiler: **No changes** (Rhai → .krx compilation platform-independent)
- keyrx_daemon: **Add** `platform/macos/` module (isolated platform code)
- keyrx_ui: **No changes** (React + WASM frontend unchanged)

**Multi-Device Support** (tech.md line 134-267):
- Global state model (line 141-145): macOS shares same `ExtendedState` for cross-device modifiers
- Device identification (line 147-167): Uses IOKit to extract serial numbers (parallel to evdev/Windows)
- Single entry point (line 169-218): macOS loads same `main.rhai` as Linux/Windows

### Project Structure (structure.md)

**Directory Organization** (structure.md line 67-70):
```
keyrx_daemon/src/platform/
├── mod.rs                     # Add macOS to create_platform()
├── common.rs                  # Unchanged
├── linux/                     # Unchanged
├── windows/                   # Unchanged
├── macos/                     # NEW
│   ├── mod.rs                # MacosPlatform struct
│   ├── input_capture.rs      # rdev-based input wrapper
│   ├── output_injection.rs   # enigo-based output wrapper
│   ├── device_discovery.rs   # IOKit USB enumeration
│   ├── keycode_map.rs        # CGKeyCode ↔ KeyRx KeyCode mapping
│   ├── tray.rs               # tray-icon integration
│   └── permissions.rs        # Accessibility permission checks
└── mock.rs                   # Unchanged
```

**Naming Conventions** (structure.md line 109-135):
- Files: `snake_case.rs` (e.g., `device_discovery.rs`, `keycode_map.rs`)
- Structs: `PascalCase` (e.g., `MacosPlatform`, `MacosInputCapture`)
- Functions: `snake_case` (e.g., `check_accessibility_permission`, `convert_event`)

**Import Order** (structure.md line 136-153):
```rust
// 1. Standard library
use std::sync::mpsc::{channel, Receiver};

// 2. External dependencies
use rdev::{listen, Event, EventType};
use enigo::{Enigo, Key, Direction};

// 3. Internal workspace crates
use keyrx_core::{KeyEvent, KeyCode};

// 4. Current crate modules
use crate::platform::common::PlatformError;
```

**Module Boundaries** (structure.md line 560-595):
- Core vs Platform-Specific (line 562-572): macOS code in daemon only, core remains no_std
- Public vs Internal (line 575-584): Platform trait is public API, implementation details internal
- Optional Features (line 592-595): macOS support feature-gated with `#[cfg(target_os = "macos")]`

## Code Reuse Analysis

### Existing Components to Leverage

**1. Platform Trait Abstraction** (`keyrx_daemon/src/platform/mod.rs`):
- **Reuse**: Implement existing `Platform` trait (lines 320-337)
- **Extension**: Add macOS arm to `create_platform()` factory function
- **No Changes**: Trait definition remains identical (preserves Linux/Windows compatibility)

**2. Common Error Types** (`keyrx_daemon/src/platform/common.rs`):
- **Reuse**: Use existing `PlatformError` enum for macOS errors
- **Extension**: Add macOS-specific error variants (e.g., `AccessibilityPermissionDenied`)
- **Leverage**: DeviceInfo struct for device enumeration results

**3. tray-icon Crate** (already used for Windows):
- **Reuse**: Directly use `tray-icon` crate (already supports macOS)
- **Pattern**: Follow Windows tray implementation in `keyrx_daemon/src/platform/windows/tray.rs`
- **Integration**: Use existing `SystemTray` trait and `TrayControlEvent` enum

**4. WASM Frontend** (keyrx_ui):
- **Reuse**: No changes needed (WASM simulation platform-independent)
- **Testing**: Use existing WASM simulator to validate macOS configuration

**5. Configuration System**:
- **Reuse**: .krx binary format works identically on macOS
- **Reuse**: Rhai compiler (keyrx_compiler) generates same output
- **Leverage**: Memory-mapped .krx loading via memmap2 crate

### Integration Points

**1. Existing Platform Factory** (`keyrx_daemon/src/platform/mod.rs:320-337`):
```rust
// Add macOS to conditional compilation
#[cfg(target_os = "macos")]
pub fn create_platform() -> Result<Box<dyn Platform>, PlatformError> {
    Ok(Box::new(macos::MacosPlatform::new()?))
}
```

**2. CI/CD Pipeline** (`.github/workflows/ci.yml`):
- **Integration**: Add macOS runner to GitHub Actions matrix
- **Pattern**: Follow existing Linux/Windows CI jobs
- **Extension**: Add code signing step for macOS binaries

**3. Build System** (`Cargo.toml`):
```toml
[target.'cfg(target_os = "macos")'.dependencies]
rdev = "0.5.3"
enigo = "0.2"
iokit-sys = "0.4"
objc2 = "0.5"
accessibility-sys = "0.1"
# tray-icon already exists (cross-platform)
```

**4. Daemon Main** (`keyrx_daemon/src/main.rs`):
- **Integration**: No changes needed (uses `create_platform()` factory)
- **Reuse**: Command-line arguments work identically on macOS

## Architecture

### Rust-First Strategy: Minimize Objective-C FFI

**Design Philosophy**: Leverage battle-tested Rust crates to avoid writing raw Objective-C FFI code, reducing:
- Memory leak risks (manual retain/release eliminated)
- Development complexity (unsafe blocks minimized)
- Maintenance burden (crate authors handle macOS API changes)

**Crate Selection Rationale**:

| Component | Crate | Safety Level | Justification |
|-----------|-------|--------------|---------------|
| Input Capture | **rdev** v0.5.x | 100% Safe API | Wraps CGEventTap internally; cross-platform |
| Output Injection | **enigo** v0.2.x | 100% Safe API | Wraps CGEventPost internally; cross-platform |
| System Tray | **tray-icon** v0.19.x | 100% Safe API | Already used for Windows; macOS native |
| Device Enumeration | **iokit-sys** v0.4.x | FFI Required | No high-level alternative; isolated unsafe code |
| Objective-C Helpers | **objc2** v0.5.x | RAII Safety | Auto-managed memory via `Retained<T>` |
| Permission Check | **accessibility-sys** v0.1.x | Safe Wrapper | Single function: `AXIsProcessTrusted()` |

**Memory Safety Architecture**:

```
┌─────────────────────────────────────────────────────────┐
│              MacosPlatform (100% Safe Rust)             │
│  ┌─────────────────────────────────────────────────┐   │
│  │ input_capture.rs    (rdev - Safe)               │   │
│  │ output_injection.rs (enigo - Safe)              │   │
│  │ tray.rs             (tray-icon - Safe)          │   │
│  │ permissions.rs      (accessibility-sys - Safe)  │   │
│  │ keycode_map.rs      (Pure Rust - Safe)          │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │ device_discovery.rs (iokit-sys - UNSAFE)        │   │
│  │ - Only module with unsafe blocks                │   │
│  │ - <5% of total platform code                    │   │
│  │ - RAII wrappers for IOKit objects               │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
              │ (implements Platform trait)
              ▼
┌─────────────────────────────────────────────────────────┐
│        macOS System APIs (Objective-C/C)                │
│  - CGEventTap (Quartz) - via rdev                       │
│  - CGEventPost (Quartz) - via enigo                     │
│  - NSStatusBar (AppKit) - via tray-icon                 │
│  - IOKit (USB) - via iokit-sys (direct FFI)             │
└─────────────────────────────────────────────────────────┘
```

**Unsafe Code Budget**: Target <5% of macOS platform codebase (only in `device_discovery.rs`)

### Modular Design Principles

**Single File Responsibility**:
- `input_capture.rs`: Keyboard event capture only (rdev integration)
- `output_injection.rs`: Keyboard event injection only (enigo integration)
- `device_discovery.rs`: USB device enumeration only (IOKit FFI)
- `keycode_map.rs`: Key code translation only (CGKeyCode ↔ KeyRx)
- `tray.rs`: Menu bar integration only (tray-icon integration)
- `permissions.rs`: Accessibility permission checks only

**Component Isolation**:
- Each module has clear input/output contracts
- Modules communicate via defined interfaces (not shared state)
- Testing: Each module can be tested in isolation with mocks

**Service Layer Separation**:
- Platform layer (daemon): OS-specific code only
- Core layer (keyrx_core): Platform-agnostic logic
- UI layer (keyrx_ui): Platform-independent interface

## Components and Interfaces

### Component 1: MacosPlatform

- **Purpose:** Orchestrates macOS platform components and implements Platform trait
- **Interfaces:**
  ```rust
  pub struct MacosPlatform {
      input: MacosInputCapture,
      output: MacosOutputInjector,
      devices: Vec<DeviceInfo>,
  }

  impl Platform for MacosPlatform {
      fn initialize(&mut self) -> Result<(), PlatformError>;
      fn capture_input(&mut self) -> Result<KeyEvent, PlatformError>;
      fn inject_output(&mut self, event: KeyEvent) -> Result<(), PlatformError>;
      fn list_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError>;
      fn shutdown(&mut self) -> Result<(), PlatformError>;
  }
  ```
- **Dependencies:** MacosInputCapture, MacosOutputInjector, device_discovery module
- **Reuses:** Platform trait definition, PlatformError enum, DeviceInfo struct

### Component 2: MacosInputCapture

- **Purpose:** Captures keyboard events using rdev crate (CGEventTap wrapper)
- **Interfaces:**
  ```rust
  pub struct MacosInputCapture {
      event_rx: Receiver<KeyEvent>,
  }

  impl MacosInputCapture {
      pub fn new() -> Result<Self, PlatformError>;
      pub fn next_event(&mut self) -> Result<KeyEvent, PlatformError>;
  }

  // Internal helper
  fn convert_event(event: rdev::Event) -> Option<KeyEvent>;
  ```
- **Dependencies:** rdev crate, keycode_map module, std::sync::mpsc
- **Reuses:** KeyEvent struct from keyrx_core

**Implementation Pattern**:
```rust
use rdev::{listen, Event, EventType};
use std::sync::mpsc::{channel, Receiver};

pub struct MacosInputCapture {
    event_rx: Receiver<KeyEvent>,
}

impl MacosInputCapture {
    pub fn new() -> Result<Self, PlatformError> {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            listen(move |event: Event| {
                if let Some(key_event) = convert_event(event) {
                    tx.send(key_event).ok();
                }
            }).expect("Failed to start event listener");
        });

        Ok(Self { event_rx: rx })
    }

    pub fn next_event(&mut self) -> Result<KeyEvent, PlatformError> {
        self.event_rx.recv()
            .map_err(|_| PlatformError::InputDeviceError {
                message: "Event channel closed".to_string(),
            })
    }
}

fn convert_event(event: Event) -> Option<KeyEvent> {
    match event.event_type {
        EventType::KeyPress(key) => Some(KeyEvent::Press(map_rdev_key(key))),
        EventType::KeyRelease(key) => Some(KeyEvent::Release(map_rdev_key(key))),
        _ => None,
    }
}
```

### Component 3: MacosOutputInjector

- **Purpose:** Injects remapped keyboard events using enigo crate (CGEventPost wrapper)
- **Interfaces:**
  ```rust
  pub struct MacosOutputInjector {
      enigo: Enigo,
  }

  impl MacosOutputInjector {
      pub fn new() -> Result<Self, PlatformError>;
      pub fn inject(&mut self, event: KeyEvent) -> Result<(), PlatformError>;
  }
  ```
- **Dependencies:** enigo crate, keycode_map module
- **Reuses:** KeyEvent struct from keyrx_core

**Implementation Pattern**:
```rust
use enigo::{Enigo, Key, Direction, Settings};

pub struct MacosOutputInjector {
    enigo: Enigo,
}

impl MacosOutputInjector {
    pub fn new() -> Result<Self, PlatformError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| PlatformError::OutputDeviceError {
                message: format!("Failed to create enigo: {}", e),
            })?;
        Ok(Self { enigo })
    }

    pub fn inject(&mut self, event: KeyEvent) -> Result<(), PlatformError> {
        let (key, direction) = match event {
            KeyEvent::Press(k) => (map_keycode_to_enigo(k), Direction::Press),
            KeyEvent::Release(k) => (map_keycode_to_enigo(k), Direction::Release),
        };

        self.enigo.key(key, direction)
            .map_err(|e| PlatformError::OutputDeviceError {
                message: format!("Injection failed: {}", e),
            })
    }
}
```

### Component 4: Device Discovery (IOKit FFI)

- **Purpose:** Enumerates USB keyboard devices and extracts VID/PID/serial using IOKit
- **Interfaces:**
  ```rust
  pub fn list_keyboard_devices() -> Result<Vec<DeviceInfo>, PlatformError>;

  // Internal unsafe helpers
  unsafe fn extract_device_info(device: io_object_t) -> Option<DeviceInfo>;
  unsafe fn get_device_property<T>(device: io_object_t, key: &str) -> Option<T>;
  ```
- **Dependencies:** iokit-sys crate, core-foundation crate
- **Reuses:** DeviceInfo struct from common.rs

**RAII Pattern for Resource Safety**:
```rust
use iokit_sys::*;
use core_foundation::base::TCFType;

pub fn list_keyboard_devices() -> Result<Vec<DeviceInfo>, PlatformError> {
    unsafe {
        // Create matching dictionary (auto-released via CFRelease)
        let matching = IOServiceMatching(kIOUSBDeviceClassName);

        // Get iterator (RAII wrapper needed)
        let mut iterator: io_iterator_t = 0;
        let result = IOServiceGetMatchingServices(
            kIOMainPortDefault,
            matching,
            &mut iterator
        );

        if result != KERN_SUCCESS {
            return Err(PlatformError::DeviceEnumerationError);
        }

        // RAII: Ensure iterator is released even on early return
        let _iterator_guard = IOObjectGuard(iterator);

        let mut devices = Vec::new();
        loop {
            let device = IOIteratorNext(iterator);
            if device == 0 { break; }

            // RAII: Ensure device is released
            let _device_guard = IOObjectGuard(device);

            if let Some(info) = extract_device_info(device) {
                devices.push(info);
            }
        }

        Ok(devices)
    }
}

// RAII wrapper for IOKit objects
struct IOObjectGuard(io_object_t);

impl Drop for IOObjectGuard {
    fn drop(&mut self) {
        unsafe { IOObjectRelease(self.0); }
    }
}
```

### Component 5: Keycode Mapping

- **Purpose:** Bidirectional translation between macOS CGKeyCode and keyrx KeyCode
- **Interfaces:**
  ```rust
  pub fn cgkeycode_to_keyrx(code: u16) -> Option<KeyCode>;
  pub fn keyrx_to_cgkeycode(key: KeyCode) -> Option<u16>;

  // For rdev integration
  pub fn rdev_key_to_keyrx(key: rdev::Key) -> Option<KeyCode>;
  pub fn keyrx_to_enigo_key(key: KeyCode) -> Option<enigo::Key>;
  ```
- **Dependencies:** None (pure Rust logic)
- **Reuses:** KeyCode enum from keyrx_core

**Implementation Pattern**:
```rust
pub fn cgkeycode_to_keyrx(code: u16) -> Option<KeyCode> {
    match code {
        0x00 => Some(KeyCode::A),
        0x01 => Some(KeyCode::S),
        0x02 => Some(KeyCode::D),
        // ... 100+ mappings
        0x35 => Some(KeyCode::Escape),
        0x24 => Some(KeyCode::Return),
        _ => None,
    }
}

pub fn keyrx_to_cgkeycode(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::A => Some(0x00),
        KeyCode::S => Some(0x01),
        // ... reverse mappings
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_mapping() {
        for code in 0..=127u16 {
            if let Some(key) = cgkeycode_to_keyrx(code) {
                assert_eq!(keyrx_to_cgkeycode(key), Some(code),
                    "Bidirectional mapping failed for CGKeyCode {}", code);
            }
        }
    }
}
```

### Component 6: Accessibility Permission Checker

- **Purpose:** Detects Accessibility permission status and provides actionable errors
- **Interfaces:**
  ```rust
  pub fn check_accessibility_permission() -> bool;
  pub fn get_permission_error_message() -> String;
  ```
- **Dependencies:** accessibility-sys crate
- **Reuses:** PlatformError enum

**Implementation Pattern**:
```rust
use accessibility_sys::accessibility::AXIsProcessTrusted;

pub fn check_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn get_permission_error_message() -> String {
    r#"Accessibility permission required.

keyrx needs Accessibility permission to intercept keyboard events.

To grant permission:
1. Open System Preferences
2. Go to Security & Privacy → Privacy → Accessibility
3. Click the lock icon and enter your password
4. Check the box next to "keyrx"
5. Restart keyrx

For detailed setup instructions with screenshots:
https://docs.keyrx.io/setup/macos
"#.to_string()
}
```

### Component 7: System Tray Integration

- **Purpose:** Displays menu bar icon and handles menu events
- **Interfaces:**
  ```rust
  pub struct MacosSystemTray {
      _tray: TrayIcon,
      menu_rx: Receiver<MenuEvent>,
  }

  impl SystemTray for MacosSystemTray {
      fn new() -> Result<Self, PlatformError>;
      fn poll_event(&mut self) -> Option<TrayControlEvent>;
      fn shutdown(&mut self) -> Result<(), PlatformError>;
  }
  ```
- **Dependencies:** tray-icon crate (already in Cargo.toml)
- **Reuses:** Windows tray implementation pattern from `platform/windows/tray.rs`

**Implementation Pattern** (nearly identical to Windows):
```rust
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};

pub struct MacosSystemTray {
    _tray: TrayIcon,
    menu_rx: Receiver<MenuEvent>,
}

impl SystemTray for MacosSystemTray {
    fn new() -> Result<Self, PlatformError> {
        let menu = Menu::new();
        menu.append(&MenuItem::new("Open Web UI", true, None))?;
        menu.append(&MenuItem::new("Reload Config", true, None))?;
        menu.append(&MenuItem::new("Exit", true, None))?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("keyrx - Keyboard Remapper")
            .build()?;

        // Event channel setup...
        Ok(Self { _tray: tray, menu_rx })
    }

    // Implementation matches Windows version...
}
```

## Data Models

No new data models required. macOS implementation reuses existing structures:

### Existing Model: DeviceInfo (from `platform/common.rs`)
```rust
pub struct DeviceInfo {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial: Option<String>,
    pub path: String,  // macOS: IOKit device path
}
```

### Existing Model: KeyEvent (from `keyrx_core`)
```rust
pub enum KeyEvent {
    Press(KeyCode),
    Release(KeyCode),
}
```

### Existing Model: PlatformError (from `platform/common.rs`)
```rust
pub enum PlatformError {
    Unsupported { os: String },
    InputDeviceError { message: String },
    OutputDeviceError { message: String },
    DeviceEnumerationError,
    PermissionDenied { message: String },  // Add for macOS Accessibility
    // ... existing variants
}
```

**Extension for macOS**:
```rust
// Add to PlatformError enum
PermissionDenied { message: String },  // Accessibility permission denied
```

## Error Handling

### Error Scenarios

#### 1. Accessibility Permission Denied
- **Description:** User has not granted Accessibility permission in System Preferences
- **Handling:**
  - Detect using `AXIsProcessTrusted()` at startup
  - Return `PlatformError::PermissionDenied` with full setup instructions
  - Log structured JSON: `{"event":"permission_denied","type":"accessibility","instructions_url":"..."}`
- **User Impact:** Clear error message with step-by-step instructions displayed in terminal and web UI

#### 2. rdev Event Listener Fails to Start
- **Description:** CGEventTap registration fails (e.g., permission issue, system overload)
- **Handling:**
  - Catch error in `MacosInputCapture::new()`
  - Return `PlatformError::InputDeviceError` with diagnostic message
  - Log CGEventTap error code if available
- **User Impact:** Daemon fails to start with clear error message

#### 3. enigo Output Injection Fails
- **Description:** CGEventPost fails (rare, system-level issue)
- **Handling:**
  - Log error with keycode and error details
  - Increment failure metric (for monitoring)
  - Continue processing (don't crash daemon)
- **User Impact:** Single key event may be lost; logged for debugging

#### 4. IOKit Device Enumeration Fails
- **Description:** USB enumeration returns error (permissions, system state)
- **Handling:**
  - Log warning with error details
  - Fall back to input-only mode (no device filtering)
  - Device list API returns empty array with warning
- **User Impact:** Device-specific configs won't work; global config still functional

#### 5. Code Signing Certificate Missing (Distribution)
- **Description:** Binary not signed, Gatekeeper blocks execution
- **Handling:**
  - Documentation explains code signing requirement
  - CI/CD fails if certificate not configured
  - Unsigned builds marked as "development only"
- **User Impact:** User must right-click → "Open" to bypass Gatekeeper (documented)

## Testing Strategy

### Unit Testing

**Coverage Target**: ≥80% for all macOS platform code

**Key Components to Test**:
1. **keycode_map.rs**:
   - Test all 100+ CGKeyCode ↔ KeyRx mappings
   - Verify bidirectional mapping correctness
   - Property-based testing: `cgkeycode_to_keyrx(keyrx_to_cgkeycode(x)) == Some(x)`

2. **permissions.rs**:
   - Mock `AXIsProcessTrusted()` for both granted/denied states
   - Verify error message contains setup instructions

3. **device_discovery.rs**:
   - Mock IOKit responses (requires significant FFI mocking)
   - Test RAII cleanup (IOObject release called)
   - Test error handling for missing serial numbers

**Testing Pattern**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_mapping_coverage() {
        // Ensure all macOS keycodes are mapped
        let known_codes = vec![0x00, 0x01, 0x02, /* ... */];
        for code in known_codes {
            assert!(cgkeycode_to_keyrx(code).is_some(),
                "CGKeyCode {} not mapped", code);
        }
    }

    #[test]
    fn test_permission_error_message() {
        let msg = get_permission_error_message();
        assert!(msg.contains("System Preferences"));
        assert!(msg.contains("Accessibility"));
    }
}
```

### Integration Testing

**Approach**: Test platform components working together (without real macOS APIs)

**Key Flows to Test**:
1. **Input Capture → Keycode Mapping**:
   - Mock rdev Event stream
   - Verify correct KeyEvent produced
   - Test event ordering (FIFO guarantee)

2. **Keycode Mapping → Output Injection**:
   - Mock enigo injection
   - Verify correct CGEventPost parameters
   - Test modifier state handling

3. **Full Pipeline (Mocked)**:
   - Simulate: Capture A → Remap to B → Inject B
   - Verify latency tracking works
   - Test error propagation

**Testing Pattern**:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_capture_remap_inject_flow() {
        // Mock components
        let mut platform = create_mock_macos_platform();

        // Simulate input
        mock_inject_rdev_event(Event::KeyPress(rdev::Key::KeyA));

        // Capture
        let captured = platform.capture_input().unwrap();
        assert_eq!(captured, KeyEvent::Press(KeyCode::A));

        // Inject (mocked)
        platform.inject_output(KeyEvent::Press(KeyCode::B)).unwrap();

        // Verify mock received correct event
        assert_eq!(mock_get_last_injected(), KeyCode::B);
    }
}
```

### End-to-End Testing

**Approach**: Run on real macOS hardware with Accessibility permission granted

**User Scenarios to Test**:
1. **First-time Setup**:
   - Install keyrx on fresh macOS
   - Verify permission prompt behavior
   - Grant permission and verify daemon starts

2. **Basic Remapping**:
   - Load simple config (A → B)
   - Press A key physically
   - Verify B appears in text editor

3. **Multi-Device Configuration**:
   - Connect two USB keyboards
   - Configure device-specific mappings
   - Verify correct keyboard triggers correct config

4. **Menu Bar Integration**:
   - Click menu bar icon
   - Select "Reload Config"
   - Verify config reloads without restart

5. **Crash Recovery**:
   - Simulate daemon crash (kill -9)
   - Verify no keys stuck in pressed state
   - Restart daemon, verify functionality restored

**CI/CD Integration**:
```yaml
# .github/workflows/ci.yml
jobs:
  test-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Build
        run: cargo build --target x86_64-apple-darwin
      - name: Unit Tests
        run: cargo test -p keyrx_daemon --lib macos
      - name: Integration Tests (no Accessibility)
        run: cargo test -p keyrx_daemon --test macos_integration
      # E2E tests skipped (no Accessibility in CI)
```

**Manual E2E Test Checklist** (run before release):
- [ ] Event capture latency <1ms (measure with timestamping)
- [ ] Event injection latency <1ms
- [ ] Menu bar icon appears and menus work
- [ ] Config reload completes within 500ms
- [ ] Device enumeration lists all connected keyboards
- [ ] Cross-device modifiers work (Shift on keyboard A affects keyboard B)
- [ ] Daemon survives 10,000 key presses without crash
- [ ] Memory usage stable (<50MB) over 1 hour session
- [ ] CPU usage <1% idle, <5% under load
- [ ] No memory leaks (Xcode Instruments verification)
