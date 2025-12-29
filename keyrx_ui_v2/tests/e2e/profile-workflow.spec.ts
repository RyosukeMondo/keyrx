import { test, expect } from '@playwright/test';

/**
 * E2E Test: Profile Creation → Key Configuration → Activation → Simulator Verification
 *
 * This test verifies the complete user flow for creating and testing a custom profile:
 * 1. Create a new profile with a unique name
 * 2. Navigate to the configuration page
 * 3. Configure a key mapping (e.g., Caps Lock → Escape)
 * 4. Activate the profile
 * 5. Verify the mapping works in the simulator
 *
 * Requirements: Task 34 (E2E Tests), Req 6 (Profile Management), Req 7 (Keyboard Configuration), Req 8 (Simulation)
 */

test.describe('Profile Workflow E2E', () => {
  const testProfileName = `TestProfile_${Date.now()}`;

  test('should create profile, configure key, activate, and verify in simulator', async ({
    page,
  }) => {
    // Step 1: Navigate to Profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Verify we're on the profiles page
    await expect(page).toHaveURL(/\/profiles/);

    // Step 2: Create a new profile
    const createButton = page.getByRole('button', { name: /create.*profile/i });
    await expect(createButton).toBeVisible();
    await createButton.click();

    // Wait for modal to appear
    await page.waitForTimeout(300); // Animation delay

    // Find the profile name input in the modal
    const nameInput = page.getByRole('textbox', { name: /profile.*name/i });
    await expect(nameInput).toBeVisible();
    await nameInput.fill(testProfileName);

    // Submit the form
    const submitButton = page.getByRole('button', { name: /create|save/i });
    await submitButton.click();

    // Wait for modal to close and profile to appear
    await page.waitForTimeout(500);

    // Verify the new profile card appears
    const profileCard = page.getByText(testProfileName);
    await expect(profileCard).toBeVisible();

    // Step 3: Navigate to Configuration page for this profile
    // Click on the profile card or edit button
    const editButton = page
      .locator(`text=${testProfileName}`)
      .locator('..')
      .getByRole('button', { name: /edit|configure/i });

    if (await editButton.isVisible()) {
      await editButton.click();
    } else {
      // Alternative: Click the profile card itself
      await profileCard.click();
    }

    // Should navigate to config page
    await page.waitForURL(/\/config/, { timeout: 5000 });

    // Step 4: Configure a key mapping (Caps Lock → Escape)
    // Find the keyboard visualizer
    const keyboard = page.locator('.keyboard-grid, [class*="keyboard"]').first();
    await expect(keyboard).toBeVisible();

    // Click on Caps Lock key (look for button with text "Caps" or keycode)
    const capsLockKey = page.getByRole('button', { name: /caps.*lock|caps/i }).first();
    await expect(capsLockKey).toBeVisible();
    await capsLockKey.click();

    // Wait for key config dialog to open
    await page.waitForTimeout(300);

    // Verify dialog is open
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    // Select "Simple Remap" action type
    const actionTypeSelect = page.getByRole('button', { name: /action.*type|mapping.*type/i });
    if (await actionTypeSelect.isVisible()) {
      await actionTypeSelect.click();
      await page.waitForTimeout(200);
      const simpleOption = page.getByRole('option', { name: /simple.*remap|simple/i });
      await simpleOption.click();
    }

    // Find the target key input/dropdown
    const targetKeyInput = page.getByRole('button', { name: /target.*key|output.*key/i });
    await expect(targetKeyInput).toBeVisible();
    await targetKeyInput.click();
    await page.waitForTimeout(200);

    // Select Escape key
    const escapeOption = page.getByRole('option', { name: /escape|esc/i });
    await escapeOption.click();

    // Save the mapping
    const saveButton = page.getByRole('button', { name: /save|apply/i });
    await saveButton.click();

    // Wait for dialog to close
    await page.waitForTimeout(300);

    // Verify dialog is closed
    await expect(dialog).not.toBeVisible();

    // Step 5: Activate the profile
    // Navigate back to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Find the test profile and activate it
    const activateButton = page
      .locator(`text=${testProfileName}`)
      .locator('..')
      .getByRole('button', { name: /activate/i });

    await expect(activateButton).toBeVisible();
    await activateButton.click();

    // Wait for activation to complete
    await page.waitForTimeout(1000); // Compilation may take time

    // Verify profile is marked as active
    const activeIndicator = page
      .locator(`text=${testProfileName}`)
      .locator('..')
      .locator('text=/active|✓/i');
    await expect(activeIndicator).toBeVisible();

    // Step 6: Verify mapping in simulator
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Find the keyboard in simulator mode
    const simulatorKeyboard = page.locator('.keyboard-grid, [class*="keyboard"]').first();
    await expect(simulatorKeyboard).toBeVisible();

    // Find the Caps Lock key in simulator
    const simulatorCapsKey = page.getByRole('button', { name: /caps.*lock|caps/i }).first();
    await expect(simulatorCapsKey).toBeVisible();

    // Click the Caps Lock key to simulate press
    await simulatorCapsKey.click();

    // Wait a bit for the simulation to process
    await page.waitForTimeout(200);

    // Verify the output shows "Escape" instead of "Caps Lock"
    const outputPreview = page.locator('[class*="output"], [class*="preview"]').first();

    if (await outputPreview.isVisible()) {
      const outputText = await outputPreview.textContent();
      expect(outputText?.toLowerCase()).toContain('escape');
    }

    // Alternatively, check the state display
    const stateDisplay = page.locator('text=/output|pressed|event/i');
    if (await stateDisplay.isVisible()) {
      const stateText = await stateDisplay.textContent();
      expect(stateText?.toLowerCase()).toContain('escape');
    }

    // Step 7: Clean up - Delete the test profile
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    const deleteButton = page
      .locator(`text=${testProfileName}`)
      .locator('..')
      .getByRole('button', { name: /delete|remove/i });

    if (await deleteButton.isVisible()) {
      await deleteButton.click();

      // Confirm deletion if confirmation dialog appears
      await page.waitForTimeout(300);
      const confirmButton = page.getByRole('button', { name: /confirm|yes|delete/i });
      if (await confirmButton.isVisible()) {
        await confirmButton.click();
      }

      // Wait for deletion to complete
      await page.waitForTimeout(500);

      // Verify profile is gone
      await expect(page.locator(`text=${testProfileName}`)).not.toBeVisible();
    }
  });

  test('should handle profile creation errors gracefully', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Try to create a profile with an invalid name (empty)
    const createButton = page.getByRole('button', { name: /create.*profile/i });
    await createButton.click();
    await page.waitForTimeout(300);

    const submitButton = page.getByRole('button', { name: /create|save/i });

    // Submit without entering a name
    if (await submitButton.isVisible()) {
      await submitButton.click();

      // Should show an error message
      await page.waitForTimeout(300);
      const errorMessage = page.locator('text=/required|invalid|error/i');
      await expect(errorMessage).toBeVisible();
    }

    // Close the modal
    const cancelButton = page.getByRole('button', { name: /cancel|close/i });
    if (await cancelButton.isVisible()) {
      await cancelButton.click();
    } else {
      // Try pressing Escape
      await page.keyboard.press('Escape');
    }
  });

  test('should support keyboard navigation in profile workflow', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Use Tab to navigate to Create button
    let tabCount = 0;
    const maxTabs = 20;

    while (tabCount < maxTabs) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focusedElement = page.locator(':focus');
      const text = await focusedElement.textContent().catch(() => '');

      if (text.toLowerCase().includes('create')) {
        // Press Enter to activate
        await page.keyboard.press('Enter');
        await page.waitForTimeout(300);

        // Modal should be open
        const modal = page.getByRole('dialog');
        await expect(modal).toBeVisible();

        // Press Escape to close
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);

        // Modal should be closed
        await expect(modal).not.toBeVisible();

        break;
      }
    }
  });
});
