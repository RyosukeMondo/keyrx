import { expect, afterEach, beforeAll, beforeEach, afterAll, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import * as axeMatchers from 'vitest-axe/matchers';
import { server } from './mocks/server';
import { resetMockData } from './mocks/handlers';
import { setupMockWebSocket, cleanupMockWebSocket } from '../../tests/helpers/websocket';
import { getQuarantinedTestPatterns } from '../../tests/quarantine-manager';

// Extend Vitest's expect with jest-dom matchers
expect.extend(matchers);

// Extend Vitest's expect with axe matchers
expect.extend(axeMatchers);

// Mock window.matchMedia for animation tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {}, // deprecated
    removeListener: () => {}, // deprecated
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => true,
  }),
});

// Mock scrollIntoView for jsdom
Element.prototype.scrollIntoView = function() {
  // No-op implementation for tests
};

// Mock ResizeObserver for recharts/responsive components
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

/**
 * MSW server lifecycle hooks
 *
 * HYBRID MOCKING APPROACH:
 * - MSW handles HTTP REST API mocking (this server)
 * - jest-websocket-mock handles WebSocket mocking (tests/helpers/websocket.ts)
 *
 * This approach uses the best tool for each job:
 * - MSW: Proven excellent for HTTP mocking
 * - jest-websocket-mock: Better react-use-websocket compatibility
 *
 * WebSocket setup is now automatic for all tests (as of task 1.1).
 * Tests can still call setupMockWebSocket() manually if needed for custom configuration.
 */
beforeAll(() =>
  server.listen({
    onUnhandledRequest(request) {
      // Skip WebSocket requests - handled by jest-websocket-mock
      if (request.url.startsWith('ws:') || request.url.startsWith('wss:')) {
        return;
      }

      // Log warning for unhandled HTTP requests
      console.error(`[MSW] Unhandled ${request.method} request to ${request.url}`);
      throw new Error(
        `Unhandled ${request.method} request to ${request.url}. ` +
          `Add a handler for this endpoint in src/test/mocks/handlers.ts`
      );
    },
  })
);

/**
 * Skip quarantined tests in normal mode (unless RUN_QUARANTINE=true)
 *
 * This checks if the current test is in the quarantine list and skips it.
 * Quarantined tests can be run separately with: npm run test:quarantine
 */
beforeEach(async (context) => {
  // Only skip quarantined tests if not in quarantine mode
  if (process.env.RUN_QUARANTINE !== 'true') {
    const quarantinedTests = getQuarantinedTestPatterns();

    // Build full test path from context
    // Vitest provides: context.task.name, context.task.suite?.name, context.task.file?.name
    if (context.task && context.task.file) {
      // Extract relative file path from full path
      const fullPath = context.task.file.name;
      const relativePath = fullPath.replace(process.cwd() + '/', '');

      // Build full test path: "file > suite > test"
      const suites: string[] = [];
      let current = context.task.suite;
      while (current && current.name) {
        suites.unshift(current.name);
        current = current.suite;
      }

      const fullTestPath = [relativePath, ...suites, context.task.name]
        .filter(Boolean)
        .join(' > ');

      // Check if this test is quarantined
      if (quarantinedTests.some(pattern => fullTestPath.includes(pattern) || pattern.includes(fullTestPath))) {
        // Skip with informative message
        console.log(`⏭️  Skipping quarantined test: ${fullTestPath}`);
        context.skip();
        return;
      }
    }
  }

  // Setup WebSocket mock for all tests
  await setupMockWebSocket();
});

/**
 * Reset handlers between tests to prevent test pollution
 */
afterEach(() => {
  server.resetHandlers();
});

/**
 * Close MSW server after all tests complete
 */
afterAll(() => {
  server.close();
});

/**
 * Cleanup after each test
 *
 * This ensures test isolation by:
 * 1. Cleaning up React components
 * 2. Resetting HTTP mock data
 * 3. Cleaning up WebSocket mock (automatic as of task 1.1)
 */
afterEach(() => {
  cleanup();
  // Reset HTTP mock data to prevent test pollution
  resetMockData();
  // Clean up WebSocket mock
  cleanupMockWebSocket();
});
