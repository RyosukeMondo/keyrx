# Design Document - Unwrap Remediation

## Overview

**Specification**: unwrap-remediation
**Created**: 2025-12-31
**Status**: Draft

**Design Goals**:
1. Eliminate all critical panic points (18 occurrences)
2. Establish robust error handling infrastructure
3. Maintain performance in hot paths
4. Preserve backward compatibility
5. Enable graceful degradation on errors

---

## 1. Architecture Overview

### 1.1 Current Architecture Issues

**Issue 1: Missing Error Type Hierarchy**
- **Location**: All modules (keyrx_daemon, keyrx_compiler, keyrx_core)
- **Problem**: Functions return generic `Result<T>` without module-specific errors, forcing callers to use `unwrap()` for flow control
- **Files Affected**: 15 production files with unwraps
- **Impact**: No semantic error information, poor error messages, non-recoverable failures

**Issue 2: Unsafe Mutex Access Patterns**
- **Location**: `keyrx_daemon/src/platform/windows/rawinput.rs`, `keyrx_core/src/wasm/mod.rs`
- **Problem**: `lock().unwrap()` in hot paths (message handler, WASM API), single panic poisons mutex permanently
- **Files Affected**: 2 files, 18 occurrences
- **Impact**: Cascading failures, entire daemon crashes on single panic

**Issue 3: Binary Format Parsing Without Validation**
- **Location**: `keyrx_compiler/src/serialize.rs`
- **Problem**: `try_into().unwrap()` during .krx deserialization, corrupted/truncated files crash daemon
- **Files Affected**: 1 file, 6 occurrences
- **Impact**: User data corruption causes crash, no recovery possible

**Issue 4: No Error Propagation Strategy**
- **Location**: Platform trait implementations
- **Problem**: Trait methods promise `Result<T>` but implementations panic on lock failure
- **Files Affected**: 3 platform implementations
- **Impact**: Broken promise of error propagation, violates Liskov Substitution Principle

**Issue 5: Network Operations Without Error Context**
- **Location**: `keyrx_daemon/src/ipc/unix_socket.rs`
- **Problem**: Socket operations unwrap without path/error context
- **Files Affected**: 1 file, 7 occurrences
- **Impact**: Bind failures provide no diagnostic information

### 1.2 Target Architecture

**Solution 1: Error Type Hierarchy**
```
DaemonError (top-level)
├── PlatformError
│   ├── Poisoned(String)
│   ├── InitializationFailed(String)
│   └── DeviceError(String)
├── SerializationError
│   ├── InvalidMagic { expected: u32, found: u32 }
│   ├── InvalidVersion { expected: u32, found: u32 }
│   ├── InvalidSize { expected: usize, found: usize }
│   └── CorruptedData(String)
├── SocketError
│   ├── BindFailed { path: PathBuf, error: io::Error }
│   ├── ListenFailed { error: io::Error }
│   ├── NotConnected
│   └── AlreadyConnected
├── RegistryError
│   ├── IOError(io::ErrorKind)
│   ├── CorruptedRegistry(String)
│   └── FailedToLoad(io::ErrorKind)
└── RecorderError
    ├── NotRecording
    ├── AlreadyRecording
    └── PlaybackFailed(usize)  // frame number
```

**Solution 2: Mutex Poison Recovery**
- Pattern: Detect poison, attempt recovery or clear mutex
- Implementation: `recover_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, PlatformError>`
- Strategy: On poison, log error, recreate mutex if possible, return error otherwise

**Solution 3: Binary Format Validation**
- Pattern: Validate at each step, return specific error variant
- Implementation: `validate_magic()`, `validate_version()`, `validate_size()` helpers
- Strategy: Early return on validation failure with context

**Solution 4: Error Propagation via From Implementations**
- Pattern: All module errors convert to `DaemonError` via `From` trait
- Implementation: `impl From<PlatformError> for DaemonError`
- Strategy: Use `?` operator throughout, top-level converts to exit code

**Solution 5: Connection State Machine**
- Pattern: Explicit state tracking prevents invalid operations
- Implementation: `enum State { Disconnected, Connecting, Connected }`
- Strategy: State transitions validated, errors on invalid state

---

## 2. Module Design

### 2.1 Error Type Definitions

**File: `keyrx_daemon/src/error.rs` (extend existing)**

```rust
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Platform-specific operation errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PlatformError {
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),

    #[error("Platform initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Device operation failed: {0}")]
    DeviceError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Binary serialization/deserialization errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializationError {
    #[error("Invalid magic number: expected {expected:#010x}, found {found:#010x}")]
    InvalidMagic { expected: u32, found: u32 },

    #[error("Unsupported version: expected {expected}, found {found}")]
    InvalidVersion { expected: u32, found: u32 },

    #[error("Invalid size: expected {expected} bytes, found {found} bytes")]
    InvalidSize { expected: usize, found: usize },

    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// IPC socket operation errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SocketError {
    #[error("Failed to bind socket at {path:?}: {error}")]
    BindFailed { path: PathBuf, error: io::Error },

    #[error("Failed to listen on socket: {error}")]
    ListenFailed { error: io::Error },

    #[error("Socket not connected")]
    NotConnected,

    #[error("Socket already connected")]
    AlreadyConnected,

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Device registry operation errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RegistryError {
    #[error("IO error: {0:?}")]
    IOError(io::ErrorKind),

    #[error("Corrupted registry: {0}")]
    CorruptedRegistry(String),

    #[error("Failed to load registry: {0:?}")]
    FailedToLoad(io::ErrorKind),
}

/// Macro recorder operation errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RecorderError {
    #[error("Not currently recording")]
    NotRecording,

    #[error("Already recording")]
    AlreadyRecording,

    #[error("Playback failed at frame {0}")]
    PlaybackFailed(usize),
}

// Conversions to top-level DaemonError
impl From<PlatformError> for DaemonError {
    fn from(err: PlatformError) -> Self {
        DaemonError::Platform(err)
    }
}

impl From<SerializationError> for DaemonError {
    fn from(err: SerializationError) -> Self {
        DaemonError::Serialization(err)
    }
}

// ... similar for other error types
```

### 2.2 Mutex Poison Recovery Pattern

**File: `keyrx_daemon/src/platform/recovery.rs` (new)**

```rust
use std::sync::{Mutex, MutexGuard, PoisonError};
use crate::error::PlatformError;

/// Attempts to acquire a mutex lock, recovering from poisoned state
///
/// # Examples
///
/// ```
/// let mutex = Mutex::new(42);
/// let guard = recover_lock(&mutex)?;
/// assert_eq!(*guard, 42);
/// ```
///
/// # Errors
///
/// Returns `PlatformError::Poisoned` if mutex is poisoned and cannot be recovered
pub fn recover_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, PlatformError> {
    mutex.lock().or_else(|poison_error: PoisonError<MutexGuard<T>>| {
        log::warn!("Mutex poisoned, attempting recovery");
        // For most cases, we can still use the poisoned guard
        // The data may be inconsistent, but we log the warning
        Ok(poison_error.into_inner())
    })
}

/// Attempts to acquire a mutex lock with context for error messages
pub fn recover_lock_with_context<T>(
    mutex: &Mutex<T>,
    context: &str,
) -> Result<MutexGuard<T>, PlatformError> {
    mutex.lock().or_else(|poison_error: PoisonError<MutexGuard<T>>| {
        log::error!("Mutex poisoned in {}: recovering", context);
        Ok(poison_error.into_inner())
    })
}
```

**Usage in Windows platform (before):**
```rust
// keyrx_daemon/src/platform/windows/rawinput.rs:91
let mut context_guard = bridge_context.lock().unwrap();  // PANICS
```

**Usage in Windows platform (after):**
```rust
use crate::platform::recovery::recover_lock_with_context;

// keyrx_daemon/src/platform/windows/rawinput.rs:91
let mut context_guard = recover_lock_with_context(
    &bridge_context,
    "Windows message handler"
)?;
```

### 2.3 Binary Format Validation Pattern

**File: `keyrx_compiler/src/serialize.rs` (modify existing)**

```rust
use crate::error::SerializationError;

const MAGIC: u32 = 0x4B5258_00;  // "KRX\0"
const VERSION: u32 = 1;

/// Validates magic number in binary format
fn validate_magic(bytes: &[u8]) -> Result<(), SerializationError> {
    if bytes.len() < 4 {
        return Err(SerializationError::InvalidSize {
            expected: 4,
            found: bytes.len(),
        });
    }

    let magic_bytes: [u8; 4] = bytes[0..4]
        .try_into()
        .map_err(|_| SerializationError::CorruptedData(
            "Failed to read magic number".to_string()
        ))?;

    let found_magic = u32::from_le_bytes(magic_bytes);
    if found_magic != MAGIC {
        return Err(SerializationError::InvalidMagic {
            expected: MAGIC,
            found: found_magic,
        });
    }

    Ok(())
}

/// Validates version number in binary format
fn validate_version(bytes: &[u8]) -> Result<(), SerializationError> {
    if bytes.len() < 4 {
        return Err(SerializationError::InvalidSize {
            expected: 4,
            found: bytes.len(),
        });
    }

    let version_bytes: [u8; 4] = bytes[0..4]
        .try_into()
        .map_err(|_| SerializationError::CorruptedData(
            "Failed to read version number".to_string()
        ))?;

    let found_version = u32::from_le_bytes(version_bytes);
    if found_version != VERSION {
        return Err(SerializationError::InvalidVersion {
            expected: VERSION,
            found: found_version,
        });
    }

    Ok(())
}

// Usage (before):
// let magic_array: [u8; 4] = magic.try_into().unwrap();  // PANICS

// Usage (after):
validate_magic(&file_bytes)?;
let magic_array: [u8; 4] = file_bytes[0..4].try_into().unwrap();  // Safe after validation
// OR better:
let magic_array: [u8; 4] = file_bytes[0..4].try_into()
    .map_err(|_| SerializationError::CorruptedData("Invalid magic".into()))?;
```

### 2.4 Socket Connection State Machine

**File: `keyrx_daemon/src/ipc/unix_socket.rs` (modify existing)**

```rust
use crate::error::SocketError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct UnixSocketServer {
    stream: Option<UnixStream>,
    state: ConnectionState,
    socket_path: PathBuf,
}

impl UnixSocketServer {
    pub fn send(&mut self, data: &[u8]) -> Result<(), SocketError> {
        if self.state != ConnectionState::Connected {
            return Err(SocketError::NotConnected);
        }

        let stream = self.stream
            .as_mut()
            .ok_or(SocketError::NotConnected)?;

        stream.write_all(data)
            .map_err(|e| SocketError::Io(e))?;

        Ok(())
    }

    pub fn bind(&mut self) -> Result<(), SocketError> {
        if self.state == ConnectionState::Connected {
            return Err(SocketError::AlreadyConnected);
        }

        self.state = ConnectionState::Connecting;

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|error| SocketError::BindFailed {
                path: self.socket_path.clone(),
                error,
            })?;

        log::info!("Socket bound at {:?}", self.socket_path);

        let (stream, _) = listener.accept()
            .map_err(|error| SocketError::ListenFailed { error })?;

        self.stream = Some(stream);
        self.state = ConnectionState::Connected;

        log::info!("Socket connected");
        Ok(())
    }
}
```

### 2.5 Device Registry Resilience

**File: `keyrx_daemon/src/config/device_registry.rs` (modify existing)**

```rust
use crate::error::RegistryError;

impl DeviceRegistry {
    /// Loads registry from file, creating empty registry on failure
    pub fn load() -> Result<Self, RegistryError> {
        let path = Self::registry_path();

        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                // Try to parse, fall back to empty on corruption
                match serde_json::from_str(&contents) {
                    Ok(registry) => {
                        log::debug!("Loaded device registry from {:?}", path);
                        Ok(registry)
                    }
                    Err(e) => {
                        log::warn!("Corrupted registry at {:?}: {}. Creating empty registry.", path, e);
                        let empty = Self::default();
                        empty.save()?;  // Save empty registry
                        Ok(empty)
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::info!("No registry file found, creating new registry");
                let empty = Self::default();
                empty.save()?;
                Ok(empty)
            }
            Err(e) => {
                Err(RegistryError::FailedToLoad(e.kind()))
            }
        }
    }

    pub fn save(&self) -> Result<(), RegistryError> {
        let path = Self::registry_path();
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| RegistryError::CorruptedRegistry(e.to_string()))?;

        std::fs::write(&path, contents)
            .map_err(|e| RegistryError::IOError(e.kind()))?;

        log::debug!("Saved device registry to {:?}", path);
        Ok(())
    }
}
```

---

## 3. Test Strategy

### 3.1 Unit Tests

**Target**: ≥100% coverage for error types and recovery functions

**Test Categories**:

1. **Error Type Tests**
   - All error variants constructible
   - Display implementation produces expected messages
   - From conversions work correctly
   - Error context preserved through conversions

2. **Mutex Poison Recovery Tests**
   - Poisoned mutex recovers successfully
   - Warning logged on poison
   - Subsequent operations succeed after recovery
   - Concurrent access works after recovery

3. **Binary Format Validation Tests**
   - Valid format passes all validations
   - Invalid magic number detected
   - Invalid version detected
   - Truncated file detected
   - Corrupted data detected

4. **Socket State Machine Tests**
   - State transitions valid
   - Invalid state transitions rejected
   - Operations fail in wrong state
   - Error context includes socket path

**Example Test**:
```rust
#[test]
fn test_mutex_poison_recovery() {
    let mutex = Arc::new(Mutex::new(42));
    let mutex_clone = mutex.clone();

    // Poison the mutex by panicking while holding lock
    let handle = std::thread::spawn(move || {
        let _guard = mutex_clone.lock().unwrap();
        panic!("Intentional panic to poison mutex");
    });

    let _ = handle.join();  // Thread panicked

    // Recovery should succeed
    let result = recover_lock(&mutex);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn test_invalid_magic_number() {
    let bad_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];  // Wrong magic
    let result = validate_magic(&bad_bytes);
    assert!(matches!(
        result,
        Err(SerializationError::InvalidMagic { expected: 0x4B5258_00, found: 0xFFFFFFFF })
    ));
}

#[test]
fn test_socket_state_machine() {
    let mut socket = UnixSocketServer::new(PathBuf::from("/tmp/test.sock"));

    // Cannot send while disconnected
    let result = socket.send(b"test");
    assert!(matches!(result, Err(SocketError::NotConnected)));

    // Bind succeeds
    socket.bind().unwrap();

    // Cannot bind again
    let result = socket.bind();
    assert!(matches!(result, Err(SocketError::AlreadyConnected)));

    // Send succeeds after connected
    socket.send(b"test").unwrap();
}
```

### 3.2 Integration Tests

**Target**: All public APIs tested, ≥90% coverage

**Test Scenarios**:

1. **End-to-End Error Propagation**
   - Platform error propagates to daemon error
   - Daemon error converts to exit code
   - Error logged with full context

2. **Resilience Tests**
   - Daemon continues after recoverable error
   - Graceful degradation on device failure
   - Registry recovery from corruption

3. **Error Message Quality**
   - User-facing errors are actionable
   - Developer errors include file:line
   - Errors include system context (errno, etc.)

**Example Integration Test**:
```rust
#[test]
fn test_corrupted_krx_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let krx_path = temp_dir.path().join("corrupted.krx");

    // Write truncated file
    std::fs::write(&krx_path, vec![0xFF; 2]).unwrap();

    // Daemon should handle gracefully
    let result = Daemon::load_config(&krx_path);
    assert!(matches!(
        result,
        Err(DaemonError::Serialization(SerializationError::InvalidSize { .. }))
    ));

    // Error message should be helpful
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid size"));
    assert!(msg.contains("expected"));
    assert!(msg.contains("found"));
}
```

### 3.3 Performance Tests

**Criterion Benchmarks**:

1. **Lock Recovery Overhead**
   - Baseline: Normal lock acquisition
   - Test: Recovery lock acquisition
   - Target: <10μs difference

2. **Error Path Overhead**
   - Baseline: Happy path (no errors)
   - Test: Error path with Result propagation
   - Target: <1% difference

**Example Benchmark**:
```rust
fn bench_lock_recovery(c: &mut Criterion) {
    let mutex = Mutex::new(42);

    c.bench_function("normal_lock", |b| {
        b.iter(|| {
            let _guard = mutex.lock().unwrap();
        })
    });

    c.bench_function("recover_lock", |b| {
        b.iter(|| {
            let _guard = recover_lock(&mutex).unwrap();
        })
    });
}
```

---

## 4. Migration Strategy

### Phase 1: Foundation (Tasks 1-5)

**Goal**: Establish error type infrastructure and recovery utilities

**Tasks**:
1. Create error type definitions (PlatformError, SerializationError, SocketError, RegistryError, RecorderError)
2. Implement mutex poison recovery utilities
3. Implement binary format validation helpers
4. Add From conversions to DaemonError
5. Write unit tests for all error types (≥100% coverage)

**Deliverables**:
- `keyrx_daemon/src/error.rs` (extended)
- `keyrx_daemon/src/platform/recovery.rs` (new)
- `keyrx_compiler/src/serialize.rs` (validation helpers)
- Unit tests with 100% coverage

**Success Criteria**:
- All error types compile without warnings
- All unit tests pass
- Coverage ≥100% on new error code

### Phase 2: Critical Fixes (Tasks 6-8)

**Goal**: Eliminate critical panic points (P0)

**Tasks**:
6. Replace lock().unwrap() in Windows platform (8 occurrences)
7. Replace lock().unwrap() in WASM API (10 occurrences)
8. Replace try_into().unwrap() in serialization (6 occurrences)

**Deliverables**:
- `keyrx_daemon/src/platform/windows/rawinput.rs` (modified)
- `keyrx_core/src/wasm/mod.rs` (modified)
- `keyrx_compiler/src/serialize.rs` (modified)
- Integration tests for error scenarios

**Success Criteria**:
- 0 lock().unwrap() in production code
- 0 try_into().unwrap() in production code
- All existing tests pass
- New error scenario tests pass

### Phase 3: High Priority Fixes (Tasks 9-11)

**Goal**: Fix high-priority error handling gaps (P1)

**Tasks**:
9. Replace IPC socket unwraps (7 occurrences)
10. Replace runtime initialization unwrap (1 occurrence)
11. Ensure Platform trait error propagation (10 implementations)

**Deliverables**:
- `keyrx_daemon/src/ipc/unix_socket.rs` (modified with state machine)
- `keyrx_daemon/src/main.rs` (modified)
- Platform trait implementations (verified error propagation)
- Integration tests

**Success Criteria**:
- Socket state machine implemented
- All platform errors propagate correctly
- Error messages include context
- Integration tests verify error paths

### Phase 4: Medium Priority Fixes (Tasks 12-14)

**Goal**: Add resilience to non-critical components (P2)

**Tasks**:
12. Add signal handler error handling (3 occurrences)
13. Implement device registry resilience (15 occurrences)
14. Add macro recorder error handling (15 occurrences)

**Deliverables**:
- `keyrx_daemon/src/daemon/signals/linux.rs` (modified)
- `keyrx_daemon/src/config/device_registry.rs` (modified)
- `keyrx_daemon/src/macro_recorder.rs` (modified)
- Unit and integration tests

**Success Criteria**:
- Signal failures logged, daemon continues
- Registry recovers from corruption
- Recorder errors isolated from test failures

### Phase 5: Documentation and Cleanup (Tasks 15-17)

**Goal**: Document remaining unwraps and establish quality gates (P3)

**Tasks**:
15. Document all remaining production unwraps with safety rationale
16. Create pre-commit hook to prevent new unwraps
17. Write error handling strategy ADR

**Deliverables**:
- Safety comments on ≤60 remaining unwraps
- `.git/hooks/pre-commit` (unwrap detection)
- `docs/error-handling-strategy.md`
- Updated code review checklist

**Success Criteria**:
- All production unwraps have `// SAFETY:` comment
- Pre-commit hook blocks new unwraps
- Documentation complete

### Phase 6: Validation (Tasks 18-20)

**Goal**: Verify all success criteria met

**Tasks**:
18. Run full test suite (unit + integration)
19. Run performance benchmarks
20. Run coverage analysis

**Deliverables**:
- Test report (all tests passing)
- Benchmark report (no regression)
- Coverage report (≥90% on modified code)

**Success Criteria**:
- Production unwraps: ≤60 (91% reduction)
- All tests passing
- Benchmarks within 5% of baseline
- Coverage targets met

---

## 5. Alternative Designs Considered

### Alternative 1: Anyhow for All Errors

**Approach**: Use `anyhow::Error` throughout instead of custom error types

**Pros**:
- Less boilerplate (no error enum definitions)
- Easy error chaining with `.context()`
- Good for prototyping

**Cons**:
- Type erasure loses error semantics
- Cannot pattern match on error variants
- Harder to implement recovery strategies
- Poor API design (no compile-time error checking)

**Decision**: Rejected. Custom error types provide better API semantics and enable recovery.

### Alternative 2: Error Codes Instead of Enums

**Approach**: Use integer error codes like C APIs

**Pros**:
- FFI-compatible
- Small memory footprint

**Cons**:
- Error messages separate from codes (easy to desync)
- No type safety
- Requires documentation lookup
- Not idiomatic Rust

**Decision**: Rejected. Rust error enums are safer and more ergonomic.

### Alternative 3: Never Unwrap (Propagate All Errors)

**Approach**: Remove 100% of unwraps, even in tests

**Pros**:
- Absolute guarantee of no panics
- Consistent error handling everywhere

**Cons**:
- Test code becomes verbose
- Legitimate infallible operations require boilerplate
- Diminishing returns (test unwraps are acceptable)

**Decision**: Rejected. Target ≤60 production unwraps with documentation is pragmatic.

### Alternative 4: Panic Handler Instead of Recovery

**Approach**: Install custom panic handler that logs and continues

**Pros**:
- No code changes required
- Centralized panic handling

**Cons**:
- Violates Rust's safety guarantees
- Unpredictable state after panic
- Hard to reason about correctness
- Not recommended by Rust community

**Decision**: Rejected. Proper error handling is safer and more predictable.

---

## 6. Open Questions

1. **Question**: Should we use `eyre` instead of `thiserror` for better error context chaining?
   - **Answer**: Defer to implementation. `thiserror` is more established, `eyre` has better context. Both are acceptable.

2. **Question**: Should mutex poison recovery attempt to clear/reset the mutex or just use the poisoned guard?
   - **Answer**: Use poisoned guard (data may be inconsistent but accessible). Document in rustdoc.

3. **Question**: Should we add telemetry/monitoring for error rates?
   - **Answer**: Out of scope for this spec. Track as future enhancement.

4. **Question**: Should test unwraps be allowed or should we enforce Result in tests too?
   - **Answer**: Test unwraps are acceptable (919 current). Focus on production code only.

---

## 7. Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Performance regression in hot paths | High | Medium | Benchmark before/after, optimize if needed |
| Breaking API changes | High | Low | Maintain backward compatibility, use `#[non_exhaustive]` |
| Incomplete error coverage | Medium | Medium | Comprehensive testing, code review checklist |
| Mutex recovery strategy unsound | High | Low | Thorough review of poison semantics, document assumptions |
| Binary validation too strict | Medium | Low | Test with real-world .krx files, add escape hatches if needed |

---

## 8. Future Enhancements

**Post-V1 Improvements**:
1. Error telemetry integration (send error rates to monitoring)
2. User-facing error code lookup tool
3. Automatic error recovery strategies (retry with backoff)
4. Error internationalization (i18n)
5. Structured error logging (JSON format)

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-31 | Claude (debt-to-spec) | Initial design from audit |

---

## Approval

**Status**: Pending Review

**Reviewers**:
- [ ] Senior Engineer (error handling strategy)
- [ ] Platform Team Lead (mutex recovery approach)
- [ ] Security Team (binary validation strategy)

**Approval Date**: _________________
