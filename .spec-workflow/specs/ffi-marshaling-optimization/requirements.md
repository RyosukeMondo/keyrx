# Requirements Document

## Introduction

KeyRx's FFI layer has 11 export modules (3,667 LOC) with inconsistent marshaling patterns. Data structures crossing the Rust-Dart boundary repeat serialization logic, error conversion, and JSON handling. This creates maintenance burden, performance overhead, and inconsistency. This spec creates a unified marshaling layer.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Efficient data transfer to UI
- **Maintainability**: Consistent marshaling patterns
- **Developer Experience**: Clear FFI contracts

Per tech.md: "Clear Interfaces" for FFI

## Requirements

### Requirement 1: Unified Marshaling Trait

**User Story:** As a developer, I want consistent marshaling, so that FFI data transfer is predictable.

#### Acceptance Criteria

1. WHEN data crosses FFI boundary THEN it SHALL use FfiMarshaler trait
2. IF a type needs FFI export THEN it SHALL implement the trait
3. WHEN marshaling fails THEN clear error SHALL be returned
4. IF type is complex THEN incremental transfer SHALL be possible

### Requirement 2: Error Marshaling

**User Story:** As a developer, I want consistent error passing, so that Flutter handles all errors.

#### Acceptance Criteria

1. WHEN error crosses FFI THEN it SHALL use FfiError type
2. IF error has code THEN it SHALL be preserved across boundary
3. WHEN error has context THEN it SHALL be serialized
4. IF error is internal THEN details SHALL be sanitized

### Requirement 3: Large Data Transfer

**User Story:** As a developer, I want efficient large data transfer, so that recordings don't lag the UI.

#### Acceptance Criteria

1. WHEN data is large (>1MB) THEN streaming SHALL be available
2. IF shared memory is possible THEN it SHALL be used
3. WHEN transferring arrays THEN batch encoding SHALL be used
4. IF transfer fails THEN partial data SHALL be recoverable

### Requirement 4: Callback Consolidation

**User Story:** As a developer, I want unified callbacks, so that event handling is consistent.

#### Acceptance Criteria

1. WHEN events are sent to Flutter THEN single callback pattern SHALL be used
2. IF multiple event types exist THEN they SHALL share callback infrastructure
3. WHEN callback is registered THEN type safety SHALL be enforced
4. IF callback fails THEN fallback SHALL exist

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Marshaling Pattern**: One way to cross FFI
- **Type Safety**: Compile-time FFI contract verification
- **Separation of Concerns**: Marshaling separate from business logic

### Performance
- Marshaling overhead SHALL be < 100 microseconds for typical data
- Large transfers SHALL not block UI thread
- Memory copies SHALL be minimized

### Maintainability
- Adding new FFI type SHALL require < 10 LOC
- Marshaling code SHALL be generated where possible
- FFI contracts SHALL be documented
