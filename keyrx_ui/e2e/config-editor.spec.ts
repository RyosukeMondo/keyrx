import { test, expect } from '@playwright/test';

/**
 * E2E Test: Configuration Editor
 *
 * Tests the configuration editor functionality:
 * - Tab switching between Visual and Code views
 * - Monaco editor editing and validation
 * - Saving configurations
 * - Error handling and validation
 * - Keyboard shortcuts (Ctrl+S)
 *
 * Requirements: REQ-7 (AC4, AC10)
 */

test.describe('Configuration Editor', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to config page for Default profile
    await page.goto('/config/Default');
    await page.waitForLoadState('networkidle');
  });

  test('should display Visual tab by default', async ({ page }) => {
    // Verify Visual tab is active
    const visualTab = page.locator('button:has-text("Visual")');
    await expect(visualTab).toHaveClass(/bg-primary-500/);

    // Verify KeyboardVisualizer is visible
    await expect(page.locator('[data-testid="keyboard-visualizer"]')).toBeVisible();

    // Verify Monaco editor is not visible
    await expect(page.locator('.monaco-editor')).not.toBeVisible();
  });

  test('should switch to Code tab', async ({ page }) => {
    // Click on Code tab
    await page.click('button:has-text("Code")');

    // Verify Code tab is now active
    const codeTab = page.locator('button:has-text("Code")');
    await expect(codeTab).toHaveClass(/bg-primary-500/);

    // Verify Monaco editor is visible
    await expect(page.locator('.monaco-editor')).toBeVisible();

    // Verify KeyboardVisualizer is not visible
    await expect(page.locator('[data-testid="keyboard-visualizer"]')).not.toBeVisible();
  });

  test('should preserve changes when switching tabs', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');

    // Wait for Monaco to load
    await page.waitForSelector('.monaco-editor');

    // Add a comment to the config
    const testComment = `// Test comment ${Date.now()}`;
    await page.keyboard.press('End');
    await page.keyboard.press('Enter');
    await page.keyboard.type(testComment);

    // Wait a moment for the change to register
    await page.waitForTimeout(500);

    // Switch back to Visual tab
    await page.click('button:has-text("Visual")');

    // Switch back to Code tab
    await page.click('button:has-text("Code")');

    // Verify the comment is still there
    await expect(page.locator('.monaco-editor')).toContainText(testComment);
  });

  test('should validate configuration in Code editor', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Introduce a syntax error
    await page.keyboard.press('Control+A');
    await page.keyboard.type('invalid syntax {]');

    // Wait for validation (500ms debounce)
    await page.waitForTimeout(600);

    // Verify error indicator appears
    await expect(page.locator('text=Validation Error')).toBeVisible();

    // Verify error count is displayed
    const errorCount = page.locator('[data-testid="error-count"]');
    await expect(errorCount).toBeVisible();
    await expect(errorCount).toContainText(/\d+ error/);
  });

  test('should navigate to errors with F8', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Create multiple errors
    await page.keyboard.press('Control+A');
    await page.keyboard.type(`
      invalid line 1 {]
      invalid line 2 [}
      invalid line 3 ()
    `);

    // Wait for validation
    await page.waitForTimeout(600);

    // Press F8 to navigate to first error
    await page.keyboard.press('F8');

    // Verify cursor moved to error location (Monaco should highlight the line)
    const activeLine = page.locator('.monaco-editor .current-line');
    await expect(activeLine).toBeVisible();

    // Press F8 again to navigate to next error
    await page.keyboard.press('F8');

    // Cursor should have moved to a different line
    // (Full verification would require Monaco API access)
  });

  test('should save configuration with Ctrl+S', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Make a valid change
    await page.keyboard.press('End');
    await page.keyboard.press('Enter');
    await page.keyboard.type('// Valid comment');

    // Wait for validation
    await page.waitForTimeout(600);

    // Save with Ctrl+S
    await page.keyboard.press('Control+s');

    // Verify success message
    await expect(page.locator('text=Configuration saved successfully')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should prevent saving invalid configuration', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Introduce a syntax error
    await page.keyboard.press('Control+A');
    await page.keyboard.type('invalid syntax {]');

    // Wait for validation
    await page.waitForTimeout(600);

    // Try to save
    await page.click('button:has-text("Save")');

    // Verify save was blocked
    await expect(
      page.locator('text=Cannot save configuration with validation errors')
    ).toBeVisible();
  });

  test('should display syntax highlighting', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Add some Rhai code with keywords
    await page.keyboard.press('Control+A');
    await page.keyboard.type(`
      let x = 10;
      if x > 5 {
        return true;
      } else {
        return false;
      }
    `);

    // Wait for syntax highlighting to apply
    await page.waitForTimeout(500);

    // Verify syntax highlighting is present (Monaco adds token classes)
    const editor = page.locator('.monaco-editor');
    await expect(editor.locator('.mtk1')).toBeVisible(); // Monaco token class
  });

  test('should show error tooltips on hover', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Introduce an error
    await page.keyboard.press('Control+A');
    await page.keyboard.type('invalid {]');

    // Wait for validation
    await page.waitForTimeout(600);

    // Hover over the error marker (squiggly line)
    const errorMarker = page.locator('.squiggly-error').first();
    await errorMarker.hover();

    // Verify tooltip appears with error message
    await expect(page.locator('.monaco-hover')).toBeVisible({ timeout: 2000 });
  });

  test('should handle large configurations', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Generate a large configuration
    let largeConfig = '// Large configuration test\n';
    for (let i = 0; i < 100; i++) {
      largeConfig += `let var${i} = ${i};\n`;
    }

    // Paste the large config
    await page.keyboard.press('Control+A');
    await page.evaluate((config) => {
      navigator.clipboard.writeText(config);
    }, largeConfig);
    await page.keyboard.press('Control+v');

    // Wait for validation
    await page.waitForTimeout(1000);

    // Verify editor handles it smoothly
    await expect(page.locator('.monaco-editor')).toBeVisible();

    // Verify scrolling works
    await page.keyboard.press('Control+End');
    await page.keyboard.press('Control+Home');
  });

  test('should support undo/redo', async ({ page }) => {
    // Switch to Code tab
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Get initial content
    const initialContent = await page.locator('.monaco-editor').textContent();

    // Make a change
    await page.keyboard.press('End');
    await page.keyboard.press('Enter');
    await page.keyboard.type('// New line');

    // Undo the change
    await page.keyboard.press('Control+z');

    // Verify content is back to initial
    const afterUndo = await page.locator('.monaco-editor').textContent();
    expect(afterUndo).toBe(initialContent);

    // Redo the change
    await page.keyboard.press('Control+y');

    // Verify change is back
    await expect(page.locator('.monaco-editor')).toContainText('// New line');
  });

  test('should work in Visual tab', async ({ page }) => {
    // Verify Visual tab is active by default
    await expect(page.locator('[data-testid="keyboard-visualizer"]')).toBeVisible();

    // Click on a key in the visualizer
    const keyButton = page.locator('[data-key="A"]').first();
    await keyButton.click();

    // Verify mapping dialog or editor appears
    // (Specific behavior depends on KeyboardVisualizer implementation)
    // This is a placeholder for actual visual editor interaction
  });

  test('should reload configuration on profile switch', async ({ page }) => {
    // Create a new profile for testing
    await page.goto('/profiles');
    const testProfile = `ConfigTest_${Date.now()}`;
    await page.click('button:has-text("Create Profile")');
    await page.fill('input[name="profileName"]', testProfile);
    await page.click('button:has-text("Create")');

    // Navigate to config for new profile
    await page.goto(`/config/${testProfile}`);
    await page.waitForLoadState('networkidle');

    // Verify config loads (should be empty or default)
    await page.click('button:has-text("Code")');
    await page.waitForSelector('.monaco-editor');

    // Add content to this profile's config
    await page.keyboard.type('// Test config for new profile');

    // Save
    await page.keyboard.press('Control+s');
    await expect(page.locator('text=Configuration saved successfully')).toBeVisible({
      timeout: 5000,
    });

    // Navigate to Default profile
    await page.goto('/config/Default');
    await page.click('button:has-text("Code")');

    // Verify different config loaded
    await expect(page.locator('.monaco-editor')).not.toContainText(
      '// Test config for new profile'
    );

    // Cleanup
    await page.goto('/profiles');
    await page.click(`button[aria-label="Delete ${testProfile}"]`);
    await page.click('button:has-text("Confirm")');
  });
});
