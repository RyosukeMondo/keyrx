import { test, expect, type Page } from '@playwright/test';

/**
 * Switching tabs must never produce a failed request, an HTTP error or a
 * console error/warning, at desktop and phone width (BottomNav). Also guards
 * the "Failed to fetch" a user saw after the daemon was upgraded under an
 * open tab (stale hashed page chunk).
 */

const VIEWPORTS = [
  { name: 'desktop', width: 1280, height: 800 },
  { name: 'phone', width: 390, height: 800 },
] as const;

const ROUTES = ['/', '/devices', '/monitor'] as const;

function watch(page: Page): string[] {
  const problems: string[] = [];
  page.on('console', (m) => {
    if (m.type() === 'error' || m.type() === 'warning') {
      problems.push(`console.${m.type()}: ${m.text().slice(0, 300)}`);
    }
  });
  page.on('pageerror', (e) => problems.push(`pageerror: ${e.message}`));
  page.on('requestfailed', (r) =>
    problems.push(`request failed: ${r.method()} ${r.url()} ${r.failure()?.errorText}`)
  );
  page.on('response', (r) => {
    if (r.status() >= 400) {
      problems.push(`HTTP ${r.status()}: ${r.request().method()} ${r.url()}`);
    }
  });
  return problems;
}

async function expectNoFailure(page: Page) {
  await expect(page.getByText(/failed to fetch/i)).toHaveCount(0);
  await expect(page.getByText(/something went wrong/i)).toHaveCount(0);
}

for (const viewport of VIEWPORTS) {
  test.describe(`tabs at ${viewport.name} width`, () => {
    test.use({ viewport: { width: viewport.width, height: viewport.height } });

    test('every tab loads without a failed request or console error', async ({ page }) => {
      const problems = watch(page);
      await page.goto('/');
      await page.waitForLoadState('networkidle');

      for (const route of [...ROUTES, '/'] as const) {
        const link = page.locator(`nav a[href="${route}"]:visible`).first();
        await link.click();
        await page.waitForLoadState('networkidle');
        await expect(page).toHaveURL(new RegExp(`${route === '/' ? '/$' : route}$`));
        await expectNoFailure(page);
      }
      expect(problems).toEqual([]);
    });

    test('every in-page tab (role=tab) works', async ({ page }) => {
      const problems = watch(page);
      for (const route of ROUTES) {
        await page.goto(route);
        await page.waitForLoadState('networkidle');
        const tabs = page.locator('[role=tab]:visible');
        for (let i = 0; i < (await tabs.count()); i++) {
          await tabs.nth(i).click();
          await page.waitForLoadState('networkidle');
          await expectNoFailure(page);
        }
      }
      expect(problems).toEqual([]);
    });

    test('a page chunk renamed by an upgrade recovers by reloading once', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');

      // The old tab asks for a chunk the new daemon no longer has: 404 once.
      let served404 = 0;
      await page.route('**/assets/DevicesPage-*.js', (route) => {
        if (served404++ === 0) return route.fulfill({ status: 404, body: 'Not Found' });
        return route.continue();
      });
      const navigations: string[] = [];
      page.on('framenavigated', (f) => f === page.mainFrame() && navigations.push(f.url()));

      await page.locator('nav a[href="/devices"]:visible').first().click();
      await expect(page).toHaveURL(/\/devices$/);
      await page.waitForLoadState('networkidle');
      await expectNoFailure(page);
      expect(served404).toBeGreaterThanOrEqual(1);
      expect(navigations.length).toBeGreaterThanOrEqual(1); // it reloaded
    });
  });
}

test('the daemon answers a missing asset with 404, not the app shell', async ({ request }) => {
  const missing = await request.get('/assets/DevicesPage-OLDHASH.js');
  expect(missing.status()).toBe(404);
  const route = await request.get('/devices');
  expect(route.status()).toBe(200);
  expect(route.headers()['content-type']).toContain('text/html');
});
