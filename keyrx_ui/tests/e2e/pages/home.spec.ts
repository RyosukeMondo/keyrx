/**
 * HomePage E2E Tests
 *
 * Tests the main dashboard page (/) to verify:
 * - Page loads without console errors
 * - Active profile card renders correctly
 * - Device list card renders correctly
 * - Quick stats card renders correctly
 * - No excessive or duplicate API requests
 *
 * These tests verify the complete integration of HomePage with
 * the daemon API and ensure efficient network usage.
 */

import { test, expect } from '../fixtures/daemon';
import { NetworkMonitor } from '../fixtures/network-monitor';

test.describe('HomePage', () => {
  test('should load without console errors', async ({ page }) => {
    // Capture console errors
    const consoleErrors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Navigate to home page
    await page.goto('/');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    // Verify no console errors
    expect(consoleErrors).toEqual([]);
  });

  test('should render dashboard heading', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check for main heading
    const heading = page.getByRole('heading', { name: /dashboard/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should render active profile card', async ({ page, daemon }) => {
    // Create a test profile to ensure there's an active profile
    const testProfileName = `e2e-home-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Activate the test profile
    await page.request.post(`${daemon.apiUrl}/api/profiles/${testProfileName}/activate`);

    // Navigate to home page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Wait for active profile card to load
    // Look for any heading or text that indicates the profile card
    const profileSection = page.locator('text=/active profile/i').first();
    await expect(profileSection).toBeVisible({ timeout: 10000 });

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should render device list card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Look for device list section
    // The card might show "No devices" or actual devices
    const deviceSection = page.locator('text=/device/i').first();
    await expect(deviceSection).toBeVisible({ timeout: 10000 });
  });

  test('should render quick stats card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Look for stats section
    // Stats might include latency, events, uptime, etc.
    const statsSection = page.locator('text=/stat|latency|event|uptime/i').first();
    await expect(statsSection).toBeVisible({ timeout: 10000 });
  });

  test('should not make excessive API requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // HomePage should make reasonable number of requests
    // Expected requests:
    // - GET /api/status (1x)
    // - GET /api/profiles (1-2x - initial load + possible refresh)
    // - GET /api/devices (1-2x)
    // - GET /api/metrics/latency (0-1x)
    // Total: ~3-6 requests is reasonable

    const totalRequests = monitor.getRequests().length;
    expect(totalRequests).toBeLessThanOrEqual(10); // Allow some headroom

    // Print summary for debugging
    if (process.env.DEBUG_NETWORK) {
      monitor.printSummary();
    }

    monitor.stop();
  });

  test('should not make duplicate requests within 100ms', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Verify no duplicate requests (catches rapid-fire bug patterns)
    monitor.assertNoDuplicateRequests();

    monitor.stop();
  });

  test('should handle daemon connection gracefully', async ({ page, daemon }) => {
    // Verify daemon is ready before test
    const isReady = await daemon.isReady();
    expect(isReady).toBe(true);

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Page should load without throwing errors
    // (If daemon is not available, page might show error state - that's OK)
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });

  test('should have accessible dashboard structure', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check for main landmark
    const main = page.getByRole('main');
    await expect(main).toBeVisible();

    // Check for aria-label on main
    const mainLabel = await main.getAttribute('aria-label');
    expect(mainLabel).toBeTruthy();

    // Check for region with dashboard overview
    const overview = page.getByRole('region', { name: /dashboard overview/i });
    await expect(overview).toBeVisible();
  });

  test('should render all dashboard cards within 5 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    // Wait for all cards to be present
    // Using waitFor with multiple locators
    await Promise.all([
      page.locator('text=/active profile|profile/i').first().waitFor({ timeout: 5000 }),
      page.locator('text=/device/i').first().waitFor({ timeout: 5000 }),
      page.locator('text=/stat|latency|event|uptime/i').first().waitFor({ timeout: 5000 }),
    ]);

    const loadTime = Date.now() - startTime;

    // Performance check: should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  test('should handle navigation from home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Verify we can navigate to other pages from home
    // This tests that the app structure is working

    // Look for navigation links (sidebar or bottom nav)
    const devicesLink = page.getByRole('link', { name: /device/i }).first();
    if (await devicesLink.isVisible()) {
      await devicesLink.click();

      // Wait for navigation
      await page.waitForURL(/\/devices/);

      // Verify we navigated away from home
      expect(page.url()).toContain('/devices');
    }
  });
});
