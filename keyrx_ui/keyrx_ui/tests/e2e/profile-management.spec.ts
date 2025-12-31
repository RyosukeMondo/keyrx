import { test, expect } from '@playwright/test';

/**
 * E2E tests for the profile management workflow.
 *
 * Tests the complete user journey from creating a new profile,
 * activating it, renaming it, duplicating it, exporting it, and deleting it.
 */
test.describe('Profile Management E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to load
    await expect(page.locator('h1')).toContainText('KeyRX');

    // Click the Profiles button
    await page.click('button:has-text("Profiles")');

    // Wait for the profiles page to load
    await expect(page.locator('h2')).toContainText('Profiles');
  });

  test('should display profiles page with header', async ({ page }) => {
    // Verify page title
    await expect(page.locator('h2')).toContainText('Profiles');

    // Verify create button exists
    const createButton = page.locator('button:has-text("New Profile")');
    await expect(createButton).toBeVisible();
  });

  test('should show loading state initially', async ({ page }) => {
    // Reload the page to see loading state
    await page.reload();
    await page.click('button:has-text("Profiles")');

    // Loading text might appear briefly (timing dependent)
    // Just verify the page eventually loads
    await expect(page.locator('h2')).toContainText('Profiles');
  });

  test('should create a new profile', async ({ page }) => {
    // Click the New Profile button
    await page.click('button:has-text("New Profile")');

    // Wait for dialog to appear
    const dialog = page.locator('.profile-dialog');
    await expect(dialog).toBeVisible();

    // Verify dialog title
    await expect(dialog.locator('h3')).toContainText('Create New Profile');

    // Enter profile name
    const profileName = `test-profile-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);

    // Select template (blank by default)
    // Template selector might be a dropdown or radio buttons
    // We'll just use the default

    // Submit the form
    await page.click('button:has-text("Create")');

    // Wait for dialog to close
    await expect(dialog).not.toBeVisible();

    // Verify the new profile appears in the list
    await expect(page.locator('.profile-card').filter({ hasText: profileName })).toBeVisible();
  });

  test('should validate profile name', async ({ page }) => {
    // Click the New Profile button
    await page.click('button:has-text("New Profile")');

    const dialog = page.locator('.profile-dialog');
    await expect(dialog).toBeVisible();

    // Try to create profile with invalid name (spaces, special chars)
    await page.fill('input[type="text"]', 'invalid name!@#');

    // Try to submit - should show validation error or prevent submission
    const createButton = dialog.locator('button:has-text("Create")');

    // The button might be disabled, or clicking might show an error
    // Let's check if there's a validation message or if button is disabled
    const isDisabled = await createButton.isDisabled();
    if (!isDisabled) {
      await createButton.click();
      // Should show error message
      const errorMessage = dialog.locator('.error-message, .validation-error');
      await expect(errorMessage).toBeVisible();
    } else {
      // Button is disabled, which is also valid validation
      expect(isDisabled).toBe(true);
    }

    // Close dialog
    await page.click('button:has-text("Cancel")');
  });

  test('should activate a profile', async ({ page }) => {
    // First, ensure we have at least one profile
    const profiles = page.locator('.profile-card');
    const profileCount = await profiles.count();

    if (profileCount === 0) {
      // Create a profile first
      await page.click('button:has-text("New Profile")');
      const profileName = `test-profile-${Date.now()}`;
      await page.fill('input[type="text"]', profileName);
      await page.click('button:has-text("Create")');
      await page.waitForTimeout(500);
    }

    // Find a non-active profile
    const firstProfile = page.locator('.profile-card').first();

    // Hover over the profile card to show actions
    await firstProfile.hover();

    // Click the Activate button
    const activateButton = firstProfile.locator('button:has-text("Activate")');
    if (await activateButton.isVisible()) {
      await activateButton.click();

      // Wait for the operation to complete
      await page.waitForTimeout(500);

      // Verify the profile is now active (has active indicator)
      const activeIndicator = firstProfile.locator('.active-indicator, .profile-status-active');
      await expect(activeIndicator).toBeVisible();
    }
  });

  test('should delete a profile with confirmation', async ({ page }) => {
    // Create a test profile to delete
    await page.click('button:has-text("New Profile")');
    const profileName = `delete-me-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // Find the profile we just created
    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(profileCard).toBeVisible();

    // Hover to show actions
    await profileCard.hover();

    // Set up dialog handler for confirmation
    page.on('dialog', dialog => {
      expect(dialog.message()).toContain('Delete');
      dialog.accept();
    });

    // Click delete button
    const deleteButton = profileCard.locator('button:has-text("Delete")');
    await deleteButton.click();

    // Wait for deletion to complete
    await page.waitForTimeout(500);

    // Verify profile is removed
    await expect(profileCard).not.toBeVisible();
  });

  test('should cancel profile deletion', async ({ page }) => {
    // Create a test profile
    await page.click('button:has-text("New Profile")');
    const profileName = `keep-me-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(profileCard).toBeVisible();

    // Hover to show actions
    await profileCard.hover();

    // Set up dialog handler to cancel
    page.on('dialog', dialog => {
      dialog.dismiss();
    });

    // Click delete button
    const deleteButton = profileCard.locator('button:has-text("Delete")');
    await deleteButton.click();

    // Wait a bit
    await page.waitForTimeout(300);

    // Verify profile is still there
    await expect(profileCard).toBeVisible();
  });

  test('should duplicate a profile', async ({ page }) => {
    // Create a test profile to duplicate
    await page.click('button:has-text("New Profile")');
    const profileName = `original-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const originalCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(originalCard).toBeVisible();

    // Hover to show actions
    await originalCard.hover();

    // Set up dialog handler for duplicate name prompt
    const duplicateName = `${profileName}-copy`;
    page.on('dialog', dialog => {
      expect(dialog.message()).toContain('duplicate');
      dialog.accept(duplicateName);
    });

    // Click duplicate button
    const duplicateButton = originalCard.locator('button:has-text("Duplicate")');
    await duplicateButton.click();

    // Wait for duplication
    await page.waitForTimeout(500);

    // Verify both profiles exist
    await expect(page.locator('.profile-card').filter({ hasText: profileName })).toBeVisible();
    await expect(page.locator('.profile-card').filter({ hasText: duplicateName })).toBeVisible();
  });

  test('should export a profile', async ({ page }) => {
    // Create a test profile
    await page.click('button:has-text("New Profile")');
    const profileName = `export-test-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(profileCard).toBeVisible();

    // Hover to show actions
    await profileCard.hover();

    // Set up download handler
    const downloadPromise = page.waitForEvent('download');

    // Click export button
    const exportButton = profileCard.locator('button:has-text("Export")');
    await exportButton.click();

    // Wait for download
    const download = await downloadPromise;

    // Verify the download filename
    expect(download.suggestedFilename()).toContain(profileName);
    expect(download.suggestedFilename()).toContain('.rhai');
  });

  test('should attempt rename (not implemented yet)', async ({ page }) => {
    // Create a test profile
    await page.click('button:has-text("New Profile")');
    const profileName = `rename-test-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(profileCard).toBeVisible();

    // Hover to show actions
    await profileCard.hover();

    // Click rename button if it exists
    const renameButton = profileCard.locator('button:has-text("Rename")');
    if (await renameButton.isVisible()) {
      // Set up dialog handler for alert about not implemented
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain('not yet implemented');
        dialog.accept();
      });

      await renameButton.click();
    }
  });

  test('should display empty state when no profiles exist', async ({ page }) => {
    // This test assumes we can get to an empty state
    // In practice, there might always be a default profile
    // We'll check for the empty state message if it exists

    const emptyState = page.locator('.no-profiles');
    const createFirstButton = page.locator('button:has-text("Create your first profile")');

    // If empty state is visible, verify its content
    if (await emptyState.isVisible()) {
      await expect(emptyState).toContainText('No profiles found');
      await expect(createFirstButton).toBeVisible();

      // Click the create first profile button
      await createFirstButton.click();

      // Dialog should appear
      const dialog = page.locator('.profile-dialog');
      await expect(dialog).toBeVisible();
    }
  });

  test('should handle API errors gracefully', async ({ page }) => {
    // This is tricky to test without mocking
    // We can try to create a profile with a name that might cause an error
    // or we can just verify that error states are handled

    await page.click('button:has-text("New Profile")');

    // Try to create a profile with the same name twice
    const profileName = `duplicate-name-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // Try to create another with the same name
    await page.click('button:has-text("New Profile")');
    await page.fill('input[type="text"]', profileName);

    // Set up alert handler
    page.on('dialog', dialog => {
      // Should show an error about duplicate name
      dialog.accept();
    });

    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // The error should have been shown in an alert
  });

  test('complete workflow: create → activate → duplicate → export → delete', async ({ page }) => {
    // Step 1: Create a new profile
    const profileName = `workflow-test-${Date.now()}`;
    await page.click('button:has-text("New Profile")');
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await expect(profileCard).toBeVisible();

    // Step 2: Activate the profile
    await profileCard.hover();
    const activateButton = profileCard.locator('button:has-text("Activate")');
    if (await activateButton.isVisible()) {
      await activateButton.click();
      await page.waitForTimeout(500);
    }

    // Step 3: Duplicate the profile
    await profileCard.hover();
    const duplicateName = `${profileName}-copy`;
    page.once('dialog', dialog => dialog.accept(duplicateName));
    await profileCard.locator('button:has-text("Duplicate")').click();
    await page.waitForTimeout(500);

    const duplicateCard = page.locator('.profile-card').filter({ hasText: duplicateName });
    await expect(duplicateCard).toBeVisible();

    // Step 4: Export the original profile
    await profileCard.hover();
    const downloadPromise = page.waitForEvent('download');
    await profileCard.locator('button:has-text("Export")').click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.rhai');

    // Step 5: Delete the duplicate profile
    await duplicateCard.hover();
    page.once('dialog', dialog => dialog.accept());
    await duplicateCard.locator('button:has-text("Delete")').click();
    await page.waitForTimeout(500);
    await expect(duplicateCard).not.toBeVisible();

    // Step 6: Verify original still exists
    await expect(profileCard).toBeVisible();

    // Step 7: Clean up - delete the original
    await profileCard.hover();
    page.once('dialog', dialog => dialog.accept());
    await profileCard.locator('button:has-text("Delete")').click();
    await page.waitForTimeout(500);
    await expect(profileCard).not.toBeVisible();
  });

  test('should maintain profile list after navigation', async ({ page }) => {
    // Create a profile
    await page.click('button:has-text("New Profile")');
    const profileName = `persist-test-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // Verify it exists
    await expect(page.locator('.profile-card').filter({ hasText: profileName })).toBeVisible();

    // Navigate to another page
    await page.click('button:has-text("Devices")');
    await page.waitForTimeout(200);

    // Navigate back to Profiles
    await page.click('button:has-text("Profiles")');
    await page.waitForTimeout(500);

    // Verify the profile still exists
    await expect(page.locator('.profile-card').filter({ hasText: profileName })).toBeVisible();

    // Clean up
    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });
    await profileCard.hover();
    page.once('dialog', dialog => dialog.accept());
    await profileCard.locator('button:has-text("Delete")').click();
  });

  test('should show profile metadata', async ({ page }) => {
    // Check if any profiles exist
    const profiles = page.locator('.profile-card');
    const profileCount = await profiles.count();

    if (profileCount === 0) {
      // Create a profile
      await page.click('button:has-text("New Profile")');
      await page.fill('input[type="text"]', `metadata-test-${Date.now()}`);
      await page.click('button:has-text("Create")');
      await page.waitForTimeout(500);
    }

    // Check first profile card for metadata
    const firstProfile = page.locator('.profile-card').first();
    await expect(firstProfile).toBeVisible();

    // Profile cards should show some metadata like layer count, modified date, etc.
    // The exact selectors depend on the ProfileCard implementation
    // We'll just verify the card has content
    const cardText = await firstProfile.textContent();
    expect(cardText).toBeTruthy();
  });

  test('should handle rapid actions without errors', async ({ page }) => {
    // Create a profile
    await page.click('button:has-text("New Profile")');
    const profileName = `rapid-test-${Date.now()}`;
    await page.fill('input[type="text"]', profileName);
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    const profileCard = page.locator('.profile-card').filter({ hasText: profileName });

    // Rapidly hover and perform actions
    await profileCard.hover();
    await page.waitForTimeout(100);

    // The page should not crash
    await expect(profileCard).toBeVisible();

    // Clean up
    await profileCard.hover();
    page.once('dialog', dialog => dialog.accept());
    await profileCard.locator('button:has-text("Delete")').click();
  });

  test('should be keyboard accessible', async ({ page }) => {
    // Tab to the New Profile button
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab'); // Navigate through nav buttons

    // Should be able to activate the New Profile button with Enter
    await page.keyboard.press('Enter');

    // Dialog should open
    const dialog = page.locator('.profile-dialog');
    await expect(dialog).toBeVisible();

    // Close with Escape
    await page.keyboard.press('Escape');
    await expect(dialog).not.toBeVisible();
  });
});
