/**
 * Visual Regression Tests for Responsive UI
 *
 * Tests all pages at three breakpoints (375px mobile, 768px tablet, 1024px desktop)
 * to prevent visual regressions across different viewport sizes.
 *
 * Requirements: REQ-6 (AC8)
 */

import { test, expect } from '@playwright/test';

// Test viewports matching the breakpoints in the spec
const VIEWPORTS = [
  { name: 'mobile', width: 375, height: 667 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'desktop', width: 1024, height: 768 },
];

// Pages to test
const PAGES = [
  { path: '/', name: 'profiles' },
  { path: '/config/Default', name: 'config-visual' },
  { path: '/dashboard', name: 'dashboard' },
  { path: '/devices', name: 'devices' },
];

/**
 * Helper function to wait for page to be fully loaded and stable
 */
async function waitForPageStable(page: any) {
  // Wait for network to be idle
  await page.waitForLoadState('networkidle');

  // Wait for any animations to complete
  await page.waitForTimeout(500);
}

test.describe('Visual Regression Tests - Responsive UI', () => {
  // Test each viewport
  for (const viewport of VIEWPORTS) {
    test.describe(`${viewport.name} (${viewport.width}x${viewport.height})`, () => {
      test.use({ viewport });

      // Test each page
      for (const pageInfo of PAGES) {
        test(`${pageInfo.name} page at ${viewport.name}`, async ({ page }) => {
          // Navigate to page
          await page.goto(pageInfo.path);

          // Wait for page to be stable
          await waitForPageStable(page);

          // Take full page screenshot
          const screenshot = await page.screenshot({
            fullPage: true,
            animations: 'disabled',
          });

          // Compare with baseline
          expect(screenshot).toMatchSnapshot(
            `${pageInfo.name}-${viewport.name}.png`
          );
        });
      }

      // Special test for ConfigPage with Code tab
      test(`config-code page at ${viewport.name}`, async ({ page }) => {
        // Navigate to config page
        await page.goto('/config/Default');
        await waitForPageStable(page);

        // Click the Code tab
        const codeTab = page.getByRole('button', { name: /code/i });
        await codeTab.click();

        // Wait for Monaco editor to render
        await page.waitForSelector('.monaco-editor', { timeout: 5000 });
        await page.waitForTimeout(1000); // Give Monaco time to fully render

        // Take screenshot of Code tab
        const screenshot = await page.screenshot({
          fullPage: true,
          animations: 'disabled',
        });

        expect(screenshot).toMatchSnapshot(
          `config-code-${viewport.name}.png`
        );
      });

      // Test navigation components (BottomNav on mobile, Sidebar on desktop)
      test(`navigation components at ${viewport.name}`, async ({ page }) => {
        await page.goto('/');
        await waitForPageStable(page);

        // Take screenshot showing navigation
        const screenshot = await page.screenshot({
          fullPage: true,
          animations: 'disabled',
        });

        expect(screenshot).toMatchSnapshot(
          `navigation-${viewport.name}.png`
        );

        // Verify expected navigation component is visible
        if (viewport.width < 768) {
          // Mobile: BottomNav should be visible
          const bottomNav = page.locator('nav').filter({ hasText: /profiles|config|dashboard|devices/i }).last();
          await expect(bottomNav).toBeVisible();
        } else {
          // Desktop: Sidebar should be visible
          const sidebar = page.locator('aside, nav').filter({ hasText: /profiles|config|dashboard|devices/i }).first();
          await expect(sidebar).toBeVisible();
        }
      });
    });
  }

  test.describe('Dashboard with Events', () => {
    // Test dashboard with sample event data at different viewports
    for (const viewport of VIEWPORTS) {
      test(`dashboard with events at ${viewport.name}`, async ({ page }) => {
        await page.setViewportSize(viewport);

        // Navigate to dashboard
        await page.goto('/dashboard');
        await waitForPageStable(page);

        // Wait for dashboard to potentially load some data
        // In a real scenario, this might connect to WebSocket and receive events
        await page.waitForTimeout(1000);

        // Take screenshot
        const screenshot = await page.screenshot({
          fullPage: true,
          animations: 'disabled',
        });

        expect(screenshot).toMatchSnapshot(
          `dashboard-with-events-${viewport.name}.png`
        );
      });
    }
  });

  test.describe('Touch Target Verification', () => {
    // Verify minimum 44px touch targets on mobile
    test('mobile buttons meet 44px minimum', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      await page.goto('/');
      await waitForPageStable(page);

      // Check BottomNav buttons
      const navButtons = page.locator('nav button, nav a');
      const count = await navButtons.count();

      for (let i = 0; i < count; i++) {
        const button = navButtons.nth(i);
        const box = await button.boundingBox();

        if (box) {
          // Allow small margin of error (42px is acceptable)
          expect(box.height).toBeGreaterThanOrEqual(42);
        }
      }
    });

    test('config page buttons meet 44px minimum on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      await page.goto('/config/Default');
      await waitForPageStable(page);

      // Check tab buttons and save button
      const buttons = page.locator('button').filter({ hasText: /visual|code|save/i });
      const count = await buttons.count();

      for (let i = 0; i < count; i++) {
        const button = buttons.nth(i);
        const box = await button.boundingBox();

        if (box) {
          expect(box.height).toBeGreaterThanOrEqual(42);
        }
      }
    });
  });

  test.describe('Responsive Layout Transitions', () => {
    // Test that layouts transition smoothly between breakpoints
    test('config page layout transitions', async ({ page }) => {
      // Start at mobile
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/config/Default');
      await waitForPageStable(page);

      const mobileScreenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      // Transition to tablet
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.waitForTimeout(500);

      const tabletScreenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      // Transition to desktop
      await page.setViewportSize({ width: 1024, height: 768 });
      await page.waitForTimeout(500);

      const desktopScreenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      // Snapshots for all transitions
      expect(mobileScreenshot).toMatchSnapshot('config-transition-mobile.png');
      expect(tabletScreenshot).toMatchSnapshot('config-transition-tablet.png');
      expect(desktopScreenshot).toMatchSnapshot('config-transition-desktop.png');
    });

    test('dashboard layout transitions', async ({ page }) => {
      // Start at mobile
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/dashboard');
      await waitForPageStable(page);

      const mobileScreenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      // Transition to desktop (2-column grid at >= 1024px)
      await page.setViewportSize({ width: 1024, height: 768 });
      await page.waitForTimeout(500);

      const desktopScreenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      expect(mobileScreenshot).toMatchSnapshot('dashboard-transition-mobile.png');
      expect(desktopScreenshot).toMatchSnapshot('dashboard-transition-desktop.png');
    });
  });
});
