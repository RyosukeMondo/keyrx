# Requirements Document

## Introduction

The **AI-Dev-Foundation** spec establishes the critical infrastructure that enables fully autonomous AI-driven development of the keyrx project. This foundation includes:
- **4-crate workspace initialization** (keyrx_core, keyrx_compiler, keyrx_daemon, keyrx_ui)
- **AI-friendly build/test/launch scripts** with machine-parseable output
- **CLAUDE.md documentation** for AI agent guidance
- **Pre-commit hooks and CI/CD** for automated quality enforcement

Without this foundation, AI agents cannot work autonomously—they would lack:
- Consistent, parseable feedback from build/test operations
- Clear rules and patterns for code organization
- Automated verification of code quality standards
- Deterministic, reproducible development workflows

This spec is the **prerequisite for all other feature development**, as it creates the environment where AI agents can confidently implement, test, and verify their work without human intervention.

## Alignment with Product Vision

This spec directly implements the **"AI Coding Agent First"** product principle (product.md):

**From Product Principles**:
> "keyrx is designed to be verified, modified, and deployed by AI agents without human intervention."

**Key Alignments**:

1. **SSOT (Single Source of Truth)**:
   - Scripts output consistent markers (`=== accomplished ===`, `=== failed ===`)
   - AI agents can verify build/test results by parsing structured output
   - No ambiguity in success/failure states

2. **Structured Logging**:
   - All scripts emit JSON-formatted logs when `--json` flag is used
   - Epoch-timestamped log files enable correlation and audit trails
   - AI agents can programmatically analyze failures without human interpretation

3. **Observability & Controllability**:
   - CLAUDE.md documents all scripts, patterns, and conventions
   - AI agents can discover rules/patterns by reading a single source
   - Every operation has clear success criteria

4. **Zero Manual Testing**:
   - Pre-commit hooks enforce clippy, rustfmt, tests automatically
   - CI/CD fails builds that violate quality standards
   - AI agents get immediate, deterministic feedback

**Quality Metrics Enabled**:
- 80% minimum test coverage (enforced by pre-commit hook)
- Max 500 lines/file, max 50 lines/function (enforced by clippy)
- Consistent code formatting (rustfmt)

## Requirements

### Requirement 1: Workspace Initialization

**User Story:** As an **AI coding agent**, I want **all 4 crates initialized with proper structure**, so that **I can immediately start implementing features without scaffolding overhead**.

#### Acceptance Criteria (EARS)

1. WHEN the workspace is initialized THEN the system SHALL create a root `Cargo.toml` with workspace configuration for 4 crates: `keyrx_core`, `keyrx_compiler`, `keyrx_daemon`, `keyrx_ui`

2. WHEN the workspace is initialized THEN the system SHALL create each crate with:
   - `Cargo.toml` with correct dependencies (per tech.md specifications)
   - `src/` directory with entry point (`lib.rs` for libraries, `main.rs` for binaries)
   - `README.md` with crate purpose and basic usage
   - Placeholder modules matching structure.md specifications

3. WHEN `keyrx_core` is initialized THEN it SHALL:
   - Be configured as `no_std` (enables WASM compilation)
   - Include dependencies: rkyv, boomphf, fixedbitset, arrayvec
   - Have placeholder modules: `config.rs`, `lookup.rs`, `dfa.rs`, `state.rs`, `simulator.rs`
   - Include `benches/` directory with Criterion setup
   - Include `fuzz/` directory with cargo-fuzz setup

4. WHEN `keyrx_compiler` is initialized THEN it SHALL:
   - Be a binary crate with CLI argument parsing (clap)
   - Include dependencies: rhai, serde, clap
   - Have placeholder modules: `parser.rs`, `mphf_gen.rs`, `dfa_gen.rs`, `serialize.rs`
   - Include `tests/integration/` directory

5. WHEN `keyrx_daemon` is initialized THEN it SHALL:
   - Be a binary crate with platform-specific features (`linux`, `windows`, `web`)
   - Include Linux dependencies (feature-gated): evdev, uinput, nix
   - Include Windows dependencies (feature-gated): windows-sys
   - Include web server dependencies (feature-gated): axum, tower-http, tokio
   - Have platform-specific modules: `platform/linux.rs`, `platform/windows.rs`
   - Have web server modules: `web/mod.rs`, `web/api.rs`, `web/ws.rs`, `web/static_files.rs`
   - Include `ui_dist/` directory for embedded UI files

6. WHEN `keyrx_ui` is initialized THEN it SHALL:
   - Have `package.json` with React 18+, TypeScript 5+, Vite dependencies
   - Have `vite.config.ts` configured for WASM integration
   - Include `src/` with `App.tsx`, `components/`, `wasm/`, `hooks/` directories
   - Include `.gitignore` for node_modules, dist

7. WHEN the workspace is initialized THEN it SHALL create a root `.gitignore` with:
   - Rust build artifacts (`target/`, `Cargo.lock`)
   - Node.js artifacts (`node_modules/`, `dist/`)
   - Log files (`scripts/logs/*.log`)
   - OS-specific files (`.DS_Store`, `Thumbs.db`)

### Requirement 2: AI-Friendly Build Scripts

**User Story:** As an **AI coding agent**, I want **consistent, parseable build/test scripts**, so that **I can autonomously verify my work without human interpretation**.

#### Acceptance Criteria (EARS)

1. WHEN a script is executed THEN it SHALL output consistent status markers:
   - `=== accomplished ===` on successful completion
   - `=== failed ===` on failure
   - `=== warning ===` for non-critical issues

2. WHEN a script is executed THEN it SHALL:
   - Write timestamped logs to `scripts/logs/[script]_$(date +%s).log`
   - Use epoch timestamp in filename (e.g., `build_1766294000.log`)
   - Output structured logs in format: `[YYYY-MM-DD HH:MM:SS] [LEVEL] message`

3. WHEN a script is executed with `--error` flag THEN it SHALL:
   - Output ONLY error-level messages
   - Filter out INFO and DEBUG messages
   - Enable AI agents to focus on failures without noise

4. WHEN a script is executed with `--json` flag THEN it SHALL:
   - Output machine-readable JSON format
   - Include fields: `timestamp`, `level`, `message`, `context`
   - Exit with JSON summary: `{"status": "success|failed", "duration_ms": 1234}`

5. WHEN a script is executed with `--quiet` flag THEN it SHALL:
   - Suppress all output except final status marker
   - Enable silent CI/CD execution

6. WHEN a script is executed with `--log-file <path>` THEN it SHALL:
   - Write logs to the specified path instead of default location
   - Create parent directories if they don't exist

7. WHEN `build.sh` is executed THEN it SHALL:
   - Run `cargo build --workspace`
   - Support `--release` flag for optimized builds
   - Support `--watch` flag for continuous builds (using `cargo-watch`)
   - Output success marker on clean build, failure marker on errors

8. WHEN `verify.sh` is executed THEN it SHALL:
   - Run `cargo clippy -- -D warnings` (treat warnings as errors)
   - Run `cargo fmt --check`
   - Run `cargo test --workspace`
   - Run `cargo tarpaulin` to check test coverage (80% minimum)
   - Fail if any check fails
   - Output summary of all checks with pass/fail status

9. WHEN `test.sh` is executed THEN it SHALL:
   - Support `--unit` flag: run only unit tests
   - Support `--integration` flag: run only integration tests
   - Support `--fuzz` flag: run fuzzing for specified duration (default 60s)
   - Run all tests by default
   - Output test results with pass/fail counts

10. WHEN `launch.sh` is executed THEN it SHALL:
    - Support `--headless` flag: start daemon without web UI
    - Support `--debug` flag: enable debug logging
    - Start daemon with default config (or specified config via `--config` flag)
    - Output daemon PID and listening ports

11. WHEN any script exits THEN it SHALL:
    - Return exit code 0 on success
    - Return exit code 1 on error
    - Return exit code 2 on warnings (non-critical)

### Requirement 3: CLAUDE.md Documentation

**User Story:** As an **AI coding agent**, I want **comprehensive, structured documentation**, so that **I can discover rules, patterns, and conventions without asking humans**.

#### Acceptance Criteria (EARS)

1. WHEN `scripts/CLAUDE.md` is created THEN it SHALL document:
   - Purpose of each script (`build.sh`, `verify.sh`, `test.sh`, `launch.sh`)
   - All supported flags and their effects
   - Expected output format (status markers, log format)
   - Example usage for common scenarios

2. WHEN `scripts/CLAUDE.md` is created THEN it SHALL include:
   - **Script Reference Table**: Script name, purpose, common flags, exit codes
   - **Output Format Specification**: Status markers, log structure, JSON schema
   - **Example Commands**: At least 3 examples per script
   - **Failure Scenarios**: Common errors and how to interpret them

3. WHEN `scripts/CLAUDE.md` is created THEN it SHALL be:
   - **Structured**: Use markdown headers for easy navigation
   - **Minimal**: Focus on actionable information, avoid prose
   - **Examples-first**: Show examples before explaining

4. WHEN `.claude/CLAUDE.md` (root) is created THEN it SHALL document:
   - **Project Structure**: 4-crate workspace overview
   - **Code Quality Rules**: Max 500 lines/file, max 50 lines/function, 80% coverage
   - **Architecture Patterns**: SOLID, DI, SSOT, KISS (from structure.md)
   - **Naming Conventions**: Rust (snake_case), TypeScript (camelCase/PascalCase)
   - **Import Patterns**: Rust module structure, TypeScript import order

5. WHEN `.claude/CLAUDE.md` is created THEN it SHALL include:
   - **AI-Agent Quick Start**: Steps to verify environment, run first build, run tests
   - **Common Tasks**: How to add a new module, add a test, run specific tests
   - **Troubleshooting**: Common errors and fixes

### Requirement 4: Pre-Commit Hooks

**User Story:** As an **AI coding agent**, I want **automated quality enforcement**, so that **I get immediate feedback before committing code**.

#### Acceptance Criteria (EARS)

1. WHEN pre-commit hooks are installed THEN they SHALL run before every `git commit`

2. WHEN pre-commit hooks are executed THEN they SHALL:
   - Run `cargo clippy -- -D warnings` (treat warnings as errors)
   - Run `cargo fmt --check` (fail if code is not formatted)
   - Run `cargo test --workspace` (fail if tests fail)
   - Abort commit if any check fails

3. WHEN pre-commit hooks are installed THEN the system SHALL create `.git/hooks/pre-commit` with:
   - Executable permissions (`chmod +x`)
   - Calls to `scripts/verify.sh --quiet`
   - Clear failure messages indicating which check failed

4. WHEN `scripts/setup_hooks.sh` is created THEN it SHALL:
   - Install pre-commit hook to `.git/hooks/pre-commit`
   - Output success message confirming installation
   - Be idempotent (safe to run multiple times)

### Requirement 5: CI/CD Setup

**User Story:** As an **AI coding agent**, I want **automated CI/CD**, so that **all code changes are automatically verified before merging**.

#### Acceptance Criteria (EARS)

1. WHEN `.github/workflows/ci.yml` is created THEN it SHALL:
   - Run on every push to any branch
   - Run on every pull request
   - Execute `scripts/verify.sh` (clippy, fmt, tests, coverage)
   - Fail the workflow if verification fails

2. WHEN CI workflow runs THEN it SHALL:
   - Cache Cargo dependencies for faster builds
   - Cache npm dependencies (for keyrx_ui)
   - Run on multiple platforms: `ubuntu-latest`, `windows-latest`
   - Upload coverage reports to CI artifacts

3. WHEN `.github/workflows/release.yml` is created THEN it SHALL:
   - Run only on git tags matching `v*.*.*` (semver)
   - Build release binaries for Linux (x86_64) and Windows (x86_64)
   - Cross-compile using `cross` crate
   - Create GitHub Release with binaries attached

4. WHEN CI/CD workflows are created THEN they SHALL:
   - Have clear job names (e.g., "Clippy Lint", "Format Check", "Unit Tests")
   - Output structured logs readable by AI agents
   - Include timeout limits (30 minutes max per job)

### Requirement 6: Makefile Orchestration

**User Story:** As an **AI coding agent**, I want **simple top-level commands**, so that **I can build/test/deploy without memorizing complex scripts**.

#### Acceptance Criteria (EARS)

1. WHEN a `Makefile` is created THEN it SHALL define targets:
   - `make build`: Run `scripts/build.sh`
   - `make verify`: Run `scripts/verify.sh`
   - `make test`: Run `scripts/test.sh`
   - `make launch`: Run `scripts/launch.sh`
   - `make clean`: Remove build artifacts (`target/`, `node_modules/`, `dist/`)
   - `make setup`: Install pre-commit hooks and dev tools

2. WHEN `make` is run without target THEN it SHALL:
   - Run `make build` by default
   - Output available targets with descriptions

3. WHEN `make setup` is run THEN it SHALL:
   - Install pre-commit hooks (via `scripts/setup_hooks.sh`)
   - Install required tools: `cargo-watch`, `cargo-tarpaulin`, `cargo-fuzz`, `wasm-pack`
   - Verify installations by checking versions
   - Output success message

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility**: Each script does ONE thing (build, verify, test, launch)
- **Modular Design**: Scripts can be composed (e.g., `verify.sh` calls `build.sh`)
- **Clear Interfaces**: All scripts accept same flags (`--error`, `--json`, `--quiet`, `--log-file`)
- **Idempotency**: Scripts can be run multiple times safely (e.g., `make setup`)

### Performance

- **Build Time**: Initial build <5 minutes on modern hardware
- **Test Time**: All unit tests <30 seconds
- **CI Time**: Full CI pipeline <10 minutes (with caching)
- **Script Startup**: Script overhead <100ms (parsing args, setting up logs)

### Reliability

- **Exit Codes**: Scripts MUST return correct exit codes (0=success, 1=error, 2=warning)
- **Atomicity**: Scripts either succeed completely or fail completely (no partial states)
- **Error Messages**: All failures include actionable error messages
- **Determinism**: Same inputs → same outputs (no randomness)

### Usability (for AI Agents)

- **Consistent Patterns**: All scripts follow same flag conventions
- **Machine-Parseable**: JSON output mode for structured data
- **Self-Documenting**: Scripts output usage help with `--help` flag
- **Discoverable**: `scripts/CLAUDE.md` is the single source of truth

### Security

- **No Secrets in Logs**: Scripts MUST NOT log sensitive data (API keys, passwords)
- **Safe Defaults**: Scripts run with least privileges (no unnecessary `sudo`)
- **Input Validation**: Scripts validate all arguments before execution

### Compatibility

- **Linux**: Scripts work on Ubuntu 22.04+, Fedora 38+, Arch Linux (Bash 5+)
- **Windows**: Scripts work on Windows 10+ (PowerShell 7+, or WSL with Bash)
- **CI**: Scripts work in GitHub Actions Ubuntu and Windows runners
