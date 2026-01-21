# Design: Automated API E2E Testing with Auto-Fix

## Overview

This design document describes the architecture and implementation details for an automated end-to-end testing system that exercises the KeyRx web UI backend via REST API, validates responses, and iteratively fixes issues automatically.

## Goals

1. **Comprehensive API Testing**: Cover all REST endpoints with multiple scenarios (30+ test cases)
2. **Automated Issue Resolution**: Fix common issues (network, schema, data) without manual intervention
3. **Fast Feedback**: Complete test suite in < 2 minutes
4. **Clear Diagnostics**: Actionable error messages with detailed diffs
5. **Easy to Extend**: Add new tests with minimal boilerplate (< 20 lines)

## Non-Goals

- Browser UI testing (covered by e2e-playwright-testing spec)
- Performance profiling or load testing
- Security testing (penetration, fuzzing)
- Manual test execution (CLI automation only)

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Test Runner                          │
│  - Parse arguments (--daemon-path, --fix, --max-iterations) │
│  - Orchestrate entire test flow                             │
│  - Output progress and final report                          │
└─────────────┬───────────────────────────────────────────────┘
              │
              ├─► DaemonFixture (start/stop/health)
              ├─► ApiClient (typed REST calls)
              ├─► TestExecutor (run test suite)
              ├─► ResponseComparator (diff actual vs expected)
              ├─► IssueClassifier (categorize failures)
              ├─► FixOrchestrator (apply fixes, retry)
              └─► Reporter (console, JSON, HTML)
```

### Component Diagram

```
┌────────────────┐
│ CLI Arguments  │
└───────┬────────┘
        │
        ▼
┌────────────────────────────────────────────────────────────┐
│                  Test Runner (Main Loop)                   │
│                                                            │
│  1. startDaemon()                                         │
│  2. executeTests()                                        │
│  3. compareResults()                                      │
│  4. if failures && --fix:                                 │
│       classifyIssues() → applyFixes() → retryTests()     │
│  5. generateReports()                                     │
│  6. stopDaemon()                                          │
└────────────────────────────────────────────────────────────┘
        │
        ├─► DaemonFixture
        │   ├─ start(config): Promise<void>
        │   ├─ waitUntilReady(timeout): Promise<void>
        │   ├─ stop(): Promise<void>
        │   └─ getLogs(): string[]
        │
        ├─► ApiClient
        │   ├─ getStatus(): Promise<StatusResponse>
        │   ├─ getDevices(): Promise<DeviceResponse[]>
        │   ├─ getProfiles(): Promise<ProfileResponse[]>
        │   ├─ createProfile(name): Promise<void>
        │   ├─ deleteProfile(name): Promise<void>
        │   ├─ activateProfile(name): Promise<void>
        │   ├─ patchDevice(id, updates): Promise<void>
        │   ├─ getMetrics(): Promise<MetricsResponse>
        │   └─ getLayouts(): Promise<LayoutResponse[]>
        │
        ├─► TestExecutor
        │   ├─ runAll(client, cases): Promise<TestSuiteResult>
        │   └─ runSingle(client, case): Promise<TestResult>
        │
        ├─► ResponseComparator
        │   └─ compare(actual, expected, options): ComparisonResult
        │
        ├─► IssueClassifier
        │   └─ classify(testResult): Issue[]
        │
        ├─► FixOrchestrator
        │   └─ fixAndRetry(results, maxIter): Promise<FixResult[]>
        │       ├─ RestartDaemonStrategy
        │       ├─ UpdateExpectedResultStrategy
        │       ├─ ReseedFixtureStrategy
        │       └─ RetryTestStrategy
        │
        └─► Reporter
            ├─ formatConsole(results): string
            ├─ formatJson(results): string
            └─ generateHtml(results, outputPath): void
```

### Data Flow

```
Test Case Definition
    │
    ▼
Test Executor ─► API Client ─► Daemon ─► Actual Response
    │                                         │
    ▼                                         ▼
Expected Results ──────────────────► Response Comparator
Database                                      │
                                              ▼
                                     Comparison Result
                                              │
                                      ┌───────┴────────┐
                                      │                │
                                  [MATCH]          [MISMATCH]
                                      │                │
                                      ▼                ▼
                                  PASS           Issue Classifier
                                                       │
                                                       ▼
                                                 Issue Categories
                                                       │
                                                       ▼
                                                Fix Orchestrator
                                                       │
                                            ┌──────────┴──────────┐
                                            │                     │
                                        Apply Fix            [NO FIX]
                                            │                     │
                                            ▼                     ▼
                                      Retry Test               FAIL
                                            │
                                    ┌───────┴───────┐
                                [FIXED]        [STILL FAILING]
                                    │                 │
                                    ▼                 ▼
                                  PASS        Iterate (max 3x)
```

## Detailed Component Design

### 1. CLI Test Runner

**File**: `scripts/automated-e2e-test.ts`

**Responsibilities**:
- Parse command-line arguments
- Orchestrate test flow (start daemon → run tests → fix → report)
- Handle graceful shutdown (SIGINT/SIGTERM)
- Stream progress to stdout

**Interface**:
```typescript
interface CliOptions {
  daemonPath: string;         // Path to keyrx_daemon binary
  port: number;               // Daemon port (default: 9867)
  maxIterations: number;      // Max fix attempts (default: 3)
  fix: boolean;               // Enable auto-fix (default: false)
  reportJson: string | null;  // JSON output path (default: null)
  verbose: boolean;           // Verbose logging (default: false)
}

async function main(options: CliOptions): Promise<number>
```

**Implementation Notes**:
- Use `commander` or `yargs` for CLI parsing
- Exit codes: 0 (success), 1 (tests failed), 2 (error)
- Handle Ctrl+C gracefully (cleanup daemon, save partial results)

### 2. DaemonFixture

**File**: `scripts/fixtures/daemon-fixture.ts`

**Responsibilities**:
- Start daemon as subprocess
- Poll health endpoint until ready
- Stop daemon gracefully
- Collect daemon logs

**Interface**:
```typescript
class DaemonFixture {
  constructor(binaryPath: string, port: number);

  async start(config?: string): Promise<void>;
  async waitUntilReady(timeout: number): Promise<void>;
  async stop(): Promise<void>;
  getLogs(): string[];
  isRunning(): boolean;
}
```

**Implementation Notes**:
- Use `child_process.spawn()` to start daemon
- Capture stdout/stderr with `on('data')`
- Health check: `GET /api/status` returns 200
- Graceful shutdown: SIGTERM → wait 5s → SIGKILL (Linux) or taskkill (Windows)

### 3. ApiClient

**File**: `scripts/api-client/client.ts`

**Responsibilities**:
- Typed API calls for all endpoints
- Response validation using Zod schemas
- Automatic retry with exponential backoff
- Clear error messages

**Interface**:
```typescript
class ApiClient {
  constructor(baseUrl: string, timeout: number = 5000);

  async getStatus(): Promise<z.infer<typeof StatusSchema>>;
  async getDevices(): Promise<z.infer<typeof DeviceSchema>[]>;
  async getProfiles(): Promise<z.infer<typeof ProfileSchema>[]>;
  async createProfile(name: string): Promise<void>;
  async deleteProfile(name: string): Promise<void>;
  async activateProfile(name: string): Promise<void>;
  async patchDevice(id: string, updates: DeviceUpdate): Promise<void>;
  async getMetrics(): Promise<z.infer<typeof MetricsSchema>>;
  async getLayouts(): Promise<z.infer<typeof LayoutSchema>[]>;
}
```

**Implementation Notes**:
- Use `axios` or native `fetch` with `node-fetch` polyfill
- Import Zod schemas from `keyrx_ui/src/api/schemas.ts`
- Retry on network errors: 3 attempts, backoff 100ms → 200ms → 400ms
- Throw typed errors: `NetworkError`, `ValidationError`, `ApiError`

### 4. Test Case Definitions

**File**: `scripts/test-cases/api-tests.ts`

**Responsibilities**:
- Define all test cases (30+)
- Arrange-Act-Assert pattern
- Cleanup after each test

**Interface**:
```typescript
interface TestCase {
  id: string;                             // Unique identifier (e.g., "api-status-healthy")
  name: string;                           // Human-readable name
  endpoint: string;                       // API endpoint
  scenario: string;                       // Scenario name (e.g., "healthy", "empty")
  setup: () => Promise<void>;             // Pre-test setup (create profile, etc.)
  execute: (client: ApiClient) => Promise<any>;  // API call
  assert: (response: any, expected: any) => ValidationResult;  // Validation
  cleanup: () => Promise<void>;           // Post-test cleanup
}

const testCases: TestCase[] = [
  {
    id: 'api-status-healthy',
    name: 'GET /api/status - healthy daemon',
    endpoint: '/api/status',
    scenario: 'healthy',
    setup: async () => { /* no setup */ },
    execute: async (client) => client.getStatus(),
    assert: (actual, expected) => /* ... */,
    cleanup: async () => { /* no cleanup */ },
  },
  // ... 29 more test cases
];
```

**Implementation Notes**:
- Use unique test IDs for tracking
- Setup creates test data (profiles, devices)
- Cleanup always runs (use `finally`)
- Assert uses ResponseComparator

### 5. TestExecutor

**File**: `scripts/test-executor/executor.ts`

**Responsibilities**:
- Execute test suite sequentially
- Collect results with timing
- Handle test failures gracefully

**Interface**:
```typescript
interface TestResult {
  id: string;
  name: string;
  status: 'pass' | 'fail';
  duration: number;            // Milliseconds
  error?: string;
  actual?: any;
  expected?: any;
  diff?: Diff[];
}

interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  duration: number;
  results: TestResult[];
}

class TestExecutor {
  async runAll(client: ApiClient, cases: TestCase[]): Promise<TestSuiteResult>;
  async runSingle(client: ApiClient, case: TestCase): Promise<TestResult>;
}
```

**Implementation Notes**:
- Use `performance.now()` for timing
- Continue on test failure (don't stop suite)
- Timeout per test: 30s (configurable)
- Ensure cleanup runs even on error

### 6. ResponseComparator

**File**: `scripts/comparator/response-comparator.ts`

**Responsibilities**:
- Deep equality check with diff output
- Ignore fields (timestamps, IDs)
- Semantic comparison (array order, whitespace)

**Interface**:
```typescript
interface ComparisonOptions {
  ignoreFields?: string[];      // Fields to ignore (e.g., ['timestamp', 'id'])
  ignoreArrayOrder?: boolean;   // Semantic array comparison
  ignoreWhitespace?: boolean;   // Ignore leading/trailing whitespace
}

interface Diff {
  path: string;                 // Path to differing field (e.g., "devices[0].name")
  expected: any;
  actual: any;
}

interface ComparisonResult {
  matches: boolean;
  diff?: Diff[];
  ignoredFields: string[];
}

class ResponseComparator {
  compare(actual: any, expected: any, options?: ComparisonOptions): ComparisonResult;
}
```

**Implementation Notes**:
- Use `deep-diff` or `jest-diff` library
- Handle nested objects, arrays, null/undefined
- Handle circular references (use WeakMap)
- Provide clear path to diff (e.g., `devices[0].name`)

### 7. Expected Results Database

**File**: `scripts/fixtures/expected-results.json`

**Structure**:
```json
{
  "version": "1.0",
  "endpoints": {
    "/api/status": {
      "scenarios": {
        "healthy": {
          "status": 200,
          "body": {
            "version": "0.1.0",
            "uptime": 12345,
            "config_hash": "abc123",
            "active_profile": "Default"
          }
        },
        "starting": {
          "status": 503,
          "body": {
            "message": "Daemon is starting"
          }
        }
      }
    },
    "/api/devices": {
      "scenarios": {
        "empty": {
          "status": 200,
          "body": []
        },
        "multiple": {
          "status": 200,
          "body": [
            {
              "id": "device-1",
              "name": "Test Keyboard",
              "path": "/dev/input/event0",
              "layout": "us"
            }
          ]
        }
      }
    }
    // ... other endpoints
  }
}
```

**Implementation Notes**:
- Versioned for compatibility tracking
- Use JSON Schema for validation
- Update when API contract changes (with auto-fix or manually)

### 8. IssueClassifier

**File**: `scripts/auto-fix/issue-classifier.ts`

**Responsibilities**:
- Classify failures into categories (network, validation, logic, data)
- Extract fixable patterns
- Priority scoring (1=auto-fix, 2=needs hint, 3=manual)

**Interface**:
```typescript
interface Issue {
  type: 'network' | 'validation' | 'logic' | 'data';
  fixable: boolean;
  priority: number;          // 1=auto-fix, 2=hint, 3=manual
  description: string;
  suggestedFix?: string;
  testResult: TestResult;
}

class IssueClassifier {
  classify(testResult: TestResult): Issue[];
}
```

**Classification Rules**:
```typescript
// Network issues
if (error.code === 'ECONNREFUSED') {
  return { type: 'network', fixable: true, priority: 1, suggestedFix: 'RestartDaemonStrategy' };
}

// Validation issues (schema mismatch)
if (error instanceof z.ZodError) {
  return { type: 'validation', fixable: true, priority: 2, suggestedFix: 'UpdateExpectedResultStrategy' };
}

// Logic issues (wrong value)
if (diff.path && diff.expected !== diff.actual) {
  return { type: 'logic', fixable: false, priority: 3, suggestedFix: 'Manual investigation' };
}

// Data issues (empty array)
if (actual.length === 0 && expected.length > 0) {
  return { type: 'data', fixable: true, priority: 1, suggestedFix: 'ReseedFixtureStrategy' };
}
```

### 9. Fix Strategies

**File**: `scripts/auto-fix/fix-strategies.ts`

**Responsibilities**:
- Implement fix strategies for different issue types
- Idempotent (safe to apply multiple times)
- Conservative (never modify code)

**Interface**:
```typescript
interface FixContext {
  daemonFixture: DaemonFixture;
  apiClient: ApiClient;
  testCase: TestCase;
}

interface FixResult {
  success: boolean;
  message: string;
  retry: boolean;        // Should retry test after fix?
}

interface FixStrategy {
  canFix(issue: Issue): boolean;
  apply(issue: Issue, context: FixContext): Promise<FixResult>;
}
```

**Implementations**:

```typescript
// 1. Restart Daemon (network errors)
class RestartDaemonStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'network' && issue.fixable;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    await context.daemonFixture.stop();
    await context.daemonFixture.start();
    await context.daemonFixture.waitUntilReady(30000);
    return { success: true, message: 'Daemon restarted', retry: true };
  }
}

// 2. Update Expected Results (schema mismatches)
class UpdateExpectedResultStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'validation' && issue.fixable;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    // In CI: require human approval (fail with message)
    if (process.env.CI) {
      return {
        success: false,
        message: 'Schema mismatch requires human approval. Update expected-results.json manually.',
        retry: false
      };
    }

    // Locally: update expected-results.json
    const { endpoint, scenario } = issue.testResult;
    const actualResponse = issue.testResult.actual;
    updateExpectedResults(endpoint, scenario, actualResponse);
    return { success: true, message: 'Updated expected results', retry: true };
  }
}

// 3. Reseed Fixtures (data issues)
class ReseedFixtureStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'data' && issue.fixable;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    // Clean up stale data
    await context.apiClient.deleteAllProfiles();

    // Re-create test profile
    await context.apiClient.createProfile('TestProfile');

    return { success: true, message: 'Fixtures reseeded', retry: true };
  }
}

// 4. Retry Test (transient failures)
class RetryTestStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.fixable && issue.priority === 1;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    // Wait before retry (exponential backoff)
    const delay = Math.pow(2, issue.retryCount || 0) * 100;
    await new Promise(resolve => setTimeout(resolve, delay));
    return { success: true, message: `Retrying after ${delay}ms`, retry: true };
  }
}
```

### 10. FixOrchestrator

**File**: `scripts/auto-fix/fix-orchestrator.ts`

**Responsibilities**:
- Apply fixes in priority order
- Retry tests after each fix
- Track fix history (prevent infinite loops)

**Interface**:
```typescript
interface FixAttempt {
  strategy: string;
  success: boolean;
  message: string;
}

interface FixResult {
  testId: string;
  fixAttempts: FixAttempt[];
  finalStatus: 'fixed' | 'failed';
}

class FixOrchestrator {
  constructor(
    strategies: FixStrategy[],
    maxIterations: number = 3
  );

  async fixAndRetry(
    testResults: TestResult[],
    context: FixContext
  ): Promise<FixResult[]>;
}
```

**Implementation**:
```typescript
async fixAndRetry(testResults: TestResult[], context: FixContext): Promise<FixResult[]> {
  const results: FixResult[] = [];
  const fixHistory = new Set<string>();  // Prevent infinite loops

  for (const testResult of testResults.filter(r => r.status === 'fail')) {
    const fixResult: FixResult = {
      testId: testResult.id,
      fixAttempts: [],
      finalStatus: 'failed'
    };

    let iterations = 0;
    let currentResult = testResult;

    while (iterations < this.maxIterations && currentResult.status === 'fail') {
      // Classify issues
      const issues = this.classifier.classify(currentResult);

      // Sort by priority (1=highest)
      issues.sort((a, b) => a.priority - b.priority);

      let fixed = false;
      for (const issue of issues) {
        // Check fix history to prevent infinite loops
        const historyKey = `${issue.testId}-${issue.suggestedFix}`;
        if (fixHistory.has(historyKey)) continue;

        // Find applicable strategy
        const strategy = this.strategies.find(s => s.canFix(issue));
        if (!strategy) continue;

        // Apply fix
        const result = await strategy.apply(issue, context);
        fixResult.fixAttempts.push({
          strategy: strategy.constructor.name,
          success: result.success,
          message: result.message
        });

        fixHistory.add(historyKey);

        if (result.success && result.retry) {
          // Retry test
          currentResult = await this.executor.runSingle(context.apiClient, context.testCase);
          fixed = true;
          break;
        }
      }

      if (!fixed) break;  // No fix applied, stop iterating
      iterations++;
    }

    fixResult.finalStatus = currentResult.status === 'pass' ? 'fixed' : 'failed';
    results.push(fixResult);
  }

  return results;
}
```

### 11. Reporters

**File**: `scripts/comparator/validation-reporter.ts`, `scripts/reporters/html-reporter.ts`

**Responsibilities**:
- Format test results for humans (console) and machines (JSON)
- Generate visual HTML report
- Include diff visualization

**Console Output**:
```
Running 30 tests...

✓ GET /api/status - healthy daemon (45ms)
✓ GET /api/devices - empty state (32ms)
✗ GET /api/profiles - multiple profiles (123ms)
  Expected: [{ name: "Default" }, { name: "Gaming" }]
  Actual:   [{ name: "Default" }]
  Diff:     Missing profile "Gaming" at index 1

Applying fixes...
  → RestartDaemonStrategy: Daemon restarted
  Retry 1/3: ✓ GET /api/profiles - multiple profiles (98ms)

Summary: 30 passed, 0 failed (2.1s)
```

**JSON Output** (`--report-json`):
```json
{
  "version": "1.0",
  "timestamp": "2026-01-21T12:00:00Z",
  "summary": {
    "total": 30,
    "passed": 30,
    "failed": 0,
    "duration": 2100
  },
  "results": [
    {
      "id": "api-status-healthy",
      "name": "GET /api/status - healthy daemon",
      "status": "pass",
      "duration": 45,
      "fixAttempts": []
    }
  ]
}
```

**HTML Report**:
- Summary card (total/passed/failed)
- Test list (filterable by status)
- Detail view per test (request/response/expected/actual/diff)
- Fix attempt history
- Syntax highlighting for JSON

## Performance Considerations

### Test Execution Time
- Target: < 2 minutes for 30 tests
- Strategies:
  - Sequential execution (avoid race conditions)
  - Fast daemon startup (< 10s)
  - Efficient API calls (< 5s per test)
  - Minimal setup/cleanup

### Memory Usage
- Daemon: ~50MB
- Test runner: ~20MB
- Expected results: ~100KB
- Total: ~70MB (acceptable)

### CI Resource Usage
- CPU: 1 core sufficient
- Memory: 512MB sufficient
- Disk: 100MB for artifacts

## Security Considerations

### Secrets Management
- Never log full request/response bodies
- Redact sensitive fields (API keys, tokens)
- Use environment variables for secrets

### Input Validation
- Validate all CLI arguments
- Sanitize file paths (prevent directory traversal)
- Validate JSON schemas

### Fix Strategies
- Never execute arbitrary code
- Never modify application code
- Only update config/fixtures
- Require human approval for schema changes in CI

## Error Handling

### Error Categories
1. **Network Errors**: ECONNREFUSED, ETIMEDOUT, ENOTFOUND
2. **Validation Errors**: Zod schema mismatches
3. **Logic Errors**: Wrong values, unexpected state
4. **Data Errors**: Empty fixtures, stale data

### Error Propagation
- Fail-fast validation at entry
- Propagate errors with context
- User sees clear error messages

### Logging
- Structured JSON logs with timestamp, level, service, event, context
- Example: `{ "timestamp": "2026-01-21T12:00:00Z", "level": "error", "service": "automated-e2e", "event": "test_failed", "context": { "testId": "...", "error": "..." } }`

## Testing Strategy

### Unit Tests
- `api-client.test.ts`: Mock daemon responses
- `response-comparator.test.ts`: Synthetic diffs
- `fix-strategies.test.ts`: Mock issues
- Coverage: 80%+ overall, 90%+ for critical paths

### Integration Tests
- `tests/integration/full-flow.test.ts`: End-to-end flow with real daemon

### Determinism
- Use fixed timestamps in expected results
- Avoid race conditions (sequential execution)
- Use virtual clock for timeout tests

## Future Enhancements

### Phase 8 (Future)
1. **Parallel Execution**: Speed up suite with worker threads
2. **Snapshot Testing**: Auto-generate expected results from actual responses
3. **AI-Powered Fixes**: Use LLM to suggest fixes for logic bugs
4. **Performance Tracking**: Track response times, alert on slowdowns
5. **Contract-First Development**: Generate tests from OpenAPI spec

## References

- **Similar Projects**: Jest, Playwright, Cypress (inspiration)
- **Libraries**: Zod (validation), deep-diff (comparison), chalk (colors)
- **Standards**: REST API best practices, TAP protocol (test output)

## Appendix: File Size Estimates

| File | Estimated Lines |
|------|-----------------|
| automated-e2e-test.ts | 300 |
| daemon-fixture.ts | 150 |
| api-client.ts | 250 |
| test-cases/api-tests.ts | 400 |
| test-executor.ts | 200 |
| response-comparator.ts | 200 |
| issue-classifier.ts | 150 |
| fix-strategies.ts | 300 |
| fix-orchestrator.ts | 250 |
| validation-reporter.ts | 200 |
| html-reporter.ts | 250 |
| **Total** | **2650** |

All files < 500 lines ✅
