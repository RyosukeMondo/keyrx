# Tasks Document

## Phase 1: Investigation and Root Cause Analysis

- [x] 1.1 Reproduce test failures locally
  - Command: `cargo test test_c_api_null_label_clears test_macro_generates_doc -- --exact`
  - Verify both tests fail with expected errors
  - Capture exact error messages
  - Purpose: Confirm failures and understand errors
  - _Leverage: cargo test_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA engineer

Task: Reproduce test failures following requirements 1.1 and 2.1. Run `cargo test test_c_api_null_label_clears -- --exact` and `cargo test test_macro_generates_doc -- --exact`. Capture full output including error messages, stack traces, and any debug output. Save to .spec-workflow/specs/fix-failing-tests/failure_reproduction.txt. Verify errors match: (1) "assertion failed: msg.starts_with("ok:")" for FFI test, (2) "Documentation should be registered" for doc test.

Restrictions: Just reproduce and document. Do not attempt fixes yet. Ensure failures are consistent across multiple runs.

Success: Both test failures reproduced and documented. Exact error messages captured. Failures are consistent and match expectations. Ready for root cause analysis.

After completing:
1. Mark [-] before starting
2. Run tests and capture output
3. Use log-implementation to record findings
4. Mark [x] when complete_

- [x] 1.2 Analyze FFI device registry test failure
  - File: core/src/ffi/domains/device_registry.rs:571
  - Read test code and understand what it's testing
  - Read implementation of set_device_label FFI function
  - Check actual response format returned
  - Purpose: Identify root cause of FFI test failure
  - _Leverage: Test code, implementation code_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust debugging specialist

Task: Analyze FFI test failure following requirement 1.1. Read `test_c_api_null_label_clears` test in core/src/ffi/domains/device_registry.rs around line 571. Understand: (1) What test is validating, (2) What response format it expects, (3) What device_key and null label should do. Then read FFI implementation function that's called. Add temporary debug println! to see actual response. Run test again to capture actual vs expected. Document findings in .spec-workflow/specs/fix-failing-tests/ffi_test_analysis.md with root cause conclusion.

Restrictions: Analysis only. Add debug output temporarily but don't fix yet. Identify one of: format changed, null handling wrong, error returned incorrectly, test expectation wrong.

Success: Root cause identified and documented. Understand why test fails. Know what needs to change (test expectation vs implementation). Clear fix strategy documented.

After completing:
1. Mark [-], analyze, document findings, log analysis, mark [x]_

- [x] 1.3 Analyze scripting documentation test failure
  - File: core/src/scripting/docs/test_example.rs:46
  - Read test code and understand what it's testing
  - Check doc registry initialization
  - Review similar passing doc tests for patterns
  - Purpose: Identify root cause of doc test failure
  - _Leverage: Test code, doc registry code_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust debugging specialist

Task: Analyze doc test failure following requirement 2.1. Read `test_macro_generates_doc` test in core/src/scripting/docs/test_example.rs around line 46. Understand: (1) What `#[rhai_doc]` macro should do, (2) What registry should contain, (3) What test is checking. Find other passing doc tests and compare setup. Check if initialize() is called. Check if macro is working. Add debug output to see registry state. Document findings in .spec-workflow/specs/fix-failing-tests/doc_test_analysis.md with root cause conclusion.

Restrictions: Analysis only. Identify one of: registry not initialized, macro not registering, wrong function name, test isolation issue.

Success: Root cause identified. Understand why test fails. Know what needs to change. Clear fix strategy documented.

After completing:
1. Mark [-], analyze, document, log, mark [x]_

## Phase 2: Implement Fixes

- [x] 2.1 Fix FFI device registry test
  - File: core/src/ffi/domains/device_registry.rs (test or implementation)
  - Apply fix based on root cause from 1.2
  - Update test expectation or fix implementation as needed
  - Remove temporary debug code
  - Purpose: Make test pass
  - _Leverage: Root cause analysis from 1.2_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust developer with FFI expertise

Task: Fix FFI test following requirements 1.1-1.5. Based on root cause from 1.2, apply appropriate fix: (A) If format changed - update test assertion to expect new format, (B) If null handling wrong - fix implementation to treat null as clear, (C) If error returned - fix error handling to return success on null, (D) If test wrong - fix test logic. Make minimal changes. Add comment explaining fix. Remove debug code.

Restrictions: Minimal fix only. Don't refactor unrelated code. Preserve test intent. Ensure null label clears device label correctly. Don't break other device_registry tests.

Success: test_c_api_null_label_clears passes consistently. Null label correctly clears device label. Other device_registry tests still pass. Fix is minimal and clear.

After completing:
1. Mark [-], implement fix, test locally, use log-implementation with detailed artifacts, mark [x]_

- [x] 2.2 Fix scripting documentation test
  - File: core/src/scripting/docs/test_example.rs (test or macro/registry)
  - Apply fix based on root cause from 1.3
  - Add initialization or fix registration as needed
  - Ensure macro generates documentation correctly
  - Purpose: Make test pass
  - _Leverage: Root cause analysis from 1.3_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust developer with macro expertise

Task: Fix doc test following requirements 2.1-2.5. Based on root cause from 1.3, apply appropriate fix: (A) If not initialized - add registry initialization in test setup, (B) If macro not working - fix macro or test annotation, (C) If wrong name - update test to use correct function name, (D) If isolation - set up proper test environment. Make minimal changes. Add comment explaining fix.

Restrictions: Minimal fix. Don't refactor doc system. Preserve test intent. Ensure macro registration works. Don't break other doc tests.

Success: test_macro_generates_doc passes consistently. Documentation registered correctly. Other doc tests still pass. Fix is minimal and clear.

After completing:
1. Mark [-], implement fix, test locally, use log-implementation with artifacts, mark [x]_

## Phase 3: Verification and Testing

- [x] 3.1 Run specific fixed tests multiple times
  - Command: `for i in {1..10}; do cargo test test_c_api_null_label_clears test_macro_generates_doc -- --exact || break; done`
  - Verify both tests pass consistently
  - Check for flakiness
  - Purpose: Ensure fixes are stable
  - _Leverage: Fixed tests from Phase 2_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA stability tester

Task: Verify test stability following requirement 3. Run both fixed tests 10 times in loop: `for i in {1..10}; do cargo test test_c_api_null_label_clears test_macro_generates_doc -- --exact || break; done`. Document if all 10 passes succeed or if any fail. Check for any non-deterministic behavior. If flaky, investigate and fix.

Restrictions: Verification only. If tests are flaky, document but handle in separate fix task.

Success: Both tests pass all 10 times. No flakiness. Tests are stable and reliable.

After completing:
1. Mark [-], run stability tests, document results, log findings, mark [x]_

- [x] 3.2 Run full library test suite
  - Command: `cargo test --lib`
  - Verify all library tests pass (2,440+ tests)
  - Check for any new failures
  - Purpose: Ensure no regressions in library
  - _Leverage: cargo test_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA regression tester

Task: Run full library test suite following requirement 3. Execute `cargo test --lib --verbose` and capture output. Verify: (1) All tests pass, (2) Test count matches expected (~2,440), (3) No new failures introduced, (4) Fixed tests show as passing. Document any failures. Save output to .spec-workflow/specs/fix-failing-tests/test_results_lib.txt.

Restrictions: Verification only. If other tests fail, document for follow-up but don't fix in this task.

Success: All library tests pass. Test count correct. No regressions. Fixed tests confirmed passing. Results documented.

After completing:
1. Mark [-], run tests, capture output, log results, mark [x]_

- [x] 3.3 Run complete test suite
  - Command: `cargo test --all`
  - Verify all tests pass including integration and E2E tests
  - Check total test count
  - Purpose: Ensure full project test coverage
  - _Leverage: cargo test_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA comprehensive tester

Task: Run complete test suite following requirement 3. Execute `cargo test --all --verbose` and capture output. Verify all tests pass across all packages. Document: (1) Total test count, (2) Pass/fail summary, (3) Any warnings or issues, (4) Test runtime. Save to .spec-workflow/specs/fix-failing-tests/test_results_all.txt.

Restrictions: Verification only. Document any issues but don't fix.

Success: All tests pass. Complete coverage verified. No failures anywhere. Results documented.

After completing:
1. Mark [-], run tests, document, log, mark [x]_

- [x] 3.4 Run clippy and verify no new warnings
  - Command: `cargo clippy --all-targets -- -D warnings`
  - Verify fixes didn't introduce warnings
  - Check code quality maintained
  - Purpose: Ensure code quality standards
  - _Leverage: cargo clippy_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code quality specialist

Task: Run clippy validation. Execute `cargo clippy --all-targets -- -D warnings`. Verify no warnings. If warnings exist from fixes, document them. Check specifically around modified code from fixes.

Restrictions: Verification only. Document warnings but don't fix.

Success: Clippy passes with no warnings. Code quality maintained. Any warnings documented.

After completing:
1. Mark [-], run clippy, document, log, mark [x]_

## Phase 4: Enable and Measure Coverage

- [x] 4.1 Run code coverage measurement
  - Command: `cargo llvm-cov --lib --summary-only`
  - Measure overall code coverage percentage
  - Generate detailed coverage report
  - Purpose: Enable coverage metrics now that tests pass
  - _Leverage: cargo llvm-cov_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code coverage analyst

Task: Measure code coverage following requirement 4. Run `cargo llvm-cov --lib --summary-only` to get overall percentage. Then run `cargo llvm-cov --lib --html` to generate detailed report. Document: (1) Overall coverage percentage, (2) Compare to 80% target, (3) Critical path modules coverage, (4) Compare to 90% target for critical paths. Save summary to .spec-workflow/specs/fix-failing-tests/coverage_results.txt.

Restrictions: Measurement only. Don't add tests to improve coverage yet.

Success: Coverage successfully measured. Overall percentage known. Critical paths identified. Gaps documented. Report generated.

After completing:
1. Mark [-], measure coverage, document results, log findings, mark [x]_

- [x] 4.2 Document coverage gaps if any
  - File: .spec-workflow/specs/fix-failing-tests/coverage_gaps.md
  - Identify modules below 80% coverage
  - Identify critical paths below 90% coverage
  - Purpose: Plan future coverage improvements
  - _Leverage: Coverage report from 4.1_
  - _Requirements: 4.2, 4.3_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code coverage analyst

Task: Document coverage gaps following requirements 4.2-4.3. Review HTML coverage report from 4.1. Identify: (1) Modules below 80% overall, (2) Critical paths (services, api, engine, ffi) below 90%, (3) Specific files/functions with low coverage, (4) Recommend priorities for adding tests. Create table with module, current coverage, target, gap. Document in coverage_gaps.md.

Restrictions: Documentation only. Don't write tests yet. Provide actionable recommendations.

Success: All coverage gaps documented with specific line numbers. Priorities identified. Recommendations clear. Serves as roadmap for future test additions.

After completing:
1. Mark [-], analyze and document gaps, log findings, mark [x]_

## Phase 5: CI Validation and Documentation

- [x] 5.1 Run full CI checks
  - Command: `just ci-check` or manual CI steps
  - Verify all CI steps pass: fmt, clippy, test, doc
  - Ensure CI pipeline is green
  - Purpose: Final validation before completion
  - _Leverage: Project CI configuration_
  - _Requirements: 3.1, 3.5_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: CI/CD specialist

Task: Run full CI validation following requirement 3. Execute `just ci-check` or run manually: (1) cargo fmt --check, (2) cargo clippy --all-targets -- -D warnings, (3) cargo test --all, (4) cargo doc --no-deps. Verify all pass. Document results. Capture any failures.

Restrictions: Validation only. If CI fails, document for follow-up.

Success: All CI checks pass. Pipeline green. Project ready for merge. Results documented.

After completing:
1. Mark [-], run CI, document, log results, mark [x]_

- [x] 5.2 Update CODEBASE_EVALUATION.md with results
  - File: CODEBASE_EVALUATION.md
  - Add section documenting test fix results
  - Record coverage measurements
  - Note impact on CI/CD
  - Purpose: Document completion of #3 priority
  - _Leverage: All verification results_
  - _Requirements: All_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical writer

Task: Update evaluation document. Add section "# Implementation Results - Fix Failing Tests" at end of CODEBASE_EVALUATION.md. Document: (1) Tests fixed - list both tests, (2) Root causes - explain what was wrong, (3) Fixes applied - what changed, (4) Test results - 2,440/2,440 passing, (5) Coverage enabled - percentage achieved, (6) CI status - now passing, (7) Time taken - actual vs estimated. Use clear format with before/after comparison.

Restrictions: Be factual and specific. Use actual measurements. Document lessons learned.

Success: Evaluation updated with comprehensive results. Impact clear. Stakeholders see value. Future reference available.

After completing:
1. Mark [-], update docs, log update, mark [x]_

- [ ] 5.3 Document fix details for future reference
  - File: .spec-workflow/specs/fix-failing-tests/FIX_SUMMARY.md
  - Create detailed fix documentation
  - Include root causes, solutions, prevention tips
  - Purpose: Help future developers understand fixes
  - _Leverage: All analysis and fix work_
  - _Requirements: All_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation specialist

Task: Create fix summary document. Structure: (1) Executive Summary - both tests fixed, (2) Test 1 Details - what failed, why, how fixed, (3) Test 2 Details - what failed, why, how fixed, (4) Prevention - how to avoid similar issues, (5) Lessons Learned - key takeaways. Include code snippets showing before/after. Keep under 300 lines.

Restrictions: Write for future developers. Be clear and educational. Include concrete examples.

Success: Fix summary created. Comprehensive reference for similar issues. Helps prevent recurrence. Educational for team.

After completing:
1. Mark [-], write summary, log creation, mark [x]_

## Phase 6: Cleanup and Final Steps

- [ ] 6.1 Remove temporary debug code if any
  - Files: Any files with debug output added during investigation
  - Clean up println!, eprintln!, dbg! macros added for debugging
  - Ensure no debug artifacts remain
  - Purpose: Clean codebase
  - _Leverage: Investigation work from Phase 1_
  - _Requirements: Maintainability_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code cleanup specialist

Task: Remove debug code. Search for: (1) println! or eprintln! added during investigation, (2) dbg! macros, (3) TODO comments added for tracking, (4) Temporary files created during debugging. Remove all debugging artifacts. Ensure code is clean.

Restrictions: Only remove temporary debug code. Don't remove intentional logging or test output.

Success: No debug artifacts remain. Code is clean. Codebase ready for commit.

After completing:
1. Mark [-], clean code, verify clean, log cleanup, mark [x]_

- [ ] 6.2 Format code and verify consistency
  - Command: `cargo fmt`
  - Ensure all modified code properly formatted
  - Verify formatting check passes
  - Purpose: Maintain code formatting standards
  - _Leverage: cargo fmt_
  - _Requirements: Code quality_
  - _Prompt: Implement the task for spec fix-failing-tests. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code formatting specialist

Task: Format code. Run `cargo fmt` to auto-format all modified files. Then run `cargo fmt --check` to verify. Ensure formatting is consistent with project standards.

Restrictions: Just run formatter. Trust cargo fmt output.

Success: All code formatted correctly. `cargo fmt --check` passes. Code style consistent.

After completing:
1. Mark [-], format, verify, log, mark [x]_

## Summary

**Total Tasks:** 16 tasks across 6 phases

**Estimated Effort:** 1-2 hours

**Expected Impact:**
- ✅ CI/CD pipeline unblocked (was broken)
- ✅ All 2,440 tests passing (was 2,438/2,440)
- ✅ Code coverage measurable (was blocked)
- ✅ Quality metrics enabled
- ✅ Confident PR merges
- ✅ Fast turnaround (<2 hours actual vs 1-2 hour estimate)

**Key Deliverables:**
- 2 tests fixed with documented root causes
- Full test suite passing (100% pass rate)
- Code coverage measured and documented
- CI pipeline green
- Comprehensive fix documentation for future reference
- Prevention guidelines to avoid similar issues
