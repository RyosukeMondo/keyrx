# Automated E2E Testing with Auto-Fix

Comprehensive automated end-to-end testing system for the KeyRx daemon REST API. Executes test suites, validates responses, and automatically fixes common issues through iterative remediation.

## Architecture

```mermaid
graph TB
    A[Test Runner] --> B[Daemon Fixture]
    B --> C[Start Daemon]
    C --> D[API Client]
    D --> E[Test Executor]
    E --> F{Test Results}
    F -->|Pass| G[Generate Report]
    F -->|Fail| H{Auto-Fix Enabled?}
    H -->|Yes| I[Issue Classifier]
    I --> J[Fix Strategies]
    J --> K[Apply Fixes]
    K --> L[Retry Tests]
    L --> F
    H -->|No| G
    G --> M[Metrics Recorder]
    M --> N[Dashboard]
```

### Components

- **Test Runner** ([automated-e2e-test.ts](../automated-e2e-test.ts)) - Main orchestrator
- **Daemon Fixture** ([fixtures/daemon-fixture.ts](../fixtures/daemon-fixture.ts)) - Manages daemon lifecycle
- **API Client** ([api-client/client.ts](../api-client/client.ts)) - Type-safe API interactions
- **Test Executor** ([test-executor/executor.ts](../test-executor/executor.ts)) - Runs test suite
- **Response Comparator** ([comparator/response-comparator.ts](../comparator/response-comparator.ts)) - Deep equality checks
- **Issue Classifier** ([auto-fix/issue-classifier.ts](../auto-fix/issue-classifier.ts)) - Identifies fixable patterns
- **Fix Strategies** ([auto-fix/fix-strategies.ts](../auto-fix/fix-strategies.ts)) - Automated remediation
- **Fix Orchestrator** ([auto-fix/fix-orchestrator.ts](../auto-fix/fix-orchestrator.ts)) - Coordinates fix attempts
- **Test Metrics** ([metrics/test-metrics.ts](../metrics/test-metrics.ts)) - Collects and analyzes metrics
- **Dashboard** ([dashboard/e2e-dashboard.html](../dashboard/e2e-dashboard.html)) - Visual metrics interface

## Quick Start

### Run Automated Tests

```bash
# From project root
npm run test:e2e:auto

# Or with explicit options
npx tsx scripts/automated-e2e-test.ts \
  --daemon-path target/release/keyrx_daemon \
  --port 9867 \
  --fix \
  --max-iterations 3 \
  --report-json test-results.json
```

### View Test Report

```bash
# Generate HTML report from JSON results
npm run test:e2e:auto:report

# Open report.html in browser
```

### View Metrics

```bash
# View recent metrics summary
npm run metrics:report

# View latest test run
npm run metrics:latest

# Open dashboard
open scripts/dashboard/e2e-dashboard.html
```

## Configuration

### Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--daemon-path <path>` | Path to daemon binary | `target/release/keyrx_daemon` |
| `--port <number>` | API port | `9867` |
| `--max-iterations <number>` | Max auto-fix attempts | `3` |
| `--fix` | Enable auto-fix mode | `false` |
| `--report-json <path>` | JSON report output path | None |
| `--metrics-file <path>` | Metrics JSONL file | `metrics.jsonl` |

### Test Configuration

Test cases are defined in [test-cases/api-tests.ts](../test-cases/api-tests.ts). Each test follows this structure:

```typescript
{
  id: 'test-001',
  name: 'GET /api/status - healthy daemon',
  endpoint: '/api/status',
  scenario: 'healthy',
  category: 'status',
  priority: 1,
  setup: async () => {
    // Setup test preconditions
  },
  execute: async (client: ApiClient) => {
    return await client.getStatus();
  },
  assert: (response, expected) => {
    // Compare response with expected results
  },
  cleanup: async () => {
    // Clean up test artifacts
  }
}
```

### Expected Results Database

Expected API responses are stored in [fixtures/expected-results.json](../fixtures/expected-results.json):

```json
{
  "version": "1.0",
  "apiVersion": "0.1.0",
  "endpoints": {
    "/api/status": {
      "scenarios": {
        "healthy": {
          "status": 200,
          "body": {
            "status": "running",
            "version": "0.1.0"
          }
        }
      }
    }
  }
}
```

**When to Update:**
- API contract changes (new fields, different responses)
- Endpoint behavior changes
- Bug fixes that change expected output

**How to Update:**
1. Run test to get actual response
2. Verify response is correct
3. Update `expected-results.json` with new values
4. Re-run test to confirm

## Auto-Fix Strategies

The system includes built-in strategies for common issues:

### 1. Restart Daemon Strategy

**Fixes:** Network errors, connection refused, timeouts

**How it works:** Stops and restarts the daemon, waits for health check

**Location:** [auto-fix/fix-strategies.ts](../auto-fix/fix-strategies.ts)

### 2. Update Expected Result Strategy

**Fixes:** Schema mismatches, type errors, unexpected fields

**How it works:** Updates `expected-results.json` with actual response (requires manual review)

**Location:** [auto-fix/fix-strategies.ts](../auto-fix/fix-strategies.ts)

### 3. Retry Test Strategy

**Fixes:** Transient failures, race conditions

**How it works:** Waits and retries the test

**Location:** [auto-fix/fix-strategies.ts](../auto-fix/fix-strategies.ts)

### Adding Custom Strategies

See [DEV_GUIDE.md](./DEV_GUIDE.md) for detailed instructions on creating new fix strategies.

## Metrics & Monitoring

### Metrics Collection

Metrics are automatically recorded after each test run in JSONL format (one JSON object per line):

```json
{
  "timestamp": "2026-01-21T03:00:00.000Z",
  "totalTests": 30,
  "passedTests": 28,
  "failedTests": 2,
  "passRate": 93.3,
  "duration": 45000,
  "fixAttempts": 4,
  "fixSuccesses": 2,
  "fixSuccessRate": 50.0,
  "averageTestDuration": 1500,
  "slowestTests": [...]
}
```

### Query Metrics

```bash
# View summary report (last 10 runs)
npm run metrics:report

# View latest run
npm run metrics:latest

# Clear metrics
npm run metrics:clear
```

### Dashboard

Open [scripts/dashboard/e2e-dashboard.html](../dashboard/e2e-dashboard.html) in a browser to view:

- Current pass rate (gauge chart)
- Pass rate trend (line chart, last 30 days)
- Average duration trend
- Top 10 slowest tests
- Health status indicators

The dashboard loads `metrics.jsonl` via file upload or auto-fetches from the default location.

## CI Integration

### GitHub Actions Workflow

Tests run automatically on pull requests via [.github/workflows/e2e-auto.yml](../../.github/workflows/e2e-auto.yml).

**Workflow:**
1. Build daemon in release mode
2. Run automated E2E tests with auto-fix
3. Upload test results as artifacts
4. Generate and upload HTML report
5. Comment summary on PR

**Artifacts:**
- `test-results.json` - Raw test results
- `test-report.html` - Visual HTML report
- `metrics.jsonl` - Historical metrics

### Running in CI

The workflow is triggered automatically on:
- Pull requests to `main`
- Changes to `keyrx_daemon/**` or `keyrx_ui/**`

**Manual trigger:**
```bash
gh workflow run e2e-auto.yml
```

## Troubleshooting

### Daemon won't start

**Symptoms:** "Failed to start daemon" error

**Solutions:**
1. Check daemon binary exists: `ls target/release/keyrx_daemon`
2. Rebuild daemon: `cargo build --release -p keyrx_daemon`
3. Check port not in use: `lsof -i :9867`
4. Check daemon logs in test output

### Tests failing unexpectedly

**Symptoms:** Previously passing tests now fail

**Solutions:**
1. Check API contract changes: review recent daemon commits
2. Update expected results: compare actual vs expected in test output
3. Run single test: modify test file to focus on failing test
4. Check daemon logs for errors

### Auto-fix not working

**Symptoms:** Fixes applied but tests still fail

**Solutions:**
1. Check max iterations not exceeded (default: 3)
2. Verify issue is fixable (see fix strategy conditions)
3. Check fix logs for error messages
4. Manually apply fix to understand what's needed

### Metrics not recording

**Symptoms:** `metrics.jsonl` not created or not updated

**Solutions:**
1. Check file permissions in project root
2. Verify test completed successfully
3. Check for errors in metrics recording code
4. Manually create empty file: `touch metrics.jsonl`

### Dashboard not loading

**Symptoms:** Dashboard shows "Load a metrics.jsonl file"

**Solutions:**
1. Use file upload to select `metrics.jsonl`
2. Ensure metrics file exists and has valid JSONL format
3. Check browser console for errors
4. Verify Chart.js CDN is accessible

### Port conflicts

**Symptoms:** "Address already in use" error

**Solutions:**
1. Use different port: `--port 9868`
2. Kill process using port: `lsof -ti:9867 | xargs kill -9`
3. Wait for daemon cleanup from previous run

### Windows-specific issues

**Symptoms:** Tests fail on Windows but pass on Linux

**Solutions:**
1. Ensure daemon path ends with `.exe`: `--daemon-path target/release/keyrx_daemon.exe`
2. Check Windows firewall isn't blocking localhost connections
3. Use PowerShell or Git Bash for running tests
4. Check line endings (LF vs CRLF) in config files

## Performance Tips

### Faster Test Runs

1. **Run tests in parallel** (currently sequential for isolation)
2. **Use in-memory fixtures** instead of file-based
3. **Cache daemon builds** in CI
4. **Skip cleanup** for faster local development (use with caution)

### Reduce Flakiness

1. **Increase timeouts** for slow systems
2. **Add retry logic** for network operations
3. **Use deterministic test data** (fixed timestamps, seeded RNG)
4. **Avoid race conditions** (wait for async operations)

## Best Practices

### Test Isolation

- Each test should be independent
- Clean up resources (profiles, configs) after each test
- Use unique test data to avoid conflicts
- Don't rely on test execution order

### Type Safety

- Use Zod schemas for API response validation
- Define TypeScript interfaces for all data structures
- Enable strict mode in TypeScript config
- Avoid `any` types

### Error Handling

- Fail fast on setup errors
- Provide actionable error messages
- Log errors with context (test ID, timestamp)
- Don't swallow errors in cleanup

### Maintainability

- Keep files under 500 lines
- Extract shared utilities
- Follow existing code patterns
- Document complex logic

## File Size Limits

All files in this system respect the 500-line limit:

- `automated-e2e-test.ts`: ~440 lines
- `test-metrics.ts`: ~380 lines
- `fix-orchestrator.ts`: ~450 lines
- `e2e-dashboard.html`: ~600 lines (HTML/CSS/JS combined)

## Related Documentation

- [Developer Guide](./DEV_GUIDE.md) - Adding tests and fix strategies
- [Example Test](./examples/example-test.ts) - Reference implementation
- [API Contract Validation](../validate-api-contracts.ts) - Schema validation tool

## Support

For issues or questions:
1. Check this README and troubleshooting section
2. Review test logs and daemon output
3. Check existing test cases for patterns
4. Consult the Developer Guide for extending the system
