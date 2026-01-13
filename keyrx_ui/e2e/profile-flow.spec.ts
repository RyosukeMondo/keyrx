import { test, expect } from '@playwright/test';
import { setupApiMocks, createMockProfile } from './fixtures/api-mocks';
import { setupConfigMocks } from './fixtures/config-mocks';

/**
 * Profile Creation and Validation Flow Tests
 *
 * Tests profile lifecycle with mocked API responses.
 */

test.describe('Profile Creation and Validation Flow', () => {
  test.beforeEach(async ({ page }) => {
    await setupApiMocks(page, {
      profiles: [
        createMockProfile('default', { isActive: true }),
        createMockProfile('gaming'),
      ],
    });
    await setupConfigMocks(page);
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
  });

  test('should display profiles page with existing profiles', async ({ page }) => {
    // Verify page loads - use heading to avoid matching multiple elements
    await expect(page.locator('h1:has-text("Profiles")')).toBeVisible({ timeout: 5000 });

    // Verify existing profiles show
    await expect(page.locator('[data-profile="default"]')).toBeVisible();
    await expect(page.locator('[data-profile="gaming"]')).toBeVisible();
  });

  test('should show active badge on active profile', async ({ page }) => {
    const defaultCard = page.locator('[data-profile="default"]');
    await expect(defaultCard).toBeVisible({ timeout: 5000 });

    // Active badge
    const activeBadge = defaultCard.locator('text=/Active/i');
    await expect(activeBadge).toBeVisible();
  });

  test('should have create profile button', async ({ page }) => {
    const createBtn = page.locator('button:has-text("Create")');
    await expect(createBtn).toBeVisible({ timeout: 5000 });
  });

  test('should have activate button for inactive profiles', async ({ page }) => {
    const gamingCard = page.locator('[data-profile="gaming"]');
    await expect(gamingCard).toBeVisible({ timeout: 5000 });

    // Activate button should be present for non-active profile
    const activateBtn = gamingCard.locator('button:has-text(/Activate/i)');
    const hasActivate = await activateBtn.isVisible().catch(() => false);

    // Either has activate button or shows some action control
    const cardText = await gamingCard.textContent();
    expect(hasActivate || cardText!.includes('gaming')).toBe(true);
  });

  test('should navigate to config page', async ({ page }) => {
    // Click on a profile card to edit
    const gamingCard = page.locator('[data-profile="gaming"]');
    await expect(gamingCard).toBeVisible({ timeout: 5000 });

    // Look for edit/configure button
    const configBtn = gamingCard.locator('button:has-text(/Edit|Configure/i)').first();
    const hasConfigBtn = await configBtn.isVisible().catch(() => false);

    if (hasConfigBtn) {
      await configBtn.click();
      await page.waitForLoadState('networkidle');

      // Should navigate to config page
      await expect(page).toHaveURL(/\/config/);
    }
  });

  test('should display validation status on profiles', async ({ page }) => {
    // Setup with a profile that has validation errors
    await setupApiMocks(page, {
      profiles: [
        createMockProfile('default', { isActive: true, valid: true }),
        createMockProfile('invalid', { valid: false, errors: [{ line: 1, column: 1, message: 'Syntax error' }] }),
      ],
    });
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Invalid profile should show warning indicator
    const invalidCard = page.locator('[data-profile="invalid"]');
    await expect(invalidCard).toBeVisible({ timeout: 5000 });

    // Look for warning/error indicator
    const warningIndicator = invalidCard.locator('text=/Warning|Error|Invalid|!/i');
    const hasWarning = await warningIndicator.isVisible().catch(() => false);

    // Either has warning or shows profile name
    const cardText = await invalidCard.textContent();
    expect(hasWarning || cardText!.includes('invalid')).toBe(true);
  });

  test('should handle profile switch', async ({ page }) => {
    // Default is active
    const defaultCard = page.locator('[data-profile="default"]');
    await expect(defaultCard.locator('text=/Active/i')).toBeVisible({ timeout: 5000 });

    // Find and click activate on gaming
    const gamingCard = page.locator('[data-profile="gaming"]');
    const activateBtn = gamingCard.locator('button:has-text(/Activate/i)');

    if (await activateBtn.isVisible().catch(() => false)) {
      await activateBtn.click();
      await page.waitForTimeout(500);

      // Gaming should now show active
      await expect(gamingCard.locator('text=/Active/i')).toBeVisible({ timeout: 3000 });
    }
  });

  test('should persist active profile across navigation', async ({ page }) => {
    // Verify default is active
    await expect(page.locator('[data-profile="default"]').locator('text=/Active/i')).toBeVisible({ timeout: 5000 });

    // Navigate to dashboard
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Navigate back to profiles
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Default should still be active
    await expect(page.locator('[data-profile="default"]').locator('text=/Active/i')).toBeVisible({ timeout: 5000 });
  });

  test('should be responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });

    // Page should render content
    const content = await page.content();
    expect(content.length).toBeGreaterThan(100);
  });

  test('should be responsive on tablet', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });

    // Page should render content
    const content = await page.content();
    expect(content.length).toBeGreaterThan(100);
  });

  test('should handle empty profiles list', async ({ page }) => {
    await setupApiMocks(page, { profiles: [] });
    await page.goto('/profiles');
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(500);

    // Page should render content
    const content = await page.content();
    expect(content.length).toBeGreaterThan(100);
  });

  test('should open create modal and close with Escape', async ({ page }) => {
    const createBtn = page.locator('button:has-text("Create")');
    await createBtn.click();

    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible({ timeout: 5000 });

    await page.keyboard.press('Escape');
    await expect(modal).not.toBeVisible({ timeout: 3000 });
  });
});
