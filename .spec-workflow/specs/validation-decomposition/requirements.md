# Requirements Document

## Introduction

The validation module contains `conflicts.rs` at 1,321 LOC with 61 functions handling three distinct concerns: remap/block conflict detection, combo shadowing detection, and circular dependency detection. Additionally, 44% of the file is inline tests. This spec decomposes the monolithic file into focused submodules with clear separation of concerns.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Safety First**: Validation is critical for user trust; it must be maintainable
- **Testable Configs**: Modular validators enable targeted testing
- **Visual > Abstract**: Clear module boundaries make validation pipeline visible

Per tech.md: "Single Responsibility Principle: Each file should have a single, well-defined purpose"

## Requirements

### Requirement 1: Conflict Detection Module

**User Story:** As a developer, I want remap/block conflict detection in its own module, so that I can understand and modify conflict logic without wading through combo and cycle code.

#### Acceptance Criteria

1. WHEN two remaps target the same key THEN the detector SHALL report a conflict
2. IF a remap conflicts with a block THEN both operations SHALL be listed in the error
3. WHEN conflicts are detected THEN the report SHALL include source locations
4. IF no conflicts exist THEN the detector SHALL return an empty result efficiently

### Requirement 2: Combo Shadowing Module

**User Story:** As a developer, I want combo shadowing detection separate from general conflict detection, so that I can optimize shadowing algorithms independently.

#### Acceptance Criteria

1. WHEN a combo's keys are individually remapped THEN shadowing SHALL be detected
2. IF a combo shadows another combo THEN both combos SHALL be reported
3. WHEN analyzing shadowing THEN the detector SHALL consider key ordering
4. IF shadowing analysis is expensive THEN it SHALL be skippable via flag

### Requirement 3: Circular Dependency Module

**User Story:** As a developer, I want circular remap detection in its own module, so that graph algorithms are isolated and testable.

#### Acceptance Criteria

1. WHEN A remaps to B and B remaps to A THEN a cycle SHALL be detected
2. IF cycles involve more than 2 keys THEN the full cycle path SHALL be reported
3. WHEN detecting cycles THEN the algorithm SHALL use efficient graph traversal
4. IF no cycles exist THEN the check SHALL complete in O(n) time

### Requirement 4: Test Extraction

**User Story:** As a test developer, I want validation tests in dedicated test files, so that source files are focused on implementation.

#### Acceptance Criteria

1. WHEN tests exist inline THEN they SHALL be moved to `validation/tests/`
2. IF tests require internal access THEN the module SHALL expose `#[cfg(test)]` helpers
3. WHEN running tests THEN all existing tests SHALL pass without modification
4. IF new tests are added THEN they SHALL go in the appropriate test module

### Requirement 5: Shared Utilities Extraction

**User Story:** As a developer, I want shared validation utilities in a common module, so that conflict, shadowing, and cycle detectors can reuse code.

#### Acceptance Criteria

1. WHEN multiple detectors traverse operations THEN a shared `OperationVisitor` SHALL be used
2. IF error types are shared THEN they SHALL live in `validation/error.rs`
3. WHEN building reports THEN a common `ReportBuilder` SHALL be available
4. IF location tracking is needed THEN `Span` type SHALL be shared

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each file handles one detection concern
- **Modular Design**: Detectors are pluggable into validation engine
- **Dependency Management**: Detectors don't import each other directly
- **Clear Interfaces**: `Detector` trait defines common interface

### Performance
- Conflict detection SHALL complete in O(n) for n operations
- Cycle detection SHALL use DFS with O(V+E) complexity
- Memory usage SHALL not exceed O(n) for intermediate state

### Security
- Validation SHALL not execute user scripts
- Error messages SHALL not leak file system paths outside project

### Reliability
- All existing validation behavior SHALL be preserved
- No new validation false positives or negatives

### Usability
- Each detector module SHALL be < 400 LOC
- Adding a new detector SHALL require < 100 LOC boilerplate
