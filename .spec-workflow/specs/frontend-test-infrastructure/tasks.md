# Tasks Document

## Phase 1: MSW WebSocket Handler Implementation (CRITICAL - Fixes 70% of Failures)

### 1.1. Create MSW WebSocket Handlers

- [x] 1.1.1. Implement WebSocket handler architecture
  - Files: `keyrx_ui/src/test/mocks/websocketHandlers.ts`
  - Implement `createWebSocketHandlers()` function returning MSW WebSocket handlers
  - Handle connection, subscribe, query, and unsubscribe message types
  - Implement state management for connections and daemon state
  - Implement `handleSubscribe()` for channel subscriptions (daemon-state, latency, key-events)
  - Implement `handleQuery()` for RPC queries (getProfiles, getDevices, getActiveProfile)
  - Implement `broadcastEvent()` for test helper functions
  - Implement `resetWebSocketState()` for test isolation
  - _Leverage: `msw` v2.12.7 `ws.link()` API, TypeScript strict mode_
  - _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript WebSocket Developer with MSW expertise | Task: Implement comprehensive WebSocket handler architecture in keyrx_ui/src/test/mocks/websocketHandlers.ts, creating MSW-based WebSocket mock server with connection management, message routing, and state management | Restrictions: Must use MSW v2 ws.link() API, implement proper connection cleanup, ensure thread-safe state management, handle JSON parsing gracefully, do not use global mutable state | _Leverage: msw v2.12.7 ws.link() API for WebSocket mocking, TypeScript Map/Set for connection tracking | _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW) - Remove fake timer issues, enable type-safe message handling | Success: WebSocket handler connects clients automatically, routes messages by type correctly, state resets between tests, no race conditions with client lifecycle | Instructions: 1. Edit tasks.md to mark this task [-] as in-progress. 2. Implement the code. 3. Use log-implementation tool to record: taskId="1.1.1", summary (1-2 sentences), filesModified, filesCreated, statistics (linesAdded, linesRemoved), and REQUIRED artifacts field with functions (createWebSocketHandlers, handleSubscribe, handleQuery, broadcastEvent, resetWebSocketState), classes (if any), and integrations (MSW server integration). 4. Edit tasks.md to mark this task [x] as completed._

- [x] 1.1.2. Create WebSocket test helper functions
  - Files: `keyrx_ui/src/test/mocks/websocketHelpers.ts`
  - Implement `setDaemonState(state)` for simulating daemon state changes
  - Implement `sendLatencyUpdate(stats)` for simulating latency events
  - Implement `sendKeyEvent(event)` for simulating key events
  - Implement `simulateDisconnect()` for testing offline scenarios
  - Implement `waitForWebSocketConnection()` utility
  - _Leverage: `broadcastEvent()` from websocketHandlers.ts_
  - _Requirements: Requirement 5 (Create MSW Test Utilities)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Test Utility Developer | Task: Create WebSocket test helper functions in keyrx_ui/src/test/mocks/websocketHelpers.ts, providing convenient APIs for simulating daemon events in tests | Restrictions: Must use broadcastEvent() from websocketHandlers.ts, ensure type safety for event payloads, add JSDoc documentation with examples | _Leverage: broadcastEvent() function, TypeScript interfaces for type safety | _Requirements: Requirement 5 (Create MSW Test Utilities) - Pre-configured handlers for common scenarios | Success: Helper functions are easy to use, well-documented with examples, type-safe, work reliably in tests | Instructions: 1. Mark [-]. 2. Implement helpers. 3. Log (taskId="1.1.2", artifacts with functions and usage examples). 4. Mark [x]._

- [x] 1.1.3. Update MSW server to include WebSocket handlers
  - Files: `keyrx_ui/src/test/mocks/server.ts`
  - Import `createWebSocketHandlers()` from websocketHandlers.ts
  - Add WebSocket handlers to MSW server setup
  - Update server configuration to handle WebSocket connections
  - _Leverage: Existing MSW server setup, `setupServer()` from msw/node_
  - _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: MSW Configuration Specialist | Task: Update MSW server in keyrx_ui/src/test/mocks/server.ts to include WebSocket handlers alongside existing HTTP handlers | Restrictions: Do not break existing HTTP mocking, ensure WebSocket and HTTP handlers coexist, verify server starts correctly in test setup | _Leverage: setupServer() from msw/node, createWebSocketHandlers() from task 1.1.1 | _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW) - Automatic WebSocket mocking via MSW | Success: Server includes both HTTP and WebSocket handlers, tests run without manual WebSocket setup, no breaking changes to existing tests | Instructions: 1. Mark [-]. 2. Update server.ts. 3. Log (taskId="1.1.3", summary, filesModified, integrations). 4. Mark [x]._

- [x] 1.1.4. Update test setup to use MSW WebSocket handlers
  - Files: `keyrx_ui/src/test/setup.ts`
  - Remove jest-websocket-mock setup code
  - Ensure MSW server starts with WebSocket handlers
  - Add `resetWebSocketState()` to afterEach cleanup
  - Update comments to reflect MSW WebSocket usage
  - _Leverage: MSW server from task 1.1.3, `resetWebSocketState()` from task 1.1.1_
  - _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Infrastructure Engineer | Task: Update global test setup in keyrx_ui/src/test/setup.ts to use MSW WebSocket handlers, removing jest-websocket-mock dependencies | Restrictions: Ensure backward compatibility with existing tests, verify MSW server lifecycle (beforeAll, afterEach, afterAll) works correctly, do not break HTTP mocking | _Leverage: MSW server with WebSocket handlers, resetWebSocketState() for cleanup | _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW) - Automatic WebSocket cleanup | Success: All tests use MSW WebSocket automatically, no manual setup required, state resets between tests, no test pollution | Instructions: 1. Mark [-]. 2. Update setup.ts. 3. Log (taskId="1.1.4"). 4. Mark [x]._

### 1.2. Update Test Utilities

- [x] 1.2.1. Extend renderWithProviders with WebSocket support
  - Files: `keyrx_ui/tests/testUtils.tsx`
  - Re-export WebSocket helper functions from websocketHelpers.ts
  - Add `waitForWebSocketConnection()` utility
  - Add `overrideWebSocketHandlers()` for custom test scenarios
  - Update JSDoc documentation with WebSocket examples
  - _Leverage: Existing renderWithProviders, WebSocket helpers from task 1.1.2_
  - _Requirements: Requirement 5 (Create MSW Test Utilities)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Testing Library Expert | Task: Extend testUtils.tsx with WebSocket support, re-exporting helper functions and adding utilities for WebSocket testing | Restrictions: Do not break existing renderWithProviders API, maintain backward compatibility, add comprehensive JSDoc with examples | _Leverage: Existing renderWithProviders implementation, WebSocket helpers from task 1.1.2 | _Requirements: Requirement 5 (Create MSW Test Utilities) - Easy access to WebSocket helpers | Success: Tests can import WebSocket helpers from testUtils, documentation is clear, backward compatible | Instructions: 1. Mark [-]. 2. Extend testUtils.tsx. 3. Log (taskId="1.2.1", artifacts with functions). 4. Mark [x]._

## Phase 2: Vitest Configuration Setup (Separates Test Categories)

### 2.1. Create Base Vitest Configuration

- [x] 2.1.1. Create base Vitest config
  - Files: `keyrx_ui/vitest.config.base.ts` (NEW), `keyrx_ui/vite.config.ts` (updated to use vitest/config)
  - Extract shared test configuration from vite.config.ts
  - Define base test settings (globals, environment, setupFiles, coverage)
  - Export config for reuse by unit and integration configs
  - _Leverage: Vitest `defineConfig()` and `mergeConfig()`, existing vite.config.ts_
  - _Requirements: Requirement 2 (Separate Test Categories with Vitest Config)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vitest Configuration Specialist | Task: Create base Vitest configuration in keyrx_ui/vitest.config.ts by extracting and organizing test settings from vite.config.ts | Restrictions: Do not break existing build config, ensure test config is separated from build config, use mergeConfig() for proper config composition | _Leverage: Vitest defineConfig() and mergeConfig() APIs, existing vite.config.ts test settings | _Requirements: Requirement 2 (Separate Test Categories with Vitest Config) - Base config for reuse | Success: Base config exports cleanly, can be imported by other configs, no duplication of settings | Instructions: 1. Mark [-]. 2. Create vitest.config.ts. 3. Log (taskId="2.1.1"). 4. Mark [x]._

- [ ] 2.1.2. Create unit test configuration
  - Files: `keyrx_ui/vitest.unit.config.ts`
  - Import and merge base vitest.config.ts
  - Include only unit test files (src/**/*.test.{ts,tsx}, exclude integration/a11y/e2e/performance)
  - Set fast timeout (5000ms for tests, 3000ms for hooks)
  - Configure test name as "unit" for CI reporting
  - _Leverage: Base config from task 2.1.1, Vitest `mergeConfig()`_
  - _Requirements: Requirement 2 (Separate Test Categories with Vitest Config)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vitest Configuration Engineer | Task: Create unit test configuration in keyrx_ui/vitest.unit.config.ts, separating fast unit tests from slower integration tests | Restrictions: Must exclude integration, a11y, e2e, and performance tests, use short timeouts for fast feedback, import base config via mergeConfig() | _Leverage: Base vitest.config.ts from task 2.1.1, Vitest mergeConfig() API | _Requirements: Requirement 2 (Separate Test Categories with Vitest Config) - Fast unit test execution | Success: Unit tests run in <5 seconds, integration tests excluded, timeout failures caught quickly | Instructions: 1. Mark [-]. 2. Create vitest.unit.config.ts. 3. Log (taskId="2.1.2"). 4. Mark [x]._

- [ ] 2.1.3. Create integration test configuration
  - Files: `keyrx_ui/vitest.integration.config.ts`
  - Import and merge base vitest.config.ts
  - Include integration, a11y test files (src/**/__integration__/*.test.{ts,tsx}, tests/integration/**, tests/a11y/**)
  - Set longer timeout (30000ms for tests, 10000ms for hooks)
  - Configure test name as "integration" for CI reporting
  - _Leverage: Base config from task 2.1.1, Vitest `mergeConfig()`_
  - _Requirements: Requirement 2 (Separate Test Categories with Vitest Config)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vitest Configuration Engineer | Task: Create integration test configuration in keyrx_ui/vitest.integration.config.ts for slower integration and accessibility tests | Restrictions: Must include only integration and a11y tests, use longer timeouts (30s), import base config via mergeConfig() | _Leverage: Base vitest.config.ts from task 2.1.1, Vitest mergeConfig() API | _Requirements: Requirement 2 (Separate Test Categories with Vitest Config) - Separate integration test execution | Success: Integration tests run with adequate timeouts, unit tests not included, CI reports show "integration" test suite | Instructions: 1. Mark [-]. 2. Create vitest.integration.config.ts. 3. Log (taskId="2.1.3"). 4. Mark [x]._

### 2.2. Update npm Scripts

- [ ] 2.2.1. Update package.json test scripts
  - Files: `keyrx_ui/package.json`
  - Update `test` script to use vitest.unit.config.ts (fast by default)
  - Update `test:watch` to use vitest.unit.config.ts
  - Add `test:unit` and `test:unit:watch` scripts
  - Add `test:integration` and `test:integration:watch` scripts
  - Update `test:coverage` to run both unit and integration tests
  - Update `test:all` to run all test categories sequentially
  - _Leverage: New Vitest configs from tasks 2.1.2 and 2.1.3_
  - _Requirements: Requirement 2 (Separate Test Categories with Vitest Config)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: npm Scripts Developer | Task: Update package.json test scripts to use new Vitest configurations, ensuring npm test runs only fast unit tests | Restrictions: Do not break existing script names, ensure backward compatibility where possible, npm test should be <5 seconds | _Leverage: vitest.unit.config.ts and vitest.integration.config.ts from Phase 2.1 | _Requirements: Requirement 2 (Separate Test Categories with Vitest Config) - Fast default test command | Success: npm test runs unit tests only (<5s), test:integration runs integration tests, test:all runs everything | Instructions: 1. Mark [-]. 2. Update package.json scripts section. 3. Log (taskId="2.2.1"). 4. Mark [x]._

## Phase 3: WebSocket Test Migration (Fixes 10/17 Failing Tests)

### 3.1. Migrate WebSocket Unit Tests

- [ ] 3.1.1. Migrate websocket.test.ts to use MSW
  - Files: `keyrx_ui/src/api/websocket.test.ts`
  - Remove MockWebSocket class definition
  - Remove `vi.useFakeTimers()` and `vi.useRealTimers()` (not needed with MSW)
  - Update tests to rely on MSW WebSocket handlers instead of MockWebSocket
  - Use `waitFor()` from @testing-library/react for async assertions
  - Fix 10 failing tests: connection, duplicate connections, disconnect, reconnect, event/state/latency messages, invalid JSON, state tracking, send messages
  - _Leverage: MSW WebSocket handlers from Phase 1.1, waitFor() from React Testing Library_
  - _Requirements: Requirement 3 (Fix WebSocket Unit Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: WebSocket Test Migration Specialist | Task: Migrate websocket.test.ts from MockWebSocket to MSW WebSocket handlers, fixing all 10 failing tests | Restrictions: Remove ALL MockWebSocket code, do not use fake timers (use real async with waitFor), ensure all 10 tests pass after migration, verify no race conditions | _Leverage: MSW WebSocket handlers from task 1.1.1, waitFor() from @testing-library/react for async assertions | _Requirements: Requirement 3 (Fix WebSocket Unit Tests) - 10/17 tests currently failing due to timer issues | Success: All 10 failing tests now pass, no fake timers used, tests are deterministic and stable, WebSocket lifecycle works correctly | Instructions: 1. Mark [-]. 2. Migrate tests. 3. Log (taskId="3.1.1", summary, filesModified, statistics with linesRemoved showing MockWebSocket removal). 4. Mark [x]._

### 3.2. Verify WebSocket Integration

- [ ] 3.2.1. Add WebSocket integration smoke test
  - Files: `keyrx_ui/tests/integration/websocket-msw.test.ts` (NEW)
  - Test: MSW WebSocket server connects automatically in tests
  - Test: Helper functions (setDaemonState, sendLatencyUpdate) broadcast correctly
  - Test: State resets between tests (no pollution)
  - Test: Custom handlers can override default handlers via server.use()
  - _Leverage: MSW WebSocket handlers from Phase 1.1, test helpers from task 1.1.2_
  - _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer | Task: Create comprehensive WebSocket integration smoke test in keyrx_ui/tests/integration/websocket-msw.test.ts, verifying MSW WebSocket infrastructure works correctly | Restrictions: Must test automatic connection, helper function broadcasting, state isolation, and custom handler overrides | _Leverage: MSW WebSocket handlers from Phase 1.1, setDaemonState/sendLatencyUpdate helpers | _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW) - Verify MSW integration works | Success: Smoke test verifies WebSocket infrastructure, catches regressions, ensures state isolation works | Instructions: 1. Mark [-]. 2. Create smoke test. 3. Log (taskId="3.2.1"). 4. Mark [x]._

## Phase 4: Component Integration Test Fixes (Fixes ~15 Failing Component Tests)

### 4.1. Fix Dashboard Component Tests

- [ ] 4.1.1. Fix ActiveProfileCard integration test
  - Files: `keyrx_ui/src/components/dashboard/ActiveProfileCard.test.tsx`
  - Replace manual WebSocket mock with MSW handlers
  - Use `setDaemonState({ activeProfile: '...' })` to simulate profile changes
  - Fix async assertions with `waitFor()`
  - Verify component updates when WebSocket sends activeProfile change
  - _Leverage: MSW WebSocket handlers, setDaemonState() helper, waitFor()_
  - _Requirements: Requirement 4 (Fix Component Integration Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Component Test Engineer | Task: Fix ActiveProfileCard integration test by replacing manual WebSocket mocking with MSW handlers | Restrictions: Remove manual WebSocket mocks, use setDaemonState() helper for events, verify component renders updated profile name | _Leverage: MSW WebSocket from Phase 1.1, setDaemonState() helper, waitFor() for async assertions | _Requirements: Requirement 4 (Fix Component Integration Tests) - WebSocket state updates | Success: Test passes consistently, component updates via WebSocket, no manual mocking code | Instructions: 1. Mark [-]. 2. Fix test. 3. Log (taskId="4.1.1"). 4. Mark [x]._

- [ ] 4.1.2. Fix DeviceListCard integration test
  - Files: `keyrx_ui/src/components/dashboard/DeviceListCard.test.tsx`
  - Replace manual WebSocket mock with MSW handlers
  - Use `sendServerMessage()` to simulate device_connected events
  - Fix async assertions with `waitFor()`
  - Verify component updates when WebSocket sends device events
  - _Leverage: MSW WebSocket handlers, sendServerMessage() helper_
  - _Requirements: Requirement 4 (Fix Component Integration Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Component Test Engineer | Task: Fix DeviceListCard integration test by using MSW WebSocket handlers for device events | Restrictions: Remove manual mocks, use MSW helpers for device_connected events, verify list updates correctly | _Leverage: MSW WebSocket handlers, sendServerMessage() for custom events | _Requirements: Requirement 4 (Fix Component Integration Tests) - Real-time device updates | Success: Test passes, device list updates via WebSocket, realistic event flow | Instructions: 1. Mark [-]. 2. Fix test. 3. Log (taskId="4.1.2"). 4. Mark [x]._

- [ ] 4.1.3. Fix QuickStatsCard integration test
  - Files: `keyrx_ui/src/components/dashboard/QuickStatsCard.test.tsx`
  - Replace manual WebSocket mock with MSW handlers
  - Use `sendLatencyUpdate({ avg, min, max })` to simulate latency events
  - Fix async assertions with `waitFor()`
  - Verify component renders updated metrics
  - _Leverage: MSW WebSocket handlers, sendLatencyUpdate() helper_
  - _Requirements: Requirement 4 (Fix Component Integration Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Component Test Engineer | Task: Fix QuickStatsCard integration test using MSW WebSocket and sendLatencyUpdate() helper | Restrictions: Remove manual mocks, use sendLatencyUpdate() for metrics, verify component displays updated stats | _Leverage: MSW WebSocket handlers, sendLatencyUpdate() helper function | _Requirements: Requirement 4 (Fix Component Integration Tests) - Metrics via WebSocket | Success: Test passes, stats update via WebSocket, component renders correct values | Instructions: 1. Mark [-]. 2. Fix test. 3. Log (taskId="4.1.3"). 4. Mark [x]._

### 4.2. Fix Page Integration Tests

- [ ] 4.2.1. Fix ConfigPage integration test
  - Files: `keyrx_ui/src/pages/__integration__/ConfigPage.integration.test.tsx`
  - Replace manual WebSocket mock with MSW handlers
  - Use `renderWithProviders(<ConfigPage />, { wrapWithRouter: true })` (Router required for useParams)
  - Use WebSocket helpers for profile state updates
  - Fix async assertions with `waitFor()` and `findBy` queries
  - Verify full page workflow (load config → edit → validate → WebSocket updates)
  - _Leverage: MSW WebSocket handlers, renderWithProviders with router, WebSocket helpers_
  - _Requirements: Requirement 4 (Fix Component Integration Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Specialist | Task: Fix ConfigPage integration test using MSW WebSocket handlers and renderWithProviders with router support | Restrictions: Use renderWithProviders with wrapWithRouter: true, test full page workflow, verify WebSocket state sync | _Leverage: MSW WebSocket handlers, renderWithProviders({ wrapWithRouter: true }), WebSocket helpers | _Requirements: Requirement 4 (Fix Component Integration Tests) - Full page with WebSocket | Success: Integration test covers full workflow, WebSocket syncs state, page renders correctly | Instructions: 1. Mark [-]. 2. Fix integration test. 3. Log (taskId="4.2.1"). 4. Mark [x]._

- [ ] 4.2.2. Fix useUnifiedApi hook test
  - Files: `keyrx_ui/src/hooks/__tests__/useUnifiedApi.test.ts`
  - Replace manual WebSocket mock with MSW handlers
  - Use `renderHook()` from @testing-library/react with QueryClientProvider wrapper
  - Use WebSocket helpers to simulate daemon events
  - Verify hook state updates correctly via WebSocket
  - _Leverage: MSW WebSocket handlers, renderHook(), QueryClientProvider_
  - _Requirements: Requirement 4 (Fix Component Integration Tests)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hooks Test Engineer | Task: Fix useUnifiedApi hook test using MSW WebSocket handlers and renderHook() | Restrictions: Use renderHook() with QueryClientProvider wrapper, verify hook subscribes and unsubscribes correctly, test state updates | _Leverage: MSW WebSocket handlers, renderHook() from @testing-library/react, WebSocket helpers | _Requirements: Requirement 4 (Fix Component Integration Tests) - Hook with WebSocket config | Success: Hook test passes, WebSocket connection/disconnection works, state updates correctly | Instructions: 1. Mark [-]. 2. Fix hook test. 3. Log (taskId="4.2.2"). 4. Mark [x]._

## Phase 5: E2E Test Configuration (Separates E2E from Unit Tests)

### 5.1. Update Playwright Configuration

- [ ] 5.1.1. Configure Playwright webServer
  - Files: `keyrx_ui/playwright.config.ts`
  - Add webServer configuration to start dev server automatically
  - Set webServer.command to `npm run dev`
  - Set webServer.url to `http://localhost:5173`
  - Set webServer.reuseExistingServer to `!process.env.CI` (don't reuse in CI)
  - Configure baseURL to `http://localhost:5173`
  - _Leverage: Playwright defineConfig API_
  - _Requirements: Requirement 6 (Update E2E Test Configuration)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Playwright Configuration Specialist | Task: Configure Playwright webServer in keyrx_ui/playwright.config.ts to automatically start dev server for E2E tests | Restrictions: Must start server automatically, shut down cleanly after tests, reuse server in local dev but not CI | _Leverage: Playwright webServer configuration, Vite dev server | _Requirements: Requirement 6 (Update E2E Test Configuration) - Auto-start test server | Success: Playwright starts server before tests, tests run against localhost:5173, server shuts down after | Instructions: 1. Mark [-]. 2. Update playwright.config.ts. 3. Log (taskId="5.1.1"). 4. Mark [x]._

- [ ] 5.1.2. Configure Playwright test sharding
  - Files: `keyrx_ui/playwright.config.ts`
  - Configure projects for different browsers (chromium, firefox)
  - Enable sharding for parallel execution in CI
  - Set retries for flaky test handling (1 retry in CI, 0 locally)
  - Configure trace and screenshot capture on failure
  - _Leverage: Playwright projects and sharding configuration_
  - _Requirements: Requirement 6 (Update E2E Test Configuration)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Playwright Performance Engineer | Task: Configure Playwright test sharding and browser projects in keyrx_ui/playwright.config.ts for parallel E2E execution | Restrictions: Configure chromium and firefox projects, enable sharding for CI, set retries (1 in CI, 0 local), capture traces/screenshots on failure only | _Leverage: Playwright projects, sharding, and trace configuration | _Requirements: Requirement 6 (Update E2E Test Configuration) - Parallel E2E execution | Success: E2E tests run in parallel across browsers, CI shards tests for speed, failures captured with traces | Instructions: 1. Mark [-]. 2. Configure sharding. 3. Log (taskId="5.1.2"). 4. Mark [x]._

- [ ] 5.1.3. Verify E2E tests don't run with unit tests
  - Files: `keyrx_ui/vitest.unit.config.ts`, `keyrx_ui/vitest.integration.config.ts`
  - Verify e2e/** is excluded from Vitest configs
  - Add tests/e2e/** and tests/performance/** to exclude patterns
  - Ensure npm test (vitest.unit.config.ts) never runs E2E tests
  - _Leverage: Vitest exclude configuration_
  - _Requirements: Requirement 2 (Separate Test Categories with Vitest Config)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Configuration Auditor | Task: Verify E2E and performance tests are excluded from Vitest unit and integration configs | Restrictions: Must exclude e2e/**, tests/e2e/**, tests/performance/** from all Vitest configs, verify npm test doesn't load Playwright | _Leverage: Vitest exclude configuration patterns | _Requirements: Requirement 2 (Separate Test Categories) - Prevent E2E tests in unit test runs | Success: npm test never tries to run E2E tests, test:e2e is separate script, no Playwright errors in unit tests | Instructions: 1. Mark [-]. 2. Verify exclusions. 3. Log (taskId="5.1.3"). 4. Mark [x]._

## Phase 6: Documentation and Cleanup

### 6.1. Create Testing Documentation

- [ ] 6.1.1. Create unit testing guide
  - Files: `keyrx_ui/docs/testing/unit-testing-guide.md`
  - Document how to write unit tests with MSW WebSocket mocking
  - Provide examples of using renderWithProviders
  - Show WebSocket helper usage (setDaemonState, sendLatencyUpdate, etc.)
  - Explain when to use unit vs integration tests
  - _Leverage: MSW WebSocket examples from migrated tests_
  - _Requirements: Non-functional requirement (Maintainability - Documentation)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Documentation Writer | Task: Create comprehensive unit testing guide in keyrx_ui/docs/testing/unit-testing-guide.md with MSW WebSocket examples | Restrictions: Include real code examples from tests, explain renderWithProviders usage, document WebSocket helpers, provide decision tree for unit vs integration tests | _Leverage: Migrated tests from Phase 3 and 4 for real examples | _Requirements: Maintainability requirement - Documentation for all public APIs | Success: Developers can write tests following guide, examples are copy-paste ready, decision tree helps choose test type | Instructions: 1. Mark [-]. 2. Create guide. 3. Log (taskId="6.1.1"). 4. Mark [x]._

- [ ] 6.1.2. Create integration testing guide
  - Files: `keyrx_ui/docs/testing/integration-testing-guide.md`
  - Document integration test patterns with MSW
  - Show full page testing with renderWithProviders({ wrapWithRouter: true })
  - Explain WebSocket state management in integration tests
  - Provide examples of testing multi-component interactions
  - _Leverage: Integration tests from Phase 4.2_
  - _Requirements: Non-functional requirement (Maintainability - Documentation)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Documentation Writer | Task: Create integration testing guide in keyrx_ui/docs/testing/integration-testing-guide.md covering full page testing and WebSocket state | Restrictions: Include examples from ConfigPage integration test, explain router setup, document async patterns with waitFor(), provide multi-component interaction examples | _Leverage: ConfigPage.integration.test.tsx and other Phase 4.2 tests | _Requirements: Maintainability requirement - Integration test documentation | Success: Guide covers integration testing comprehensively, examples are realistic, developers understand when to write integration tests | Instructions: 1. Mark [-]. 2. Create guide. 3. Log (taskId="6.1.2"). 4. Mark [x]._

- [ ] 6.1.3. Update root README with test commands
  - Files: `keyrx_ui/README.md`
  - Add "Testing" section explaining test categories
  - Document npm test scripts (test, test:unit, test:integration, test:e2e, test:all)
  - Explain when to run which test category
  - Provide quick start examples
  - _Leverage: Updated package.json scripts from task 2.2.1_
  - _Requirements: Non-functional requirement (Developer Experience)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer Experience Engineer | Task: Update keyrx_ui/README.md with comprehensive testing section explaining test categories and commands | Restrictions: Keep concise but comprehensive, provide clear examples, explain test categories (unit/integration/e2e), link to detailed guides | _Leverage: package.json scripts from task 2.2.1 | _Requirements: Developer Experience - Clear error messages and documentation | Success: Developers know which test command to run, README explains test categories clearly, quick start is provided | Instructions: 1. Mark [-]. 2. Update README. 3. Log (taskId="6.1.3"). 4. Mark [x]._

### 6.2. Remove Deprecated Code

- [ ] 6.2.1. Remove jest-websocket-mock dependency
  - Files: `keyrx_ui/package.json`, `keyrx_ui/tests/helpers/websocket.ts`
  - Remove `jest-websocket-mock` from devDependencies in package.json
  - Delete `tests/helpers/websocket.ts` (deprecated in favor of MSW WebSocket)
  - Update any remaining imports of deprecated helpers
  - Run `npm install` to update lockfile
  - _Leverage: Completed migration from Phase 3 and 4_
  - _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dependency Cleanup Specialist | Task: Remove jest-websocket-mock dependency and deprecated WebSocket helper file after successful migration to MSW | Restrictions: ONLY remove after ALL tests migrated (Phase 3 and 4 complete), verify no imports remain, update lockfile with npm install | _Leverage: Grep to find remaining imports, npm install to update lockfile | _Requirements: Requirement 1 (Refactor WebSocket Mocking with MSW) - Single mocking library | Success: jest-websocket-mock removed from package.json, deprecated helpers deleted, no broken imports, lockfile updated | Instructions: 1. Mark [-]. 2. Remove dependency and file. 3. Log (taskId="6.2.1", filesModified=[package.json, package-lock.json], filesCreated=[], statistics with linesRemoved). 4. Mark [x]._

### 6.3. Verify Test Infrastructure

- [ ] 6.3.1. Run full test suite and verify pass rate
  - Files: N/A (verification task)
  - Run `npm run test:unit` and verify >99% pass rate
  - Run `npm run test:integration` and verify >95% pass rate
  - Run `npm run test:e2e` and verify >90% pass rate
  - Run `npm run test:coverage` and verify 80% thresholds met
  - Verify test execution times meet targets (<5s unit, <30s integration, <3min E2E)
  - _Leverage: All previous tasks completed_
  - _Requirements: All requirements (Success Metrics)_
  - _Prompt: Implement the task for spec frontend-test-infrastructure, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Test Infrastructure Validator | Task: Execute full test suite across all categories and verify pass rates and execution times meet requirements | Restrictions: Must run all test categories, verify pass rates (unit >99%, integration >95%, E2E >90%), check execution times (<5s/<30s/<3min), verify coverage >80% | _Leverage: All test infrastructure from previous phases | _Requirements: Success metrics from requirements.md - Test pass rates and execution times | Success: All test categories pass at required rates, execution times within targets, coverage meets 80% threshold | Instructions: 1. Mark [-]. 2. Run test suite. 3. Record results in implementation log (taskId="6.3.1", summary with pass rates and times). 4. Mark [x]._

---

**Created**: 2026-01-04
**Spec Name**: frontend-test-infrastructure
**Status**: Tasks Created
**Total Tasks**: 30 tasks across 6 phases
**Estimated Effort**: 3 weeks
**Priority Order**: Phase 1 (CRITICAL) → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6

## Implementation Notes

1. **Blocking Dependencies:**
   - Phase 3 depends on Phase 1 complete
   - Phase 4 depends on Phase 1 complete
   - Phase 6.2 depends on Phase 3 and 4 complete
   - Phase 6.3 depends on all phases complete

2. **Can be done in parallel:**
   - Phase 1 and Phase 2 are independent
   - Phase 3 and Phase 4 can run in parallel after Phase 1
   - Phase 5 is independent of other phases

3. **Critical Path:**
   - Phase 1 (MSW WebSocket) is CRITICAL - blocks 70% of failures
   - Phase 3 (WebSocket unit tests) - fixes 10/17 failing tests
   - Phase 4 (Component tests) - fixes ~15 failing component tests

4. **Success Criteria:**
   - Unit tests: >99% pass rate (currently 88%)
   - Integration tests: >95% pass rate (currently 50%)
   - E2E tests: >90% pass rate (currently 0% due to config)
   - Unit test speed: <5 seconds
   - Integration test speed: <30 seconds
   - E2E test speed: <3 minutes
