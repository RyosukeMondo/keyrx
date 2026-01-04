import { test, expect } from '@playwright/test';

test.describe('Version Display and Config Page', () => {
  test('should display version in sidebar and load ConfigPage', async ({ page }) => {
    // Navigate to home page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check version is displayed in sidebar (desktop view)
    await page.setViewportSize({ width: 1280, height: 720 });
    const versionText = page.locator('text=/v0\\.1\\.0/');
    await expect(versionText).toBeVisible({ timeout: 10000 });

    // Log version text for verification
    const version = await versionText.textContent();
    console.log('Version displayed:', version);

    // Navigate to ConfigPage
    await page.goto('/config');
    await page.waitForLoadState('networkidle');

    // Wait for either Visual Editor button or loading state
    // ConfigPage should render (not redirect to home)
    const visualEditorButton = page.locator('button:has-text("Visual Editor")');
    const loadingText = page.locator('text=/Loading configuration/');

    // One of these should appear (either loaded or loading)
    await expect(
      Promise.race([
        visualEditorButton.waitFor({ timeout: 15000 }),
        loadingText.waitFor({ timeout: 15000 })
      ])
    ).resolves.toBeTruthy();

    // Check we're NOT on home page (shouldn't see "Active Profile" heading)
    const homePageMarker = page.locator('text=/Active Profile/');
    await expect(homePageMarker).not.toBeVisible();

    // Take screenshot for verification
    await page.screenshot({ path: '/tmp/config-page-screenshot.png', fullPage: true });
    console.log('Screenshot saved to /tmp/config-page-screenshot.png');
  });
});
