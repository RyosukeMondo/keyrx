/**
 * Network Request Efficiency Tests
 *
 * These tests systematically verify network efficiency across all pages:
 * - No excessive API requests on page load
 * - No duplicate requests to same endpoint within 100ms
 * - User actions don't trigger unexpected request patterns
 * - No more than 10 requests of the same type per page
 *
 * These tests are critical for catching performance bugs like:
 * - Rapid PATCH requests when dropdown values change
 * - Multiple requests fired on component mount
 * - Inefficient polling or data fetching
 * - Race conditions causing duplicate API calls
 *
 * Requirements covered:
 * - 4.1: Detect rapid duplicate requests to same endpoint
 * - 4.2: Verify page load makes expected number of requests
 * - 4.3: Verify user actions don't trigger unexpected requests
 * - 4.4: Log and fail on excessive API calls (>10 of same type per page)
 */

import { test, expect } from '../fixtures/daemon';
import { NetworkMonitor } from '../fixtures/network-monitor';

/**
 * Expected request counts per page (allows some tolerance)
 * Format: { endpoint: maxCount }
 */
const PAGE_REQUEST_LIMITS = {
  home: {
    '/api/status': 2,
    '/api/devices': 2,
    '/api/profiles': 2,
    '/api/profiles/active': 2,
    '/api/metrics/latency': 2,
    totalMax: 10,
  },
  devices: {
    '/api/devices': 3,
    '/api/settings/global-layout': 2,
    totalMax: 8,
  },
  profiles: {
    '/api/profiles': 3,
    '/api/profiles/active': 2,
    totalMax: 8,
  },
  config: {
    '/api/profiles': 2,
    '/api/profiles/active': 2,
    totalMax: 10,
  },
  metrics: {
    '/api/metrics/latency': 3,
    '/api/metrics/events': 2,
    totalMax: 10,
  },
  simulator: {
    '/api/profiles': 2,
    '/api/profiles/active': 2,
    totalMax: 8,
  },
};

test.describe('Network Request Efficiency - Page Loads', () => {
  test('HomePage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Wait a bit more to catch any delayed requests
    await page.waitForTimeout(500);

    // Assert specific endpoint limits
    const limits = PAGE_REQUEST_LIMITS.home;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    // Assert total request limit
    monitor.assertTotalRequests(limits.totalMax);

    // Print summary for debugging
    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('HomePage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // No duplicate requests within 100ms
    monitor.assertNoDuplicateRequests(100);
  });

  test('DevicesPage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const limits = PAGE_REQUEST_LIMITS.devices;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    monitor.assertTotalRequests(limits.totalMax);

    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('DevicesPage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    monitor.assertNoDuplicateRequests(100);
  });

  test('ProfilesPage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const limits = PAGE_REQUEST_LIMITS.profiles;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    monitor.assertTotalRequests(limits.totalMax);

    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('ProfilesPage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    monitor.assertNoDuplicateRequests(100);
  });

  test('ConfigPage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000); // Monaco editor takes time to load

    const limits = PAGE_REQUEST_LIMITS.config;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    monitor.assertTotalRequests(limits.totalMax);

    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('ConfigPage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    monitor.assertNoDuplicateRequests(100);
  });

  test('MetricsPage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const limits = PAGE_REQUEST_LIMITS.metrics;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    monitor.assertTotalRequests(limits.totalMax);

    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('MetricsPage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    monitor.assertNoDuplicateRequests(100);
  });

  test('SimulatorPage should not make excessive requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const limits = PAGE_REQUEST_LIMITS.simulator;
    for (const [endpoint, maxCount] of Object.entries(limits)) {
      if (endpoint === 'totalMax') continue;
      monitor.assertNoExcessiveRequests(endpoint, maxCount as number);
    }

    monitor.assertTotalRequests(limits.totalMax);

    if (process.env.DEBUG) {
      monitor.printSummary();
    }
  });

  test('SimulatorPage should not have duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    monitor.assertNoDuplicateRequests(100);
  });
});

test.describe('Network Request Efficiency - User Actions', () => {
  test('DevicesPage: changing global layout should send exactly ONE request', async ({
    page,
  }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Wait for page to fully load
    await page.waitForTimeout(500);

    // Reset monitor to only track requests from user action
    monitor.reset();

    // Find and click the global layout dropdown
    const layoutDropdown = page.locator('select, [role="combobox"]').first();
    await expect(layoutDropdown).toBeVisible({ timeout: 10000 });

    // Change the layout
    await layoutDropdown.selectOption({ index: 1 });

    // Wait for request to complete
    await page.waitForTimeout(500);

    // Should send exactly ONE PATCH request to /api/settings/global-layout
    // This is the critical test - catches the rapid PATCH bug
    const patchRequests = monitor
      .getRequests()
      .filter((r) => r.method === 'PATCH' && r.url.includes('/api/settings/global-layout'));

    expect(
      patchRequests.length,
      `Expected exactly 1 PATCH request to /api/settings/global-layout, got ${patchRequests.length}. ` +
        `This indicates the rapid request bug is present!`
    ).toBe(1);

    // Also verify no duplicate requests occurred
    monitor.assertNoDuplicateRequests(100);
  });

  test('DevicesPage: changing device layout should send exactly ONE request', async ({
    page,
  }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Find device-specific layout dropdown (not the global one)
    const deviceCards = page.locator('[data-testid*="device-card"]');
    const firstCard = deviceCards.first();

    // Wait for device cards to be visible
    await expect(firstCard).toBeVisible({ timeout: 10000 });

    // Find layout dropdown within the card
    const layoutDropdown = firstCard.locator('select, [role="combobox"]').first();

    // Skip if no dropdown (device might not have layout selector)
    const dropdownCount = await layoutDropdown.count();
    if (dropdownCount === 0) {
      console.log('Skipping test - no device layout dropdown found');
      return;
    }

    // Change the layout
    await layoutDropdown.selectOption({ index: 1 });
    await page.waitForTimeout(500);

    // Should send exactly ONE PATCH request to /api/devices/:id
    const patchRequests = monitor
      .getRequests()
      .filter((r) => r.method === 'PATCH' && r.url.includes('/api/devices/'));

    expect(
      patchRequests.length,
      `Expected exactly 1 PATCH request to /api/devices/:id, got ${patchRequests.length}`
    ).toBe(1);

    monitor.assertNoDuplicateRequests(100);
  });

  test('ProfilesPage: creating profile should not trigger excessive requests', async ({
    page,
    daemon,
  }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Click "New Profile" button
    const newProfileBtn = page.getByRole('button', { name: /new profile/i });
    await expect(newProfileBtn).toBeVisible({ timeout: 10000 });
    await newProfileBtn.click();

    // Wait for modal
    await page.waitForSelector('[role="dialog"]', { timeout: 5000 });

    // Fill in profile name
    const profileName = `e2e-efficiency-${Date.now()}`;
    const nameInput = page.getByLabel(/profile name/i);
    await nameInput.fill(profileName);

    // Click create
    const createBtn = page.getByRole('button', { name: /^create$/i });
    await createBtn.click();

    // Wait for profile to be created
    await page.waitForTimeout(1000);

    // Should make reasonable number of requests:
    // - POST /api/profiles (1x - create profile)
    // - GET /api/profiles (1-2x - refresh list)
    monitor.assertNoExcessiveRequests('/api/profiles', 3);
    monitor.assertTotalRequests(5);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);

    // Cleanup
    await daemon.deleteTestProfile(profileName);
  });

  test('ProfilesPage: activating profile should not trigger excessive requests', async ({
    page,
    daemon,
  }) => {
    // Create test profile
    const profileName = `e2e-activate-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;
    await daemon.createTestProfile(profileName, testConfig);

    const monitor = new NetworkMonitor(page);

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Find and click activate button for test profile
    const profileRow = page.locator(`text="${profileName}"`).locator('..').locator('..');
    const activateBtn = profileRow.getByRole('button', { name: /activate/i });
    await expect(activateBtn).toBeVisible({ timeout: 10000 });
    await activateBtn.click();

    // Wait for activation
    await page.waitForTimeout(1000);

    // Should make reasonable requests:
    // - POST /api/profiles/:name/activate (1x)
    // - GET /api/profiles (0-2x - refresh)
    // - GET /api/profiles/active (0-2x - refresh active)
    monitor.assertTotalRequests(6);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);

    // Cleanup
    await daemon.deleteTestProfile(profileName);
  });

  test('ConfigPage: typing in editor should not trigger API requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/config');
    await page.waitForLoadState('networkidle');

    // Wait for Monaco editor to load
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Reset monitor
    monitor.reset();

    // Type in editor
    await page.keyboard.type('// test comment');

    // Wait a bit
    await page.waitForTimeout(1000);

    // Should NOT make any API requests while typing
    const requests = monitor.getRequests();
    expect(
      requests.length,
      `Expected 0 API requests while typing, got ${requests.length}. ` +
        `Typing should not trigger API calls!`
    ).toBe(0);
  });

  test('ConfigPage: saving config should not trigger excessive requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/config');
    await page.waitForLoadState('networkidle');

    // Wait for Monaco editor
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Make a small edit
    await page.keyboard.type('\n// test edit');

    // Reset monitor
    monitor.reset();

    // Click save button
    const saveBtn = page.getByRole('button', { name: /save/i });
    await expect(saveBtn).toBeVisible();
    await saveBtn.click();

    // Wait for save to complete
    await page.waitForTimeout(1000);

    // Should make reasonable requests:
    // - PATCH /api/profiles/:name/config (1x - save config)
    // - GET /api/profiles (0-1x - optional refresh)
    monitor.assertTotalRequests(3);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);
  });

  test('SimulatorPage: simulating key should send exactly ONE request', async ({ page }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Find key input
    const keyInput = page.getByLabel(/key/i).or(page.locator('input[type="text"]')).first();
    await expect(keyInput).toBeVisible({ timeout: 10000 });

    // Enter a key
    await keyInput.fill('a');

    // Find and click simulate button
    const simulateBtn = page.getByRole('button', { name: /simulate/i });
    await expect(simulateBtn).toBeVisible();
    await simulateBtn.click();

    // Wait for simulation
    await page.waitForTimeout(500);

    // Should send exactly ONE request to simulator API
    // (endpoint may vary - POST /api/simulator or similar)
    const postRequests = monitor.getRequests().filter((r) => r.method === 'POST');

    expect(
      postRequests.length,
      `Expected exactly 1 POST request for simulation, got ${postRequests.length}`
    ).toBe(1);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);
  });

  test('MetricsPage: clearing events should send exactly ONE request', async ({ page }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Find clear button (if exists)
    const clearBtn = page.getByRole('button', { name: /clear/i });
    const clearBtnCount = await clearBtn.count();

    if (clearBtnCount === 0) {
      console.log('Skipping test - no clear button found');
      return;
    }

    await expect(clearBtn).toBeVisible();
    await clearBtn.click();

    // Wait for clear to complete
    await page.waitForTimeout(500);

    // Should send exactly ONE DELETE request
    const deleteRequests = monitor.getRequests().filter((r) => r.method === 'DELETE');

    expect(
      deleteRequests.length,
      `Expected exactly 1 DELETE request for clearing events, got ${deleteRequests.length}`
    ).toBe(1);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);
  });
});

test.describe('Network Request Efficiency - Navigation', () => {
  test('navigating between pages should not cause request storms', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Navigate through all pages
    const pages = ['/', '/devices', '/profiles', '/config', '/metrics', '/simulator'];

    for (const pagePath of pages) {
      await page.goto(pagePath);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(500);
    }

    // After visiting 6 pages, total requests should be reasonable
    // Expect ~5-10 requests per page = 30-60 total
    const totalRequests = monitor.getRequests().length;

    expect(
      totalRequests,
      `Total requests (${totalRequests}) exceeds reasonable limit for 6 pages. ` +
        `Expected ≤80 requests. This indicates inefficient data fetching.`
    ).toBeLessThanOrEqual(80);

    // No duplicate requests across all navigation
    monitor.assertNoDuplicateRequests(100);
  });

  test('rapidly switching pages should not cause duplicate requests', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Rapidly switch between two pages
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    await page.waitForTimeout(500);

    // Should not have any duplicate requests within 100ms
    monitor.assertNoDuplicateRequests(100);

    // Total requests should be reasonable
    monitor.assertTotalRequests(30);
  });

  test('browser back/forward should not cause excessive requests', async ({ page }) => {
    // Visit multiple pages
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Start monitoring
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Go back
    await page.goBack();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Go forward
    await page.goForward();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Back/forward should make minimal requests (cached data)
    // Allow some requests for state refresh
    monitor.assertTotalRequests(15);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);
  });
});

test.describe('Network Request Efficiency - Edge Cases', () => {
  test('rapid page reloads should not cause cumulative request buildup', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Reload the same page multiple times
    for (let i = 0; i < 3; i++) {
      await page.goto('/devices');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(300);
    }

    // Total requests should scale linearly, not exponentially
    // 3 loads × ~5 requests = ~15 requests expected
    monitor.assertTotalRequests(25);

    // No duplicate requests
    monitor.assertNoDuplicateRequests(100);
  });

  test('concurrent user actions should not cause request storms', async ({ page }) => {
    const monitor = new NetworkMonitor(page);

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Reset monitor
    monitor.reset();

    // Perform multiple actions rapidly (simulating fast user)
    // This tests that UI properly debounces/throttles requests

    // Find dropdowns
    const dropdowns = page.locator('select, [role="combobox"]');
    const count = await dropdowns.count();

    if (count > 0) {
      // Change first dropdown multiple times rapidly
      const firstDropdown = dropdowns.first();
      await firstDropdown.selectOption({ index: 1 });
      await page.waitForTimeout(50); // Very short wait
      await firstDropdown.selectOption({ index: 0 });
      await page.waitForTimeout(50);
      await firstDropdown.selectOption({ index: 1 });

      // Wait for all requests to complete
      await page.waitForTimeout(1000);

      // Should not make excessive requests due to rapid changes
      // Proper implementation should debounce or only send last value
      monitor.assertTotalRequests(5);
    }
  });

  test('no API endpoint should receive more than 10 requests per page', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Load a page
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Check that no single endpoint was called more than 10 times
    const summary = monitor.getSummary();
    const violations: string[] = [];

    for (const [endpoint, count] of Object.entries(summary)) {
      if (count > 10) {
        violations.push(`${endpoint}: ${count} requests`);
      }
    }

    expect(
      violations.length,
      `Found ${violations.length} endpoint(s) with more than 10 requests:\n` +
        violations.join('\n') +
        '\n\nThis indicates a serious efficiency bug!'
    ).toBe(0);
  });

  test('WebSocket connections should not cause REST API request spam', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    // Load page with WebSocket connection
    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');

    // Wait for WebSocket to connect and send some messages
    await page.waitForTimeout(2000);

    // WebSocket messages should NOT trigger REST API requests
    // Metrics page should make initial load requests only
    const limits = PAGE_REQUEST_LIMITS.metrics;
    monitor.assertTotalRequests(limits.totalMax);
  });
});
