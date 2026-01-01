# End-to-End Tests

This directory contains Playwright-based end-to-end tests for the KeyRX UI.

## Overview

The E2E tests verify complete user workflows in real browsers (Chromium, Firefox, WebKit) against a running daemon instance.

## Test Files

- **profile-crud.spec.ts**: Tests profile management operations
  - Creating profiles
  - Activating profiles
  - Renaming profiles
  - Duplicating profiles
  - Deleting profiles
  - Profile validation
  - Workflow scenarios

- **config-editor.spec.ts**: Tests the configuration editor
  - Tab switching (Visual/Code)
  - Monaco editor functionality
  - Syntax validation
  - Error navigation (F8)
  - Saving configurations (Ctrl+S)
  - Undo/redo
  - Large configurations

- **dashboard-monitoring.spec.ts**: Tests real-time dashboard features
  - WebSocket connection status
  - Daemon state updates
  - Latency metrics visualization
  - Event timeline
  - Pause/resume/clear events
  - Responsive layouts
  - Reconnection handling

## Running Tests

### All E2E Tests
```bash
npm run test:e2e
```

### Interactive UI Mode
```bash
npm run test:e2e:ui
```

### Headed Mode (visible browser)
```bash
npm run test:e2e:headed
```

### Debug Mode
```bash
npm run test:e2e:debug
```

### Specific Test File
```bash
npx playwright test e2e/profile-crud.spec.ts
```

### Specific Test
```bash
npx playwright test e2e/profile-crud.spec.ts -g "should create a new profile"
```

### Specific Browser
```bash
npx playwright test --project=chromium
npx playwright test --project=firefox
npx playwright test --project=webkit
```

## Configuration

E2E tests are configured in `playwright.config.ts`:

- **Base URL**: `http://localhost:9867` (daemon with embedded UI)
- **Web Server**: Automatically starts daemon before tests
- **Browsers**: Chromium, Firefox, WebKit, plus mobile viewports
- **Retries**: 2 retries on CI, 0 locally
- **Artifacts**: Screenshots and videos on failure

## Prerequisites

1. **Build the UI**:
   ```bash
   npm run build
   ```

2. **Daemon must be buildable**:
   ```bash
   cd .. && cargo build -p keyrx_daemon
   ```

3. **Playwright browsers installed**:
   ```bash
   npx playwright install
   ```

## Test Structure

Each test file follows this pattern:

```typescript
test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to page
    await page.goto('/path');
    await page.waitForLoadState('networkidle');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    // Act
    // Assert
  });
});
```

## Best Practices

1. **Use data-testid attributes**: For stable selectors
   ```typescript
   await page.locator('[data-testid="connection-banner"]')
   ```

2. **Wait for conditions**: Don't use arbitrary timeouts
   ```typescript
   await expect(element).toBeVisible({ timeout: 5000 });
   ```

3. **Clean up test data**: Delete created profiles/configs
   ```typescript
   await page.click(`button[aria-label="Delete ${testProfile}"]`);
   ```

4. **Test in isolation**: Each test should be independent
   - Don't rely on previous test state
   - Create test data in beforeEach or within test

5. **Use meaningful assertions**:
   ```typescript
   await expect(page.locator('text=Error')).toBeVisible();
   ```

## Debugging

### View test results:
```bash
npx playwright show-report
```

### Debug specific test:
```bash
npx playwright test --debug e2e/profile-crud.spec.ts
```

### Visual trace viewer:
```bash
npx playwright show-trace test-results/trace.zip
```

## CI/CD Integration

Tests run in CI with:
- Headless browsers
- 2 retries on failure
- Screenshot/video capture on failure
- Single worker (no parallelization)

## Troubleshooting

### Daemon fails to start
- Ensure daemon builds successfully: `cargo build -p keyrx_daemon`
- Check daemon logs for errors
- Verify port 9867 is not in use

### Tests timeout
- Increase timeout in individual tests
- Check network connectivity
- Verify daemon is responding at `http://localhost:9867`

### Flaky tests
- Add explicit waits for dynamic content
- Use `waitForLoadState('networkidle')`
- Increase timeout for slow operations

### Screenshots/videos not captured
- Check `playwright.config.ts` settings
- Ensure test is actually failing
- Check `test-results/` directory

## Requirements Verification

These E2E tests verify:
- **REQ-7 (AC4)**: End-to-end testing with real browser
- **REQ-7 (AC10)**: All tests run in CI/CD pipeline

All acceptance criteria are verified through automated E2E tests.
