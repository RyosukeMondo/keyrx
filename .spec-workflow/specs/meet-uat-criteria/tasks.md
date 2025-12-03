# Tasks Document

## UAT Core Infrastructure

- [x] 1. Create UAT module structure
  - Files: core/src/uat/mod.rs (new)
  - Create module with public exports for all UAT components
  - _Leverage: existing module patterns in core/src/_
  - _Requirements: All_
  - _Prompt: Role: Rust systems engineer | Task: Create core/src/uat/mod.rs with module declarations and public exports for: runner, golden, gates, coverage, perf, fuzz, report. Add mod uat to core/src/lib.rs | Restrictions: ≤30 lines; follow existing module patterns; no implementation yet | Success: `cargo check` passes with empty submodules._

- [x] 2. Implement UAT test discovery
  - Files: core/src/uat/runner.rs (new)
  - Implement UatTest struct and discovery of uat_* functions
  - _Leverage: core/src/test_harness/ discovery patterns_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Role: Rust developer specializing in test frameworks | Task: Create runner.rs with UatTest struct (name, file, category, priority, requirements, latency_threshold) and discover() function that scans tests/uat/ for uat_* Rhai functions | Restrictions: ≤150 lines; reuse test_harness patterns; return Vec<UatTest> | Success: Discovers uat_* functions with parsed metadata._

- [x] 3. Implement UAT metadata parsing
  - Files: core/src/uat/runner.rs (extend)
  - Parse @category, @priority, @requirement, @latency from Rhai comments
  - _Leverage: Rhai comment parsing_
  - _Requirements: 1.5, 1.6_
  - _Prompt: Role: Parser developer | Task: Extend runner.rs with parse_metadata() that extracts @category, @priority (P0/P1/P2), @requirement (comma-separated IDs), @latency (microseconds) from Rhai function comments | Restrictions: ≤100 lines; regex-based parsing; handle missing metadata gracefully | Success: All metadata types parsed correctly from test comments._

- [x] 4. Implement UAT test execution
  - Files: core/src/uat/runner.rs (extend)
  - Add UatRunner struct with run() and run_fail_fast() methods
  - _Leverage: core/src/test_harness/ execution engine_
  - _Requirements: 3.1, 3.7_
  - _Prompt: Role: Test framework engineer | Task: Extend runner.rs with UatRunner struct and run(filter: UatFilter) method that executes discovered tests, collecting UatResults (total, passed, failed, skipped, duration, test results) | Restrictions: ≤200 lines; reuse test_harness execution; support fail-fast mode | Success: `keyrx uat` runs all uat_* tests and reports results._

- [x] 5. Implement UAT filter and categorization
  - Files: core/src/uat/runner.rs (extend)
  - Add UatFilter with category, priority, and pattern filtering
  - _Leverage: existing filter patterns_
  - _Requirements: 3.2, 3.3_
  - _Prompt: Role: Rust developer | Task: Extend runner.rs with UatFilter struct supporting categories (Vec<String>), priorities (Vec<Priority>), pattern (Option<String>). Implement filter matching in run() | Restrictions: ≤80 lines; AND logic for multiple filters | Success: `keyrx uat --category core --priority P0` filters correctly._

## Quality Gates

- [x] 6. Implement quality gate configuration
  - Files: core/src/uat/gates.rs (new)
  - Create QualityGate struct and TOML loading
  - _Leverage: existing TOML config patterns_
  - _Requirements: 4.1, 4.7_
  - _Prompt: Role: Configuration engineer | Task: Create gates.rs with QualityGate struct (pass_rate, p0_open, p1_open, max_latency_us, coverage_min) and load() function parsing .keyrx/quality-gates.toml with named gates (default, alpha, beta, rc, ga) | Restrictions: ≤120 lines; use serde for TOML; provide sensible defaults | Success: Loads quality gate config with gate selection._

- [x] 7. Implement quality gate evaluation
  - Files: core/src/uat/gates.rs (extend)
  - Add QualityGateEnforcer with evaluate() method
  - _Leverage: UatResults from runner_
  - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6_
  - _Prompt: Role: Quality assurance engineer | Task: Extend gates.rs with QualityGateEnforcer.evaluate(gate, results) returning GateResult with passed bool and Vec<GateViolation>. Check all criteria: pass_rate, p0_open, p1_open, max_latency_us, coverage_min | Restrictions: ≤150 lines; clear violation messages | Success: Gate evaluation catches all threshold violations._

## Golden Sessions

- [x] 8. Implement golden session data structures
  - Files: core/src/uat/golden.rs (new)
  - Create GoldenSession struct and JSON serialization
  - _Leverage: core/src/session/ format patterns_
  - _Requirements: 2.6_
  - _Prompt: Role: Data modeling engineer | Task: Create golden.rs with GoldenSession struct (name, version, created, metadata, events, expected_outputs) using serde for JSON. Ensure human-readable output | Restrictions: ≤100 lines; JSON format matches design spec | Success: Golden sessions serialize to readable JSON._

- [x] 9. Implement golden session recording
  - Files: core/src/uat/golden.rs (extend)
  - Add GoldenSessionManager.record() method
  - _Leverage: core/src/session/ recording logic_
  - _Requirements: 2.1_
  - _Prompt: Role: Session recording engineer | Task: Extend golden.rs with GoldenSessionManager.record(name, script) that executes script, captures events and outputs, saves to tests/golden/<name>.krx as JSON | Restrictions: ≤120 lines; reuse session recording; validate name format | Success: `keyrx record-golden basic --script test.rhai` creates golden session._

- [x] 10. Implement golden session verification
  - Files: core/src/uat/golden.rs (extend)
  - Add verify() method with semantic comparison
  - _Leverage: session replay logic_
  - _Requirements: 2.2, 2.3, 2.4_
  - _Prompt: Role: Comparison algorithm engineer | Task: Extend golden.rs with GoldenSessionManager.verify(name) that replays session and compares output semantically (ignoring non-deterministic timestamps). Return GoldenVerifyResult with differences | Restrictions: ≤150 lines; detailed diff output; event index tracking | Success: `keyrx verify-golden basic` reports exact differences._

- [ ] 11. Implement golden session update
  - Files: core/src/uat/golden.rs (extend)
  - Add update() method with confirmation
  - _Leverage: record() method_
  - _Requirements: 2.5_
  - _Prompt: Role: CLI developer | Task: Extend golden.rs with GoldenSessionManager.update(name, confirm) that re-records golden session. Require explicit confirmation flag | Restrictions: ≤50 lines; safety confirmation required | Success: `keyrx update-golden basic` requires --confirm flag._

## Coverage Mapping

- [ ] 12. Implement requirements coverage mapping
  - Files: core/src/uat/coverage.rs (new)
  - Create CoverageMapper with requirement linking
  - _Leverage: @requirement metadata from tests_
  - _Requirements: 5.1, 5.2_
  - _Prompt: Role: Traceability engineer | Task: Create coverage.rs with CoverageMapper.build(tests, results) that creates CoverageMap linking requirement IDs to tests based on @requirement metadata | Restrictions: ≤120 lines; handle multiple requirements per test | Success: Coverage map shows all requirement-test links._

- [ ] 13. Implement coverage report generation
  - Files: core/src/uat/coverage.rs (extend)
  - Add report() method with status calculation
  - _Leverage: CoverageMap data_
  - _Requirements: 5.3, 5.4, 5.5, 5.6_
  - _Prompt: Role: Report engineer | Task: Extend coverage.rs with CoverageMapper.report() generating CoverageReport with RequirementCoverage entries (id, linked_tests, status: Verified/AtRisk/Uncovered, last_verified) | Restrictions: ≤100 lines; include all status fields | Success: `keyrx uat --coverage-report` shows requirement matrix._

## Performance UAT

- [ ] 14. Implement performance UAT runner
  - Files: core/src/uat/perf.rs (new)
  - Create PerformanceUat with latency measurement
  - _Leverage: core/benches/latency.rs patterns_
  - _Requirements: 7.1, 7.2, 7.3_
  - _Prompt: Role: Performance engineer | Task: Create perf.rs with PerformanceUat.run() that executes tests with @latency threshold, measuring p50/p95/p99/max latencies and collecting violations | Restrictions: ≤150 lines; reuse benchmark timing; microsecond precision | Success: `keyrx uat --perf` reports latency percentiles._

- [ ] 15. Implement baseline comparison
  - Files: core/src/uat/perf.rs (extend)
  - Add compare_baseline() for regression detection
  - _Leverage: git branch comparison_
  - _Requirements: 7.4, 7.5, 7.6_
  - _Prompt: Role: Regression detection engineer | Task: Extend perf.rs with PerformanceUat.compare_baseline(branch) that compares current latencies against baseline. Fail on >100µs regression | Restrictions: ≤100 lines; git integration for baseline fetch | Success: CI detects latency regressions vs main branch._

## Fuzz Testing

- [ ] 16. Implement fuzz engine
  - Files: core/src/uat/fuzz.rs (new)
  - Create FuzzEngine with random key sequence generation
  - _Leverage: existing proptest/fuzz infrastructure_
  - _Requirements: 8.1, 8.2, 8.4_
  - _Prompt: Role: Fuzz testing engineer | Task: Create fuzz.rs with FuzzEngine.run(duration, count) that generates random key sequences (min 10,000), executes against engine, reports sequences_tested, duration, unique_paths | Restrictions: ≤200 lines; leverage proptest generators | Success: `keyrx uat --fuzz` runs 10,000+ sequences._

- [ ] 17. Implement crash sequence saving
  - Files: core/src/uat/fuzz.rs (extend)
  - Add crash detection and sequence saving
  - _Leverage: session recording for crash sequences_
  - _Requirements: 8.3, 8.5, 8.6_
  - _Prompt: Role: Crash analysis engineer | Task: Extend fuzz.rs to detect crashes, save failing sequences to tests/crashes/<timestamp>.krx as reproducible recordings | Restrictions: ≤80 lines; timestamps in filename; JSON format | Success: Crashes are saved and reproducible via `keyrx replay`._

## Report Generation

- [ ] 18. Implement HTML report generator
  - Files: core/src/uat/report.rs (new)
  - Create ReportGenerator with HTML output
  - _Leverage: template patterns_
  - _Requirements: 9.1, 9.2, 9.3, 9.4_
  - _Prompt: Role: Report generation engineer | Task: Create report.rs with ReportGenerator.generate_html(data) that creates comprehensive HTML report with summary, test results by category, coverage matrix, performance metrics, gate status, trend comparison | Restrictions: ≤250 lines; embedded CSS; no external dependencies | Success: `keyrx uat --report` generates readable HTML._

- [ ] 19. Implement Markdown report generator
  - Files: core/src/uat/report.rs (extend)
  - Add Markdown output for PR comments
  - _Leverage: HTML report data_
  - _Requirements: 9.5, 9.6_
  - _Prompt: Role: Markdown engineer | Task: Extend report.rs with generate_markdown(data) for GitHub PR comment format. Support --report-format md and --report-output path | Restrictions: ≤100 lines; GitHub-flavored markdown | Success: Report suitable for PR comment posting._

- [ ] 20. Implement JSON report output
  - Files: core/src/uat/report.rs (extend)
  - Add JSON output for machine parsing
  - _Leverage: serde serialization_
  - _Requirements: 3.5_
  - _Prompt: Role: API engineer | Task: Extend report.rs with generate_json(data) for machine-readable output. Include all metrics and results | Restrictions: ≤50 lines; serde_json | Success: `keyrx uat --json` outputs parseable JSON._

## CLI Commands

- [ ] 21. Implement keyrx uat command
  - Files: core/src/cli/commands/uat.rs (new)
  - Create UAT command with all flags
  - _Leverage: existing CLI command patterns_
  - _Requirements: 3.1-3.7_
  - _Prompt: Role: CLI engineer | Task: Create uat.rs with keyrx uat command supporting: --category, --priority, --json, --fail-fast, --perf, --fuzz, --coverage-report, --report, --report-format, --report-output, --gate. Return appropriate exit codes (0/1/2/3) | Restrictions: ≤200 lines; clap derive macros; integrate all UAT components | Success: `keyrx uat --help` shows all options._

- [ ] 22. Implement keyrx golden commands
  - Files: core/src/cli/commands/golden.rs (new)
  - Create record-golden, verify-golden, update-golden commands
  - _Leverage: GoldenSessionManager_
  - _Requirements: 2.1, 2.2, 2.5_
  - _Prompt: Role: CLI developer | Task: Create golden.rs with subcommands: record-golden <name> --script <path>, verify-golden <name>, update-golden <name> --confirm. Wire to GoldenSessionManager | Restrictions: ≤150 lines; clear error messages | Success: All golden session commands functional._

- [ ] 23. Implement keyrx regression command
  - Files: core/src/cli/commands/regression.rs (new)
  - Create regression command for all golden sessions
  - _Leverage: GoldenSessionManager.verify()_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Prompt: Role: Regression testing engineer | Task: Create regression.rs with keyrx regression that replays all golden sessions, reports regressions with session name/event index/expected/actual, supports --update-all for intentional changes | Restrictions: ≤120 lines; fail CI on regression | Success: `keyrx regression` verifies all golden sessions._

- [ ] 24. Implement keyrx ci-check command
  - Files: core/src/cli/commands/ci_check.rs (new)
  - Create unified CI command
  - _Leverage: all test runners_
  - _Requirements: 10.1-10.6_
  - _Prompt: Role: CI/CD engineer | Task: Create ci_check.rs with keyrx ci-check that runs: unit tests, integration tests, UAT tests, regression tests, performance tests. Support --gate <name>, --json. Collect all failures before reporting. Exit codes: 1=test fail, 2=gate fail, 3=crash | Restrictions: ≤200 lines; run all checks even on failure | Success: `keyrx ci-check --gate beta` runs full CI suite._

## Integration & Testing

- [ ] 25. Create sample UAT tests
  - Files: tests/uat/core/basic.rhai (new), tests/uat/layers/switching.rhai (new)
  - Create example UAT tests demonstrating all features
  - _Leverage: existing test patterns_
  - _Requirements: 1.1-1.6_
  - _Prompt: Role: QA engineer | Task: Create sample UAT tests in tests/uat/ with uat_* functions, demonstrating @category, @priority, @requirement, @latency metadata. Cover basic typing, layer switching, combo execution | Restrictions: ≤200 lines total; realistic scenarios | Success: Sample tests discoverable and runnable._

- [ ] 26. Create default quality gate config
  - Files: .keyrx/quality-gates.toml (new)
  - Create quality gate configuration with all presets
  - _Leverage: design spec_
  - _Requirements: 4.1-4.7_
  - _Prompt: Role: DevOps engineer | Task: Create .keyrx/quality-gates.toml with gates: default (95% pass, 0 P0, 2 P1, 1000µs, 80% coverage), alpha (relaxed), beta, rc, ga (strict) | Restrictions: ≤40 lines; documented comments | Success: All gate presets defined and loadable._

- [ ] 27. Create initial golden sessions
  - Files: tests/golden/basic_typing.krx (new), tests/golden/layer_switch.krx (new)
  - Record baseline golden sessions
  - _Leverage: keyrx record-golden command_
  - _Requirements: 2.1, 2.6_
  - _Prompt: Role: QA engineer | Task: Record golden sessions for basic typing and layer switching using keyrx record-golden. Ensure JSON format, human-readable | Restrictions: Cover core functionality; ≤5 sessions initially | Success: Golden sessions exist and verify successfully._

- [ ] 28. Add UAT module to Cargo.toml
  - Files: core/Cargo.toml (modify)
  - Add any new dependencies for UAT system
  - _Leverage: existing dependencies_
  - _Requirements: All_
  - _Prompt: Role: Build engineer | Task: Add any required dependencies to core/Cargo.toml for UAT system (likely: none new, reuse existing serde, toml, chrono) | Restrictions: Minimize new dependencies; ≤5 lines added | Success: `cargo build` succeeds with UAT module._

- [ ] 29. Write UAT runner unit tests
  - Files: core/src/uat/runner.rs (extend with #[cfg(test)])
  - Add unit tests for discovery, parsing, filtering
  - _Leverage: existing test patterns_
  - _Requirements: 1.1-1.6, 3.1-3.3_
  - _Prompt: Role: Test engineer | Task: Add unit tests to runner.rs for: metadata parsing, filter matching, result aggregation. Use #[cfg(test)] module | Restrictions: ≤150 lines; test edge cases | Success: `cargo test uat::runner` passes._

- [ ] 30. Write quality gate unit tests
  - Files: core/src/uat/gates.rs (extend with #[cfg(test)])
  - Add unit tests for gate loading and evaluation
  - _Leverage: existing test patterns_
  - _Requirements: 4.1-4.7_
  - _Prompt: Role: Test engineer | Task: Add unit tests to gates.rs for: config loading, threshold evaluation, violation detection. Test all gate criteria | Restrictions: ≤120 lines; cover all thresholds | Success: `cargo test uat::gates` passes._

- [ ] 31. Write integration tests for UAT system
  - Files: core/tests/uat_integration.rs (new)
  - End-to-end tests for UAT workflow
  - _Leverage: existing integration test patterns_
  - _Requirements: All_
  - _Prompt: Role: Integration test engineer | Task: Create uat_integration.rs with tests for: full UAT run, golden session lifecycle, gate evaluation, report generation | Restrictions: ≤200 lines; use temp directories | Success: `cargo test --test uat_integration` passes._

- [ ] 32. Update CI workflow for UAT
  - Files: .github/workflows/ci.yml (modify)
  - Add UAT step to CI pipeline
  - _Leverage: existing CI structure_
  - _Requirements: 10.1-10.6_
  - _Prompt: Role: CI engineer | Task: Add UAT job to ci.yml that runs `keyrx ci-check --gate beta --json`. Fail PR on gate failure. Upload report as artifact | Restrictions: ≤30 lines added; after test job | Success: CI runs UAT checks on every PR._

- [ ] 33. Documentation and README update
  - Files: README.md (modify)
  - Add UAT system documentation
  - _Leverage: existing README structure_
  - _Requirements: All_
  - _Prompt: Role: Technical writer | Task: Add "## UAT System" section to README explaining: keyrx uat command, golden sessions, quality gates, CI integration | Restrictions: ≤50 lines; include command examples | Success: New developers understand UAT workflow._
