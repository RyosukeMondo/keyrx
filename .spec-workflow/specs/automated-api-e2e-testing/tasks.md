# Tasks: Automated API E2E Testing with Auto-Fix

## Overview
Implement automated end-to-end testing system that exercises web UI features via REST API, validates responses, and iteratively fixes issues automatically.

## Architecture
```
Test Runner (Node.js/TypeScript)
    ↓
Launch Daemon
    ↓
Execute API Test Suite → Compare with Expected Results
    ↓                           ↓
[PASS] ────────────────────→ Report Success
    ↓
[FAIL] → Auto-Fix Engine → Retry Test
    ↓           ↓
Report Failure  Iterate (max 3 attempts)
```

---

## Phase 1: Test Infrastructure

### Task 1: Create Test Runner Framework
- [x] 1.1 Create test runner script
  - File: `scripts/automated-e2e-test.ts`
  - CLI interface with options: `--daemon-path`, `--port`, `--max-iterations`, `--fix`, `--report-json`
  - Daemon lifecycle management (start/stop/restart)
  - Test suite orchestration
  - Purpose: Central orchestration of automated testing
  - _Leverage: scripts/validate-api-contracts.ts (daemon interaction pattern)_
  - _Requirements: Automated test execution, daemon management_
  - _Prompt: Role: Test Infrastructure Engineer | Task: Create automated-e2e-test.ts CLI tool that starts daemon, runs test suite, manages lifecycle. Options: --daemon-path (default: target/release/keyrx_daemon), --port (default: 9867), --max-iterations (default: 3), --fix (enable auto-fix), --report-json (output JSON report). Use child_process to spawn daemon, wait for /api/status ready, handle cleanup on exit. | Restrictions: Must handle Windows/Linux daemon paths, timeout daemon startup after 30s, ensure cleanup on SIGINT/SIGTERM | Success: CLI runs, starts daemon, executes tests, outputs results_

- [x] 1.2 Create daemon fixture with health checks
  - File: `scripts/fixtures/daemon-fixture.ts`
  - Start daemon with test profile
  - Health check polling (GET /api/status)
  - Graceful shutdown with timeout
  - Log collection for debugging
  - Purpose: Reliable daemon lifecycle for tests
  - _Leverage: scripts/validate-api-contracts.ts_
  - _Requirements: Robust daemon management_
  - _Prompt: Role: Test Infrastructure Developer | Task: Create DaemonFixture class with methods: start(config), waitUntilReady(timeout), stop(), getLogs(). Start spawns daemon subprocess, waitUntilReady polls /api/status every 100ms up to timeout, stop sends SIGTERM then SIGKILL after 5s, getLogs reads stderr/stdout. | Restrictions: Handle port conflicts (retry with next port), capture all daemon output, work on Windows (use taskkill) and Linux (SIGTERM) | Success: Can reliably start/stop daemon, passes health checks_

- [x] 1.3 Create test result database schema
  - File: `scripts/fixtures/expected-results.json`
  - JSON schema for expected API responses
  - Versioned by API contract version
  - Organized by endpoint and scenario
  - Purpose: Single source of truth for expected behavior
  - _Leverage: None (new file)_
  - _Requirements: Structured expected results_
  - _Prompt: Role: QA Data Engineer | Task: Create expected-results.json with structure: { "version": "1.0", "endpoints": { "/api/status": { "scenarios": { "healthy": { "status": 200, "body": {...} }, "starting": {...} } }, "/api/devices": {...}, "/api/profiles": {...} } }. Include all REST endpoints with typical scenarios (success, empty, error). Base on API contract. | Restrictions: Use JSON Schema for validation, include all documented endpoints | Success: Complete expected results database for all API endpoints_

---

## Phase 2: API Test Suite

### Task 2: Create API Client Library
- [x] 2.1 Create typed API client
  - File: `scripts/api-client/client.ts`
  - Type-safe functions for all REST endpoints
  - Request/response validation using Zod schemas
  - Automatic retry with exponential backoff
  - Purpose: DRY, type-safe API testing
  - _Leverage: keyrx_ui/src/api/schemas.ts (Zod schemas)_
  - _Requirements: Type-safe API access_
  - _Prompt: Role: API Client Developer | Task: Create ApiClient class with methods for all endpoints: getStatus(), getDevices(), getProfiles(), createProfile(name), activateProfile(name), deleteProfile(name), patchDevice(id, updates), getMetrics(), getLayouts(), setProfileConfig(name, config). Use axios or fetch with typed responses. Import Zod schemas from keyrx_ui/src/api/schemas.ts. Auto-retry on network errors (3 attempts, exponential backoff). | Restrictions: Validate all responses against Zod schemas, throw typed errors on validation failure, timeout requests after 5s | Success: All API endpoints accessible via typed methods_

- [x] 2.2 Create API test case definitions
  - File: `scripts/test-cases/api-tests.ts`
  - Test case definitions for each endpoint
  - Arrange-Act-Assert pattern
  - Parametrized tests for multiple scenarios
  - Purpose: Comprehensive API contract testing
  - _Leverage: scripts/api-client/client.ts, scripts/fixtures/expected-results.json_
  - _Requirements: Complete API coverage_
  - _Prompt: Role: QA Test Engineer | Task: Create TestCase interface and array of test cases: { id: string, name: string, endpoint: string, scenario: string, setup: () => Promise<void>, execute: (client: ApiClient) => Promise<Response>, assert: (response, expected) => ValidationResult }. Define tests for all endpoints: GET /api/status (healthy, starting), GET /api/devices (empty, multiple), GET /api/profiles (default, multiple), POST /api/profiles (create, duplicate), DELETE /api/profiles/:name (existing, not-found), PATCH /api/devices/:id (valid, invalid), GET /api/metrics/latency, GET /api/layouts. | Restrictions: Each test must be isolated (cleanup after), use descriptive names, follow AAA pattern | Success: 30+ test cases covering all endpoints and scenarios_

- [x] 2.3 Create test executor
  - File: `scripts/test-executor/executor.ts`
  - Execute test suite sequentially
  - Collect results with timing
  - Handle test failures gracefully
  - Purpose: Orchestrate test execution
  - _Leverage: scripts/test-cases/api-tests.ts, scripts/api-client/client.ts_
  - _Requirements: Reliable test execution_
  - _Prompt: Role: Test Execution Engineer | Task: Create TestExecutor class with methods: runAll(client, cases), runSingle(client, case). For each test: call setup(), execute(client), assert(response, expected), collect result { id, name, status: 'pass'|'fail', duration, error?, actual?, expected? }. Continue on failure (don't stop suite). Add timeout per test (30s). Log progress to console. Return TestSuiteResult { total, passed, failed, duration, results: TestResult[] }. | Restrictions: Isolate tests (catch errors per test), measure duration accurately, ensure cleanup runs even on error | Success: Executes all tests, handles failures, provides detailed results_

---

## Phase 3: Result Comparison & Validation

### Task 3: Create Result Comparator
- [x] 3.1 Create response comparator
  - File: `scripts/comparator/response-comparator.ts`
  - Deep equality check with diff output
  - Ignore fields (timestamps, IDs)
  - Semantic comparison (array order, whitespace)
  - Purpose: Accurate result validation
  - _Leverage: None (use diff library like jest-diff or deep-diff)_
  - _Requirements: Robust comparison logic_
  - _Prompt: Role: Test Validation Engineer | Task: Create ResponseComparator class with method: compare(actual, expected, options?): ComparisonResult. ComparisonResult { matches: boolean, diff?: Diff[], ignoredFields: string[] }. Use deep-diff or jest-diff for object comparison. Options: ignoreFields (e.g., ['timestamp', 'id']), ignoreArrayOrder (semantic comparison), ignoreWhitespace. Return detailed diff on mismatch with path to differing field. | Restrictions: Handle nested objects, arrays, null/undefined, circular references, special JSON types (Date) | Success: Accurately compares responses, provides actionable diffs_

- [x] 3.2 Create validation reporter
  - File: `scripts/comparator/validation-reporter.ts`
  - Format comparison results for humans
  - Generate JSON report for machines
  - Diff visualization (color-coded, side-by-side)
  - Purpose: Clear failure reporting
  - _Leverage: scripts/comparator/response-comparator.ts_
  - _Requirements: Readable failure reports_
  - _Prompt: Role: Reporting Engineer | Task: Create ValidationReporter class with methods: formatHuman(result), formatJson(result). Human format: color-coded pass/fail (green/red), test name, duration, diff output (expected vs actual with +/- lines). JSON format: structured { version, timestamp, summary: { total, passed, failed }, results: [...] }. Use chalk for colors. For diffs, use side-by-side format with line numbers. | Restrictions: Handle no-color terminals (check env CI or NO_COLOR), keep JSON machine-parseable, limit diff output (max 100 lines) | Success: Clear human-readable and JSON reports_

---

## Phase 4: Auto-Fix Engine

### Task 4: Create Auto-Fix System
- [x] 4.1 Create issue classifier
  - File: `scripts/auto-fix/issue-classifier.ts`
  - Classify failure types (network, validation, logic, data)
  - Extract fixable patterns (timeout, missing field, wrong type)
  - Priority scoring (easy to hard)
  - Purpose: Identify fixable issues
  - _Leverage: scripts/comparator/response-comparator.ts (diff analysis)_
  - _Requirements: Intelligent issue detection_
  - _Prompt: Role: Issue Analysis Engineer | Task: Create IssueClassifier class with method: classify(testResult): Issue[]. Issue { type: 'network'|'validation'|'logic'|'data', fixable: boolean, priority: number, description: string, suggestedFix?: string }. Analyze test failure patterns: network errors (ECONNREFUSED, timeout) → retry/restart daemon, validation errors (wrong type, missing field) → schema mismatch, logic errors (wrong value) → business logic bug, data errors (empty array) → fixture issue. Priority: 1 (auto-fixable), 2 (needs hint), 3 (manual). | Restrictions: Don't guess fixes for complex logic bugs, only suggest fixes for patterns you can detect | Success: Classifies failures into actionable categories_

- [x] 4.2 Create auto-fix strategies
  - File: `scripts/auto-fix/fix-strategies.ts`
  - Strategy pattern for different fix types
  - Network fixes (restart daemon, wait longer)
  - Schema fixes (update expected results)
  - Data fixes (re-seed fixtures)
  - Purpose: Automated issue remediation
  - _Leverage: scripts/auto-fix/issue-classifier.ts, scripts/fixtures/daemon-fixture.ts_
  - _Requirements: Automated fixes for common issues_
  - _Prompt: Role: Auto-Fix Engineer | Task: Create FixStrategy interface { canFix(issue): boolean, apply(issue, context): Promise<FixResult> } and implementations: RestartDaemonStrategy (network errors), UpdateExpectedResultStrategy (schema mismatches), ReseedFixtureStrategy (data issues), RetryTestStrategy (transient failures). FixResult { success: boolean, message: string, retry: boolean }. Context includes daemon fixture, API client, test case. | Restrictions: Each strategy must be idempotent (safe to apply multiple times), log all actions, never modify code (only config/fixtures) | Success: Strategies can fix network, schema, and data issues_

- [x] 4.3 Create fix orchestrator
  - File: `scripts/auto-fix/fix-orchestrator.ts`
  - Apply fixes in priority order
  - Retry tests after each fix
  - Track fix history (prevent infinite loops)
  - Purpose: Coordinate auto-fix attempts
  - _Leverage: scripts/auto-fix/issue-classifier.ts, scripts/auto-fix/fix-strategies.ts_
  - _Requirements: Intelligent fix iteration_
  - _Prompt: Role: Fix Orchestration Engineer | Task: Create FixOrchestrator class with method: fixAndRetry(testResults, maxIterations): Promise<FixResult[]>. For each failed test: classify issues, sort by priority, apply strategies until fixed or max iterations reached. Track fix history (Set<issueId>) to prevent infinite loops. Retry test after each fix. Return results: { testId, fixAttempts: { strategy, success, message }[], finalStatus: 'fixed'|'failed' }. Stop iterating if test passes. | Restrictions: Max 3 iterations per test, skip tests with no fixable issues, timeout entire process after 5 minutes | Success: Iteratively applies fixes, retries tests, prevents infinite loops_

---

## Phase 5: Integration & Reporting

### Task 5: Integrate Components
- [x] 5.1 Wire up test runner with auto-fix
  - File: `scripts/automated-e2e-test.ts` (update)
  - Connect test executor → comparator → auto-fix → retry loop
  - Command-line flags for fix behavior
  - Progress reporting during execution
  - Purpose: End-to-end automated testing flow
  - _Leverage: All previous components_
  - _Requirements: Complete integration_
  - _Prompt: Role: Integration Engineer | Task: Update automated-e2e-test.ts to integrate all components. Flow: startDaemon() → executeTests() → compareResults() → if failures and --fix flag: classifyIssues() → applyFixes() → retryTests() → repeat. Add progress logging: "Starting daemon...", "Running 30 tests...", "15 passed, 5 failed", "Applying fixes...", "Retry 1/3: 3 tests fixed, 2 remaining". Output final report via ValidationReporter. | Restrictions: Handle Ctrl+C gracefully (stop daemon, save partial results), stream progress to stdout (don't buffer), respect --max-iterations flag | Success: Complete flow from daemon start to final report with auto-fix_

- [x] 5.2 Create HTML test report generator
  - File: `scripts/reporters/html-reporter.ts`
  - Generate visual HTML report
  - Test results with pass/fail indicators
  - Diff visualization with syntax highlighting
  - Fix attempt history
  - Purpose: Visual test result inspection
  - _Leverage: scripts/comparator/validation-reporter.ts (JSON output)_
  - _Requirements: Human-friendly visual reports_
  - _Prompt: Role: Reporting Engineer | Task: Create HtmlReporter class with method: generate(testResults, outputPath). HTML structure: summary card (total/passed/failed), test list (filterable by status), detail view per test (request/response/expected/actual/diff with syntax highlighting), fix attempt history. Use template HTML with embedded JSON data + client-side JS for interactivity. Style with simple CSS (no framework). Export standalone HTML file. | Restrictions: No external dependencies in HTML (must work offline), keep file size < 500KB, use syntax highlighting for JSON (highlight.js via CDN) | Success: Generates standalone HTML report with test results and diffs_

- [ ] 5.3 Add npm script and Makefile target
  - File: `package.json` (keyrx_ui), `Makefile` (root)
  - Add `test:e2e:auto` script
  - Add `make e2e-auto` target
  - Purpose: Easy invocation
  - _Leverage: scripts/automated-e2e-test.ts_
  - _Requirements: Convenient test execution_
  - _Prompt: Role: DevOps Engineer | Task: Add to keyrx_ui/package.json: "test:e2e:auto": "tsx ../scripts/automated-e2e-test.ts --daemon-path ../target/release/keyrx_daemon --fix --report-json test-results.json", "test:e2e:auto:report": "tsx ../scripts/reporters/html-reporter.ts test-results.json report.html". Add to root Makefile: `e2e-auto: build` target that builds release daemon then runs npm run test:e2e:auto. | Restrictions: Ensure tsx is installed (add as dev dependency), build daemon before running tests | Success: npm run test:e2e:auto and make e2e-auto work_

---

## Phase 6: CI Integration & Monitoring

### Task 6: CI Integration
- [ ] 6.1 Create GitHub Actions workflow
  - File: `.github/workflows/e2e-auto.yml`
  - Run automated e2e tests on PR
  - Upload test reports as artifacts
  - Comment results on PR
  - Purpose: Automated testing in CI
  - _Leverage: .github/workflows/ci.yml (existing patterns)_
  - _Requirements: CI automation_
  - _Prompt: Role: CI/CD Engineer | Task: Create e2e-auto.yml workflow triggered on PR. Jobs: build (compile daemon release), e2e-tests (run automated-e2e-test.ts, upload JSON results as artifact, generate HTML report, upload as artifact). Use ubuntu-latest. Install Node.js 18, Rust, build daemon, run npm run test:e2e:auto. On failure, upload logs and report. Add PR comment with summary (use actions/github-script). | Restrictions: Timeout workflow after 15 minutes, cache cargo build, only run on changes to daemon or UI code | Success: Workflow runs on PR, uploads artifacts, comments results_

- [ ] 6.2 Add test metrics collection
  - File: `scripts/metrics/test-metrics.ts`
  - Collect metrics: pass rate, duration, fix success rate
  - Track trends over time
  - Export to JSON for dashboarding
  - Purpose: Monitor test quality over time
  - _Leverage: scripts/automated-e2e-test.ts (test results)_
  - _Requirements: Historical metrics tracking_
  - _Prompt: Role: Metrics Engineer | Task: Create TestMetrics class with method: record(testResults, timestamp). Metrics: totalTests, passedTests, failedTests, duration, fixAttempts, fixSuccesses, averageTestDuration, slowestTests (top 5). Append to metrics.jsonl (JSON Lines format) with timestamp. Add report() method to generate summary: pass rate trend (last 10 runs), average duration, most flaky tests. | Restrictions: Use JSON Lines (one JSON object per line) for efficient appending, rotate file after 1000 lines, handle missing file gracefully | Success: Metrics collected and queryable_

- [ ] 6.3 Create dashboard monitoring setup
  - File: `scripts/dashboard/e2e-dashboard.html`
  - Real-time test status dashboard
  - Fetch metrics from CI artifacts or local runs
  - Charts for pass rate, duration trends
  - Purpose: Visual monitoring of test health
  - _Leverage: scripts/metrics/test-metrics.ts (metrics.jsonl)_
  - _Requirements: Test health visibility_
  - _Prompt: Role: Dashboard Engineer | Task: Create standalone HTML dashboard that loads metrics.jsonl and displays: current pass rate (gauge), pass rate trend (line chart, last 30 days), average duration trend, top 10 slowest tests, top 10 flakiest tests. Use Chart.js (via CDN) for charts. Add refresh button to reload data. Support loading from file (local) or URL (CI artifact). | Restrictions: Standalone HTML (no build step), handle missing/malformed data gracefully, keep page size < 200KB, mobile-responsive | Success: Dashboard visualizes test metrics_

---

## Phase 7: Documentation & Examples

### Task 7: Documentation
- [ ] 7.1 Create comprehensive README
  - File: `scripts/automated-e2e-testing/README.md`
  - Architecture overview with diagram
  - Quick start guide
  - Configuration options
  - Troubleshooting guide
  - Purpose: Developer onboarding
  - _Leverage: All scripts and components_
  - _Requirements: Complete documentation_
  - _Prompt: Role: Technical Writer | Task: Create README.md with sections: Overview (architecture diagram), Quick Start (npm run test:e2e:auto), Configuration (flags: --daemon-path, --port, --max-iterations, --fix), Expected Results Database (how to update expected-results.json), Auto-Fix Strategies (how to add new strategies), Troubleshooting (common errors and solutions), CI Integration (workflow usage). Include code examples. | Restrictions: Keep concise (max 2000 words), use diagrams (Mermaid), link to implementation files | Success: Complete, clear documentation_

- [ ] 7.2 Create developer guide for adding tests
  - File: `scripts/automated-e2e-testing/DEV_GUIDE.md`
  - How to add new test cases
  - How to update expected results
  - How to write fix strategies
  - Purpose: Enable contributions
  - _Leverage: All scripts and components_
  - _Requirements: Contribution guide_
  - _Prompt: Role: Developer Advocate | Task: Create DEV_GUIDE.md with sections: Adding Test Cases (TestCase interface, example), Updating Expected Results (when/how to update expected-results.json), Writing Fix Strategies (FixStrategy interface, example implementation), Running Tests Locally (commands, debugging), Best Practices (test isolation, determinism, performance). Include step-by-step tutorials. | Restrictions: Use concrete examples, keep pragmatic (not academic), max 1500 words | Success: Developers can add tests and fix strategies_

- [ ] 7.3 Create example test case
  - File: `scripts/automated-e2e-testing/examples/example-test.ts`
  - Complete example test case with setup/execute/assert
  - Demonstrates best practices
  - Well-commented
  - Purpose: Template for new tests
  - _Leverage: scripts/test-cases/api-tests.ts_
  - _Requirements: Reference implementation_
  - _Prompt: Role: Example Code Author | Task: Create example-test.ts demonstrating complete test case: setup (create test profile), execute (activate profile via API), assert (verify active profile in /api/status), cleanup (delete test profile). Add extensive comments explaining each part. Show error handling, type safety, assertion patterns. Include both simple and complex scenarios. | Restrictions: Follow existing test patterns, demonstrate best practices, keep under 150 lines | Success: Clear, well-commented example test_

---

## Task Dependencies

```
Phase 1 (Infrastructure)
    1.1 → 1.2 → 1.3
         ↓
Phase 2 (Test Suite)
    2.1 → 2.2 → 2.3
         ↓
Phase 3 (Comparison)
    3.1 → 3.2
         ↓
Phase 4 (Auto-Fix)
    4.1 → 4.2 → 4.3
         ↓
Phase 5 (Integration)
    5.1 (depends on all above) → 5.2 → 5.3
         ↓
Phase 6 (CI)
    6.1 → 6.2 → 6.3
         ↓
Phase 7 (Documentation)
    7.1, 7.2, 7.3 (parallel, depend on implementation)
```

## Success Criteria

### Functional Requirements
1. ✅ Automated test runner starts daemon and executes all API tests
2. ✅ Test results compared against expected results with detailed diffs
3. ✅ Auto-fix engine resolves network, schema, and data issues
4. ✅ Iterative retry loop (max 3 attempts) with fix history tracking
5. ✅ HTML and JSON reports generated
6. ✅ CI integration runs tests on PR

### Quality Requirements
1. ✅ Test suite covers all REST API endpoints (GET/POST/PATCH/DELETE)
2. ✅ Response validation using Zod schemas (type-safe)
3. ✅ Fix success rate > 60% for fixable issues
4. ✅ Test execution time < 2 minutes (30 tests)
5. ✅ Dashboard shows test health trends

### Developer Experience
1. ✅ One command to run: `npm run test:e2e:auto`
2. ✅ Clear failure messages with actionable diffs
3. ✅ HTML report for visual inspection
4. ✅ Easy to add new tests (template provided)
5. ✅ Documentation complete and clear

## Technical Debt Prevention

1. **Determinism**: All tests must be deterministic (no flakiness)
   - Use fixed timestamps, seeded RNG
   - Avoid race conditions (sequential execution)

2. **Isolation**: Tests must not interfere with each other
   - Clean up resources (profiles, devices, config)
   - Use unique test data per test

3. **Performance**: Keep test suite fast (< 2 minutes)
   - Parallelize where safe
   - Use in-memory fixtures where possible

4. **Maintainability**: Code must be < 500 lines per file
   - Extract shared utilities
   - Follow existing patterns

5. **Type Safety**: Use TypeScript strict mode
   - No `any` types
   - Validate all responses with Zod

## Notes

- This spec focuses on **REST API testing**, not browser UI testing (that's covered by e2e-playwright-testing)
- Auto-fix engine can only fix **known patterns** (network, schema, data), not arbitrary logic bugs
- Fix strategies should be **conservative** (never modify code, only config/fixtures)
- Test suite should be **fast** (aim for < 2 minutes total execution)
- Expected results database is **versioned** (update when API contract changes)
- Dashboard is **optional** but useful for long-term monitoring

## Total Tasks: 20

| Phase | Tasks | Complexity |
|-------|-------|------------|
| 1. Infrastructure | 3 | Medium |
| 2. Test Suite | 3 | Medium |
| 3. Comparison | 2 | Easy |
| 4. Auto-Fix | 3 | Hard |
| 5. Integration | 3 | Medium |
| 6. CI Integration | 3 | Medium |
| 7. Documentation | 3 | Easy |

Estimated completion: 3-5 days for experienced developer
