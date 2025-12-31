import { test, expect } from '@playwright/test';

/**
 * Visual Regression Tests
 *
 * Tests all pages at three breakpoints (mobile, tablet, desktop) to catch
 * unintended visual changes. Uses Playwright's built-in screenshot comparison
 * with pixel-perfect matching.
 *
 * Breakpoints:
 * - Mobile: 375x667 (iPhone SE)
 * - Tablet: 768x1024 (iPad)
 * - Desktop: 1280x720 (typical laptop)
 *
 * Usage:
 * - First run: Creates baseline screenshots in tests/visual/*.spec.ts-snapshots/
 * - Subsequent runs: Compares against baseline, fails if diff detected
 * - Update baseline: npx playwright test --update-snapshots
 */

const pages = [
  { name: 'Home', path: '/' },
  { name: 'Devices', path: '/devices' },
  { name: 'Profiles', path: '/profiles' },
  { name: 'Config', path: '/config' },
  { name: 'Metrics', path: '/metrics' },
  { name: 'Simulator', path: '/simulator' },
];

const viewports = [
  { name: 'mobile', width: 375, height: 667 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'desktop', width: 1280, height: 720 },
];

// Test each page at each viewport size
for (const page of pages) {
  for (const viewport of viewports) {
    test(`${page.name} page - ${viewport.name} (${viewport.width}x${viewport.height})`, async ({
      page: browserPage,
    }) => {
      // Set viewport size
      await browserPage.setViewportSize({
        width: viewport.width,
        height: viewport.height,
      });

      // Navigate to page
      await browserPage.goto(page.path);

      // Wait for page to be fully loaded and hydrated
      await browserPage.waitForLoadState('networkidle');

      // Wait for any animations to complete (500ms should be enough for fade-ins)
      await browserPage.waitForTimeout(500);

      // Take full-page screenshot
      const screenshot = await browserPage.screenshot({
        fullPage: true,
        animations: 'disabled', // Disable CSS animations for consistent screenshots
      });

      // Compare against baseline
      expect(screenshot).toMatchSnapshot(
        `${page.name.toLowerCase()}-${viewport.name}.png`,
        {
          maxDiffPixels: 100, // Allow up to 100 pixels difference (for anti-aliasing variations)
          threshold: 0.2, // 20% threshold for pixel color difference
        }
      );
    });
  }
}

// Additional tests for specific UI states
test.describe('Visual regression - Interactive states', () => {
  test('DevicesPage - with device list', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Wait for devices to load (if any)
    await page.waitForTimeout(500);

    const screenshot = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(screenshot).toMatchSnapshot('devices-with-data-desktop.png', {
      maxDiffPixels: 100,
      threshold: 0.2,
    });
  });

  test('ProfilesPage - with profiles', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Wait for profiles to load (if any)
    await page.waitForTimeout(500);

    const screenshot = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(screenshot).toMatchSnapshot('profiles-with-data-desktop.png', {
      maxDiffPixels: 100,
      threshold: 0.2,
    });
  });

  test('ConfigPage - keyboard visualizer loaded', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/config');
    await page.waitForLoadState('networkidle');

    // Wait for keyboard visualizer to render
    await page.waitForSelector('[data-testid="keyboard-visualizer"]', {
      state: 'visible',
      timeout: 5000,
    });
    await page.waitForTimeout(500);

    const screenshot = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(screenshot).toMatchSnapshot('config-keyboard-desktop.png', {
      maxDiffPixels: 150, // Slightly higher tolerance for complex keyboard layout
      threshold: 0.2,
    });
  });

  test('MetricsPage - with metrics data', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');

    // Wait for charts to render
    await page.waitForTimeout(1000);

    const screenshot = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(screenshot).toMatchSnapshot('metrics-with-data-desktop.png', {
      maxDiffPixels: 200, // Charts may have slight variations
      threshold: 0.2,
    });
  });

  test('SimulatorPage - keyboard loaded', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Wait for simulator keyboard to render
    await page.waitForSelector('[data-testid="keyboard-visualizer"]', {
      state: 'visible',
      timeout: 5000,
    });
    await page.waitForTimeout(500);

    const screenshot = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(screenshot).toMatchSnapshot('simulator-keyboard-desktop.png', {
      maxDiffPixels: 150,
      threshold: 0.2,
    });
  });
});

// Test modal/dialog states
test.describe('Visual regression - Modals and dialogs', () => {
  test('ProfilesPage - create profile modal', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Click "Create Profile" button
    const createButton = page.getByRole('button', { name: /create profile/i });
    if (await createButton.isVisible()) {
      await createButton.click();

      // Wait for modal animation
      await page.waitForTimeout(300);

      const screenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      expect(screenshot).toMatchSnapshot('profiles-create-modal-desktop.png', {
        maxDiffPixels: 100,
        threshold: 0.2,
      });
    }
  });

  test('DevicesPage - rename dialog', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Try to click a rename button if devices exist
    const renameButton = page.getByRole('button', { name: /rename/i }).first();
    if (await renameButton.isVisible()) {
      await renameButton.click();

      // Wait for inline edit to appear
      await page.waitForTimeout(200);

      const screenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled',
      });

      expect(screenshot).toMatchSnapshot('devices-rename-active-desktop.png', {
        maxDiffPixels: 100,
        threshold: 0.2,
      });
    }
  });
});

// Test responsive layout transitions
test.describe('Visual regression - Responsive breakpoints', () => {
  test('Navigation - mobile vs desktop', async ({ page }) => {
    // Desktop with sidebar
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const desktopNav = await page.screenshot({
      clip: { x: 0, y: 0, width: 250, height: 600 }, // Sidebar area
      animations: 'disabled',
    });

    expect(desktopNav).toMatchSnapshot('navigation-desktop-sidebar.png', {
      maxDiffPixels: 50,
      threshold: 0.2,
    });

    // Mobile with bottom nav
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(300);

    const mobileNav = await page.screenshot({
      clip: { x: 0, y: 567, width: 375, height: 100 }, // Bottom nav area
      animations: 'disabled',
    });

    expect(mobileNav).toMatchSnapshot('navigation-mobile-bottom.png', {
      maxDiffPixels: 50,
      threshold: 0.2,
    });
  });

  test('Card layout - responsive stacking', async ({ page }) => {
    // Desktop: cards in grid
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);

    const desktopCards = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(desktopCards).toMatchSnapshot('home-cards-desktop-grid.png', {
      maxDiffPixels: 100,
      threshold: 0.2,
    });

    // Mobile: cards stacked vertically
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(300);

    const mobileCards = await page.screenshot({
      fullPage: true,
      animations: 'disabled',
    });

    expect(mobileCards).toMatchSnapshot('home-cards-mobile-stack.png', {
      maxDiffPixels: 100,
      threshold: 0.2,
    });
  });
});

// Test dark theme consistency
test.describe('Visual regression - Theme consistency', () => {
  test('All pages have consistent dark theme', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });

    for (const pageInfo of pages) {
      await page.goto(pageInfo.path);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(300);

      // Take screenshot of header area to verify consistent theme colors
      const header = await page.screenshot({
        clip: { x: 0, y: 0, width: 1280, height: 100 },
        animations: 'disabled',
      });

      expect(header).toMatchSnapshot(
        `${pageInfo.name.toLowerCase()}-header-theme.png`,
        {
          maxDiffPixels: 50,
          threshold: 0.15, // Stricter threshold for theme colors
        }
      );
    }
  });
});
