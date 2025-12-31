# Windows Quality Improvements - Design

## Overview

This document outlines the design approach for systematically finding and fixing Windows-specific bugs in the keyrx daemon, mirroring the successful Linux bug hunt methodology.

## Design Principles

### 1. Systematic Investigation
- **Code Review First**: Read all Windows-specific code paths before writing tests
- **Pattern Matching**: Look for common bug patterns (unwrap, unsafe, lock misuse)
- **Severity Classification**: CRITICAL > HIGH > MEDIUM > LOW
- **Test-Driven Fixes**: Write regression tests before fixing bugs

### 2. Windows-Specific Focus
- **Unique Architecture**: Raw Input API, message pump, raw pointers
- **Platform Constraints**: Cannot use Linux testing approaches (evdev, uinput)
- **Windows API Quirks**: Error handling, threading model, resource management

### 3. Quality Bar
- **Match Linux Quality**: Achieve same robustness as Linux implementation
- **Comprehensive Testing**: Tests for all bugs found
- **Documentation**: Windows-specific architectural decisions documented

## Investigation Strategy

### Phase 1: Code Audit (2-3 hours)

#### 1.1 Memory Safety Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/rawinput.rs`
- Focus: Lines 52-60 (Box::into_raw), 177-187 (Drop)

**Investigation Approach**:
```rust
// Look for:
1. Raw pointer creation: Box::into_raw()
2. Pointer reconstruction in Drop: Box::from_raw()
3. Thread safety of raw pointer access
4. Lifetime mismatches between HWND and Rust ownership
```

**Bug Patterns to Find**:
- Use-after-free if window destroyed while wnd_proc executing
- Double-free if Drop called multiple times
- Dangling pointers if window handle outlives Rust object

#### 1.2 RwLock Poisoning Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/device_map.rs`
- `keyrx_daemon/src/platform/windows/rawinput.rs`

**Investigation Approach**:
```rust
// Look for:
1. .write().unwrap() - panics on poisoned lock
2. .read().unwrap() - panics on poisoned lock
3. Panic handlers in lock-holding code
4. Lock acquisition in Windows message callbacks
```

**Bug Patterns to Find**:
- Cascade failures if one thread poisons lock
- Panic in wnd_proc crashes daemon (no recovery)
- No fallback error handling

#### 1.3 Message Queue Audit
**Target Files**:
- `keyrx_daemon/src/main.rs:185-220` (message pump)
- `keyrx_daemon/src/platform/windows/rawinput.rs:214-230` (GetRawInputData)

**Investigation Approach**:
```rust
// Look for:
1. Unbounded memory allocation in GetRawInputData
2. No rate limiting on keyboard events
3. No timeout/watchdog for stuck messages
4. Lost events if queue fills up
```

**Bug Patterns to Find**:
- Memory exhaustion under high event rate
- Deadlock if message queue blocks

#### 1.4 Error Recovery Audit
**Target Files**:
- `keyrx_daemon/src/main.rs` (main loop)
- `keyrx_daemon/src/platform/windows/mod.rs:43-52` (init)

**Investigation Approach**:
```rust
// Look for:
1. Unhandled panics in DispatchMessageW
2. No restart mechanism on crash
3. Partial cleanup on init() failure
4. No recovery from Windows API errors
```

#### 1.5 Device Hotplug Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:243-260` (WM_INPUT_DEVICE_CHANGE)

**Investigation Approach**:
```rust
// Look for:
1. let _ = add_device() - silent failures
2. No config reload on device add
3. Race condition: device removed during event processing
4. Stale device references after removal
```

#### 1.6 Windows API Error Handling Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/rawinput.rs` (RegisterClassExW, CreateWindowExW)
- `keyrx_daemon/src/platform/windows/inject.rs` (SendInput)

**Investigation Approach**:
```rust
// Look for:
1. Missing GetLastError() calls
2. No validation of Windows API return values
3. Silent failures in SendInput
4. Swallowed errors in window creation
```

#### 1.7 Scancode Mapping Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/keycode.rs:161-176`
- `keyrx_daemon/src/platform/windows/inject.rs:49-65`

**Investigation Approach**:
```rust
// Look for:
1. MapVirtualKeyW layout-dependency (placeholder note)
2. Lost information in scancode → VK → KeyCode conversion
3. Hardcoded extended key list (incomplete)
4. IME/special keys not handled
```

#### 1.8 Resource Cleanup Audit
**Target Files**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:177-187` (Drop)
- `keyrx_daemon/src/platform/windows/mod.rs`

**Investigation Approach**:
```rust
// Look for:
1. Window handle leaks on error paths
2. Raw Input registration not unregistered
3. Channel leaks if receiver dropped
4. Memory allocated in GetRawInputData not freed
```

### Phase 2: Test Writing (3-4 hours)

#### 2.1 Test Infrastructure

**Create Windows-specific test utilities**:
```rust
// keyrx_daemon/tests/windows/mod.rs
pub mod utils {
    pub struct VirtualWindowsKeyboard {
        // Simulate keyboard input via SendInput
    }

    pub fn create_test_config_windows() -> TestConfig {
        // Windows-specific test config
    }

    pub fn inject_keyboard_event(key: VirtualKey, press: bool) -> Result<()> {
        // Use SendInput for testing
    }
}
```

#### 2.2 Memory Safety Tests

**Test 1: Use-After-Free**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_rawinput_manager_drop_safety() {
    // Create RawInputManager
    // Drop it while simulating wnd_proc callback
    // Verify no crash or undefined behavior
}
```

**Test 2: Double-Free**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_window_external_destruction() {
    // Create RawInputManager
    // Externally destroy window with DestroyWindow
    // Verify Drop doesn't cause double-free
}
```

#### 2.3 RwLock Tests

**Test 3: Lock Poisoning Cascade**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_rwlock_poison_recovery() {
    // Deliberately panic thread holding write lock
    // Verify subsequent lock attempts don't cascade
    // Verify daemon continues operating
}
```

**Test 4: Panic in wnd_proc**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_wndproc_panic_handling() {
    // Trigger panic in wnd_proc callback
    // Verify daemon doesn't crash
    // Verify error is logged
}
```

#### 2.4 Message Queue Tests

**Test 5: High Event Rate**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_message_queue_flood() {
    // Flood message queue with keyboard events
    // Verify no memory exhaustion
    // Verify no lost events
}
```

**Test 6: GetRawInputData Bounds**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_rawinput_buffer_allocation() {
    // Send large raw input data
    // Verify bounded memory allocation
    // Verify proper buffer cleanup
}
```

#### 2.5 Error Recovery Tests

**Test 7: Windows API Failure Recovery**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_api_error_recovery() {
    // Simulate Windows API failures (mocking)
    // Verify daemon logs errors
    // Verify daemon continues operating
}
```

#### 2.6 Device Hotplug Tests

**Test 8: Device Hot-Unplug**
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_device_hotplug_handling() {
    // Simulate WM_INPUT_DEVICE_CHANGE
    // Verify no panic on device removal
    // Verify device list updated correctly
}
```

**Test 9: Device Add Silent Failure**
```rust
#[test]
fn test_device_add_error_logging() {
    // Read source code for WM_INPUT_DEVICE_CHANGE handler
    // Verify it doesn't use `let _ = add_device()`
    // Verify errors are logged
}
```

### Phase 3: Bug Fixing (2-3 hours)

#### Priority Order:
1. **CRITICAL**: Memory safety issues (use-after-free, double-free)
2. **HIGH**: RwLock poisoning, device hot-unplug crashes
3. **MEDIUM**: Message queue overflow, error recovery
4. **LOW**: Logging, scancode mapping improvements

#### Fix Strategy:
```rust
// Pattern: Replace unsafe code with safe abstractions
// BEFORE:
let ptr = Box::into_raw(Box::new(manager));
SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

// AFTER (safer):
use std::sync::Arc;
let manager = Arc::new(Mutex::new(manager));
let ptr = Arc::into_raw(manager);
SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
// Ensure Arc::from_raw called in Drop
```

```rust
// Pattern: Replace unwrap with error handling
// BEFORE:
self.device_map.write().unwrap().add_device(info);

// AFTER:
match self.device_map.write() {
    Ok(mut map) => map.add_device(info),
    Err(e) => {
        error!("RwLock poisoned: {:?}", e);
        // Attempt recovery or controlled shutdown
    }
}
```

```rust
// Pattern: Add bounds checking
// BEFORE:
let size = GetRawInputData(...);
let buffer = vec![0u8; size as usize]; // Unbounded allocation

// AFTER:
const MAX_RAWINPUT_SIZE: u32 = 4096; // Reasonable limit
let size = GetRawInputData(...).min(MAX_RAWINPUT_SIZE);
let buffer = vec![0u8; size as usize];
```

## Testing Challenges

### Windows-Specific Constraints

**1. No Virtual Device API**
- **Problem**: Windows has no equivalent to Linux uinput
- **Solution**: Use SendInput for simulated keyboard events
- **Limitation**: Cannot test device enumeration/grab logic

**2. Admin Privileges Required**
- **Problem**: Raw Input API requires admin mode for full functionality
- **Solution**: Document manual testing steps, run tests as admin in CI

**3. Message Pump Complexity**
- **Problem**: Hard to simulate Windows message queue behavior
- **Solution**: Use unit tests for components, integration tests for full daemon

**4. Thread Safety Testing**
- **Problem**: Difficult to deterministically trigger race conditions
- **Solution**: Use stress tests with high concurrency

### Test Organization

```
keyrx_daemon/tests/
├── windows/
│   ├── mod.rs                    # Windows-specific test utilities
│   ├── memory_safety_tests.rs   # Use-after-free, double-free tests
│   ├── rwlock_tests.rs           # Lock poisoning, panic handling
│   ├── message_queue_tests.rs   # High event rate, buffer bounds
│   ├── error_recovery_tests.rs  # API failure handling
│   ├── device_hotplug_tests.rs  # WM_INPUT_DEVICE_CHANGE tests
│   └── code_inspection_tests.rs # Source code verification tests
└── common/
    └── regression_tests.rs       # Cross-platform regression tests
```

## Architecture Improvements (Stretch Goals)

### Reduce Raw Pointer Usage
**Current**:
```rust
let ptr = Box::into_raw(Box::new(manager));
SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
```

**Improved**:
```rust
use std::sync::Arc;

pub struct WindowContext {
    manager: Arc<Mutex<RawInputManager>>,
}

// Store Arc in window data, automatic cleanup via Drop
```

### Replace RwLock with Better Sync Primitive
**Current**: RwLock with .unwrap() - panic cascades

**Options**:
1. **Mutex**: Simpler, no read/write distinction needed
2. **Arc<Mutex>**: Better for shared ownership
3. **lock-free structures**: For high-performance paths

**Recommendation**: Use `Arc<Mutex<T>>` for device_map - simpler and more robust

### Add Windows-Specific Documentation
**Create**: `keyrx_daemon/docs/windows_architecture.md`

**Contents**:
- Raw Input API overview
- Message pump vs poll() comparison
- Memory management strategy
- Thread safety guarantees
- Testing limitations

## Success Metrics

### Minimum (Must Achieve)
- ✅ Find at least 5 Windows-specific bugs
- ✅ All bugs categorized by severity
- ✅ Tests written for all bugs
- ✅ Root causes documented

### Target (Goal)
- ✅ Find 8+ Windows-specific bugs
- ✅ Fix all CRITICAL and HIGH bugs
- ✅ Achieve same quality bar as Linux
- ✅ Windows-specific E2E tests created

### Stretch (Nice to Have)
- ✅ Reduce raw pointer usage (safer abstractions)
- ✅ Windows architecture documentation
- ✅ Windows CI/CD testing pipeline

## Comparison: Windows vs Linux Bug Fixes

### Common Fixes (Benefit Both Platforms)
| Bug | Linux Fix | Windows Benefit |
|-----|-----------|-----------------|
| BUG #38 (Config reload logging) | ✅ Fixed | ✅ Fixed (same code path) |
| BUG #37 (Device hot-unplug panic) | ✅ Fixed | ✅ Fixed (same code path) |

### Platform-Specific Fixes Needed
| Area | Linux Solution | Windows Needs |
|------|----------------|---------------|
| Shutdown | Signal handlers (SIGTERM) | WM_QUIT message handling |
| Event loop | poll() with backoff | Message pump timeout/watchdog |
| Error flags | POLLERR/POLLHUP detection | Windows API error codes |
| Device grab | EVIOCGRAB ioctl rollback | N/A (no exclusive grab) |

## Design Decisions

### Decision 1: Keep Raw Input API
**Question**: Should we switch to Low-Level Hooks API?

**Decision**: **NO** - Keep Raw Input API

**Rationale**:
- Raw Input is more performant (fewer callbacks)
- Works in admin and non-admin modes
- Architectural change too large for quality improvement

**Trade-off**: Accept implicit global interception (no per-device grab)

### Decision 2: Keep Message Pump Architecture
**Question**: Should we use a separate event loop thread?

**Decision**: **NO** - Keep message pump on main thread

**Rationale**:
- Windows message pump requires main thread
- Threading adds complexity without clear benefit
- Focus on fixing bugs in existing architecture

**Trade-off**: Accept message pump blocking constraints

### Decision 3: Use Arc<Mutex> for Shared State
**Question**: How to handle RwLock poisoning?

**Decision**: Replace RwLock with Arc<Mutex> for device_map

**Rationale**:
- Simpler error handling (no poisoning)
- Performance difference negligible for device map
- Better thread safety guarantees

**Trade-off**: Lose read/write optimization (acceptable for device_map)

### Decision 4: Manual Windows Testing
**Question**: Should we set up Windows CI runners?

**Decision**: **DEFER** - Manual testing on Windows PC first, CI later

**Rationale**:
- Windows CI setup is complex (admin rights, drivers)
- Manual testing sufficient for initial quality improvements
- CI can be added after bugs fixed

**Trade-off**: Slower test feedback, but more pragmatic

## Open Questions

1. **Raw pointer safety**: Can we eliminate raw pointers entirely without major refactoring?
   - **Answer TBD**: Investigate Arc-based window context design

2. **RwLock vs Mutex**: Is read performance critical for device_map?
   - **Answer TBD**: Benchmark both approaches

3. **Headless mode**: Should daemon work without tray icon?
   - **Answer TBD**: Check user requirements

4. **Message queue testing**: How to reliably test overflow conditions?
   - **Answer TBD**: Research Windows test frameworks

5. **Windows CI**: When to invest in Windows CI/CD?
   - **Answer TBD**: After manual testing phase complete

## References

- Linux Bug Hunt Results: `/tmp/bug-fixes-complete.md`
- Windows Raw Input API: https://docs.microsoft.com/en-us/windows/win32/inputdev/raw-input
- Rust Unsafe Code Guidelines: https://rust-lang.github.io/unsafe-code-guidelines/
- Windows Message Pump: https://docs.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues
