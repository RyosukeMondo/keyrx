/**
 * E2E Test Config Editor Mocks
 *
 * Provides mock responses for config editor E2E tests.
 */

import type { Page, Route } from '@playwright/test';

export const defaultConfigContent = `// Configuration for profile
// Example: Simple key remapping
// map("A", "B");

// Add your key mappings here...
`;

/**
 * Setup config editor API mocks
 */
export async function setupConfigMocks(
  page: Page,
  options: {
    profileName?: string;
    configContent?: string;
    failOnSave?: boolean;
    validationErrors?: Array<{ line: number; column: number; message: string }>;
  } = {}
): Promise<void> {
  const profileName = options.profileName ?? 'default';
  let configContent = options.configContent ?? defaultConfigContent;

  // GET /api/profiles/:name/config
  await page.route('**/api/profiles/*/config', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          source: configContent,
          profile: profileName,
        }),
      });
    } else if (route.request().method() === 'PUT') {
      if (options.failOnSave) {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Failed to save configuration' }),
        });
        return;
      }

      const body = route.request().postDataJSON();
      if (body.source) {
        configContent = body.source;
      }

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true }),
      });
    } else {
      await route.continue();
    }
  });

  // POST /api/profiles/validate
  await page.route('**/api/profiles/validate', async (route: Route) => {
    const errors = options.validationErrors ?? [];
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        valid: errors.length === 0,
        errors,
      }),
    });
  });

  // GET /api/profiles/:name/validate
  await page.route('**/api/profiles/*/validate', async (route: Route) => {
    const errors = options.validationErrors ?? [];
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        valid: errors.length === 0,
        errors,
      }),
    });
  });
}
