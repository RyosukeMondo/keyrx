# Requirements Document

## Introduction

The KeyRx codebase currently uses global singleton services (`lazy_static!`) which prevents proper unit testing, dependency injection, and parallel test execution. This spec addresses the #1 priority improvement: implementing proper dependency injection to enable fast, isolated unit tests and maintainable code architecture.

**Problem Statement:**
- Global singletons in `core/src/api.rs` (DEVICE_SERVICE, PROFILE_SERVICE, RUNTIME_SERVICE) block all unit testing
- Services hardcode their dependencies, preventing mock injection
- No service traits exist, making interface-based programming impossible
- Tests must use real I/O (file system, config loading), making them slow (2.46s for full suite)
- Cannot test error paths without complex filesystem mocking
- Violates SOLID principles (Dependency Inversion, Interface Segregation)

**Value Proposition:**
- **50-80% faster tests** (2.46s → ~0.2s for full suite)
- **Enable true unit testing** - tests run in <1ms with no I/O
- **Parallel test execution** - no global state conflicts
- **Easy error path testing** - inject failing mocks
- **Better code architecture** - follows SOLID principles, adheres to user's DI mandate
- **Hours saved per week** - tests run hundreds of times per day during development

## Alignment with Product Vision

This feature supports the core development principles outlined in `~/.claude/CLAUDE.md`:

- **Code Quality Enforcement**: Implements mandatory DI as required by user guidelines
- **Architecture**: Enforces SOLID principles, specifically Dependency Inversion Principle (DIP)
- **Testability**: Removes testability blockers by eliminating hard dependencies
- **No testability blockers**: All external deps injected (APIs, DBs, queues) as mandated

## Requirements

### Requirement 1: Service Trait Abstractions

**User Story:** As a developer, I want service interfaces (traits) defined for all core services, so that I can write tests with mock implementations and follow interface-based programming patterns.

#### Acceptance Criteria

1. WHEN `DeviceServiceTrait` is defined THEN it SHALL include all public methods from `DeviceService` with identical signatures
2. WHEN `ProfileServiceTrait` is defined THEN it SHALL include all public methods from `ProfileService` with identical signatures
3. WHEN `RuntimeServiceTrait` is defined THEN it SHALL include all public methods from `RuntimeService` with identical signatures
4. WHEN existing services implement their traits THEN all methods SHALL delegate to existing implementations without behavior changes
5. IF a service trait method is `async` THEN it SHALL use `#[async_trait]` annotation
6. WHEN traits are defined THEN they SHALL be `Send + Sync` to support multi-threaded environments

**Success Metrics:**
- 3 new trait definitions created (`DeviceServiceTrait`, `ProfileServiceTrait`, `RuntimeServiceTrait`)
- All 3 existing service structs implement their corresponding traits
- Zero behavior changes to existing service logic
- All async methods properly annotated with `#[async_trait]`

### Requirement 2: Injectable API Context

**User Story:** As a developer, I want the API layer to accept injected service dependencies, so that I can provide real or mock implementations based on the execution context (production vs tests).

#### Acceptance Criteria

1. WHEN `ApiContext` struct is created THEN it SHALL hold trait objects (`Arc<dyn Trait>`) for all three services
2. WHEN `ApiContext::new()` is called with service implementations THEN it SHALL store them for method dispatch
3. WHEN `ApiContext::with_defaults()` is called THEN it SHALL create real service implementations for production use
4. WHEN an API method is called on `ApiContext` THEN it SHALL delegate to the appropriate injected service
5. IF backward compatibility is required THEN a global `GLOBAL_API` context SHALL be provided using `with_defaults()`
6. WHEN top-level convenience functions are called (e.g., `api::list_devices()`) THEN they SHALL delegate to `GLOBAL_API` for backward compatibility

**Success Metrics:**
- `ApiContext` struct created with 3 trait object fields
- `ApiContext::new()` constructor accepts injected dependencies
- `ApiContext::with_defaults()` provides production implementation
- All 28 API functions refactored to accept `&self` on `ApiContext`
- Backward-compatible top-level functions maintain existing API surface
- Zero breaking changes to existing code using `api.rs`

### Requirement 3: Constructor Injection for Services

**User Story:** As a developer, I want services to accept their dependencies through constructors, so that I can control what implementations are used and test services in isolation.

#### Acceptance Criteria

1. WHEN `DeviceService::new()` is called THEN it SHALL accept `registry: Option<DeviceRegistry>` and `bindings: DeviceBindings` as parameters
2. WHEN `ProfileService::new()` is called THEN it SHALL accept `config_manager: ConfigManager` as a parameter
3. WHEN `RuntimeService::new()` is called THEN it SHALL accept its dependencies as constructor parameters
4. IF a service has optional dependencies THEN they SHALL be `Option<T>` parameters with documented fallback behavior
5. WHEN a service method needs a dependency THEN it SHALL use the injected instance, NOT a global fallback
6. IF `DeviceService::get_registry()` fallback to global runtime exists THEN it SHALL be removed

**Success Metrics:**
- All service constructors accept their dependencies as parameters
- No hardcoded `::default()` or `::new()` calls within service constructors
- Fallback to `with_revolutionary_runtime()` in `DeviceService` removed
- `ProfileService` no longer hardcodes `ConfigManager::default()`
- All dependencies are explicitly visible in constructor signatures

### Requirement 4: Mock Service Implementations

**User Story:** As a test developer, I want mock implementations of all service traits, so that I can write fast, isolated unit tests without real I/O or external dependencies.

#### Acceptance Criteria

1. WHEN `MockDeviceService` is created THEN it SHALL implement `DeviceServiceTrait`
2. WHEN `MockProfileService` is created THEN it SHALL implement `ProfileServiceTrait`
3. WHEN `MockRuntimeService` is created THEN it SHALL implement `RuntimeServiceTrait`
4. WHEN a mock service is constructed with test data THEN it SHALL return that data from trait methods
5. WHEN a mock service is configured to fail THEN it SHALL return the specified error from trait methods
6. IF a mock is called multiple times THEN it SHALL support call counting and verification
7. WHEN a mock method is async THEN it SHALL return immediately without actual I/O

**Builder Pattern for Mocks:**
```rust
let mock = MockDeviceService::new()
    .with_devices(vec![test_device])
    .with_error_on_get(DeviceServiceError::DeviceNotFound("test".into()));
```

**Success Metrics:**
- 3 mock service structs created with builder patterns
- Each mock implements its corresponding trait
- Mocks support configurable success and error responses
- Mocks track method calls for verification
- All mocks are pure in-memory (zero I/O)

### Requirement 5: Test Migration and Coverage

**User Story:** As a project maintainer, I want existing tests updated to use dependency injection, so that the test suite runs faster and is more maintainable.

#### Acceptance Criteria

1. WHEN unit tests for API functions are written THEN they SHALL use `ApiContext` with injected mocks
2. WHEN unit tests for services are written THEN they SHALL inject mock dependencies into service constructors
3. IF a test requires real I/O THEN it SHALL be moved to `core/tests/integration/`
4. WHEN unit tests run THEN they SHALL complete in <1ms per test (no I/O)
5. WHEN the full test suite runs THEN it SHALL complete in <0.5s (target: 0.2s for 2,440 tests)
6. IF tests use `ApiContext` directly THEN they SHALL NOT use global singletons

**Test Example:**
```rust
#[tokio::test]
async fn test_list_devices_success() {
    let mock = Arc::new(MockDeviceService::new().with_devices(vec![test_device()]));
    let api = ApiContext::new(mock, mock_profile(), mock_runtime());

    let devices = api.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
    // ✅ No I/O, runs in <1ms
}
```

**Success Metrics:**
- All API unit tests migrated to use `ApiContext` with mocks
- All service unit tests inject mock dependencies
- Unit test directory contains only pure unit tests (no I/O)
- Integration tests properly separated and labeled
- Test suite runtime reduced from 2.46s to <0.5s
- Test coverage maintained or improved (target: 80% overall, 90% critical paths)

### Requirement 6: Documentation and Examples

**User Story:** As a new contributor, I want clear documentation and examples of dependency injection patterns, so that I can write new code following the project's architecture.

#### Acceptance Criteria

1. WHEN `core/src/services/README.md` is created THEN it SHALL document the DI pattern with examples
2. WHEN a developer reads the docs THEN they SHALL understand how to inject services in tests
3. WHEN example tests are provided THEN they SHALL demonstrate both success and error path testing
4. IF best practices exist THEN they SHALL be documented with code examples
5. WHEN FFI layer documentation is updated THEN it SHALL explain how FFI uses `ApiContext`

**Documentation Sections Required:**
- Overview of DI pattern in KeyRx
- How to create and use `ApiContext`
- How to write unit tests with mocks
- How to inject services into new code
- Migration guide for existing code
- Best practices and anti-patterns

**Success Metrics:**
- README.md created with comprehensive DI documentation
- At least 3 example tests provided (success, error, complex scenario)
- Migration guide helps developers update existing code
- FFI layer integration documented

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each trait file defines one service interface
- **Dependency Inversion Principle**: High-level modules (API) depend on abstractions (traits), not concrete implementations
- **Interface Segregation Principle**: Service traits expose only necessary public methods
- **Open/Closed Principle**: Services can be extended through trait implementations without modifying existing code
- **Modular Design**: Traits, implementations, and mocks are in separate modules
- **Clear Interfaces**: Trait definitions serve as contracts between layers

**File Structure:**
```
core/src/
├── services/
│   ├── traits.rs          # Service trait definitions
│   ├── device.rs          # DeviceService (implements DeviceServiceTrait)
│   ├── profile.rs         # ProfileService (implements ProfileServiceTrait)
│   ├── runtime.rs         # RuntimeService (implements RuntimeServiceTrait)
│   └── mocks.rs           # Mock implementations for testing
├── api.rs                 # ApiContext with injected dependencies
└── ...
```

### Performance

- **Test Runtime**: Full unit test suite SHALL complete in <0.5s (target: 0.2s)
- **Individual Test Speed**: Pure unit tests SHALL complete in <1ms each
- **Zero Overhead**: Trait object dispatch SHALL add <1ns overhead (vtable lookup)
- **Memory Efficiency**: `Arc<dyn Trait>` SHALL add only 16 bytes per service reference
- **Compilation Time**: Trait definitions SHALL NOT significantly impact compile times
- **Parallel Execution**: Tests SHALL run in parallel without global state conflicts

**Benchmark Targets:**
```
Before DI: 2,440 tests in 2.46s = ~1ms/test (with I/O)
After DI:  2,440 tests in 0.2s = 0.08ms/test (pure unit tests)
```

### Reliability

- **Backward Compatibility**: Existing code using `api::list_devices()` SHALL continue working
- **Zero Behavior Changes**: Service logic SHALL remain identical after refactoring
- **Error Handling**: All error paths SHALL behave identically before and after
- **Type Safety**: Rust's type system SHALL prevent injection of incompatible implementations
- **Thread Safety**: All service traits SHALL be `Send + Sync`

### Testability

- **100% Mockable**: Every service SHALL have a mock implementation
- **Isolated Tests**: Unit tests SHALL NOT depend on external state or I/O
- **Deterministic**: Mock responses SHALL be predictable and configurable
- **Error Simulation**: Mocks SHALL easily simulate error conditions
- **Call Verification**: Mocks SHALL support verifying method calls and arguments

### Maintainability

- **Clear Contracts**: Trait definitions serve as API documentation
- **Easy Mocking**: Builder pattern makes mock configuration intuitive
- **Refactoring Safety**: Trait changes trigger compile errors in all implementations
- **Code Reuse**: Mock implementations reduce test boilerplate
- **Migration Path**: Backward-compatible global API allows gradual migration

### Security

- **No Privilege Escalation**: Mocks SHALL NOT bypass security checks in real implementations
- **Secrets Handling**: Mock services SHALL NOT log or expose real credentials
- **Input Validation**: Service implementations SHALL validate inputs regardless of injection method

### Usability (Developer Experience)

- **Simple API**: `ApiContext::new(device, profile, runtime)` is straightforward
- **Obvious Defaults**: `ApiContext::with_defaults()` for production use
- **Test Clarity**: Mock builder pattern is self-documenting
- **Error Messages**: Trait object casting failures SHALL produce helpful error messages
- **IDE Support**: Traits SHALL provide autocomplete for all service methods
