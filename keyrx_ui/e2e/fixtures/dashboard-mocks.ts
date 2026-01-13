/**
 * E2E Test Dashboard Mocks
 *
 * Provides mock responses for dashboard-related E2E tests.
 */

import type { Page, Route } from '@playwright/test';

export interface MockDaemonState {
  modifiers: number[];
  locks: number[];
  layer: number;
}

export interface MockKeyEvent {
  keyCode: number;
  eventType: 'press' | 'release';
  input: number;
  output: number;
  timestamp: number;
  latency: number;
}

export interface MockLatencyMetrics {
  avg: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
}

export const defaultDashboardData = {
  daemonState: {
    modifiers: [],
    locks: [],
    layer: 0,
  } as MockDaemonState,

  latencyMetrics: {
    avg: 1500,
    p95: 2500,
    p99: 3500,
    min: 500,
    max: 5000,
  } as MockLatencyMetrics,
};

/**
 * Setup dashboard-specific API mocks
 */
export async function setupDashboardMocks(
  page: Page,
  options: {
    daemonState?: MockDaemonState;
    latencyMetrics?: MockLatencyMetrics;
    connected?: boolean;
  } = {}
): Promise<void> {
  const daemonState = options.daemonState ?? defaultDashboardData.daemonState;
  const latencyMetrics = options.latencyMetrics ?? defaultDashboardData.latencyMetrics;
  const connected = options.connected ?? true;

  // GET /api/daemon/state
  await page.route('**/api/daemon/state', async (route: Route) => {
    if (!connected) {
      await route.abort('connectionrefused');
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(daemonState),
    });
  });

  // GET /api/metrics/latency
  await page.route('**/api/metrics/latency', async (route: Route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(latencyMetrics),
    });
  });

  // GET /api/status
  await page.route('**/api/status', async (route: Route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        status: connected ? 'connected' : 'disconnected',
        version: '0.1.0',
        daemon_running: connected,
        uptime_secs: connected ? 3600 : null,
        active_profile: 'default',
        device_count: 2,
      }),
    });
  });

  // Abort WebSocket connections (they don't work in E2E without real backend)
  await page.route('**/ws/**', async (route: Route) => {
    await route.abort('connectionrefused');
  });
}

/**
 * Generate mock key events
 */
export function generateMockKeyEvents(count: number): MockKeyEvent[] {
  const events: MockKeyEvent[] = [];
  const now = Date.now() * 1000; // microseconds

  for (let i = 0; i < count; i++) {
    events.push({
      keyCode: 65 + (i % 26), // A-Z
      eventType: i % 2 === 0 ? 'press' : 'release',
      input: 65 + (i % 26),
      output: 65 + (i % 26),
      timestamp: now - i * 10000,
      latency: 500 + Math.random() * 2000,
    });
  }

  return events;
}
