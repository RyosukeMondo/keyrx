# E2E Test Documentation

## Overview

This directory contains end-to-end (E2E) tests for KeyRx using Playwright. E2E tests verify complete user workflows by running against a real daemon instance and UI.

## Test Stability Features

The E2E test suite includes several features to ensure stable, reliable tests:

### 1. Automatic Retries (2 retries on CI)

Flaky network tests are automatically retried up to 2 times on CI to handle transient failures:

```typescript
// Configured in playwright.e2e.config.ts
retries: process.env.CI ? 2 : 0
```

### 2. Daemon Health Checks

The global setup (`global-setup.ts`) ensures the daemon is running before tests start:

- Starts the daemon automatically
- Waits for daemon to be ready (polls `/api/status` endpoint)
- Fails fast if daemon doesn't start within 30 seconds

Additionally, use the `ensureDaemonReady()` helper in individual tests:

```typescript
import { ensureDaemonReady } from './helpers';

test.beforeEach(async () => {
  await ensureDaemonReady();
});
```

### 3. Test Isolation

Each test gets a **fresh browser context** automatically:

- Fresh localStorage and sessionStorage
- Fresh cookies
- Fresh permissions
- Fresh viewport

For explicit storage clearing, use the helper:

```typescript
import { setupFreshTestEnvironment } from './helpers';

test.beforeEach(async ({ page }) => {
  await setupFreshTestEnvironment(page);
});
```

### 4. Debug Artifacts on Failure

When a test fails, Playwright automatically captures:

- **Screenshots** - Full page screenshot at time of failure
- **Videos** - Recording of the entire test run
- **Traces** - Interactive trace viewer with DOM snapshots, network logs, and timeline

Artifacts are saved to: `test-results/e2e/`

View trace files: `npx playwright show-trace test-results/e2e/trace.zip`

### 5. Sequential Execution

E2E tests run sequentially (`workers: 1`) because the daemon state is shared:

```typescript
// Configured in playwright.e2e.config.ts
fullyParallel: false,
workers: 1,
```

This prevents race conditions and state pollution between tests.

## Test Helpers

Located in `helpers.ts`, these utilities improve test stability:

### Daemon Health Checks

```typescript
import { ensureDaemonReady, checkDaemonHealth, waitForDaemonReady } from './helpers';

// Check if daemon is responding
const isReady = await checkDaemonHealth();

// Wait for daemon to be ready (with timeout)
await waitForDaemonReady(5000);

// Throw error if daemon is not ready (for use in beforeEach)
await ensureDaemonReady();
```

### Test Isolation

```typescript
import { clearBrowserStorage, setupFreshTestEnvironment } from './helpers';

// Clear all browser storage (localStorage, sessionStorage, cookies)
await clearBrowserStorage(page);

// Complete fresh environment (daemon check + clear storage)
await setupFreshTestEnvironment(page);
```

### Retry Helpers

```typescript
import { retryOnFailure } from './helpers';

// Retry flaky operation up to 2 times
await retryOnFailure(async () => {
  await page.click('#submit-button');
  await expect(page.locator('.success-message')).toBeVisible();
}, 2, 500);
```

### Page Ready Waits

```typescript
import { waitForPageReady, waitForWebSocketConnection } from './helpers';

// Wait for page to be fully loaded (network idle + React hydration)
await waitForPageReady(page);

// Wait for WebSocket to connect
await waitForWebSocketConnection(page);
```

## Writing Stable Tests

See `examples/stable-test-pattern.spec.ts` for complete examples.

### Pattern 1: Fresh Environment Before Each Test

```typescript
test.beforeEach(async ({ page }) => {
  await setupFreshTestEnvironment(page);
});

test('should do something', async ({ page }) => {
  await page.goto('/');
  await waitForPageReady(page);
  // ... test code
});
```

### Pattern 2: Explicit Daemon Check

```typescript
test('should interact with daemon', async ({ page }) => {
  await page.goto('/devices');
  await waitForPageReady(page);

  // Ensure daemon is ready before critical operation
  await ensureDaemonReady();

  // Perform operation that requires daemon
  await page.click('[data-testid="enable-device"]');
  // ...
});
```

### Pattern 3: Retry Flaky Operations

```typescript
test('should handle network delays', async ({ page }) => {
  await page.goto('/profiles');

  // Wrap in retry for network-dependent operations
  await retryOnFailure(async () => {
    await page.click('#activate-profile');
    await expect(page.locator('.profile-active')).toBeVisible();
  });
});
```

### Pattern 4: Avoid Arbitrary Timeouts

❌ **Bad** - Fragile test with arbitrary waits:

```typescript
await page.click('#submit');
await page.waitForTimeout(2000); // Might be too short or too long
```

✅ **Good** - Wait for specific condition:

```typescript
await page.click('#submit');
await waitForPageReady(page);
await expect(page.locator('.success-message')).toBeVisible({ timeout: 5000 });
```

## Running E2E Tests

### Local Development

```bash
# Run all E2E tests
npm run test:e2e

# Run specific test file
npx playwright test tests/e2e/device-workflow.spec.ts

# Run with UI mode (interactive debugging)
npx playwright test --ui

# Run with headed browser (see what's happening)
npx playwright test --headed
```

### CI Environment

E2E tests run automatically in CI with:
- 2 automatic retries for flaky tests
- Artifact uploads (screenshots, videos, traces)
- Pre-built daemon binary for faster startup

## Debugging Failed Tests

### 1. View Trace Files

```bash
# After a failed test, view the trace
npx playwright show-trace test-results/e2e/trace.zip
```

The trace viewer shows:
- DOM snapshots at each step
- Network requests
- Console logs
- Screenshots
- Timeline of events

### 2. Run with Debug Mode

```bash
# Opens Playwright Inspector for step-by-step debugging
PWDEBUG=1 npx playwright test tests/e2e/device-workflow.spec.ts
```

### 3. Check Daemon Logs

If daemon-related failures occur, check:

```bash
# View daemon stderr/stdout
DEBUG_DAEMON=1 npx playwright test
```

### 4. Take Manual Screenshots

```typescript
import { takeDebugScreenshot } from './helpers';

test('debugging test', async ({ page }) => {
  await page.goto('/');

  // Take screenshot at any point
  await takeDebugScreenshot(page, 'before-click');

  await page.click('#button');

  await takeDebugScreenshot(page, 'after-click');
});
```

## Best Practices

1. **Use descriptive test names** - Clearly state what the test verifies
2. **One user flow per test** - Tests should be focused and independent
3. **Use data-testid for critical elements** - More stable than CSS selectors
4. **Avoid page.waitForTimeout()** - Use explicit waits for conditions
5. **Clean up test data** - Reset state after tests that modify daemon state
6. **Use helpers** - Leverage `helpers.ts` for common patterns
7. **Test error states** - Don't just test happy paths

## Configuration Files

- `playwright.e2e.config.ts` - Main E2E configuration
- `global-setup.ts` - Starts daemon before all tests
- `global-teardown.ts` - Stops daemon after all tests
- `helpers.ts` - Reusable test utilities

## Troubleshooting

### "Daemon not ready" errors

Check:
1. Is daemon binary built? (`cargo build -p keyrx_daemon`)
2. Is port 9867 already in use? (`lsof -i :9867`)
3. Check daemon logs with `DEBUG_DAEMON=1`

### Flaky test failures

1. Add `ensureDaemonReady()` to `beforeEach`
2. Use `retryOnFailure()` for network operations
3. Replace `waitForTimeout()` with explicit waits
4. Check if test isolation is needed (`setupFreshTestEnvironment()`)

### "Test timeout" errors

1. Increase timeout in test: `test('...', async ({ page }) => { ... }, 60000)`
2. Increase global timeout in `playwright.e2e.config.ts`
3. Use `waitForPageReady()` instead of guessing delays

### CI-only failures

1. Check CI logs for daemon startup errors
2. Verify network timeouts are reasonable (CI might be slower)
3. Ensure retries are enabled (should be automatic on CI)
4. Review trace files uploaded as artifacts

## Contributing

When adding new E2E tests:

1. Follow patterns in `examples/stable-test-pattern.spec.ts`
2. Use helpers from `helpers.ts`
3. Add comments explaining complex user flows
4. Test locally AND in CI before merging
5. Update this README if introducing new patterns
