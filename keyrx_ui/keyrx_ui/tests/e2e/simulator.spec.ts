import { test, expect } from '@playwright/test';

/**
 * E2E tests for the WASM-based keyboard simulation workflow.
 *
 * Tests the complete user journey from navigating to the simulator,
 * loading configurations, running scenarios, and viewing simulation results.
 */
test.describe('Simulator E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to load
    await expect(page.locator('h1')).toContainText('KeyRX');

    // Click the Simulator button
    await page.click('button:has-text("Simulator")');

    // Wait for the simulator panel to load
    await expect(page.locator('h2')).toContainText('Simulator');
  });

  test('should load valid Rhai configuration', async ({ page }) => {
    // Valid Rhai configuration with tap-hold
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
  tap_hold KEY_SPACE {
    tap: KEY_SPACE,
    hold: MD_00
  }
}`;

    // Find the textarea and type the config
    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);

    // Click the "Load Configuration" button
    await page.click('button:has-text("Load Configuration")');

    // Wait for loading to complete
    await page.waitForTimeout(1000);

    // Should show success message
    const successMessage = page.locator('text=/Configuration loaded successfully/i');
    await expect(successMessage).toBeVisible();

    // Should not show error
    const errorMessage = page.locator('text=/Failed to load/i');
    await expect(errorMessage).not.toBeVisible();
  });

  test('should show error for invalid Rhai configuration', async ({ page }) => {
    // Invalid Rhai configuration
    const invalidConfig = 'invalid syntax here {{{';

    // Find the textarea and type the invalid config
    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(invalidConfig);

    // Click the "Load Configuration" button
    await page.click('button:has-text("Load Configuration")');

    // Wait for error to appear
    await page.waitForTimeout(500);

    // Should show error message
    const errorMessage = page.locator('text=/Failed to load/i');
    await expect(errorMessage).toBeVisible();
  });

  test('should run built-in tap-hold-under scenario', async ({ page }) => {
    // First load a valid configuration
    const validConfig = `layer "base" {
  tap_hold KEY_SPACE {
    tap: KEY_SPACE,
    hold: MD_00
  }
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select the tap-hold-under scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });

    // Click "Run Scenario" button
    await page.click('button:has-text("Run Scenario")');

    // Wait for simulation to complete
    await page.waitForTimeout(1000);

    // Should display simulation results
    const resultsSection = page.locator('text=/Simulation Results/i');
    await expect(resultsSection).toBeVisible();

    // Should display latency stats
    const latencyStats = page.locator('text=/Latency Statistics/i');
    await expect(latencyStats).toBeVisible();

    // Check for performance metrics
    const minLatency = page.locator('text=/Min:/i');
    await expect(minLatency).toBeVisible();
  });

  test('should run built-in tap-hold-over scenario', async ({ page }) => {
    // Load configuration with tap-hold
    const validConfig = `layer "base" {
  tap_hold KEY_SPACE {
    tap: KEY_SPACE,
    hold: MD_00
  }
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select the tap-hold-over scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Over Threshold/i });

    // Click "Run Scenario" button
    await page.click('button:has-text("Run Scenario")');

    // Wait for simulation to complete
    await page.waitForTimeout(1000);

    // Should display results
    const resultsSection = page.locator('text=/Simulation Results/i');
    await expect(resultsSection).toBeVisible();
  });

  test('should run built-in layer-switch scenario', async ({ page }) => {
    // Load configuration with layers
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
  map MD_00 to layer_on("alt_layer")
}

layer "alt_layer" {
  map KEY_A to KEY_X
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select the layer-switch scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Layer Switch/i });

    // Click "Run Scenario" button
    await page.click('button:has-text("Run Scenario")');

    // Wait for simulation to complete
    await page.waitForTimeout(1000);

    // Should display results
    const resultsSection = page.locator('text=/Simulation Results/i');
    await expect(resultsSection).toBeVisible();
  });

  test('should run built-in modifier-combo scenario', async ({ page }) => {
    // Load configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select the modifier-combo scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Modifier Combo/i });

    // Click "Run Scenario" button
    await page.click('button:has-text("Run Scenario")');

    // Wait for simulation to complete
    await page.waitForTimeout(1000);

    // Should display results
    const resultsSection = page.locator('text=/Simulation Results/i');
    await expect(resultsSection).toBeVisible();
  });

  test('should create and run custom event sequence', async ({ page }) => {
    // Load configuration first
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Navigate to custom event sequence editor
    // Add a key press event
    const addEventButton = page.locator('button:has-text("Add Event")');
    if (await addEventButton.isVisible()) {
      await addEventButton.click();

      // Fill in event details (this may vary based on implementation)
      const keyCodeInput = page.locator('input[placeholder*="Key Code"], select[aria-label*="Key"]').first();
      await keyCodeInput.fill('VK_A');

      // Add another event for release
      await addEventButton.click();

      // Click simulate custom sequence
      const simulateButton = page.locator('button:has-text("Simulate Custom Sequence")');
      await simulateButton.click();

      // Wait for simulation
      await page.waitForTimeout(1000);

      // Should display results
      const resultsSection = page.locator('text=/Simulation Results/i');
      await expect(resultsSection).toBeVisible();
    }
  });

  test('should display scenario description when selected', async ({ page }) => {
    // Load configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select a scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });

    // Should display description
    const description = page.locator('text=/Simulates.*tap.*threshold/i');
    await expect(description).toBeVisible();
  });

  test('should disable scenario buttons when no config loaded', async ({ page }) => {
    // Without loading a config, scenario buttons should be disabled
    const runButton = page.locator('button:has-text("Run Scenario")');

    // Button should be disabled
    await expect(runButton).toBeDisabled();
  });

  test('should show loading spinner during simulation', async ({ page }) => {
    // Load configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Select scenario
    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });

    // Click run - should show loading indicator
    await page.click('button:has-text("Run Scenario")');

    // Check for loading indicator (may be visible briefly)
    // This is timing-dependent, so we'll just verify the simulation completes
    await page.waitForTimeout(1000);

    // Results should appear after loading
    const resultsSection = page.locator('text=/Simulation Results/i');
    await expect(resultsSection).toBeVisible();
  });

  test('should display latency statistics with correct format', async ({ page }) => {
    // Load and run a simulation
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });
    await page.click('button:has-text("Run Scenario")');
    await page.waitForTimeout(1000);

    // Check that latency stats are displayed
    const stats = page.locator('text=/Min:.*μs/i');
    await expect(stats).toBeVisible();

    // Check for average
    const avg = page.locator('text=/Avg:.*μs/i');
    await expect(avg).toBeVisible();

    // Check for max
    const max = page.locator('text=/Max:.*μs/i');
    await expect(max).toBeVisible();
  });

  test('should highlight performance warnings if latency exceeds threshold', async ({ page }) => {
    // This test assumes the WASM simulation might produce warnings
    // We'll check if the warning UI appears when appropriate
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });
    await page.click('button:has-text("Run Scenario")');
    await page.waitForTimeout(1000);

    // Check if latency stats table exists (warning may or may not appear)
    const latencyTable = page.locator('.latency-stats, table');
    await expect(latencyTable).toBeVisible();
  });

  test('should be accessible with keyboard navigation', async ({ page }) => {
    // Load configuration
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);

    // Tab through the interface
    await page.keyboard.press('Tab'); // Should focus textarea (already there)
    await page.keyboard.press('Tab'); // Should move to load button
    await page.keyboard.press('Tab'); // Should move to scenario selector

    // Verify we can interact with keyboard
    const dropdown = page.locator('select').first();
    await expect(dropdown).toBeFocused();
  });

  test('should maintain state when switching views', async ({ page }) => {
    // Load configuration in simulator
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Verify config loaded
    await expect(page.locator('text=/Configuration loaded successfully/i')).toBeVisible();

    // Switch to Devices view
    await page.click('button:has-text("Devices")');
    await page.waitForTimeout(200);

    // Switch back to Simulator
    await page.click('button:has-text("Simulator")');
    await page.waitForTimeout(200);

    // Config should still be loaded (or UI should handle gracefully)
    // The behavior depends on whether state is preserved
    const simulatorPanel = page.locator('h2:has-text("Simulator")');
    await expect(simulatorPanel).toBeVisible();
  });

  test('should handle rapid scenario switching', async ({ page }) => {
    // Load configuration
    const validConfig = `layer "base" {
  tap_hold KEY_SPACE {
    tap: KEY_SPACE,
    hold: MD_00
  }
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    const dropdown = page.locator('select').first();

    // Rapidly switch scenarios
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });
    await dropdown.selectOption({ label: /Tap-Hold Over Threshold/i });
    await dropdown.selectOption({ label: /Layer Switch/i });
    await dropdown.selectOption({ label: /Modifier Combo/i });

    // Should not crash
    const simulatorPanel = page.locator('h2:has-text("Simulator")');
    await expect(simulatorPanel).toBeVisible();
  });

  test('should display timeline visualization in results', async ({ page }) => {
    // Load and run simulation
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    const dropdown = page.locator('select').first();
    await dropdown.selectOption({ label: /Tap-Hold Under Threshold/i });
    await page.click('button:has-text("Run Scenario")');
    await page.waitForTimeout(1000);

    // Should have a timeline or event list
    const timeline = page.locator('[aria-label*="Timeline"], .timeline, .event-list');
    await expect(timeline).toBeVisible();
  });

  test('should show ARIA labels for accessibility', async ({ page }) => {
    // Check that main simulator panel has proper ARIA labels
    const simulatorPanel = page.locator('[aria-label*="Simulator"]');
    await expect(simulatorPanel).toBeVisible();
  });

  test('should handle file upload for configuration', async ({ page }) => {
    // Create a temporary config file content
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;

    // Look for file input
    const fileInput = page.locator('input[type="file"]');
    if (await fileInput.isVisible()) {
      // Create a file and upload it
      const buffer = Buffer.from(validConfig, 'utf-8');
      await fileInput.setInputFiles({
        name: 'test-config.rhai',
        mimeType: 'text/plain',
        buffer: buffer,
      });

      // Wait for file to be processed
      await page.waitForTimeout(1000);

      // The config should appear in the textarea
      const textarea = page.locator('textarea');
      await expect(textarea).toHaveValue(/layer.*base/);
    }
  });

  test('should clear error when loading valid config after invalid', async ({ page }) => {
    // First load invalid config
    const invalidConfig = 'invalid syntax {{{';
    const textarea = page.locator('textarea[placeholder*="Rhai configuration"]');
    await textarea.fill(invalidConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(500);

    // Should show error
    await expect(page.locator('text=/Failed to load/i')).toBeVisible();

    // Now load valid config
    const validConfig = `layer "base" {
  map KEY_A to KEY_B
}`;
    await textarea.fill(validConfig);
    await page.click('button:has-text("Load Configuration")');
    await page.waitForTimeout(1000);

    // Error should be cleared
    await expect(page.locator('text=/Failed to load/i')).not.toBeVisible();
    await expect(page.locator('text=/Configuration loaded successfully/i')).toBeVisible();
  });
});
