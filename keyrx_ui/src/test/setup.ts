import { expect, afterEach, beforeAll, afterAll } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import * as axeMatchers from 'vitest-axe/matchers';
import { server } from './mocks/server';
import { resetMockData } from './mocks/handlers';
import { resetWebSocketState } from './mocks/websocketHandlers';

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
 * The MSW server handles both HTTP REST API mocking and WebSocket RPC mocking.
 * WebSocket connections to ws://localhost:3030/ws are automatically intercepted
 * and handled by the MSW WebSocket handlers configured in server.ts.
 *
 * No manual WebSocket setup is required in individual tests - all WebSocket
 * connections are automatically mocked by MSW.
 */
beforeAll(() =>
  server.listen({
    onUnhandledRequest(request) {
      // Log warning for unhandled HTTP requests (not WebSocket)
      if (!request.url.startsWith('ws:') && !request.url.startsWith('wss:')) {
        console.error(`[MSW] Unhandled ${request.method} request to ${request.url}`);
        throw new Error(
          `Unhandled ${request.method} request to ${request.url}. ` +
            `Add a handler for this endpoint in src/test/mocks/handlers.ts`
        );
      }
    },
  })
);

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
 * 3. Resetting WebSocket state (connections, subscriptions, daemon state)
 */
afterEach(() => {
  cleanup();
  // Reset HTTP mock data to prevent test pollution
  resetMockData();
  // Reset WebSocket state (connections, subscriptions, daemon state)
  resetWebSocketState();
});
