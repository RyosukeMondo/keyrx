# macOS Platform Support - Feasibility & Implementation Guide

**Document Status**: Research & Planning
**Last Updated**: 2026-01-20
**Author**: Architecture Team
**Target Audience**: Developers evaluating macOS platform addition

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Platform Architecture](#current-platform-architecture)
3. [Feature Parity Analysis](#feature-parity-analysis)
4. [Rust-First Implementation Strategy](#rust-first-implementation-strategy)
5. [Memory Safety & FFI Best Practices](#memory-safety--ffi-best-practices)
6. [Technical Implementation Plan](#technical-implementation-plan)
7. [Effort Estimation](#effort-estimation)
8. [Risk Assessment](#risk-assessment)
9. [Strategic Recommendations](#strategic-recommendations)
10. [Appendix: Technical References](#appendix-technical-references)

---

## Executive Summary

### Feasibility Assessment

**Verdict**: ✅ **Technically Feasible with Moderate Effort (8-12 weeks)**

macOS platform support is achievable by leveraging high-level Rust crates that provide safe abstractions over macOS system APIs. This approach **significantly reduces** Objective-C FFI complexity and memory management risks compared to direct FFI usage.

### Key Findings

| Aspect | Status | Notes |
|--------|--------|-------|
| **Technical Feasibility** | ✅ High | Mature Rust ecosystem for macOS APIs |
| **Architecture Compatibility** | ✅ Excellent | Platform trait abstraction proven with Windows/Linux |
| **Memory Safety** | ✅ Manageable | RAII-based crates eliminate manual retain/release |
| **Development Complexity** | ⚠️ Medium | Main thread requirements, Accessibility permissions |
| **Feature Parity** | ✅ 95%+ | All core features achievable; exclusive grab limited |
| **Effort Estimate** | 8-12 weeks | Using high-level Rust crates (revised down from 16-25 weeks) |

### Critical Success Factors

1. ✅ **Rust-First Libraries Available**: `rdev`, `enigo`, `tray-icon` provide safe abstractions
2. ✅ **Platform Trait Architecture**: No core changes needed
3. ⚠️ **User Permissions**: Accessibility API requires manual user approval
4. ⚠️ **Apple Ecosystem**: Code signing/notarization adds distribution complexity
5. ✅ **Testing Infrastructure**: Can extend existing CI/CD patterns

---

## Current Platform Architecture

### Design Philosophy

keyrx uses a **trait-based platform abstraction** that enables zero-cost multi-platform support:

```rust
// keyrx_daemon/src/platform/mod.rs
pub trait Platform {
    fn initialize(&mut self) -> Result<(), PlatformError>;
    fn capture_input(&mut self) -> Result<KeyEvent, PlatformError>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<(), PlatformError>;
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError>;
    fn shutdown(&mut self) -> Result<(), PlatformError>;
}
```

### Current Implementations

| Platform | Lines of Code | Primary APIs | Status |
|----------|---------------|--------------|--------|
| **Linux** | ~2,959 | evdev (`/dev/input/event*`), uinput (`/dev/uinput`) | ✅ Production |
| **Windows** | ~4,859 | Raw Input (WM_INPUT), SendInput API | ✅ Production |
| **macOS** | 0 | N/A | ❌ Not implemented |

### Platform Factory Pattern

```rust
// keyrx_daemon/src/platform/mod.rs:320-337
#[cfg(target_os = "linux")]
pub fn create_platform() -> Result<Box<dyn Platform>, PlatformError> {
    Ok(Box::new(linux::LinuxPlatform::new()))
}

#[cfg(target_os = "windows")]
pub fn create_platform() -> Result<Box<dyn Platform>, PlatformError> {
    Ok(Box::new(windows::WindowsPlatform::new()))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn create_platform() -> Result<Box<dyn Platform>, PlatformError> {
    Err(PlatformError::Unsupported {
        os: std::env::consts::OS.to_string(),
    })
}
```

**Change Required**: Add `#[cfg(target_os = "macos")]` arm to support macOS.

---

## Feature Parity Analysis

### Comprehensive Feature Comparison

| Feature | Linux | Windows | macOS | Implementation Approach |
|---------|-------|---------|-------|------------------------|
| **Keyboard Input Capture** | ✅ evdev | ✅ Raw Input | ✅ CGEventTap | `rdev` crate (safe wrapper) |
| **Output Injection** | ✅ uinput | ✅ SendInput | ✅ CGEventPost | `enigo` crate (safe wrapper) |
| **Device Enumeration** | ✅ sysfs | ✅ SetupAPI | ✅ IOKit | `iokit-sys` + manual FFI |
| **Device Metadata (VID/PID)** | ✅ Yes | ✅ Yes | ✅ Yes | USB descriptor parsing |
| **System Tray** | ✅ AppIndicator3 | ✅ tray-icon | ✅ tray-icon | Already cross-platform |
| **Multi-device Support** | ✅ Yes | ✅ Yes | ✅ Yes | No architectural barriers |
| **Exclusive Device Grab** | ✅ EVIOCGRAB | ⚠️ N/A | ⚠️ Limited | CGEventTap always shared |
| **Microsecond Timestamps** | ✅ Yes | ✅ Yes | ✅ Yes | CGEvent timestamps available |
| **<1ms Latency** | ✅ Verified | ✅ Verified | ⚠️ Needs testing | CGEventTap typically sub-ms |
| **Permission Model** | Group-based | None | ⚠️ Accessibility | User must approve in System Preferences |

### Feature Gaps & Limitations

#### 1. **Exclusive Device Grab** (⚠️ Limited on macOS)

**Linux**: Can exclusively grab device via `EVIOCGRAB` ioctl, preventing other apps from receiving events.

**macOS**: CGEventTap operates system-wide; no per-device exclusive grab. Events are always shared with the system.

**Mitigation**:
- Document as known limitation
- Suppression possible: Return `None` from CGEventTap callback to block event from reaching other apps
- Sufficient for most use cases (remapping doesn't require true exclusive access)

#### 2. **Accessibility Permission Requirement** (⚠️ UX Friction)

**Challenge**: macOS Accessibility API requires user to:
1. Open System Preferences → Security & Privacy → Accessibility
2. Unlock settings (admin password)
3. Check box next to keyrx daemon

**Mitigation**:
- Detect permission status at runtime via `AXIsProcessTrusted()`
- Display clear setup instructions if denied
- Provide automated AppleScript to open System Preferences (if possible)
- One-time setup cost (persists across launches)

**Code Pattern** (via `accessibility-sys` crate):
```rust
use accessibility_sys::accessibility::AXIsProcessTrusted;

pub fn check_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}
```

#### 3. **Main Thread Event Loop** (⚠️ Architectural Difference)

**Linux/Windows**: Input capture can run on any thread.

**macOS**: CGEventTap callbacks must be registered on the **main thread** (via CFRunLoop).

**Mitigation**:
- Spawn main thread for event loop (standard pattern)
- Use `std::thread::spawn` for auxiliary tasks
- Communication via channels (`crossbeam-channel` already in use)

---

## Rust-First Implementation Strategy

### Philosophy: Maximize Safe Rust, Minimize Unsafe FFI

**Goal**: Leverage battle-tested Rust crates to avoid writing raw Objective-C FFI code, reducing:
- Memory leak risks (manual retain/release)
- Development complexity (unsafe block minimization)
- Maintenance burden (crate authors handle macOS API changes)

### Recommended Crate Stack

| Layer | Purpose | Crate | Safety Level | Justification |
|-------|---------|-------|--------------|---------------|
| **Input Capture** | Keyboard event listening | **`rdev`** v0.5.x | 100% Safe API | Cross-platform, wraps CGEventTap internally |
| **Output Injection** | Keyboard event synthesis | **`enigo`** v0.2.x | 100% Safe API | Cross-platform, wraps CGEventPost internally |
| **System Tray** | Menu bar icon | **`tray-icon`** v0.19.x | 100% Safe API | Already used for Windows; macOS support built-in |
| **Device Enumeration** | USB device listing | **`iokit-sys`** v0.4.x + manual | FFI Required | No high-level alternative; contains unsafe blocks |
| **Foundation (if needed)** | Core Graphics helpers | **`objc2`** v0.5.x + **`objc2-core-graphics`** | RAII Memory Safety | Auto-managed via `Retained<T>` wrapper |
| **Permissions Check** | Accessibility status | **`accessibility-sys`** v0.1.x | Safe wrapper | Single function: `AXIsProcessTrusted()` |

### Architecture Diagram: Rust-First Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    keyrx_daemon (Rust)                      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            MacosPlatform (Rust struct)              │   │
│  │  - Implements Platform trait                        │   │
│  │  - Coordinates sub-components                       │   │
│  └─────────────────────────────────────────────────────┘   │
│           │              │              │              │    │
│           ▼              ▼              ▼              ▼    │
│  ┌──────────────┐ ┌───────────┐ ┌───────────┐ ┌──────────┐│
│  │ rdev (Safe)  │ │enigo(Safe)│ │tray-icon  │ │ iokit-sys││
│  │ Input Loop   │ │ Injection │ │ Menu Bar  │ │ USB Enum ││
│  └──────────────┘ └───────────┘ └───────────┘ └──────────┘│
│           │              │              │              │    │
│           ▼              ▼              ▼              ▼    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │       macOS System APIs (Objective-C/C)              │  │
│  │  - CGEventTap (Quartz)                               │  │
│  │  - CGEventPost (Quartz)                              │  │
│  │  - NSStatusBar (AppKit)                              │  │
│  │  - IOKit (USB)                                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Key Insight**: By using `rdev` and `enigo`, keyrx developers **never touch** raw CGEventTap or CGEventPost FFI—the crates handle it.

---

## Memory Safety & FFI Best Practices

### 1. RAII Pattern for Automatic Memory Management

**Problem**: Objective-C uses reference counting (`retain`/`release`). Manual management risks:
- **Use-after-free**: Accessing object after release
- **Memory leaks**: Forgetting to release retained objects
- **Double-free**: Releasing same object twice

**Solution**: Use `Retained<T>` wrapper from `objc2` crate (RAII semantics).

#### Example: Manual FFI (❌ Avoid)

```rust
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

unsafe {
    let obj: *mut Object = msg_send![class!(MyClass), new];
    // ... use obj ...
    let _: () = msg_send![obj, release];  // ❌ Easy to forget!
}
```

**Risks**:
- Compiler doesn't enforce release
- Exception before release = leak
- Double release if called twice

#### Example: RAII Wrapper (✅ Recommended)

```rust
use objc2::rc::Retained;
use objc2::{msg_send_id, ClassType};

let obj: Retained<MyClass> = unsafe {
    msg_send_id![MyClass::class(), new]  // Auto-retained
};
// ... use obj ...
// obj.release() called automatically via Drop trait
```

**Safety Guarantees**:
- ✅ Compiler enforces single owner (Rust ownership rules)
- ✅ Automatic release on scope exit (Drop impl)
- ✅ Cannot access after move (borrow checker)
- ✅ Exception-safe (Drop runs during unwinding)

### 2. Thread Safety Markers

**Problem**: Some macOS APIs (e.g., AppKit) must only be called from the main thread.

**Solution**: `objc2` provides compile-time enforcement via `MainThreadOnly<T>` wrapper.

```rust
use objc2::MainThreadOnly;
use objc2_app_kit::NSApplication;

// This type can ONLY be used on the main thread
let app: MainThreadOnly<NSApplication> = MainThreadOnly::new(
    NSApplication::sharedApplication()
);

// Compiler error if accessed from worker thread ✅
std::thread::spawn(|| {
    app.terminate();  // ❌ Compile error: NSApplication not Send
});
```

### 3. Minimize Unsafe Blocks

**Strategy**: Limit unsafe code to crate boundaries; keep application logic safe.

| Code Location | Unsafe % | Reason |
|---------------|----------|--------|
| `rdev` internals | ~20% | CGEventTap callback registration |
| `enigo` internals | ~15% | CGEventPost FFI |
| **keyrx macOS platform code** | **<5% target** | Only IOKit device enumeration |
| keyrx core logic | 0% | Platform-agnostic (no_std) |

**Audit Pattern**:
```bash
# Find all unsafe blocks in macOS platform code
rg "unsafe" keyrx_daemon/src/platform/macos/ --count-matches

# Target: <10 occurrences (most in device_discovery.rs)
```

### 4. Error Propagation (No Silent Failures)

**Anti-pattern**: Silent failure if Accessibility permission denied (rdev does this).

**Best Practice**: Explicit permission check with actionable error.

```rust
use accessibility_sys::accessibility::AXIsProcessTrusted;

pub fn initialize(&mut self) -> Result<(), PlatformError> {
    if !unsafe { AXIsProcessTrusted() } {
        return Err(PlatformError::PermissionDenied {
            message: "Accessibility permission required. \
                      Open System Preferences → Security & Privacy → Accessibility \
                      and enable keyrx.".to_string(),
        });
    }

    // Proceed with CGEventTap setup
    Ok(())
}
```

### 5. Memory Leak Detection

**Tool**: Xcode Instruments (Allocations, Leaks)

**Process**:
1. Build keyrx daemon with debug symbols (`cargo build --release`)
2. Run via Instruments: `instruments -t Leaks ./target/release/keyrx_daemon`
3. Perform event capture/injection cycles
4. Check for leaked allocations

**Acceptance Criteria**: Zero leaks after 10,000 key events.

---

## Technical Implementation Plan

### Proposed File Structure

```
keyrx_daemon/src/platform/
├── mod.rs                     # Platform trait & factory (add macOS arm)
├── common.rs                  # Shared types (unchanged)
├── linux/                     # Existing
├── windows/                   # Existing
├── macos/                     # NEW
│   ├── mod.rs                # MacosPlatform struct
│   ├── input_capture.rs      # rdev-based input wrapper
│   ├── output_injection.rs   # enigo-based output wrapper
│   ├── device_discovery.rs   # IOKit USB enumeration (unsafe blocks)
│   ├── keycode_map.rs        # CGKeyCode ↔ KeyRx KeyCode mapping
│   ├── tray.rs               # tray-icon integration (reuse Windows pattern)
│   └── permissions.rs        # Accessibility permission checks
└── mock.rs                   # Existing (testing)
```

**Estimated Lines of Code**: ~1,200-1,500 (vs. 2,959 for Linux, 4,859 for Windows)
**Reason**: High-level crates eliminate boilerplate.

### Implementation Phases

#### Phase 1: Foundation (Week 1-2)

**Goals**:
- Set up development environment (macOS hardware or VM)
- Create `macos/` module structure
- Implement `MacosPlatform` skeleton

**Deliverables**:
- `keyrx_daemon/src/platform/macos/mod.rs` compiles
- `create_platform()` recognizes `target_os = "macos"`
- Basic error types defined

**Acceptance Criteria**:
```bash
cargo build --target x86_64-apple-darwin  # Succeeds
```

#### Phase 2: Input Capture (Week 3-4)

**Goals**:
- Integrate `rdev` for keyboard event listening
- Implement `capture_input()` method
- Map `rdev::Event` to `KeyEvent`

**Key Implementation**:
```rust
// keyrx_daemon/src/platform/macos/input_capture.rs
use rdev::{listen, Event, EventType, Key};
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
        EventType::KeyPress(key) => Some(KeyEvent::Press(map_key(key))),
        EventType::KeyRelease(key) => Some(KeyEvent::Release(map_key(key))),
        _ => None,
    }
}
```

**Testing**:
- Integration test: Simulate key presses, verify events received
- Performance test: Measure end-to-end latency (target <1ms)

#### Phase 3: Output Injection (Week 5)

**Goals**:
- Integrate `enigo` for keyboard event injection
- Implement `inject_output()` method
- Test with remapped events

**Key Implementation**:
```rust
// keyrx_daemon/src/platform/macos/output_injection.rs
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
            KeyEvent::Press(k) => (map_keycode(k), Direction::Press),
            KeyEvent::Release(k) => (map_keycode(k), Direction::Release),
        };

        self.enigo.key(key, direction)
            .map_err(|e| PlatformError::OutputDeviceError {
                message: format!("Injection failed: {}", e),
            })
    }
}
```

#### Phase 4: Device Enumeration (Week 6-7)

**Goals**:
- Enumerate USB keyboard devices via IOKit
- Extract VID/PID/serial number
- Implement `list_devices()` method

**Challenges**: Only area requiring significant unsafe FFI.

**Key Implementation** (abbreviated):
```rust
// keyrx_daemon/src/platform/macos/device_discovery.rs
use iokit_sys::*;
use core_foundation::base::TCFType;

pub fn list_keyboard_devices() -> Result<Vec<DeviceInfo>, PlatformError> {
    unsafe {
        // Create IOKit matching dictionary for USB keyboards
        let matching = IOServiceMatching(kIOUSBDeviceClassName);

        // Get matching services
        let mut iterator: io_iterator_t = 0;
        let result = IOServiceGetMatchingServices(
            kIOMainPortDefault,
            matching,
            &mut iterator
        );

        if result != KERN_SUCCESS {
            return Err(PlatformError::DeviceEnumerationError);
        }

        let mut devices = Vec::new();
        loop {
            let device = IOIteratorNext(iterator);
            if device == 0 { break; }

            // Extract properties (VID, PID, serial)
            if let Some(info) = extract_device_info(device) {
                devices.push(info);
            }

            IOObjectRelease(device);
        }

        IOObjectRelease(iterator);
        Ok(devices)
    }
}
```

**Safety Audit**:
- Review all unsafe blocks with checklist
- Ensure proper resource cleanup (IOObjectRelease)
- Test with various USB devices

#### Phase 5: System Tray Integration (Week 8)

**Goals**:
- Reuse `tray-icon` crate (already used for Windows)
- Implement menu items: Reload Config, Exit
- Handle menu events

**Key Insight**: Minimal work—`tray-icon` is cross-platform.

```rust
// keyrx_daemon/src/platform/macos/tray.rs
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};

pub struct MacosSystemTray {
    _tray: TrayIcon,
    menu_rx: Receiver<MenuEvent>,
}

impl SystemTray for MacosSystemTray {
    // Same implementation as Windows version
    // tray-icon handles macOS-specific details
}
```

#### Phase 6: Keycode Mapping (Week 9)

**Goals**:
- Create bidirectional map: macOS CGKeyCode ↔ keyrx KeyCode
- Cover 100+ keys (A-Z, 0-9, modifiers, function keys, special keys)
- Comprehensive unit tests

**Reference**: [macOS Virtual Key Codes](https://developer.apple.com/documentation/appkit/nsevent/specialkey)

```rust
// keyrx_daemon/src/platform/macos/keycode_map.rs
pub fn cgkeycode_to_keyrx(code: u16) -> Option<KeyCode> {
    match code {
        0x00 => Some(KeyCode::A),
        0x01 => Some(KeyCode::S),
        // ... 100+ mappings
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_letter_mapping() {
        assert_eq!(cgkeycode_to_keyrx(0x00), Some(KeyCode::A));
        // ... exhaustive tests
    }
}
```

#### Phase 7: Testing & CI/CD (Week 10-11)

**Goals**:
- Add macOS to GitHub Actions matrix
- Set up VM or self-hosted runner
- Integration tests for full event capture → remap → injection flow

**CI Configuration**:
```yaml
# .github/workflows/ci.yml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]  # Add macOS
        rust: [stable]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Build
        run: cargo build --workspace
      - name: Test
        run: cargo test --workspace
      # macOS-specific: Skip integration tests requiring Accessibility permission
      - name: Integration Tests (non-macOS)
        if: matrix.os != 'macos-latest'
        run: cargo test --test integration
```

#### Phase 8: Documentation & Code Signing (Week 12)

**Goals**:
- Document macOS setup process (Accessibility permissions)
- Set up code signing for distribution
- Update user guides

**Code Signing Workflow**:
1. Obtain Apple Developer account ($99/year)
2. Generate Developer ID Application certificate
3. Sign binary: `codesign --sign "Developer ID Application" keyrx_daemon`
4. Notarize: `xcrun notarytool submit keyrx_daemon.zip`
5. Automate in CI/CD via secrets

---

## Effort Estimation

### Revised Timeline (Rust-First Approach)

| Phase | Duration | Deliverables | Risk Level |
|-------|----------|--------------|------------|
| 1. Foundation | 2 weeks | Module structure, skeleton | Low |
| 2. Input Capture | 2 weeks | `rdev` integration, event mapping | Low |
| 3. Output Injection | 1 week | `enigo` integration | Low |
| 4. Device Enumeration | 2 weeks | IOKit USB discovery (unsafe FFI) | Medium |
| 5. System Tray | 1 week | `tray-icon` reuse | Low |
| 6. Keycode Mapping | 1 week | CGKeyCode ↔ KeyRx mapping | Low |
| 7. Testing & CI/CD | 2 weeks | Automated tests, GitHub Actions | Medium |
| 8. Documentation & Signing | 1 week | User guides, code signing | Low |
| **TOTAL** | **12 weeks** | Production-ready macOS support | **Low-Medium** |

**Comparison**:
- **Previous Estimate** (raw FFI): 16-25 weeks
- **Rust-First Estimate**: 12 weeks
- **Reduction**: ~40-50% due to safe crate abstractions

**Assumptions**:
- Team has Rust experience (no learning curve)
- Access to macOS hardware for testing
- No major architectural surprises in `rdev`/`enigo`
- Apple developer account already provisioned

**Optimistic Scenario**: 8 weeks (if `rdev`/`enigo` integration trivial)
**Pessimistic Scenario**: 14 weeks (if IOKit enumeration requires debugging)

### Resource Requirements

| Resource | Cost | Notes |
|----------|------|-------|
| **Development Time** | 12 weeks | 1 senior Rust developer |
| **macOS Hardware** | $1,000-$2,000 | Mac Mini or MacBook (Intel or Apple Silicon) |
| **Apple Developer Account** | $99/year | Required for code signing/notarization |
| **CI/CD Runner** | $0-$200/month | GitHub Actions macOS minutes or self-hosted |
| **TOTAL (Year 1)** | ~$50K-$80K | Assuming $150K/year developer salary |

---

## Risk Assessment

### High-Level Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **rdev latency exceeds 1ms** | Low | High | Benchmark in Week 3; fallback to `core-graphics` if needed |
| **Accessibility permission UX friction** | High | Medium | Clear documentation, automated setup script |
| **IOKit FFI complexity** | Medium | Medium | Allocate 2 weeks; seek community examples |
| **Code signing workflow issues** | Medium | Low | Test early (Week 8); automate with CI/CD |
| **Apple API changes** | Low | Medium | Pin crate versions; monitor release notes |
| **Community adoption** | Medium | Low | Market research before committing |

### Detailed Risk Analysis

#### Risk 1: Event Capture Latency

**Description**: CGEventTap may introduce latency >1ms, degrading user experience.

**Mitigation Strategy**:
1. **Week 3 Benchmark**: Measure `rdev` latency on real hardware
   - Acceptable: <1ms (proceed with `rdev`)
   - Unacceptable: 1-5ms (fallback to raw `core-graphics` for optimization)
2. **Fallback Plan**: Use `core-graphics` directly with manual CFRunLoop integration
   - Adds 2-3 weeks to timeline
   - Requires more unsafe code (increases maintenance burden)

**Acceptance Criteria**: 95th percentile latency <1ms under load.

#### Risk 2: Memory Leaks in FFI Code

**Description**: Manual FFI in IOKit device enumeration could leak memory.

**Mitigation Strategy**:
1. Use Xcode Instruments to detect leaks during development
2. Comprehensive code review of all unsafe blocks
3. RAII wrappers for IOKit objects where possible
4. Automated leak testing in CI/CD (if feasible)

**Acceptance Criteria**: Zero leaks in Instruments after 10,000 events.

#### Risk 3: Accessibility Permission Confusion

**Description**: Users may not understand how to grant Accessibility permission.

**Mitigation Strategy**:
1. Detect permission status at launch (`AXIsProcessTrusted()`)
2. Display actionable error message with step-by-step instructions
3. Optionally: AppleScript to auto-open System Preferences
4. Link to video tutorial in documentation

**Example Error Message**:
```
Error: Accessibility permission required

keyrx needs Accessibility permission to intercept keyboard events.

To grant permission:
1. Open System Preferences
2. Go to Security & Privacy → Privacy → Accessibility
3. Click the lock icon and enter your password
4. Check the box next to "keyrx"
5. Restart keyrx

For help, visit: https://docs.keyrx.io/setup/macos
```

#### Risk 4: Apple Silicon Compatibility

**Description**: Implementation may work on Intel Macs but fail on Apple Silicon (ARM64).

**Mitigation Strategy**:
1. Test on both architectures (GitHub Actions supports both)
2. Use `#[cfg(target_arch)]` conditionally if needed
3. Universal binary creation: `cargo build --target universal-apple-darwin`

**Acceptance Criteria**: Tests pass on both Intel and ARM64 runners.

---

## Strategic Recommendations

### Option A: Implement macOS Support (Recommended If...)

**Pursue if ANY of these conditions apply**:

1. **User Demand**: Community requests exceed 50 votes (GitHub issues, Discord)
2. **Market Opportunity**: macOS/Linux dual-boot users are target demographic
3. **Competitive Advantage**: Differentiate from Karabiner-Elements with cross-platform configs
4. **Team Capacity**: Dedicated developer can allocate 12 consecutive weeks

**Benefits**:
- Expand user base (~15% desktop market share)
- Strengthen cross-platform story (Windows + Linux + macOS)
- Leverage existing `tray-icon` investment
- Minimal architecture changes (trait-based design pays off)

**Timeline**: 12 weeks to production-ready release.

### Option B: Defer macOS Support (Recommended for MVP)

**Defer if MOST of these conditions apply**:

1. **Low Demand**: No user requests for macOS support
2. **Resource Constraints**: Team focused on core features (gaming mode, macros)
3. **Market Saturation**: Karabiner-Elements already dominant on macOS
4. **ROI Concerns**: Windows + Linux sufficient for target users (product roadmap states this)

**Benefits**:
- Focus on high-value features (per-app configs, advanced macros)
- Avoid 12-week investment with uncertain return
- Leverage Linux/Windows as reference implementations if reconsidered later

**Action Items**:
- Document architecture for future implementers
- Keep trait abstraction clean (no hardcoded platform assumptions)
- Revisit if user demand materializes

### Option C: Community-Driven Implementation

**Best of both worlds**:

1. **Document Thoroughly**: Create this guide + architecture diagrams
2. **Mentor Contributors**: Provide PR review/guidance if community tackles it
3. **Accept PRs**: Merge if quality meets standards (tests, safety, documentation)
4. **Low Risk**: No core team commitment unless community delivers

**Requirements for Acceptance**:
- ✅ Implements `Platform` trait completely
- ✅ Achieves ≥80% test coverage
- ✅ Passes CI/CD on macOS runners
- ✅ Documentation includes setup guide
- ✅ No unsafe code in application logic (only IOKit FFI)

---

## Appendix: Technical References

### Recommended Crates

| Crate | Version | Purpose | Documentation |
|-------|---------|---------|---------------|
| **rdev** | 0.5.3 | Cross-platform event capture | [docs.rs/rdev](https://docs.rs/rdev/) |
| **enigo** | 0.2.x | Cross-platform event injection | [docs.rs/enigo](https://docs.rs/enigo/) |
| **tray-icon** | 0.19.x | System tray (Windows/macOS/Linux) | [docs.rs/tray-icon](https://docs.rs/tray-icon/) |
| **iokit-sys** | 0.4.x | IOKit FFI bindings (USB enumeration) | [docs.rs/iokit-sys](https://docs.rs/iokit-sys/) |
| **objc2** | 0.5.x | Modern Objective-C FFI with RAII | [docs.rs/objc2](https://docs.rs/objc2/) |
| **objc2-core-graphics** | 0.2.x | Core Graphics safe bindings | [docs.rs/objc2-core-graphics](https://docs.rs/objc2-core-graphics/) |
| **accessibility-sys** | 0.1.x | Accessibility API (permission checks) | [docs.rs/accessibility-sys](https://docs.rs/accessibility-sys/) |

### Apple Documentation

- [CGEventTap Reference](https://developer.apple.com/documentation/coregraphics/1454426-cgeventtapcreate)
- [CGEventPost Reference](https://developer.apple.com/documentation/coregraphics/1456527-cgeventpost)
- [IOKit Fundamentals](https://developer.apple.com/library/archive/documentation/DeviceDrivers/Conceptual/IOKitFundamentals/)
- [Accessibility API Guide](https://developer.apple.com/documentation/accessibility)

### Existing Rust Projects (Reference Implementations)

- **Karabiner-Elements** (C++): https://github.com/pqrs-org/Karabiner-Elements
- **rdev** (Rust): https://github.com/Narsil/rdev
- **enigo** (Rust): https://github.com/enigo-rs/enigo
- **Tauri macOS Input Monitor**: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/macos-input-monitor

### Development Environment Setup

#### Hardware Requirements

- **Intel Mac** or **Apple Silicon Mac** (M1/M2/M3)
- macOS 12 (Monterey) or later
- 8GB+ RAM, 10GB+ free disk space

#### Software Requirements

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add macOS target (if cross-compiling from Linux)
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Install keyrx dependencies
cd keyrx
make setup  # Existing script works on macOS
```

#### Permission Grant (One-Time Setup)

```bash
# Build keyrx daemon
cargo build --release --bin keyrx_daemon

# Run once to trigger permission prompt
./target/release/keyrx_daemon

# macOS will show permission denial alert
# Follow on-screen instructions to grant Accessibility permission

# Verify permission granted
./scripts/check_macos_permissions.sh  # To be created
```

### Testing Strategy

#### Unit Tests
```bash
cargo test -p keyrx_daemon --lib macos
```

#### Integration Tests (Requires Accessibility Permission)
```bash
# Skip on CI/CD if permission not available
cargo test -p keyrx_daemon --test macos_integration
```

#### Manual Testing Checklist

- [ ] Event capture: Press A-Z keys, verify events logged
- [ ] Event injection: Remap A→B, verify B appears in text editor
- [ ] Multi-device: Connect USB keyboard, verify enumeration
- [ ] System tray: Right-click menu, verify Reload/Exit work
- [ ] Latency: Measure end-to-end delay (<1ms target)
- [ ] Memory: Run Instruments, verify no leaks after 10,000 events
- [ ] Permissions: Deny Accessibility, verify graceful error message

### Code Review Checklist

Before merging macOS platform code:

- [ ] All unsafe blocks documented with safety invariants
- [ ] No manual `retain`/`release` (use `Retained<T>` wrappers)
- [ ] Error messages actionable (e.g., permission instructions)
- [ ] Tests achieve ≥80% coverage
- [ ] No clippy warnings (`cargo clippy --all-targets`)
- [ ] Formatted (`cargo fmt --check`)
- [ ] Documentation updated (README, setup guides)
- [ ] CI/CD passes on macOS runners

---

## Conclusion

macOS platform support is **highly feasible** with a **Rust-first approach** leveraging mature ecosystem crates (`rdev`, `enigo`, `tray-icon`). This strategy:

✅ **Reduces complexity**: 40-50% less development time vs. raw FFI
✅ **Enhances safety**: RAII memory management eliminates manual retain/release
✅ **Improves maintainability**: Crate authors handle macOS API changes
✅ **Preserves architecture**: No changes to `Platform` trait or core logic

**Recommended Path Forward**:
1. **If pursuing**: Allocate 12 weeks, follow phased implementation plan
2. **If deferring**: Document architecture for future revisit or community contribution
3. **If uncertain**: Prototype Phase 1-2 (4 weeks) to validate `rdev` latency before full commitment

**Decision Point**: Evaluate user demand and team capacity to determine which recommendation aligns with product strategy.

---

**Document Version**: 1.0
**Maintainer**: Architecture Team
**Next Review**: After user demand assessment or before macOS implementation kickoff
