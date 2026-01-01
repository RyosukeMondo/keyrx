import { test, expect } from '@playwright/test';

/**
 * E2E Test: Profile CRUD Operations
 *
 * Tests the complete profile lifecycle:
 * - Creating a new profile
 * - Activating a profile
 * - Renaming a profile
 * - Duplicating a profile
 * - Deleting a profile
 *
 * Requirements: REQ-7 (AC4, AC10)
 */

test.describe('Profile CRUD Operations', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
  });

  test('should create a new profile', async ({ page }) => {
    // Click create profile button
    await page.click('button:has-text("Create Profile")');

    // Fill in profile name
    const testProfileName = `TestProfile_${Date.now()}`;
    await page.fill('input[name="profileName"]', testProfileName);

    // Submit form
    await page.click('button:has-text("Create")');

    // Verify profile appears in list
    await expect(page.locator(`text=${testProfileName}`)).toBeVisible();

    // Cleanup: delete the test profile
    await page.click(`button[aria-label="Delete ${testProfileName}"]`);
    await page.click('button:has-text("Confirm")');
  });

  test('should activate a profile', async ({ page }) => {
    // Create a test profile first
    const testProfileName = `ActivateTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', testProfileName);
    await page.click('button:has-text("Create")');

    // Wait for profile to appear
    await expect(page.locator(`text=${testProfileName}`)).toBeVisible();

    // Activate the profile
    await page.click(`button[aria-label="Activate ${testProfileName}"]`);

    // Verify activation indicator (e.g., checkmark or "Active" badge)
    const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
    await expect(profileCard.locator('text=Active')).toBeVisible();

    // Cleanup
    await page.click(`button[aria-label="Delete ${testProfileName}"]`);
    await page.click('button:has-text("Confirm")');
  });

  test('should rename a profile', async ({ page }) => {
    // Create a test profile
    const originalName = `RenameTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', originalName);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${originalName}`)).toBeVisible();

    // Open rename dialog
    await page.click(`button[aria-label="Rename ${originalName}"]`);

    // Enter new name
    const newName = `${originalName}_Renamed`;
    await page.fill('input[name="newProfileName"]', newName);
    await page.click('button:has-text("Rename")');

    // Verify new name appears and old name is gone
    await expect(page.locator(`text=${newName}`)).toBeVisible();
    await expect(page.locator(`text=${originalName}`)).not.toBeVisible();

    // Cleanup
    await page.click(`button[aria-label="Delete ${newName}"]`);
    await page.click('button:has-text("Confirm")');
  });

  test('should duplicate a profile', async ({ page }) => {
    // Create a test profile
    const originalName = `DuplicateTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', originalName);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${originalName}`)).toBeVisible();

    // Duplicate the profile
    await page.click(`button[aria-label="Duplicate ${originalName}"]`);

    // Verify duplicate appears (typically named "Copy of OriginalName")
    const duplicateName = `${originalName}_copy`;
    await expect(page.locator(`text=${duplicateName}`)).toBeVisible();

    // Cleanup both profiles
    await page.click(`button[aria-label="Delete ${originalName}"]`);
    await page.click('button:has-text("Confirm")');
    await page.click(`button[aria-label="Delete ${duplicateName}"]`);
    await page.click('button:has-text("Confirm")');
  });

  test('should delete a profile', async ({ page }) => {
    // Create a test profile
    const testProfileName = `DeleteTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', testProfileName);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${testProfileName}`)).toBeVisible();

    // Delete the profile
    await page.click(`button[aria-label="Delete ${testProfileName}"]`);

    // Confirm deletion
    await page.click('button:has-text("Confirm")');

    // Verify profile is gone
    await expect(page.locator(`text=${testProfileName}`)).not.toBeVisible();
  });

  test('should handle profile activation workflow', async ({ page }) => {
    // Create two test profiles
    const profile1 = `Workflow1_${Date.now()}`;
    const profile2 = `Workflow2_${Date.now()}`;

    // Create first profile
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', profile1);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${profile1}`)).toBeVisible();

    // Create second profile
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', profile2);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${profile2}`)).toBeVisible();

    // Activate first profile
    await page.click(`button[aria-label="Activate ${profile1}"]`);
    await expect(
      page.locator(`[data-profile="${profile1}"]`).locator('text=Active')
    ).toBeVisible();

    // Switch to second profile
    await page.click(`button[aria-label="Activate ${profile2}"]`);
    await expect(
      page.locator(`[data-profile="${profile2}"]`).locator('text=Active')
    ).toBeVisible();

    // Verify first profile is no longer active
    await expect(
      page.locator(`[data-profile="${profile1}"]`).locator('text=Active')
    ).not.toBeVisible();

    // Cleanup
    await page.click(`button[aria-label="Delete ${profile1}"]`);
    await page.click('button:has-text("Confirm")');
    await page.click(`button[aria-label="Delete ${profile2}"]`);
    await page.click('button:has-text("Confirm")');
  });

  test('should validate profile name requirements', async ({ page }) => {
    // Try to create profile with empty name
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', '');
    await page.click('button:has-text("Create")');

    // Verify error message
    await expect(page.locator('text=Profile name is required')).toBeVisible();

    // Try to create profile with invalid characters
    await page.fill('input[name="profileName"]', '../InvalidProfile');
    await page.click('button:has-text("Create")');

    // Verify error message
    await expect(
      page.locator('text=Profile name contains invalid characters')
    ).toBeVisible();
  });

  test('should prevent duplicate profile names', async ({ page }) => {
    // Create a test profile
    const testProfileName = `DuplicateNameTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', testProfileName);
    await page.click('button:has-text("Create")');
    await expect(page.locator(`text=${testProfileName}`)).toBeVisible();

    // Try to create another profile with the same name
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', testProfileName);
    await page.click('button:has-text("Create")');

    // Verify error message
    await expect(
      page.locator('text=Profile with this name already exists')
    ).toBeVisible();

    // Cleanup
    await page.click('button:has-text("Cancel")');
    await page.click(`button[aria-label="Delete ${testProfileName}"]`);
    await page.click('button:has-text("Confirm")');
  });
});
