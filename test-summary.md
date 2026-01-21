# E2E Test Suite Status - 2026-01-22

## Fixed Issue
Fixed a bug in `scripts/automated-e2e-test.ts` where the auto-fix engine was trying to iterate over `fixResult.fixAttempts` directly, but the correct structure is `fixResult.testResults` where each test result contains a `fixAttempts` array.

**Commit:** e0f8320f - fix(test): correct fixResult.fixAttempts iteration bug

## Current Test Results

**Overall:** 26/83 tests passing (31.3%)

### Category Breakdown

| Category | Passed/Total | Pass Rate | Duration |
|----------|--------------|-----------|----------|
| Macros | 6/8 | 75.0% | 13ms |
| Simulator | 5/7 | 71.4% | 7ms |
| Health | 2/3 | 66.7% | 8ms |
| Profiles | 7/20 | 35.0% | 36ms |
| Layouts | 1/3 | 33.3% | 3ms |
| Metrics | 1/4 | 25.0% | 4ms |
| Websocket | 1/5 | 20.0% | 20.02s |
| Config | 2/11 | 18.2% | 12ms |
| Devices | 1/15 | 6.7% | 6.98s |
| Status | 0/1 | 0.0% | 1ms |
| Workflows | 0/6 | 0.0% | 1.27s |

**Total Duration:** 28.468 seconds

## Key Issues Identified

### 1. Device Management (6.7% pass rate)
- Most device-related tests failing with schema validation errors
- Layout assignment issues
- Device enable/disable problems

### 2. Workflows (0% pass rate)
- All workflow tests failing
- Socket connection errors
- HTTP 405 errors
- Invalid request formats

### 3. WebSocket (20% pass rate)
- Subscription timeouts (5 seconds)
- Event notification failures
- Only basic connect/disconnect works

### 4. Configuration (18.2% pass rate)
- Generator errors
- Key mapping failures
- Layer management issues

### 5. Status Endpoint (0% pass rate)
- Failing with undefined status

## Next Steps

The spec `rest-api-comprehensive-e2e` is marked as completed with all 54 tasks checked off. However, the verification checklist shows that the tests need significant fixes to achieve the 100% pass rate goal.

The main areas requiring attention:
1. Device management API responses and schema validation
2. WebSocket subscription/notification infrastructure
3. Workflow test scenarios and prerequisites
4. Configuration/mapping API error handling
5. Status endpoint implementation

## Running Tests

```bash
# Run full test suite with auto-fix
npx tsx scripts/automated-e2e-test.ts \
  --daemon-path target/release/keyrx_daemon \
  --port 9867 \
  --fix \
  --max-iterations 3 \
  --report-json test-results.json

# View results
cat test-results.json | jq '.summary'
```
