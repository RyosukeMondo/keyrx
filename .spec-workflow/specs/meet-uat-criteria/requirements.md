# Requirements Document

## Introduction

This spec implements an **Automated UAT System** that shifts traditional User Acceptance Testing left into the CI/CD pipeline. By leveraging KeyRx's unique capabilities (deterministic input/output, session recording/replay, CLI-first architecture), we can automate 95% of UAT scenarios, reducing manual testing to smoke verification only.

**Philosophy:** If a UAT scenario can be described, it can be automated. Manual UAT should only verify "feel" and edge cases that automation cannot capture.

## Alignment with Product Vision

From product.md:
- **CLI First, GUI Later**: Every feature is CLI-exercisable, enabling full automation
- **Script Testing Framework**: Built-in `simulate_tap()`, `assert_output()` primitives
- **Deterministic Replay**: Session recording enables bug reproduction and regression testing
- **AI Agent Autonomy**: Structured output enables automated verification

From tech.md:
- **Performance Requirements**: <1ms latency, CI fails on >100µs regression
- **Reliability Target**: Zero crashes under 100,000+ random key combinations
- **Fuzz Testing**: Property-based testing with proptest

This spec formalizes these capabilities into a comprehensive automated UAT system.

## Requirements

### Requirement 1: UAT Scenario Definition Language

**User Story:** As a QA engineer, I want to define UAT scenarios in Rhai scripts using the existing test harness, so that acceptance criteria become executable tests.

#### Acceptance Criteria

1. WHEN I write a function prefixed with `uat_`, THEN the test runner SHALL discover and categorize it as a UAT test
2. WHEN I use `simulate_tap()`, `simulate_hold()`, `assert_output()`, THEN these SHALL work identically to `test_` functions
3. WHEN a UAT scenario requires timing verification, THEN `assert_timing(min_ms, max_ms)` SHALL validate event timing
4. WHEN a UAT scenario requires state verification, THEN `assert_layer_active()`, `assert_modifier_active()` SHALL validate engine state
5. WHEN I define a UAT test, THEN I SHALL be able to add metadata: `// @category: core`, `// @priority: P0`
6. WHEN UAT tests run, THEN results SHALL include category and priority in output

### Requirement 2: Golden Session Repository

**User Story:** As a developer, I want a repository of "golden" session recordings that represent expected behavior, so that regressions can be detected by replay comparison.

#### Acceptance Criteria

1. WHEN I run `keyrx record-golden <name> --script <script>`, THEN system SHALL record a session to `tests/golden/<name>.krx`
2. WHEN a golden session exists, THEN `keyrx verify-golden <name>` SHALL replay and compare output
3. WHEN replay output differs from golden, THEN system SHALL report specific differences (event index, expected vs actual)
4. WHEN golden sessions are verified, THEN comparison SHALL use semantic equality (ignoring non-deterministic timestamps)
5. WHEN a golden session needs updating, THEN `keyrx update-golden <name>` SHALL re-record with confirmation
6. WHEN golden sessions are checked into git, THEN they SHALL be human-readable JSON (not binary)

### Requirement 3: UAT Test Runner with Metrics

**User Story:** As a CI system, I want to run all UAT scenarios and receive structured metrics, so that quality gates can be enforced automatically.

#### Acceptance Criteria

1. WHEN I run `keyrx uat`, THEN system SHALL discover and run all `uat_*` functions across all test files
2. WHEN I run `keyrx uat --category core`, THEN only tests with `@category: core` SHALL run
3. WHEN I run `keyrx uat --priority P0,P1`, THEN only P0 and P1 priority tests SHALL run
4. WHEN tests complete, THEN system SHALL output metrics: total, passed, failed, skipped, duration
5. WHEN I use `--json` flag, THEN output SHALL be machine-parseable JSON with full details
6. WHEN tests complete, THEN exit code SHALL be: 0=all pass, 1=error, 2=failures, 3=timeout
7. WHEN `--fail-fast` is specified, THEN execution SHALL stop on first failure

### Requirement 4: Quality Gate Definitions

**User Story:** As a release manager, I want configurable quality gates that define pass/fail criteria, so that releases meet consistent standards.

#### Acceptance Criteria

1. WHEN a quality gate config exists at `.keyrx/quality-gates.toml`, THEN `keyrx uat` SHALL enforce it
2. WHEN gate specifies `pass_rate = 95`, THEN UAT SHALL fail if <95% tests pass
3. WHEN gate specifies `p0_open = 0`, THEN UAT SHALL fail if any P0 tests fail
4. WHEN gate specifies `p1_open = 2`, THEN UAT SHALL fail if >2 P1 tests fail
5. WHEN gate specifies `max_latency_us = 1000`, THEN UAT SHALL fail if any event exceeds 1ms
6. WHEN gate specifies `coverage_min = 80`, THEN UAT SHALL fail if code coverage <80%
7. WHEN multiple gates exist (alpha, beta, rc, ga), THEN `--gate <name>` SHALL select which to apply

### Requirement 5: Coverage Mapping (Requirements → Tests)

**User Story:** As a QA lead, I want to see which requirements are covered by which tests, so that coverage gaps are visible.

#### Acceptance Criteria

1. WHEN a test includes `// @requirement: 1.1`, THEN it SHALL be linked to requirement 1.1
2. WHEN I run `keyrx uat --coverage-report`, THEN system SHALL output requirement coverage matrix
3. WHEN a requirement has no linked tests, THEN report SHALL flag it as "uncovered"
4. WHEN a requirement has failing tests, THEN report SHALL show "at risk"
5. WHEN all linked tests pass, THEN requirement SHALL show "verified"
6. WHEN report is generated, THEN it SHALL include: requirement ID, linked tests, status, last verified date

### Requirement 6: Automated Regression Detection

**User Story:** As a developer, I want automated detection of behavioral regressions via session replay, so that bugs are caught before UAT.

#### Acceptance Criteria

1. WHEN I run `keyrx regression`, THEN system SHALL replay all golden sessions and compare outputs
2. WHEN a regression is detected, THEN system SHALL output: session name, event index, expected output, actual output
3. WHEN regression tests run in CI, THEN failures SHALL block merge
4. WHEN a behavioral change is intentional, THEN `keyrx update-golden --all` SHALL update all affected sessions
5. WHEN regression tests complete, THEN report SHALL include: total sessions, passed, regressed, timing delta

### Requirement 7: Performance UAT (Latency Verification)

**User Story:** As a performance engineer, I want automated verification that latency requirements are met, so that performance regressions are caught.

#### Acceptance Criteria

1. WHEN I run `keyrx uat --perf`, THEN system SHALL run performance-specific UAT tests
2. WHEN a test includes `// @latency: 1000`, THEN it SHALL fail if event processing exceeds 1000µs
3. WHEN latency tests run, THEN system SHALL report: p50, p95, p99, max latencies
4. WHEN latency exceeds threshold, THEN report SHALL include specific events that exceeded
5. WHEN `--benchmark-baseline main` is specified, THEN comparison SHALL be against main branch baseline
6. WHEN regression >100µs is detected, THEN test SHALL fail with detailed comparison

### Requirement 8: Fuzz-Based UAT (Chaos Testing)

**User Story:** As a reliability engineer, I want automated chaos testing with random inputs, so that edge cases and crashes are discovered.

#### Acceptance Criteria

1. WHEN I run `keyrx uat --fuzz`, THEN system SHALL generate random key sequences and verify no crashes
2. WHEN fuzz testing runs, THEN it SHALL generate at least 10,000 random key combinations
3. WHEN a crash is detected, THEN system SHALL save the failing sequence to `tests/crashes/<timestamp>.krx`
4. WHEN fuzz testing completes without crashes, THEN it SHALL report: sequences tested, duration, unique paths explored
5. WHEN `--fuzz-duration 60s` is specified, THEN fuzzing SHALL run for specified duration
6. WHEN a crash sequence is saved, THEN it SHALL be reproducible via `keyrx replay`

### Requirement 9: UAT Report Generation

**User Story:** As a stakeholder, I want a comprehensive UAT report, so that release readiness is documented.

#### Acceptance Criteria

1. WHEN I run `keyrx uat --report`, THEN system SHALL generate HTML/Markdown report
2. WHEN report is generated, THEN it SHALL include: summary, test results by category, coverage matrix, performance metrics
3. WHEN report is generated, THEN it SHALL include: quality gate status (pass/fail), blocking issues
4. WHEN report is generated, THEN it SHALL include: trend comparison with previous run (if available)
5. WHEN `--report-format md` is specified, THEN output SHALL be Markdown suitable for GitHub PR comment
6. WHEN `--report-output <path>` is specified, THEN report SHALL be written to specified path

### Requirement 10: CI Integration Command

**User Story:** As a DevOps engineer, I want a single command that runs all automated UAT checks, so that CI pipelines are simple.

#### Acceptance Criteria

1. WHEN I run `keyrx ci-check`, THEN system SHALL run: unit tests, integration tests, UAT tests, regression tests, performance tests
2. WHEN I run `keyrx ci-check --gate beta`, THEN system SHALL apply beta quality gate criteria
3. WHEN any check fails, THEN system SHALL continue running remaining checks (collect all failures)
4. WHEN all checks complete, THEN system SHALL output consolidated report
5. WHEN `--json` is specified, THEN output SHALL be machine-parseable for CI systems
6. WHEN checks fail, THEN exit code SHALL indicate failure type (1=test fail, 2=gate fail, 3=crash)

## Non-Functional Requirements

### Code Architecture and Modularity

- **Test Organization**: UAT tests organized by feature area in `tests/uat/` directory
- **Golden Sessions**: Stored in `tests/golden/` with descriptive names
- **Config Files**: Quality gates in `.keyrx/quality-gates.toml`
- **Reports**: Generated to `target/uat-report/`

### Performance

- **UAT Suite Runtime**: Full suite SHALL complete in <5 minutes
- **Individual Test Timeout**: Default 10 seconds per test
- **Fuzz Testing**: 10,000 sequences in <60 seconds

### Reliability

- **Deterministic Results**: Same inputs SHALL produce same results across runs
- **Isolation**: Tests SHALL not affect each other's state
- **Reproducibility**: All failures SHALL be reproducible via saved session

### Usability

- **Progressive Disclosure**: `keyrx uat` runs standard suite, flags enable advanced features
- **Clear Output**: Pass/fail status visible at a glance
- **Actionable Errors**: Failures include reproduction steps
