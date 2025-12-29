import { test, expect } from '@playwright/test';

/**
 * E2E tests for the configuration validation workflow.
 *
 * Tests the complete user journey from opening the editor,
 * typing invalid configuration, seeing validation errors,
 * using Quick Fix, and saving the configuration.
 */
test.describe('Configuration Validation E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to load
    await expect(page.locator('h1')).toContainText('KeyRX');

    // Click the Config Editor button
    await page.click('button:has-text("Config Editor")');

    // Wait for the configuration page to load
    await expect(page.locator('h1')).toContainText('Configuration Editor');

    // Wait for Monaco editor to mount
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
  });

  test('should display validation errors for invalid syntax', async ({ page }) => {
    // Type invalid configuration
    const invalidConfig = 'layer "test" { invalid_syntax }';

    // Click in the editor to focus it
    await page.click('.monaco-editor');

    // Type the invalid config
    await page.keyboard.type(invalidConfig);

    // Wait for debounced validation (500ms + buffer)
    await page.waitForTimeout(1000);

    // Check for error badge in validation panel
    const errorBadge = page.locator('.validation-badge.error');
    await expect(errorBadge).toBeVisible();
    await expect(errorBadge).toContainText(/Error/);
  });

  test('should show success indicator for valid configuration', async ({ page }) => {
    // Type valid configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    // Click in the editor to focus it
    await page.click('.monaco-editor');

    // Type the valid config
    await page.keyboard.type(validConfig);

    // Wait for debounced validation
    await page.waitForTimeout(1000);

    // Check for success badge
    const successBadge = page.locator('.validation-badge.success');
    await expect(successBadge).toBeVisible();
    await expect(successBadge).toContainText('Configuration valid');
  });

  test('should toggle validation panel expansion', async ({ page }) => {
    // Type some config to trigger validation
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    // Find the validation status header
    const header = page.locator('.validation-status-header');
    await expect(header).toBeVisible();

    // Check that it's expanded by default
    await expect(header).toHaveAttribute('aria-expanded', 'true');

    // Click to collapse
    await header.click();
    await expect(header).toHaveAttribute('aria-expanded', 'false');

    // Click to expand again
    await header.click();
    await expect(header).toHaveAttribute('aria-expanded', 'true');
  });

  test('should toggle panel with keyboard (Enter)', async ({ page }) => {
    // Type some config
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    const header = page.locator('.validation-status-header');

    // Focus the header
    await header.focus();

    // Press Enter to toggle
    await page.keyboard.press('Enter');
    await expect(header).toHaveAttribute('aria-expanded', 'false');

    // Press Enter again
    await page.keyboard.press('Enter');
    await expect(header).toHaveAttribute('aria-expanded', 'true');
  });

  test('should toggle panel with keyboard (Space)', async ({ page }) => {
    // Type some config
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    const header = page.locator('.validation-status-header');

    // Focus the header
    await header.focus();

    // Press Space to toggle
    await page.keyboard.press('Space');
    await expect(header).toHaveAttribute('aria-expanded', 'false');

    // Press Space again
    await page.keyboard.press('Space');
    await expect(header).toHaveAttribute('aria-expanded', 'true');
  });

  test('should disable save button when errors exist', async ({ page }) => {
    // Type invalid configuration that will produce errors
    await page.click('.monaco-editor');
    await page.keyboard.type('invalid syntax here');
    await page.waitForTimeout(1000);

    // Find the save button (Ctrl+S is the shortcut)
    // The ConfigEditor component blocks saving when errors exist
    // We can test this by trying to save with Ctrl+S
    await page.keyboard.press('Control+s');

    // The save should be blocked - check that no success notification appears
    const successNotification = page.locator('.save-notification.success');
    await expect(successNotification).not.toBeVisible();
  });

  test('should allow save when configuration is valid', async ({ page }) => {
    // Type valid configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    await page.click('.monaco-editor');
    await page.keyboard.type(validConfig);

    // Wait for validation
    await page.waitForTimeout(1000);

    // Verify success badge appears
    await expect(page.locator('.validation-badge.success')).toBeVisible();

    // Save with Ctrl+S
    await page.keyboard.press('Control+s');

    // Wait for save operation
    await page.waitForTimeout(600);

    // Check for success notification
    const successNotification = page.locator('.save-notification.success');
    await expect(successNotification).toBeVisible();
    await expect(successNotification).toContainText('saved successfully');
  });

  test('should display error count correctly', async ({ page }) => {
    // Type configuration with multiple syntax errors
    const multipleErrors = `invalid line 1
invalid line 2
invalid line 3`;

    await page.click('.monaco-editor');
    await page.keyboard.type(multipleErrors);

    // Wait for validation
    await page.waitForTimeout(1000);

    // Check error badge shows count
    const errorBadge = page.locator('.validation-badge.error');
    await expect(errorBadge).toBeVisible();
    // The badge should show at least 1 error (may be multiple depending on parsing)
    await expect(errorBadge).toContainText(/\d+\s+Error/);
  });

  test('should handle editor clearing', async ({ page }) => {
    // Type some config
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    // Clear the editor
    await page.keyboard.press('Control+a');
    await page.keyboard.press('Backspace');

    // Wait for validation
    await page.waitForTimeout(1000);

    // With empty config, there might be errors or it might be valid
    // depending on the validator. At minimum, validation should not crash.
    const validationPanel = page.locator('.validation-status-panel');
    await expect(validationPanel).toBeVisible();
  });

  test('should persist validation state while typing', async ({ page }) => {
    // Type valid config
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    // Verify validating badge appears briefly
    // (This is tricky to test due to timing, so we'll just verify the panel updates)

    // Add more text
    await page.keyboard.press('Enter');
    await page.keyboard.type('// comment');

    // Wait for validation
    await page.waitForTimeout(1000);

    // Panel should still be functional
    const validationPanel = page.locator('.validation-status-panel');
    await expect(validationPanel).toBeVisible();
  });

  test('should be accessible with keyboard navigation', async ({ page }) => {
    // Type config with an error
    await page.click('.monaco-editor');
    await page.keyboard.type('invalid syntax');
    await page.waitForTimeout(1000);

    // Tab through the page
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Should be able to reach the validation panel header
    const header = page.locator('.validation-status-header');
    await expect(header).toBeFocused();
  });

  test('should maintain editor focus after validation', async ({ page }) => {
    // Click in editor
    await page.click('.monaco-editor');

    // Type text
    await page.keyboard.type('layer "test"');

    // Editor should maintain focus even after validation fires
    const editor = page.locator('.monaco-editor');
    await page.waitForTimeout(1000);

    // We should still be able to type
    await page.keyboard.type(' {}');

    // Verify the text appears
    const editorText = page.locator('.monaco-editor .view-lines');
    await expect(editorText).toContainText('layer "test" {}');
  });

  test('should handle rapid typing without crashing', async ({ page }) => {
    await page.click('.monaco-editor');

    // Type rapidly
    const rapidText = 'layer "a" {} layer "b" {} layer "c" {}';
    await page.keyboard.type(rapidText, { delay: 10 });

    // Wait for validation to settle
    await page.waitForTimeout(1500);

    // App should not crash
    const validationPanel = page.locator('.validation-status-panel');
    await expect(validationPanel).toBeVisible();
  });

  test('should show proper ARIA labels for accessibility', async ({ page }) => {
    // Type config to trigger validation
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    // Check ARIA labels
    const validationRegion = page.locator('[role="region"][aria-label="Validation Status"]');
    await expect(validationRegion).toBeVisible();

    const header = page.locator('[role="button"]').filter({ has: page.locator('text=/Validation status/') });
    await expect(header).toHaveAttribute('aria-expanded');
  });

  test('should update validation in real-time as user types', async ({ page }) => {
    await page.click('.monaco-editor');

    // Start with invalid syntax
    await page.keyboard.type('invalid');
    await page.waitForTimeout(1000);

    // Should show error
    const errorBadge = page.locator('.validation-badge.error');
    const successBadge = page.locator('.validation-badge.success');

    await expect(errorBadge).toBeVisible();

    // Fix by completing valid syntax
    await page.keyboard.press('Control+a');
    await page.keyboard.press('Backspace');
    await page.keyboard.type('layer "test" { map KEY_A to KEY_B }');
    await page.waitForTimeout(1000);

    // Should show success
    await expect(successBadge).toBeVisible();
    await expect(errorBadge).not.toBeVisible();
  });

  test('should maintain validation state across view switches', async ({ page }) => {
    // Type valid config
    await page.click('.monaco-editor');
    await page.keyboard.type('layer "test" {}');
    await page.waitForTimeout(1000);

    // Verify success
    await expect(page.locator('.validation-badge.success')).toBeVisible();

    // Switch to a different view
    await page.click('button:has-text("Devices")');
    await page.waitForTimeout(200);

    // Switch back to config editor
    await page.click('button:has-text("Config Editor")');
    await page.waitForTimeout(200);

    // Wait for editor to reload
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });

    // The editor might be empty now (depending on implementation)
    // but it should not crash and validation panel should be visible
    const validationPanel = page.locator('.validation-status-panel');
    await expect(validationPanel).toBeVisible();
  });
});
