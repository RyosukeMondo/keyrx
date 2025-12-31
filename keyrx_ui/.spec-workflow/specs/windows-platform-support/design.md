# Design: Windows Platform Support

**Spec Name**: windows-platform-support
**Created**: 2024-12-24
**Status**: Draft
**Version**: 0.1.0

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Component Design](#component-design)
3. [Data Structures](#data-structures)
4. [Algorithms](#algorithms)
5. [Platform Integration](#platform-integration)
6. [Error Handling](#error-handling)
7. [Testing Strategy](#testing-strategy)
8. [Performance Considerations](#performance-considerations)

---

## Overview

This document details the technical design for implementing Windows platform support in keyrx, following the low-level hooks approach to achieve feature parity with the existing Linux implementation while maintaining sub-millisecond latency.

## Steering Document Alignment

### Technical Standards (tech.md)

This implementation follows the established technical patterns:

- **Four-Crate Architecture**: Windows code integrated into existing keyrx_daemon crate, isolating platform-specific logic
- **no_std Core Preservation**: keyrx_core remains platform-agnostic; all Windows APIs used only in keyrx_daemon/src/platform/windows/*
- **Rust Language Standards**: Uses windows-sys crate (official Microsoft Rust bindings), maintaining memory safety without GC
- **Zero-Cost Abstractions**: Windows hooks abstracted behind Platform trait (InputDevice/OutputDevice), no runtime overhead
- **Lock-Free Hot Path**: Event routing uses crossbeam_channel (lock-free MPMC), matching Linux implementation pattern
- **Dependency Injection**: Platform-specific code injected via trait objects, enabling testing with mock implementations

### Project Structure (structure.md)

Follows the documented module organization:

```
keyrx_daemon/src/platform/
├── mod.rs              # Platform trait (existing)
├── linux.rs            # Existing evdev/uinput implementation
└── windows/            # NEW Windows implementation
    ├── mod.rs          # Windows platform exports
    ├── keycode.rs      # VK ↔ KeyCode mapping (static arrays)
    ├── hook.rs         # SetWindowsHookEx wrapper
    ├── inject.rs       # SendInput wrapper
    ├── input.rs        # InputDevice trait impl
    ├── output.rs       # OutputDevice trait impl
    └── tray.rs         # System tray icon (tray-icon crate)
```

**Naming Conventions Compliance**:
- Modules: `snake_case` (keycode.rs, hook.rs)
- Structs: `PascalCase` (WindowsKeyboardHook, EventInjector)
- Functions: `snake_case` (vk_to_keycode, inject_key_event)
- Constants: `UPPER_SNAKE_CASE` (VK_TO_KEYCODE, MAX_HOOK_TIMEOUT)

**Code Size Guidelines**:
- Each module <500 lines (enforced by pre-commit hooks)
- Each function <50 lines (SLAP principle)
- Test coverage ≥95% for platform/windows/* (per requirements)

## Code Reuse Analysis

### Existing Components to Leverage

**From keyrx_core (no_std)**:
- `KeyCode` enum: Already platform-agnostic, used for VK mapping
- `EventType`: Directly maps to WM_KEYDOWN/WM_KEYUP
- `DeviceState`: Global state shared across all devices
- `process_event()`: Core remapping logic (100% code reuse, zero changes needed)

**From keyrx_daemon/src/platform/mod.rs**:
- `InputDevice` trait: Windows implementation will implement this
- `OutputDevice` trait: Windows implementation will implement this
- `DeviceError` enum: Extended with Windows-specific errors (HookInstallFailed, InjectionFailed)

**From workspace dependencies**:
- `crossbeam_channel`: Already used for Linux, reused for hook → processor communication
- `parking_lot`: Already in workspace, used for tray icon synchronization
- `log` crate: Existing structured logging infrastructure

### Integration Points

**With keyrx_core (Platform-Agnostic)**:
- Windows platform layer calls `keyrx_core::process_event()` with converted KeyEvent
- Zero changes to core logic required
- Same .krx config format consumed on Windows and Linux

**With keyrx_daemon Main Loop**:
- Linux: evdev epoll loop on main thread
- Windows: Windows message loop on main thread (required for hooks)
- Platform abstraction hides this difference from main.rs

**With Existing Build System**:
- Cargo feature gates: `#[cfg(windows)]` and `--features windows`
- CI/CD: Extends existing `.github/workflows/` to add Windows runner
- No changes to Linux build process

**With Web Server (Optional)**:
- Windows daemon can optionally embed axum web server (same as Linux)
- `--features web` works on both platforms
- UI served on same port (http://localhost:9876)

### New Dependencies Required

**Windows-Specific (Feature-Gated)**:
- `windows-sys` (0.48+): Raw Windows API bindings (already in Cargo.toml)
- `tray-icon` (0.14+): Cross-platform tray icon (NEW dependency)

## 1. Architecture Overview

### 1.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────┐
│  Windows OS Kernel                                       │
│  └─→ Raw Input API (device enumeration)                 │
│  └─→ User32.dll (keyboard hooks, message loop)          │
└──────────────────┬───────────────────────────────────────┘
                   │ Win32 API
┌──────────────────▼───────────────────────────────────────┐
│  keyrx_daemon.exe (Windows Desktop Application)          │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Platform Layer (platform/windows.rs)              │ │
│  │  ├─→ WindowsKeyboardHook (SetWindowsHookEx)        │ │
│  │  ├─→ VirtualKeyMapper (VK_* ↔ KeyCode)            │ │
│  │  ├─→ EventInjector (SendInput)                     │ │
│  │  └─→ TrayIcon (Shell_NotifyIcon)                   │ │
│  └────────────────┬───────────────────────────────────┘ │
│                   │ Platform trait                       │
│  ┌────────────────▼───────────────────────────────────┐ │
│  │  Core Logic (keyrx_core)                           │ │
│  │  ├─→ Event Processing                              │ │
│  │  ├─→ State Management                              │ │
│  │  └─→ Tap/Hold DFA                                  │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### 1.2 Design Principles

**Principle 1: Platform Abstraction**
- Windows-specific code isolated in `platform/windows.rs`
- Implements `InputDevice` and `OutputDevice` traits
- Core logic (keyrx_core) remains platform-agnostic

**Principle 2: Minimal Dependencies**
- Use `windows` crate (official Microsoft bindings)
- Use `tray-icon` crate for cross-platform tray support
- No heavyweight frameworks (no Tauri/Electron for v0.2.0)

**Principle 3: Performance First**
- Zero-copy event handling where possible
- Hook callback does minimal work (delegate to async processor)
- No heap allocation in hot path

---

## 2. Component Design

### 2.1 WindowsKeyboardHook

**Purpose**: Install and manage Windows low-level keyboard hook.

**Responsibilities**:
1. Install hook via `SetWindowsHookEx(WH_KEYBOARD_LL, ...)`
2. Route keyboard events to callback function
3. Block original events when remapped
4. Clean up hook on exit (RAII pattern)

**API**:
```rust
pub struct WindowsKeyboardHook {
    hook_handle: HHOOK,
    event_sender: Sender<RawKeyEvent>,
}

impl WindowsKeyboardHook {
    pub fn new(event_sender: Sender<RawKeyEvent>) -> Result<Self, HookError>;
    pub fn is_installed(&self) -> bool;
}

impl Drop for WindowsKeyboardHook {
    fn drop(&mut self) {
        // Ensure UnhookWindowsHookEx called even on panic
    }
}
```

**Thread Safety**:
- Hook callback runs on main thread (required by Windows)
- Events sent to processing thread via crossbeam_channel
- Drop implementation ensures cleanup even on panic

---

### 2.2 VirtualKeyMapper

**Purpose**: Bidirectional mapping between Windows Virtual Key codes and KeyRx KeyCode enum.

**Implementation Strategy**:

**Option A: Static Arrays** (Chosen)
```rust
const VK_TO_KEYCODE: [Option<KeyCode>; 256] = [
    None,                    // 0x00 (reserved)
    Some(KeyCode::Escape),   // VK_ESCAPE = 0x1B
    Some(KeyCode::A),        // VK_A = 0x41
    // ... 256 entries
];

const KEYCODE_TO_VK: phf::Map<KeyCode, u32> = phf_map! {
    KeyCode::Escape => VK_ESCAPE,
    KeyCode::A => VK_A,
    // ... all variants
};

pub fn vk_to_keycode(vk: u32) -> Option<KeyCode> {
    VK_TO_KEYCODE.get(vk as usize).copied().flatten()
}

pub fn keycode_to_vk(kc: KeyCode) -> Option<u32> {
    KEYCODE_TO_VK.get(&kc).copied()
}
```

**Rationale**:
- ✅ O(1) lookup (array index for VK→KeyCode)
- ✅ Compile-time verification (all 256 VK codes handled)
- ✅ Zero runtime cost (data in .rodata section)

**Alternative Considered: HashMap**
- ❌ Runtime overhead (hashing)
- ❌ Heap allocation

---

### 2.3 EventInjector

**Purpose**: Inject remapped keyboard events into Windows input stream.

**API**:
```rust
pub struct EventInjector;

impl EventInjector {
    pub fn inject_key_event(
        &self,
        keycode: KeyCode,
        event_type: EventType, // Press or Release
        modifiers: ModifierState,
    ) -> Result<(), InjectionError>;
}
```

**Implementation**:
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD};

fn inject_key_event(kc: KeyCode, event_type: EventType, mods: ModifierState) -> Result<()> {
    let vk = keycode_to_vk(kc).ok_or(InjectionError::UnmappedKey)?;

    let mut input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk as u16,
                wScan: 0,
                dwFlags: if event_type == EventType::Release {
                    KEYEVENTF_KEYUP
                } else {
                    0
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }

    Ok(())
}
```

**Modifier Handling**:
- Modifiers injected as separate events before main key
- Order: Shift down → Ctrl down → Key down → Key up → Ctrl up → Shift up

---

### 2.4 TrayIcon Component

**Purpose**: System tray icon for daemon control.

**Dependencies**: `tray-icon` crate (v0.14+)

**API**:
```rust
pub struct TrayIconController {
    tray_icon: TrayIcon,
    menu_rx: Receiver<TrayMenuEvent>,
}

pub enum TrayMenuEvent {
    ReloadConfig,
    Exit,
}

impl TrayIconController {
    pub fn new() -> Result<Self, TrayError>;
    pub fn poll_events(&mut self) -> Option<TrayMenuEvent>;
}
```

**Menu Structure**:
```
┌─────────────────────┐
│ 🔄 Reload Config    │
│ ────────────────────│
│ 🚪 Exit            │
└─────────────────────┘
```

**Event Handling**:
- Tray events processed in main event loop
- Clicking "Reload Config" triggers config reload
- Clicking "Exit" initiates graceful shutdown

---

## 3. Data Structures

### 3.1 RawKeyEvent

**Purpose**: Platform-agnostic representation of Windows KBDLLHOOKSTRUCT.

```rust
#[derive(Debug, Clone)]
pub struct RawKeyEvent {
    pub vk_code: u32,           // Virtual Key code
    pub scan_code: u32,         // Hardware scan code
    pub flags: u32,             // Event flags (LLKHF_*)
    pub time: u32,              // Event timestamp
    pub is_extended: bool,      // Extended key flag (arrows, etc.)
    pub is_injected: bool,      // Event was injected by SendInput
}

impl From<&KBDLLHOOKSTRUCT> for RawKeyEvent {
    fn from(kbd: &KBDLLHOOKSTRUCT) -> Self {
        Self {
            vk_code: kbd.vkCode,
            scan_code: kbd.scanCode,
            flags: kbd.flags,
            time: kbd.time,
            is_extended: (kbd.flags & LLKHF_EXTENDED) != 0,
            is_injected: (kbd.flags & LLKHF_INJECTED) != 0,
        }
    }
}
```

**Design Decision**: Why separate RawKeyEvent from KeyEvent?
- `KBDLLHOOKSTRUCT` is Windows-specific (contains LPARAM, etc.)
- `RawKeyEvent` is platform-neutral (can be sent across threads)
- `KeyEvent` (from keyrx_core) is fully platform-agnostic

---

### 3.2 ModifierState

**Purpose**: Track state of modifier keys (Shift, Ctrl, Alt, Win).

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl ModifierState {
    pub fn from_vk_code(vk: u32, is_down: bool) -> Option<Self> {
        match vk {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => Some(Self { shift: is_down, ..Default::default() }),
            VK_CONTROL | VK_LCONTROL | VK_RCONTROL => Some(Self { ctrl: is_down, ..Default::default() }),
            VK_MENU | VK_LMENU | VK_RMENU => Some(Self { alt: is_down, ..Default::default() }),
            VK_LWIN | VK_RWIN => Some(Self { win: is_down, ..Default::default() }),
            _ => None,
        }
    }
}
```

---

## 4. Algorithms

### 4.1 Hook Callback Algorithm

**Critical Path**: Hook callback must return <50μs to avoid input lag.

**Pseudocode**:
```rust
unsafe extern "system" fn keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let kbd = *(lparam as *const KBDLLHOOKSTRUCT);
    let event_type = match wparam {
        WM_KEYDOWN | WM_SYSKEYDOWN => EventType::Press,
        WM_KEYUP | WM_SYSKEYUP => EventType::Release,
        _ => return CallNextHookEx(None, code, wparam, lparam),
    };

    // Ignore injected events (prevent infinite loop)
    if (kbd.flags & LLKHF_INJECTED) != 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    // Convert to RawKeyEvent and send to processing thread
    let raw_event = RawKeyEvent::from(&kbd);
    if let Some(sender) = HOOK_STATE.event_sender.as_ref() {
        let _ = sender.try_send(raw_event); // Non-blocking
    }

    // Block original event (we'll inject remapped version)
    return 1; // Non-zero = block event
}
```

**Performance Optimizations**:
- ✅ No heap allocation
- ✅ Minimal branching
- ✅ Non-blocking send (try_send instead of send)
- ✅ Event processing happens in separate thread

---

### 4.2 Event Processing Pipeline

**Flow**:
```
Hook Callback (main thread)
  └─→ try_send(RawKeyEvent)
       └─→ Channel (lock-free MPMC)
            └─→ Processing Thread
                 ├─→ vk_to_keycode(vk)
                 ├─→ process_event(KeyEvent)  // keyrx_core
                 └─→ For each output:
                      └─→ inject_key_event(KeyCode)
```

**Async Processing**:
```rust
async fn event_processor(
    mut event_rx: Receiver<RawKeyEvent>,
    config: Arc<DeviceConfig>,
    mut state: DeviceState,
) {
    while let Ok(raw_event) = event_rx.recv() {
        // Convert VK to KeyCode
        let Some(keycode) = vk_to_keycode(raw_event.vk_code) else {
            warn!("Unmapped VK code: {:#x}", raw_event.vk_code);
            continue;
        };

        let event_type = if (raw_event.flags & LLKHF_UP) != 0 {
            EventType::Release
        } else {
            EventType::Press
        };

        let key_event = KeyEvent::new(keycode, event_type, raw_event.time as u64);

        // Core processing (platform-agnostic)
        let outputs = keyrx_core::process_event(&key_event, &config, &mut state);

        // Inject remapped events
        for output in outputs {
            let injector = EventInjector;
            let _ = injector.inject_key_event(
                output.keycode,
                output.event_type,
                output.modifiers,
            );
        }
    }
}
```

---

### 4.3 Message Loop Integration

**Problem**: Windows hooks require a message loop to function.

**Solution**: Run Windows message loop on main thread.

```rust
fn main() -> Result<()> {
    // Install hook (requires main thread)
    let hook = WindowsKeyboardHook::new(event_tx)?;

    // Spawn processing thread
    let processor = tokio::spawn(event_processor(event_rx, config, state));

    // Run Windows message loop (blocks main thread)
    run_message_loop()?;

    // Cleanup
    drop(hook); // Calls UnhookWindowsHookEx
    processor.abort();
    Ok(())
}

fn run_message_loop() -> Result<()> {
    unsafe {
        let mut msg = MSG::default();
        while GetMessage(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }
    }
    Ok(())
}
```

---

## 5. Platform Integration

### 5.1 Trait Implementation

**InputDevice Trait**:
```rust
impl InputDevice for WindowsKeyboardInput {
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        // Events come from hook callback via channel
        match self.event_rx.recv() {
            Ok(raw_event) => {
                let keycode = vk_to_keycode(raw_event.vk_code)
                    .ok_or(DeviceError::UnmappedKey)?;
                let event_type = if (raw_event.flags & LLKHF_UP) != 0 {
                    EventType::Release
                } else {
                    EventType::Press
                };
                Ok(KeyEvent::new(keycode, event_type, raw_event.time as u64))
            }
            Err(_) => Err(DeviceError::EndOfStream),
        }
    }

    fn grab(&mut self) -> Result<(), DeviceError> {
        // Windows hooks are implicitly "grabbed" (exclusive)
        Ok(())
    }

    fn release(&mut self) -> Result<(), DeviceError> {
        // No-op (hook cleanup happens in Drop)
        Ok(())
    }
}
```

**OutputDevice Trait**:
```rust
impl OutputDevice for WindowsKeyboardOutput {
    fn send_event(&mut self, event: KeyEvent) -> Result<(), DeviceError> {
        let injector = EventInjector;
        injector.inject_key_event(event.keycode, event.event_type, ModifierState::default())
            .map_err(|e| DeviceError::InjectionFailed(e.to_string()))
    }
}
```

---

### 5.2 Cargo Features

**Cargo.toml**:
```toml
[features]
default = ["web"]
linux = ["dep:evdev", "dep:uinput", "dep:nix"]
windows = ["dep:windows", "dep:tray-icon"]
web = ["dep:axum", "dep:tower-http", "dep:tokio"]

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_UI_Shell",
]}
tray-icon = "0.14"
```

**Build Command**:
```bash
cargo build --release --target x86_64-pc-windows-msvc --features windows
```

---

## 6. Error Handling

### 6.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WindowsError {
    #[error("Failed to install keyboard hook: {0}")]
    HookInstallFailed(String),

    #[error("Failed to inject event: {0}")]
    InjectionFailed(String),

    #[error("Tray icon creation failed: {0}")]
    TrayIconFailed(String),

    #[error("Unmapped virtual key code: {0:#x}")]
    UnmappedVirtualKey(u32),
}
```

### 6.2 Error Recovery

**Hook Installation Failure**:
```rust
match WindowsKeyboardHook::new(event_tx) {
    Ok(hook) => hook,
    Err(e) => {
        eprintln!("Failed to install keyboard hook: {}", e);
        eprintln!("Possible causes:");
        eprintln!("  - Another keyboard hook is already active");
        eprintln!("  - Insufficient privileges (try running as admin)");
        std::process::exit(1);
    }
}
```

**Injection Failure**:
- Log warning but continue (don't crash daemon)
- Increment error counter for monitoring

---

## 7. Testing Strategy

### 7.1 Unit Tests

**VirtualKeyMapper Tests**:
```rust
#[test]
fn test_vk_to_keycode_all_letters() {
    for letter in b'A'..=b'Z' {
        let vk = letter as u32;
        assert!(vk_to_keycode(vk).is_some());
    }
}

#[test]
fn test_keycode_to_vk_roundtrip() {
    let kc = KeyCode::A;
    let vk = keycode_to_vk(kc).unwrap();
    let kc2 = vk_to_keycode(vk).unwrap();
    assert_eq!(kc, kc2);
}
```

**Event Injection Tests**:
```rust
#[test]
fn test_inject_simple_keypress() {
    let injector = EventInjector;
    let result = injector.inject_key_event(
        KeyCode::A,
        EventType::Press,
        ModifierState::default(),
    );
    assert!(result.is_ok());
}
```

### 7.2 Integration Tests

**Hook Lifecycle Test**:
```rust
#[test]
fn test_hook_install_and_cleanup() {
    let (tx, _rx) = crossbeam_channel::unbounded();
    let hook = WindowsKeyboardHook::new(tx).unwrap();
    assert!(hook.is_installed());
    drop(hook);
    // Verify hook uninstalled (cannot directly test, but Drop should run)
}
```

### 7.3 Manual Testing Checklist

- [ ] Install daemon, verify tray icon appears
- [ ] Press keys, verify remapping works
- [ ] Hold modifiers (Shift, Ctrl), verify preserved
- [ ] Right-click tray icon, verify menu appears
- [ ] Click "Reload Config", verify config reloaded
- [ ] Click "Exit", verify daemon exits cleanly
- [ ] Start daemon again, verify no resource leaks

---

## 8. Performance Considerations

### 8.1 Latency Budget

**Target**: <1ms end-to-end

**Breakdown**:
```
Hook callback:           <50μs   (5%)
Channel send:            <10μs   (1%)
VK→KeyCode lookup:       <5μs    (0.5%)
Core processing:         <100μs  (10%)
KeyCode→VK lookup:       <5μs    (0.5%)
SendInput:               <50μs   (5%)
OS scheduling:           <780μs  (78%)
─────────────────────────────────────
Total:                   <1000μs (100%)
```

### 8.2 Optimizations

**1. Static Dispatch**:
- Use trait objects sparingly (heap allocation + vtable overhead)
- Prefer concrete types where possible

**2. Zero-Copy Deserialization**:
- Config loaded via memory-mapped file (memmap2)
- rkyv zero-copy deserialization

**3. Lock-Free Channels**:
- crossbeam_channel for hook → processor communication
- No mutex contention

**4. Batch Event Injection**:
- SendInput accepts array (batch multiple events)
- Reduces syscall overhead

---

## 9. Security Considerations

### 9.1 Code Injection Prevention

**Config File Validation**:
- .krx files validated via rkyv checksum
- Malformed files rejected at load time

**Hook Callback Safety**:
- No user input processed in callback (only Windows-provided KBDLLHOOKSTRUCT)
- Bounds checking on VK code array access

### 9.2 Privilege Escalation

**No Admin Required**:
- Low-level hooks work in user context
- No driver installation (avoids kernel-mode code)

---

## 10. Migration Path

### 10.1 From Linux to Windows

**User Workflow**:
1. Copy .krx file from Linux to Windows
2. Run `keyrx_daemon.exe run --config my-config.krx`
3. Config works identically (platform-agnostic)

**Platform Differences**:
- Linux: systemd service
- Windows: Desktop app with tray icon
- Both: Same .krx config format

---

## 11. Future Enhancements (Out of Scope for v0.2.0)

### Deferred Features

1. **Per-Application Configs**: Window title-based config switching
2. **Gaming Mode**: Suspend hooks when game detected
3. **MSI Installer**: Professional installation experience
4. **Auto-Update**: Self-updating binary

---

## 12. References

- Windows API Documentation: https://learn.microsoft.com/en-us/windows/win32/
- tray-icon crate: https://docs.rs/tray-icon/
- windows crate: https://docs.rs/windows/
- Keyboard Hook Example: https://learn.microsoft.com/en-us/windows/win32/winmsg/using-hooks

---

**Document Status**: Ready for Review
