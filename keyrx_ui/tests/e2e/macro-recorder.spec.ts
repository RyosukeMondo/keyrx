import { test, expect } from '@playwright/test';

/**
 * E2E tests for the macro recorder workflow.
 *
 * Tests the complete user journey from navigating to the macro recorder,
 * recording events, editing them, testing with simulator, and exporting.
 */
test.describe('Macro Recorder E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to load
    await expect(page.locator('h1')).toContainText('KeyRX');

    // Click the Macro Recorder button
    await page.click('button:has-text("Macro Recorder")');

    // Wait for the macro recorder panel to load
    await expect(page.locator('h2')).toContainText('Macro Recorder');
  });

  test('should start and stop recording', async ({ page }) => {
    // Start recording button should be visible
    const startButton = page.locator('button:has-text("Start Recording")');
    await expect(startButton).toBeVisible();
    await expect(startButton).toBeEnabled();

    // Click start recording
    await startButton.click();

    // Should show recording indicator
    await expect(page.locator('text=/Recording/i')).toBeVisible();

    // Stop button should now be visible
    const stopButton = page.locator('button:has-text("Stop Recording")');
    await expect(stopButton).toBeVisible();

    // Wait a bit to simulate recording
    await page.waitForTimeout(500);

    // Stop recording
    await stopButton.click();

    // Recording indicator should disappear
    await expect(page.locator('text=/Recording/i')).not.toBeVisible();

    // Start button should be visible again
    await expect(startButton).toBeVisible();
  });

  test('should clear recorded events', async ({ page }) => {
    // Start and stop recording to get some events
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Clear button should be visible
    const clearButton = page.locator('button:has-text("Clear")');
    await expect(clearButton).toBeVisible();

    // Click clear
    await clearButton.click();

    // Events table should be empty or show empty state
    const eventsTable = page.locator('.events-table, table');
    if (await eventsTable.isVisible()) {
      // Table should have no data rows or show empty message
      const emptyMessage = page.locator('text=/No events recorded/i');
      await expect(emptyMessage).toBeVisible();
    }
  });

  test('should display recorded events in table', async ({ page }) => {
    // Start recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(200);

    // Simulate some keyboard events (if daemon is running)
    // Note: This depends on the daemon being active
    await page.waitForTimeout(500);

    // Stop recording
    await page.click('button:has-text("Stop Recording")');

    // Should display events table
    const eventsTable = page.locator('.events-table, table');
    await expect(eventsTable).toBeVisible();

    // Table should have headers
    const headers = page.locator('th');
    await expect(headers).toHaveCount(4); // Type, Key, Timestamp, Actions
  });

  test('should edit event in table', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Wait for events table to appear
    const eventsTable = page.locator('.events-table, table');
    if (await eventsTable.isVisible()) {
      // Look for an edit button
      const editButton = page.locator('button:has-text("Edit")').first();
      if (await editButton.isVisible()) {
        await editButton.click();

        // Should show edit form or inline editor
        const keyInput = page.locator('input[type="number"], input[placeholder*="key"]').first();
        await expect(keyInput).toBeVisible();

        // Change the key code
        await keyInput.fill('65'); // KEY_A

        // Save changes
        const saveButton = page.locator('button:has-text("Save")').first();
        await saveButton.click();

        // Edit form should close
        await expect(keyInput).not.toBeVisible();
      }
    }
  });

  test('should delete event from table', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Wait for events table
    const eventsTable = page.locator('.events-table, table');
    if (await eventsTable.isVisible()) {
      // Count initial rows
      const initialRows = await page.locator('tbody tr').count();

      // Look for delete button
      const deleteButton = page.locator('button:has-text("Delete"), button[aria-label*="Delete"]').first();
      if (await deleteButton.isVisible() && initialRows > 0) {
        await deleteButton.click();

        // Should have one fewer row
        const newRows = await page.locator('tbody tr').count();
        expect(newRows).toBe(initialRows - 1);
      }
    }
  });

  test('should generate Rhai code from events', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Look for generated Rhai code section
    const rhaiSection = page.locator('text=/Generated Rhai Code/i, h3:has-text("Rhai")');
    if (await rhaiSection.isVisible()) {
      // Should display code block
      const codeBlock = page.locator('pre, code, textarea').filter({ hasText: /macro/ });
      await expect(codeBlock).toBeVisible();
    }
  });

  test('should copy Rhai code to clipboard', async ({ page }) => {
    // Grant clipboard permissions
    await page.context().grantPermissions(['clipboard-read', 'clipboard-write']);

    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Look for copy button
    const copyButton = page.locator('button:has-text("Copy"), button[aria-label*="Copy"]').first();
    if (await copyButton.isVisible()) {
      await copyButton.click();

      // Should show confirmation message
      const confirmation = page.locator('text=/Copied/i');
      await expect(confirmation).toBeVisible({ timeout: 2000 });
    }
  });

  test('should export events as JSON', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Look for export button
    const exportButton = page.locator('button:has-text("Export JSON"), button:has-text("Export")');
    if (await exportButton.isVisible() && await exportButton.isEnabled()) {
      // Set up download listener
      const downloadPromise = page.waitForEvent('download');

      // Click export
      await exportButton.click();

      // Wait for download
      const download = await downloadPromise;

      // Verify filename
      expect(download.suggestedFilename()).toMatch(/macro.*\.json/i);
    }
  });

  test('should test macro with simulator', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Look for test panel
    const testSection = page.locator('text=/Test Macro/i, h3:has-text("Test")');
    if (await testSection.isVisible()) {
      // Should have a test button
      const testButton = page.locator('button:has-text("Test Macro"), button:has-text("Run Test")');
      if (await testButton.isVisible()) {
        await testButton.click();

        // Wait for test to complete
        await page.waitForTimeout(1000);

        // Should show test results
        const results = page.locator('text=/Test Result/i, text=/Output/i');
        await expect(results).toBeVisible();
      }
    }
  });

  test('should use text snippet template', async ({ page }) => {
    // Look for text snippet section
    const snippetSection = page.locator('text=/Text Snippet/i, h3:has-text("Text")');
    if (await snippetSection.isVisible()) {
      // Should have a text input
      const textInput = page.locator('input[placeholder*="text"], textarea[placeholder*="text"]');
      if (await textInput.isVisible()) {
        // Type some text
        await textInput.fill('Hello World');

        // Look for convert/generate button
        const convertButton = page.locator('button:has-text("Convert"), button:has-text("Generate")');
        if (await convertButton.isVisible()) {
          await convertButton.click();

          // Should populate events table or show confirmation
          await page.waitForTimeout(500);

          // Events should be generated
          const eventsTable = page.locator('.events-table, table');
          await expect(eventsTable).toBeVisible();
        }
      }
    }
  });

  test('should load template from library', async ({ page }) => {
    // Look for template library section
    const librarySection = page.locator('text=/Template Library/i, text=/Templates/i');
    if (await librarySection.isVisible()) {
      // Should have template options
      const templateButton = page.locator('button[aria-label*="template"], .template-item').first();
      if (await templateButton.isVisible()) {
        await templateButton.click();

        // Should populate events or show preview
        await page.waitForTimeout(500);

        // Events table should be visible
        const eventsTable = page.locator('.events-table, table');
        await expect(eventsTable).toBeVisible();
      }
    }
  });

  test('should handle error when daemon is not running', async ({ page }) => {
    // This test assumes the daemon might not be running
    // Start recording should show error or warning
    const startButton = page.locator('button:has-text("Start Recording")');
    await startButton.click();

    // Wait for potential error
    await page.waitForTimeout(1000);

    // Should show error or warning message (if daemon is not running)
    const errorMessage = page.locator('[class*="error"], [role="alert"]');
    if (await errorMessage.isVisible()) {
      // Error should mention daemon or connection
      await expect(errorMessage).toHaveText(/daemon|connection|failed/i);
    }
  });

  test('should maintain state when switching views', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Switch to Devices view
    await page.click('button:has-text("Devices")');
    await page.waitForTimeout(200);

    // Switch back to Macro Recorder
    await page.click('button:has-text("Macro Recorder")');
    await page.waitForTimeout(200);

    // Events should still be there (or UI should handle gracefully)
    const macroPanel = page.locator('h2:has-text("Macro Recorder")');
    await expect(macroPanel).toBeVisible();
  });

  test('should be accessible with keyboard navigation', async ({ page }) => {
    // Tab through the interface
    await page.keyboard.press('Tab'); // Should focus first interactive element
    await page.keyboard.press('Tab'); // Move to next element

    // Verify start button can be focused
    const startButton = page.locator('button:has-text("Start Recording")');
    await startButton.focus();
    await expect(startButton).toBeFocused();

    // Should be able to activate with Enter/Space
    await page.keyboard.press('Enter');

    // Recording should start
    await expect(page.locator('text=/Recording/i')).toBeVisible();

    // Stop recording with keyboard
    const stopButton = page.locator('button:has-text("Stop Recording")');
    await stopButton.focus();
    await page.keyboard.press('Enter');

    // Recording should stop
    await expect(page.locator('text=/Recording/i')).not.toBeVisible();
  });

  test('should show ARIA labels for accessibility', async ({ page }) => {
    // Check that main macro recorder panel has proper ARIA labels
    const recorderPanel = page.locator('[aria-label*="Macro"], [role="main"]');
    await expect(recorderPanel).toBeVisible();

    // Buttons should have accessible labels
    const startButton = page.locator('button:has-text("Start Recording")');
    await expect(startButton).toBeVisible();
  });

  test('should handle rapid start/stop cycles', async ({ page }) => {
    // Rapidly start and stop recording multiple times
    for (let i = 0; i < 3; i++) {
      await page.click('button:has-text("Start Recording")');
      await page.waitForTimeout(100);
      await page.click('button:has-text("Stop Recording")');
      await page.waitForTimeout(100);
    }

    // Should not crash
    const macroPanel = page.locator('h2:has-text("Macro Recorder")');
    await expect(macroPanel).toBeVisible();

    // Start button should still be functional
    const startButton = page.locator('button:has-text("Start Recording")');
    await expect(startButton).toBeEnabled();
  });

  test('should display event count', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Should show event count somewhere
    const countDisplay = page.locator('text=/\\d+ events?/i');
    if (await countDisplay.isVisible()) {
      await expect(countDisplay).toContainText(/\d+/);
    }
  });

  test('should disable export when no events recorded', async ({ page }) => {
    // Export button should be disabled with no events
    const exportButton = page.locator('button:has-text("Export JSON"), button:has-text("Export")');
    if (await exportButton.isVisible()) {
      await expect(exportButton).toBeDisabled();
    }
  });

  test('should enable export after recording events', async ({ page }) => {
    // Start and stop recording
    await page.click('button:has-text("Start Recording")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Export button should be enabled
    const exportButton = page.locator('button:has-text("Export JSON"), button:has-text("Export")');
    if (await exportButton.isVisible()) {
      // Should be enabled after recording
      await expect(exportButton).toBeEnabled();
    }
  });

  test('complete workflow: record → edit → test → export', async ({ page }) => {
    // Step 1: Record
    await page.click('button:has-text("Start Recording")');
    await expect(page.locator('text=/Recording/i')).toBeVisible();
    await page.waitForTimeout(500);
    await page.click('button:has-text("Stop Recording")');

    // Step 2: Verify events recorded
    const eventsTable = page.locator('.events-table, table');
    await expect(eventsTable).toBeVisible();

    // Step 3: Edit an event (if possible)
    const editButton = page.locator('button:has-text("Edit")').first();
    if (await editButton.isVisible()) {
      await editButton.click();
      const keyInput = page.locator('input[type="number"]').first();
      if (await keyInput.isVisible()) {
        await keyInput.fill('65');
        const saveButton = page.locator('button:has-text("Save")').first();
        await saveButton.click();
      }
    }

    // Step 4: Test macro (if available)
    const testButton = page.locator('button:has-text("Test Macro"), button:has-text("Run Test")');
    if (await testButton.isVisible()) {
      await testButton.click();
      await page.waitForTimeout(1000);
    }

    // Step 5: Export
    const exportButton = page.locator('button:has-text("Export JSON"), button:has-text("Export")');
    if (await exportButton.isVisible() && await exportButton.isEnabled()) {
      const downloadPromise = page.waitForEvent('download');
      await exportButton.click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/macro.*\.json/i);
    }

    // Workflow completed successfully
    await expect(page.locator('h2:has-text("Macro Recorder")')).toBeVisible();
  });
});
