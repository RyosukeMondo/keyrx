import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Configuration
 *
 * This configuration is specifically for end-to-end testing against a live
 * keyrx_daemon instance. Unlike the standard config, this:
 * - Runs tests sequentially (workers: 1) due to shared daemon state
 * - Tests against both UI (localhost:5173) and daemon (localhost:9867)
 * - Uses global setup/teardown to manage daemon lifecycle
 *
 * See https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests/e2e',

  /* Run tests sequentially - daemon state is shared across tests */
  fullyParallel: false,
  workers: 1,

  /* Fail the build on CI if you accidentally left test.only in the source code */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only - 1 retry for flaky tests */
  retries: process.env.CI ? 1 : 0,

  /* Reporter to use */
  reporter: [
    ['html', { outputFolder: 'playwright-report-e2e' }],
    ['json', { outputFile: 'test-results/e2e-results.json' }],
    ['list'],
  ],

  /* Global setup/teardown for daemon lifecycle */
  globalSetup: './tests/e2e/global-setup.ts',
  globalTeardown: './tests/e2e/global-teardown.ts',

  /* Shared settings for all the projects below */
  use: {
    /* Base URL to use in actions like `await page.goto('/')` */
    baseURL: 'http://localhost:5173',

    /* Collect trace on failure for debugging */
    trace: 'retain-on-failure',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',

    /* Video on failure */
    video: 'retain-on-failure',

    /* Extra context for API testing */
    extraHTTPHeaders: {
      'Accept': 'application/json',
    },
  },

  /* Configure snapshot comparison */
  expect: {
    toMatchSnapshot: {
      /* Allow small pixel differences (font rendering, anti-aliasing) */
      maxDiffPixels: 100,
    },
    /* Longer timeout for E2E tests that interact with daemon */
    timeout: 10000,
  },

  /* Configure projects - chromium only for E2E to reduce test time */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  /* Run Vite dev server before starting the tests */
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  /* Global timeout for entire test run */
  timeout: 30 * 1000,

  /* Output directory for test artifacts */
  outputDir: 'test-results/e2e',
});
