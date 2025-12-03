# Requirements Document

## Introduction

This spec addresses critical code quality issues identified in the KeyRx codebase that block testability, violate architectural principles, and create maintenance burden. These issues must be resolved before implementing new features to ensure a solid foundation.

**Philosophy:** Clean code enables velocity. Fixing these issues now prevents compounding technical debt.

## Alignment with CLAUDE.md Guidelines

From CLAUDE.md:
- **Code Metrics**: Max 500 lines/file, Max 50 lines/function, 80% test coverage
- **Architecture**: SOLID, DI mandatory, SSOT, KISS, SLAP
- **All external deps injected**: APIs, DBs, queues
- **No testability blockers**: No globals, mockable dependencies

## Issue Categories

### Priority 1: Testability Blockers (Must Fix)
Issues that prevent proper unit testing:
- Global state requiring serial test execution
- Non-injectable dependencies
- Hardcoded service access

### Priority 2: SSOT Violations (Must Fix)
Multiple sources of truth causing data inconsistency:
- Duplicate state representations
- Unsynchronized data copies

### Priority 3: SLAP/SRP Violations (Should Fix)
Code organization issues:
- Functions mixing abstraction levels
- Files with too many responsibilities

### Priority 4: DRY Violations (Should Fix)
Code duplication creating maintenance burden

---

## Requirements

### Requirement 1: Eliminate Global Runtime State (Rust)

**User Story:** As a developer, I want to run tests in parallel without interference, so that CI is fast and reliable.

#### Acceptance Criteria

1. WHEN tests run, THEN they SHALL execute in parallel without `#[serial]` attribute
2. WHEN a test needs a runtime, THEN it SHALL create its own isolated instance
3. WHEN the engine initializes, THEN runtime SHALL be passed via dependency injection
4. WHEN bypass mode is tested, THEN each test SHALL have independent state
5. WHEN FFI callbacks are tested, THEN they SHALL be mockable per test

### Requirement 2: Injectable Services in Flutter (Flutter)

**User Story:** As a developer, I want to inject mock services into widgets, so that I can test UI components in isolation.

#### Acceptance Criteria

1. WHEN a page needs a service, THEN it SHALL accept it via constructor parameter
2. WHEN a page is constructed, THEN required services SHALL NOT be nullable
3. WHEN testing a page, THEN mock services SHALL be injectable without Provider
4. WHEN TextEditingControllers are needed, THEN they SHALL be injectable or created in a testable pattern
5. WHEN services are accessed, THEN it SHALL NOT be via Provider.of in initState

### Requirement 3: Single Source of Truth for State (Flutter)

**User Story:** As a developer, I want one canonical source for each piece of state, so that data is always consistent.

#### Acceptance Criteria

1. WHEN layer configuration exists, THEN it SHALL be stored ONLY in AppState
2. WHEN editor pages need layers, THEN they SHALL read from AppState
3. WHEN mappings are edited, THEN a single MappingRepository SHALL be the source
4. WHEN switching between editors, THEN mappings SHALL persist correctly
5. WHEN classification results stream, THEN a single buffer SHALL exist in service layer

### Requirement 4: Consolidate Rust State Representations

**User Story:** As a developer, I want one state model, so that changes don't require updating multiple structs.

#### Acceptance Criteria

1. WHEN engine state is serialized, THEN ONE struct SHALL represent it
2. WHEN `EngineState` and `PendingDecisionState` overlap, THEN they SHALL be unified
3. WHEN state is converted for FFI, THEN conversion SHALL happen at boundary only
4. WHEN state snapshots are created, THEN they SHALL derive from single source

### Requirement 5: Reduce Function Complexity (Rust)

**User Story:** As a developer, I want functions under 50 lines, so that code is readable and testable.

#### Acceptance Criteria

1. WHEN `process_event_traced()` is refactored, THEN it SHALL be ≤50 lines
2. WHEN the function is split, THEN each sub-function SHALL have single responsibility
3. WHEN sub-functions are created, THEN they SHALL be independently testable
4. WHEN abstraction levels are mixed, THEN they SHALL be separated into layers

### Requirement 6: Reduce File Size (Rust)

**User Story:** As a developer, I want files under 500 lines, so that code is navigable and maintainable.

#### Acceptance Criteria

1. WHEN `run.rs` is refactored, THEN it SHALL be ≤500 lines
2. WHEN `discover.rs` is refactored, THEN it SHALL be ≤500 lines
3. WHEN `runtime.rs` is refactored, THEN it SHALL be ≤500 lines
4. WHEN `exports.rs` is refactored, THEN it SHALL be ≤500 lines
5. WHEN files are split, THEN each new file SHALL have clear single purpose

### Requirement 7: Reduce Page Complexity (Flutter)

**User Story:** As a developer, I want pages focused on UI composition only, so that business logic is testable separately.

#### Acceptance Criteria

1. WHEN `editor_page.dart` is refactored, THEN business logic SHALL move to services
2. WHEN `visual_editor_page.dart` is refactored, THEN file I/O SHALL move to services
3. WHEN `console.dart` is refactored, THEN command parsing SHALL move to services
4. WHEN pages are refactored, THEN each SHALL be ≤300 lines
5. WHEN validation logic exists, THEN it SHALL be in dedicated validator classes

### Requirement 8: Eliminate Duplicate Code Patterns

**User Story:** As a developer, I want shared patterns extracted, so that fixes apply everywhere.

#### Acceptance Criteria

1. WHEN layer action handling is needed, THEN ONE function SHALL handle both cases
2. WHEN error handling follows a pattern, THEN a helper SHALL be used
3. WHEN service initialization is needed, THEN a mixin/base class SHALL provide it
4. WHEN stream subscriptions are created, THEN a helper pattern SHALL be used
5. WHEN duplicated code is found, THEN it SHALL be extracted to shared location

### Requirement 9: Improve Dependency Injection (Rust)

**User Story:** As a developer, I want all engine dependencies injectable, so that I can test with mocks.

#### Acceptance Criteria

1. WHEN AdvancedEngine is created, THEN key state tracker SHALL be injectable
2. WHEN AdvancedEngine is created, THEN modifier state SHALL be injectable
3. WHEN AdvancedEngine is created, THEN layer stack SHALL be injectable
4. WHEN testing engine, THEN mock implementations SHALL be substitutable
5. WHEN traits exist, THEN they SHALL follow interface segregation principle

---

## Non-Functional Requirements

### Test Coverage
- After refactoring, test coverage SHALL be ≥80%
- All new extracted functions SHALL have unit tests
- Parallel test execution SHALL work without failures

### Performance
- Refactoring SHALL NOT introduce performance regression
- Benchmark results SHALL remain within 5% of baseline

### Compatibility
- All existing tests SHALL continue to pass
- All CLI commands SHALL work identically
- All UI features SHALL work identically
