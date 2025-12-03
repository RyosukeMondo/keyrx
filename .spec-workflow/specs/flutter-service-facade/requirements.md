# Requirements Document

## Introduction

The Flutter UI layer has grown to 19 services in `ui/lib/services/` with complex interdependencies. Pages like `editor_page.dart` must inject 7+ services, making testing difficult and refactoring risky. This spec introduces a Service Facade pattern that provides a unified, simplified API surface while maintaining the flexibility of the underlying service architecture.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Visual > Abstract**: Clean service layer enables focused UI development without plumbing concerns
- **Testable Configs**: Single facade mock replaces 7+ service mocks in tests
- **Progressive Complexity**: Simple facade for common operations, direct service access for advanced cases

Per tech.md: "Dependency Injection: All external dependencies injected for testability" and the existing ServiceRegistry pattern.

## Requirements

### Requirement 1: Unified Service Facade

**User Story:** As a Flutter developer, I want to access common KeyRx operations through a single facade, so that I don't need to inject and coordinate 7+ services in each page.

#### Acceptance Criteria

1. WHEN a page needs engine, device, and script operations THEN it SHALL inject only `KeyrxFacade`
2. IF a facade method wraps multiple service calls THEN it SHALL coordinate them atomically
3. WHEN errors occur in underlying services THEN the facade SHALL translate them to user-friendly messages
4. IF a rare operation is needed THEN the developer MAY access underlying services through facade getters

### Requirement 2: Simplified Testing Interface

**User Story:** As a test developer, I want to mock a single facade instead of 7+ services, so that widget tests are easier to write and maintain.

#### Acceptance Criteria

1. WHEN testing a page THEN the test SHALL inject a `MockKeyrxFacade`
2. IF the facade interface changes THEN only the mock needs updating (not 7+ service mocks)
3. WHEN stubbing facade methods THEN the test SHALL use standard Mockito patterns
4. IF integration tests need real services THEN they SHALL use `KeyrxFacade.real()` factory

### Requirement 3: State Aggregation

**User Story:** As a Flutter developer, I want the facade to provide aggregated state streams, so that I can observe combined engine/device/script state in one subscription.

#### Acceptance Criteria

1. WHEN engine state changes THEN the facade SHALL emit an aggregated state update
2. IF device connection changes THEN the aggregated state SHALL include device status
3. WHEN script validation completes THEN the aggregated state SHALL include validation results
4. IF multiple state changes occur rapidly THEN the facade SHALL debounce emissions (100ms)

### Requirement 4: Operation Coordination

**User Story:** As a Flutter developer, I want the facade to handle multi-step operations, so that I don't need to manually sequence service calls.

#### Acceptance Criteria

1. WHEN starting the engine with a script THEN the facade SHALL validate → load → start in sequence
2. IF any step fails THEN the facade SHALL rollback partial changes and report the failure point
3. WHEN stopping the engine THEN the facade SHALL stop recording → stop engine → clean up state
4. IF an operation is in progress THEN calling another operation SHALL queue or reject appropriately

### Requirement 5: Error Translation

**User Story:** As a Flutter developer, I want technical errors translated to user-friendly messages, so that I can display them directly in the UI.

#### Acceptance Criteria

1. WHEN a Rust FFI error occurs THEN the facade SHALL translate it using `ErrorTranslator`
2. IF a network timeout occurs THEN the facade SHALL provide a retry-able error type
3. WHEN validation fails THEN the facade SHALL include structured validation errors
4. IF an unexpected error occurs THEN the facade SHALL log details and provide generic user message

### Requirement 6: Backward Compatibility with ServiceRegistry

**User Story:** As a maintainer, I want the facade to coexist with ServiceRegistry, so that migration can be incremental.

#### Acceptance Criteria

1. WHEN creating a facade THEN it SHALL accept a ServiceRegistry as dependency
2. IF legacy code uses ServiceRegistry directly THEN it SHALL continue to work unchanged
3. WHEN both facade and direct service access are used THEN state SHALL remain consistent
4. IF full migration completes THEN ServiceRegistry MAY be deprecated but not removed

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Facade handles coordination, services handle domain logic
- **Modular Design**: Each facade method is independently testable
- **Dependency Management**: Facade depends on ServiceRegistry, not individual services
- **Clear Interfaces**: Abstract `KeyrxFacade` interface enables mock injection

### Performance
- Facade method calls SHALL add < 1ms overhead over direct service calls
- State aggregation SHALL use lazy evaluation (compute only when observed)
- Memory overhead SHALL be < 1MB for facade + state management

### Security
- Facade SHALL NOT expose internal service implementation details
- Error messages SHALL NOT leak sensitive information (file paths, stack traces)
- All public methods SHALL validate inputs before forwarding to services

### Reliability
- Facade SHALL handle service disposal gracefully
- Concurrent facade calls SHALL be thread-safe (Dart isolate-safe)
- Tests SHALL achieve 90% coverage of facade methods

### Usability
- Migrating a page to use facade SHALL require < 30 minutes
- Facade API SHALL be discoverable through IDE autocomplete
- Common operations (start/stop engine, load script) SHALL be single method calls
