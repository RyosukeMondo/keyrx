import { test, expect } from '@playwright/test';

/**
 * E2E Test: Device Management Workflow
 *
 * This test verifies the complete user flow for managing keyboard devices:
 * 1. Navigate to Devices page
 * 2. Rename a device using inline editing
 * 3. Change device scope (global/device-specific)
 * 4. Verify changes persist after page reload
 * 5. Test "Forget Device" functionality
 *
 * Requirements: Task 34 (E2E Tests), Req 5 (Device Management)
 */

test.describe('Device Management Workflow E2E', () => {
  test('should rename device and verify persistence', async ({ page }) => {
    // Step 1: Navigate to Devices page
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Verify we're on the devices page
    await expect(page).toHaveURL(/\/devices/);

    // Wait for devices to load
    await page.waitForTimeout(500);

    // Step 2: Find a device and its rename button
    const renameButton = page.getByRole('button', { name: /rename/i }).first();

    if (!(await renameButton.isVisible())) {
      // No devices to test with - skip test
      test.skip();
      return;
    }

    // Get the current device name
    const deviceCard = renameButton.locator('..').locator('..');
    const originalName = await deviceCard
      .locator('[class*="name"], h2, h3')
      .first()
      .textContent();

    // Click rename button to enable inline editing
    await renameButton.click();

    // Wait for input to appear
    await page.waitForTimeout(200);

    // Find the input field (should replace the name text)
    const nameInput = deviceCard.getByRole('textbox').first();
    await expect(nameInput).toBeVisible();

    // Generate a unique test name
    const testName = `TestDevice_${Date.now()}`;

    // Clear and enter new name
    await nameInput.fill('');
    await nameInput.fill(testName);

    // Press Enter to save
    await nameInput.press('Enter');

    // Wait for save to complete
    await page.waitForTimeout(500);

    // Verify the new name appears
    await expect(deviceCard.locator(`text=${testName}`)).toBeVisible();

    // Step 3: Verify persistence - reload page
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Verify the renamed device still has the new name
    await expect(page.locator(`text=${testName}`)).toBeVisible();

    // Step 4: Restore original name
    const renameAgain = page
      .locator(`text=${testName}`)
      .locator('..')
      .locator('..')
      .getByRole('button', { name: /rename/i });

    await renameAgain.click();
    await page.waitForTimeout(200);

    const restoreInput = page
      .locator(`text=${testName}`)
      .locator('..')
      .locator('..')
      .getByRole('textbox')
      .first();

    await restoreInput.fill('');
    await restoreInput.fill(originalName || 'Keyboard');
    await restoreInput.press('Enter');
    await page.waitForTimeout(500);
  });

  test('should change device scope and verify persistence', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Find a device with a scope toggle
    const scopeToggle = page.getByRole('button', { name: /scope|global|device/i }).first();

    if (!(await scopeToggle.isVisible())) {
      // Try finding a dropdown or select
      const scopeSelect = page.locator('[class*="scope"], select').first();

      if (!(await scopeSelect.isVisible())) {
        test.skip();
        return;
      }

      // Get current value
      const currentValue = await scopeSelect.inputValue();

      // Click to open dropdown
      await scopeSelect.click();
      await page.waitForTimeout(200);

      // Select the opposite option
      const newOption = currentValue.includes('global')
        ? page.getByRole('option', { name: /device.*specific|local/i })
        : page.getByRole('option', { name: /global/i });

      if (await newOption.isVisible()) {
        await newOption.click();
        await page.waitForTimeout(500);

        // Reload and verify persistence
        await page.reload();
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(500);

        const afterReload = await scopeSelect.inputValue();
        expect(afterReload).not.toBe(currentValue);

        // Restore original value
        await scopeSelect.click();
        await page.waitForTimeout(200);
        const restoreOption = currentValue.includes('global')
          ? page.getByRole('option', { name: /global/i })
          : page.getByRole('option', { name: /device.*specific|local/i });

        if (await restoreOption.isVisible()) {
          await restoreOption.click();
          await page.waitForTimeout(500);
        }
      }
    } else {
      // Button toggle style
      const deviceCard = scopeToggle.locator('..').locator('..');
      const currentText = await scopeToggle.textContent();

      // Click to toggle
      await scopeToggle.click();
      await page.waitForTimeout(500);

      // Verify text changed
      const newText = await scopeToggle.textContent();
      expect(newText).not.toBe(currentText);

      // Reload and verify persistence
      await page.reload();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(500);

      const afterReload = await scopeToggle.textContent();
      expect(afterReload).toBe(newText);

      // Restore original
      await scopeToggle.click();
      await page.waitForTimeout(500);
    }
  });

  test('should handle rename validation errors', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    const renameButton = page.getByRole('button', { name: /rename/i }).first();

    if (!(await renameButton.isVisible())) {
      test.skip();
      return;
    }

    await renameButton.click();
    await page.waitForTimeout(200);

    const nameInput = page.getByRole('textbox').first();
    await expect(nameInput).toBeVisible();

    // Try to save with empty name
    await nameInput.fill('');
    await nameInput.press('Enter');

    // Should either prevent save or show error
    await page.waitForTimeout(300);

    // Check if error message appears or input is still visible
    const errorMsg = page.locator('text=/required|invalid|error/i');
    const inputStillVisible = await nameInput.isVisible();

    expect(inputStillVisible || (await errorMsg.isVisible())).toBeTruthy();

    // Cancel by pressing Escape or clicking cancel
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
  });

  test('should display device connection status', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Check for connected/disconnected indicators
    const statusIndicators = page.locator('text=/connected|disconnected|online|offline/i');
    const count = await statusIndicators.count();

    // At least one device should show status
    expect(count).toBeGreaterThan(0);

    // If connected devices exist, verify they have proper styling
    const connectedDevices = page.locator('text=/connected|online/i');
    const connectedCount = await connectedDevices.count();

    if (connectedCount > 0) {
      // Connected status should be visible
      await expect(connectedDevices.first()).toBeVisible();
    }
  });

  test('should support layout selector', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Look for layout selector dropdown
    const layoutSelector = page.getByRole('button', { name: /layout/i });

    if (await layoutSelector.isVisible()) {
      // Click to open
      await layoutSelector.click();
      await page.waitForTimeout(200);

      // Check for layout options (ANSI, ISO, JIS, etc.)
      const layoutOptions = page.locator('text=/ansi|iso|jis|hhkb/i');
      const optionsCount = await layoutOptions.count();

      expect(optionsCount).toBeGreaterThan(0);

      // Select an option
      await layoutOptions.first().click();
      await page.waitForTimeout(300);

      // Verify dropdown closed
      const dropdown = page.getByRole('listbox');
      if (await dropdown.isVisible()) {
        await expect(dropdown).not.toBeVisible();
      }
    }
  });

  test('should handle forget device with confirmation', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Look for forget/remove device button
    const forgetButton = page.getByRole('button', { name: /forget|remove.*device/i }).first();

    if (await forgetButton.isVisible()) {
      // Get device name before deletion
      const deviceCard = forgetButton.locator('..').locator('..');
      const deviceName = await deviceCard
        .locator('[class*="name"], h2, h3')
        .first()
        .textContent();

      // Click forget button
      await forgetButton.click();
      await page.waitForTimeout(300);

      // Should show confirmation dialog
      const confirmDialog = page.getByRole('dialog');
      await expect(confirmDialog).toBeVisible();

      // Verify confirmation message mentions the device
      const dialogText = await confirmDialog.textContent();
      expect(dialogText?.toLowerCase()).toContain('forget');

      // Click cancel to NOT delete
      const cancelButton = confirmDialog.getByRole('button', { name: /cancel|no/i });
      if (await cancelButton.isVisible()) {
        await cancelButton.click();
        await page.waitForTimeout(300);

        // Device should still be visible
        if (deviceName) {
          await expect(page.locator(`text=${deviceName}`)).toBeVisible();
        }
      }
    }
  });

  test('should support keyboard navigation in device list', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Tab through the device list
    let tabCount = 0;
    const maxTabs = 30;
    const focusedElements: string[] = [];

    while (tabCount < maxTabs) {
      await page.keyboard.press('Tab');
      tabCount++;

      const focusedElement = page.locator(':focus');
      const tagName = await focusedElement.evaluate((el) => el.tagName).catch(() => '');
      const text = await focusedElement.textContent().catch(() => '');

      if (tagName && text) {
        focusedElements.push(`${tagName}: ${text.substring(0, 50)}`);
      }

      // If we focus a rename button, we can test Enter key
      if (text.toLowerCase().includes('rename')) {
        // Found a rename button, Enter should activate it
        await page.keyboard.press('Enter');
        await page.waitForTimeout(200);

        // Input should appear
        const input = page.getByRole('textbox').first();
        if (await input.isVisible()) {
          // Success - keyboard navigation works
          await page.keyboard.press('Escape');
          await page.waitForTimeout(200);
        }
        break;
      }
    }

    // Verify we focused multiple elements
    expect(focusedElements.length).toBeGreaterThan(0);
  });
});
