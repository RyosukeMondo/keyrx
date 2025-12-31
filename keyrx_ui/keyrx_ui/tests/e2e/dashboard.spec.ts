import { test, expect } from '@playwright/test';

/**
 * E2E tests for the real-time daemon dashboard workflow.
 *
 * Tests the complete user journey:
 * 1. Dashboard loads
 * 2. WebSocket connects to daemon
 * 3. Real-time events appear in UI (state, metrics, events)
 * 4. UI handles connection states (connecting, connected, disconnected)
 */
test.describe('Real-time Dashboard E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to load
    await expect(page.locator('h1')).toContainText('KeyRX');

    // Click the Dashboard button
    await page.click('button:has-text("Dashboard")');

    // Wait for the dashboard page to load
    await expect(page.locator('h2').first()).toContainText('Daemon State');
  });

  test('should display dashboard page with all panels', async ({ page }) => {
    // Verify all dashboard panels are present
    await expect(page.locator('h2:has-text("Daemon State")')).toBeVisible();
    await expect(page.locator('h2:has-text("Latency Metrics")')).toBeVisible();
    await expect(page.locator('h2:has-text("Event Timeline")')).toBeVisible();

    // Verify connection banner is present
    const connectionBanner = page.locator('.connection-banner');
    await expect(connectionBanner).toBeVisible();
  });

  test('should show connection status indicators', async ({ page }) => {
    // Connection banner should be visible
    const connectionBanner = page.locator('.connection-banner');
    await expect(connectionBanner).toBeVisible();

    // Should show one of the connection states
    const statusText = connectionBanner.locator('.connection-status span');
    await expect(statusText).toBeVisible();

    // Status indicator dot should be visible
    const statusIndicator = connectionBanner.locator('.status-indicator');
    await expect(statusIndicator).toBeVisible();
  });

  test('should display state indicator panel', async ({ page }) => {
    // State panel should be visible
    const statePanel = page.locator('.state-panel');
    await expect(statePanel).toBeVisible();

    // Should contain state indicator component
    const stateIndicator = page.locator('.state-indicator-panel, .state-badges');
    await expect(stateIndicator).toBeVisible({ timeout: 5000 });
  });

  test('should display metrics chart', async ({ page }) => {
    // Metrics panel should be visible
    const metricsPanel = page.locator('.metrics-panel');
    await expect(metricsPanel).toBeVisible();

    // Chart component should be present (recharts creates SVG)
    const chart = metricsPanel.locator('.recharts-wrapper, svg.recharts-surface');
    await expect(chart).toBeVisible({ timeout: 5000 });
  });

  test('should display event timeline', async ({ page }) => {
    // Events panel should be visible
    const eventsPanel = page.locator('.events-panel');
    await expect(eventsPanel).toBeVisible();

    // Timeline component should be present
    const timeline = eventsPanel.locator('.event-timeline, [role="list"]');
    await expect(timeline).toBeVisible({ timeout: 5000 });
  });

  test('should handle WebSocket connection simulation', async ({ page }) => {
    // Mock WebSocket messages by injecting events into the store
    await page.evaluate(() => {
      // Access the Zustand store and update it
      const storeEvent = {
        type: 'state' as const,
        data: {
          currentLayer: 0,
          activeModifiers: [1, 2],
          activeLocks: [0],
        },
        timestamp: Date.now(),
      };

      // Dispatch an event to the store
      // Note: This assumes the store is accessible globally or via window
      // In real scenarios, WebSocket would send this data
      window.dispatchEvent(new CustomEvent('test-dashboard-event', {
        detail: storeEvent
      }));
    });

    // Wait a moment for the event to be processed
    await page.waitForTimeout(500);

    // Dashboard should remain functional
    await expect(page.locator('.dashboard-page')).toBeVisible();
  });

  test('should display pause/resume controls in event timeline', async ({ page }) => {
    const eventsPanel = page.locator('.events-panel');

    // Look for pause/resume button
    const pauseButton = eventsPanel.locator('button:has-text("Pause"), button:has-text("Resume"), button[aria-label*="pause"], button[aria-label*="resume"]');

    // Timeline controls should be visible
    await expect(pauseButton).toBeVisible({ timeout: 5000 });
  });

  test('should show empty state when no events', async ({ page }) => {
    // Wait for dashboard to fully load
    await page.waitForTimeout(1000);

    const eventsPanel = page.locator('.events-panel');

    // Should either show events or an empty state message
    const hasEvents = await eventsPanel.locator('.event-item, .timeline-item').count() > 0;
    const hasEmptyState = await eventsPanel.locator(':has-text("No events"), :has-text("no events"), :has-text("empty")').isVisible().catch(() => false);

    // One of these should be true
    expect(hasEvents || hasEmptyState).toBeTruthy();
  });

  test('should maintain dashboard state during connection changes', async ({ page }) => {
    // Get initial state
    const initialText = await page.locator('.connection-status span').textContent();
    expect(initialText).toBeTruthy();

    // Wait a moment to let any connection state changes settle
    await page.waitForTimeout(2000);

    // Dashboard should remain visible and functional
    await expect(page.locator('.dashboard-page')).toBeVisible();
    await expect(page.locator('h2:has-text("Daemon State")')).toBeVisible();
    await expect(page.locator('h2:has-text("Latency Metrics")')).toBeVisible();
    await expect(page.locator('h2:has-text("Event Timeline")')).toBeVisible();
  });

  test('should be responsive and not crash under normal conditions', async ({ page }) => {
    // Wait for dashboard to stabilize
    await page.waitForTimeout(1000);

    // Verify no JavaScript errors
    const errors: string[] = [];
    page.on('pageerror', (error) => {
      errors.push(error.message);
    });

    // Interact with the page
    await page.mouse.move(100, 100);
    await page.mouse.move(200, 200);

    // Wait for any async operations
    await page.waitForTimeout(500);

    // Should have no errors
    expect(errors).toHaveLength(0);
  });

  test('should display dashboard metrics with proper labels', async ({ page }) => {
    const metricsPanel = page.locator('.metrics-panel');

    // Should have chart axes labels or indicators
    // Recharts typically renders text elements for labels
    const hasLabels = await metricsPanel.locator('text, .recharts-label').count() > 0;

    // Chart should have some visual representation
    expect(hasLabels).toBeTruthy();
  });

  test('should allow navigation back to other views', async ({ page }) => {
    // Navigate to a different view
    await page.click('button:has-text("Devices")');

    // Wait for view change
    await page.waitForTimeout(300);

    // Should see devices view (DeviceList renders specific content)
    await expect(page.locator('button:has-text("Devices")')).toHaveClass(/active/);

    // Navigate back to dashboard
    await page.click('button:has-text("Dashboard")');

    // Wait for view change
    await page.waitForTimeout(300);

    // Dashboard should be visible again
    await expect(page.locator('h2:has-text("Daemon State")')).toBeVisible();
    await expect(page.locator('button:has-text("Dashboard")')).toHaveClass(/active/);
  });

  test('should render without accessibility violations', async ({ page }) => {
    // Wait for dashboard to fully load
    await page.waitForTimeout(1000);

    // Check for basic ARIA attributes
    const dashboard = page.locator('.dashboard-page');
    await expect(dashboard).toBeVisible();

    // Connection status should have proper semantics
    const statusIndicator = page.locator('.status-indicator');
    await expect(statusIndicator).toBeVisible();

    // Panels should be properly structured
    const panels = page.locator('.panel');
    const panelCount = await panels.count();
    expect(panelCount).toBeGreaterThanOrEqual(2); // At least state and metrics panels
  });

  test('should display real-time updates indicator', async ({ page }) => {
    // Connection banner should show current status
    const connectionStatus = page.locator('.connection-status');
    await expect(connectionStatus).toBeVisible();

    const statusText = await connectionStatus.locator('span').textContent();

    // Should show one of the expected connection states
    expect(statusText).toMatch(/connecting|connected|disconnected|reconnect/i);
  });

  test('should handle dashboard layout properly', async ({ page }) => {
    // Dashboard grid should be visible
    const dashboardGrid = page.locator('.dashboard-grid');
    await expect(dashboardGrid).toBeVisible();

    // All three main sections should be rendered
    const statePanel = page.locator('.state-panel');
    const metricsPanel = page.locator('.metrics-panel');
    const eventsPanel = page.locator('.events-panel');

    await expect(statePanel).toBeVisible();
    await expect(metricsPanel).toBeVisible();
    await expect(eventsPanel).toBeVisible();
  });

  test('should load dashboard quickly', async ({ page }) => {
    const startTime = Date.now();

    // Navigate to dashboard (we're already there, but time the render)
    await page.click('button:has-text("Devices")');
    await page.waitForTimeout(100);
    await page.click('button:has-text("Dashboard")');

    // Wait for dashboard to be visible
    await expect(page.locator('.dashboard-page')).toBeVisible();

    const loadTime = Date.now() - startTime;

    // Dashboard should load in under 3 seconds
    expect(loadTime).toBeLessThan(3000);
  });

  test('should persist dashboard state across rapid navigation', async ({ page }) => {
    // Rapidly navigate between views
    await page.click('button:has-text("Simulator")');
    await page.waitForTimeout(100);

    await page.click('button:has-text("Dashboard")');
    await page.waitForTimeout(100);

    await page.click('button:has-text("Config Editor")');
    await page.waitForTimeout(100);

    await page.click('button:has-text("Dashboard")');

    // Dashboard should still render correctly
    await expect(page.locator('.dashboard-page')).toBeVisible();
    await expect(page.locator('h2:has-text("Daemon State")')).toBeVisible();
  });

  test('should show connection banner with appropriate styling', async ({ page }) => {
    const connectionBanner = page.locator('.connection-banner');
    await expect(connectionBanner).toBeVisible();

    // Banner should have a connection status class
    const bannerClass = await connectionBanner.getAttribute('class');
    expect(bannerClass).toMatch(/connection-(connected|connecting|disconnected)/);
  });
});
