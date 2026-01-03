# Design Document

## Introduction

This document outlines the technical design for refactoring the KeyRx frontend test infrastructure to achieve >95% test pass rate, proper test categorization, and production-ready WebSocket mocking. The current 12% test failure rate (134/1109 tests failing) is primarily caused by inadequate WebSocket mocking and improper mixing of test types (E2E, performance, visual regression tests running with unit tests).

## Current Architecture Analysis

### Existing Test Infrastructure

**File Structure:**
```
keyrx_ui/
├── vite.config.ts              # Single config for all tests
├── src/test/
│   ├── setup.ts                # Global test setup
│   └── mocks/
│       ├── server.ts           # MSW server (HTTP only)
│       └── handlers.ts         # MSW HTTP handlers
├── tests/
│   ├── testUtils.tsx           # renderWithProviders + helpers
│   ├── helpers/
│   │   └── websocket.ts        # jest-websocket-mock helpers
│   ├── a11y/                   # Accessibility tests
│   ├── integration/            # Integration tests
│   └── factories/              # Test data factories
├── e2e/                        # Playwright E2E tests
└── tests/
    ├── performance/            # Lighthouse/bundle tests
    └── e2e/
        └── visual/             # Visual regression tests
```

**Technology Stack:**
- **Test Runner**: Vitest 1.0.4
- **HTTP Mocking**: MSW 2.12.7 (HTTP only)
- **WebSocket Mocking**: jest-websocket-mock 2.5.0
- **Component Testing**: React Testing Library 14.1.2
- **E2E Testing**: Playwright 1.40.1
- **Accessibility**: vitest-axe, @axe-core/playwright
- **Coverage**: @vitest/coverage-v8

### Current Issues

**1. WebSocket Mocking Problems**

Current implementation uses `jest-websocket-mock`:
```typescript
// tests/helpers/websocket.ts
import WS from 'jest-websocket-mock';

export async function setupMockWebSocket(url = WS_URL): Promise<WS> {
  mockServer = new WS(url, { jsonProtocol: true });
  return mockServer;
}
```

**Issues:**
- Separate mocking library from MSW (inconsistent API)
- Manual setup required in each test file
- Race conditions with `react-use-websocket` lifecycle
- Not integrated with MSW's request interception
- 10/17 WebSocket unit tests failing (src/api/websocket.test.ts)
- ~15 component integration tests failing (ConfigPage, DevicesPage, etc.)

**2. Test Category Mixing**

All tests run through single `vite.config.ts`:
```json
"scripts": {
  "test": "vitest run",              // Runs ALL tests (unit + E2E + perf + visual)
  "test:e2e": "playwright test e2e", // Playwright E2E
  "test:a11y": "vitest run tests/a11y" // Accessibility only
}
```

**Problems:**
- `npm test` tries to run Playwright tests without server running → fails
- Performance tests require Lighthouse → fails if not installed
- Visual tests require Percy/Chromatic → fails in CI
- Slow test feedback (unit tests wait for E2E tests to timeout)

**3. Test Isolation Issues**

Tests fail when run in parallel due to shared global state:
- WebSocket mock not properly reset between tests
- MSW handlers leak state between tests
- React Query cache not cleared properly

## Technology Choices

### Decision 1: WebSocket Mocking - MSW vs. jest-websocket-mock

**Option A: Migrate to MSW WebSocket Support** ⭐ **RECOMMENDED**

MSW v2+ supports WebSocket mocking via `ws.link()`:

```typescript
import { ws } from 'msw';
import { setupServer } from 'msw/node';

const handlers = [
  ws.link('ws://localhost:3030/ws')
    .addEventListener('connection', ({ client }) => {
      client.addEventListener('message', (event) => {
        if (event.data === 'ping') {
          client.send('pong');
        }
      });
    })
];

export const server = setupServer(...handlers);
```

**Pros:**
- ✅ Single mocking library (MSW for both HTTP and WebSocket)
- ✅ Consistent API across all network requests
- ✅ Better integration with test setup (global server)
- ✅ Type-safe WebSocket message handling
- ✅ Automatic cleanup via MSW lifecycle
- ✅ No race conditions with fake timers

**Cons:**
- ⚠️ Migration effort (update 15+ test files)
- ⚠️ MSW WebSocket support relatively new (v2+)
- ⚠️ Need to verify compatibility with react-use-websocket

**Option B: Keep jest-websocket-mock**

**Pros:**
- ✅ Already working for some tests
- ✅ Well-documented and mature
- ✅ No migration required

**Cons:**
- ❌ Separate library from MSW (two mocking paradigms)
- ❌ Manual setup/teardown in each test
- ❌ Race conditions with component lifecycle
- ❌ 10/17 tests currently failing
- ❌ Not integrated with MSW server

**Decision: Use MSW WebSocket Support**

**Rationale:**
1. Unified mocking strategy reduces cognitive load
2. MSW handles setup/teardown automatically
3. Better long-term maintainability
4. Type-safe WebSocket handlers
5. Eliminates fake timer issues

**Migration Plan:**
1. Create MSW WebSocket handlers alongside HTTP handlers
2. Update test utilities to use MSW server
3. Migrate tests file-by-file (start with failing tests)
4. Remove jest-websocket-mock dependency after migration complete

---

### Decision 2: Test Configuration Strategy

**Option A: Multiple Vitest Configs** ⭐ **RECOMMENDED**

Create separate config files for different test categories:

```
keyrx_ui/
├── vite.config.ts                    # Build config + shared test config
├── vitest.config.ts                  # Base test config (imported by others)
├── vitest.unit.config.ts             # Unit tests only
├── vitest.integration.config.ts      # Integration tests only
└── playwright.config.ts              # E2E tests
```

**Pros:**
- ✅ Fast feedback (unit tests run in <5s)
- ✅ Clear separation of concerns
- ✅ Different configurations per test type (timeouts, setup files)
- ✅ CI can run test categories in parallel
- ✅ `npm test` only runs unit tests (fast by default)

**Cons:**
- ⚠️ More config files to maintain
- ⚠️ Duplication of shared config (mitigated by importing base config)

**Option B: Single Config with Filters**

Use workspace filtering within single config:

**Pros:**
- ✅ Single source of truth
- ✅ Less duplication

**Cons:**
- ❌ All tests loaded into memory (slow startup)
- ❌ Hard to have different configs per test type
- ❌ `npm test` still slow (loads E2E tests)

**Decision: Multiple Vitest Configs**

**Rationale:**
1. Developer experience: `npm test` should be <5 seconds
2. CI efficiency: Parallel test execution
3. Flexibility: Different timeouts/setup per test type
4. Industry standard: React, Vue, Angular all use multiple configs

**Implementation:**

```typescript
// vitest.config.ts (base config)
import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: './src/test/setup.ts',
      css: true,
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html'],
        exclude: ['node_modules/**', 'dist/**', 'src/test/**', '**/*.test.{ts,tsx}'],
        thresholds: { lines: 80, functions: 80, branches: 80, statements: 80 },
      },
    },
  })
);

// vitest.unit.config.ts
import { defineConfig, mergeConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      include: [
        'src/**/*.test.{ts,tsx}',
        '!src/**/__integration__/*.test.{ts,tsx}',
        '!tests/integration/**',
        '!tests/a11y/**',
        '!tests/performance/**',
        '!e2e/**',
      ],
      testTimeout: 5000, // Fast unit tests
    },
  })
);

// vitest.integration.config.ts
import { defineConfig, mergeConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      include: [
        'src/**/__integration__/*.test.{ts,tsx}',
        'tests/integration/**/*.test.{ts,tsx}',
        'tests/a11y/**/*.test.{ts,tsx}',
      ],
      testTimeout: 30000, // Longer timeout for integration tests
    },
  })
);
```

---

### Decision 3: Test Utility Architecture

**Extend Existing `testUtils.tsx`**

Current structure is good, extend it with MSW WebSocket support:

```typescript
// tests/testUtils.tsx (extended)
import { setupServer } from 'msw/node';
import { http, ws } from 'msw';
import { createWebSocketHandlers } from './msw/websocketHandlers';

// Re-export existing utilities
export { renderWithProviders, renderPage, renderPure } from './testUtils.base';

// MSW server with both HTTP and WebSocket handlers
export const server = setupServer(
  ...httpHandlers,
  ...createWebSocketHandlers()
);

// Setup/teardown in global setup file (src/test/setup.ts)
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

**Benefits:**
1. Backward compatible with existing tests
2. Automatic WebSocket mocking for all tests
3. No manual setup/teardown required
4. Consistent with HTTP mocking

---

## Detailed Design

### 1. MSW WebSocket Handler Architecture

**File Structure:**
```
src/test/mocks/
├── server.ts                    # MSW server with HTTP + WebSocket handlers
├── handlers.ts                  # HTTP handlers (existing)
├── websocketHandlers.ts         # WebSocket handlers (NEW)
└── websocketHelpers.ts          # Helper functions for tests (NEW)
```

**WebSocket Handler Implementation:**

```typescript
// src/test/mocks/websocketHandlers.ts
import { ws } from 'msw';
import type { WebSocketClientConnection } from 'msw';

export interface DaemonEvent {
  type: 'event';
  channel: 'daemon-state' | 'latency' | 'key-events';
  data: any;
}

export interface QueryMessage {
  type: 'query';
  method: string;
  params?: any;
}

// Shared state for WebSocket handlers
const connections = new Set<WebSocketClientConnection>();
let activeProfile = 'default';
let daemonRunning = false;

/**
 * Create default WebSocket handlers for KeyRx daemon WebSocket API
 */
export function createWebSocketHandlers() {
  return [
    ws.link('ws://localhost:3030/ws')
      .addEventListener('connection', ({ client }) => {
        connections.add(client);

        // Send initial handshake
        client.send(JSON.stringify({
          type: 'connected',
          sessionId: `test-session-${Date.now()}`,
          timestamp: Date.now(),
        }));

        // Handle incoming messages
        client.addEventListener('message', (event) => {
          try {
            const message = JSON.parse(event.data as string);

            switch (message.type) {
              case 'subscribe':
                // Handle subscription
                handleSubscribe(client, message);
                break;

              case 'query':
                // Handle RPC queries
                handleQuery(client, message);
                break;

              case 'unsubscribe':
                // Handle unsubscription
                handleUnsubscribe(client, message);
                break;

              default:
                console.warn(`Unknown WebSocket message type: ${message.type}`);
            }
          } catch (err) {
            console.debug('Non-JSON message received:', event.data);
          }
        });

        // Handle client disconnection
        client.addEventListener('close', () => {
          connections.delete(client);
        });
      })
  ];
}

function handleSubscribe(client: WebSocketClientConnection, message: any) {
  const { channel } = message;

  // Send initial state for the channel
  switch (channel) {
    case 'daemon-state':
      client.send(JSON.stringify({
        type: 'event',
        channel: 'daemon-state',
        data: { running: daemonRunning, activeProfile },
      }));
      break;

    case 'latency':
      client.send(JSON.stringify({
        type: 'event',
        channel: 'latency',
        data: { avg: 1.2, min: 0.5, max: 3.8, p50: 1.1, p95: 2.5, p99: 3.2 },
      }));
      break;
  }
}

function handleQuery(client: WebSocketClientConnection, message: QueryMessage) {
  const { method, params } = message;

  // Mock responses for common queries
  const responses: Record<string, any> = {
    getProfiles: {
      profiles: [
        { name: 'default', displayName: 'Default Profile', isActive: true },
        { name: 'gaming', displayName: 'Gaming Profile', isActive: false },
      ],
    },
    getDevices: {
      devices: [
        { id: 'device-1', name: 'Test Keyboard 1', path: '/dev/input/event0' },
      ],
    },
    getActiveProfile: {
      profile: activeProfile,
    },
  };

  const response = responses[method] || { error: 'Unknown method' };

  client.send(JSON.stringify({
    type: 'response',
    method,
    data: response,
  }));
}

function handleUnsubscribe(client: WebSocketClientConnection, message: any) {
  // No-op for now, subscriptions are per-connection
}

/**
 * Broadcast event to all connected clients (for test helpers)
 */
export function broadcastEvent(event: DaemonEvent) {
  const message = JSON.stringify(event);
  connections.forEach(client => client.send(message));
}

/**
 * Reset WebSocket handler state (call in afterEach)
 */
export function resetWebSocketState() {
  connections.clear();
  activeProfile = 'default';
  daemonRunning = false;
}
```

**Test Helper Functions:**

```typescript
// src/test/mocks/websocketHelpers.ts
import { broadcastEvent } from './websocketHandlers';

/**
 * Simulate daemon state change
 */
export function setDaemonState(state: { running?: boolean; activeProfile?: string }) {
  broadcastEvent({
    type: 'event',
    channel: 'daemon-state',
    data: state,
  });
}

/**
 * Simulate latency stats update
 */
export function sendLatencyUpdate(stats: { avg: number; min: number; max: number }) {
  broadcastEvent({
    type: 'event',
    channel: 'latency',
    data: stats,
  });
}

/**
 * Simulate key event
 */
export function sendKeyEvent(event: { keyCode: string; type: 'press' | 'release' }) {
  broadcastEvent({
    type: 'event',
    channel: 'key-events',
    data: event,
  });
}
```

**Usage in Tests:**

```typescript
import { renderWithProviders, waitFor } from '../tests/testUtils';
import { setDaemonState } from '../src/test/mocks/websocketHelpers';
import { ActiveProfileCard } from './ActiveProfileCard';

test('displays active profile from WebSocket', async () => {
  const { getByText } = renderWithProviders(<ActiveProfileCard />);

  // WebSocket automatically connects via MSW handlers
  // Wait for initial connection and state
  await waitFor(() => expect(getByText('default')).toBeInTheDocument());

  // Simulate profile change
  setDaemonState({ activeProfile: 'gaming' });

  // Component updates via WebSocket
  await waitFor(() => expect(getByText('gaming')).toBeInTheDocument());
});
```

---

### 2. Vitest Configuration Structure

**Base Config (vitest.config.ts):**

```typescript
import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: './src/test/setup.ts',
      css: true,
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html', 'lcov'],
        exclude: [
          'node_modules/**',
          'dist/**',
          'src/test/**',
          '**/*.test.{ts,tsx}',
          '**/*.spec.{ts,tsx}',
          'src/wasm/pkg/**',
          'e2e/**',
          'tests/performance/**',
        ],
        thresholds: {
          lines: 80,
          functions: 80,
          branches: 80,
          statements: 80,
        },
      },
    },
  })
);
```

**Unit Tests Config (vitest.unit.config.ts):**

```typescript
import { defineConfig, mergeConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      name: 'unit',
      include: [
        'src/**/*.test.{ts,tsx}',
        '!src/**/__integration__/*.test.{ts,tsx}',
        '!tests/**',
        '!e2e/**',
      ],
      exclude: [
        'node_modules/**',
        'dist/**',
        'tests/**',
        'e2e/**',
      ],
      testTimeout: 5000, // Unit tests should be fast
      hookTimeout: 3000,
    },
  })
);
```

**Integration Tests Config (vitest.integration.config.ts):**

```typescript
import { defineConfig, mergeConfig } from 'vitest/config';
import baseConfig from './vitest.config';

export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      name: 'integration',
      include: [
        'src/**/__integration__/*.test.{ts,tsx}',
        'tests/integration/**/*.test.{ts,tsx}',
        'tests/a11y/**/*.test.{ts,tsx}',
      ],
      testTimeout: 30000, // Integration tests can be slower
      hookTimeout: 10000,
    },
  })
);
```

**Package.json Scripts:**

```json
{
  "scripts": {
    "test": "vitest run --config vitest.unit.config.ts",
    "test:watch": "vitest --config vitest.unit.config.ts",
    "test:unit": "vitest run --config vitest.unit.config.ts",
    "test:unit:watch": "vitest --config vitest.unit.config.ts",
    "test:integration": "vitest run --config vitest.integration.config.ts",
    "test:integration:watch": "vitest --config vitest.integration.config.ts",
    "test:coverage": "vitest run --coverage --config vitest.unit.config.ts && vitest run --coverage --config vitest.integration.config.ts",
    "test:all": "npm run test:unit && npm run test:integration && npm run test:e2e",
    "test:e2e": "playwright test e2e",
    "test:e2e:ui": "playwright test e2e --ui",
    "test:performance": "npm run test:perf:bundle && npm run test:lighthouse",
    "test:visual": "playwright test tests/e2e/visual/responsive.spec.ts"
  }
}
```

---

### 3. Test Utilities Enhancement

**Extended testUtils.tsx:**

```typescript
// tests/testUtils.tsx (with MSW WebSocket support)
import React, { ReactElement } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { WasmProviderWrapper } from './WasmProviderWrapper';

// Re-export everything from base testUtils
export * from './testUtils.base';

// Re-export MSW WebSocket helpers
export {
  setDaemonState,
  sendLatencyUpdate,
  sendKeyEvent,
} from '../src/test/mocks/websocketHelpers';

// Re-export MSW server for custom handlers in tests
export { server } from '../src/test/mocks/server';

// Re-export testing utilities
export * from '@testing-library/react';
export { userEvent } from '@testing-library/user-event';

/**
 * Custom render with providers (existing, unchanged)
 */
export function renderWithProviders(
  ui: ReactElement,
  options: TestRenderOptions = {}
): RenderResult {
  // Implementation unchanged from existing code
  // ...
}

/**
 * Wait for WebSocket connection to establish
 * Useful when component needs WebSocket data before rendering
 */
export async function waitForWebSocketConnection(timeout = 1000): Promise<void> {
  await waitFor(
    () => {
      // Check if WebSocket is connected (implementation specific)
      // This is a helper for tests that need to ensure WS is ready
    },
    { timeout }
  );
}

/**
 * Simulate WebSocket disconnection
 * Useful for testing offline/reconnection scenarios
 */
export function simulateWebSocketDisconnect(): void {
  // Trigger disconnect event via MSW
  // Implementation will use MSW's client.close()
}

/**
 * Override MSW WebSocket handlers for specific test
 * @example
 * overrideWebSocketHandlers([
 *   ws.link('ws://localhost:3030/ws')
 *     .addEventListener('connection', ({ client }) => {
 *       client.send(JSON.stringify({ custom: 'response' }));
 *     })
 * ]);
 */
export function overrideWebSocketHandlers(handlers: any[]): void {
  const { server } = require('../src/test/mocks/server');
  server.use(...handlers);
}
```

---

### 4. Migration Strategy

**Phase 1: Setup (Week 1)**
1. Create MSW WebSocket handlers (websocketHandlers.ts)
2. Create WebSocket helper functions (websocketHelpers.ts)
3. Update MSW server to include WebSocket handlers
4. Create multiple Vitest configs (unit, integration)
5. Update package.json scripts

**Phase 2: Migration (Week 2)**
1. Migrate failing WebSocket unit tests (src/api/websocket.test.ts)
   - 10 tests currently failing
   - Convert from jest-websocket-mock to MSW
2. Migrate component integration tests
   - ActiveProfileCard.test.tsx
   - DeviceListCard.test.tsx
   - QuickStatsCard.test.tsx
   - ConfigPage.integration.test.tsx
3. Verify all tests pass with MSW WebSocket handlers

**Phase 3: Cleanup (Week 3)**
1. Remove jest-websocket-mock dependency
2. Update documentation
3. Add examples to test utilities docs
4. Run full test suite to verify 95%+ pass rate

---

## File Organization

**New/Modified Files:**

```
keyrx_ui/
├── vitest.config.ts                     # Base test config (NEW)
├── vitest.unit.config.ts                # Unit test config (NEW)
├── vitest.integration.config.ts         # Integration test config (NEW)
├── playwright.config.ts                 # E2E config (EXISTING, update)
├── package.json                         # Update scripts (MODIFIED)
├── src/test/
│   ├── setup.ts                         # Add MSW WebSocket setup (MODIFIED)
│   └── mocks/
│       ├── server.ts                    # Add WebSocket handlers (MODIFIED)
│       ├── handlers.ts                  # Existing HTTP handlers (UNCHANGED)
│       ├── websocketHandlers.ts         # WebSocket handlers (NEW)
│       └── websocketHelpers.ts          # Test helpers (NEW)
├── tests/
│   ├── testUtils.tsx                    # Extend with WS helpers (MODIFIED)
│   └── helpers/
│       └── websocket.ts                 # DEPRECATED (remove after migration)
└── docs/
    └── testing/
        ├── unit-testing-guide.md        # Add WebSocket testing examples (NEW)
        └── integration-testing-guide.md # Add integration test guide (NEW)
```

---

## Performance Considerations

**Test Execution Time Targets:**

| Test Category | Current | Target | Strategy |
|---------------|---------|--------|----------|
| Unit tests | ~10s (with failures) | <5s | Separate config, parallel execution |
| Integration tests | ~45s (with failures) | <30s | MSW auto-cleanup, proper isolation |
| E2E tests | N/A (not running) | <3min | Playwright sharding |

**Optimization Strategies:**

1. **Parallel Execution:**
   - Unit tests run in parallel (Vitest default)
   - Integration tests run sequentially (if needed for stability)
   - E2E tests sharded across 4 workers

2. **Test Isolation:**
   - MSW handlers reset after each test
   - React Query cache cleared after each test
   - WebSocket state reset automatically

3. **Selective Test Running:**
   - `npm test` runs only unit tests (fast feedback)
   - CI runs all test categories in parallel
   - Developers can run specific categories as needed

---

## Risk Analysis

### Risk 1: MSW WebSocket Compatibility with react-use-websocket

**Impact**: HIGH
**Probability**: MEDIUM

**Mitigation:**
- Test MSW WebSocket with react-use-websocket early in Phase 1
- Create proof-of-concept with one component test
- If incompatible, keep jest-websocket-mock as fallback
- Document any workarounds needed

**Contingency Plan:**
- If MSW WebSocket doesn't work: Keep jest-websocket-mock
- Focus on test categorization instead (still fixes 25 failing E2E tests)
- Revisit MSW WebSocket in future when library matures

### Risk 2: Test Migration Breaking Existing Tests

**Impact**: MEDIUM
**Probability**: MEDIUM

**Mitigation:**
- Migrate one test file at a time
- Run full test suite after each migration
- Keep jest-websocket-mock until all tests migrated
- Use feature flags to enable/disable MSW WebSocket per test

**Contingency Plan:**
- Rollback to jest-websocket-mock if migration causes instability
- Migrate only critical tests first (websocket.test.ts, ConfigPage)

### Risk 3: Performance Regression

**Impact**: LOW
**Probability**: LOW

**Mitigation:**
- Benchmark test execution time before and after
- Monitor CI pipeline duration
- Optimize MSW handlers if needed (lazy initialization)

**Contingency Plan:**
- Adjust Vitest parallelism settings
- Use `vitest --no-coverage` for faster local development

---

## Testing Strategy

### How to Test the Test Infrastructure

**1. Smoke Tests:**
```bash
# Verify all test categories run
npm run test:unit        # Should complete in <5s
npm run test:integration # Should complete in <30s
npm run test:e2e        # Should complete in <3min

# Verify coverage
npm run test:coverage   # Should meet 80% thresholds
```

**2. Integration Tests for Test Utilities:**

```typescript
// tests/testUtils.test.ts
import { renderWithProviders } from './testUtils';
import { setDaemonState } from '../src/test/mocks/websocketHelpers';

test('WebSocket mock sends initial handshake', async () => {
  const TestComponent = () => {
    const [message, setMessage] = useState(null);
    useEffect(() => {
      const ws = new WebSocket('ws://localhost:3030/ws');
      ws.onmessage = (e) => setMessage(JSON.parse(e.data));
    }, []);
    return <div>{message?.type}</div>;
  };

  const { getByText } = renderWithProviders(<TestComponent />);
  await waitFor(() => expect(getByText('connected')).toBeInTheDocument());
});

test('WebSocket helper broadcasts events', async () => {
  // Test that setDaemonState() successfully broadcasts to all clients
});
```

**3. Verify Test Isolation:**

```typescript
test('tests do not share WebSocket state', async () => {
  // Run two tests sequentially
  // Verify second test doesn't see first test's state
});
```

---

## Success Metrics

1. **Test Pass Rate:**
   - Unit tests: >99% (currently 88%)
   - Integration tests: >95% (currently 50%)
   - E2E tests: >90% (currently 0% due to config issues)

2. **Test Execution Time:**
   - Unit tests: <5 seconds (currently ~10s with failures)
   - Integration tests: <30 seconds (currently timeout at 45s)
   - Full suite: <5 minutes in CI

3. **Developer Experience:**
   - `npm test` passes on clean install
   - Test output is clear and actionable
   - No manual WebSocket setup in test files

4. **Code Quality:**
   - 80% line coverage maintained
   - Zero test flakiness (deterministic)
   - All tests pass in both CI and local environments

---

## Appendix

### A. MSW WebSocket API Reference

**Key APIs:**

```typescript
import { ws } from 'msw';

// Create WebSocket handler
ws.link('ws://localhost:3030/ws')
  .addEventListener('connection', ({ client }) => {
    // client: WebSocketClientConnection

    // Send message to client
    client.send('Hello');
    client.send(JSON.stringify({ type: 'connected' }));

    // Listen for messages from client
    client.addEventListener('message', (event) => {
      console.log('Received:', event.data);
    });

    // Handle close
    client.addEventListener('close', () => {
      console.log('Client disconnected');
    });
  });
```

### B. Alternative Approaches Considered

**Approach 1: Fix jest-websocket-mock Issues**

Keep jest-websocket-mock, but fix the timing issues:
- Use `vi.useRealTimers()` for WebSocket tests
- Add proper cleanup in `afterEach`
- Wrap WebSocket operations in `act()`

**Rejected because:**
- Still maintains two mocking libraries (MSW + jest-websocket-mock)
- Doesn't address race conditions with react-use-websocket
- More complex test setup (manual mock initialization)

**Approach 2: Use Real WebSocket Server in Tests**

Start actual WebSocket server for integration tests:
- Use `keyrx_daemon` in test mode
- Start server before tests, shut down after

**Rejected because:**
- Much slower (actual server startup)
- More complex CI setup
- Harder to simulate edge cases (disconnections, errors)
- Doesn't help with unit tests

---

**Created**: 2026-01-04
**Spec Name**: frontend-test-infrastructure
**Status**: Design Complete
**Next Step**: Create tasks.md
