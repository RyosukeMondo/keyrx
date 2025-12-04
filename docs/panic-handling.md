# Panic Handling Architecture

## Overview

KeyRx implements a comprehensive panic handling system to ensure keyboard functionality remains available even during critical failures. The system consists of multiple layers of protection working together to catch, log, recover from, and prevent panics in production code.

## Architecture Components

The panic handling architecture consists of four main components:

1. **PanicGuard** - Catches and recovers from panics in critical code paths
2. **CircuitBreaker** - Prevents cascading failures by failing fast
3. **FallbackEngine** - Provides minimal passthrough mode when main engine fails
4. **Lint Enforcement** - Prevents new panic-inducing code at compile time

### Component Interaction

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Code                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  Clippy Lints (Compile Time)                                 │
│  - Deny unwrap_used, expect_used, panic                      │
│  - Enforced in CI/CD pipeline                                │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  PanicGuard (Runtime Protection)                             │
│  - Wraps critical operations with catch_unwind               │
│  - Captures backtraces and logs panic information            │
│  - Converts panics to CriticalError                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  CircuitBreaker (Failure Prevention)                         │
│  - Tracks consecutive failures                               │
│  - Opens circuit after threshold exceeded                    │
│  - Triggers fallback activation                              │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  FallbackEngine (Graceful Degradation)                       │
│  - Activates when main engine fails                          │
│  - Passes all input through unchanged                        │
│  - Zero dependencies, cannot fail                            │
└─────────────────────────────────────────────────────────────┘
```

## 1. PanicGuard

**Location:** `core/src/safety/panic_guard.rs`

### Purpose

PanicGuard wraps potentially panicking operations with `std::panic::catch_unwind`, converting panics into recoverable `CriticalError` values. This ensures panics never escape to FFI boundaries or crash the main event loop.

### Key Features

- Captures backtraces before unwinding for debugging
- Logs all panic information with context
- Records telemetry for monitoring
- Provides multiple recovery strategies

### Usage Patterns

#### Basic Usage

```rust
use keyrx_core::safety::panic_guard::PanicGuard;

let result = PanicGuard::new("keyboard_callback")
    .execute(|| {
        // Code that might panic
        process_keyboard_input(data)
    });

match result {
    Ok(value) => handle_success(value),
    Err(err) => {
        // Panic was caught and converted to CriticalError
        tracing::error!("Callback panicked: {}", err);
        execute_fallback();
    }
}
```

#### With Default Fallback

```rust
let value = PanicGuard::new("parse_config")
    .execute_or_default(
        || parse_config_file(),
        Config::default()
    );
```

#### With Custom Fallback Logic

```rust
let value = PanicGuard::new("load_mappings")
    .execute_or_else(
        || load_mappings_from_disk(),
        |err| {
            tracing::error!("Failed to load mappings: {}", err);
            notify_user_of_failure();
            load_default_mappings()
        }
    );
```

### When to Use PanicGuard

Use PanicGuard in these situations:

1. **Driver Callbacks**: Windows/Linux input event callbacks
2. **FFI Boundaries**: Before crossing from Rust into Flutter/Dart
3. **User Script Execution**: When evaluating Rhai scripts
4. **Plugin Code**: When calling third-party or user-provided code
5. **External Dependencies**: When calling libraries that might panic

### Integration Points

PanicGuard is integrated at the following critical points:

| Location | Context | Purpose |
|----------|---------|---------|
| `drivers/windows/mod.rs` | `low_level_keyboard_proc` | Catch panics in Windows hook callback |
| `drivers/linux/reader.rs` | Event loop | Catch panics in Linux input reader |
| `engine/mod.rs` | `process_event` | Protect main event processing |
| `scripting/runtime.rs` | Script evaluation | Catch panics from user scripts |
| `ffi/exports_*.rs` | FFI functions | Prevent panics crossing FFI boundary |

## 2. CircuitBreaker

**Location:** `core/src/safety/circuit_breaker.rs`

### Purpose

CircuitBreaker implements the circuit breaker pattern to prevent cascading failures. When a component fails repeatedly, the circuit opens and fails fast instead of attempting operations that will likely fail.

### States

```
     failure_threshold exceeded
Closed ─────────────────────────> Open
  ▲                                  │
  │                                  │ timeout elapsed
  │                                  ▼
  └─────────────────────────── HalfOpen
      success_threshold reached
```

- **Closed**: Normal operation, requests pass through
- **Open**: Circuit is tripped, requests fail fast without attempting the operation
- **HalfOpen**: Testing recovery, limited requests probe if the system has recovered

### Configuration

```rust
use keyrx_core::safety::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

let config = CircuitBreakerConfig {
    failure_threshold: 5,      // Open after 5 consecutive failures
    success_threshold: 2,      // Close after 2 consecutive successes
    timeout: Duration::from_secs(30),  // Wait 30s before half-open
};

let breaker = CircuitBreaker::new("keyboard_driver", config);
```

### Usage Pattern

```rust
match breaker.call(|| {
    driver.read_event()
}) {
    Ok(event) => process_event(event),
    Err(CriticalError::CircuitOpen { context }) => {
        tracing::warn!("Circuit open: {}", context);
        activate_fallback();
    }
    Err(err) => {
        tracing::error!("Operation failed: {}", err);
        // Circuit breaker tracks this failure
    }
}
```

### Telemetry Integration

CircuitBreaker records telemetry events for monitoring:

- `circuit_breaker_open`: When circuit transitions to open state
- `circuit_breaker_close`: When circuit transitions back to closed state
- `circuit_breaker_half_open`: When circuit enters half-open testing state

These events are exported via FFI and can trigger notifications in the Flutter UI.

### When to Use CircuitBreaker

Use CircuitBreaker for:

1. **Driver Operations**: Wrap driver initialization and event reading
2. **External Services**: Protect calls to external APIs or services
3. **Resource-Intensive Operations**: Operations that might exhaust resources
4. **Unreliable Dependencies**: Components known to occasionally fail

### ResilientDriver Integration

CircuitBreaker is integrated into drivers via the `ResilientDriver` wrapper:

```rust
// drivers/mod.rs
pub struct ResilientDriver<D> {
    driver: D,
    circuit_breaker: Arc<CircuitBreaker>,
    fallback_engine: Arc<FallbackEngine>,
}

impl<D: Driver> ResilientDriver<D> {
    pub async fn read_event(&mut self) -> Result<InputEvent, KeyRxError> {
        self.circuit_breaker.call(|| {
            self.driver.read_event()
        }).map_err(|e| {
            // Circuit opened - activate fallback
            self.fallback_engine.activate(
                FallbackReason::CircuitBreakerOpen
            );
            e.into()
        })
    }
}
```

## 3. FallbackEngine

**Location:** `core/src/engine/fallback.rs`

### Purpose

FallbackEngine provides a minimal, zero-dependency safety net that ensures keyboard input continues working even when the main engine fails catastrophically. It implements simple passthrough behavior with no remapping, state tracking, or external dependencies.

### Design Principles

1. **Minimal Dependencies**: No script runtime, state store, or metrics
2. **Always Works**: Cannot fail - all inputs pass through unchanged
3. **Observable**: Tracks activation reason and event count for debugging
4. **Thread-Safe**: Can be shared across threads using Arc

### Activation Reasons

```rust
pub enum FallbackReason {
    /// Main engine panicked during event processing
    Panic(String),
    /// Circuit breaker opened due to repeated failures
    CircuitBreakerOpen,
    /// Critical error in driver or core component
    CriticalError(String),
    /// Manual activation for testing or maintenance
    Manual,
}
```

### Usage Pattern

```rust
use keyrx_core::engine::fallback::{FallbackEngine, FallbackReason};

let fallback = FallbackEngine::new();

// Activate when main engine fails
if let Err(err) = main_engine.process_event(&event) {
    tracing::error!("Main engine failed: {}", err);
    fallback.activate(FallbackReason::CriticalError(
        format!("Engine failure: {}", err)
    ));
}

// Process events through fallback
let action = if fallback.is_active() {
    fallback.process_event(&event)  // Always returns PassThrough
} else {
    main_engine.process_event(&event)?
};

// Deactivate when main engine recovers
if main_engine.is_healthy() {
    fallback.deactivate();
}
```

### Monitoring

FallbackEngine exposes observability methods:

```rust
// Check if fallback is active
if fallback.is_active() {
    // Get activation reason
    if let Some(reason) = fallback.get_reason() {
        tracing::warn!("Fallback active: {}", reason);
    }

    // Get event count
    let count = fallback.event_count();
    tracing::info!("Processed {} events in fallback mode", count);
}
```

### Integration with UI

The fallback state is exported via FFI for UI notifications:

```rust
// ffi/exports_telemetry.rs
#[no_mangle]
pub extern "C" fn keyrx_is_fallback_active() -> bool {
    FALLBACK_ENGINE.is_active()
}

#[no_mangle]
pub extern "C" fn keyrx_get_fallback_reason() -> *mut c_char {
    // Returns JSON with reason and event count
}
```

This allows the Flutter UI to display a notification when fallback mode is active.

## 4. Lint Enforcement

**Location:** `core/Cargo.toml`

### Purpose

Prevent new panic-inducing code from being introduced by enforcing strict Clippy lints at compile time.

### Configuration

```toml
[lints.clippy]
unwrap_used = "deny"     # Denies .unwrap() calls
expect_used = "deny"     # Denies .expect() calls
panic = "deny"           # Denies panic!() macro
```

### Enforcement Strategy

1. **Compilation**: Lints run during `cargo build` and `cargo check`
2. **CI/CD**: Pre-commit hooks run `cargo clippy -- -D warnings`
3. **Code Review**: Automated checks in pull requests

### Allowed Exceptions

In rare cases, `#[allow()]` annotations are permitted with justification:

```rust
// SAFETY: We verified above that the vector has exactly 3 elements.
// This indexing cannot panic because we validated the length.
#[allow(clippy::indexing_slicing)]
let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
```

See [Panic Prevention Lints](./panic-prevention-lints.md) for detailed guidelines.

## Error Flow

### Normal Operation (No Panic)

```
User Input
    ↓
Driver reads event
    ↓
CircuitBreaker: call() → Execute operation
    ↓
PanicGuard: execute() → Run callback
    ↓
Engine processes event
    ↓
Output actions executed
```

### Panic Recovery Flow

```
User Input
    ↓
Driver reads event
    ↓
CircuitBreaker: call() → Execute operation
    ↓
PanicGuard: execute() → Run callback
    ↓
Engine panics!
    ↓
PanicGuard catches panic
    ├─ Capture backtrace
    ├─ Log error with context
    ├─ Record telemetry
    └─ Return CriticalError::CallbackPanic
    ↓
CircuitBreaker records failure
    ↓
After threshold: Circuit opens
    ↓
FallbackEngine activates
    ↓
All subsequent events pass through unchanged
    ↓
UI notified of fallback mode
```

### Recovery Flow

```
Time passes (circuit timeout)
    ↓
CircuitBreaker transitions to HalfOpen
    ↓
Test operations succeed
    ↓
After success_threshold: Circuit closes
    ↓
FallbackEngine deactivates
    ↓
Normal operation resumes
    ↓
UI notification cleared
```

## Testing Panic Handling

### Panic Injection Tests

**Location:** `core/tests/panic_recovery_test.rs`

The test suite injects panics at various points to verify recovery:

```rust
#[test]
fn test_driver_callback_panic_recovery() {
    let guard = PanicGuard::new("test_callback");

    let result = guard.execute(|| {
        panic!("Simulated driver panic");
    });

    assert!(matches!(result, Err(CriticalError::CallbackPanic { .. })));
}

#[test]
fn test_circuit_breaker_opens_after_failures() {
    let breaker = CircuitBreaker::new("test", config);

    // Trigger threshold failures
    for _ in 0..5 {
        let _ = breaker.call(|| Err(CriticalError::DriverError { .. }));
    }

    // Circuit should now be open
    let result = breaker.call(|| Ok(()));
    assert!(matches!(result, Err(CriticalError::CircuitOpen { .. })));
}
```

### Manual Testing

You can manually trigger panic scenarios using the CLI:

```bash
# Inject panic in driver callback
keyrx debug inject-panic driver

# Inject panic in engine
keyrx debug inject-panic engine

# Trigger fallback mode
keyrx debug activate-fallback

# View panic telemetry
keyrx debug show-telemetry
```

## Best Practices

### 1. Always Wrap Critical Operations

```rust
// ✅ GOOD: Wrapped with PanicGuard
fn process_callback(data: KeyboardData) -> Result<(), CriticalError> {
    PanicGuard::new("keyboard_callback").execute(|| {
        unsafe_process_input(data)
    })
}

// ❌ BAD: No panic protection
fn process_callback(data: KeyboardData) {
    unsafe_process_input(data)  // Could panic and crash!
}
```

### 2. Use CircuitBreaker for Repeated Operations

```rust
// ✅ GOOD: Circuit breaker prevents repeated failures
let breaker = CircuitBreaker::new("driver", config);
loop {
    match breaker.call(|| driver.read_event()) {
        Ok(event) => process(event),
        Err(CriticalError::CircuitOpen { .. }) => {
            // Circuit open, wait before retry
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Err(e) => tracing::error!("Read error: {}", e),
    }
}

// ❌ BAD: Continuous retry without circuit breaker
loop {
    match driver.read_event() {
        Ok(event) => process(event),
        Err(e) => {
            // Keeps retrying even if driver is broken
            tracing::error!("Read error: {}", e);
        }
    }
}
```

### 3. Activate Fallback on Critical Failures

```rust
// ✅ GOOD: Fallback ensures keyboard works
match engine.process_event(&event) {
    Ok(action) => execute_action(action),
    Err(e) => {
        tracing::error!("Engine failed: {}", e);
        fallback.activate(FallbackReason::CriticalError(
            format!("{}", e)
        ));
        // Keyboard still works via passthrough
    }
}

// ❌ BAD: Keyboard stops working on engine failure
match engine.process_event(&event) {
    Ok(action) => execute_action(action),
    Err(e) => {
        tracing::error!("Engine failed: {}", e);
        // No fallback - keyboard is now unresponsive!
    }
}
```

### 4. Never Panic Across FFI

```rust
// ✅ GOOD: Errors returned as codes or null pointers
#[no_mangle]
pub extern "C" fn keyrx_process_event(event: *const Event) -> i32 {
    let result = PanicGuard::new("ffi_process_event")
        .execute(|| {
            let event = unsafe { &*event };
            process_event_internal(event)
        });

    match result {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("FFI call failed: {}", e);
            -1
        }
    }
}

// ❌ BAD: Panic crosses FFI boundary
#[no_mangle]
pub extern "C" fn keyrx_bad_example(event: *const Event) -> i32 {
    let event = unsafe { &*event };
    process_event_internal(event).unwrap()  // UB if this panics!
}
```

## Monitoring and Observability

### Telemetry Events

The panic handling system records the following telemetry:

| Event | Trigger | Exported to FFI |
|-------|---------|-----------------|
| `panic_caught` | PanicGuard catches panic | Yes |
| `panic_recovered` | Operation succeeds after panic | Yes |
| `circuit_breaker_open` | Circuit opens | Yes |
| `circuit_breaker_close` | Circuit closes | Yes |
| `fallback_activated` | Fallback mode starts | Yes |
| `fallback_deactivated` | Fallback mode ends | Yes |

### FFI Exports for UI

```c
// Get panic count since start
uint64_t keyrx_get_panic_count();

// Get recovery count
uint64_t keyrx_get_recovery_count();

// Check if fallback is active
bool keyrx_is_fallback_active();

// Get circuit breaker state (0=closed, 1=open, 2=half-open)
uint8_t keyrx_get_circuit_state();
```

### Log Output

All panic events are logged with structured information:

```json
{
  "timestamp": "2025-12-04T16:30:00Z",
  "level": "ERROR",
  "service": "keyrx",
  "event": "panic_caught",
  "component": "panic_guard",
  "context": "keyboard_callback",
  "panic_message": "index out of bounds",
  "backtrace": "..."
}
```

## Troubleshooting

### Fallback Mode Won't Deactivate

**Symptoms:** Fallback engine stays active, main engine not processing

**Diagnosis:**
```bash
keyrx debug show-fallback-state
```

**Possible Causes:**
1. Circuit breaker still open → Check `keyrx_get_circuit_state()`
2. Main engine not recovered → Check engine health
3. Bug in recovery detection → Review circuit breaker thresholds

### Excessive Panics Logged

**Symptoms:** Many `panic_caught` events in telemetry

**Diagnosis:**
```bash
keyrx debug show-telemetry | grep panic_caught
```

**Possible Causes:**
1. User script has bugs → Review script code
2. Driver issue → Check driver logs
3. State corruption → Clear state and restart

### Circuit Opens Too Frequently

**Symptoms:** Circuit breaker opens after a few failures

**Solution:** Adjust circuit breaker thresholds in configuration:

```rust
CircuitBreakerConfig {
    failure_threshold: 10,  // Increase from 5
    success_threshold: 2,
    timeout: Duration::from_secs(60),  // Increase from 30
}
```

## Related Documentation

- [Panic Prevention Lints](./panic-prevention-lints.md) - Compile-time enforcement
- [FFI Panic Safety](./ffi-panic-safety.md) - FFI-specific patterns
- [Error Handling](./errors/README.md) - Error type design
- [Emergency Exit Safety](./emergency-exit-safety.md) - Keyboard escape hatch
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html) - External reference

## Changelog

- **2025-12-04**: Initial documentation for panic handling architecture
- **2025-12-04**: Added PanicGuard, CircuitBreaker, and FallbackEngine components
- **2025-12-04**: Documented lint enforcement and telemetry integration
