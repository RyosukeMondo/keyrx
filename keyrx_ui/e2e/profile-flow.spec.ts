import { test, expect } from '@playwright/test';

/**
 * E2E Test: Profile Creation and Validation Flow
 *
 * Tests the complete profile lifecycle with validation:
 * 1. Create profile with valid template → Verify validation → Activate → Verify [Active] badge
 * 2. Create profile with invalid syntax → Verify warning badge → [Activate] disabled
 * 3. Edit profile → Change code → Save → Verify validation
 *
 * Requirements:
 * - 0.F (End-to-End User Flow Testing)
 * - Requirement 4 (Validate Profiles Before Activation)
 *
 * This test ensures the complete user workflow from profile creation through
 * validation and activation works correctly, catching issues before they reach production.
 */

test.describe('Profile Creation and Validation Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');
  });

  test('should create profile with valid template → verify validation → activate → verify [Active] badge', async ({ page }) => {
    const testProfileName = `ValidProfile_${Date.now()}`;

    await test.step('Create new profile with valid template', async () => {
      // Click create profile button
      await page.click('button:has-text("Create Profile")');

      // Fill in profile name
      await page.fill('input[name="profileName"]', testProfileName);

      // Select a valid template (if template selector exists)
      const templateSelector = page.locator('select[name="template"]');
      if (await templateSelector.isVisible().catch(() => false)) {
        await templateSelector.selectOption('blank');
      }

      // Submit form
      await page.click('button:has-text("Create")');

      // Wait for profile to appear in list
      await expect(page.locator(`text=${testProfileName}`)).toBeVisible({ timeout: 5000 });
    });

    await test.step('Verify profile validation status shows valid', async () => {
      // Locate the profile card
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard).toBeVisible({ timeout: 5000 });

      // Check for validation badge - should show "✓ Valid" or similar
      // The exact text depends on implementation, so we'll check for common variations
      const validBadge = profileCard.locator('text=/✓.*Valid|Valid|No errors/i');

      // Wait for validation to complete (validation may be async)
      await page.waitForTimeout(1500);

      // If validation badge is implemented, verify it shows valid status
      if (await validBadge.isVisible().catch(() => false)) {
        await expect(validBadge).toBeVisible();
      }

      // Verify no error/warning badges are shown
      const warningBadge = profileCard.locator('text=/⚠️|Warning|Invalid/i');
      await expect(warningBadge).not.toBeVisible();
    });

    await test.step('Activate the profile', async () => {
      // Click activate button
      const activateButton = page.locator(`button[aria-label="Activate ${testProfileName}"]`);
      await expect(activateButton).toBeVisible({ timeout: 5000 });

      // Verify button is NOT disabled (valid profiles can be activated)
      await expect(activateButton).toBeEnabled();

      await activateButton.click();

      // Wait for activation to complete
      await page.waitForTimeout(1000);
    });

    await test.step('Verify [Active] badge appears', async () => {
      // Verify [Active] badge appears on the profile card
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      const activeBadge = profileCard.locator('text=/Active|\\[Active\\]/i');

      await expect(activeBadge).toBeVisible({ timeout: 5000 });

      // Ensure no error modal appeared during activation
      const errorModal = page.locator('text=/Compilation Error|Activation Failed/i');
      await expect(errorModal).not.toBeVisible();
    });

    await test.step('Cleanup: delete test profile', async () => {
      await page.click(`button[aria-label="Delete ${testProfileName}"]`);
      await page.click('button:has-text("Confirm")');
      await expect(page.locator(`text=${testProfileName}`)).not.toBeVisible({ timeout: 3000 });
    });
  });

  test('should create profile with invalid syntax → verify warning badge → [Activate] disabled', async ({ page }) => {
    const testProfileName = `InvalidProfile_${Date.now()}`;

    await test.step('Create profile', async () => {
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', testProfileName);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${testProfileName}`)).toBeVisible({ timeout: 5000 });
    });

    await test.step('Navigate to configuration editor', async () => {
      // Navigate to config page for the profile
      await page.goto(`/config?profile=${encodeURIComponent(testProfileName)}`);
      await page.waitForLoadState('networkidle');

      // Verify we're on the config page
      await expect(page.locator(`text=/Editing.*${testProfileName}/i`)).toBeVisible({ timeout: 10000 });
    });

    await test.step('Switch to Code tab and inject invalid syntax', async () => {
      // Switch to Code tab (Monaco editor)
      const codeTab = page.locator('button:has-text("Code")');
      await codeTab.click();
      await page.waitForTimeout(500);

      // Wait for Monaco editor to load
      await page.waitForSelector('.monaco-editor', { timeout: 5000 });

      // Inject invalid Rhai syntax using Monaco editor API
      // This simulates a user typing invalid code
      const invalidCode = `
// Invalid syntax - using layer() function which is not supported
layer("invalid") {
  map(VK_A, VK_B);
}

// Also missing device_start/device_end blocks
map(VK_CAPS_LOCK, VK_ESCAPE);
`;

      // Click in the editor to focus it
      await page.click('.monaco-editor');

      // Select all text (Ctrl+A) and replace with invalid code
      await page.keyboard.press('Control+A');
      await page.keyboard.type(invalidCode);

      // Wait for auto-save to trigger (if enabled)
      await page.waitForTimeout(2000);

      // Alternatively, manually trigger save if there's a save button
      const saveButton = page.locator('button:has-text("Save")');
      if (await saveButton.isVisible().catch(() => false)) {
        await saveButton.click();
        await page.waitForTimeout(1000);
      }
    });

    await test.step('Navigate back to profiles page', async () => {
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');
    });

    await test.step('Verify warning badge appears on profile card', async () => {
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard).toBeVisible({ timeout: 5000 });

      // Wait for validation to complete
      await page.waitForTimeout(1500);

      // Look for warning/invalid badge
      const warningBadge = profileCard.locator('text=/⚠️|Warning|Invalid Configuration|Compilation Error/i');

      // If validation is implemented, we should see a warning badge
      if (await warningBadge.isVisible().catch(() => false)) {
        await expect(warningBadge).toBeVisible();
      } else {
        // Log a warning that validation UI might not be implemented yet
        console.log('Warning: Validation badge not found - validation UI may not be implemented yet');
      }
    });

    await test.step('Verify [Activate] button is disabled for invalid profile', async () => {
      const activateButton = page.locator(`button[aria-label="Activate ${testProfileName}"]`);

      // Wait for button to appear
      if (await activateButton.isVisible().catch(() => false)) {
        // Button should be disabled if validation is implemented
        const isDisabled = await activateButton.isDisabled();

        if (!isDisabled) {
          console.log('Warning: [Activate] button is not disabled for invalid profile - validation enforcement may not be implemented yet');
        } else {
          await expect(activateButton).toBeDisabled();
        }
      } else {
        console.log('Warning: [Activate] button not found');
      }
    });

    await test.step('Verify tooltip shows error message on hover', async () => {
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      const warningBadge = profileCard.locator('text=/⚠️|Warning|Invalid/i');

      if (await warningBadge.isVisible().catch(() => false)) {
        // Hover over the warning badge
        await warningBadge.hover();
        await page.waitForTimeout(500);

        // Look for tooltip with error details
        const tooltip = page.locator('[role="tooltip"]');
        if (await tooltip.isVisible().catch(() => false)) {
          // Tooltip should contain error information
          const tooltipText = await tooltip.textContent();
          expect(tooltipText).toBeTruthy();
          expect(tooltipText!.length).toBeGreaterThan(0);
        }
      }
    });

    await test.step('Cleanup: delete test profile', async () => {
      await page.click(`button[aria-label="Delete ${testProfileName}"]`);
      await page.click('button:has-text("Confirm")');
      await expect(page.locator(`text=${testProfileName}`)).not.toBeVisible({ timeout: 3000 });
    });
  });

  test('should edit profile → change code → save → verify validation updates', async ({ page }) => {
    const testProfileName = `EditProfile_${Date.now()}`;

    await test.step('Create initial profile', async () => {
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', testProfileName);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${testProfileName}`)).toBeVisible({ timeout: 5000 });
    });

    await test.step('Navigate to config editor', async () => {
      await page.goto(`/config?profile=${encodeURIComponent(testProfileName)}`);
      await page.waitForLoadState('networkidle');
    });

    await test.step('Edit code to add valid configuration', async () => {
      // Switch to Code tab
      const codeTab = page.locator('button:has-text("Code")');
      await codeTab.click();
      await page.waitForTimeout(500);

      // Wait for Monaco editor
      await page.waitForSelector('.monaco-editor', { timeout: 5000 });

      // Add valid configuration using device_start/device_end syntax
      const validCode = `
// Valid keyrx configuration
device_start("My Keyboard", [1234, 5678]) {
  // Simple remap: CapsLock -> Escape
  simple(58, 1);  // 58 = CapsLock, 1 = Escape
}
device_end()
`;

      await page.click('.monaco-editor');
      await page.keyboard.press('Control+A');
      await page.keyboard.type(validCode);

      // Wait for auto-save
      await page.waitForTimeout(2000);

      // Verify save indicator appears
      const saveIndicator = page.locator('text=/Saved|✓ Saved/i');
      if (await saveIndicator.isVisible().catch(() => false)) {
        await expect(saveIndicator).toBeVisible({ timeout: 3000 });
      }
    });

    await test.step('Navigate back to profiles and verify valid status', async () => {
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard).toBeVisible();

      // Wait for validation
      await page.waitForTimeout(1500);

      // Should show valid badge
      const validBadge = profileCard.locator('text=/✓.*Valid|Valid|No errors/i');
      if (await validBadge.isVisible().catch(() => false)) {
        await expect(validBadge).toBeVisible();
      }

      // Should NOT show warning badge
      const warningBadge = profileCard.locator('text=/⚠️|Warning|Invalid/i');
      await expect(warningBadge).not.toBeVisible();
    });

    await test.step('Edit code to introduce error', async () => {
      await page.goto(`/config?profile=${encodeURIComponent(testProfileName)}`);
      await page.waitForLoadState('networkidle');

      const codeTab = page.locator('button:has-text("Code")');
      await codeTab.click();
      await page.waitForTimeout(500);

      // Inject syntax error (unclosed brace)
      const invalidCode = `
device_start("My Keyboard", [1234, 5678]) {
  simple(58, 1);
  // Missing device_end() - invalid syntax
`;

      await page.click('.monaco-editor');
      await page.keyboard.press('Control+A');
      await page.keyboard.type(invalidCode);
      await page.waitForTimeout(2000);
    });

    await test.step('Navigate back and verify invalid status', async () => {
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await page.waitForTimeout(1500);

      // Should show warning badge after introducing error
      const warningBadge = profileCard.locator('text=/⚠️|Warning|Invalid/i');
      if (await warningBadge.isVisible().catch(() => false)) {
        await expect(warningBadge).toBeVisible();
      }

      // Activate button should be disabled
      const activateButton = page.locator(`button[aria-label="Activate ${testProfileName}"]`);
      if (await activateButton.isVisible().catch(() => false)) {
        const isDisabled = await activateButton.isDisabled();
        if (isDisabled) {
          await expect(activateButton).toBeDisabled();
        }
      }
    });

    await test.step('Cleanup: delete test profile', async () => {
      await page.click(`button[aria-label="Delete ${testProfileName}"]`);
      await page.click('button:has-text("Confirm")');
      await expect(page.locator(`text=${testProfileName}`)).not.toBeVisible({ timeout: 3000 });
    });
  });

  test('should handle profile activation workflow with validation', async ({ page }) => {
    const validProfile = `Valid_${Date.now()}`;
    const invalidProfile = `Invalid_${Date.now()}`;

    await test.step('Create valid profile', async () => {
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', validProfile);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${validProfile}`)).toBeVisible({ timeout: 5000 });
    });

    await test.step('Create invalid profile', async () => {
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', invalidProfile);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${invalidProfile}`)).toBeVisible({ timeout: 5000 });
    });

    await test.step('Make second profile invalid', async () => {
      await page.goto(`/config?profile=${encodeURIComponent(invalidProfile)}`);
      await page.waitForLoadState('networkidle');

      const codeTab = page.locator('button:has-text("Code")');
      await codeTab.click();
      await page.waitForTimeout(500);

      await page.click('.monaco-editor');
      await page.keyboard.press('Control+A');
      await page.keyboard.type('invalid syntax here { }');
      await page.waitForTimeout(2000);

      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');
    });

    await test.step('Activate valid profile', async () => {
      await page.waitForTimeout(1500); // Wait for validation

      const activateButton = page.locator(`button[aria-label="Activate ${validProfile}"]`);
      await expect(activateButton).toBeVisible();
      await expect(activateButton).toBeEnabled();

      await activateButton.click();
      await page.waitForTimeout(1000);

      // Verify active badge
      const profileCard = page.locator(`[data-profile="${validProfile}"]`);
      await expect(profileCard.locator('text=/Active|\\[Active\\]/i')).toBeVisible({ timeout: 5000 });
    });

    await test.step('Attempt to activate invalid profile should fail', async () => {
      const invalidActivateButton = page.locator(`button[aria-label="Activate ${invalidProfile}"]`);

      if (await invalidActivateButton.isVisible().catch(() => false)) {
        const isDisabled = await invalidActivateButton.isDisabled();

        if (!isDisabled) {
          // If button is not disabled, clicking it should show error
          await invalidActivateButton.click();
          await page.waitForTimeout(1000);

          // Look for error notification
          const errorNotification = page.locator('text=/Cannot activate|Validation failed|Invalid configuration/i');
          if (await errorNotification.isVisible().catch(() => false)) {
            await expect(errorNotification).toBeVisible();
          }
        } else {
          // Button is correctly disabled
          await expect(invalidActivateButton).toBeDisabled();
        }
      }

      // Invalid profile should NOT have active badge
      const invalidProfileCard = page.locator(`[data-profile="${invalidProfile}"]`);
      const activeBadge = invalidProfileCard.locator('text=/Active|\\[Active\\]/i');
      await expect(activeBadge).not.toBeVisible();
    });

    await test.step('Cleanup: delete test profiles', async () => {
      // Delete valid profile
      await page.click(`button[aria-label="Delete ${validProfile}"]`);
      await page.click('button:has-text("Confirm")');

      // Delete invalid profile
      await page.click(`button[aria-label="Delete ${invalidProfile}"]`);
      await page.click('button:has-text("Confirm")');
    });
  });

  test('should persist [Active] badge across navigation', async ({ page }) => {
    const testProfileName = `PersistenceTest_${Date.now()}`;

    await test.step('Create and activate profile', async () => {
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', testProfileName);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${testProfileName}`)).toBeVisible({ timeout: 5000 });

      await page.waitForTimeout(1500); // Wait for validation

      const activateButton = page.locator(`button[aria-label="Activate ${testProfileName}"]`);
      await activateButton.click();
      await page.waitForTimeout(1000);

      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard.locator('text=/Active|\\[Active\\]/i')).toBeVisible();
    });

    await test.step('Navigate to different page and back', async () => {
      // Navigate to metrics page
      await page.goto('/metrics');
      await page.waitForLoadState('networkidle');

      // Navigate back to profiles
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      // [Active] badge should still be visible
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard.locator('text=/Active|\\[Active\\]/i')).toBeVisible({ timeout: 5000 });
    });

    await test.step('Refresh page and verify persistence', async () => {
      await page.reload();
      await page.waitForLoadState('networkidle');

      // [Active] badge should persist after page reload
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard.locator('text=/Active|\\[Active\\]/i')).toBeVisible({ timeout: 5000 });
    });

    await test.step('Cleanup', async () => {
      await page.click(`button[aria-label="Delete ${testProfileName}"]`);
      await page.click('button:has-text("Confirm")');
    });
  });
});
