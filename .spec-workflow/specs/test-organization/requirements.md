# Requirements Document

## Introduction

The test codebase has organizational issues: `validation_integration.rs` is 27K LOC, `phase_1_3_integration_test.rs` is 21K LOC, and 118 `#[cfg(test)]` modules are scattered throughout source files. This spec establishes a consistent test organization strategy with manageable file sizes and clear categorization.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Testable Configs**: Tests must be maintainable to remain valuable
- **CLI First, GUI Later**: Test infrastructure supports both CLI and GUI testing
- **Safety First**: Comprehensive tests catch regressions

Per CLAUDE.md: "80% test coverage minimum (90% for critical paths)" and "Max 500 lines/file"

## Requirements

### Requirement 1: Test File Size Limits

**User Story:** As a developer, I want test files under 500 LOC, so that I can navigate and understand test suites.

#### Acceptance Criteria

1. WHEN a test file exceeds 500 LOC THEN it SHALL be split into focused sub-files
2. IF tests share setup THEN shared fixtures SHALL be extracted to helper modules
3. WHEN splitting THEN test names SHALL remain stable for CI reporting
4. IF a category has many tests THEN subdirectories SHALL be used

### Requirement 2: Test Categorization

**User Story:** As a developer, I want tests organized by type (unit, integration, e2e), so that I can run the appropriate level of testing.

#### Acceptance Criteria

1. WHEN running `cargo test` THEN unit tests SHALL run by default
2. IF integration tests are needed THEN `cargo test --test integration` SHALL run them
3. WHEN tests require real devices THEN they SHALL be marked `#[ignore]` with reason
4. IF e2e tests exist THEN they SHALL be in `tests/e2e/` directory

### Requirement 3: Inline Test Extraction

**User Story:** As a developer, I want inline tests moved to test files, so that source files focus on implementation.

#### Acceptance Criteria

1. WHEN source has > 100 LOC of inline tests THEN they SHALL be moved to test files
2. IF tests need private access THEN `#[cfg(test)]` helpers SHALL be exposed
3. WHEN moving tests THEN coverage SHALL remain the same or improve
4. IF inline tests remain THEN they SHALL be for truly internal behavior only

### Requirement 4: Shared Test Fixtures

**User Story:** As a test developer, I want shared fixtures and builders, so that I don't duplicate test setup across files.

#### Acceptance Criteria

1. WHEN multiple tests need similar setup THEN a shared fixture SHALL be available
2. IF fixtures need cleanup THEN they SHALL implement Drop
3. WHEN builders are used THEN they SHALL follow builder pattern consistently
4. IF fixtures are expensive THEN lazy initialization SHALL be used

### Requirement 5: Test Documentation

**User Story:** As a new developer, I want test organization documented, so that I know where to add new tests.

#### Acceptance Criteria

1. WHEN adding a test THEN the test README SHALL explain where it goes
2. IF a test category exists THEN its purpose SHALL be documented
3. WHEN tests fail THEN the error message SHALL indicate the test's purpose
4. IF naming conventions exist THEN they SHALL be documented

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each test file tests one module/feature
- **Modular Design**: Test utilities are reusable across test files
- **Dependency Management**: Tests don't depend on test execution order
- **Clear Interfaces**: Fixtures have clear setup/teardown

### Performance
- Unit tests SHALL complete in < 30 seconds total
- Integration tests SHALL complete in < 5 minutes
- Parallel test execution SHALL be supported

### Security
- Tests SHALL not require elevated privileges unless testing drivers
- Test fixtures SHALL not leave artifacts on disk

### Reliability
- Tests SHALL be deterministic (no flaky tests)
- Tests SHALL run in any order
- Tests SHALL clean up after themselves

### Usability
- Finding tests for a module SHALL take < 30 seconds
- Adding a new test SHALL be obvious from documentation
- Test failures SHALL clearly indicate what failed and why
