# Requirements Document

## Introduction

The CLI entry point (`core/src/bin/keyrx.rs`, 681 LOC) has critical issues with exit code handling. The current implementation uses brittle string matching to extract exit codes from error messages, and there's a compiler error where `ExitCode::from(i32)` is used but only `from(u8)` is implemented. This spec addresses the complete refactoring of CLI command dispatch and exit code propagation.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **CLI First, GUI Later**: CLI is the foundation; it must be rock-solid
- **Testable Configs**: Proper exit codes enable CI/CD integration and scripted testing
- **Performance > Features**: Clean command dispatch reduces overhead

Per tech.md: All features must be CLI-exercisable with semantic exit codes (0=success, 1=error, 2=assertion fail, 3=timeout).

## Requirements

### Requirement 1: Structured Exit Code System

**User Story:** As a CLI user, I want consistent exit codes across all commands, so that I can reliably script KeyRx operations in CI/CD pipelines.

#### Acceptance Criteria

1. WHEN any command succeeds THEN the CLI SHALL return exit code 0
2. IF a command fails with a general error THEN the CLI SHALL return exit code 1
3. WHEN a test assertion fails THEN the CLI SHALL return exit code 2
4. IF a command times out THEN the CLI SHALL return exit code 3
5. WHEN validation fails THEN the CLI SHALL return exit code 4

### Requirement 2: Type-Safe Exit Code Propagation

**User Story:** As a developer, I want exit codes propagated through the type system, so that I can't accidentally lose exit code information.

#### Acceptance Criteria

1. WHEN a command returns THEN it SHALL use `CommandResult<T>` type that carries exit code
2. IF an error occurs THEN the error type SHALL include the intended exit code
3. WHEN errors are chained THEN the original exit code SHALL be preserved
4. IF multiple errors occur THEN the most severe exit code SHALL be used

### Requirement 3: Command Struct Extraction

**User Story:** As a maintainer, I want command definitions separate from the entry point, so that adding new commands doesn't bloat keyrx.rs.

#### Acceptance Criteria

1. WHEN a command is defined THEN its struct SHALL live in `cli/commands/{name}.rs`
2. IF the entry point file exceeds 200 LOC THEN it SHALL only contain dispatch logic
3. WHEN adding a new command THEN no changes to keyrx.rs SHALL be required
4. IF a command has subcommands THEN they SHALL be organized in a subdirectory

### Requirement 4: Error Context Preservation

**User Story:** As a user, I want error messages to include context, so that I can understand what failed and why.

#### Acceptance Criteria

1. WHEN an error occurs THEN the message SHALL include the command that failed
2. IF a file operation fails THEN the path SHALL be included in the error
3. WHEN a validation error occurs THEN line/column information SHALL be included
4. IF the error is recoverable THEN a suggested action SHALL be provided

### Requirement 5: Exit Code Documentation

**User Story:** As a script author, I want documented exit codes, so that I can handle all possible outcomes.

#### Acceptance Criteria

1. WHEN `keyrx --help` is run THEN exit codes SHALL be listed
2. IF `keyrx exit-codes` is run THEN a detailed exit code table SHALL be shown
3. WHEN a command fails THEN the exit code meaning SHALL be logged at debug level
4. IF exit codes change THEN the CHANGELOG SHALL document the change

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Entry point only dispatches; commands implement logic
- **Modular Design**: Each command is a self-contained module
- **Dependency Management**: Commands don't import each other
- **Clear Interfaces**: `CommandResult<T>` is the universal return type

### Performance
- Command dispatch overhead SHALL be < 1ms
- Exit code determination SHALL not require string parsing

### Security
- Exit codes SHALL not leak sensitive information
- Error messages SHALL sanitize file paths if needed

### Reliability
- All exit paths SHALL have defined exit codes
- Panic handler SHALL return exit code 101 (Rust convention)

### Usability
- Adding a new command SHALL require < 50 LOC boilerplate
- Exit codes SHALL follow Unix conventions where applicable
