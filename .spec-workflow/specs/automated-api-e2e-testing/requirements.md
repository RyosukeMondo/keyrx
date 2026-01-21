# Requirements: Automated API E2E Testing with Auto-Fix

## 1. Functional Requirements

### 1.1 Test Execution
- **REQ-1.1.1**: System shall execute all API test cases sequentially
- **REQ-1.1.2**: System shall collect test results with accurate timing (millisecond precision)
- **REQ-1.1.3**: System shall handle test failures gracefully without stopping entire suite
- **REQ-1.1.4**: System shall timeout individual tests after 30 seconds
- **REQ-1.1.5**: System shall support filtering tests by endpoint or scenario

### 1.2 Daemon Management
- **REQ-1.2.1**: System shall start daemon subprocess before test execution
- **REQ-1.2.2**: System shall poll daemon health (/api/status) with 100ms interval until ready
- **REQ-1.2.3**: System shall timeout daemon startup after 30 seconds
- **REQ-1.2.4**: System shall stop daemon gracefully (SIGTERM) after tests complete
- **REQ-1.2.5**: System shall force-kill daemon (SIGKILL/taskkill) if graceful shutdown fails after 5s
- **REQ-1.2.6**: System shall capture all daemon stdout/stderr for debugging

### 1.3 API Client
- **REQ-1.3.1**: System shall provide typed client for all REST endpoints (GET/POST/PATCH/DELETE)
- **REQ-1.3.2**: System shall validate all API responses against Zod schemas
- **REQ-1.3.3**: System shall retry network requests up to 3 times with exponential backoff
- **REQ-1.3.4**: System shall timeout individual requests after 5 seconds
- **REQ-1.3.5**: System shall provide clear error messages on validation failures

### 1.4 Result Comparison
- **REQ-1.4.1**: System shall perform deep equality comparison of actual vs expected responses
- **REQ-1.4.2**: System shall ignore specified fields (timestamps, IDs) during comparison
- **REQ-1.4.3**: System shall provide detailed diff output showing exact differences
- **REQ-1.4.4**: System shall support semantic comparison (ignore array order, whitespace)
- **REQ-1.4.5**: System shall handle nested objects, arrays, null/undefined, circular references

### 1.5 Auto-Fix System
- **REQ-1.5.1**: System shall classify failures into categories: network, validation, logic, data
- **REQ-1.5.2**: System shall prioritize fixable issues (1=auto-fix, 2=needs hint, 3=manual)
- **REQ-1.5.3**: System shall apply fix strategies in priority order
- **REQ-1.5.4**: System shall retry test after each fix attempt
- **REQ-1.5.5**: System shall limit fix attempts to 3 iterations per test
- **REQ-1.5.6**: System shall track fix history to prevent infinite loops
- **REQ-1.5.7**: System shall never modify application code (only config/fixtures)

### 1.6 Reporting
- **REQ-1.6.1**: System shall output human-readable console report with color-coding
- **REQ-1.6.2**: System shall generate machine-parseable JSON report
- **REQ-1.6.3**: System shall generate visual HTML report with syntax highlighting
- **REQ-1.6.4**: System shall include fix attempt history in all reports
- **REQ-1.6.5**: System shall provide clear diff visualization (side-by-side with line numbers)

## 2. Non-Functional Requirements

### 2.1 Performance
- **REQ-2.1.1**: Test suite shall complete in < 2 minutes (30 tests)
- **REQ-2.1.2**: Individual test execution shall take < 5 seconds
- **REQ-2.1.3**: Daemon startup shall complete in < 10 seconds
- **REQ-2.1.4**: Fix application shall complete in < 30 seconds per attempt

### 2.2 Reliability
- **REQ-2.2.1**: All tests shall be deterministic (no flakiness)
- **REQ-2.2.2**: Tests shall be isolated (no interference between tests)
- **REQ-2.2.3**: System shall handle daemon crashes gracefully
- **REQ-2.2.4**: System shall clean up resources on SIGINT/SIGTERM
- **REQ-2.2.5**: Fix strategies shall be idempotent (safe to apply multiple times)

### 2.3 Maintainability
- **REQ-2.3.1**: All source files shall be < 500 lines (excluding comments/blank lines)
- **REQ-2.3.2**: All functions shall be < 50 lines
- **REQ-2.3.3**: Test coverage shall be ≥ 80% overall, ≥ 90% for critical paths
- **REQ-2.3.4**: Code shall follow SOLID principles and dependency injection
- **REQ-2.3.5**: Documentation shall be comprehensive and up-to-date

### 2.4 Usability
- **REQ-2.4.1**: System shall provide CLI with clear help text
- **REQ-2.4.2**: Error messages shall be actionable and clear
- **REQ-2.4.3**: Progress shall be visible during execution (streaming output)
- **REQ-2.4.4**: HTML report shall work offline (no external dependencies)
- **REQ-2.4.5**: Adding new tests shall require < 20 lines of code

### 2.5 Security
- **REQ-2.5.1**: System shall never log secrets or PII
- **REQ-2.5.2**: System shall redact sensitive fields in logs/reports
- **REQ-2.5.3**: System shall validate all inputs (fail-fast)
- **REQ-2.5.4**: Fix strategies shall never execute arbitrary code

### 2.6 Compatibility
- **REQ-2.6.1**: System shall run on Windows, Linux, macOS
- **REQ-2.6.2**: System shall support Node.js 18+
- **REQ-2.6.3**: System shall integrate with GitHub Actions
- **REQ-2.6.4**: System shall work in CI environment (no TTY)
- **REQ-2.6.5**: System shall respect NO_COLOR environment variable

## 3. API Coverage Requirements

### 3.1 Endpoints (11 total)
- **REQ-3.1.1**: GET /api/status - daemon health and version
- **REQ-3.1.2**: GET /api/devices - list all input devices
- **REQ-3.1.3**: GET /api/profiles - list all configuration profiles
- **REQ-3.1.4**: GET /api/profiles/:name/config - get profile configuration
- **REQ-3.1.5**: POST /api/profiles - create new profile
- **REQ-3.1.6**: DELETE /api/profiles/:name - delete profile
- **REQ-3.1.7**: POST /api/profiles/:name/activate - activate profile
- **REQ-3.1.8**: PATCH /api/devices/:id - update device configuration
- **REQ-3.1.9**: GET /api/metrics/latency - get latency statistics
- **REQ-3.1.10**: GET /api/layouts - get available keyboard layouts
- **REQ-3.1.11**: POST /api/config/reload - reload configuration

### 3.2 Scenarios per Endpoint (minimum)
- **REQ-3.2.1**: Success case (200/201 response)
- **REQ-3.2.2**: Empty state (e.g., no devices, no profiles)
- **REQ-3.2.3**: Error case (4xx/5xx response)
- **REQ-3.2.4**: Edge cases (invalid input, missing fields)

### 3.3 Total Test Cases
- **REQ-3.3.1**: Minimum 30 test cases covering all endpoints and scenarios

## 4. Auto-Fix Strategy Requirements

### 4.1 Network Issues
- **REQ-4.1.1**: Detect ECONNREFUSED, ETIMEDOUT, ENOTFOUND errors
- **REQ-4.1.2**: Apply RestartDaemonStrategy (restart daemon, wait, retry)
- **REQ-4.1.3**: Apply RetryTestStrategy (retry with exponential backoff)
- **REQ-4.1.4**: Log restart attempts and reasons

### 4.2 Schema Issues
- **REQ-4.2.1**: Detect Zod validation errors (wrong type, missing field, extra field)
- **REQ-4.2.2**: Apply UpdateExpectedResultStrategy (update expected-results.json)
- **REQ-4.2.3**: Require human approval in CI (fail with clear message)
- **REQ-4.2.4**: Log schema mismatches with exact diff

### 4.3 Data Issues
- **REQ-4.3.1**: Detect empty arrays when data expected
- **REQ-4.3.2**: Detect stale data (old profiles, devices)
- **REQ-4.3.3**: Apply ReseedFixtureStrategy (clean up, re-create test data)
- **REQ-4.3.4**: Log data cleanup actions

### 4.4 Transient Issues
- **REQ-4.4.1**: Detect race conditions (timing-sensitive failures)
- **REQ-4.4.2**: Apply RetryTestStrategy with delay
- **REQ-4.4.3**: Limit retries to 3 attempts
- **REQ-4.4.4**: Log retry attempts with delay duration

## 5. CI Integration Requirements

### 5.1 GitHub Actions Workflow
- **REQ-5.1.1**: Trigger on pull_request and push to main
- **REQ-5.1.2**: Build daemon in release mode
- **REQ-5.1.3**: Run automated e2e tests with --fix flag
- **REQ-5.1.4**: Upload test results as artifacts (JSON, HTML, logs)
- **REQ-5.1.5**: Comment test summary on PR
- **REQ-5.1.6**: Fail workflow if tests fail after auto-fix
- **REQ-5.1.7**: Timeout workflow after 15 minutes

### 5.2 Metrics Collection
- **REQ-5.2.1**: Collect pass rate, duration, fix attempts, fix successes
- **REQ-5.2.2**: Store metrics in JSON Lines format (metrics.jsonl)
- **REQ-5.2.3**: Track trends over time (last 30 days)
- **REQ-5.2.4**: Identify flaky tests (fail-pass ratio)
- **REQ-5.2.5**: Identify slow tests (duration > 5s)

### 5.3 Dashboard
- **REQ-5.3.1**: Display current pass rate (gauge)
- **REQ-5.3.2**: Display pass rate trend (line chart)
- **REQ-5.3.3**: Display average duration trend
- **REQ-5.3.4**: Display top 10 slowest tests
- **REQ-5.3.5**: Display top 10 flakiest tests
- **REQ-5.3.6**: Support loading from file (local) or URL (CI artifact)

## 6. Documentation Requirements

### 6.1 User Documentation
- **REQ-6.1.1**: README.md with overview, quick start, configuration
- **REQ-6.1.2**: Architecture diagram (Mermaid)
- **REQ-6.1.3**: Troubleshooting guide with common errors and solutions
- **REQ-6.1.4**: Examples of running locally and in CI

### 6.2 Developer Documentation
- **REQ-6.2.1**: DEV_GUIDE.md with contribution guide
- **REQ-6.2.2**: How to add new test cases (step-by-step)
- **REQ-6.2.3**: How to update expected results
- **REQ-6.2.4**: How to write fix strategies
- **REQ-6.2.5**: Example test case with extensive comments

### 6.3 Code Documentation
- **REQ-6.3.1**: JSDoc comments on all public functions
- **REQ-6.3.2**: Module-level comments explaining purpose
- **REQ-6.3.3**: Complex logic explained with inline comments

## 7. Test Coverage Requirements

### 7.1 API Endpoints
- **REQ-7.1.1**: Cover all 11 REST endpoints
- **REQ-7.1.2**: Cover all HTTP methods (GET, POST, PATCH, DELETE)
- **REQ-7.1.3**: Cover all response codes (200, 201, 400, 404, 500)

### 7.2 Edge Cases
- **REQ-7.2.1**: Empty responses (no devices, no profiles)
- **REQ-7.2.2**: Invalid input (malformed JSON, missing fields)
- **REQ-7.2.3**: Concurrent requests (race conditions)
- **REQ-7.2.4**: Large payloads (stress testing)

### 7.3 Error Scenarios
- **REQ-7.3.1**: Network errors (connection refused, timeout)
- **REQ-7.3.2**: Validation errors (schema mismatch)
- **REQ-7.3.3**: Logic errors (wrong value, unexpected state)
- **REQ-7.3.4**: Data errors (stale fixtures, missing data)

## Requirements Traceability Matrix

| Requirement | Task(s) | Priority | Status |
|-------------|---------|----------|--------|
| REQ-1.1.x | 2.3 | High | Pending |
| REQ-1.2.x | 1.2 | High | Pending |
| REQ-1.3.x | 2.1 | High | Pending |
| REQ-1.4.x | 3.1 | High | Pending |
| REQ-1.5.x | 4.1-4.3 | High | Pending |
| REQ-1.6.x | 3.2, 5.2 | Medium | Pending |
| REQ-2.1.x | All | High | Pending |
| REQ-2.2.x | All | High | Pending |
| REQ-2.3.x | All | High | Pending |
| REQ-2.4.x | 1.1, 5.1, 7.x | Medium | Pending |
| REQ-2.5.x | All | High | Pending |
| REQ-2.6.x | All | Medium | Pending |
| REQ-3.1.x | 2.2 | High | Pending |
| REQ-3.2.x | 2.2 | High | Pending |
| REQ-3.3.x | 2.2 | High | Pending |
| REQ-4.1.x | 4.2 | High | Pending |
| REQ-4.2.x | 4.2 | High | Pending |
| REQ-4.3.x | 4.2 | High | Pending |
| REQ-4.4.x | 4.2 | Medium | Pending |
| REQ-5.1.x | 6.1 | Medium | Pending |
| REQ-5.2.x | 6.2 | Low | Pending |
| REQ-5.3.x | 6.3 | Low | Pending |
| REQ-6.1.x | 7.1 | Medium | Pending |
| REQ-6.2.x | 7.2 | Medium | Pending |
| REQ-6.3.x | All | Medium | Pending |
| REQ-7.1.x | 2.2 | High | Pending |
| REQ-7.2.x | 2.2 | Medium | Pending |
| REQ-7.3.x | 2.2, 4.x | High | Pending |

## Success Metrics

### Quantitative
- ✅ Pass rate > 95% after auto-fix
- ✅ Fix success rate > 60% for fixable issues
- ✅ Test execution time < 2 minutes
- ✅ Zero flaky tests (fail-pass ratio = 0%)
- ✅ Test coverage ≥ 80%

### Qualitative
- ✅ Clear, actionable error messages
- ✅ Easy to add new tests (< 20 lines)
- ✅ Easy to debug failures (HTML report)
- ✅ Comprehensive documentation
- ✅ Positive developer feedback

## Out of Scope

- ❌ Browser UI testing (covered by e2e-playwright-testing spec)
- ❌ Performance profiling (covered by future spec)
- ❌ Load testing (covered by future spec)
- ❌ Security testing (covered by future spec)
- ❌ Visual regression testing (covered by future spec)

## Assumptions

1. Daemon supports all documented REST API endpoints
2. Zod schemas in `keyrx_ui/src/api/schemas.ts` are complete and up-to-date
3. Daemon can run without keyboard permissions (API-only mode)
4. Node.js 18+ is available in CI environment
5. Tests run on single machine (no distributed testing)

## Dependencies

1. **keyrx_daemon**: Must be built and executable
2. **Node.js 18+**: Runtime for test scripts
3. **TypeScript 5+**: Type checking and compilation
4. **Zod**: Schema validation
5. **Axios or fetch**: HTTP client
6. **Jest or Vitest**: Test runner
7. **GitHub Actions**: CI/CD platform

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky tests | High | Enforce determinism, use fixed timestamps, avoid race conditions |
| Slow tests | Medium | Set strict timeouts, parallelize where safe, optimize fixtures |
| CI environment issues | Medium | Test locally first, use same environment as CI (Docker), handle no-TTY |
| Fix strategies too aggressive | High | Conservative fixes only (never modify code), require human approval for schema changes |
| Infinite fix loops | High | Track fix history, limit to 3 iterations, log all attempts |
| Daemon crashes | Medium | Graceful error handling, restart capability, collect logs |
