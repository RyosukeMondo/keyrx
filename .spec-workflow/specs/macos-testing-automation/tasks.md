# Tasks Document

## Layer 1: Enhanced Mock Tests

- [x] 1.1 Create CGEvent conversion round-trip tests
  - File: keyrx_daemon/tests/macos_mock_tests.rs (new file)
  - Implement test functions for all 140+ keycode mappings
  - Test CGKeyCode → KeyCode → CGKeyCode with zero data loss
  - Test edge cases: unknown keycodes, reserved values
  - Purpose: Validate keycode conversion correctness without Accessibility permissions
  - _Leverage: keyrx_daemon/src/platform/macos/keycode_map.rs (conversion functions), existing macos_integration.rs test patterns_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer specializing in property-based testing and data validation | Task: Create comprehensive CGEvent conversion tests covering all 140+ keycode mappings in keyrx_daemon/tests/macos_mock_tests.rs, validating bidirectional conversion (CGKeyCode → KeyCode → CGKeyCode) with zero data loss per requirements 1.1, 1.2, 1.3, leveraging existing conversion functions from keycode_map.rs and test patterns from macos_integration.rs | Restrictions: Do not require Accessibility permissions (must run in CI), test only conversion logic not event capture, use #[cfg(target_os = "macos")] guard, follow 500-line file limit, do not duplicate existing macos_integration.rs tests | Success: All 140+ keycodes tested with round-trip validation, edge cases covered (unknown codes, reserved values), tests pass in CI without permissions, achieve ≥90% coverage of keycode_map.rs. After completion: (1) Edit tasks.md to mark this task [-] as in-progress before starting, (2) After implementation, use log-implementation tool with detailed artifacts field (functions tested, test coverage achieved, edge cases covered), (3) Edit tasks.md to mark [x] as completed_

- [x] 1.2 Add platform initialization error path tests
  - File: keyrx_daemon/tests/macos_mock_tests.rs (continue from 1.1)
  - Test MacosPlatform initialization without Accessibility permission
  - Verify graceful failure with descriptive error message
  - Test permission checking returns false in mock environment
  - Purpose: Validate error handling when permissions denied
  - _Leverage: keyrx_daemon/src/platform/macos/permissions.rs (check_accessibility_permission), keyrx_daemon/src/platform/macos/mod.rs (MacosPlatform::initialize)_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer specializing in error path testing and fault injection | Task: Add platform initialization tests to macos_mock_tests.rs validating graceful failure when Accessibility permission is denied per requirement 1.3, testing MacosPlatform::initialize() error paths and permission checking logic from permissions.rs | Restrictions: Must not require actual Accessibility permissions, test only error handling logic not real permission API, use descriptive assertion messages, ensure tests are deterministic | Success: Permission denied scenario returns appropriate error, error message includes setup instructions, tests pass without actual permissions, error handling is graceful (no panics). After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with error scenarios tested and error messages validated, (3) Edit tasks.md to mark [x] as completed_

- [x] 1.3 Create device discovery mock tests
  - File: keyrx_daemon/tests/macos_mock_tests.rs (continue from 1.2)
  - Test IOKit device enumeration with mock responses
  - Edge cases: 0 devices, 10 devices, invalid serial numbers, disconnected devices
  - Test USB and Bluetooth keyboard identification
  - Purpose: Validate device discovery logic without physical hardware
  - _Leverage: keyrx_daemon/src/platform/macos/device_discovery.rs (enumerate_devices), MockInput patterns from macos_integration.rs_
  - _Requirements: 1.3, 1.4, 1.5_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Platform Test Engineer specializing in device enumeration and IOKit testing | Task: Implement device discovery mock tests in macos_mock_tests.rs covering edge cases (0 devices, multiple devices, invalid serials, USB/Bluetooth) per requirements 1.3, 1.4, 1.5, testing device_discovery.rs enumeration logic with mock IOKit responses | Restrictions: Do not require real USB/Bluetooth devices, mock all IOKit calls, test only enumeration logic not actual hardware interaction, ensure tests are isolated and repeatable | Success: All edge cases tested (0, 1, 10 devices), serial number extraction validated, USB vs Bluetooth discrimination works, tests are deterministic and fast (<1s). After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with edge cases covered and test utilities created, (3) Edit tasks.md to mark [x] as completed_

- [x] 1.4 Verify mock test coverage
  - Files: Run coverage analysis on platform/macos/ modules
  - Use cargo tarpaulin or similar to measure line coverage
  - Ensure ≥90% coverage of platform/macos/*.rs files
  - Identify and add tests for uncovered code paths
  - Purpose: Ensure comprehensive test coverage without permissions
  - _Leverage: scripts/verify-coverage.sh (if exists), cargo tarpaulin command_
  - _Requirements: 1.6_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in test coverage analysis and gap identification | Task: Measure and verify test coverage of platform/macos/ modules achieves ≥90% line coverage per requirement 1.6, using cargo tarpaulin or similar tools, identify uncovered code paths and add tests to close gaps | Restrictions: Focus only on platform/macos/ directory not entire codebase, exclude FFI bindings that cannot be tested without permissions, do not lower coverage standards to pass | Success: Coverage report generated showing ≥90% for platform/macos/*.rs, all significant code paths tested, uncovered code is FFI bindings or explicitly excluded, coverage integrated into CI. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with coverage percentages and gaps identified, (3) Edit tasks.md to mark [x] as completed_

## Layer 2: E2E Tests with Permission Checks

- [x] 2.1 Create MacosE2EHarness infrastructure
  - File: keyrx_daemon/tests/e2e_macos_harness.rs (new file)
  - Implement harness for spawning daemon subprocess
  - Add config compilation integration (reuse from e2e_harness.rs)
  - Implement graceful shutdown (SIGTERM → wait 5s → SIGKILL)
  - Add permission checking with informative skip messages
  - Purpose: Provide E2E test orchestration infrastructure
  - _Leverage: keyrx_daemon/tests/e2e_harness.rs (E2EError, E2EConfig, harness patterns), keyrx_daemon/src/platform/macos/permissions.rs (check_accessibility_permission), keyrx_compiler::serialize_
  - _Requirements: 2.1, 2.2, 2.3, 2.7_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Test Engineer specializing in process management and E2E test infrastructure | Task: Create MacosE2EHarness in e2e_macos_harness.rs for spawning daemon subprocesses and managing test lifecycle per requirements 2.1, 2.2, 2.3, 2.7, reusing E2EError/E2EConfig patterns from e2e_harness.rs, integrating config compilation and implementing graceful shutdown with 5s timeout | Restrictions: Must use #[cfg(target_os = "macos")] guard, do not create virtual devices (macOS has no uinput), ensure daemon processes never orphaned, implement Drop trait for cleanup, follow existing E2E harness patterns | Success: Harness can spawn daemon with compiled config, graceful shutdown works (SIGTERM → SIGKILL), temp files cleaned up on Drop, permission checks integrated, matches e2e_harness.rs patterns. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with classes created (MacosE2EHarness, setup/teardown methods) and process management verified, (3) Edit tasks.md to mark [x] as completed_

- [x] 2.2 Implement basic E2E tests
  - File: keyrx_daemon/tests/e2e_macos_basic.rs (new file)
  - Create test_macos_e2e_basic_remap with A → B remapping
  - Add permission check at test level (skip if no permission)
  - Verify daemon starts without errors
  - Verify daemon loads config successfully
  - Verify daemon shuts down gracefully
  - Purpose: Validate basic daemon lifecycle and config loading
  - _Leverage: keyrx_daemon/tests/e2e_macos_harness.rs (MacosE2EHarness), keyrx_daemon/src/platform/macos/permissions.rs (check_accessibility_permission), e2e_windows_basic.rs patterns_
  - _Requirements: 2.1, 2.2, 2.4, 2.7_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Engineer specializing in integration testing and subprocess validation | Task: Create basic E2E tests in e2e_macos_basic.rs validating daemon startup, config loading, and graceful shutdown per requirements 2.1, 2.2, 2.4, 2.7, using MacosE2EHarness and implementing permission checks that skip tests when Accessibility permission absent | Restrictions: Must auto-skip in CI (no permission failures), print informative messages when skipping, use #[serial] attribute for test isolation, test only daemon lifecycle not full input/output pipeline, ensure tests are idempotent | Success: Daemon spawns successfully, config loads without errors, graceful shutdown works, tests skip gracefully without permissions, tests pass with permissions enabled, harness cleanup verified. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with test functions created and lifecycle stages validated, (3) Edit tasks.md to mark [x] as completed_

- [x] 2.3 Create multi-device E2E tests
  - File: keyrx_daemon/tests/e2e_macos_multidevice.rs (new file)
  - Test device-specific configuration loading
  - Test serial number-based device identification
  - Test multiple keyboards with different mappings (if hardware available)
  - Purpose: Validate device discrimination with real hardware
  - _Leverage: keyrx_daemon/tests/e2e_macos_harness.rs (MacosE2EHarness), keyrx_daemon/tests/e2e_linux_multidevice.rs patterns, keyrx_daemon/src/platform/macos/device_discovery.rs_
  - _Requirements: 2.2, 2.5_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Multi-device Test Engineer specializing in device discrimination and hardware testing | Task: Create multi-device E2E tests in e2e_macos_multidevice.rs validating device-specific configurations and serial number identification per requirements 2.2, 2.5, using MacosE2EHarness and following e2e_linux_multidevice.rs patterns | Restrictions: Must gracefully handle single-device scenario (most developers), use #[serial] attribute, skip if no permission, test only device identification not full remapping, document multi-device setup requirements | Success: Device enumeration works correctly, serial number extraction validated, device-specific configs load for correct devices, tests skip gracefully on single-device systems, documentation includes multi-device setup guide. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with device scenarios tested and configuration patterns validated, (3) Edit tasks.md to mark [x] as completed_

- [x] 2.4 Add tap-hold timing E2E test (if feasible)
  - File: keyrx_daemon/tests/e2e_macos_basic.rs (continue from 2.2)
  - Test timing-sensitive tap-hold behavior with real delays
  - Verify QMK-style permissive hold timing
  - Document timing constraints for manual verification
  - Purpose: Validate timing accuracy of DFA state machine
  - _Leverage: keyrx_daemon/tests/e2e_macos_harness.rs (MacosE2EHarness), keyrx_core timing constants_
  - _Requirements: 2.6_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Timing Test Engineer specializing in real-time behavior validation and state machine testing | Task: Add tap-hold timing E2E test to e2e_macos_basic.rs validating DFA timing accuracy and QMK-style permissive hold per requirement 2.6, using MacosE2EHarness with real time delays and documenting timing verification requirements | Restrictions: Must account for system timer accuracy limitations, document expected vs actual timing, consider test flakiness due to system load, may require manual verification step | Success: Timing test demonstrates <1ms processing latency, permissive hold behavior documented, test includes timing constraints documentation, flakiness minimized or test marked as manual-only. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with timing measurements and accuracy validation, (3) Edit tasks.md to mark [x] as completed_

- [x] 2.5 Verify E2E tests auto-skip in CI
  - Files: Test CI behavior of E2E tests
  - Run tests in GitHub Actions macOS runner (no Accessibility permission)
  - Verify exit code 0 (success) when tests skip
  - Verify skip messages appear in CI logs
  - Purpose: Ensure CI reliability with E2E test skipping
  - _Leverage: .github/workflows/ci.yml (existing CI config), GitHub Actions logs_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD Engineer specializing in GitHub Actions and test automation | Task: Verify E2E tests auto-skip gracefully in GitHub Actions CI environment without Accessibility permissions per requirements 2.1, 2.2, checking exit codes and log messages to ensure CI reliability | Restrictions: Do not attempt to grant Accessibility permissions in CI, must verify skip behavior not disable tests, ensure logs are clear about why tests skipped, maintain CI performance (no long timeouts) | Success: E2E tests skip with exit code 0 in CI, skip messages visible in logs, CI job completes successfully, no hanging or timeouts, documentation updated with CI behavior. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with CI behavior validated and exit codes verified, (3) Edit tasks.md to mark [x] as completed_

## Layer 3: Automated Test Runner Script

- [x] 3.1 Create permission checker script
  - File: scripts/check_macos_permission.sh (new file)
  - Build minimal test binary that checks Accessibility permission
  - Return exit code 0 if granted, 1 if denied
  - Keep script simple and fast (<1 second execution)
  - Purpose: Provide non-blocking permission check for test runner
  - _Leverage: keyrx_daemon/src/platform/macos/permissions.rs (check_accessibility_permission)_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Shell Script Engineer specializing in build automation and process management | Task: Create permission checker script scripts/check_macos_permission.sh that builds minimal binary and checks Accessibility permission per requirement 3.2, returning appropriate exit codes and executing in <1 second | Restrictions: Must be POSIX-compliant bash, no dependencies beyond cargo/rustc, suppress build output (redirect to /dev/null), handle missing toolchain gracefully, do not block or hang | Success: Script returns 0 if permission granted, 1 if denied, executes in <1s, build errors handled gracefully, script is idempotent. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with script behavior and exit codes documented, (3) Edit tasks.md to mark [x] as completed_

- [x] 3.2 Create main test runner script
  - File: scripts/test_macos_full.sh (new file)
  - Orchestrate full test suite: mock tests → permission check → E2E tests → benchmarks
  - Print clear progress indicators (✅, ⚠️, ❌) for each phase
  - Handle Ctrl+C gracefully (trap and cleanup)
  - Exit with appropriate code (0 = success, 1 = test failure)
  - Purpose: Provide single entry point for complete macOS test suite
  - _Leverage: scripts/check_macos_permission.sh (permission checking), existing cargo test/bench commands, scripts/test.sh patterns (if exists)_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer specializing in test automation and shell scripting | Task: Create comprehensive test runner script scripts/test_macos_full.sh orchestrating mock tests, permission checks, E2E tests, and benchmarks per requirements 3.1, 3.2, 3.3, 3.4, 3.6, using check_macos_permission.sh and providing clear progress reporting | Restrictions: Must be POSIX-compliant bash, use set -euo pipefail for safety, handle interruption (Ctrl+C) gracefully, provide clear visual feedback, exit codes must be meaningful (0=success, 1=failure), mock test failure must stop execution | Success: Script runs all test phases in order, permission warning is non-blocking, progress indicators are clear, Ctrl+C cleanup works, exit codes are correct, script documented with usage examples. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with script structure and phase orchestration documented, (3) Edit tasks.md to mark [x] as completed_

- [x] 3.3 Add interactive manual test prompt
  - File: scripts/test_macos_full.sh (continue from 3.2)
  - Detect interactive terminal with [ -t 0 ] check
  - Prompt user to run manual tests with y/N option
  - Skip prompt in non-interactive mode (CI)
  - Purpose: Provide optional manual test execution
  - _Leverage: bash read command, terminal detection_
  - _Requirements: 3.5_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: UX Engineer specializing in command-line interfaces and user interaction | Task: Add interactive manual test prompt to test_macos_full.sh detecting terminal mode and prompting user per requirement 3.5, ensuring graceful behavior in non-interactive CI environments | Restrictions: Must not block in CI (use [ -t 0 ] check), prompt must be clear and concise, default to No (safety), handle EOF gracefully, respect user's choice immediately | Success: Prompt appears only in interactive terminals, CI skips prompt automatically, y/N options work correctly, user experience is smooth, manual test documentation included. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with interactive behavior validated, (3) Edit tasks.md to mark [x] as completed_

- [x] 3.4 Add test summary reporting
  - File: scripts/test_macos_full.sh (continue from 3.3)
  - Print summary after all tests complete
  - Include: tests run, coverage %, benchmark results, manual test recommendations
  - Use clear formatting with visual indicators
  - Purpose: Provide actionable test results summary
  - _Leverage: cargo test output parsing, benchmark result formatting_
  - _Requirements: 3.7_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Test Reporting Engineer specializing in output formatting and result aggregation | Task: Add comprehensive test summary reporting to test_macos_full.sh showing tests run, coverage, benchmarks, and recommendations per requirement 3.7, using clear formatting and visual indicators | Restrictions: Must parse cargo output correctly, handle missing data gracefully (e.g., no benchmarks), keep summary concise (<20 lines), use color codes only if terminal supports them (tput check) | Success: Summary is clear and actionable, includes all required metrics, formatting is professional, works in both color and non-color terminals, summary helps developer understand test status. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with summary format and metrics documented, (3) Edit tasks.md to mark [x] as completed_

- [x] 3.5 Test script in both interactive and CI modes
  - Files: scripts/test_macos_full.sh (testing)
  - Test in interactive terminal (manual run)
  - Test in CI mode (GitHub Actions)
  - Verify exit codes in both scenarios
  - Verify output formatting and progress indicators
  - Purpose: Ensure script works in all environments
  - _Leverage: GitHub Actions macOS runner, local terminal_
  - _Requirements: 3.4_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in environment testing and cross-platform validation | Task: Test test_macos_full.sh script in both interactive and CI environments per requirement 3.4, validating behavior, exit codes, and output formatting in each mode | Restrictions: Must test both with and without Accessibility permissions, verify non-interactive mode detection works, check exit code propagation, ensure no hanging or timeouts in CI | Success: Script works correctly in interactive mode (prompts appear), script works in CI mode (prompts skipped), exit codes correct in all scenarios, output readable in both modes, documentation includes both usage examples. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with test scenarios and validations performed, (3) Edit tasks.md to mark [x] as completed_

## Documentation and Integration

- [x] 4.1 Update CI workflow for macOS testing
  - File: .github/workflows/ci.yml (modify existing)
  - Add macOS test job using test_macos_full.sh
  - Verify job runs on macos-latest runner
  - Ensure test failure fails the build
  - Purpose: Integrate macOS tests into CI pipeline
  - _Leverage: existing Linux/Windows test jobs in ci.yml_
  - _Requirements: 3.1, 4.1_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD Engineer specializing in GitHub Actions and workflow optimization | Task: Update .github/workflows/ci.yml to add macOS test job using test_macos_full.sh per requirements 3.1, 4.1, following existing Linux/Windows test job patterns | Restrictions: Must run on macos-latest runner, must not cache Accessibility permissions (none available), keep job fast (<10 min), propagate test failures correctly, add job to required checks | Success: macOS test job runs in CI, E2E tests skip gracefully, mock tests execute successfully, job failure fails build, job performance is acceptable (<10 min). After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with CI integration details, (3) Edit tasks.md to mark [x] as completed_

- [ ] 4.2 Create macOS testing documentation
  - File: docs/development/MACOS_TESTING_GUIDE.md (new file)
  - Document three-layer testing strategy
  - Explain Accessibility permission requirements
  - Provide developer setup instructions
  - Include troubleshooting section
  - Purpose: Help developers understand and use macOS test suite
  - _Leverage: existing docs structure, permissions.rs error messages_
  - _Requirements: All requirements (documentation)_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer specializing in developer documentation and testing guides | Task: Create comprehensive macOS testing guide in docs/development/MACOS_TESTING_GUIDE.md documenting three-layer strategy, permission requirements, setup instructions, and troubleshooting, supporting all requirements | Restrictions: Keep documentation concise (<500 lines), use clear examples, include copy-pasteable commands, update .claude/CLAUDE.md with reference to new guide, follow existing docs style | Success: Documentation is clear and actionable, covers all three layers, includes troubleshooting for common issues, has examples for each test type, developers can follow guide without external help. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with documentation structure and sections created, (3) Edit tasks.md to mark [x] as completed_

- [ ] 4.3 Verify test coverage parity with Linux/Windows
  - Files: Compare test coverage across platforms
  - Run coverage reports for Linux, Windows, macOS
  - Verify macOS has ≥ equivalent coverage
  - Document any intentional differences (e.g., no VirtualKeyboard)
  - Purpose: Ensure cross-platform test quality parity
  - _Leverage: cargo tarpaulin, coverage comparison scripts_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Platform Quality Engineer specializing in cross-platform testing and coverage analysis | Task: Compare and verify test coverage parity between macOS, Linux, and Windows per requirements 4.1, 4.2, 4.3, documenting coverage metrics and any intentional platform differences | Restrictions: Must account for platform-specific code (e.g., Linux uinput vs macOS no equivalent), compare only equivalent functionality, document coverage differences with justification, maintain ≥80% overall coverage | Success: Coverage comparison report generated, macOS meets ≥80% threshold, platform differences documented and justified, no significant coverage gaps identified, report integrated into CI. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with coverage comparison data and platform differences documented, (3) Edit tasks.md to mark [x] as completed_

## Final Validation

- [ ] 5.1 Run full test suite locally with permissions
  - Action: Execute test_macos_full.sh on developer machine with Accessibility permission
  - Verify all test layers pass: mock tests, E2E tests, benchmarks
  - Check for flaky tests (run 3 times)
  - Verify cleanup (no orphaned processes, temp files removed)
  - Purpose: Validate complete test suite end-to-end
  - _Leverage: scripts/test_macos_full.sh_
  - _Requirements: All_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: QA Lead specializing in end-to-end validation and quality assurance | Task: Execute complete test suite locally with Accessibility permissions enabled, running test_macos_full.sh multiple times to detect flakiness and validate cleanup behavior per all requirements | Restrictions: Must test on real macOS hardware (not VM), verify all three layers execute, check process cleanup with ps/lsof, run at least 3 iterations, document any issues found | Success: All test layers pass consistently (3/3 runs), no flaky tests detected, cleanup verified (no orphaned processes or temp files), benchmarks meet <1ms requirement, summary report is accurate. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with validation results and any issues discovered, (3) Edit tasks.md to mark [x] as completed_

- [ ] 5.2 Run test suite in CI without permissions
  - Action: Trigger GitHub Actions CI workflow for macOS
  - Verify E2E tests skip gracefully (not fail)
  - Verify mock tests pass
  - Verify benchmarks execute
  - Verify CI job completes with exit code 0
  - Purpose: Validate CI behavior without Accessibility permissions
  - _Leverage: GitHub Actions macOS runner_
  - _Requirements: 2.1, 2.2, 3.1_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD Validation Engineer specializing in automated testing and GitHub Actions | Task: Trigger and validate GitHub Actions CI workflow for macOS per requirements 2.1, 2.2, 3.1, ensuring E2E tests skip gracefully, mock tests pass, and job completes successfully without Accessibility permissions | Restrictions: Do not modify permission settings in CI, verify exit code 0 (success), check logs for skip messages, ensure job performance acceptable, validate test result reporting | Success: CI job passes with exit code 0, E2E tests skip with informative messages, mock tests execute and pass, benchmarks complete, job logs are clear, no timeouts or hangs. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with CI validation results and logs analyzed, (3) Edit tasks.md to mark [x] as completed_

- [ ] 5.3 Update project documentation
  - Files: .claude/CLAUDE.md, README.md (if applicable)
  - Add reference to MACOS_TESTING_GUIDE.md
  - Document test_macos_full.sh usage in quick start
  - Update testing section with macOS coverage information
  - Purpose: Make macOS testing discoverable to developers
  - _Leverage: existing documentation structure_
  - _Requirements: Documentation completeness_
  - _Prompt: Implement the task for spec macos-testing-automation, first run mcp__spec-workflow__spec-workflow-guide to get the workflow guide then implement the task: Role: Documentation Maintainer specializing in developer experience and documentation structure | Task: Update project documentation (.claude/CLAUDE.md, README.md) to reference macOS testing guide and document test_macos_full.sh usage, making testing discoverable | Restrictions: Keep updates concise and consistent with existing docs, add to appropriate sections (testing, quick start), maintain documentation structure, do not duplicate content unnecessarily | Success: CLAUDE.md references MACOS_TESTING_GUIDE.md, test_macos_full.sh documented in quick start, testing section updated with macOS information, documentation is consistent and easy to navigate. After completion: (1) Edit tasks.md to mark this task [-] as in-progress, (2) Use log-implementation tool with documentation updates and locations modified, (3) Edit tasks.md to mark [x] as completed_
