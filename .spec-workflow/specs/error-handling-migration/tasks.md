# Tasks Document - Error Handling Migration

- [x] 1. Add error handling dependencies to workspace
  - Files: Cargo.toml (workspace)
  - Add `anyhow = "1.0"` to workspace.dependencies
  - Add `thiserror = "2.0"` to workspace.dependencies
  - Purpose: Integrate industry-standard error handling libraries
  - _Leverage: Existing workspace.dependencies section in Cargo.toml_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Build Engineer with expertise in workspace dependency management | Task: Add anyhow and thiserror dependencies to workspace Cargo.toml following requirements 1.1-1.4, leveraging existing dependency patterns. anyhow for application code (keyrx_daemon), thiserror for library code (keyrx_core, keyrx_compiler) | Restrictions: Do not break existing builds, maintain version compatibility, ensure no circular dependencies | Success: cargo build succeeds, dependencies resolve correctly, all crates compile with new dependencies available. After implementation, edit tasks.md to mark this task as complete [x], then use log-implementation tool to record implementation details with artifacts (include dependency versions and locations)_

- [x] 2. Define CoreError types in keyrx_core
  - Files: keyrx_core/src/error.rs (create new), keyrx_core/src/lib.rs (modify)
  - Create error.rs with CoreError enum using thiserror
  - Define error variants: InvalidState, Validation, Config
  - Add CoreResult<T> type alias
  - Export error types from lib.rs
  - Purpose: Establish base error types for core library
  - _Leverage: thiserror derive macro, existing keyrx_core module structure_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Library Developer with expertise in error type design and thiserror | Task: Create CoreError enum in keyrx_core/src/error.rs with InvalidState, Validation, and Config variants following requirements 2.1-2.5. Use thiserror derive macros for automatic trait implementations. Define CoreResult<T> type alias. | Restrictions: Must not break existing keyrx_core API, maintain no_std compatibility where possible, keep error types lightweight | Success: CoreError compiles, implements Error trait, all variants have clear error messages, exported from lib.rs. Edit tasks.md [-] → [x], log implementation with artifacts (error enum definition, exported types)_

- [x] 3. Define DaemonError hierarchy
  - Files: keyrx_daemon/src/error.rs (modify existing)
  - Extend existing DaemonError enum with error variants
  - Add ConfigError, PlatformError, WebError, CliError variants with #[from] conversions
  - Define DaemonResult<T> type alias
  - Purpose: Create comprehensive error hierarchy for daemon
  - _Leverage: Existing keyrx_daemon/src/error.rs structure, CoreError from keyrx_core_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior Rust Developer with expertise in error hierarchies and type design | Task: Extend DaemonError in keyrx_daemon/src/error.rs with comprehensive error variants (ConfigError, PlatformError, WebError, CliError) following requirements 2.1-2.5. Use #[from] attribute for automatic conversions. Integrate with CoreError from keyrx_core. | Restrictions: Maintain existing error variants for backward compatibility, ensure error chain is preserved, avoid circular dependencies | Success: DaemonError compiles with all variants, automatic From conversions work, error messages are descriptive. Edit tasks.md [-] → [x], log with artifacts (error enum, From implementations)_

- [x] 4. Create ConfigError types
  - Files: keyrx_daemon/src/config/error.rs (create new), keyrx_daemon/src/config/mod.rs (modify)
  - Define ConfigError enum with variants: FileNotFound, ParseError, InvalidProfile, CompilationFailed
  - Add rich context fields (path, name, reason) to each variant
  - Export from config module
  - Purpose: Provide detailed configuration error types
  - _Leverage: thiserror, serde_json::Error, std::path::PathBuf_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Developer with expertise in configuration management and error handling | Task: Create ConfigError enum in keyrx_daemon/src/config/error.rs with FileNotFound, ParseError, InvalidProfile, CompilationFailed variants following requirements 4.1-4.5. Include rich context fields (paths, names, reasons) in each variant. | Restrictions: Do not expose sensitive configuration data in errors, sanitize file paths, maintain structured error information | Success: ConfigError compiles, error messages include actionable context, exported from config module. Edit tasks.md [-] → [x], log with artifacts (error types, field definitions)_

- [x] 5. Create PlatformError types
  - Files: keyrx_daemon/src/platform/error.rs (create new), keyrx_daemon/src/platform/mod.rs (modify)
  - Define PlatformError enum with variants: DeviceAccess, InjectionFailed, Unsupported, InitializationFailed
  - Include device names, reasons, and suggestions in error messages
  - Export from platform module
  - Purpose: Handle platform-specific operation errors
  - _Leverage: thiserror, existing platform module structure_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Programmer with expertise in platform abstraction and error handling | Task: Create PlatformError enum in keyrx_daemon/src/platform/error.rs with DeviceAccess, InjectionFailed, Unsupported, InitializationFailed variants following requirements 5.1-5.5. Include device names, reasons, and user-friendly suggestions. | Restrictions: Error messages must work on both Linux and Windows, avoid platform-specific assumptions in error types, keep error types platform-agnostic | Success: PlatformError compiles, errors include helpful suggestions, works on all platforms. Edit tasks.md [-] → [x], log with artifacts (error enum, suggestion text)_

- [-] 6. Create CLI error formatting utilities
  - Files: keyrx_daemon/src/cli/error.rs (create new), keyrx_daemon/src/cli/mod.rs (modify)
  - Implement format_cli_error(error: &DaemonError, json: bool) -> String
  - Add format_json_error() for structured JSON output
  - Add format_human_error() with colors and suggestions
  - Purpose: Format errors for CLI output
  - _Leverage: Existing cli/common.rs output_error function, colored crate for terminal colors_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI Developer with expertise in user experience and error formatting | Task: Create CLI error formatting utilities in keyrx_daemon/src/cli/error.rs with format_cli_error(), format_json_error(), and format_human_error() functions following requirements 3.1-3.5. Integrate with existing output_error() from cli/common.rs. Use colored crate for terminal output. | Restrictions: Maintain existing CLI output format compatibility, ensure JSON output is machine-parseable, keep human output concise and actionable | Success: Error formatting functions work correctly, JSON output is valid, human output includes colors and suggestions. Edit tasks.md [-] → [x], log with artifacts (formatting functions, integration points)_

- [ ] 7. Implement IntoResponse for web errors
  - Files: keyrx_daemon/src/web/error.rs (create new), keyrx_daemon/src/web/mod.rs (modify)
  - Implement IntoResponse trait for DaemonError
  - Map error types to appropriate HTTP status codes
  - Return JSON error responses
  - Purpose: Convert errors to HTTP responses for web API
  - _Leverage: axum::response::IntoResponse, serde_json for JSON responses_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Web API Developer with expertise in REST error handling and axum framework | Task: Implement IntoResponse trait for DaemonError in keyrx_daemon/src/web/error.rs, mapping error types to HTTP status codes and JSON responses following requirements 5.1-5.5. ConfigError → 400 Bad Request, Platform/IO → 500 Internal Server Error. | Restrictions: Do not expose internal error details to clients, use appropriate HTTP status codes, ensure JSON error format is consistent | Success: IntoResponse implementation compiles, correct status codes returned, error messages are client-safe. Edit tasks.md [-] → [x], log with artifacts (trait implementation, status code mappings)_

- [ ] 8. Migrate CLI handlers to Result types
  - Files: keyrx_daemon/src/cli/config.rs, keyrx_daemon/src/cli/profiles.rs, keyrx_daemon/src/cli/devices.rs, keyrx_daemon/src/cli/simulate.rs
  - Change handler function signatures from () to Result<(), DaemonError>
  - Replace unwrap() calls with ? operator
  - Add .context() for error context
  - Purpose: Eliminate unwraps in CLI module (40-50 unwraps)
  - _Leverage: anyhow::Context trait, existing CLI structure_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI Developer with expertise in error propagation and anyhow | Task: Migrate CLI handler functions in config.rs, profiles.rs, devices.rs, simulate.rs to return Result<(), DaemonError> following requirements 3.1-3.5. Replace all unwrap() with ? operator. Add .context() for operation details. Target: eliminate 40-50 unwraps. | Restrictions: Maintain existing CLI command behavior, ensure error messages are user-friendly, do not change command-line interface | Success: All CLI handlers return Result, unwraps eliminated, error messages include helpful context. Edit tasks.md [-] → [x], log with artifacts (function signatures changed, unwraps eliminated count, context added)_

- [ ] 9. Migrate configuration loading functions
  - Files: keyrx_daemon/src/config/profile_manager.rs, keyrx_daemon/src/config/layout_manager.rs, keyrx_daemon/src/config_loader.rs
  - Change function signatures to return Result<T, ConfigError>
  - Replace unwrap() with ? and add context
  - Add validation error aggregation (collect all errors, not just first)
  - Purpose: Improve config error handling (30-40 unwraps)
  - _Leverage: ConfigError types from task 4, anyhow::Context_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Configuration Management Specialist with expertise in validation and error handling | Task: Migrate configuration loading functions in profile_manager.rs, layout_manager.rs, config_loader.rs to return Result<T, ConfigError> following requirements 4.1-4.5. Replace unwrap() with ? and add context. Implement validation error aggregation to show all errors at once. Target: eliminate 30-40 unwraps. | Restrictions: Do not break existing configuration file format, maintain backward compatibility, ensure error messages show line numbers for parse errors | Success: Config functions return Result, unwraps eliminated, validation shows all errors not just first. Edit tasks.md [-] → [x], log with artifacts (function signatures, validation aggregation logic, unwraps eliminated)_

- [ ] 10. Migrate platform layer error handling
  - Files: keyrx_daemon/src/platform/linux/mod.rs, keyrx_daemon/src/platform/linux/input_capture.rs, keyrx_daemon/src/platform/linux/output_injection.rs, keyrx_daemon/src/platform/mod.rs
  - Update Platform trait methods to return Result<T, PlatformError>
  - Migrate Linux implementation to use PlatformError
  - Add error recovery for non-critical operations
  - Purpose: Prevent daemon crashes on platform errors (40-50 unwraps)
  - _Leverage: PlatformError from task 5, existing Platform trait_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Programmer with expertise in Linux input/output APIs and error recovery | Task: Migrate Platform trait and Linux implementation to return Result<T, PlatformError> following requirements 5.1-5.5. Update trait methods and impl blocks. Add error recovery for non-critical failures. Ensure daemon event loop doesn't panic. Target: eliminate 40-50 unwraps. | Restrictions: Maintain platform abstraction, do not break trait interface for Windows implementation, ensure errors are logged but don't crash daemon | Success: Platform trait uses Result types, Linux implementation uses PlatformError, event loop handles errors gracefully. Edit tasks.md [-] → [x], log with artifacts (trait changes, impl changes, error recovery logic, unwraps eliminated)_

- [ ] 11. Migrate web handlers to Result types
  - Files: keyrx_daemon/src/web/api/profiles.rs, keyrx_daemon/src/web/api/devices.rs, keyrx_daemon/src/web/api/config.rs, keyrx_daemon/src/web/api/metrics.rs
  - Change handler signatures to return Result<Json<T>, DaemonError>
  - Replace unwrap() with ? operator
  - Errors automatically converted to HTTP responses via IntoResponse
  - Purpose: Return proper HTTP error responses (30-40 unwraps)
  - _Leverage: IntoResponse implementation from task 7, axum handler patterns_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Web API Developer with expertise in axum and REST error handling | Task: Migrate web API handlers in profiles.rs, devices.rs, config.rs, metrics.rs to return Result<Json<T>, DaemonError> following requirements 5.1-5.5. Replace unwrap() with ?. Leverage IntoResponse implementation from task 7 for automatic error conversion to HTTP responses. Target: eliminate 30-40 unwraps. | Restrictions: Maintain existing API response formats for success cases, ensure proper HTTP status codes, do not expose internal errors to clients | Success: All handlers return Result, unwraps eliminated, HTTP error responses are correct. Edit tasks.md [-] → [x], log with artifacts (handler signatures, HTTP status codes, unwraps eliminated)_

- [ ] 12. Add error handling to WebSocket implementation
  - Files: keyrx_daemon/src/web/ws.rs, keyrx_daemon/src/web/ws_rpc.rs
  - Add error handling for WebSocket message parsing
  - Log errors instead of panicking in WebSocket event loop
  - Return error messages to clients for invalid requests
  - Purpose: Prevent WebSocket crashes (20-30 unwraps)
  - _Leverage: DaemonError, structured logging_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Real-time Systems Developer with expertise in WebSocket protocols and error handling | Task: Add error handling to WebSocket implementation in ws.rs and ws_rpc.rs following requirements 5.1-5.5. Replace unwrap() in message parsing with proper error handling. Log errors instead of panicking. Send error responses to clients for invalid requests. Target: eliminate 20-30 unwraps. | Restrictions: Do not disconnect clients on recoverable errors, ensure WebSocket protocol compliance, maintain real-time performance | Success: WebSocket implementation handles errors gracefully, errors logged with context, clients receive error messages. Edit tasks.md [-] → [x], log with artifacts (error handling logic, logging calls, unwraps eliminated)_

- [ ] 13. Update clippy lints configuration
  - Files: Cargo.toml (workspace section)
  - Change unwrap_used from "warn" to "deny" in workspace.lints.clippy
  - Change expect_used from "warn" to "deny"
  - Purpose: Enforce no new unwraps via compiler
  - _Leverage: Existing clippy lints added in Option B_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Build Engineer with expertise in clippy lint configuration | Task: Upgrade clippy lints in Cargo.toml from "warn" to "deny" for unwrap_used and expect_used following requirements 6.1-6.5. This enforces no new unwraps can be added. Run cargo clippy to verify no violations. | Restrictions: Ensure all existing unwraps are either eliminated or have #[allow] attributes with SAFETY comments, do not break CI build | Success: Clippy lints set to deny, cargo clippy passes with no unwrap violations. Edit tasks.md [-] → [x], log with artifacts (lint configuration changes)_

- [ ] 14. Add SAFETY comments for legitimate unwraps
  - Files: Audit all remaining unwraps in codebase via `cargo clippy --workspace`
  - Add #[allow(clippy::unwrap_used)] with SAFETY comment explaining why unwrap is safe
  - Document legitimate cases: mutex poisoning, static regexes, known-valid array indices
  - Purpose: Document intentional unwraps
  - _Leverage: Clippy output to find remaining unwraps_
  - _Requirements: 6.4_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Code Quality Engineer with expertise in Rust safety and documentation | Task: Audit remaining unwraps using cargo clippy, add #[allow(clippy::unwrap_used)] with SAFETY comments explaining why each unwrap is safe following requirement 6.4. Document cases: mutex poisoning is unrecoverable, static regexes known valid, array indices within bounds by construction. | Restrictions: Only add #[allow] for truly safe unwraps, provide clear justification in SAFETY comment, prefer Result propagation when possible | Success: All remaining unwraps have SAFETY comments, clippy passes with allow attributes, documentation explains each case. Edit tasks.md [-] → [x], log with artifacts (SAFETY comments added, justifications)_

- [ ] 15. Deprecate manual unwrap counting script
  - Files: scripts/check_unwraps.sh, scripts/verify.sh, .git/hooks/pre-commit
  - Remove check_unwraps.sh from verify.sh checks
  - Update pre-commit hook to rely on clippy lints instead
  - Add deprecation notice to check_unwraps.sh script
  - Purpose: Remove redundant manual unwrap counting
  - _Leverage: Clippy lints from task 13 replace manual counting_
  - _Requirements: 6.5_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in build automation and quality gates | Task: Deprecate manual unwrap counting script check_unwraps.sh following requirement 6.5. Remove from verify.sh and pre-commit hook. Clippy lints now enforce unwrap policy automatically. Add deprecation notice to script explaining migration to clippy. | Restrictions: Ensure clippy enforcement is active before removing manual check, update documentation to reflect new approach | Success: check_unwraps.sh removed from verification pipeline, clippy enforces unwrap policy, documentation updated. Edit tasks.md [-] → [x], log with artifacts (scripts modified, deprecation notice)_

- [ ] 16. Write error handling documentation
  - Files: docs/ERROR_HANDLING_GUIDE.md (create new), docs/UNWRAP_HANDLING_STRATEGY.md (update)
  - Document error type hierarchy and usage patterns
  - Provide before/after migration examples
  - Add troubleshooting guide for common error scenarios
  - Document SAFETY comment requirements
  - Purpose: Guide developers on error handling best practices
  - _Leverage: Existing docs/UNWRAP_HANDLING_STRATEGY.md as foundation_
  - _Requirements: All requirements (comprehensive documentation)_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with expertise in Rust and developer documentation | Task: Create comprehensive error handling guide in docs/ERROR_HANDLING_GUIDE.md covering all requirements. Document error type hierarchy (CoreError, DaemonError, ConfigError, PlatformError). Provide before/after examples for common patterns. Include troubleshooting guide. Update existing UNWRAP_HANDLING_STRATEGY.md with migration completion notes. | Restrictions: Keep documentation concise and practical, include code examples, ensure examples compile | Success: ERROR_HANDLING_GUIDE.md created with comprehensive coverage, examples are correct, UNWRAP_HANDLING_STRATEGY.md updated. Edit tasks.md [-] → [x], log with artifacts (documentation files created/updated, example count)_

- [ ] 17. Update unit tests for Result types
  - Files: All test files affected by function signature changes
  - Update test assertions to handle Result types
  - Add tests for error scenarios and error messages
  - Ensure test coverage remains ≥80%
  - Purpose: Maintain test quality during migration
  - _Leverage: Existing test infrastructure, error types from previous tasks_
  - _Requirements: 7.1, 7.2, 7.3_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Rust testing and error handling verification | Task: Update all affected unit tests to handle Result return types following requirements 7.1-7.3. Change assertions from direct values to unwrapped Results. Add new tests for error scenarios. Verify error messages are correct. Ensure coverage stays ≥80%. | Restrictions: Do not reduce test coverage, test both success and error paths, maintain test independence | Success: All tests pass with updated signatures, error scenarios tested, coverage ≥80%. Edit tasks.md [-] → [x], log with artifacts (test files updated, new error tests added, coverage metrics)_

- [ ] 18. Integration testing for error propagation
  - Files: keyrx_daemon/tests/ (add new integration tests)
  - Write integration tests that verify errors propagate correctly through full stack
  - Test CLI → Service → Platform error flow
  - Test Web → Handler → Service error flow
  - Verify error messages include proper context at each layer
  - Purpose: Ensure error handling works end-to-end
  - _Leverage: Existing integration test infrastructure_
  - _Requirements: 7.1, 7.2, 7.3_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer with expertise in error flow testing | Task: Create integration tests in keyrx_daemon/tests/ verifying error propagation through full call stack following requirements 7.1-7.3. Test CLI → Service → Platform flow and Web → Handler → Service flow. Verify errors include context at each layer. Test error recovery scenarios. | Restrictions: Use real daemon components not mocks, ensure tests are reliable, avoid flaky tests | Success: Integration tests pass, error propagation verified, context preserved through layers. Edit tasks.md [-] → [x], log with artifacts (integration test files, error flows tested)_

- [ ] 19. Verify production error behavior
  - Files: Run daemon with intentional errors in production-like environment
  - Test missing config file scenario
  - Test invalid profile syntax scenario
  - Test device permission denied scenario
  - Verify error messages are helpful and actionable
  - Purpose: Validate end-to-end error handling experience
  - _Leverage: Full daemon with all error handling integrated_
  - _Requirements: All requirements (end-to-end validation)_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Validation Engineer with expertise in production testing and user experience | Task: Perform end-to-end validation of error handling following all requirements. Run daemon with intentional errors: missing config file, invalid profile syntax, device permission denied. Verify error messages are clear, actionable, and guide users to resolution. Document results. | Restrictions: Test in production-like environment, verify both CLI and Web error paths, ensure errors don't crash daemon | Success: All error scenarios tested, messages are helpful, daemon handles errors gracefully. Edit tasks.md [-] → [x], log with artifacts (test scenarios, error message examples, validation results)_

- [ ] 20. Final cleanup and verification
  - Files: All modified files
  - Run cargo clippy --workspace to verify no unwrap violations
  - Run cargo test --workspace to verify all tests pass
  - Run cargo doc to verify documentation builds
  - Update UNWRAP_HANDLING_STRATEGY.md with migration completion status
  - Purpose: Ensure migration is complete and quality gates pass
  - _Leverage: Full error handling implementation from all previous tasks_
  - _Requirements: All requirements (final verification)_
  - _Prompt: Implement the task for spec error-handling-migration, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Release Engineer with expertise in quality verification and final validation | Task: Perform final verification of complete error handling migration covering all requirements. Run cargo clippy (no violations), cargo test (all pass), cargo doc (builds cleanly). Verify unwrap count reduced from 405 to ≤50. Update UNWRAP_HANDLING_STRATEGY.md with completion status. Generate migration report. | Restrictions: Do not proceed if any quality gate fails, ensure documentation is updated, verify no regressions | Success: Clippy passes with deny level, all tests pass, unwraps ≤50, documentation updated, migration report generated. Edit tasks.md [-] → [x], log with artifacts (verification results, unwrap count reduction, migration report)_
