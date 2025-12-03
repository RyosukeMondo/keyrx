# Requirements Document: Code Quality Fixes

## Introduction

This spec addresses code quality violations identified during a comprehensive audit of the KeyRx Rust core codebase. The audit found:
- 1 failing unit test
- 12 files exceeding the 500-line limit defined in steering documents
- Several files with function counts that may benefit from decomposition

These fixes ensure compliance with the code quality standards defined in `structure.md` and `CLAUDE.md`.

## Alignment with Product Vision

Per `structure.md` Code Size Guidelines:
- **Max 500 lines/file** (excluding comments/blank lines)
- **Max 50 lines/function**
- **80% test coverage minimum** (currently at 80.96% - passing)

Per `tech.md` Key Patterns:
- **Modular Drivers**: OS adapters implement generic traits
- **No Global State**: All instances are self-contained structs

## Requirements

### REQ-1: Fix Failing Test

**User Story:** As a developer, I want all unit tests to pass, so that CI/CD pipelines succeed and code quality is maintained.

#### Acceptance Criteria

1. WHEN running `cargo test --lib` THEN the system SHALL report 0 failures
2. IF test `device_profiles_dir_prefers_xdg_config_home` runs THEN it SHALL pass with correct XDG_CONFIG_HOME handling
3. WHEN environment variables are set in tests THEN the system SHALL properly isolate test environments

### REQ-2: Refactor FFI Module (exports_session.rs)

**User Story:** As a maintainer, I want FFI exports organized by domain, so that the codebase is easier to navigate and maintain.

#### Acceptance Criteria

1. WHEN `exports_session.rs` is refactored THEN each resulting file SHALL be under 500 lines
2. IF FFI functions are split THEN the public API SHALL remain unchanged
3. WHEN modules are separated THEN they SHALL follow single-responsibility principle:
   - Recording functions in dedicated module
   - Discovery functions in dedicated module
   - Testing/simulation functions in dedicated module
   - Session analysis functions in dedicated module

### REQ-3: Refactor UAT Module Files

**User Story:** As a maintainer, I want UAT module files to comply with size limits, so that code remains readable and maintainable.

#### Acceptance Criteria

1. WHEN `uat/report.rs` (1828 lines) is refactored THEN resulting files SHALL be under 500 lines
2. WHEN `uat/perf.rs` (1430 lines) is refactored THEN resulting files SHALL be under 500 lines
3. WHEN `uat/golden.rs` (1295 lines) is refactored THEN resulting files SHALL be under 500 lines
4. WHEN `uat/runner.rs` (1079 lines) is refactored THEN resulting files SHALL be under 500 lines
5. WHEN `uat/gates.rs` (1011 lines) is refactored THEN resulting files SHALL be under 500 lines
6. WHEN `uat/fuzz.rs` (650 lines) is refactored THEN resulting files SHALL be under 500 lines
7. WHEN `uat/coverage.rs` (637 lines) is refactored THEN resulting files SHALL be under 500 lines
8. IF modules are split THEN existing public APIs SHALL remain unchanged

### REQ-4: Refactor Engine/CLI Files

**User Story:** As a maintainer, I want engine and CLI files to comply with size limits for better code organization.

#### Acceptance Criteria

1. WHEN `engine/tracing.rs` (621 lines) is refactored THEN resulting files SHALL be under 500 lines
2. WHEN `cli/commands/ci_check.rs` (789 lines) is refactored THEN resulting files SHALL be under 500 lines
3. WHEN `cli/commands/regression.rs` (543 lines) is refactored THEN resulting files SHALL be under 500 lines
4. IF modules are split THEN existing public APIs SHALL remain unchanged

### REQ-5: Maintain Test Coverage

**User Story:** As a developer, I want test coverage to remain at or above 80% after refactoring, so that code quality is maintained.

#### Acceptance Criteria

1. WHEN all refactoring is complete THEN `cargo llvm-cov` SHALL report >= 80% coverage
2. IF new modules are created THEN they SHALL have unit tests
3. WHEN tests are moved THEN they SHALL continue to pass

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each file should have a single, well-defined purpose
- **Modular Design**: Split files logically by domain/function
- **Clear Interfaces**: Maintain existing public APIs through re-exports in mod.rs

### Performance
- Refactoring SHALL NOT impact runtime performance
- Compilation time may increase slightly due to more files

### Reliability
- All existing tests SHALL pass after refactoring
- No behavioral changes to existing functionality

### Maintainability
- Each resulting file SHALL have clear purpose documented in module-level comments
- Re-exports in mod.rs SHALL maintain backward compatibility
