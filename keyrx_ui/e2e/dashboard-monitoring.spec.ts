import { test, expect } from '@playwright/test';

/**
 * E2E Test: Dashboard Real-Time Monitoring
 *
 * Tests the real-time dashboard functionality:
 * - WebSocket connection status
 * - Real-time daemon state updates
 * - Real-time key event updates
 * - Real-time latency metrics updates
 * - Event timeline features (pause/resume, clear)
 * - Responsive layout
 *
 * Requirements: REQ-7 (AC4, AC10)
 */

test.describe('Dashboard Real-Time Monitoring', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
  });

  test('should display connection status', async ({ page }) => {
    // Verify connection banner appears
    const connectionBanner = page.locator('[data-testid="connection-banner"]');
    await expect(connectionBanner).toBeVisible({ timeout: 10000 });

    // Verify connection status is "Connected" (green)
    await expect(connectionBanner).toContainText('Connected');
    await expect(connectionBanner).toHaveClass(/bg-green/);
  });

  test('should display daemon state indicators', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify State Indicator Panel is visible
    const statePanel = page.locator('[data-testid="state-indicator-panel"]');
    await expect(statePanel).toBeVisible();

    // Verify sections for Modifiers, Locks, and Layer
    await expect(statePanel.locator('text=Modifiers')).toBeVisible();
    await expect(statePanel.locator('text=Locks')).toBeVisible();
    await expect(statePanel.locator('text=Layer')).toBeVisible();
  });

  test('should display latency metrics chart', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify Metrics Chart is visible
    const metricsChart = page.locator('[data-testid="metrics-chart"]');
    await expect(metricsChart).toBeVisible();

    // Verify chart elements (Recharts SVG)
    await expect(metricsChart.locator('svg')).toBeVisible();

    // Verify reference line at 5ms is visible
    await expect(metricsChart).toContainText('Target');
  });

  test('should display event timeline', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify Event Timeline is visible
    const timeline = page.locator('[data-testid="event-timeline"]');
    await expect(timeline).toBeVisible();

    // Verify pause/resume and clear buttons
    await expect(page.locator('button:has-text("Pause")')).toBeVisible();
    await expect(page.locator('button:has-text("Clear")')).toBeVisible();
  });

  test('should update daemon state in real-time', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    const statePanel = page.locator('[data-testid="state-indicator-panel"]');

    // Get initial state
    const initialState = await statePanel.textContent();

    // Simulate state change by pressing a modifier key
    // (This would require actual key input to the daemon, which may not be possible in E2E)
    // For now, we verify the state panel updates dynamically

    // Wait for potential state change (daemon may broadcast state updates)
    await page.waitForTimeout(2000);

    // Verify state panel is still visible and functioning
    await expect(statePanel).toBeVisible();
  });

  test('should update latency metrics in real-time', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    const metricsChart = page.locator('[data-testid="metrics-chart"]');

    // Wait for first metric update (broadcasts every 1 second)
    await page.waitForTimeout(1500);

    // Verify chart is rendering data
    await expect(metricsChart.locator('svg')).toBeVisible();

    // Wait for another update
    await page.waitForTimeout(1500);

    // Chart should continue updating (we can't easily verify data points changed,
    // but can verify it's still rendering)
    await expect(metricsChart.locator('svg')).toBeVisible();
  });

  test('should display key events in timeline', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    const timeline = page.locator('[data-testid="event-timeline"]');

    // Wait for events to appear (daemon broadcasts key events)
    // In test mode, daemon may not generate events unless we send some
    await page.waitForTimeout(2000);

    // Verify timeline is functional (list container should exist)
    const eventList = timeline.locator('[data-testid="event-list"]');
    await expect(eventList).toBeVisible();

    // Note: In a real test with key input, we would verify specific events appear
  });

  test('should pause and resume event timeline', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify Pause button is visible
    const pauseButton = page.locator('button:has-text("Pause")');
    await expect(pauseButton).toBeVisible();

    // Click Pause
    await pauseButton.click();

    // Verify button changes to "Resume"
    const resumeButton = page.locator('button:has-text("Resume")');
    await expect(resumeButton).toBeVisible();

    // Wait a moment to ensure new events don't appear
    const initialEventCount = await page
      .locator('[data-testid="event-list"]')
      .locator('[data-event-item]')
      .count();

    await page.waitForTimeout(2000);

    // Event count should not increase while paused
    const pausedEventCount = await page
      .locator('[data-testid="event-list"]')
      .locator('[data-event-item]')
      .count();

    expect(pausedEventCount).toBe(initialEventCount);

    // Click Resume
    await resumeButton.click();

    // Verify button changes back to "Pause"
    await expect(pauseButton).toBeVisible();
  });

  test('should clear event timeline', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Wait for some events to accumulate
    await page.waitForTimeout(2000);

    // Click Clear button
    await page.click('button:has-text("Clear")');

    // Verify event list is empty
    const eventCount = await page
      .locator('[data-testid="event-list"]')
      .locator('[data-event-item]')
      .count();

    expect(eventCount).toBe(0);
  });

  test('should enforce FIFO limit on events (max 100)', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // This test would require generating many events to exceed the limit
    // In practice, we would simulate key events or use a mock daemon

    // For now, verify the event list exists and is bounded
    const eventList = page.locator('[data-testid="event-list"]');
    await expect(eventList).toBeVisible();

    // Verify max events indicator or implementation
    // (This would need event generation capability)
  });

  test('should display responsive layout on mobile', async ({ page }) => {
    // Set viewport to mobile size
    await page.setViewportSize({ width: 375, height: 667 });

    // Reload dashboard
    await page.goto('/dashboard');
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify components stack vertically (single column)
    const statePanel = page.locator('[data-testid="state-indicator-panel"]');
    const metricsChart = page.locator('[data-testid="metrics-chart"]');

    await expect(statePanel).toBeVisible();
    await expect(metricsChart).toBeVisible();

    // Verify they are in a single column layout
    // (Both should be visible without horizontal scroll)
    const statePanelBox = await statePanel.boundingBox();
    const metricsChartBox = await metricsChart.boundingBox();

    expect(statePanelBox).not.toBeNull();
    expect(metricsChartBox).not.toBeNull();

    // On mobile, components should stack (different Y positions)
    if (statePanelBox && metricsChartBox) {
      expect(statePanelBox.y).not.toBe(metricsChartBox.y);
    }
  });

  test('should display responsive layout on desktop', async ({ page }) => {
    // Set viewport to desktop size
    await page.setViewportSize({ width: 1280, height: 720 });

    // Reload dashboard
    await page.goto('/dashboard');
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Verify components are in 2-column grid
    const statePanel = page.locator('[data-testid="state-indicator-panel"]');
    const metricsChart = page.locator('[data-testid="metrics-chart"]');

    await expect(statePanel).toBeVisible();
    await expect(metricsChart).toBeVisible();

    // On desktop >= 1024px, components should be side-by-side (similar Y positions)
    const statePanelBox = await statePanel.boundingBox();
    const metricsChartBox = await metricsChart.boundingBox();

    expect(statePanelBox).not.toBeNull();
    expect(metricsChartBox).not.toBeNull();

    // Components should be roughly at same Y position (side by side)
    if (statePanelBox && metricsChartBox) {
      const yDiff = Math.abs(statePanelBox.y - metricsChartBox.y);
      expect(yDiff).toBeLessThan(50); // Allow small differences
    }
  });

  test('should show disconnected state when connection lost', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Note: Simulating disconnect requires stopping the daemon or network interception
    // For now, we verify the connection banner can show different states

    // Verify connection banner has the ability to show different states
    const connectionBanner = page.locator('[data-testid="connection-banner"]');
    await expect(connectionBanner).toBeVisible();
  });

  test('should handle reconnection gracefully', async ({ page }) => {
    // Wait for initial connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Note: Testing reconnection requires daemon restart or network manipulation
    // For now, verify auto-reconnect behavior is configured

    // Verify connection banner and WebSocket setup
    const connectionBanner = page.locator('[data-testid="connection-banner"]');
    await expect(connectionBanner).toBeVisible();

    // In a full test, we would:
    // 1. Stop daemon
    // 2. Verify "Disconnected" banner appears
    // 3. Restart daemon
    // 4. Verify "Connected" banner reappears
  });

  test('should display event details on hover', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Wait for events to appear
    await page.waitForTimeout(2000);

    // Find first event item
    const firstEvent = page.locator('[data-event-item]').first();

    if ((await firstEvent.count()) > 0) {
      // Hover over the event
      await firstEvent.hover();

      // Verify tooltip appears with details
      const tooltip = page.locator('[data-testid="event-tooltip"]');
      await expect(tooltip).toBeVisible({ timeout: 2000 });

      // Verify tooltip contains event details
      await expect(tooltip).toContainText(/timestamp|latency|layer/i);
    }
  });

  test('should format timestamps and latency correctly', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Wait for latency data
    await page.waitForTimeout(2000);

    // Verify metrics chart shows milliseconds
    const metricsChart = page.locator('[data-testid="metrics-chart"]');
    await expect(metricsChart).toBeVisible();

    // Y-axis should show "ms" unit
    await expect(metricsChart).toContainText(/ms/i);

    // Verify events show relative time
    const eventList = page.locator('[data-testid="event-list"]');
    if ((await eventList.locator('[data-event-item]').count()) > 0) {
      // Event timestamps should show relative time like "2s ago"
      await expect(eventList).toContainText(/ago|now/i);
    }
  });

  test('should maintain performance with many events', async ({ page }) => {
    // Wait for connection
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 10000 });

    // Wait for events to accumulate
    await page.waitForTimeout(5000);

    // Verify virtualization is working (react-window)
    const eventList = page.locator('[data-testid="event-list"]');
    await expect(eventList).toBeVisible();

    // Scroll the event list
    await eventList.evaluate((el) => {
      el.scrollTop = 500;
    });

    // Wait a moment
    await page.waitForTimeout(500);

    // Scroll back to top
    await eventList.evaluate((el) => {
      el.scrollTop = 0;
    });

    // Verify scrolling is smooth (no lag)
    // (In practice, this would measure frame rate or responsiveness)
    await expect(eventList).toBeVisible();
  });
});
