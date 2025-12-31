# Requirements Document - Unwrap Remediation

## Overview

**Specification**: unwrap-remediation
**Created**: 2025-12-31
**Status**: Draft
**Priority**: Critical

**Problem Statement**: The KeyRx2 codebase contains 1,595 unwrap/expect calls, with 676 (42%) in production code. Critical panic points exist in hot code paths (daemon event loop, Windows platform handlers, WASM API, IPC operations) that could cause cascading failures and poor user experience.

**Success Criteria**: Reduce production unwrap/expect calls from 676 to ≤60 (91% reduction), eliminate all critical panic points, and establish error handling infrastructure for resilient operation.

---

## 1. Functional Requirements

### FR1: Custom Error Type Infrastructure (CRITICAL)
- **Description**: Create comprehensive error type enums for each module to enable proper error propagation
- **Acceptance Criteria**:
  - [ ] `PlatformError` enum with variants: `Poisoned`, `InitializationFailed`, `DeviceError`
  - [ ] `SerializationError` enum with variants: `InvalidMagic`, `InvalidVersion`, `InvalidSize`, `CorruptedData`
  - [ ] `SocketError` enum with variants: `BindFailed`, `ListenFailed`, `NotConnected`, `AlreadyConnected`
  - [ ] `RegistryError` enum with variants: `IOError`, `CorruptedRegistry`, `FailedToLoad`
  - [ ] `RecorderError` enum for macro recording failures
  - [ ] All error types implement `std::error::Error` and `Display`
  - [ ] Error types include context fields (file paths, line numbers, system error codes)
- **Priority**: Critical
- **Related Issues**: Architecture Gap 1 - Missing Custom Error Types

### FR2: Mutex Poison Recovery (CRITICAL)
- **Description**: Replace all `lock().unwrap()` patterns with poison-aware locking that recovers gracefully
- **Acceptance Criteria**:
  - [ ] 8 `lock().unwrap()` in `keyrx_daemon/src/platform/windows/rawinput.rs` replaced (lines 91, 102, 267, 275)
  - [ ] 10 `lock().unwrap()` in `keyrx_core/src/wasm/mod.rs` replaced (lines 105, 119, 130, 144, 224, 243, 279)
  - [ ] Implement `recover_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, PlatformError>` helper
  - [ ] Poisoned mutexes return `PlatformError::Poisoned` with context
  - [ ] Callers propagate error with `?` operator
  - [ ] Integration tests verify recovery behavior
- **Priority**: Critical (P0)
- **Related Issues**: Pattern 1 - Lock().Unwrap() in hot paths
- **Files Affected**: 2 files, 18 occurrences

### FR3: Binary Format Validation (CRITICAL)
- **Description**: Replace `try_into().unwrap()` in .krx file parsing with proper validation
- **Acceptance Criteria**:
  - [ ] 6 `try_into().unwrap()` in `keyrx_compiler/src/serialize.rs` replaced (lines 115, 124, 133, 163, 237, 244)
  - [ ] Magic number validation returns `SerializationError::InvalidMagic { expected, found }`
  - [ ] Version validation returns `SerializationError::InvalidVersion { expected, found }`
  - [ ] Size validation returns `SerializationError::InvalidSize { expected, found }`
  - [ ] Corrupted .krx files produce user-friendly error messages
  - [ ] Unit tests verify behavior on truncated/corrupted files
- **Priority**: Critical (P0)
- **Related Issues**: Pattern 2 - Try_into().Unwrap() in format parsing
- **Files Affected**: 1 file, 6 occurrences

### FR4: IPC Socket Error Handling (HIGH)
- **Description**: Replace socket operation unwraps with proper error propagation
- **Acceptance Criteria**:
  - [ ] 7 unwraps in `keyrx_daemon/src/ipc/unix_socket.rs` replaced (line 70 + others)
  - [ ] Implement connection state machine: `enum State { Disconnected, Connecting, Connected }`
  - [ ] `stream.as_mut()` checked with `ok_or(SocketError::NotConnected)?`
  - [ ] Bind failures include socket path and system error
  - [ ] Listen failures include system error code
  - [ ] Connection state transitions logged at INFO level
  - [ ] Integration tests verify error conditions
- **Priority**: High (P1)
- **Related Issues**: Architecture Gap 2 - No error propagation strategy
- **Files Affected**: 1 file, 7 occurrences

### FR5: Runtime Initialization Error Handling (HIGH)
- **Description**: Replace runtime creation unwrap with proper error reporting
- **Acceptance Criteria**:
  - [ ] `keyrx_daemon/src/main.rs:320` unwrap replaced with error propagation
  - [ ] Tokio runtime creation failure returns `DaemonError::RuntimeCreationFailed`
  - [ ] Error message includes helpful diagnostic: "Failed to create async runtime. Ensure system has sufficient resources."
  - [ ] Exit code 1 on failure (not panic)
  - [ ] Structured logging of failure context
- **Priority**: High (P1)
- **Related Issues**: Top violator #8
- **Files Affected**: 1 file, 1 occurrence

### FR6: Signal Handler Error Handling (MEDIUM)
- **Description**: Replace signal handler unwraps with logged errors
- **Acceptance Criteria**:
  - [ ] 3 unwraps in `keyrx_daemon/src/daemon/signals/linux.rs` replaced (lines 203, 218, 227)
  - [ ] Signal registration failures logged at ERROR level
  - [ ] Thread join failures logged with thread ID
  - [ ] Daemon continues operation if possible (degraded mode)
  - [ ] Integration tests verify signal handling resilience
- **Priority**: Medium (P2)
- **Related Issues**: Top violator #6
- **Files Affected**: 1 file, 3 occurrences

### FR7: Device Registry Resilience (MEDIUM)
- **Description**: Add error handling and recovery to device registry operations
- **Acceptance Criteria**:
  - [ ] Registry load failures create empty registry with warning
  - [ ] Corrupted registry files detected and repaired
  - [ ] All registry operations return `Result<T, RegistryError>`
  - [ ] File I/O errors include file path and error kind
  - [ ] Registry operations logged at DEBUG level
  - [ ] Unit tests verify recovery from corrupted files
- **Priority**: Medium (P2)
- **Related Issues**: Top violator #2 (test code), Architecture Gap 1
- **Files Affected**: 1 file, ~15 production occurrences

### FR8: Macro Recorder Error Handling (MEDIUM)
- **Description**: Replace recorder unwraps with proper error propagation
- **Acceptance Criteria**:
  - [ ] All recorder methods return `Result<T, RecorderError>`
  - [ ] Recording start failures logged with error context
  - [ ] Recording stop failures don't lose recorded data
  - [ ] Playback failures include frame number where failure occurred
  - [ ] Test suite isolates individual test failures
  - [ ] Unit tests verify error recovery
- **Priority**: Medium (P2)
- **Related Issues**: Top violator #1 (test code)
- **Files Affected**: 1 file, ~15 production occurrences

### FR9: Safe Unwrap Documentation (LOW)
- **Description**: Document remaining legitimate unwraps with safety rationale
- **Acceptance Criteria**:
  - [ ] `keyrx_daemon/src/daemon/state.rs:100` documented (rkyv Infallible deserialize)
  - [ ] All remaining production unwraps have comment explaining safety
  - [ ] Comment format: `// SAFETY: <why this cannot fail>`
  - [ ] Clippy `#[allow(clippy::unwrap_used)]` with justification
  - [ ] Code review checklist includes unwrap safety verification
- **Priority**: Low (P3)
- **Related Issues**: Top violator #7
- **Files Affected**: ~10 files with legitimate unwraps

### FR10: Error Propagation Consistency (MEDIUM)
- **Description**: Ensure all Platform trait implementations use consistent error handling
- **Acceptance Criteria**:
  - [ ] All `Platform` trait methods return `Result<T, PlatformError>`
  - [ ] Linux platform implementation propagates errors (no unwraps)
  - [ ] Windows platform implementation propagates errors (no unwraps)
  - [ ] Mock platform implementation propagates errors for testing
  - [ ] Error types are convertible to top-level `DaemonError`
  - [ ] Integration tests verify error propagation through trait boundaries
- **Priority**: Medium (P2)
- **Related Issues**: Architecture Gap 2 - No error propagation strategy

---

## 2. Non-Functional Requirements

### NFR1: Code Quality Metrics
- **Target Metrics**:
  - Production unwrap/expect count: ≤60 (from 676)
  - Test unwrap/expect count: <1000 (from 919, acceptable)
  - Critical unwrap count: 0 (from 18)
  - Lock().unwrap() count: 0 (from 18)
  - Try_into().unwrap() count: 0 (from 6)
- **Measurement**: Automated grep/ripgrep audit script
- **Enforcement**: Pre-commit hook blocks new unwraps in production code

### NFR2: Performance
- **Requirements**:
  - No performance regression in hot paths
  - Error path overhead: <1% of happy path
  - Lock recovery overhead: <10μs
  - Memory overhead: <1KB per error type enum
- **Measurement**: Criterion benchmarks before/after
- **Acceptance**: All benchmarks within 5% of baseline

### NFR3: Backward Compatibility
- **Requirements**:
  - Public API unchanged (Platform trait, DaemonError, etc.)
  - Error messages improved but structure preserved
  - CLI exit codes unchanged
  - Configuration format unchanged
  - .krx binary format unchanged (only validation improved)
- **Verification**: Integration tests pass without modification

### NFR4: Test Coverage
- **Requirements**:
  - New error types: 100% coverage
  - Error recovery paths: 90% coverage
  - Mutex poison recovery: 100% coverage (critical path)
  - Binary format validation: 100% coverage (security critical)
- **Measurement**: `cargo tarpaulin` on modified modules
- **Enforcement**: CI fails if coverage drops below targets

### NFR5: Documentation
- **Requirements**:
  - All error types have rustdoc with examples
  - All error variants documented with when they occur
  - Migration guide for downstream users (if any)
  - Architecture decision record (ADR) for error handling strategy
- **Deliverables**:
  - `docs/error-handling-strategy.md`
  - Rustdoc on all error types and recovery functions
  - Examples in docstrings

### NFR6: Maintainability
- **Requirements**:
  - Error types follow consistent naming: `[Module]Error`
  - Error messages follow template: `"Failed to [action]: [context]"`
  - All errors implement `From<std::io::Error>` where applicable
  - Error types are `#[non_exhaustive]` for future extension
- **Enforcement**: Code review checklist

---

## 3. Dependencies

### Tools Required
- Rust 1.70+ (for `?` operator in const contexts)
- `cargo-tarpaulin` (for coverage measurement)
- `cargo-audit` (for dependency security)
- `ripgrep` (for unwrap detection)

### Libraries Required
- `thiserror` = "1.0" (for error derive macros)
- Existing: `rkyv`, `evdev`, `windows-sys` (no changes)

### External Dependencies
- None (all error handling internal)

---

## 4. Success Metrics

### Phase 1: Critical Fixes (P0)
- [ ] 0 lock().unwrap() in production code (from 18)
- [ ] 0 try_into().unwrap() in production code (from 6)
- [ ] Custom error types implemented (5 types: PlatformError, SerializationError, SocketError, RegistryError, RecorderError)
- [ ] 100% coverage on error types and recovery functions

### Phase 2: High Priority Fixes (P1)
- [ ] IPC socket unwraps eliminated (7 occurrences)
- [ ] Runtime initialization unwrap eliminated (1 occurrence)
- [ ] Error propagation consistent across Platform trait (10 implementations)

### Phase 3: Medium Priority Fixes (P2)
- [ ] Signal handler unwraps handled gracefully (3 occurrences)
- [ ] Device registry resilience implemented (15 occurrences)
- [ ] Macro recorder error handling implemented (15 occurrences)

### Phase 4: Documentation and Cleanup (P3)
- [ ] All remaining production unwraps documented (≤60 with safety comments)
- [ ] Pre-commit hook prevents new unwraps
- [ ] Error handling strategy documented
- [ ] Code review checklist updated

### Final Validation
- [ ] Total production unwraps: ≤60 (91% reduction from 676)
- [ ] Zero clippy warnings on modified files
- [ ] All tests passing (919 test unwraps acceptable)
- [ ] No performance regression (benchmarks within 5%)
- [ ] Documentation complete (rustdoc, examples, ADR)

---

## 5. Out of Scope

**Explicitly NOT included in this specification:**
- ❌ Test code unwraps (919 acceptable, no action needed)
- ❌ Algorithmic improvements to reduce error frequency
- ❌ UI/UX changes for error display
- ❌ Internationalization of error messages
- ❌ Error telemetry/monitoring integration
- ❌ Error recovery strategies beyond graceful degradation
- ❌ Performance optimization unrelated to error handling

---

## 6. Acceptance Testing

### Test Plan

**Critical Path Tests (must pass):**
1. Mutex poison recovery: Panic in thread A, thread B recovers lock
2. Corrupted .krx file: Truncated file produces clear error message
3. Socket bind failure: Port already in use produces helpful error
4. Signal handler failure: SIGHUP registration failure doesn't crash daemon
5. Registry corruption: Corrupted JSON creates empty registry with warning

**Performance Tests:**
- Lock recovery overhead: <10μs (measured via Criterion)
- Error path overhead: <1% of happy path

**Integration Tests:**
- All existing tests pass without modification
- New error scenarios covered in integration test suite

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-31 | Claude (debt-to-spec) | Initial requirements from audit |

---

## Approval

**Status**: Pending Review

**Reviewers**:
- [ ] Technical Lead
- [ ] Platform Team
- [ ] QA Team

**Approval Date**: _________________
