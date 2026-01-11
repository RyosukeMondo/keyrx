/**
 * Network Monitor Fixture for Playwright E2E Tests
 *
 * Provides network request tracking and analysis to detect performance issues
 * such as duplicate requests, rapid request patterns, and excessive API calls.
 *
 * This fixture is critical for catching bugs like:
 * - Rapid PATCH requests when dropdown values change
 * - Duplicate API calls on page load
 * - Excessive polling or inefficient data fetching
 *
 * Usage:
 * ```typescript
 * import { test } from '@playwright/test';
 * import { NetworkMonitor } from './fixtures/network-monitor';
 *
 * test('no duplicate requests', async ({ page }) => {
 *   const monitor = new NetworkMonitor(page);
 *   monitor.start();
 *
 *   await page.goto('/devices');
 *   await page.waitForLoadState('networkidle');
 *
 *   // Assert no duplicate requests within 100ms
 *   monitor.assertNoDuplicateRequests();
 *
 *   // Assert no more than 2 requests to /api/devices
 *   monitor.assertNoExcessiveRequests('/api/devices', 2);
 * });
 * ```
 */

import { Page, Request } from '@playwright/test';
import { expect } from '@playwright/test';

/**
 * Tracked request information
 */
interface TrackedRequest {
  /** Request URL */
  url: string;
  /** Request method (GET, POST, etc.) */
  method: string;
  /** Timestamp in milliseconds */
  timestamp: number;
  /** Response status code (if completed) */
  status?: number;
}

/**
 * Network Monitor - tracks and analyzes API requests during tests
 */
export class NetworkMonitor {
  private page: Page;
  private requests: TrackedRequest[] = [];
  private requestsByEndpoint: Map<string, TrackedRequest[]> = new Map();
  private isMonitoring = false;

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Start monitoring network requests
   *
   * Attaches listeners to page to track all API requests.
   * Only tracks requests to /api/* endpoints to avoid noise from static assets.
   */
  start(): void {
    if (this.isMonitoring) {
      console.warn('NetworkMonitor already started');
      return;
    }

    this.isMonitoring = true;
    this.requests = [];
    this.requestsByEndpoint.clear();

    // Listen for all requests
    this.page.on('request', (request: Request) => {
      const url = request.url();

      // Only track API requests
      if (!url.includes('/api/')) {
        return;
      }

      const tracked: TrackedRequest = {
        url,
        method: request.method(),
        timestamp: Date.now(),
      };

      this.requests.push(tracked);

      // Group by endpoint (path without query params)
      const endpoint = this.extractEndpoint(url);
      if (!this.requestsByEndpoint.has(endpoint)) {
        this.requestsByEndpoint.set(endpoint, []);
      }
      this.requestsByEndpoint.get(endpoint)!.push(tracked);
    });

    // Listen for responses to capture status codes
    this.page.on('response', (response) => {
      const url = response.url();

      if (!url.includes('/api/')) {
        return;
      }

      // Find corresponding request and update status
      const request = this.requests
        .slice()
        .reverse()
        .find((r) => r.url === url && !r.status);

      if (request) {
        request.status = response.status();
      }
    });

    console.log('✓ NetworkMonitor started');
  }

  /**
   * Stop monitoring network requests
   */
  stop(): void {
    if (!this.isMonitoring) {
      return;
    }

    this.page.removeAllListeners('request');
    this.page.removeAllListeners('response');
    this.isMonitoring = false;

    console.log(`✓ NetworkMonitor stopped (tracked ${this.requests.length} requests)`);
  }

  /**
   * Reset tracked requests without stopping monitoring
   */
  reset(): void {
    this.requests = [];
    this.requestsByEndpoint.clear();
    console.log('✓ NetworkMonitor reset');
  }

  /**
   * Get all tracked requests
   */
  getRequests(): TrackedRequest[] {
    return [...this.requests];
  }

  /**
   * Get requests for a specific endpoint
   *
   * @param endpoint - Endpoint path (e.g., '/api/devices' or '/api/profiles/:name')
   */
  getRequestsForEndpoint(endpoint: string): TrackedRequest[] {
    // Normalize endpoint (remove query params, trailing slashes)
    const normalized = this.normalizeEndpoint(endpoint);

    // Check exact match first
    if (this.requestsByEndpoint.has(normalized)) {
      return [...this.requestsByEndpoint.get(normalized)!];
    }

    // Check pattern match (e.g., /api/devices/:id)
    const matching: TrackedRequest[] = [];
    for (const [key, requests] of this.requestsByEndpoint.entries()) {
      if (this.endpointMatches(key, normalized)) {
        matching.push(...requests);
      }
    }

    return matching;
  }

  /**
   * Get request count for an endpoint
   */
  getRequestCount(endpoint: string): number {
    return this.getRequestsForEndpoint(endpoint).length;
  }

  /**
   * Get all endpoints that received requests
   */
  getEndpoints(): string[] {
    return Array.from(this.requestsByEndpoint.keys()).sort();
  }

  /**
   * Get summary of requests by endpoint
   */
  getSummary(): Record<string, number> {
    const summary: Record<string, number> = {};
    for (const [endpoint, requests] of this.requestsByEndpoint.entries()) {
      summary[endpoint] = requests.length;
    }
    return summary;
  }

  /**
   * Assert that no duplicate requests occurred within the specified time window
   *
   * This catches rapid-fire request bugs where the same endpoint is called
   * multiple times in quick succession (e.g., dropdown onChange firing multiple times).
   *
   * @param windowMs - Time window in milliseconds to check for duplicates (default: 100ms)
   * @param endpoint - Optional specific endpoint to check (checks all if not specified)
   */
  assertNoDuplicateRequests(windowMs = 100, endpoint?: string): void {
    const requestsToCheck = endpoint
      ? this.getRequestsForEndpoint(endpoint)
      : this.requests;

    const duplicates: Array<{ endpoint: string; count: number; timespan: number }> = [];

    // Group by endpoint and method
    const grouped = new Map<string, TrackedRequest[]>();
    for (const request of requestsToCheck) {
      const key = `${request.method} ${this.extractEndpoint(request.url)}`;
      if (!grouped.has(key)) {
        grouped.set(key, []);
      }
      grouped.get(key)!.push(request);
    }

    // Check each group for duplicates within time window
    for (const [key, requests] of grouped.entries()) {
      if (requests.length < 2) continue;

      // Sort by timestamp
      const sorted = requests.sort((a, b) => a.timestamp - b.timestamp);

      // Check for requests within time window
      for (let i = 0; i < sorted.length - 1; i++) {
        const current = sorted[i];
        const next = sorted[i + 1];
        const timespan = next.timestamp - current.timestamp;

        if (timespan <= windowMs) {
          duplicates.push({
            endpoint: key,
            count: 2, // Could extend to count all in window
            timespan,
          });
        }
      }
    }

    if (duplicates.length > 0) {
      const details = duplicates
        .map(
          (d) =>
            `  - ${d.endpoint}: ${d.count} requests within ${d.timespan}ms (threshold: ${windowMs}ms)`
        )
        .join('\n');

      expect(
        duplicates.length,
        `Found ${duplicates.length} duplicate request pattern(s):\n${details}\n\nThis indicates a bug where requests are fired multiple times in rapid succession.`
      ).toBe(0);
    }
  }

  /**
   * Assert that no more than maxCount requests were made to an endpoint
   *
   * This catches inefficient code that makes excessive API calls,
   * such as calling an endpoint on every render instead of once on mount.
   *
   * @param endpoint - Endpoint to check (e.g., '/api/devices')
   * @param maxCount - Maximum allowed requests
   */
  assertNoExcessiveRequests(endpoint: string, maxCount: number): void {
    const requests = this.getRequestsForEndpoint(endpoint);
    const count = requests.length;

    if (count > maxCount) {
      const timestamps = requests
        .map((r) => `    ${r.method} ${this.extractEndpoint(r.url)} at ${r.timestamp}`)
        .join('\n');

      expect(
        count,
        `Excessive requests to ${endpoint}:\n` +
          `  Expected: ≤ ${maxCount} requests\n` +
          `  Actual: ${count} requests\n\n` +
          `Requests:\n${timestamps}\n\n` +
          `This indicates inefficient code making too many API calls.`
      ).toBeLessThanOrEqual(maxCount);
    }
  }

  /**
   * Assert total number of API requests is within expected range
   *
   * @param maxTotal - Maximum total API requests allowed
   */
  assertTotalRequests(maxTotal: number): void {
    const total = this.requests.length;

    if (total > maxTotal) {
      const summary = this.getSummary();
      const details = Object.entries(summary)
        .map(([endpoint, count]) => `  - ${endpoint}: ${count} requests`)
        .join('\n');

      expect(
        total,
        `Too many total API requests:\n` +
          `  Expected: ≤ ${maxTotal} requests\n` +
          `  Actual: ${total} requests\n\n` +
          `Breakdown:\n${details}`
      ).toBeLessThanOrEqual(maxTotal);
    }
  }

  /**
   * Print summary of all requests for debugging
   */
  printSummary(): void {
    console.log('\n=== Network Monitor Summary ===');
    console.log(`Total requests: ${this.requests.length}`);
    console.log('\nRequests by endpoint:');

    const summary = this.getSummary();
    for (const [endpoint, count] of Object.entries(summary).sort()) {
      console.log(`  ${endpoint}: ${count}`);
    }

    console.log('================================\n');
  }

  /**
   * Extract endpoint path from full URL
   * Example: http://localhost:9867/api/devices?foo=bar -> /api/devices
   */
  private extractEndpoint(url: string): string {
    try {
      const parsed = new URL(url);
      // Remove query params
      return parsed.pathname;
    } catch {
      // If URL parsing fails, try simple extraction
      const match = url.match(/\/api\/[^?]*/);
      return match ? match[0] : url;
    }
  }

  /**
   * Normalize endpoint (remove trailing slashes, query params)
   */
  private normalizeEndpoint(endpoint: string): string {
    return endpoint.replace(/\/$/, '').split('?')[0];
  }

  /**
   * Check if an endpoint matches a pattern
   * Supports simple :param style patterns
   */
  private endpointMatches(actual: string, pattern: string): boolean {
    // Exact match
    if (actual === pattern) {
      return true;
    }

    // Pattern match (e.g., /api/devices/123 matches /api/devices/:id)
    const actualParts = actual.split('/');
    const patternParts = pattern.split('/');

    if (actualParts.length !== patternParts.length) {
      return false;
    }

    for (let i = 0; i < actualParts.length; i++) {
      const patternPart = patternParts[i];
      const actualPart = actualParts[i];

      // :param matches any value
      if (patternPart.startsWith(':')) {
        continue;
      }

      // Must match exactly
      if (patternPart !== actualPart) {
        return false;
      }
    }

    return true;
  }
}

/**
 * Convenience function to create and start a network monitor
 */
export function createNetworkMonitor(page: Page): NetworkMonitor {
  const monitor = new NetworkMonitor(page);
  monitor.start();
  return monitor;
}
