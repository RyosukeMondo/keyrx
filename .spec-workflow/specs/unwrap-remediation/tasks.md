# Tasks Document - Unwrap Remediation

## Overview

**Specification**: unwrap-remediation
**Total Tasks**: 20
**Phases**: 6
**Estimated Effort**: 40-60 hours

---

## Phase 1: Foundation (Tasks 1-5)

**Goal**: Establish error type infrastructure and recovery utilities

### Task 1: Create Custom Error Type Definitions

- [x] **1. Create Custom Error Type Definitions**
  - **File**: `keyrx_daemon/src/error.rs` (extend existing)
  - **Purpose**: Define comprehensive error type hierarchy to enable proper error propagation and recovery. This replaces generic `unwrap()` patterns with semantic error handling.
  - **Requirements**: FR1, NFR1, NFR6
  - **Leverage**: Existing `DaemonError` enum in `keyrx_daemon/src/error.rs`
  - **Prompt**:

    **Role**: Rust Error Handling Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Extend the existing `keyrx_daemon/src/error.rs` file with five new error type enums using `thiserror` derive macros.

    **Error Types to Create**:

    1. **PlatformError** (for platform-specific operations):
       ```rust
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
       ```

    2. **SerializationError** (for binary format parsing):
       ```rust
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
       ```

    3. **SocketError** (for IPC operations):
       ```rust
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
       ```

    4. **RegistryError** (for device registry):
       ```rust
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
       ```

    5. **RecorderError** (for macro recording):
       ```rust
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
       ```

    **From Conversions**: Add `From<PlatformError>` for `DaemonError`, and similar for all new error types.

    **Dependencies**: Add `thiserror = "1.0"` to `keyrx_daemon/Cargo.toml`

    **Tests**: Create unit tests in `keyrx_daemon/src/error.rs` (in `#[cfg(test)] mod tests`) verifying:
    - All error variants constructible
    - Display messages match expected format
    - From conversions work correctly
    - Error types implement `std::error::Error`

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, use `thiserror` derive macros, all types `#[non_exhaustive]`
  | **Success**: ✅ 5 error types defined with all variants, ✅ From conversions to DaemonError implemented, ✅ Unit tests pass with ≥100% coverage on new types, ✅ Zero clippy warnings
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `classes: [PlatformError, SerializationError, SocketError, RegistryError, RecorderError]`, (4) Mark complete [x]

---

### Task 2: Implement Mutex Poison Recovery Utilities

- [x] **2. Implement Mutex Poison Recovery Utilities**
  - **File**: `keyrx_daemon/src/platform/recovery.rs` (new)
  - **Purpose**: Create helper functions for safe mutex access with poison recovery, enabling graceful degradation instead of panics.
  - **Requirements**: FR2, NFR2, NFR4
  - **Leverage**: Standard library `std::sync::Mutex`, `PoisonError` handling patterns
  - **Prompt**:

    **Role**: Rust Concurrency Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Create a new file `keyrx_daemon/src/platform/recovery.rs` with mutex poison recovery utilities.

    **Functions to Implement**:

    1. **recover_lock**:
       ```rust
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
               // Use poisoned guard (data may be inconsistent but accessible)
               Ok(poison_error.into_inner())
           })
       }
       ```

    2. **recover_lock_with_context**:
       ```rust
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

    **Module Declaration**: Add `pub mod recovery;` to `keyrx_daemon/src/platform/mod.rs`

    **Tests**: Create comprehensive tests in `recovery.rs` (in `#[cfg(test)] mod tests`) verifying:
    - Normal lock acquisition succeeds
    - Poisoned mutex recovery succeeds
    - Warning logged on poison (use test logging capture)
    - Subsequent operations succeed after recovery
    - Concurrent access works after recovery
    - Context string included in error message

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, comprehensive rustdoc with examples
  | **Success**: ✅ 2 recovery functions implemented, ✅ Unit tests with ≥100% coverage including poison scenarios, ✅ Rustdoc examples compile, ✅ Zero clippy warnings
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `functions: [recover_lock, recover_lock_with_context]`, (4) Mark complete [x]

---

### Task 3: Implement Binary Format Validation Helpers

- [x] **3. Implement Binary Format Validation Helpers**
  - **File**: `keyrx_compiler/src/serialize.rs` (modify existing)
  - **Purpose**: Add validation functions for .krx binary format parsing to replace `try_into().unwrap()` patterns with proper error handling.
  - **Requirements**: FR3, NFR2, NFR4
  - **Leverage**: Existing `MAGIC` and `VERSION` constants in `serialize.rs`
  - **Prompt**:

    **Role**: Binary Format Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Add validation helper functions to `keyrx_compiler/src/serialize.rs` for safe binary format parsing.

    **Functions to Implement**:

    1. **validate_magic**:
       ```rust
       /// Validates magic number in binary format
       ///
       /// # Errors
       ///
       /// Returns `SerializationError::InvalidMagic` if magic number doesn't match
       /// Returns `SerializationError::InvalidSize` if buffer too small
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
       ```

    2. **validate_version**:
       ```rust
       /// Validates version number in binary format
       ///
       /// # Errors
       ///
       /// Returns `SerializationError::InvalidVersion` if version doesn't match
       /// Returns `SerializationError::InvalidSize` if buffer too small
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
       ```

    3. **validate_size**:
       ```rust
       /// Validates that buffer has expected size
       fn validate_size(bytes: &[u8], expected: usize) -> Result<(), SerializationError> {
           if bytes.len() < expected {
               return Err(SerializationError::InvalidSize {
                   expected,
                   found: bytes.len(),
               });
           }
           Ok(())
       }
       ```

    **Import**: Add `use crate::error::SerializationError;` at top of file (assuming moved to keyrx_compiler/src/error.rs, or re-export from keyrx_daemon)

    **Tests**: Create unit tests verifying:
    - Valid magic/version passes validation
    - Invalid magic number detected with correct expected/found values
    - Invalid version detected with correct expected/found values
    - Truncated buffer detected
    - Corrupted data (wrong slice size) detected

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, comprehensive error messages with context
  | **Success**: ✅ 3 validation functions implemented, ✅ Unit tests with ≥100% coverage including edge cases, ✅ Error messages include expected and found values, ✅ Zero clippy warnings
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `functions: [validate_magic, validate_version, validate_size]`, (4) Mark complete [x]

---

### Task 4: Add From Conversions to DaemonError

- [x] **4. Add From Conversions to DaemonError**
  - **File**: `keyrx_daemon/src/error.rs` (modify existing)
  - **Purpose**: Enable automatic error propagation from module-specific errors to top-level DaemonError using the `?` operator.
  - **Requirements**: FR10, NFR3, NFR6
  - **Leverage**: Existing `DaemonError` enum definition
  - **Prompt**:

    **Role**: Rust Error Propagation Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Add `From<T>` implementations to `keyrx_daemon/src/error.rs` for automatic error conversion.

    **From Implementations to Add**:

    1. Extend `DaemonError` enum with new variants:
       ```rust
       pub enum DaemonError {
           // ... existing variants ...

           #[error("Platform error: {0}")]
           Platform(#[from] PlatformError),

           #[error("Serialization error: {0}")]
           Serialization(#[from] SerializationError),

           #[error("Socket error: {0}")]
           Socket(#[from] SocketError),

           #[error("Registry error: {0}")]
           Registry(#[from] RegistryError),

           #[error("Recorder error: {0}")]
           Recorder(#[from] RecorderError),
       }
       ```

    2. Verify `thiserror`'s `#[from]` attribute generates `From` implementations automatically

    3. Ensure error chain preserves context through conversions

    **Tests**: Create unit tests verifying:
    - PlatformError converts to DaemonError::Platform
    - SerializationError converts to DaemonError::Serialization
    - SocketError converts to DaemonError::Socket
    - RegistryError converts to DaemonError::Registry
    - RecorderError converts to DaemonError::Recorder
    - Error context preserved through conversion
    - Display implementation shows full error chain

  | **Restrictions**: Use `thiserror`'s `#[from]` attribute for automatic implementation, preserve all existing DaemonError variants
  | **Success**: ✅ 5 new DaemonError variants added, ✅ From conversions working via #[from] attribute, ✅ Unit tests verify conversion, ✅ Error context preserved, ✅ Zero clippy warnings
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `integrations: [error type conversions from modules to DaemonError]`, (4) Mark complete [x]

---

### Task 5: Write Unit Tests for Error Types

- [x] **5. Write Unit Tests for Error Types**
  - **File**: `keyrx_daemon/src/error.rs` (modify existing, add test module)
  - **Purpose**: Achieve ≥100% test coverage on all error types to ensure correctness and prevent regressions.
  - **Requirements**: NFR4, NFR6
  - **Leverage**: Existing test patterns in codebase
  - **Prompt**:

    **Role**: Rust Test Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Create comprehensive unit tests for all error types in `keyrx_daemon/src/error.rs`.

    **Test Categories**:

    1. **Error Construction Tests** (verify all variants constructible):
       ```rust
       #[test]
       fn test_platform_error_construction() {
           let err = PlatformError::Poisoned("test".into());
           assert!(matches!(err, PlatformError::Poisoned(_)));

           let err = PlatformError::InitializationFailed("test".into());
           assert!(matches!(err, PlatformError::InitializationFailed(_)));
       }
       ```

    2. **Display Implementation Tests** (verify error messages):
       ```rust
       #[test]
       fn test_serialization_error_display() {
           let err = SerializationError::InvalidMagic {
               expected: 0x4B5258_00,
               found: 0xFFFFFFFF,
           };
           let msg = err.to_string();
           assert!(msg.contains("0x04b25800"));
           assert!(msg.contains("0xffffffff"));
       }
       ```

    3. **From Conversion Tests**:
       ```rust
       #[test]
       fn test_platform_error_to_daemon_error() {
           let platform_err = PlatformError::Poisoned("mutex".into());
           let daemon_err: DaemonError = platform_err.into();
           assert!(matches!(daemon_err, DaemonError::Platform(_)));
       }
       ```

    4. **Error Trait Tests**:
       ```rust
       #[test]
       fn test_error_trait_implemented() {
           let err = PlatformError::Poisoned("test".into());
           let _: &dyn std::error::Error = &err;  // Compiles if Error implemented
       }
       ```

    **Coverage Target**: ≥100% coverage on all new error types (measured via `cargo tarpaulin`)

  | **Restrictions**: Tests in `#[cfg(test)] mod tests`, use descriptive test names, cover all variants
  | **Success**: ✅ All error variants tested, ✅ Display messages verified, ✅ From conversions verified, ✅ Coverage ≥100% on error types, ✅ All tests pass
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `statistics: { linesAdded: ~200, linesRemoved: 0 }`, (4) Mark complete [x]

---

## Phase 2: Critical Fixes (Tasks 6-8)

**Goal**: Eliminate critical panic points (P0 priority)

### Task 6: Replace lock().unwrap() in Windows Platform

- [x] **6. Replace lock().unwrap() in Windows Platform**
  - **File**: `keyrx_daemon/src/platform/windows/rawinput.rs`
  - **Purpose**: Eliminate mutex poison panics in Windows message handler hot path by using poison-aware locking.
  - **Requirements**: FR2, NFR2, NFR4
  - **Leverage**: `recover_lock_with_context` from Task 2
  - **Prompt**:

    **Role**: Windows Platform Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Replace all `lock().unwrap()` calls in `keyrx_daemon/src/platform/windows/rawinput.rs` with poison-aware recovery.

    **Locations to Fix** (8 occurrences):
    - Line 91: Windows message handler (CRITICAL - hot path)
    - Line 102: Hook callback (CRITICAL - hot path)
    - Line 267: Device registration
    - Line 275: Device unregistration
    - Lines 568, 576, 584: Test assertions

    **Replacement Pattern**:

    Before:
    ```rust
    let mut context_guard = bridge_context.lock().unwrap();
    ```

    After (production code):
    ```rust
    use crate::platform::recovery::recover_lock_with_context;

    let mut context_guard = recover_lock_with_context(
        &bridge_context,
        "Windows message handler"
    )?;
    ```

    After (test code - can keep unwrap if isolated):
    ```rust
    assert!(bridge_context.lock().unwrap().is_some());
    // OR use expect with context:
    assert!(bridge_context.lock()
        .expect("Test mutex should not be poisoned")
        .is_some());
    ```

    **Function Signature Changes**: Functions using `?` operator must return `Result<T, PlatformError>` or compatible type

    **Tests**: Add integration test verifying:
    - Message handler continues after mutex poison
    - Subsequent lock acquisitions succeed
    - Warning logged on poison

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, preserve existing behavior in happy path
  | **Success**: ✅ 0 production lock().unwrap() in rawinput.rs, ✅ All tests pass, ✅ Integration test verifies recovery, ✅ Performance unchanged (benchmark if possible)
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [rawinput.rs], statistics: { linesRemoved: 8 }`, (4) Mark complete [x]

---

### Task 7: Replace lock().unwrap() in WASM API

- [x] **7. Replace lock().unwrap() in WASM API**
  - **File**: `keyrx_core/src/wasm/mod.rs`
  - **Purpose**: Eliminate mutex poison panics in WASM API calls by using poison-aware locking.
  - **Requirements**: FR2, NFR2, NFR4
  - **Leverage**: `recover_lock` function pattern (adapt for keyrx_core)
  - **Prompt**:

    **Role**: WASM Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Replace all `lock().unwrap()` calls in `keyrx_core/src/wasm/mod.rs` with poison-aware recovery.

    **Locations to Fix** (10 occurrences):
    - Lines 105, 119, 130, 144: CONFIG_STORE access
    - Lines 224, 243, 279: Event simulation
    - Other lock unwraps as identified

    **Option 1 - Create recovery helper in keyrx_core**:
    ```rust
    // keyrx_core/src/lib.rs or keyrx_core/src/error.rs
    fn recover_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, WasmError> {
        mutex.lock().or_else(|poison_error| {
            #[cfg(feature = "std")]
            log::warn!("WASM mutex poisoned, recovering");
            Ok(poison_error.into_inner())
        })
    }
    ```

    **Option 2 - Use expect with informative messages**:
    ```rust
    let mut store = CONFIG_STORE.lock()
        .expect("WASM CONFIG_STORE mutex poisoned - check for panics in WASM API calls");
    ```

    **Recommendation**: Use Option 1 if keyrx_core has error types, otherwise Option 2 with detailed expect messages.

    **Tests**: Add unit tests verifying:
    - WASM API calls succeed after poison
    - Config store remains accessible
    - No panic on concurrent access after poison

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, maintain WASM compatibility (no_std aware)
  | **Success**: ✅ 0 production lock().unwrap() in wasm/mod.rs, ✅ All tests pass, ✅ WASM builds successfully, ✅ No panic on poison
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [wasm/mod.rs], statistics: { linesRemoved: 10 }`, (4) Mark complete [x]

---

### Task 8: Replace try_into().unwrap() in Serialization

- [x] **8. Replace try_into().unwrap() in Serialization**
  - **File**: `keyrx_compiler/src/serialize.rs`
  - **Purpose**: Eliminate crashes on corrupted .krx files by validating binary format instead of unwrapping conversions.
  - **Requirements**: FR3, NFR2, NFR4
  - **Leverage**: `validate_magic`, `validate_version`, `validate_size` from Task 3
  - **Prompt**:

    **Role**: Binary Format Security Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Replace all `try_into().unwrap()` calls in `keyrx_compiler/src/serialize.rs` with validation helpers.

    **Locations to Fix** (6 occurrences):
    - Line 115: Magic number parsing
    - Line 124: Version number parsing
    - Line 133: Size validation
    - Line 163: Additional size check
    - Line 237: Data slice conversion
    - Line 244: Final validation

    **Replacement Pattern**:

    Before:
    ```rust
    let magic_array: [u8; 4] = magic.try_into().unwrap();
    let version = u32::from_le_bytes(version_bytes.try_into().unwrap());
    ```

    After:
    ```rust
    validate_magic(&file_bytes[0..4])?;
    let magic_array: [u8; 4] = file_bytes[0..4]
        .try_into()
        .map_err(|_| SerializationError::CorruptedData(
            "Invalid magic number slice".to_string()
        ))?;

    validate_version(&file_bytes[4..8])?;
    let version_bytes: [u8; 4] = file_bytes[4..8]
        .try_into()
        .map_err(|_| SerializationError::CorruptedData(
            "Invalid version slice".to_string()
        ))?;
    let version = u32::from_le_bytes(version_bytes);
    ```

    **Function Signature Changes**: Functions must return `Result<T, SerializationError>` instead of bare types

    **Tests**: Add integration tests with malformed .krx files:
    - Truncated file (< 4 bytes)
    - Wrong magic number
    - Unsupported version
    - Corrupted data section
    - Zero-length file

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, comprehensive error messages with hex formatting for magic/version
  | **Success**: ✅ 0 try_into().unwrap() in serialize.rs, ✅ All tests pass, ✅ Integration tests verify malformed file handling, ✅ User-friendly error messages
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [serialize.rs], statistics: { linesRemoved: 6 }`, (4) Mark complete [x]

---

## Phase 3: High Priority Fixes (Tasks 9-11)

**Goal**: Fix high-priority error handling gaps (P1)

### Task 9: Replace IPC Socket unwraps with State Machine

- [x] **9. Replace IPC Socket unwraps with State Machine**
  - **File**: `keyrx_daemon/src/ipc/unix_socket.rs`
  - **Purpose**: Prevent socket operation panics by implementing explicit connection state tracking and validation.
  - **Requirements**: FR4, NFR3, NFR4
  - **Leverage**: Existing `UnixSocketServer` struct
  - **Prompt**:

    **Role**: Network Programming Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Add connection state machine to `keyrx_daemon/src/ipc/unix_socket.rs` and replace unwraps with state validation.

    **State Machine**:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ConnectionState {
        Disconnected,
        Connecting,
        Connected,
    }
    ```

    **Struct Modification**:
    ```rust
    pub struct UnixSocketServer {
        stream: Option<UnixStream>,
        state: ConnectionState,
        socket_path: PathBuf,
    }
    ```

    **Method Updates**:

    1. **send**: Check state before accessing stream
       ```rust
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
       ```

    2. **bind**: Update state transitions
       ```rust
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
       ```

    **Tests**: Add unit tests verifying:
    - Cannot send while disconnected
    - Cannot bind while connected
    - State transitions correctly
    - Error messages include socket path

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, preserve existing public API surface
  | **Success**: ✅ 0 socket unwraps, ✅ State machine implemented, ✅ All tests pass, ✅ Error context includes paths, ✅ Logging at INFO level
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [unix_socket.rs], classes: [ConnectionState enum]`, (4) Mark complete [x]

---

### Task 10: Replace Runtime Initialization unwrap

- [x] **10. Replace Runtime Initialization unwrap**
  - **File**: `keyrx_daemon/src/main.rs`
  - **Purpose**: Provide helpful error message on tokio runtime creation failure instead of panic.
  - **Requirements**: FR5, NFR5
  - **Leverage**: Existing `DaemonError` type
  - **Prompt**:

    **Role**: Daemon Lifecycle Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Replace runtime creation unwrap at line 320 in `keyrx_daemon/src/main.rs` with proper error handling.

    **Location**: Line 320 (approximately)

    Before:
    ```rust
    let runtime = tokio::runtime::Runtime::new().unwrap();
    ```

    After:
    ```rust
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| {
            eprintln!("Failed to create async runtime: {}", e);
            eprintln!("Ensure your system has sufficient resources (threads, memory)");
            std::process::exit(1);
        })?;
    ```

    OR add variant to DaemonError:
    ```rust
    // In error.rs:
    #[error("Failed to create async runtime: {0}")]
    RuntimeCreationFailed(#[from] std::io::Error),

    // In main.rs:
    let runtime = tokio::runtime::Runtime::new()
        .map_err(DaemonError::RuntimeCreationFailed)?;
    ```

    **Error Message**: Must be actionable for end users (suggest resource check)

    **Exit Code**: 1 on failure (not panic exit code 101)

  | **Restrictions**: Function size ≤50 lines, error message user-friendly
  | **Success**: ✅ No unwrap on runtime creation, ✅ Helpful error message on failure, ✅ Exit code 1, ✅ Error logged to stderr
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [main.rs], statistics: { linesRemoved: 1 }`, (4) Mark complete [x]

---

### Task 11: Ensure Platform Trait Error Propagation

- [x] **11. Ensure Platform Trait Error Propagation**
  - **File**: `keyrx_daemon/src/platform/mod.rs` (trait definition and all implementations)
  - **Purpose**: Verify all Platform trait methods return Result and implementations propagate errors correctly.
  - **Requirements**: FR10, NFR3
  - **Leverage**: Existing Platform trait
  - **Prompt**:

    **Role**: Platform Abstraction Architect

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Audit and verify error propagation consistency across all Platform trait implementations.

    **Files to Audit**:
    - `keyrx_daemon/src/platform/mod.rs` (trait definition)
    - `keyrx_daemon/src/platform/linux/*.rs` (Linux implementation)
    - `keyrx_daemon/src/platform/windows/*.rs` (Windows implementation)
    - `keyrx_daemon/src/platform/mock.rs` (Mock implementation)

    **Verification Checklist**:
    1. All trait methods return `Result<T, PlatformError>` or compatible
    2. No implementation uses `unwrap()` or `expect()` in production code
    3. Lock access uses `recover_lock` helpers
    4. Errors include context (device name, operation, etc.)
    5. Error types convertible to `DaemonError` via `From`

    **Fixes Required**:
    - If trait method returns bare type, change to `Result<T, PlatformError>`
    - If implementation uses unwrap, replace with `?` operator
    - Add error context where missing
    - Update callers to handle Result

    **Tests**: Add integration tests verifying:
    - Linux platform errors propagate correctly
    - Windows platform errors propagate correctly
    - Mock platform errors testable
    - Error chain preserved through conversions

  | **Restrictions**: Preserve backward compatibility via From conversions if needed
  | **Success**: ✅ All Platform methods return Result, ✅ No unwraps in implementations, ✅ Error propagation verified, ✅ Integration tests pass
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [platform/mod.rs, platform/linux/*.rs, platform/windows/*.rs]`, (4) Mark complete [x]

---

## Phase 4: Medium Priority Fixes (Tasks 12-14)

**Goal**: Add resilience to non-critical components (P2)

### Task 12: Add Signal Handler Error Handling

- [x] **12. Add Signal Handler Error Handling**
  - **File**: `keyrx_daemon/src/daemon/signals/linux.rs`
  - **Purpose**: Log signal handler failures instead of panicking, enabling degraded mode operation.
  - **Requirements**: FR6, NFR5
  - **Leverage**: Existing signal handler code
  - **Prompt**:

    **Role**: Linux Signal Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Replace unwraps in `keyrx_daemon/src/daemon/signals/linux.rs` with logged errors.

    **Locations to Fix** (3 occurrences):
    - Line 203: Signal registration
    - Line 218: Signal registration
    - Line 227: Thread join

    **Replacement Pattern**:

    Before:
    ```rust
    signal_hook::flag::register(signal::SIGHUP, reload_flag.clone()).unwrap();
    ```

    After:
    ```rust
    if let Err(e) = signal_hook::flag::register(signal::SIGHUP, reload_flag.clone()) {
        log::error!("Failed to register SIGHUP handler: {}. Reload-on-SIGHUP disabled.", e);
        // Continue operation in degraded mode
    }
    ```

    For thread join:
    ```rust
    if let Err(e) = handle.join() {
        log::error!("Signal handler thread panicked: {:?}", e);
    }
    ```

    **Degraded Mode**: Document that daemon continues without signal handling if registration fails

  | **Restrictions**: Function size ≤50 lines, log at ERROR level, daemon continues operation
  | **Success**: ✅ No unwraps in signal handlers, ✅ Errors logged with context, ✅ Daemon continues on failure, ✅ Tests verify degraded mode
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [signals/linux.rs], statistics: { linesRemoved: 3 }`, (4) Mark complete [x]

---

### Task 13: Implement Device Registry Resilience

- [x] **13. Implement Device Registry Resilience**
  - **File**: `keyrx_daemon/src/config/device_registry.rs`
  - **Purpose**: Recover from corrupted registry files by creating empty registry instead of crashing.
  - **Requirements**: FR7, NFR4
  - **Leverage**: Existing `DeviceRegistry` struct and methods
  - **Prompt**:

    **Role**: Configuration Management Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Add error handling and recovery to device registry operations in `keyrx_daemon/src/config/device_registry.rs`.

    **Methods to Update**:

    1. **load**: Create empty registry on corruption
       ```rust
       pub fn load() -> Result<Self, RegistryError> {
           let path = Self::registry_path();

           match std::fs::read_to_string(&path) {
               Ok(contents) => {
                   match serde_json::from_str(&contents) {
                       Ok(registry) => {
                           log::debug!("Loaded device registry from {:?}", path);
                           Ok(registry)
                       }
                       Err(e) => {
                           log::warn!("Corrupted registry at {:?}: {}. Creating empty registry.", path, e);
                           let empty = Self::default();
                           empty.save()?;
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
               Err(e) => Err(RegistryError::FailedToLoad(e.kind())),
           }
       }
       ```

    2. **save**: Return RegistryError on failure
       ```rust
       pub fn save(&self) -> Result<(), RegistryError> {
           let path = Self::registry_path();
           let contents = serde_json::to_string_pretty(self)
               .map_err(|e| RegistryError::CorruptedRegistry(e.to_string()))?;

           std::fs::write(&path, contents)
               .map_err(|e| RegistryError::IOError(e.kind()))?;

           log::debug!("Saved device registry to {:?}", path);
           Ok(())
       }
       ```

    **Callers**: Update all callers to handle Result

    **Tests**: Add unit tests with:
    - Corrupted JSON file
    - Missing registry file
    - Write-protected directory
    - Recovery creates valid empty registry

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines, preserve registry data structure
  | **Success**: ✅ Registry recovers from corruption, ✅ Empty registry created on failure, ✅ All tests pass, ✅ Warning logged on corruption
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [device_registry.rs]`, (4) Mark complete [x]

---

### Task 14: Add Macro Recorder Error Handling

- [x] **14. Add Macro Recorder Error Handling**
  - **File**: `keyrx_daemon/src/macro_recorder.rs`
  - **Purpose**: Isolate test failures by making all recorder methods return Result instead of unwrapping.
  - **Requirements**: FR8, NFR4
  - **Leverage**: Existing `MacroRecorder` struct
  - **Prompt**:

    **Role**: Macro Recording Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Convert all `MacroRecorder` methods to return `Result<T, RecorderError>` in `keyrx_daemon/src/macro_recorder.rs`.

    **Methods to Update**:

    1. **start_recording**:
       ```rust
       pub fn start_recording(&mut self) -> Result<(), RecorderError> {
           if self.is_recording {
               return Err(RecorderError::AlreadyRecording);
           }

           self.is_recording = true;
           self.frames.clear();
           log::info!("Started macro recording");
           Ok(())
       }
       ```

    2. **stop_recording**:
       ```rust
       pub fn stop_recording(&mut self) -> Result<Vec<Frame>, RecorderError> {
           if !self.is_recording {
               return Err(RecorderError::NotRecording);
           }

           self.is_recording = false;
           log::info!("Stopped macro recording, captured {} frames", self.frames.len());
           Ok(std::mem::take(&mut self.frames))
       }
       ```

    3. **playback**:
       ```rust
       pub fn playback(&self, frame: usize) -> Result<Event, RecorderError> {
           self.frames.get(frame)
               .cloned()
               .ok_or(RecorderError::PlaybackFailed(frame))
       }
       ```

    **Test Updates**: Update test code to use `?` operator or explicit error handling

    **Tests**: Add unit tests verifying:
    - Cannot start while already recording
    - Cannot stop when not recording
    - Playback fails on invalid frame number
    - Error includes frame number in message

  | **Restrictions**: File size ≤500 lines, function size ≤50 lines
  | **Success**: ✅ All recorder methods return Result, ✅ Tests isolated from failures, ✅ All tests pass, ✅ Error messages include context
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [macro_recorder.rs]`, (4) Mark complete [x]

---

## Phase 5: Documentation and Cleanup (Tasks 15-17)

**Goal**: Document remaining unwraps and establish quality gates (P3)

### Task 15: Document Remaining Production unwraps

- [x] **15. Document Remaining Production unwraps**
  - **File**: All files with remaining production unwraps (≤60 total)
  - **Purpose**: Add safety rationale comments to legitimate unwraps to prevent future confusion.
  - **Requirements**: FR9, NFR5, NFR6
  - **Leverage**: Code review checklist
  - **Prompt**:

    **Role**: Code Documentation Specialist

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Add `// SAFETY:` comments to all remaining production unwraps explaining why they cannot fail.

    **Process**:

    1. Run grep to find remaining unwraps:
       ```bash
       rg '\.unwrap\(\)' --type rust --glob '!tests/' --glob '!**/test_*.rs'
       ```

    2. For each unwrap, determine category:
       - **Infallible by design**: Operation guaranteed to succeed
       - **Already validated**: Previous check ensures success
       - **Initialization**: Unwrap during program startup (acceptable)
       - **Test code**: In `#[cfg(test)]` modules (acceptable)

    3. Add comment with format:
       ```rust
       // SAFETY: <brief explanation why this cannot fail>
       let value = operation.unwrap();
       ```

    **Examples**:

    ```rust
    // SAFETY: rkyv Infallible deserialize - archived data is valid by construction
    let config = archived_config.deserialize(&mut Infallible).unwrap();

    // SAFETY: Already validated magic number above, slice is exactly 4 bytes
    let magic_array: [u8; 4] = validated_bytes.try_into().unwrap();

    // SAFETY: Program initialization - panic is acceptable if logger setup fails
    env_logger::init().unwrap();
    ```

    **Clippy Annotation**: Add `#[allow(clippy::unwrap_used)]` with justification above each legitimate unwrap

    **Code Review Checklist**: Update checklist to verify SAFETY comments on all unwraps

  | **Restrictions**: Target ≤60 production unwraps, all must have SAFETY comment
  | **Success**: ✅ All production unwraps documented, ✅ Clippy allows with justification, ✅ Comments explain safety rationale, ✅ Code review checklist updated
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesModified: [various], statistics: { linesAdded: ~60 comments }`, (4) Mark complete [x]

---

### Task 16: Create Pre-Commit Hook to Prevent New unwraps

- [x] **16. Create Pre-Commit Hook to Prevent New unwraps**
  - **File**: `.git/hooks/pre-commit` (extend existing), `scripts/check_unwraps.sh` (new)
  - **Purpose**: Block commits introducing new unwraps in production code to prevent regressions.
  - **Requirements**: NFR1, NFR6
  - **Leverage**: Existing pre-commit infrastructure
  - **Prompt**:

    **Role**: DevOps Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Create pre-commit hook script to detect and block new unwraps in production code.

    **Script**: `scripts/check_unwraps.sh`

    ```bash
    #!/bin/bash
    # Check for new unwrap() calls in production code

    set -e

    # Count unwraps in production code (exclude tests)
    UNWRAP_COUNT=$(rg '\.unwrap\(\)' --type rust \
        --glob '!tests/' \
        --glob '!**/test_*.rs' \
        --glob '!**/*_test.rs' \
        --count 2>/dev/null | awk -F: '{sum+=$2} END {print sum}')

    # Maximum allowed (current baseline)
    MAX_UNWRAPS=60

    if [ "$UNWRAP_COUNT" -gt "$MAX_UNWRAPS" ]; then
        echo "ERROR: Too many unwrap() calls in production code"
        echo "Found: $UNWRAP_COUNT, Maximum: $MAX_UNWRAPS"
        echo ""
        echo "Files with unwraps:"
        rg '\.unwrap\(\)' --type rust \
            --glob '!tests/' \
            --glob '!**/test_*.rs' \
            --files-with-matches
        echo ""
        echo "Please replace unwraps with proper error handling or add SAFETY comments"
        exit 1
    fi

    echo "✓ unwrap() count: $UNWRAP_COUNT / $MAX_UNWRAPS (OK)"
    exit 0
    ```

    **Hook Integration**: Add to `.git/hooks/pre-commit`:
    ```bash
    # Check unwrap count
    if ! scripts/check_unwraps.sh; then
        exit 1
    fi
    ```

    **CI Integration**: Add to `.github/workflows/ci.yml`:
    ```yaml
    - name: Check unwrap count
      run: scripts/check_unwraps.sh
    ```

  | **Restrictions**: Script must be POSIX-compatible, exit 1 on failure
  | **Success**: ✅ Script detects new unwraps, ✅ Pre-commit hook blocks commits, ✅ CI check fails on violation, ✅ Clear error messages
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesCreated: [scripts/check_unwraps.sh], filesModified: [.git/hooks/pre-commit]`, (4) Mark complete [x]

---

### Task 17: Write Error Handling Strategy ADR

- [ ] **17. Write Error Handling Strategy ADR**
  - **File**: `docs/error-handling-strategy.md` (new)
  - **Purpose**: Document error handling decisions for future maintainers and contributors.
  - **Requirements**: NFR5
  - **Leverage**: Design document from this spec
  - **Prompt**:

    **Role**: Technical Writer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Create Architecture Decision Record documenting error handling strategy.

    **Document Structure**:

    ```markdown
    # Error Handling Strategy

    ## Status
    Accepted (2025-12-31)

    ## Context
    KeyRx2 originally had 676 unwrap/expect calls in production code, causing panics on error conditions. This ADR documents the remediation strategy.

    ## Decision

    ### Error Type Hierarchy
    [Describe error type design: PlatformError, SerializationError, etc.]

    ### Recovery Strategies
    [Describe mutex poison recovery, validation patterns, state machines]

    ### Acceptable unwrap() Usage
    1. Test code (919 occurrences acceptable)
    2. Infallible operations with SAFETY comment
    3. Program initialization (panic acceptable)

    ### Quality Gates
    - Maximum 60 production unwraps (down from 676)
    - Pre-commit hook blocks new unwraps
    - 100% coverage on error types

    ## Consequences

    ### Positive
    - No critical panic points in hot paths
    - Graceful degradation on errors
    - Better error messages for users

    ### Negative
    - Slightly more verbose error handling code
    - Performance overhead <1% on error paths

    ## Alternatives Considered
    [Reference design.md section 5]

    ## References
    - Spec: `.spec-workflow/specs/unwrap-remediation/`
    - Audit: `.spec-workflow/specs/unwrap-remediation/audit-report.md`
    ```

    **Audience**: Future contributors and maintainers

  | **Restrictions**: Markdown format, include code examples, reference spec documents
  | **Success**: ✅ ADR document created, ✅ Covers all key decisions, ✅ Includes rationale, ✅ References spec documents
  | **After completing this task**: (1) Mark as in-progress [-], (2) Implement, (3) Use log-implementation tool with artifacts: `filesCreated: [docs/error-handling-strategy.md]`, (4) Mark complete [x]

---

## Phase 6: Validation (Tasks 18-20)

**Goal**: Verify all success criteria met

### Task 18: Run Full Test Suite

- [ ] **18. Run Full Test Suite**
  - **Purpose**: Verify all unit and integration tests pass after error handling changes.
  - **Requirements**: NFR3, NFR4
  - **Prompt**:

    **Role**: QA Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Execute complete test suite and verify all tests pass.

    **Commands**:
    ```bash
    # Unit tests
    cargo test --workspace

    # Integration tests
    cargo test --workspace --test '*'

    # Doc tests
    cargo test --workspace --doc

    # Generate report
    cargo test --workspace -- --format=json > test-results.json
    ```

    **Success Criteria**:
    - All unit tests pass (0 failures)
    - All integration tests pass (0 failures)
    - All doc tests pass (0 failures)
    - No ignored tests in production code
    - Test execution time ≤ baseline

    **Failure Handling**: If tests fail, fix issues and re-run until all pass

  | **Success**: ✅ All tests pass, ✅ No ignored production tests, ✅ Test report generated
  | **After completing this task**: (1) Mark as in-progress [-], (2) Execute tests, (3) Use log-implementation tool with artifacts: `statistics: { testsRun: <count>, testsPassed: <count> }`, (4) Mark complete [x]

---

### Task 19: Run Performance Benchmarks

- [ ] **19. Run Performance Benchmarks**
  - **Purpose**: Verify no performance regression from error handling changes.
  - **Requirements**: NFR2
  - **Prompt**:

    **Role**: Performance Engineer

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Execute Criterion benchmarks and compare to baseline.

    **Commands**:
    ```bash
    # Run benchmarks
    cargo bench --workspace

    # Compare to baseline (if saved)
    cargo bench --workspace -- --save-baseline after_unwrap_remediation
    ```

    **Key Benchmarks**:
    - Lock acquisition (normal vs recover_lock)
    - Error path overhead (Result propagation)
    - Binary format parsing (validation overhead)

    **Success Criteria**:
    - All benchmarks within 5% of baseline
    - Lock recovery overhead <10μs
    - Error path overhead <1%

    **Failure Handling**: If regression >5%, optimize hot paths and re-benchmark

  | **Success**: ✅ Benchmarks run successfully, ✅ No regression >5%, ✅ Baseline saved
  | **After completing this task**: (1) Mark as in-progress [-], (2) Execute benchmarks, (3) Use log-implementation tool with artifacts: `statistics: { benchmarksRun: <count>, performanceChange: <percent> }`, (4) Mark complete [x]

---

### Task 20: Run Coverage Analysis

- [ ] **20. Run Coverage Analysis**
  - **Purpose**: Verify test coverage targets met on modified code.
  - **Requirements**: NFR4
  - **Prompt**:

    **Role**: Test Coverage Analyst

    **Task**: Implement the task for spec `unwrap-remediation`, first run spec-workflow-guide to get the workflow guide then implement the task: Generate coverage report and verify targets met.

    **Commands**:
    ```bash
    # Generate coverage report
    cargo tarpaulin --workspace --out Html --out Json

    # Check specific modules
    cargo tarpaulin --workspace --packages keyrx_daemon --out Json
    ```

    **Coverage Targets**:
    - Error types: ≥100%
    - Recovery functions: ≥100%
    - Modified modules: ≥90%
    - Overall workspace: ≥80%

    **Success Criteria**:
    - All targets met
    - No uncovered critical paths
    - Report saved to `tarpaulin-report.html`

    **Failure Handling**: If coverage below targets, add tests and re-measure

  | **Success**: ✅ Coverage targets met, ✅ Report generated, ✅ No uncovered critical paths
  | **After completing this task**: (1) Mark as in-progress [-], (2) Execute coverage, (3) Use log-implementation tool with artifacts: `statistics: { coveragePercent: <percent> }`, (4) Mark complete [x]

---

## Summary

**Total Tasks**: 20
**Critical (P0)**: 3 tasks (Phase 2)
**High (P1)**: 3 tasks (Phase 3)
**Medium (P2)**: 3 tasks (Phase 4)
**Low (P3)**: 3 tasks (Phase 5)
**Foundation**: 5 tasks (Phase 1)
**Validation**: 3 tasks (Phase 6)

**Estimated Effort**: 40-60 hours (2-3 hours per task average)

**Success Metrics**:
- Production unwraps: 676 → ≤60 (91% reduction)
- Critical unwraps: 18 → 0 (100% elimination)
- Test coverage: ≥90% on modified code
- Performance: No regression >5%

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-31 | Claude (debt-to-spec) | Initial tasks from design |

---

## Notes

- Tasks can be implemented in order or parallelized where dependencies allow
- Each task includes detailed prompts for autonomous execution
- Use log-implementation tool after each task to track progress
- Mark tasks [-] when starting, [x] when complete
- All tasks follow autonomous-spec-prep enhancement patterns
