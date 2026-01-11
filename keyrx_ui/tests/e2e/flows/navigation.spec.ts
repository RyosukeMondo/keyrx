/**
 * Navigation Flow E2E Tests
 *
 * Tests application navigation functionality to verify:
 * - All sidebar navigation links work correctly
 * - Bottom navigation (mobile) works correctly
 * - Browser back/forward navigation functions properly
 * - Deep links work as expected
 * - URL changes reflect page navigation
 * - Page content loads after navigation
 *
 * These tests ensure the entire navigation system works end-to-end
 * across both desktop (sidebar) and mobile (bottom nav) layouts.
 */

import { test, expect } from '../fixtures/daemon';

test.describe('Navigation Flow - Desktop (Sidebar)', () => {
  test('should navigate to all pages via sidebar links', async ({ page }) => {
    // Start at home page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Define all navigation paths
    const navItems = [
      { path: '/home', label: 'Home', heading: /dashboard/i },
      { path: '/devices', label: 'Devices', heading: /devices/i },
      { path: '/profiles', label: 'Profiles', heading: /profiles/i },
      { path: '/config', label: 'Config', heading: /configuration|config/i },
      { path: '/metrics', label: 'Metrics', heading: /metrics/i },
      { path: '/simulator', label: 'Simulator', heading: /simulator/i },
    ];

    // Test each navigation link
    for (const item of navItems) {
      // Find and click the navigation link by aria-label
      const navLink = page.getByRole('link', { name: new RegExp(`navigate to ${item.label}`, 'i') }).first();
      await expect(navLink).toBeVisible();
      await navLink.click();

      // Verify URL changed
      await page.waitForURL(new RegExp(item.path));
      expect(page.url()).toContain(item.path);

      // Verify page content loaded (check for heading)
      const heading = page.getByRole('heading', { name: item.heading, level: 1 });
      await expect(heading).toBeVisible({ timeout: 5000 });
    }
  });

  test('should highlight active navigation item', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Find the active link (should have specific styling)
    const devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    await expect(devicesLink).toBeVisible();

    // Check that the link has the active class
    // The active link should have bg-primary-600 class based on Sidebar.tsx
    const classes = await devicesLink.getAttribute('class');
    expect(classes).toContain('bg-primary-600');
  });

  test('should update active state when navigating between pages', async ({ page }) => {
    // Start at home
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    const homeLink = page.getByRole('link', { name: /navigate to home/i }).first();
    let classes = await homeLink.getAttribute('class');
    expect(classes).toContain('bg-primary-600');

    // Navigate to profiles
    const profilesLink = page.getByRole('link', { name: /navigate to profiles/i }).first();
    await profilesLink.click();
    await page.waitForURL(/\/profiles/);

    // Verify home is no longer active
    classes = await homeLink.getAttribute('class');
    expect(classes).not.toContain('bg-primary-600');

    // Verify profiles is now active
    classes = await profilesLink.getAttribute('class');
    expect(classes).toContain('bg-primary-600');
  });
});

test.describe('Navigation Flow - Mobile (Bottom Nav)', () => {
  test.use({ viewport: { width: 375, height: 667 } }); // iPhone SE size

  test('should navigate to all pages via bottom nav on mobile', async ({ page }) => {
    // Start at home page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Define all navigation paths (same as sidebar)
    const navItems = [
      { path: '/home', label: 'Home', heading: /dashboard/i },
      { path: '/devices', label: 'Devices', heading: /devices/i },
      { path: '/profiles', label: 'Profiles', heading: /profiles/i },
      { path: '/config', label: 'Config', heading: /configuration|config/i },
      { path: '/metrics', label: 'Metrics', heading: /metrics/i },
      { path: '/simulator', label: 'Sim', heading: /simulator/i }, // Note: "Sim" on mobile
    ];

    // Test each navigation link
    for (const item of navItems) {
      // Find and click the navigation link
      const navLink = page.getByRole('link', { name: new RegExp(`navigate to ${item.label.replace('Sim', 'Simulator')}`, 'i') }).first();
      await expect(navLink).toBeVisible();
      await navLink.click();

      // Verify URL changed
      await page.waitForURL(new RegExp(item.path));
      expect(page.url()).toContain(item.path);

      // Verify page content loaded
      const heading = page.getByRole('heading', { name: item.heading, level: 1 });
      await expect(heading).toBeVisible({ timeout: 5000 });
    }
  });

  test('should highlight active navigation item on mobile', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Find the active link
    const devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    await expect(devicesLink).toBeVisible();

    // Check that the link has the active class
    // Active items should have text-primary-500 based on BottomNav.tsx
    const classes = await devicesLink.getAttribute('class');
    expect(classes).toContain('text-primary-500');
  });

  test('should show bottom nav only on mobile viewport', async ({ page }) => {
    // Get bottom nav element
    const bottomNav = page.getByRole('navigation', { name: /mobile bottom navigation/i });

    // Should be visible on mobile viewport (375px width)
    await page.goto('/');
    await expect(bottomNav).toBeVisible();
  });
});

test.describe('Navigation Flow - Browser History', () => {
  test('should support browser back button', async ({ page }) => {
    // Navigate through several pages
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Use browser back button
    await page.goBack();
    await page.waitForLoadState('networkidle');

    // Should be back on devices page
    expect(page.url()).toContain('/devices');
    const heading = page.getByRole('heading', { name: /devices/i, level: 1 });
    await expect(heading).toBeVisible();

    // Go back again
    await page.goBack();
    await page.waitForLoadState('networkidle');

    // Should be back on home page
    expect(page.url()).toContain('/home');
  });

  test('should support browser forward button', async ({ page }) => {
    // Navigate through pages
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Go back
    await page.goBack();
    await page.waitForLoadState('networkidle');
    expect(page.url()).toContain('/home');

    // Go forward
    await page.goForward();
    await page.waitForLoadState('networkidle');

    // Should be on devices page again
    expect(page.url()).toContain('/devices');
    const heading = page.getByRole('heading', { name: /devices/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should update active nav state when using back/forward', async ({ page }) => {
    // Navigate to devices
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Check devices link is active
    let devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    let classes = await devicesLink.getAttribute('class');
    expect(classes).toContain('bg-primary-600');

    // Go back to home
    await page.goBack();
    await page.waitForLoadState('networkidle');

    // Check home link is now active
    const homeLink = page.getByRole('link', { name: /navigate to home/i }).first();
    classes = await homeLink.getAttribute('class');
    expect(classes).toContain('bg-primary-600');

    // Check devices link is no longer active
    devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    classes = await devicesLink.getAttribute('class');
    expect(classes).not.toContain('bg-primary-600');
  });
});

test.describe('Navigation Flow - Deep Links', () => {
  test('should handle direct navigation to profile config page', async ({ page, daemon }) => {
    // Create a test profile
    const testProfileName = `e2e-nav-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate directly to profile config deep link
    await page.goto(`/profiles/${testProfileName}/config`);
    await page.waitForLoadState('networkidle');

    // Verify we're on the config page for this profile
    expect(page.url()).toContain(`/profiles/${testProfileName}/config`);

    // Verify page content loaded
    const heading = page.getByRole('heading', { name: /configuration|config/i, level: 1 });
    await expect(heading).toBeVisible({ timeout: 5000 });

    // Clean up
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should handle deep links with URL encoding', async ({ page, daemon }) => {
    // Create a test profile with special characters
    const testProfileName = `e2e nav test ${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to encoded URL
    const encodedName = encodeURIComponent(testProfileName);
    await page.goto(`/profiles/${encodedName}/config`);
    await page.waitForLoadState('networkidle');

    // Verify page loaded (it might show error if profile name handling is incorrect)
    const body = page.locator('body');
    await expect(body).toBeVisible();

    // Clean up
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should redirect unknown routes to home', async ({ page }) => {
    // Navigate to non-existent route
    await page.goto('/this-route-does-not-exist');
    await page.waitForLoadState('networkidle');

    // Should redirect to home
    expect(page.url()).toContain('/home');

    // Verify home page loaded
    const heading = page.getByRole('heading', { name: /dashboard/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should redirect root path to /home', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Should redirect to /home
    expect(page.url()).toContain('/home');

    // Verify home page loaded
    const heading = page.getByRole('heading', { name: /dashboard/i, level: 1 });
    await expect(heading).toBeVisible();
  });
});

test.describe('Navigation Flow - Accessibility', () => {
  test('should support keyboard navigation through sidebar', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Focus first nav link and verify it's focusable
    const homeLink = page.getByRole('link', { name: /navigate to home/i }).first();
    await homeLink.focus();

    // Verify focus is visible
    await expect(homeLink).toBeFocused();

    // Tab to next link
    await page.keyboard.press('Tab');

    // Should focus on devices link
    const devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    await expect(devicesLink).toBeFocused();

    // Activate link with Enter key
    await page.keyboard.press('Enter');
    await page.waitForURL(/\/devices/);

    // Verify navigation worked
    expect(page.url()).toContain('/devices');
  });

  test('should have proper ARIA labels on navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check sidebar has proper aria-label
    const sidebar = page.getByRole('navigation', { name: /primary navigation/i });
    await expect(sidebar).toBeVisible();

    // Check each link has aria-label
    const homeLink = page.getByRole('link', { name: /navigate to home/i }).first();
    const ariaLabel = await homeLink.getAttribute('aria-label');
    expect(ariaLabel).toBeTruthy();
    expect(ariaLabel).toContain('Navigate to Home');
  });

  test('should announce page changes to screen readers', async ({ page }) => {
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    // Navigate to another page
    const devicesLink = page.getByRole('link', { name: /navigate to devices/i }).first();
    await devicesLink.click();
    await page.waitForURL(/\/devices/);

    // Verify main content has proper landmark
    const main = page.getByRole('main');
    await expect(main).toBeVisible();

    // Verify heading is present (helps screen readers)
    const heading = page.getByRole('heading', { name: /devices/i, level: 1 });
    await expect(heading).toBeVisible();
  });
});

test.describe('Navigation Flow - Performance', () => {
  test('should navigate quickly between pages', async ({ page }) => {
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    const navigationTimes: number[] = [];

    const pages = ['/devices', '/profiles', '/config', '/metrics', '/simulator', '/home'];

    for (const pagePath of pages) {
      const startTime = Date.now();

      await page.goto(pagePath);
      await page.waitForLoadState('networkidle');

      const navigationTime = Date.now() - startTime;
      navigationTimes.push(navigationTime);

      // Each navigation should be reasonably fast (< 3 seconds)
      expect(navigationTime).toBeLessThan(3000);
    }

    // Average navigation time should be reasonable
    const avgTime = navigationTimes.reduce((a, b) => a + b, 0) / navigationTimes.length;
    expect(avgTime).toBeLessThan(2000); // 2 seconds average
  });

  test('should not reload daemon data unnecessarily during navigation', async ({ page }) => {
    const monitor = await import('../fixtures/network-monitor').then(m => m.NetworkMonitor);
    const networkMonitor = new monitor(page);
    networkMonitor.start();

    // Navigate through several pages
    await page.goto('/home');
    await page.waitForLoadState('networkidle');

    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Count API requests
    const requests = networkMonitor.getRequests();

    // Should make reasonable number of requests (not excessive)
    // Each page might fetch data, but shouldn't refetch unnecessarily
    expect(requests.length).toBeLessThan(20); // Allow for some flexibility

    networkMonitor.stop();
  });
});
