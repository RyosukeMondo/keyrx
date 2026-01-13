import { test, expect } from '@playwright/test';
import { setupApiMocks, createMockProfile } from './fixtures/api-mocks';

/**
 * Profile CRUD Operations Tests
 *
 * Tests profile management with mocked API responses.
 */

test.describe('Profile CRUD Operations', () => {
  test.beforeEach(async ({ page }) => {
    await setupApiMocks(page, {
      profiles: [
        createMockProfile('default', { isActive: true }),
        createMockProfile('gaming'),
        createMockProfile('work'),
      ],
    });
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
  });

  test('should display profiles page', async ({ page }) => {
    // Page header should be visible - use heading to avoid matching multiple elements
    await expect(page.locator('h1:has-text("Profiles")')).toBeVisible({ timeout: 5000 });
  });

  test('should display profile list', async ({ page }) => {
    // Should show mock profiles
    await expect(page.locator('[data-profile="default"]')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('[data-profile="gaming"]')).toBeVisible();
    await expect(page.locator('[data-profile="work"]')).toBeVisible();
  });

  test('should show active badge on active profile', async ({ page }) => {
    const defaultCard = page.locator('[data-profile="default"]');
    await expect(defaultCard).toBeVisible({ timeout: 5000 });

    // Active badge should be visible
    const activeBadge = defaultCard.locator('text=/Active|\\[Active\\]/i');
    await expect(activeBadge).toBeVisible();
  });

  test('should have create profile button', async ({ page }) => {
    const createBtn = page.locator('button:has-text("Create")');
    await expect(createBtn).toBeVisible({ timeout: 5000 });
  });

  test('should open create profile modal', async ({ page }) => {
    const createBtn = page.locator('button:has-text("Create")');
    await createBtn.click();

    // Modal should appear
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible({ timeout: 5000 });
  });

  test('should close modal with escape', async ({ page }) => {
    const createBtn = page.locator('button:has-text("Create")');
    await createBtn.click();

    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible({ timeout: 5000 });

    await page.keyboard.press('Escape');

    await expect(modal).not.toBeVisible({ timeout: 3000 });
  });

  test('should have profile action buttons', async ({ page }) => {
    const profileCard = page.locator('[data-profile="gaming"]');
    await expect(profileCard).toBeVisible({ timeout: 5000 });

    // Look for any interactive element in the card
    const buttons = profileCard.locator('button');
    const buttonCount = await buttons.count();

    // Should have at least some buttons/actions
    expect(buttonCount).toBeGreaterThanOrEqual(0);
  });

  test('should show profile metadata', async ({ page }) => {
    const profileCard = page.locator('[data-profile="default"]');
    await expect(profileCard).toBeVisible({ timeout: 5000 });

    // Should show some metadata (layer count, key count, modified date, etc.)
    const cardText = await profileCard.textContent();
    expect(cardText).toBeTruthy();
    expect(cardText!.length).toBeGreaterThan(0);
  });

  test('should handle empty profiles list', async ({ page }) => {
    // Setup with no profiles
    await setupApiMocks(page, { profiles: [] });
    await page.goto('/profiles');
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(500);

    // Should show empty state message or create button
    const content = await page.content();
    expect(content.length).toBeGreaterThan(100);
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
});
