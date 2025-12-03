# Requirements Document: Flutter UI Fixes

## Introduction

This spec addresses code quality issues identified in the Flutter UI codebase during a comprehensive audit:
- 8 failing tests due to outdated mock configurations
- 4 files exceeding the 500-line limit
- Test infrastructure needing updates to match current ServiceRegistry API

## Alignment with Product Vision

Per `structure.md` Code Size Guidelines:
- **Max 500 lines/file** (excluding comments/blank lines)
- **All tests must pass** before deployment

Per `tech.md` Key Patterns:
- **Testability**: All services mockable via DI
- **Modular Design**: Split large files by domain

## Requirements

### REQ-1: Fix Failing Flutter Tests

**User Story:** As a developer, I want all Flutter tests to pass, so that CI/CD pipelines succeed and code quality is maintained.

#### Acceptance Criteria

1. WHEN running `flutter test` THEN the system SHALL report 0 failures
2. IF `ServiceRegistry.withOverrides()` is called THEN all required parameters SHALL be provided
3. WHEN tests use mock services THEN they SHALL match current ServiceRegistry API

### REQ-2: Refactor FFI Bridge Module

**User Story:** As a maintainer, I want the FFI bridge organized by domain, so that the codebase is easier to navigate and maintain.

#### Acceptance Criteria

1. WHEN `bridge.dart` (1841 lines) is refactored THEN each resulting file SHALL be under 500 lines
2. IF FFI methods are split THEN the public API SHALL remain unchanged
3. WHEN modules are separated THEN they SHALL follow single-responsibility:
   - Core initialization in dedicated module
   - Engine control in dedicated module
   - Audio capture in dedicated module
   - Session/recording in dedicated module
   - Discovery in dedicated module

### REQ-3: Refactor Oversized Page/Widget Files

**User Story:** As a maintainer, I want page files to comply with size limits for better code organization.

#### Acceptance Criteria

1. WHEN `run_controls_page.dart` (542 lines) is refactored THEN resulting files SHALL be under 500 lines
2. WHEN `visual_keyboard.dart` (529 lines) is refactored THEN resulting files SHALL be under 500 lines
3. WHEN `editor_page.dart` (509 lines) is refactored THEN resulting files SHALL be under 500 lines
4. IF widgets are extracted THEN existing public APIs SHALL remain unchanged

### REQ-4: Maintain Test Coverage

**User Story:** As a developer, I want test coverage maintained after refactoring.

#### Acceptance Criteria

1. WHEN all refactoring is complete THEN `flutter test` SHALL pass with 0 failures
2. IF new modules are created THEN they SHALL be testable via existing patterns
3. WHEN tests are updated THEN they SHALL continue to validate the same functionality

## Non-Functional Requirements

### Code Architecture
- **Single Responsibility**: Each file should have single purpose
- **Modular Design**: Split files logically by domain
- **Backward Compatibility**: Maintain existing public APIs through re-exports

### Reliability
- All existing tests SHALL pass after refactoring
- No behavioral changes to existing functionality
