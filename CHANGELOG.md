# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Type-Safe Exit Code System**: Introduced `ExitCode` enum for consistent, type-safe exit code handling across all CLI commands.
  - Added `ExitCode` enum in `core/src/cli/exit_codes.rs` with 9 distinct codes (0-7, 101)
  - Added `CommandResult<T>` type for propagating exit codes through command execution
  - Added `HasExitCode` trait for error types to specify their exit codes
  - Added `CommandError` enum with structured error variants
  - Added `Command` trait for standardized command interface

- **Exit Code Documentation**: New `exit-codes` subcommand displays all exit codes with descriptions.
  - Human-readable format by default
  - JSON format support via `--json` flag
  - Integrated into main `--help` output with exit code reference

- **Comprehensive Exit Code Testing**: Integration test suite (`core/tests/cli_exit_codes_test.rs`) verifying all exit code scenarios.
  - 30 tests covering all exit codes (22 active, 8 environment-specific)
  - Validates exit codes match documentation
  - Tests all command scenarios (success, errors, timeouts, etc.)

### Changed

- **CLI Exit Code Behavior**: All commands now return consistent, semantically meaningful exit codes instead of generic errors.
  - Refactored entry point (`core/src/bin/keyrx.rs`) to use type-safe dispatch (~150 LOC)
  - Migrated all commands to return `CommandResult<T>` with appropriate exit codes
  - Commands affected: `run`, `check`, `test`, `uat`, `simulate`, `doctor`, `bench`, `state`, `discover`, and others

- **Panic Handling**: Added panic hook to catch Rust panics and return exit code 101 (following Rust conventions).

- **Error Context**: Enhanced error messages with location information (file:line:col) for validation and script errors.

### Exit Code Reference

KeyRx now uses the following exit codes across all commands:

| Code | Name                | Meaning                                           |
|------|---------------------|---------------------------------------------------|
| 0    | Success             | Operation completed successfully                  |
| 1    | GeneralError        | General error (file not found, runtime error)     |
| 2    | AssertionFailed     | Test assertion or verification failure            |
| 3    | Timeout             | Operation timed out                               |
| 4    | ValidationFailed    | Configuration or script validation failed         |
| 5    | PermissionDenied    | Insufficient permissions for operation            |
| 6    | DeviceNotFound      | Required device not found                         |
| 7    | ScriptError         | Script execution error                            |
| 101  | Panic               | Unhandled panic (Rust convention)                 |

**Usage Examples:**

```bash
# View all exit codes with descriptions
keyrx exit-codes

# Get exit codes in JSON format for scripting
keyrx --json exit-codes

# Exit code 0: Success
keyrx check valid_script.rhai && echo "Validation passed"

# Exit code 2: Test failure
keyrx test tests.rhai || echo "Tests failed with exit code $?"

# Exit code 4: Validation error
keyrx check invalid.rhai  # Returns 4 on syntax errors
```

### Breaking Changes

**Note:** This is a pre-1.0 release (v0.1.0). The exit code refactor changes CLI behavior but improves consistency and scriptability.

- **Exit code changes**: Commands that previously returned exit code 1 for all errors now return more specific codes (2-7, 101) based on the error type.
- **Script integration**: If you have scripts that check for specific exit codes, update them to handle the new exit code semantics.
- **Migration guide**: Use `keyrx exit-codes` to see all current exit codes and update your automation accordingly.

### Technical Details

**Architecture:**
- Centralized exit code definitions in `core/src/cli/exit_codes.rs`
- Type-safe conversion between `ExitCode` and `std::process::ExitCode`
- Error trait system (`HasExitCode`) for automatic exit code extraction from errors
- Consistent error propagation via `CommandResult<T>` wrapper type

**Testing:**
- Integration tests ensure CLI behavior matches documentation
- Unit tests verify exit code conversions and trait implementations
- Tests cover success cases, error cases, and edge cases for all exit codes

## [0.1.0] - 2025-12-03

### Initial Release

- Initial KeyRx implementation with keyboard remapping engine
- Rhai scripting support
- Basic CLI commands (run, test, check, simulate, etc.)
- Device discovery and management
- Session recording and replay
- UAT (User Acceptance Testing) support

---

[Unreleased]: https://github.com/yourusername/keyrx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/keyrx/releases/tag/v0.1.0
