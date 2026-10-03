import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright against a RUNNING keyrx_daemon (the UI it embeds), not the Vite
 * dev server. Point it at a scratch daemon, never your real one:
 *
 *   KEYRX_PORT=19872 KEYRX_CONFIG_DIR=$(mktemp -d) XDG_RUNTIME_DIR=$(mktemp -d) \
 *     KEYRX_DEVICE_SCOPE=none keyrx_daemon run &
 *   KEYRX_E2E_URL=http://127.0.0.1:19872 npx playwright test --config=playwright.daemon.config.ts
 *
 * KEYRX_E2E_CHROME=/usr/bin/google-chrome uses a system Chrome instead of the
 * Playwright-managed browser.
 */
const baseURL = process.env.KEYRX_E2E_URL ?? 'http://127.0.0.1:9867';

export default defineConfig({
  testDir: './e2e-daemon',
  fullyParallel: false,
  workers: 1,
  reporter: [['list']],
  use: { baseURL, trace: 'retain-on-failure' },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: process.env.KEYRX_E2E_CHROME
          ? { executablePath: process.env.KEYRX_E2E_CHROME }
          : {},
      },
    },
  ],
});
