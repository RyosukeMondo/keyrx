# Tasks Document

## Phase 1: Foundation Types

- [x] 1. Create ExitCode enum
  - File: `core/src/cli/exit_codes.rs`
  - Define enum with all exit code variants (0-7, reserved 100+)
  - Implement `as_u8()`, `as_process_code()`, `description()`
  - Add Display impl for human-readable output
  - Purpose: Type-safe exit code representation
  - _Leverage: Existing constants in `config/exit_codes.rs`_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in type design | Task: Create ExitCode enum in core/src/cli/exit_codes.rs with all variants and conversions | Restrictions: Use repr(u8) for stable ABI, implement Into<std::process::ExitCode>, follow existing config pattern | _Leverage: config/exit_codes.rs constants | Success: Enum compiles, all variants have correct values, conversions work | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create CommandResult<T> type
  - File: `core/src/cli/result.rs`
  - Define struct with value, exit_code, messages fields
  - Implement success(), failure(), from_result() constructors
  - Add is_success(), exit_code(), value() accessors
  - Purpose: Carry exit codes through command execution
  - _Leverage: Rust Result pattern_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with type system expertise | Task: Create CommandResult<T> in core/src/cli/result.rs following requirements 2.1-2.3 | Restrictions: Must be ergonomic to use, support message chaining, preserve exit codes | _Leverage: Rust Result/Option patterns | Success: Type compiles, constructors work, exit codes preserved through operations | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create HasExitCode trait
  - File: `core/src/cli/traits.rs`
  - Define trait with exit_code() method
  - Implement for anyhow::Error (downcast to CommandError)
  - Implement for std::io::Error (map to appropriate codes)
  - Purpose: Allow any error to specify exit code
  - _Leverage: Rust trait patterns_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with trait design expertise | Task: Create HasExitCode trait in core/src/cli/traits.rs with implementations for common error types | Restrictions: Must work with anyhow::Error via downcast, provide sensible defaults | _Leverage: anyhow downcast pattern | Success: Trait compiles, implementations work, errors carry exit codes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 4. Create CommandError enum
  - File: `core/src/cli/error.rs`
  - Define variants: Validation, TestFailure, DeviceNotFound, PermissionDenied, Timeout, Other
  - Implement HasExitCode for each variant
  - Use thiserror for Display/Error derives
  - Purpose: Structured command errors with context
  - _Leverage: thiserror crate, existing error patterns_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in error handling | Task: Create CommandError enum in core/src/cli/error.rs with context fields and HasExitCode impl | Restrictions: Use thiserror, include location info where relevant, implement HasExitCode | _Leverage: thiserror patterns, existing error types | Success: Error variants compile, HasExitCode returns correct codes, Display shows context | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Command Infrastructure

- [x] 5. Create Command trait
  - File: `core/src/cli/command.rs`
  - Define trait with execute(), name() methods
  - Create CommandContext struct for shared state
  - Add OutputFormat and Verbosity enums
  - Purpose: Standardize command interface
  - _Leverage: clap command patterns_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with CLI expertise | Task: Create Command trait and CommandContext in core/src/cli/command.rs | Restrictions: Must work with clap, support output formats, be async-compatible if needed | _Leverage: clap derive patterns | Success: Trait compiles, CommandContext has all needed fields, integrates with clap | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Refactor entry point dispatch
  - File: `core/src/bin/keyrx.rs`
  - Remove string-based exit code extraction
  - Use CommandResult for all command returns
  - Reduce file to ~150 LOC (dispatch only)
  - Purpose: Clean entry point
  - _Leverage: New Command trait, CommandResult_
  - _Requirements: 3.2, 2.1_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring CLI | Task: Refactor keyrx.rs to use Command trait and CommandResult, remove string parsing | Restrictions: Must maintain all existing functionality, reduce to ~150 LOC, use type-safe dispatch | _Leverage: Command trait, CommandResult from previous tasks | Success: Entry point compiles, all commands work, no string-based exit code logic | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Add panic handler for exit code 101
  - File: `core/src/bin/keyrx.rs`
  - Set panic hook to catch panics
  - Log panic info at error level
  - Return exit code 101 (Rust convention)
  - Purpose: Graceful panic handling
  - _Leverage: std::panic::set_hook_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with panic handling expertise | Task: Add panic hook in keyrx.rs that logs and returns exit code 101 | Restrictions: Must not lose panic info, log appropriately, use std::panic | _Leverage: std::panic::set_hook | Success: Panics caught, logged, exit code 101 returned | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Command Migration

- [ ] 8. Migrate RunCommand
  - File: `core/src/cli/commands/run.rs`
  - Update to return CommandResult<()>
  - Extract command struct if not already separate
  - Implement Command trait
  - Purpose: First command migration
  - _Leverage: Existing run.rs, Command trait_
  - _Requirements: 3.1, 2.1_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating commands | Task: Migrate RunCommand to use CommandResult and Command trait | Restrictions: Maintain all existing functionality, return appropriate exit codes | _Leverage: Existing run.rs, Command trait | Success: RunCommand uses new types, exit codes correct, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Migrate CheckCommand
  - File: `core/src/cli/commands/check.rs`
  - Return CommandResult with ValidationFailed on error
  - Include validation errors with location info
  - Purpose: Validation command migration
  - _Leverage: Existing check.rs, CommandError::Validation_
  - _Requirements: 3.1, 4.3_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating commands | Task: Migrate CheckCommand to use CommandResult with proper validation errors | Restrictions: Include file:line:col in errors, return ValidationFailed exit code | _Leverage: Existing check.rs, CommandError::Validation | Success: CheckCommand uses new types, validation errors include location | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Migrate TestCommand and UatCommand
  - Files: `core/src/cli/commands/test.rs`, `core/src/cli/commands/uat.rs`
  - Return CommandResult with AssertionFailed on test failure
  - Include pass/fail counts in error
  - Purpose: Test commands migration
  - _Leverage: Existing test.rs/uat.rs, CommandError::TestFailure_
  - _Requirements: 3.1, 1.3_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating commands | Task: Migrate TestCommand and UatCommand to use CommandResult with test failure info | Restrictions: Return AssertionFailed (exit 2) on failures, include pass/fail counts | _Leverage: Existing test/uat commands, CommandError::TestFailure | Success: Test commands use new types, exit code 2 on failures | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Migrate SimulateCommand
  - File: `core/src/cli/commands/simulate.rs`
  - Return CommandResult with appropriate errors
  - Handle timeout with CommandError::Timeout
  - Purpose: Simulate command migration
  - _Leverage: Existing simulate.rs_
  - _Requirements: 3.1, 1.4_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating commands | Task: Migrate SimulateCommand to use CommandResult with timeout handling | Restrictions: Return Timeout exit code on timeout, proper error context | _Leverage: Existing simulate.rs, CommandError::Timeout | Success: SimulateCommand uses new types, timeout handled correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Migrate remaining commands
  - Files: All remaining `core/src/cli/commands/*.rs`
  - Apply same pattern to: doctor, bench, state, discover, etc.
  - Ensure consistent Command trait implementation
  - Purpose: Complete command migration
  - _Leverage: Pattern from previous migrations_
  - _Requirements: 3.1, 3.3_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing migration | Task: Migrate all remaining commands to use CommandResult and Command trait | Restrictions: Apply consistent pattern, maintain functionality, proper exit codes | _Leverage: Pattern from previous command migrations | Success: All commands migrated, consistent interface, all tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Documentation & Testing

- [ ] 13. Add exit-codes subcommand
  - File: `core/src/cli/commands/exit_codes.rs`
  - List all exit codes with descriptions
  - Support --json output format
  - Purpose: Exit code documentation
  - _Leverage: ExitCode enum_
  - _Requirements: 5.2_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating CLI command | Task: Create exit-codes subcommand showing all exit codes with descriptions | Restrictions: Support human and JSON output, list all codes with meanings | _Leverage: ExitCode enum, OutputFormat | Success: Command shows all exit codes, supports --json, helpful descriptions | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Update --help with exit code info
  - File: `core/src/bin/keyrx.rs`
  - Add exit code section to main help
  - Reference exit-codes subcommand
  - Purpose: Discoverability
  - _Leverage: clap after_help_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer improving CLI help | Task: Add exit code documentation to --help output | Restrictions: Use clap after_help, reference exit-codes subcommand | _Leverage: clap documentation features | Success: --help shows exit code summary, points to detailed command | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Add exit code integration tests
  - File: `core/tests/cli_exit_codes_test.rs`
  - Test each exit code scenario
  - Verify codes match documentation
  - Purpose: Exit code verification
  - _Leverage: assert_cmd crate_
  - _Requirements: All_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create integration tests verifying exit codes for all command scenarios | Restrictions: Test actual CLI invocation, verify all documented exit codes | _Leverage: assert_cmd crate | Success: Tests cover all exit code scenarios, run in CI | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Update CHANGELOG with exit code changes
  - File: `CHANGELOG.md`
  - Document new exit code system
  - List all exit codes and meanings
  - Note any breaking changes
  - Purpose: Release documentation
  - _Leverage: Keep a Changelog format_
  - _Requirements: 5.4_
  - _Prompt: Implement the task for spec cli-exit-code-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Update CHANGELOG with exit code system documentation | Restrictions: Follow Keep a Changelog format, document all codes, note breaking changes | _Leverage: Existing CHANGELOG format | Success: CHANGELOG updated, exit codes documented, migration notes if needed | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
