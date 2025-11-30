# Requirements Document: dev-tooling

## Introduction

This specification establishes development tooling infrastructure for KeyRx to ensure code quality, enable rapid iteration, and support autonomous AI-driven development. It addresses critical gaps in CI/CD, error handling patterns, documentation, and testing infrastructure identified during MVP review.

## Alignment with Product Vision

From product.md principles:
- **CLI First, GUI Later**: Robust CLI tooling enables AI agent autonomy
- **Performance > Features**: CI gates prevent performance regressions
- **Safety First**: Pre-commit hooks catch issues before they reach production

From CLAUDE.md requirements:
- Pre-commit hooks mandatory (linting, formatting, tests)
- 80% test coverage minimum
- Structured logging with JSON format
- Custom exception hierarchy with error codes

## Requirements

### REQ-1: Pre-Commit Hooks

**User Story:** As a developer, I want automated code quality checks before each commit, so that bad code never enters the repository.

#### Acceptance Criteria

1. WHEN a developer runs `git commit` THEN rustfmt SHALL check formatting automatically
2. WHEN formatting check fails THEN the commit SHALL be blocked with clear error message
3. WHEN a developer runs `git commit` THEN clippy SHALL run with `-D warnings` (deny warnings)
4. WHEN clippy finds issues THEN the commit SHALL be blocked with actionable fixes
5. WHEN a developer runs `git commit` THEN `cargo test` SHALL run for affected code
6. IF any pre-commit check fails THEN exit code SHALL be non-zero
7. WHEN hooks are installed THEN `cargo fmt --check && cargo clippy && cargo test` SHALL pass

### REQ-2: CI/CD Pipeline

**User Story:** As a maintainer, I want automated builds and tests on every PR, so that broken code is caught before merge.

#### Acceptance Criteria

1. WHEN a PR is opened THEN GitHub Actions SHALL trigger build workflow
2. WHEN CI runs THEN it SHALL build for both Linux and Windows targets
3. WHEN CI runs THEN it SHALL execute all tests with `cargo test`
4. WHEN CI runs THEN it SHALL run `cargo clippy -D warnings`
5. WHEN CI runs THEN it SHALL verify formatting with `cargo fmt --check`
6. WHEN any CI check fails THEN the PR SHALL be marked as failing
7. WHEN CI completes THEN build artifacts SHALL be available for download
8. WHEN CI runs THEN it SHALL run on Rust stable and check MSRV (1.70+)

### REQ-3: Error Propagation

**User Story:** As a developer, I want all errors to propagate properly, so that I can debug issues without silent failures.

#### Acceptance Criteria

1. WHEN a Rhai function receives invalid key name THEN it SHALL return an error (not log and ignore)
2. WHEN a CLI command fails THEN it SHALL return `Result::Err` (not call `std::process::exit`)
3. WHEN `RhaiRuntime::default()` cannot initialize THEN it SHALL NOT panic
4. WHEN any function encounters an error THEN it SHALL propagate via `Result<T, E>`
5. WHEN errors are logged THEN they SHALL include context (file, line, operation)
6. IF a recoverable error occurs THEN the system SHALL continue operating
7. WHEN `path.to_str()` fails THEN clear error message SHALL be returned (not empty string)

### REQ-4: Scripting API Documentation

**User Story:** As an AI agent, I want comprehensive scripting documentation, so that I can write and validate Rhai scripts autonomously.

#### Acceptance Criteria

1. WHEN documentation is complete THEN all public Rhai functions SHALL be documented
2. WHEN reading docs THEN `remap(from, to)` behavior and error cases SHALL be clear
3. WHEN reading docs THEN `block(key)` behavior and error cases SHALL be clear
4. WHEN reading docs THEN `pass(key)` behavior and error cases SHALL be clear
5. WHEN reading docs THEN all valid key names SHALL be listed with aliases
6. WHEN reading docs THEN hook lifecycle (`on_init`, etc.) SHALL be explained
7. WHEN reading docs THEN error handling expectations SHALL be documented
8. WHEN documentation is complete THEN example scripts SHALL demonstrate all features

### REQ-5: Mock Infrastructure

**User Story:** As a developer, I want comprehensive mock implementations, so that I can test all code paths including errors.

#### Acceptance Criteria

1. WHEN using MockInput THEN I SHALL be able to configure error responses
2. WHEN using MockRuntime THEN I SHALL be able to verify which methods were called
3. WHEN using MockState THEN I SHALL be able to track mutation history
4. WHEN testing error paths THEN mocks SHALL support configurable failure modes
5. WHEN asserting behavior THEN mocks SHALL provide call history inspection
6. WHEN mocks are used THEN they SHALL implement the same trait bounds as production code

### REQ-6: Code Quality Configuration

**User Story:** As a developer, I want consistent code style enforcement, so that the codebase remains maintainable.

#### Acceptance Criteria

1. WHEN `rustfmt.toml` exists THEN it SHALL define project formatting rules
2. WHEN `.clippy.toml` exists THEN it SHALL configure lint levels
3. WHEN `Cargo.toml` is configured THEN dev profile SHALL optimize for fast iteration
4. WHEN lints are configured THEN `unsafe` code SHALL require explicit justification
5. WHEN code is formatted THEN max line width SHALL be 100 characters
6. WHEN imports are formatted THEN they SHALL follow Rust standard ordering

### REQ-7: Thread Safety Cleanup

**User Story:** As a developer, I want clear thread safety guarantees, so that async code behaves predictably.

#### Acceptance Criteria

1. WHEN `RhaiRuntime` is refactored THEN `Rc<RefCell>` SHALL be replaced with owned or `Arc<Mutex>`
2. WHEN traits require `Send` THEN implementations SHALL be thread-safe
3. WHEN documentation exists THEN thread safety guarantees SHALL be explicit
4. IF interior mutability is needed THEN `Arc<Mutex>` or `Arc<RwLock>` SHALL be used
5. WHEN borrowing patterns exist THEN they SHALL NOT be able to panic at runtime

## Non-Functional Requirements

### Code Architecture and Modularity
- All error types implement `std::error::Error`
- Error context preserved through `anyhow::Context`
- No `unwrap()` or `expect()` in production code paths

### Performance
- Pre-commit hooks complete in < 30 seconds
- CI pipeline completes in < 10 minutes
- No performance regression from error handling changes

### Reliability
- Zero panics from error handling code
- All CI checks reproducible locally
- Hooks work on Linux, macOS, and Windows

### Maintainability
- Single source of truth for lint configuration
- Documentation generated from source where possible
- Hooks installable via single command
