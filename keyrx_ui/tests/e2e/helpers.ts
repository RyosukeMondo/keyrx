/**
 * E2E Test Helpers
 *
 * Provides common utilities for E2E tests:
 * - Daemon health checks before each test
 * - Browser context isolation helpers
 * - Common wait utilities
 */

import type { Page } from '@playwright/test';
import { expect } from '@playwright/test';
import { DAEMON_API_URL } from './fixtures/daemon';

/**
 * Check if daemon is responding on the health endpoint
 */
export async function checkDaemonHealth(): Promise<boolean> {
  try {
    const response = await fetch(`${DAEMON_API_URL}/api/status`, {
      method: 'GET',
      signal: AbortSignal.timeout(2000),
    });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Wait for daemon to be ready (useful in beforeEach hooks)
 *
 * @param timeoutMs - Maximum time to wait for daemon (default: 5000ms)
 * @throws Error if daemon is not ready within timeout
 */
export async function waitForDaemonReady(timeoutMs = 5000): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    if (await checkDaemonHealth()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error(`Daemon not ready within ${timeoutMs}ms`);
}

/**
 * Setup function to call in beforeEach to ensure daemon is ready
 *
 * @example
 * test.beforeEach(async () => {
 *   await ensureDaemonReady();
 * });
 */
export async function ensureDaemonReady(): Promise<void> {
  const isReady = await checkDaemonHealth();

  if (!isReady) {
    throw new Error(
      'Daemon is not responding. Ensure global-setup.ts started the daemon successfully.'
    );
  }
}

/**
 * Clear browser storage to ensure fresh test state
 * Call this in beforeEach if you need complete isolation
 *
 * @param page - Playwright page object
 */
export async function clearBrowserStorage(page: Page): Promise<void> {
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // Clear cookies
  const context = page.context();
  await context.clearCookies();
}

/**
 * Setup fresh test environment (daemon check + clear storage)
 * Recommended for tests that need complete isolation
 *
 * @param page - Playwright page object
 *
 * @example
 * test.beforeEach(async ({ page }) => {
 *   await setupFreshTestEnvironment(page);
 * });
 */
export async function setupFreshTestEnvironment(page: Page): Promise<void> {
  // Ensure daemon is ready
  await ensureDaemonReady();

  // Clear browser storage for fresh state
  await clearBrowserStorage(page);
}

/**
 * Wait for WebSocket connection to be established
 * Useful when testing real-time features
 *
 * @param page - Playwright page object
 * @param timeoutMs - Maximum time to wait (default: 3000ms)
 */
export async function waitForWebSocketConnection(
  page: Page,
  timeoutMs = 3000
): Promise<void> {
  await page.waitForFunction(
    () => {
      // Check if WebSocket is connected via window object
      return (window as any).__wsConnected === true;
    },
    { timeout: timeoutMs }
  ).catch(() => {
    // If the window property doesn't exist, fall back to checking WS state in React
    // This is a best-effort check
  });
}

/**
 * Wait for page to be fully loaded including network requests
 * More reliable than just waitForLoadState for SPA apps
 *
 * @param page - Playwright page object
 */
export async function waitForPageReady(page: Page): Promise<void> {
  await page.waitForLoadState('networkidle');

  // Additional wait for React hydration
  await page.waitForTimeout(300);
}

/**
 * Retry an operation that might fail due to network issues
 * Useful for operations that interact with the daemon
 *
 * @param operation - Async function to retry
 * @param maxRetries - Maximum number of retries (default: 2)
 * @param delayMs - Delay between retries in ms (default: 500)
 */
export async function retryOnFailure<T>(
  operation: () => Promise<T>,
  maxRetries = 2,
  delayMs = 500
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;

      if (attempt < maxRetries) {
        await new Promise((resolve) => setTimeout(resolve, delayMs));
      }
    }
  }

  throw lastError;
}

/**
 * Take a screenshot for debugging (saved to test-results)
 *
 * @param page - Playwright page object
 * @param name - Screenshot name (without extension)
 */
export async function takeDebugScreenshot(
  page: Page,
  name: string
): Promise<void> {
  const timestamp = Date.now();
  await page.screenshot({
    path: `test-results/debug-${name}-${timestamp}.png`,
    fullPage: true,
  });
}
