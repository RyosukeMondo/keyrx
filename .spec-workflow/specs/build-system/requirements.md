# Requirements Document

## Introduction

This spec implements the comprehensive build system, CI/CD pipelines, and release automation defined in the tech.md steering document. The goal is to achieve state-of-the-art developer productivity with one-command setup, automated quality gates, cross-platform builds, and seamless GitHub Release publishing.

## Alignment with Product Vision

From product.md:
- **CLI First, GUI Later**: Build system must support CLI-driven workflows for AI agent autonomy
- **Performance > Features**: CI must enforce <1ms latency with benchmark regression detection
- **Safety First**: Security audits must be automated in CI pipeline
- **Success Metrics**: "CI fails if any PR increases latency > 100 microseconds"

The build system directly enables these principles by automating quality enforcement and providing reproducible builds across platforms.

## Requirements

### Requirement 1: One-Command Development Setup

**User Story:** As a new contributor, I want to set up my development environment with a single command, so that I can start contributing without manual configuration steps.

#### Acceptance Criteria

1. WHEN user runs `just setup` in project root, THEN system SHALL install all Rust toolchain dependencies
2. WHEN user runs `just setup`, THEN system SHALL install all Cargo dev tools (cargo-watch, cargo-nextest, cross, cargo-llvm-cov)
3. WHEN user runs `just setup`, THEN system SHALL run `flutter pub get` for UI dependencies
4. WHEN user runs `just setup`, THEN system SHALL install pre-commit hooks via `install-hooks.sh`
5. IF any setup step fails, THEN system SHALL display clear error message with remediation steps
6. WHEN setup completes successfully, THEN user SHALL be able to run `just check` immediately

### Requirement 2: Task Runner (justfile)

**User Story:** As a developer, I want a unified task runner with discoverable commands, so that I can execute common workflows without remembering long command sequences.

#### Acceptance Criteria

1. WHEN user runs `just` without arguments, THEN system SHALL display all available recipes with descriptions
2. WHEN user runs `just dev`, THEN system SHALL start Rust watch mode with clippy and tests
3. WHEN user runs `just ui`, THEN system SHALL start Flutter with hot reload
4. WHEN user runs `just check`, THEN system SHALL run fmt, clippy, and all tests (CI-equivalent locally)
5. WHEN user runs `just build`, THEN system SHALL build release binaries for current platform
6. WHEN user runs `just build-all`, THEN system SHALL build for all supported targets (requires cross)
7. WHEN user runs `just bench`, THEN system SHALL run criterion latency benchmarks
8. WHEN user runs `just release <version>`, THEN system SHALL execute release preparation script

### Requirement 3: Pre-commit Quality Gates

**User Story:** As a developer, I want automatic quality checks before each commit, so that broken code never enters the repository.

#### Acceptance Criteria

1. WHEN developer attempts to commit, THEN pre-commit hook SHALL run `cargo fmt --check`
2. WHEN developer attempts to commit, THEN pre-commit hook SHALL run `cargo clippy -- -D warnings`
3. WHEN developer attempts to commit, THEN pre-commit hook SHALL run `cargo test --lib`
4. IF any pre-commit check fails, THEN commit SHALL be blocked with clear error message
5. WHEN all pre-commit checks pass, THEN system SHALL display "All pre-commit checks passed!" and allow commit
6. WHEN `install-hooks.sh` is run, THEN it SHALL install hooks to `.git/hooks/pre-commit`

### Requirement 4: CI Pipeline - Continuous Integration

**User Story:** As a maintainer, I want automated CI checks on every push and PR, so that code quality is enforced consistently.

#### Acceptance Criteria

1. WHEN code is pushed to main or PR is opened, THEN CI SHALL run formatting and clippy checks
2. WHEN CI runs tests, THEN it SHALL execute on both ubuntu-latest and windows-latest
3. WHEN CI runs Flutter tests, THEN it SHALL execute `flutter analyze` and `flutter test --coverage`
4. WHEN CI runs coverage, THEN it SHALL generate lcov report and upload to Codecov
5. WHEN CI runs security audit, THEN it SHALL execute `cargo audit` and fail on known vulnerabilities
6. WHEN PR CI runs benchmarks, THEN it SHALL compare against main baseline and warn on regression >100µs
7. IF any CI job fails, THEN the entire workflow SHALL be marked as failed
8. WHEN CI starts, THEN it SHALL cancel any in-progress runs for the same branch (concurrency control)

### Requirement 5: CI Pipeline - Release Automation

**User Story:** As a maintainer, I want automated release builds when I push a version tag, so that releases are consistent and require no manual steps.

#### Acceptance Criteria

1. WHEN tag matching `v*` is pushed, THEN release workflow SHALL trigger automatically
2. WHEN release builds, THEN it SHALL compile Rust for x86_64-unknown-linux-gnu
3. WHEN release builds, THEN it SHALL compile Rust for x86_64-pc-windows-msvc
4. WHEN release builds, THEN it SHALL build Flutter for linux-x64
5. WHEN release builds, THEN it SHALL build Flutter for windows-x64
6. WHEN release packages artifacts, THEN it SHALL create .tar.gz for Linux with checksums
7. WHEN release packages artifacts, THEN it SHALL create .zip for Windows with checksums
8. WHEN release publishes, THEN it SHALL create GitHub Release with all artifacts
9. WHEN release publishes, THEN it SHALL generate release notes from git commits
10. IF tag contains `-` (e.g., v1.0.0-beta.1), THEN release SHALL be marked as prerelease

### Requirement 6: CI Pipeline - Maintenance Automation

**User Story:** As a maintainer, I want automated dependency updates and security monitoring, so that the project stays current without manual effort.

#### Acceptance Criteria

1. WHEN Sunday 00:00 UTC occurs (weekly schedule), THEN maintenance workflow SHALL run
2. WHEN maintenance runs, THEN it SHALL execute `cargo update` and create PR if dependencies changed
3. WHEN maintenance runs security audit, THEN it SHALL notify on any new vulnerabilities
4. WHEN maintenance workflow is needed manually, THEN it SHALL support `workflow_dispatch` trigger

### Requirement 7: IDE Configuration

**User Story:** As a developer using VS Code, I want pre-configured IDE settings, so that I have optimal Rust and Flutter development experience out of the box.

#### Acceptance Criteria

1. WHEN developer opens project in VS Code, THEN rust-analyzer SHALL be configured with all features enabled
2. WHEN developer opens project, THEN clippy SHALL be configured as the check command with `-D warnings`
3. WHEN developer saves a file, THEN formatOnSave SHALL trigger for Rust and Dart files
4. WHEN developer opens project, THEN recommended extensions SHALL be suggested
5. WHEN developer uses F5 to debug, THEN launch configurations SHALL be available for Rust CLI and Flutter

### Requirement 8: Development Container

**User Story:** As a developer, I want a pre-configured development container, so that I can start developing immediately in any environment (GitHub Codespaces, VS Code Remote).

#### Acceptance Criteria

1. WHEN devcontainer starts, THEN it SHALL have Rust stable toolchain installed
2. WHEN devcontainer starts, THEN it SHALL have just task runner installed
3. WHEN devcontainer completes setup, THEN it SHALL run `just setup` automatically
4. WHEN devcontainer is ready, THEN developer SHALL be able to run `just check` successfully

### Requirement 9: Release Versioning and Changelog

**User Story:** As a maintainer, I want standardized versioning and automated changelog generation, so that releases are consistent and well-documented.

#### Acceptance Criteria

1. WHEN version is incremented, THEN it SHALL follow Semantic Versioning (MAJOR.MINOR.PATCH)
2. WHEN changelog is generated, THEN it SHALL follow Keep a Changelog format
3. WHEN git-cliff runs, THEN it SHALL categorize commits by conventional commit type (feat, fix, perf, etc.)
4. WHEN release tag is created, THEN version SHALL be updated in Cargo.toml and pubspec.yaml

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility**: Each workflow file handles one concern (CI, release, maintenance)
- **Modular Design**: justfile recipes are composable and independent
- **DRY Principle**: Common steps extracted into reusable patterns

### Performance

- **CI Speed**: Full CI pipeline SHALL complete in <10 minutes for typical PRs
- **Caching**: All workflows SHALL use appropriate caching (Cargo, Flutter, Rust toolchain)
- **Concurrency**: Redundant workflow runs SHALL be cancelled automatically

### Security

- **Secret Management**: All secrets SHALL be stored in GitHub Secrets, never in code
- **Dependency Audit**: `cargo audit` SHALL run on every PR and weekly
- **Permission Scoping**: Release workflow SHALL use minimal required permissions (`contents: write`)

### Reliability

- **Fail Fast**: CI SHALL fail fast on lint/format errors before running slower tests
- **Matrix Strategy**: Test failures on one OS SHALL not block tests on other OS (`fail-fast: false`)
- **Artifact Retention**: Build artifacts SHALL be retained for debugging

### Usability

- **Discoverability**: Running `just` SHALL list all available commands with descriptions
- **Error Messages**: All failures SHALL include actionable error messages
- **Documentation**: README SHALL document build system usage
