# Requirements Document

## Introduction

The FFI (Foreign Function Interface) layer between Rust core and Flutter UI has grown organically to 10 export modules (~3K LOC) with repetitive patterns, global state management via `OnceLock<Mutex<...>>`, and 28+ public FFI functions. This spec addresses architectural debt by introducing a trait-based auto-generation system that reduces boilerplate by ~70%, eliminates global state singletons, and provides a consistent, type-safe API surface.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Reduces FFI overhead through zero-copy patterns and eliminates mutex contention
- **CLI First, GUI Later**: Clean FFI contract enables both CLI and Flutter to consume the same API
- **Testable Configs**: Isolated FFI state enables parallel test execution without global state pollution

Per tech.md: "No Global State: All instances are self-contained structs" and "Dependency Injection: All external dependencies injected for testability"

## Requirements

### Requirement 1: FFI Trait System

**User Story:** As a Rust developer, I want to define FFI exports through traits, so that I can add new FFI functions without writing boilerplate callback registration code.

#### Acceptance Criteria

1. WHEN a struct implements `FfiExportable` trait THEN the system SHALL generate C-ABI wrapper functions for each method
2. IF a method returns `Result<T, E>` THEN the system SHALL serialize errors to JSON with `error:` prefix
3. WHEN a method accepts callback parameters THEN the system SHALL manage callback lifecycle automatically
4. IF a trait method is added THEN the FFI export SHALL be available without modifying multiple files

### Requirement 2: Callback Registry Consolidation

**User Story:** As a Flutter developer, I want a single callback registration mechanism, so that I don't need to call separate `keyrx_on_X_progress`, `keyrx_on_X_duplicate`, etc. functions for each domain.

#### Acceptance Criteria

1. WHEN registering callbacks THEN the system SHALL accept an event type enum and callback function pair
2. IF multiple callbacks are registered for the same event type THEN the latest SHALL replace the previous
3. WHEN an event occurs THEN the system SHALL invoke the registered callback with JSON payload
4. IF no callback is registered for an event type THEN the system SHALL silently discard the event

### Requirement 3: State Management Without Global Statics

**User Story:** As a test developer, I want FFI state to be instance-scoped, so that I can run FFI tests in parallel without state interference.

#### Acceptance Criteria

1. WHEN creating an FFI context THEN the system SHALL return an opaque handle
2. IF operations are performed with a handle THEN the system SHALL use only that handle's state
3. WHEN disposing a handle THEN the system SHALL clean up all associated resources
4. IF tests run in parallel THEN each test SHALL have isolated FFI state

### Requirement 4: Unified Response Format

**User Story:** As a Flutter developer, I want consistent JSON response formats, so that I can parse FFI responses with a single deserializer.

#### Acceptance Criteria

1. WHEN an FFI function succeeds THEN it SHALL return `ok:{...json payload...}`
2. IF an FFI function fails THEN it SHALL return `error:{code: string, message: string, details?: object}`
3. WHEN returning owned strings THEN the system SHALL provide `keyrx_free_string` for deallocation
4. IF a function returns complex data THEN it SHALL use the same JSON serialization as callbacks

### Requirement 5: Domain Module Consolidation

**User Story:** As a maintainer, I want FFI exports organized by domain, so that related functionality is colocated and easy to maintain.

#### Acceptance Criteria

1. WHEN organizing FFI exports THEN the system SHALL group by domain (engine, discovery, validation, device, testing, analysis, diagnostics)
2. IF a domain has fewer than 3 functions THEN it MAY be consolidated with a related domain
3. WHEN a new domain is added THEN it SHALL follow the established trait pattern
4. IF exports are deprecated THEN the system SHALL emit compile-time warnings

### Requirement 6: Dart Binding Synchronization

**User Story:** As a full-stack developer, I want Dart FFI bindings auto-generated from Rust, so that I never have to manually keep them in sync.

#### Acceptance Criteria

1. WHEN Rust FFI exports change THEN the build system SHALL regenerate Dart bindings
2. IF a Rust function signature changes THEN the Dart binding SHALL reflect the change
3. WHEN types are used in FFI THEN the system SHALL generate corresponding Dart types
4. IF bindings are out of sync THEN the build SHALL fail with a clear error message

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each FFI module handles one domain (discovery, validation, etc.)
- **Modular Design**: FfiExportable trait enables isolated, testable domain implementations
- **Dependency Management**: No cross-domain FFI dependencies except shared callback registry
- **Clear Interfaces**: Trait-based contracts define exact API surface

### Performance
- FFI call overhead SHALL NOT exceed 100 microseconds per call
- Callback invocations SHALL use zero-copy JSON serialization where possible
- State access SHALL NOT require global mutex locks for read operations

### Security
- All FFI inputs SHALL be validated before use (null checks, UTF-8 validation)
- Unsafe blocks SHALL be minimized and encapsulated in safe wrappers
- No raw pointer arithmetic outside of CString handling

### Reliability
- FFI panics SHALL be caught and converted to error responses
- All allocated memory SHALL have clear ownership and deallocation paths
- Tests SHALL achieve 90% coverage of FFI boundary code

### Usability
- Adding a new FFI function SHALL require < 10 lines of code
- Error messages SHALL include actionable context
- Documentation SHALL be auto-generated from trait definitions
