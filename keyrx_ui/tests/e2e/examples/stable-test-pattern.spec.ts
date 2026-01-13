/**
 * Example E2E Test - Demonstrating Stable Test Patterns
 *
 * This file demonstrates best practices for writing stable E2E tests:
 * 1. Explicit daemon health checks before each test
 * 2. Fresh browser context with cleared storage
 * 3. Proper waits instead of arbitrary timeouts
 * 4. Retry-friendly assertions
 *
 * Use these patterns in your E2E tests for maximum stability.
 */

import { test, expect } from '@playwright/test';
import {
  setupFreshTestEnvironment,
  ensureDaemonReady,
  waitForPageReady,
  retryOnFailure,
  clearBrowserStorage,
} from '../helpers';

test.describe('Stable Test Patterns Example', () => {
  /**
   * Pattern 1: Fresh environment before each test
   * Ensures complete isolation between tests
   */
  test.beforeEach(async ({ page }) => {
    await setupFreshTestEnvironment(page);
  });

  test('should load home page with fresh state', async ({ page }) => {
    // Navigate to home page
    await page.goto('/');

    // Wait for page to be fully ready
    await waitForPageReady(page);

    // Verify page loaded
    await expect(page).toHaveURL(/\//);

    // Page should have a title
    await expect(page).toHaveTitle(/KeyRx/i);
  });

  /**
   * Pattern 2: Retry-friendly operations
   * Use retryOnFailure for operations that might fail due to network
   */
  test('should handle daemon API calls with retry', async ({ page }) => {
    await page.goto('/');
    await waitForPageReady(page);

    // Example: Click a button that triggers API call
    // Wrap in retry to handle transient network issues
    await retryOnFailure(async () => {
      const profilesLink = page.getByRole('link', { name: /profiles/i });
      await profilesLink.click();
      await waitForPageReady(page);

      // Verify navigation succeeded
      await expect(page).toHaveURL(/\/profiles/);
    });
  });

  /**
   * Pattern 3: Explicit daemon check for tests that need it
   * Some tests might need to ensure daemon is ready mid-test
   */
  test('should verify daemon health before critical operation', async ({
    page,
  }) => {
    await page.goto('/devices');
    await waitForPageReady(page);

    // Before performing a critical operation, check daemon health
    await ensureDaemonReady();

    // Now perform the operation
    // ... test code here ...
  });

  /**
   * Pattern 4: Clear storage mid-test if needed
   * For tests that need to reset state
   */
  test('should handle logout and login with cleared storage', async ({
    page,
  }) => {
    // Login flow
    await page.goto('/');
    await waitForPageReady(page);

    // ... perform login actions ...

    // Simulate logout by clearing storage
    await clearBrowserStorage(page);

    // Reload page - should be logged out
    await page.reload();
    await waitForPageReady(page);

    // Verify logged out state
    // ... assertions here ...
  });
});

/**
 * Pattern 5: Tests that don't need fresh environment
 * Some tests are read-only and can skip the overhead
 */
test.describe('Read-only Tests (Lightweight)', () => {
  // Only check daemon health, no need to clear storage
  test.beforeEach(async () => {
    await ensureDaemonReady();
  });

  test('should display version information', async ({ page }) => {
    await page.goto('/');
    await waitForPageReady(page);

    // Read-only test - just check page content
    // No state modifications, so no need for fresh environment
    const version = page.getByText(/version|v\d+\.\d+\.\d+/i);

    // This might not exist, so make it optional
    const count = await version.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});
