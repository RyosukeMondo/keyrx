# Design Document

## Overview

This design implements system tray integration and multi-device event discrimination for Linux, achieving feature parity with Windows while enabling the "keyboard-aware remapping" killer feature.

**Key Components:**
1. **Cross-platform SystemTray trait**: Abstraction layer for tray icons
2. **Linux tray implementation**: Using `ksni` crate (KDE/freedesktop StatusNotifierItem protocol)
3. **Device ID propagation**: Serial numbers flow from evdev → KeyEvent → Rhai
4. **Device manager**: Enumerate and track multiple input devices

**Architecture Goal:** Maximize code reuse between platforms while respecting OS-specific tray implementations.

## Steering Document Alignment

### Technical Standards (tech.md)

**Multi-Device Architecture (QMK-Inspired):**
> "Single daemon process, global `ExtendedState` shared across ALL connected devices"

This design preserves the single-daemon model. Device IDs are metadata only—remapping still uses the global state machine.

**Platform Trait Abstraction:**
> "Platform-specific code isolated in `keyrx_daemon`, core logic remains `no_std`"

System tray is 100% in `keyrx_daemon/src/platform/`. `keyrx_core` remains unchanged (device_id is transport metadata, not core logic).

**Linux Input Subsystem:**
> "Uses evdev ioctl, device identification via `/dev/input/by-id/` (persistent serial numbers)"

Design leverages existing `EvdevInput::serial()` method. No changes to evdev layer needed—only add device_id to event pipeline.

### Project Structure (structure.md)

**Four-Crate Workspace:**
```
keyrx_daemon/
├── src/
│   ├── platform/
│   │   ├── mod.rs          # SystemTray trait added here
│   │   ├── linux/
│   │   │   ├── mod.rs      # DeviceManager added
│   │   │   └── tray.rs     # NEW: Linux tray implementation
│   │   └── windows/
│   │       └── tray.rs     # Refactored to implement SystemTray trait
```

## Code Reuse Analysis

### Existing Components to Leverage

1. **Windows Tray Implementation (`keyrx_daemon/src/platform/windows/tray.rs`)**
   - **Reuse:** Event model (`TrayControlEvent::Reload`, `TrayControlEvent::Exit`)
   - **Reuse:** Icon loading via `image` crate (`load_icon()` function)
   - **Reuse:** Menu structure (Reload, Separator, Exit)
   - **Extend:** Extract interface as `SystemTray` trait

2. **Linux EvdevInput (`keyrx_daemon/src/platform/linux/mod.rs`)**
   - **Reuse:** `EvdevInput::serial()` method (line 273-277) for device IDs
   - **Reuse:** `EvdevInput::name()` for human-readable device names
   - **Extend:** Device enumeration logic to open multiple devices simultaneously

3. **KeyEvent Structure (`keyrx_core/src/runtime/event.rs`)**
   - **Extend:** Add `device_id: Option<String>` field (non-breaking change via builder pattern)
   - **Reuse:** Existing `with_timestamp()` pattern for chaining

### Integration Points

1. **Daemon Main Loop**
   - **Current:** Platform-specific event processing in `keyrx_daemon/src/main.rs`
   - **Integration:** Add tray polling before event processing: `if let Some(event) = tray.poll_event() { handle_tray_event(event); }`

2. **Rhai Scripting Engine**
   - **Current:** `keyrx_compiler` generates static lookup tables, no runtime Rhai
   - **Integration:** Add `device_id()` method to Rhai's `event` object bindings (compile-time only, stored in .krx)

3. **Web UI**
   - **Current:** Daemon serves embedded React app via `axum` (keyrx_daemon/src/web/)
   - **Integration:** New REST endpoint `/api/devices` returning JSON list of active devices

## Architecture

### Modular Design Principles

**File Responsibility Breakdown:**
- `platform/mod.rs`: SystemTray trait only (35 lines)
- `platform/linux/tray.rs`: Linux-specific tray (120 lines, mirrors Windows structure)
- `platform/linux/device_manager.rs`: Device enumeration (80 lines)
- `platform/windows/tray.rs`: Refactored to implement trait (no logic changes, +10 lines for trait impl)

**Service Layer Separation:**
- **Platform Layer:** Captures events with device_id metadata
- **Core Layer:** Processes events (unchanged, device_id is optional metadata)
- **Presentation Layer:** Web UI displays device list (read-only)

### System Architecture Diagram

```mermaid
graph TD
    subgraph Linux Platform
        A[/dev/input/event0<br/>Laptop Keyboard] --> B[EvdevInput]
        C[/dev/input/event1<br/>USB Numpad] --> D[EvdevInput]
        E[/dev/input/event2<br/>Gaming Keyboard] --> F[EvdevInput]

        B -->|KeyEvent + device_id| G[DeviceManager]
        D -->|KeyEvent + device_id| G
        F -->|KeyEvent + device_id| G

        G -->|Merged Event Stream| H[Daemon Main Loop]

        I[Linux System Tray<br/>ksni crate] -->|TrayControlEvent| H
    end

    subgraph Cross-Platform
        H --> J[Remapping Engine<br/>keyrx_core]
        J -->|Remapped Events| K[UinputOutput]
    end

    subgraph Web UI
        H -->|WebSocket| L[React Dashboard]
        L -->|REST /api/devices| M[Device List Display]
    end

    style I fill:#e1f5e1
    style B fill:#e1f5e1
    style D fill:#e1f5e1
    style F fill:#e1f5e1
```

## Components and Interfaces

### Component 1: SystemTray Trait

**Purpose:** Cross-platform abstraction for system tray functionality

**Interfaces:**
```rust
// keyrx_daemon/src/platform/mod.rs

pub enum TrayControlEvent {
    Reload,
    Exit,
}

pub trait SystemTray {
    /// Creates a new system tray icon
    fn new() -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;

    /// Polls for tray menu events (non-blocking)
    fn poll_event(&self) -> Option<TrayControlEvent>;

    /// Cleanup (called on daemon shutdown)
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
```

**Dependencies:** None (trait only)

**Reuses:** TrayControlEvent enum from Windows implementation (moved to platform/mod.rs)

### Component 2: Linux Tray Implementation

**Purpose:** Implement system tray for Linux using `ksni` crate

**Interfaces:**
```rust
// keyrx_daemon/src/platform/linux/tray.rs

pub struct LinuxSystemTray {
    handle: ksni::Handle<TrayService>,
    event_receiver: Receiver<TrayControlEvent>,
}

impl SystemTray for LinuxSystemTray {
    fn new() -> Result<Self, Box<dyn std::error::Error>> { /* ... */ }
    fn poll_event(&self) -> Option<TrayControlEvent> { /* ... */ }
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> { /* ... */ }
}

// Internal service for ksni callbacks
struct TrayService {
    sender: Sender<TrayControlEvent>,
}

impl ksni::Tray for TrayService {
    fn activate(&mut self, _x: i32, _y: i32) { /* ... */ }
    fn id(&self) -> String { "keyrx-daemon".to_string() }
    fn title(&self) -> String { "KeyRx Daemon".to_string() }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> { /* Reload, Separator, Exit */ }
}
```

**Dependencies:**
- `ksni = "0.2"` (KDE StatusNotifierItem protocol)
- `crossbeam-channel` (already in workspace, for event passing)

**Reuses:**
- Icon loading from `assets/icon.png` (Windows approach)
- `TrayControlEvent` enum
- Menu structure (Reload, Exit)

### Component 3: DeviceManager

**Purpose:** Enumerate and manage multiple input devices with unique IDs

**Interfaces:**
```rust
// keyrx_daemon/src/platform/linux/device_manager.rs

pub struct DeviceInfo {
    pub id: String,           // Serial or fallback path
    pub name: String,         // Human-readable name
    pub path: PathBuf,        // /dev/input/eventX
    pub serial: Option<String>, // USB serial if available
}

pub struct DeviceManager {
    devices: HashMap<String, EvdevInput>,
    device_info: HashMap<String, DeviceInfo>,
}

impl DeviceManager {
    /// Enumerate all keyboard devices in /dev/input/
    pub fn enumerate() -> Result<Self, DeviceError>;

    /// Get next event from any device (blocking)
    pub fn next_event(&mut self) -> Result<(KeyEvent, String), DeviceError>;

    /// Get device info by ID
    pub fn device_info(&self, id: &str) -> Option<&DeviceInfo>;

    /// List all device IDs
    pub fn device_ids(&self) -> Vec<String>;
}
```

**Dependencies:**
- `evdev` (already in workspace)
- `std::collections::HashMap`

**Reuses:**
- `EvdevInput::open()`, `EvdevInput::serial()`, `EvdevInput::name()`
- Existing device path handling

### Component 4: KeyEvent Extension

**Purpose:** Add optional device_id field to KeyEvent

**Interfaces:**
```rust
// keyrx_core/src/runtime/event.rs

pub struct KeyEvent {
    keycode: KeyCode,
    pressed: bool,
    timestamp_us: u64,
    device_id: Option<String>,  // NEW FIELD
}

impl KeyEvent {
    // Existing constructors remain unchanged (device_id defaults to None)
    pub fn press(keycode: KeyCode) -> Self { /* ... */ }
    pub fn release(keycode: KeyCode) -> Self { /* ... */ }

    // NEW: Builder method for device_id
    pub fn with_device_id(mut self, id: String) -> Self {
        self.device_id = Some(id);
        self
    }

    // NEW: Accessor
    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }
}
```

**Dependencies:** None (pure Rust)

**Reuses:** Existing `with_timestamp()` builder pattern

## Data Models

### DeviceInfo Model
```rust
pub struct DeviceInfo {
    /// Unique identifier (serial or path-based)
    pub id: String,

    /// Human-readable name from evdev
    pub name: String,

    /// Device node path
    pub path: PathBuf,

    /// USB serial number (if available)
    pub serial: Option<String>,
}
```

### TrayControlEvent Enum
```rust
pub enum TrayControlEvent {
    /// User clicked "Reload Config"
    Reload,

    /// User clicked "Exit"
    Exit,
}
```

### Web API Response
```json
// GET /api/devices
{
  "devices": [
    {
      "id": "usb-Logitech_USB_Keyboard-event-kbd",
      "name": "Logitech USB Keyboard",
      "path": "/dev/input/event0",
      "serial": "123456",
      "active": true
    },
    {
      "id": "platform-i8042-serio-0-event-kbd",
      "name": "AT Translated Set 2 keyboard",
      "path": "/dev/input/event1",
      "serial": null,
      "active": true
    }
  ]
}
```

## Error Handling

### Error Scenarios

1. **System Tray Unavailable (Headless Server)**
   - **Handling:** Log warning, continue without tray
   - **User Impact:** Daemon runs, but no GUI control (CLI still works)
   - **Code:**
     ```rust
     match LinuxSystemTray::new() {
         Ok(tray) => Some(tray),
         Err(e) => {
             log::warn!("System tray unavailable: {}. Use CLI to control daemon.", e);
             None
         }
     }
     ```

2. **Device Enumeration Failure (Permission Denied)**
   - **Handling:** Log error, exit with code 1
   - **User Impact:** Daemon won't start; user must fix permissions (add to `input` group)
   - **Code:**
     ```rust
     DeviceManager::enumerate().map_err(|e| {
         eprintln!("Failed to enumerate devices: {}", e);
         eprintln!("Try: sudo usermod -aG input $USER && reboot");
         std::process::exit(1);
     })
     ```

3. **Device Hot-Unplug (USB Keyboard Removed)**
   - **Handling:** V1: Ignore hot-plug (requires restart). V2: Detect and update device list.
   - **User Impact:** If device unplugged, daemon continues with remaining devices
   - **Code:** (V2 future enhancement, not in this spec)

4. **Tray Event Channel Overflow**
   - **Handling:** Use bounded channel with capacity 10; if full, drop oldest event
   - **User Impact:** Rapid clicks on tray menu may miss events (acceptable, user can retry)
   - **Code:**
     ```rust
     let (tx, rx) = crossbeam_channel::bounded(10);
     // sender.try_send() instead of send() to avoid blocking
     ```

## Testing Strategy

### Unit Testing

**Tray Trait:**
- Test that Windows implementation still compiles after trait refactor (no behavior change)
- Mock SystemTray for Linux to verify poll_event() returns correct events

**DeviceManager:**
- Test device enumeration with mock `/dev/input/` paths
- Test device_id generation (serial available vs fallback to path)
- Test next_event() merges events from multiple devices correctly

**KeyEvent Extension:**
- Test `with_device_id()` builder pattern
- Test backward compatibility: events without device_id still process correctly
- Test device_id accessor returns None for legacy events

### Integration Testing

**Linux Tray:**
- Manual test: Run daemon, verify tray icon appears in KDE/GNOME panel
- Click "Reload Config" → verify daemon reloads .krx (check logs)
- Click "Exit" → verify daemon exits cleanly (no stuck grabs)

**Multi-Device:**
- Connect 2 USB keyboards
- Verify daemon enumerates both with distinct IDs
- Verify events are tagged with correct device_id
- Verify web UI shows both devices in /api/devices

**Rhai Integration:**
- Write Rhai script: `if event.device_id() == "numpad" { remap(Key::A, Key::F13) }`
- Compile to .krx
- Verify only numpad events trigger remap (main keyboard unaffected)

### End-to-End Testing

**Scenario: USB Numpad as Stream Deck**
1. User connects USB numpad
2. Daemon enumerates devices, assigns ID "usb-NumericKeypad-123"
3. User writes Rhai config:
   ```rhai
   if event.device_id() == "usb-NumericKeypad-123" {
       map(NUM_1, F13);  // OBS scene 1
       map(NUM_2, F14);  // OBS scene 2
   }
   ```
4. Compile to .krx, reload via tray
5. Press Num1 on numpad → F13 injected
6. Press 1 on main keyboard → normal `1` (not remapped)
7. Verify in web UI: numpad shows green indicator when active

**Coverage Target:** 85% for new code (DeviceManager, tray implementations)
