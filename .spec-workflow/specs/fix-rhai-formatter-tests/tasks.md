# Tasks: Fix RhaiFormatter Tests

- [x] 1. Analyze failing tests
  - Review rhaiFormatter.test.ts failures, understand expectations vs actual
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec fix-rhai-formatter-tests. First run spec-workflow-guide, then implement: Run `npm test rhaiFormatter` to see failures. Analyze each failing test. Determine if formatter behavior is correct or tests need update. Document findings. | Restrictions: Analysis only, no changes yet | Success: All failures analyzed, fix strategy documented | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 2. Update test expectations
  - Fix test assertions to match current formatter behavior
  - _Leverage: src/utils/rhaiFormatter.ts_
  - _Prompt: Role: Test Engineer | Task: Implement the task for spec fix-rhai-formatter-tests. First run spec-workflow-guide, then implement: Update failing test expectations in rhaiFormatter.test.ts. Adjust line counts, expected output strings to match current formatter (which adds device_start/end wrappers). Manually verify formatter output is correct. | Restrictions: Don't change formatter logic unless buggy, fix tests to match correct behavior | Success: All 17 tests pass, expectations correct, formatter verified | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 3. Verify all tests pass
  - Run full test suite
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec fix-rhai-formatter-tests. First run spec-workflow-guide, then implement: Run `npm test rhaiFormatter` to verify all tests pass. Run full test suite to ensure no regressions. | Restrictions: All tests must pass | Success: rhaiFormatter tests pass, no regressions | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_
