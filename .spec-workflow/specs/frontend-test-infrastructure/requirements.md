# Requirements Document

## Introduction

The KeyRx frontend currently has 134 failing tests out of 1109 total tests (12% failure rate), with 44 test files affected. The root cause is inadequate WebSocket mocking infrastructure and improper test categorization. E2E tests (Playwright), performance tests (Lighthouse), and visual regression tests (Percy) are mixed with unit tests, causing failures when special tools aren't available. The WebSocket mocking uses `vi.useFakeTimers()` with `MockWebSocket`, creating async/timing conflicts that break integration tests.

This specification addresses the systematic refactoring of the frontend test infrastructure to achieve >95% pass rate for unit/integration tests, proper separation of test categories, and production-ready WebSocket mocking using Mock Service Worker (MSW).

## Alignment with Product Vision

This specification directly supports KeyRx's product vision by:

1. **AI-First Testing**: Enables autonomous AI agents to verify implementations without human UAT
2. **Development Velocity**: Developers get fast, reliable feedback from unit tests (<5s run time)
3. **CI/CD Reliability**: Proper test categorization enables efficient CI pipelines (unit tests → integration tests → E2E tests)
4. **Code Quality**: Comprehensive test coverage prevents regressions and ensures production readiness

## Requirements

### Requirement 1: Refactor WebSocket Mocking with MSW

**User Story**: As a developer running unit tests, I want WebSocket connections to be mocked reliably without timing issues, so that integration tests pass consistently and provide accurate feedback

**Priority**: 🔴 **CRITICAL** - Blocks ~70% of failing tests

#### Acceptance Criteria

1. WHEN test imports `renderWithProviders` THEN WebSocket connections SHALL be mocked automatically via MSW
2. WHEN component connects to WebSocket THEN MSW SHALL intercept connection and simulate server behavior
3. WHEN test sends WebSocket message THEN MSW SHALL receive message and respond according to handlers
4. WHEN test expects WebSocket event THEN event SHALL be delivered without timing race conditions
5. IF test needs custom WebSocket behavior THEN test SHALL override default MSW handlers
6. WHEN integration test runs THEN WebSocket SHALL connect/disconnect cleanly without errors
7. WHEN multiple tests run in sequence THEN WebSocket state SHALL reset between tests

#### Technical Context

**Current Issues**:
- `vi.useFakeTimers()` + `MockWebSocket` setTimeout = race conditions
- 10/17 WebSocket tests failing due to timing issues
- All integration tests using WebSocket fail (~20 files)

**Solution: MSW (Mock Service Worker)**:
```typescript
// Setup MSW WebSocket handler
import { ws } from 'msw';
import { setupServer } from 'msw/node';

const server = setupServer(
  ws.link('ws://localhost:3030/ws').addEventListener('connection', ({ client }) => {
    client.addEventListener('message', (event) => {
      // Handle messages
      if (event.data === 'ping') {
        client.send('pong');
      }
    });
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

**Benefits**:
- No fake timers needed - real async behavior
- Type-safe WebSocket message handling
- Easy to customize per-test behavior
- Production-like WebSocket interactions

---

### Requirement 2: Separate Test Categories with Vitest Config

**User Story**: As a developer running `npm test`, I want only unit and integration tests to run, so that I get fast feedback without needing Playwright/Lighthouse/Percy installed

**Priority**: 🟡 **HIGH** - Improves developer experience significantly

#### Acceptance Criteria

1. WHEN developer runs `npm test` THEN only unit and integration tests SHALL execute
2. WHEN developer runs `npm run test:e2e` THEN only Playwright E2E tests SHALL execute
3. WHEN developer runs `npm run test:performance` THEN only Lighthouse/bundle tests SHALL execute
4. WHEN developer runs `npm run test:visual` THEN only visual regression tests SHALL execute
5. WHEN developer runs `npm run test:all` THEN all test categories SHALL execute sequentially
6. IF Playwright is not installed THEN `npm test` SHALL still succeed (E2E tests skipped)
7. WHEN CI runs THEN test categories SHALL run in parallel for speed

#### Technical Context

**Current Issue**: All test types mixed together
```
npm test → Runs everything → Fails due to missing Playwright/Lighthouse
```

**Solution**: Separate test scripts
```json
{
  "scripts": {
    "test": "vitest run --config vitest.unit.config.ts",
    "test:unit": "vitest run --config vitest.unit.config.ts",
    "test:integration": "vitest run --config vitest.integration.config.ts",
    "test:e2e": "playwright test",
    "test:performance": "lighthouse ci",
    "test:visual": "percy exec -- vitest run tests/visual/",
    "test:all": "npm run test:unit && npm run test:integration && npm run test:e2e"
  }
}
```

**Test File Patterns**:
- Unit tests: `src/**/*.test.{ts,tsx}` (NOT in `__integration__/` dirs)
- Integration tests: `src/**/__integration__/*.test.{ts,tsx}`, `tests/integration/*.test.tsx`
- E2E tests: `e2e/**/*.spec.ts`, `tests/e2e/**/*.spec.ts`
- Performance tests: `tests/performance/**/*.spec.ts`
- Visual tests: `tests/visual/**/*.spec.ts`

---

### Requirement 3: Fix WebSocket Unit Tests

**User Story**: As a developer verifying WebSocket functionality, I want all WebSocket unit tests to pass with reliable mocking, so that I can trust the test results when modifying WebSocket code

**Priority**: 🟡 **HIGH** - Currently 10/17 tests failing

#### Acceptance Criteria

1. WHEN test creates WebSocketManager THEN connection SHALL simulate properly with MSW
2. WHEN test calls `wsManager.connect()` THEN onOpen callback SHALL be invoked
3. WHEN test calls `wsManager.send(message)` THEN message SHALL be sent to MSW handler
4. WHEN MSW handler sends message THEN onMessage callback SHALL receive message
5. WHEN test calls `wsManager.disconnect()` THEN onClose callback SHALL be invoked
6. WHEN test creates multiple WebSocketManager instances THEN each SHALL have isolated state
7. IF connection fails THEN onError callback SHALL be invoked with error details

#### Technical Context

**Failing Tests** (`src/api/websocket.test.ts`):
- ✗ should connect to WebSocket server
- ✗ should not create duplicate connections
- ✗ should not reconnect after close()
- ✗ should handle event messages
- ✗ should handle state messages
- ✗ should handle latency messages
- ✗ should handle invalid JSON gracefully
- ✗ should track connection state changes
- ✗ should send string messages
- ✗ should send object messages as JSON

**Fix Approach**:
1. Replace `MockWebSocket` with MSW WebSocket handler
2. Remove `vi.useFakeTimers()` - use real async
3. Add `await waitFor()` for async assertions
4. Ensure handlers reset between tests

---

### Requirement 4: Fix Component Integration Tests

**User Story**: As a developer testing components with WebSocket dependencies, I want integration tests to pass reliably, so that I can verify real-world component behavior

**Priority**: 🟡 **HIGH** - Affects ~15 component test files

#### Acceptance Criteria

1. WHEN component uses `useUnifiedApi()` hook THEN WebSocket SHALL connect via MSW
2. WHEN component receives WebSocket message THEN UI SHALL update correctly
3. WHEN component unmounts THEN WebSocket SHALL disconnect cleanly
4. WHEN test renders multiple components THEN each SHALL have independent WebSocket state
5. IF WebSocket connection fails THEN component SHALL show error state
6. WHEN test needs specific WebSocket behavior THEN test SHALL provide custom MSW handler

#### Technical Context

**Affected Components** (failing tests):
- `ActiveProfileCard.test.tsx` - WebSocket state updates
- `DeviceListCard.test.tsx` - Real-time device updates
- `QuickStatsCard.test.tsx` - Metrics via WebSocket
- `ConfigPage.integration.test.tsx` - Full page with WebSocket
- `hooks/useUnifiedApi.test.ts` - Hook with WebSocket config

**Common Pattern**:
```typescript
// Before: Manual mock WebSocket
const mockWs = { connect: vi.fn(), send: vi.fn() };

// After: MSW auto-handles WebSocket
const { getByText } = renderWithProviders(<ActiveProfileCard />);
await waitFor(() => expect(getByText('Active Profile')).toBeInTheDocument());
```

---

### Requirement 5: Create MSW Test Utilities

**User Story**: As a developer writing new tests, I want pre-configured MSW handlers for common scenarios, so that I can quickly write tests without boilerplate

**Priority**: 🟢 **MEDIUM** - Quality of life improvement

#### Acceptance Criteria

1. WHEN test imports `renderWithProviders` THEN MSW server SHALL start automatically
2. WHEN test imports `createMockWebSocketHandlers()` THEN helper SHALL return common handlers
3. WHEN test needs device list response THEN `mockDevices(devices)` SHALL configure API response
4. WHEN test needs profile list response THEN `mockProfiles(profiles)` SHALL configure API response
5. WHEN test needs WebSocket events THEN `sendWebSocketEvent(event)` SHALL simulate server message
6. IF test needs custom behavior THEN test SHALL use MSW `server.use()` for one-off handlers

#### Technical Context

**Utility Functions**:
```typescript
// tests/msw/handlers.ts
export const createMockWebSocketHandlers = (config?: WebSocketConfig) => {
  return [
    ws.link('ws://localhost:3030/ws').addEventListener('connection', ({ client }) => {
      client.addEventListener('message', (event) => {
        // Default handlers for common events
      });
    })
  ];
};

export const mockDevices = (devices: Device[]) => {
  server.use(
    http.get('/api/devices', () => HttpResponse.json({ devices }))
  );
};

export const sendWebSocketEvent = (client: WebSocketClient, event: DaemonEvent) => {
  client.send(JSON.stringify(event));
};
```

**Integration with `renderWithProviders`**:
```typescript
// tests/testUtils.tsx
import { setupServer } from 'msw/node';
import { createMockWebSocketHandlers } from './msw/handlers';

const server = setupServer(...createMockWebSocketHandlers());

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

export function renderWithProviders(ui, options) {
  // Existing provider wrapping + MSW
}
```

---

### Requirement 6: Update E2E Test Configuration

**User Story**: As a CI/CD pipeline, I want E2E tests to run separately with proper Playwright setup, so that they don't block fast unit test feedback

**Priority**: 🟢 **MEDIUM** - CI/CD optimization

#### Acceptance Criteria

1. WHEN E2E tests run THEN Playwright SHALL start test server automatically
2. WHEN E2E tests finish THEN test server SHALL shut down cleanly
3. WHEN developer runs `npm run test:e2e` THEN Playwright SHALL execute all e2e/**/*.spec.ts files
4. IF Playwright is not installed THEN error message SHALL guide installation
5. WHEN CI runs THEN E2E tests SHALL run in parallel across shards
6. WHEN E2E test fails THEN screenshot and trace SHALL be saved for debugging

#### Technical Context

**Playwright Configuration** (`playwright.config.ts`):
```typescript
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.spec.ts',
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
  ],
});
```

**Separation from Unit Tests**:
- Unit tests: `vitest` (fast, no server)
- E2E tests: `playwright` (slow, requires server)
- CI: Run unit tests first → If pass, run E2E tests

---

## Non-Functional Requirements

### Performance

1. **Unit Test Speed**:
   - SHALL complete <5 seconds for all unit tests
   - SHALL complete <30 seconds for all integration tests
   - SHALL complete <3 minutes for all E2E tests

2. **CI Pipeline**:
   - Unit tests SHALL run in <10 seconds in CI (parallel execution)
   - Integration tests SHALL run in <60 seconds in CI
   - E2E tests SHALL run in <5 minutes in CI (sharded across 4 workers)

### Reliability

1. **Test Flakiness**:
   - Unit tests: 0% flake rate (deterministic mocking)
   - Integration tests: <1% flake rate (MSW reliable)
   - E2E tests: <5% flake rate (Playwright retry logic)

2. **Test Isolation**:
   - Tests SHALL NOT share state between runs
   - MSW handlers SHALL reset after each test
   - Test order SHALL NOT affect outcomes

### Maintainability

1. **Documentation**:
   - MSW handlers SHALL have JSDoc comments explaining behavior
   - Test utilities SHALL include usage examples
   - README SHALL document test categories and how to run each

2. **Code Quality**:
   - Test code SHALL follow same quality standards as production code
   - No commented-out tests (delete or fix)
   - Test names SHALL clearly describe expected behavior

### Developer Experience

1. **Error Messages**:
   - IF MSW handler missing THEN error SHALL suggest which handler to add
   - IF Playwright not installed THEN error SHALL include install command
   - IF test fails THEN error SHALL show which assertion failed with context

2. **Debugging**:
   - MSW SHALL log all intercepted requests in debug mode
   - Playwright SHALL capture screenshots on failure
   - Test utilities SHALL provide `debug()` helper to inspect state

---

## Out of Scope

1. **Test Coverage Improvements**: This spec fixes test infrastructure, not coverage gaps
2. **New Test Cases**: Only fixes existing failing tests, doesn't add new tests
3. **Performance Test Tools**: Lighthouse/bundle analysis config is out of scope
4. **Visual Regression Setup**: Percy/Chromatic integration is out of scope
5. **Backend Test Infrastructure**: Only frontend tests are addressed

---

## Success Metrics

1. **Test Pass Rate**:
   - Unit tests: >99% pass rate (was 88%)
   - Integration tests: >95% pass rate (was 50%)
   - E2E tests: >90% pass rate (was 0% due to missing setup)

2. **Test Execution Time**:
   - Unit tests: <5 seconds (currently ~10 seconds with failures)
   - Integration tests: <30 seconds (currently timeout failures)
   - E2E tests: <3 minutes for full suite

3. **Developer Satisfaction**:
   - `npm test` passes on clean install (currently fails)
   - Test output is clear and actionable
   - CI pipeline is reliable and fast

4. **Test Reliability**:
   - Zero flaky unit tests
   - <1% flaky integration tests
   - <5% flaky E2E tests (acceptable for browser tests)

---

## References

- [MSW WebSocket Support](https://mswjs.io/docs/network-behavior/websocket) - Official MSW docs
- [Vitest Configuration](https://vitest.dev/config/) - Multiple config files
- [Playwright Best Practices](https://playwright.dev/docs/best-practices) - E2E test patterns
- [Testing Library Best Practices](https://testing-library.com/docs/guiding-principles) - Component testing
- [React Testing Patterns 2026](https://react.dev/learn/testing) - Modern React testing

---

**Created**: 2026-01-04
**Spec Name**: frontend-test-infrastructure
**Status**: Requirements Complete
