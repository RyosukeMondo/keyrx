# Tasks Document

## Phase 1: Workspace Initialization

- [x] 1. Create 4-crate Cargo workspace structure
  - File: Cargo.toml (root)
  - Create workspace configuration with 4 members: keyrx_core, keyrx_compiler, keyrx_daemon, keyrx_ui
  - Add workspace-level dependencies and settings
  - Purpose: Establish foundation for multi-crate project
  - _Leverage: Cargo workspace documentation_
  - _Requirements: 1.1_
  - _Prompt: Role: Rust Developer specializing in workspace architecture | Task: Create root Cargo.toml with workspace configuration for 4 crates following requirement 1.1 | Restrictions: Must use Rust edition 2021, follow workspace best practices, do not add unnecessary global dependencies | Success: Workspace compiles with all members, resolver is set to "2", workspace dependencies are properly configured_

- [x] 1.1 Initialize keyrx_core library crate
  - File: keyrx_core/Cargo.toml, keyrx_core/src/lib.rs
  - Run `cargo new --lib keyrx_core`
  - Configure as `no_std` crate with `#![no_std]` attribute
  - Add dependencies: rkyv (with validation features), boomphf, fixedbitset, arrayvec
  - Create placeholder modules: config.rs, lookup.rs, dfa.rs, state.rs, simulator.rs
  - Create benches/ directory with Criterion setup
  - Create fuzz/ directory with cargo-fuzz setup
  - Create README.md documenting crate purpose
  - Purpose: Core keyboard remapping logic (OS-agnostic, WASM-compatible)
  - _Leverage: tech.md specifications_
  - _Requirements: 1.3_
  - _Prompt: Role: Rust Systems Programmer with expertise in no_std and embedded development | Task: Initialize keyrx_core as no_std library following requirement 1.3, adding all specified dependencies and placeholder modules from tech.md | Restrictions: Must maintain no_std compatibility (use core::, alloc::), do not use std library, ensure WASM-compatible dependencies | Success: Crate compiles as no_std library, all placeholder modules exist and compile, benches and fuzz directories are properly configured_

- [x] 1.2 Initialize keyrx_compiler binary crate
  - File: keyrx_compiler/Cargo.toml, keyrx_compiler/src/main.rs
  - Run `cargo new --bin keyrx_compiler`
  - Add dependencies: rhai, serde (with derive feature), clap (v4, with derive feature)
  - Create placeholder modules: parser.rs, mphf_gen.rs, dfa_gen.rs, serialize.rs
  - Create tests/integration/ directory with sample test
  - Create README.md documenting compiler usage
  - Purpose: Rhai-to-binary configuration compiler
  - _Leverage: tech.md specifications_
  - _Requirements: 1.4_
  - _Prompt: Role: Rust Developer with expertise in CLI applications and DSL compilers | Task: Initialize keyrx_compiler binary following requirement 1.4, adding clap for CLI, rhai for scripting, and all placeholder modules | Restrictions: Must use clap derive macros, configure serde with derive feature, create proper CLI structure | Success: Binary compiles and runs with --help flag, all placeholder modules compile, integration tests directory is set up_

- [x] 1.3 Initialize keyrx_daemon binary crate
  - File: keyrx_daemon/Cargo.toml, keyrx_daemon/src/main.rs
  - Run `cargo new --bin keyrx_daemon`
  - Add feature flags: linux, windows, web (default)
  - Add Linux dependencies (feature-gated): evdev, uinput, nix
  - Add Windows dependencies (feature-gated): windows-sys
  - Add web dependencies (default feature): axum, tower-http, tokio (with full features)
  - Create platform-specific modules: platform/linux.rs, platform/windows.rs, platform/mod.rs
  - Create web server modules: web/mod.rs, web/api.rs, web/ws.rs, web/static_files.rs
  - Create ui_dist/ directory for embedded UI files
  - Create README.md documenting daemon usage
  - Purpose: OS-level keyboard interception and web server
  - _Leverage: tech.md specifications_
  - _Requirements: 1.5_
  - _Prompt: Role: Rust Systems Programmer with expertise in platform-specific programming and async web servers | Task: Initialize keyrx_daemon binary following requirement 1.5, configuring platform-specific features (linux/windows) and axum web server | Restrictions: Must use feature gates correctly, ensure platform code only compiles on correct OS, use tokio runtime properly | Success: Daemon compiles on both Linux and Windows with correct features, web server modules are structured, platform-specific code is properly gated_

- [x] 1.4 Initialize keyrx_ui frontend project
  - File: keyrx_ui/package.json, keyrx_ui/vite.config.ts, keyrx_ui/src/App.tsx
  - Run `npm create vite@latest keyrx_ui -- --template react-ts`
  - Configure dependencies: React 18+, TypeScript 5+, vite-plugin-wasm
  - Update vite.config.ts for WASM integration
  - Create src/components/, src/wasm/, src/hooks/ directories
  - Create basic App.tsx with placeholder UI
  - Create .gitignore for node_modules, dist
  - Create README.md documenting UI development
  - Purpose: React-based web interface with WASM integration
  - _Leverage: tech.md specifications_
  - _Requirements: 1.6_
  - _Prompt: Role: Frontend Developer specializing in React, TypeScript, and WASM | Task: Initialize keyrx_ui using Vite with React and TypeScript following requirement 1.6, configuring WASM integration | Restrictions: Must use React 18+, TypeScript 5+, configure vite-plugin-wasm correctly, follow modern React practices | Success: npm install succeeds, npm run dev starts dev server, WASM plugin is configured, directory structure matches specification_

- [x] 1.5 Create root .gitignore
  - File: .gitignore (root)
  - Add Rust build artifacts: target/, Cargo.lock (for binaries)
  - Add Node.js artifacts: node_modules/, dist/, .vite/
  - Add log files: scripts/logs/*.log
  - Add OS-specific files: .DS_Store, Thumbs.db, desktop.ini
  - Add IDE files: .vscode/, .idea/, *.swp
  - Purpose: Prevent committing build artifacts and temporary files
  - _Requirements: 1.7_
  - _Prompt: Role: DevOps Engineer with expertise in Git workflows | Task: Create comprehensive .gitignore following requirement 1.7, covering Rust, Node.js, logs, and OS-specific files | Restrictions: Do not ignore source files or important configs, follow gitignore best practices | Success: Build and run workspace, verify no artifacts are tracked by git, only source files appear in git status_

## Phase 2: Script Infrastructure

- [ ] 2. Create script library with common functions
  - File: scripts/lib/common.sh
  - Create mkdir -p scripts/lib
  - Implement logging functions: log_info, log_error, log_warn, log_debug
  - Implement argument parsing helpers: parse_common_flags
  - Implement exit code checker: check_exit_code
  - Implement timestamp functions: get_timestamp, get_epoch_timestamp
  - Implement log file manager: setup_log_file
  - Implement JSON output formatter: output_json
  - Purpose: Shared utilities for all scripts (DRY principle)
  - _Leverage: Bash best practices, structure.md patterns_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Role: DevOps Engineer specializing in Bash scripting and automation | Task: Create comprehensive script library following requirements 2.1 and 2.2, implementing consistent logging, argument parsing, and JSON formatting utilities | Restrictions: Must be POSIX-compliant where possible, use Bash 5+ features when needed, ensure functions are reusable and testable | Success: Library sources without errors, all functions work correctly, logging outputs in specified format ([YYYY-MM-DD HH:MM:SS] [LEVEL] message)_

- [ ] 2.1 Create scripts/logs/ directory structure
  - File: scripts/logs/.gitkeep
  - Create mkdir -p scripts/logs
  - Add .gitkeep to track directory in git
  - Update .gitignore to ignore *.log files in this directory
  - Purpose: Centralized location for script execution logs
  - _Requirements: 2.2_
  - _Prompt: Role: DevOps Engineer with expertise in logging infrastructure | Task: Create logs directory structure following requirement 2.2 for epoch-timestamped log storage | Restrictions: Do not commit log files, ensure directory exists in fresh clones | Success: Directory exists, .gitkeep is committed, *.log files are ignored by git_

## Phase 3: Build and Verification Scripts

- [ ] 3. Implement build.sh script
  - File: scripts/build.sh
  - Add shebang (#!/bin/bash) and source scripts/lib/common.sh
  - Implement argument parsing: --release, --watch, --error, --json, --quiet, --log-file
  - Setup log file with epoch timestamp: build_$(date +%s).log
  - Implement build logic: cargo build --workspace [--release]
  - Implement watch mode: cargo watch -x "build [--release]" (check cargo-watch installed)
  - Add exit code checking with status markers (=== accomplished ===, === failed ===)
  - Add JSON output mode support
  - Make executable: chmod +x scripts/build.sh
  - Purpose: Consistent, parseable build automation
  - _Leverage: scripts/lib/common.sh, design.md Component 2_
  - _Requirements: 2.7_
  - _Prompt: Role: Build Engineer with expertise in Rust toolchain and automation | Task: Implement build.sh following requirement 2.7 and design.md Component 2, supporting all specified flags and output modes | Restrictions: Must use cargo build, check tool availability before use, handle errors gracefully, output consistent markers | Success: Script builds workspace successfully, --release flag works, --watch mode runs continuously, --json outputs valid JSON, exit codes are correct (0 on success, 1 on failure)_

- [ ] 3.1 Implement verify.sh script
  - File: scripts/verify.sh
  - Add shebang and source scripts/lib/common.sh
  - Implement argument parsing: --skip-coverage, --error, --json, --quiet, --log-file
  - Setup log file with epoch timestamp: verify_$(date +%s).log
  - Implement verification steps:
    1. Run scripts/build.sh --quiet (ensure clean build first)
    2. Run cargo clippy --workspace -- -D warnings
    3. Run cargo fmt --check
    4. Run cargo test --workspace
    5. Run cargo tarpaulin --workspace --out Xml (check 80% minimum)
  - Generate summary table with pass/fail status for each check
  - Add JSON output mode with per-check results
  - Abort on first failure with clear error message
  - Make executable: chmod +x scripts/verify.sh
  - Purpose: Comprehensive quality verification in one command
  - _Leverage: scripts/lib/common.sh, design.md Component 3_
  - _Requirements: 2.8_
  - _Prompt: Role: QA Automation Engineer with expertise in Rust tooling and quality gates | Task: Implement verify.sh following requirement 2.8 and design.md Component 3, running clippy, fmt, tests, and coverage checks | Restrictions: Must check tool availability (cargo-tarpaulin), abort on first failure, output summary table, enforce 80% minimum coverage | Success: Script runs all checks in order, stops on first failure, outputs summary table, --skip-coverage bypasses tarpaulin, coverage threshold is enforced_

- [ ] 3.2 Implement test.sh script
  - File: scripts/test.sh
  - Add shebang and source scripts/lib/common.sh
  - Implement argument parsing: --unit, --integration, --fuzz DURATION, --bench, --error, --json, --quiet, --log-file
  - Setup log file with epoch timestamp: test_$(date +%s).log
  - Implement test modes:
    - Default: cargo test --workspace
    - --unit: cargo test --lib --workspace
    - --integration: cargo test --test '*' --workspace
    - --fuzz: cd keyrx_core/fuzz && cargo fuzz run fuzz_target_1 -- -max_total_time=DURATION
    - --bench: cargo +nightly bench --workspace
  - Add test result summary (passed/failed counts)
  - Add JSON output mode
  - Make executable: chmod +x scripts/test.sh
  - Purpose: Flexible test execution with filtering
  - _Leverage: scripts/lib/common.sh, design.md Component 4_
  - _Requirements: 2.9_
  - _Prompt: Role: Test Engineer with expertise in Rust testing frameworks and fuzzing | Task: Implement test.sh following requirement 2.9 and design.md Component 4, supporting unit, integration, fuzz, and benchmark tests | Restrictions: Must validate tool availability (cargo-fuzz for --fuzz, nightly for --bench), parse test output for summary, handle different test types correctly | Success: Script runs all test types correctly, --unit runs only lib tests, --integration runs tests/ directory, --fuzz works with duration parameter, summary shows passed/failed counts_

- [ ] 3.3 Implement launch.sh script
  - File: scripts/launch.sh
  - Add shebang and source scripts/lib/common.sh
  - Implement argument parsing: --headless, --debug, --config PATH, --release, --error, --json, --quiet, --log-file
  - Setup log file with epoch timestamp: launch_$(date +%s).log
  - Build daemon first: cargo build --bin keyrx_daemon [--release]
  - Construct daemon arguments: --log-level [debug|info], --config PATH, --headless
  - Launch daemon and capture PID
  - Output daemon status: PID, listening ports (parse from daemon output)
  - Add JSON output mode with PID and ports
  - Make executable: chmod +x scripts/launch.sh
  - Purpose: Consistent daemon launch with configuration options
  - _Leverage: scripts/lib/common.sh, design.md Component 5_
  - _Requirements: 2.10_
  - _Prompt: Role: DevOps Engineer with expertise in service management and process control | Task: Implement launch.sh following requirement 2.10 and design.md Component 5, building and launching daemon with specified options | Restrictions: Must build daemon first, capture PID correctly, parse daemon output for ports, handle daemon startup failures | Success: Script builds and launches daemon successfully, outputs PID and ports, --headless suppresses browser, --debug enables debug logging, --config uses custom config file_

## Phase 4: Git Hooks and Development Tools

- [ ] 4. Implement setup_hooks.sh script
  - File: scripts/setup_hooks.sh
  - Add shebang and source scripts/lib/common.sh (optional for this script)
  - Check if .git/ directory exists (exit with error if not a git repo)
  - Create pre-commit hook at .git/hooks/pre-commit:
    - Add shebang: #!/bin/bash
    - Echo "Running pre-commit verification..."
    - Call scripts/verify.sh --quiet
    - Check exit code, abort commit if non-zero
    - Add helpful error message
  - Make hook executable: chmod +x .git/hooks/pre-commit
  - Output success message
  - Make script executable: chmod +x scripts/setup_hooks.sh
  - Purpose: Install automated quality gates before commits
  - _Leverage: design.md Component 6_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Role: DevOps Engineer with expertise in Git hooks and automation | Task: Implement setup_hooks.sh following requirements 4.1-4.3 and design.md Component 6, installing pre-commit hook that runs verify.sh | Restrictions: Must check if git repo exists, make hook executable, be idempotent (safe to run multiple times), output clear messages | Success: Script creates pre-commit hook successfully, hook runs on commit and blocks bad code, script can be run multiple times safely_

## Phase 5: CI/CD Configuration

- [ ] 5. Create GitHub Actions CI workflow
  - File: .github/workflows/ci.yml
  - Create directory: mkdir -p .github/workflows
  - Define workflow:
    - Name: "CI"
    - Triggers: push to any branch, pull requests
    - Jobs:
      - verify: runs on ubuntu-latest and windows-latest (matrix)
      - Steps: checkout, setup Rust (stable with clippy/rustfmt), cache Cargo, install tools (tarpaulin, watch), run verify.sh --json
      - Upload coverage artifact (Ubuntu only)
  - Add timeouts: 30 minutes max per job
  - Add caching for Cargo registry, git, and target/
  - Purpose: Automated verification on every push/PR
  - _Leverage: design.md Component 7, GitHub Actions best practices_
  - _Requirements: 5.1, 5.2_
  - _Prompt: Role: CI/CD Engineer with expertise in GitHub Actions and Rust workflows | Task: Create comprehensive CI workflow following requirements 5.1-5.2 and design.md Component 7, running on Linux and Windows with caching | Restrictions: Must use stable Rust toolchain, cache dependencies correctly, upload coverage only on Ubuntu, set reasonable timeouts | Success: Workflow runs on push and PR, completes successfully on both platforms, caching speeds up builds, coverage reports are uploaded_

- [ ] 5.1 Create GitHub Actions release workflow
  - File: .github/workflows/release.yml
  - Define workflow:
    - Name: "Release"
    - Triggers: push tags matching v*.*.*
    - Jobs:
      - build-release: runs on ubuntu-latest and windows-latest (matrix)
      - Steps: checkout, setup Rust (stable), build release binaries, create GitHub release
      - Upload binaries: keyrx_compiler, keyrx_daemon
  - Add matrix for targets: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc
  - Use softprops/action-gh-release for release creation
  - Purpose: Automated binary releases on version tags
  - _Leverage: design.md Component 7_
  - _Requirements: 5.3_
  - _Prompt: Role: Release Engineer with expertise in GitHub Actions and cross-compilation | Task: Create release workflow following requirement 5.3 and design.md Component 7, building and uploading binaries on version tags | Restrictions: Must only run on semver tags (v*.*.*), build release binaries (--release), upload both compiler and daemon, use action-gh-release | Success: Workflow triggers on version tags, builds release binaries for Linux and Windows, creates GitHub release with binaries attached_

## Phase 6: Documentation

- [ ] 6. Create scripts/CLAUDE.md documentation
  - File: scripts/CLAUDE.md
  - Create comprehensive script documentation:
    - **Introduction**: Purpose of automation scripts
    - **Script Reference Table**: All scripts with purpose, common flags, exit codes
    - **Output Format Specification**:
      - Status markers: === accomplished ===, === failed ===, === warning ===
      - Log format: [YYYY-MM-DD HH:MM:SS] [LEVEL] message
      - JSON schema for --json mode
    - **Flag Reference**: All common flags (--error, --json, --quiet, --log-file)
    - **Example Commands**: At least 3 examples per script
    - **Failure Scenarios**: Common errors and how to interpret them
    - **Troubleshooting**: FAQ and common issues
  - Purpose: Single source of truth for script usage (AI agents read this first)
  - _Leverage: design.md Component 8_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Role: Technical Writer with expertise in developer documentation and automation | Task: Create comprehensive scripts/CLAUDE.md following requirements 3.1-3.2 and design.md Component 8, documenting all scripts with examples and troubleshooting | Restrictions: Must be concise and scannable (AI-friendly), use tables for reference data, include concrete examples, focus on actionable information | Success: Documentation covers all scripts with clear examples, output formats are precisely specified, failure scenarios include fixes, AI agents can discover all script capabilities_

- [ ] 6.1 Create .claude/CLAUDE.md root documentation
  - File: .claude/CLAUDE.md
  - Create directory: mkdir -p .claude
  - Create comprehensive AI agent guidance:
    - **AI-Agent Quick Start**:
      - Verify environment (Rust, Node.js versions)
      - Run first build: make build
      - Run tests: make test
      - Run verification: make verify
    - **Project Structure**: 4-crate workspace overview with purposes
    - **Code Quality Rules**:
      - Max 500 lines/file (enforced by clippy)
      - Max 50 lines/function
      - 80% minimum test coverage
      - SOLID, DI, SSOT, KISS principles
    - **Architecture Patterns**:
      - SOLID principles with examples
      - Dependency Injection pattern
      - SSOT mechanism (hash-based verification)
    - **Naming Conventions**:
      - Rust: snake_case (modules/functions), PascalCase (types/traits)
      - TypeScript: camelCase (variables/functions), PascalCase (types/components)
    - **Import Patterns**:
      - Rust: module structure and use statements
      - TypeScript: import order (external, internal, relative, styles)
    - **Common Tasks**:
      - How to add a new module (with example)
      - How to add a test (with example)
      - How to run specific tests
      - How to add a dependency
  - Purpose: AI agent onboarding and development guidance
  - _Leverage: design.md Component 8, structure.md patterns_
  - _Requirements: 3.3, 3.4, 3.5_
  - _Prompt: Role: Developer Advocate with expertise in onboarding documentation and architectural patterns | Task: Create comprehensive .claude/CLAUDE.md following requirements 3.3-3.5 and design.md Component 8, providing AI agents with quick start and development patterns | Restrictions: Must be example-driven (show first, explain second), focus on actionable steps, include code snippets, reference steering documents for details | Success: Documentation enables AI agent to start contributing immediately, all quality rules are clearly stated, architecture patterns are explained with examples, common tasks have step-by-step instructions_

## Phase 7: Makefile Orchestration

- [ ] 7. Create root Makefile
  - File: Makefile (root)
  - Define .PHONY targets: help, build, verify, test, launch, clean, setup
  - Set default goal: help
  - Implement targets:
    - help: List all targets with descriptions
    - build: Call scripts/build.sh
    - verify: Call scripts/verify.sh
    - test: Call scripts/test.sh
    - launch: Call scripts/launch.sh
    - clean: Remove target/, node_modules/, dist/, logs/*.log
    - setup: Install tools (cargo-watch, cargo-tarpaulin, cargo-fuzz, wasm-pack) and run scripts/setup_hooks.sh
  - Add @ prefix to suppress command echo (cleaner output)
  - Purpose: Simple top-level commands for common operations
  - _Leverage: design.md Component 9_
  - _Requirements: 6.1, 6.2, 6.3_
  - _Prompt: Role: Build Engineer with expertise in Make and automation | Task: Create root Makefile following requirements 6.1-6.3 and design.md Component 9, providing simple commands for build, verify, test, launch, setup | Restrictions: Must use .PHONY for all targets, suppress command echo with @, make setup idempotent, clean should be safe (not delete source) | Success: All targets work correctly, make without args shows help, make setup installs all tools and hooks, make clean removes only artifacts_

## Phase 8: Testing and Validation

- [ ] 8. Write script unit tests with BATS
  - File: scripts/tests/test_build.bats, scripts/tests/test_verify.bats
  - Install BATS testing framework (document in setup target)
  - Create scripts/tests/ directory
  - Write tests for build.sh:
    - Test successful build (expect exit 0, output contains "=== accomplished ===")
    - Test with --release flag (verify optimized build)
    - Test with --json flag (validate JSON output with jq)
    - Test failure scenario (temporarily break code, expect exit 1)
  - Write tests for verify.sh:
    - Test all checks pass (clean code)
    - Test clippy failure (introduce warning)
    - Test format failure (unformatted code)
    - Test coverage failure (delete tests)
  - Purpose: Ensure scripts behave correctly and fail gracefully
  - _Leverage: design.md Testing Strategy_
  - _Requirements: All script requirements (2.7-2.11)_
  - _Prompt: Role: Test Automation Engineer with expertise in Bash testing and BATS framework | Task: Write comprehensive unit tests for build.sh and verify.sh following design.md Testing Strategy, covering success and failure scenarios | Restrictions: Must test both success and failure cases, validate output format, check exit codes, use BATS best practices (setup/teardown), do not modify production code | Success: All tests pass, scripts are validated for correct behavior, failure scenarios are tested, JSON output is validated with jq_

- [ ] 8.1 Create integration test for full workflow
  - File: scripts/tests/integration_test.sh
  - Write end-to-end workflow test:
    1. Fresh workspace check (verify all crates exist)
    2. Run make build (expect success)
    3. Run make test (expect success)
    4. Run make verify (expect success)
    5. Introduce error, run make verify (expect failure)
    6. Fix error, run make verify (expect success)
    7. Test pre-commit hook (setup, commit bad code, expect block)
  - Make executable: chmod +x scripts/tests/integration_test.sh
  - Purpose: Validate entire development workflow end-to-end
  - _Leverage: design.md Testing Strategy (Integration Testing)_
  - _Requirements: All requirements (integration test)_
  - _Prompt: Role: Integration Test Engineer with expertise in workflow validation | Task: Create comprehensive integration test following design.md Testing Strategy, validating full development workflow from workspace init to pre-commit hooks | Restrictions: Must test real workflow (not mocked), restore state after tests, verify each step succeeds/fails as expected, document test scenario | Success: Integration test covers full workflow, all steps are verified, test is reproducible, passes on clean workspace_

- [ ] 8.2 Validate AI agent autonomous workflow
  - File: scripts/tests/ai_agent_simulation.sh
  - Simulate AI agent development cycle:
    1. AI reads .claude/CLAUDE.md
    2. AI runs make build (verifies environment)
    3. AI adds new module keyrx_core/src/test_module.rs
    4. AI runs make verify (should fail - no tests)
    5. AI adds test keyrx_core/tests/test_module.rs
    6. AI runs make verify (should succeed)
    7. AI commits (pre-commit hook runs, succeeds)
  - Verify AI can complete cycle without human intervention
  - Make executable: chmod +x scripts/tests/ai_agent_simulation.sh
  - Purpose: Prove foundation enables fully autonomous AI development
  - _Leverage: design.md Testing Strategy (E2E Testing)_
  - _Requirements: All requirements (validates entire foundation)_
  - _Prompt: Role: AI/ML Engineer with expertise in autonomous agent testing and validation | Task: Create E2E AI agent simulation following design.md Testing Strategy, proving AI can autonomously develop, test, and commit code | Restrictions: Must simulate real AI workflow (read docs, run commands, interpret output), verify autonomy (no human intervention), document each step, clean up after test | Success: Simulation completes full development cycle autonomously, AI interprets script output correctly, pre-commit hooks work, no human intervention needed_

## Phase 9: Final Integration and Documentation

- [ ] 9. Verify all crates build and tests pass
  - Run make clean to remove all artifacts
  - Run make build (verify all 4 crates compile)
  - Run make test (verify all tests pass)
  - Run make verify (verify clippy, fmt, coverage all pass)
  - Fix any issues discovered
  - Purpose: Final sanity check before delivery
  - _Requirements: All requirements_
  - _Prompt: Role: QA Engineer with expertise in integration testing and release validation | Task: Perform final verification of entire workspace, ensuring all crates build, tests pass, and quality checks succeed | Restrictions: Must start from clean state (make clean), verify each step independently, document any issues found, do not skip steps | Success: All crates build successfully, all tests pass, all quality checks pass (clippy, fmt, coverage ≥80%), no errors or warnings_

- [ ] 9.1 Test on fresh clone (simulate new developer)
  - Clone repository to new location (or use fresh VM/container)
  - Follow AI-Agent Quick Start from .claude/CLAUDE.md:
    1. Verify Rust and Node.js installed
    2. Run make setup (install tools and hooks)
    3. Run make build
    4. Run make test
    5. Run make verify
  - Verify all steps complete successfully without prior knowledge
  - Document any missing instructions in CLAUDE.md
  - Purpose: Ensure onboarding documentation is complete and accurate
  - _Requirements: 3.3 (AI-Agent Quick Start)_
  - _Prompt: Role: New Developer (simulated) with no prior project knowledge | Task: Follow .claude/CLAUDE.md Quick Start exactly as written, documenting any missing steps or unclear instructions | Restrictions: Must not use prior project knowledge, follow only what's documented, report any ambiguities or failures, verify each step works | Success: Fresh clone onboards successfully following documentation only, all steps complete without errors, no undocumented dependencies or steps_

- [ ] 9.2 Create final verification checklist
  - File: .spec-workflow/specs/ai-dev-foundation/VERIFICATION.md
  - Create checklist documenting all acceptance criteria from requirements.md:
    - Workspace initialization (4 crates with correct structure)
    - Scripts (build, verify, test, launch, setup_hooks)
    - Output formats (status markers, logs, JSON)
    - Pre-commit hooks (installed and working)
    - CI/CD workflows (present and correct)
    - CLAUDE.md documentation (complete and accurate)
    - Makefile targets (all working)
  - Check each requirement systematically
  - Document evidence of completion (e.g., "✓ Requirement 1.1: workspace Cargo.toml exists with 4 members")
  - Purpose: Prove all requirements are met before spec completion
  - _Requirements: All requirements_
  - _Prompt: Role: QA Lead with expertise in acceptance testing and requirements traceability | Task: Create comprehensive verification checklist mapping all requirements to evidence of completion, systematically validating entire spec | Restrictions: Must check every acceptance criterion from requirements.md, provide concrete evidence (file exists, command succeeds), do not assume completion, be thorough | Success: Checklist covers all requirements, each item has evidence of completion, any gaps are identified and fixed, spec is provably complete_

- [ ] 9.3 Final cleanup and polish
  - Review all scripts for consistency (common.sh usage, error handling)
  - Review all documentation for accuracy and completeness
  - Verify all files have proper permissions (scripts are executable)
  - Remove any TODO comments or placeholder code
  - Run final make verify to ensure quality
  - Update .gitignore if needed
  - Commit all work with message: "feat: AI development foundation - workspace, scripts, hooks, CI/CD, docs"
  - Purpose: Polish and finalize all work before delivery
  - _Requirements: All requirements (final quality check)_
  - _Prompt: Role: Senior Developer with expertise in code quality and final delivery | Task: Perform final review and cleanup of entire AI development foundation, ensuring consistency, completeness, and quality | Restrictions: Must not break existing functionality, verify all scripts work, check all documentation is accurate, ensure commit message follows conventions | Success: All code is consistent and polished, documentation is complete and accurate, all scripts have correct permissions, git history is clean, final commit passes all quality checks_
