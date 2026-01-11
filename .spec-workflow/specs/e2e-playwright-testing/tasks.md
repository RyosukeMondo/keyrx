# Tasks: E2E Playwright Testing

## Implementation Tasks

- [x] 1. Create Playwright E2E configuration
  - File: keyrx_ui/playwright.e2e.config.ts
  - Configure Playwright for E2E testing against live daemon
  - Set up reporter, screenshot/video on failure, trace collection
  - Configure webServer for dev server
  - Purpose: Establish E2E test infrastructure
  - _Leverage: keyrx_ui/playwright.config.ts (existing config)_
  - _Requirements: 5.1, 5.4, 5.5_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer with Playwright expertise | Task: Create Playwright E2E configuration file with proper setup for testing against keyrx_daemon (port 9867), including screenshot/video capture on failure, HTML reporter, and sequential test execution. Reference existing playwright.config.ts for patterns. | Restrictions: Do not modify existing playwright.config.ts, use separate config for E2E, ensure tests run sequentially (workers: 1) due to shared daemon state | Success: Config file created, can be used with `npx playwright test --config=playwright.e2e.config.ts`. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 2. Create daemon fixture for test lifecycle
  - File: keyrx_ui/tests/e2e/fixtures/daemon.ts
  - Implement daemon start/stop/wait helpers
  - Create test profile automatically
  - Purpose: Manage daemon lifecycle in tests
  - _Leverage: scripts/validate-api-contracts.ts (daemon interaction pattern)_
  - _Requirements: 5.2_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Infrastructure Engineer | Task: Create daemon fixture that starts keyrx_daemon as subprocess, waits for it to be ready (poll /api/status), creates a test profile, and provides cleanup. Export as Playwright fixture. | Restrictions: Handle daemon startup failures gracefully, ensure cleanup runs even on test failure, use configurable paths for daemon binary | Success: Fixture can start/stop daemon, tests can depend on daemon being ready. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 3. Create network monitor fixture
  - File: keyrx_ui/tests/e2e/fixtures/network-monitor.ts
  - Track all API requests during test
  - Detect duplicate/rapid requests
  - Purpose: Catch network efficiency issues
  - _Leverage: Playwright page.on('request') API_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Test Engineer | Task: Create NetworkMonitor class that attaches to Playwright page, tracks all API requests by endpoint, counts duplicates, and provides assertions like assertNoExcessiveRequests(endpoint, maxCount) and assertNoDuplicateRequests(). | Restrictions: Only track requests to /api/* endpoints, ignore static assets, use Map for efficient counting | Success: Can detect rapid PATCH requests bug pattern, provides clear assertion messages. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 4. Create API test helpers
  - File: keyrx_ui/tests/e2e/fixtures/api.ts
  - Reusable API request functions
  - Type-safe response handling
  - Purpose: DRY API testing code
  - _Leverage: keyrx_ui/src/api/schemas.ts (response types)_
  - _Requirements: 2.1-2.11_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: API Test Developer | Task: Create typed API helper functions using Playwright's APIRequestContext for all endpoints: getStatus(), getDevices(), getProfiles(), createProfile(), activateProfile(), etc. Use Zod schemas from src/api/schemas.ts for response validation. | Restrictions: Validate all responses against schemas, throw clear errors on validation failure, use consistent error handling | Success: All API endpoints have helper functions, responses are type-checked. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 5. Create global setup/teardown
  - File: keyrx_ui/tests/e2e/global-setup.ts, keyrx_ui/tests/e2e/global-teardown.ts
  - Start daemon before all tests
  - Stop daemon after all tests
  - Purpose: Shared daemon for all E2E tests
  - _Leverage: keyrx_ui/tests/e2e/fixtures/daemon.ts_
  - _Requirements: 5.2_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Infrastructure Engineer | Task: Create global-setup.ts that starts daemon and waits for ready, global-teardown.ts that stops daemon. Store daemon PID in file for teardown access. | Restrictions: Must handle CI environment (daemon binary in different location), timeout after 30s if daemon doesn't start, cleanup on any failure | Success: Daemon starts before tests, stops after tests, CI works correctly. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 6. Create HomePage E2E tests
  - File: keyrx_ui/tests/e2e/pages/home.spec.ts
  - Test page load, content, no errors
  - Test dashboard cards render
  - Purpose: Verify HomePage works end-to-end
  - _Leverage: keyrx_ui/src/pages/HomePage.tsx_
  - _Requirements: 1.1, 1.7, 1.8_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for HomePage: test page loads without console errors, test active profile card renders, test device list card renders, test quick stats renders. Use network monitor to verify no excessive requests. | Restrictions: Use data-testid attributes where available, wait for content to load before assertions, capture console errors | Success: Tests pass when HomePage works correctly, fail when issues exist. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 7. Create DevicesPage E2E tests
  - File: keyrx_ui/tests/e2e/pages/devices.spec.ts
  - Test device list loads
  - Test layout selector works
  - Test no rapid PATCH requests
  - Purpose: Verify DevicesPage works and no network bugs
  - _Leverage: keyrx_ui/src/pages/DevicesPage.tsx_
  - _Requirements: 1.2, 1.7, 1.8, 4.1, 4.2_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for DevicesPage: test page loads with device cards, test layout dropdown works, test changing layout only sends ONE PATCH request (critical - catches rapid request bug), test rename device flow. | Restrictions: Must verify no duplicate PATCH requests on page load, use network monitor fixture, wait for API responses | Success: Tests catch the rapid PATCH request bug pattern, verify normal operation works. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 8. Create ProfilesPage E2E tests
  - File: keyrx_ui/tests/e2e/pages/profiles.spec.ts
  - Test profile list loads
  - Test create profile flow
  - Test activate profile flow
  - Purpose: Verify profile management works
  - _Leverage: keyrx_ui/src/pages/ProfilesPage.tsx_
  - _Requirements: 1.3, 1.7, 1.8, 3.1, 3.2_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for ProfilesPage: test profile list loads, test create new profile (modal → name → create → appears in list), test activate profile (click activate → profile becomes active), test delete profile. | Restrictions: Clean up test profiles after tests, use unique names for test profiles, verify UI updates after API calls | Success: Full profile CRUD flow tested end-to-end. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 9. Create ConfigPage E2E tests
  - File: keyrx_ui/tests/e2e/pages/config.spec.ts
  - Test Monaco editor loads
  - Test save config flow
  - Test validation errors
  - Purpose: Verify config editing works
  - _Leverage: keyrx_ui/src/pages/ConfigPage.tsx_
  - _Requirements: 1.4, 1.7, 1.8, 3.3_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for ConfigPage: test Monaco editor renders, test loading existing config, test editing and saving config (edit → save → reload → verify saved), test validation error display. Monaco requires special handling - wait for editor to initialize. | Restrictions: Monaco editor needs time to load, use appropriate waits, don't test editor internals (just that it works) | Success: Config editing flow works end-to-end, save persists correctly. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 10. Create MetricsPage E2E tests
  - File: keyrx_ui/tests/e2e/pages/metrics.spec.ts
  - Test metrics dashboard loads
  - Test latency stats display
  - Test event log display
  - Purpose: Verify metrics display works
  - _Leverage: keyrx_ui/src/pages/MetricsPage.tsx_
  - _Requirements: 1.5, 1.7, 1.8_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for MetricsPage: test page loads without errors, test latency stats card renders (may show "no data" initially), test event log renders, test clear event log button. | Restrictions: Metrics may be empty if no events, handle empty state gracefully, verify structure not exact values | Success: MetricsPage displays correctly with or without data. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 11. Create SimulatorPage E2E tests
  - File: keyrx_ui/tests/e2e/pages/simulator.spec.ts
  - Test simulator interface loads
  - Test key simulation works
  - Purpose: Verify simulator works
  - _Leverage: keyrx_ui/src/pages/SimulatorPage.tsx_
  - _Requirements: 1.6, 1.7, 1.8_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create Playwright tests for SimulatorPage: test page loads without errors, test key input field works, test simulate button triggers API call, test output displays results. | Restrictions: Simulator results depend on config, test structure not specific outputs, handle API errors gracefully | Success: Simulator UI works, API integration functions. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 12. Create API endpoint tests
  - File: keyrx_ui/tests/e2e/api/endpoints.spec.ts
  - Test all documented API endpoints
  - Verify response status and structure
  - Purpose: Comprehensive API contract testing via Playwright
  - _Leverage: keyrx_ui/tests/e2e/fixtures/api.ts, keyrx_ui/src/api/schemas.ts_
  - _Requirements: 2.1-2.11_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: API Test Developer | Task: Create comprehensive API tests using Playwright's request fixture: test GET /api/status, GET /api/devices, GET /api/profiles, POST /api/profiles, DELETE /api/profiles/:name, PATCH /api/devices/:id, GET /api/metrics/latency, GET /api/layouts. Validate responses against Zod schemas. | Restrictions: Use API helpers from fixtures/api.ts, clean up created test data, test both success and error cases | Success: All API endpoints tested, contracts validated, errors produce clear failures. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 13. Create navigation flow tests
  - File: keyrx_ui/tests/e2e/flows/navigation.spec.ts
  - Test all navigation links work
  - Test sidebar navigation
  - Test bottom nav (mobile)
  - Purpose: Verify app navigation works
  - _Leverage: keyrx_ui/src/components/Sidebar.tsx, keyrx_ui/src/components/BottomNav.tsx_
  - _Requirements: 3.5_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Developer | Task: Create navigation tests: test clicking each sidebar link navigates to correct page, test browser back/forward works, test deep links work (e.g., /profiles/Default/config). Test mobile viewport for bottom nav. | Restrictions: Use viewport switching for mobile tests, verify URL changes, verify page content loads | Success: All navigation paths work correctly. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 14. Create network efficiency tests
  - File: keyrx_ui/tests/e2e/network/request-efficiency.spec.ts
  - Test no excessive requests per page
  - Test no duplicate requests
  - Test user actions don't spam API
  - Purpose: Catch performance/efficiency bugs
  - _Leverage: keyrx_ui/tests/e2e/fixtures/network-monitor.ts_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Test Engineer | Task: Create network efficiency tests: for each page, verify number of API requests on load is within expected range, verify no duplicate requests to same endpoint within 100ms, verify changing dropdown doesn't cause >2 requests. Use NetworkMonitor fixture. | Restrictions: Define expected request counts per page, allow some tolerance for race conditions, fail clearly on excessive requests | Success: Catches rapid PATCH bug pattern, provides clear failure messages on network issues. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 15. Add npm script for E2E tests
  - File: keyrx_ui/package.json
  - Add test:e2e:full script for all E2E tests
  - Add test:e2e:ci script for CI environment
  - Purpose: Easy test execution
  - _Leverage: keyrx_ui/package.json (existing scripts)_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Add npm scripts to package.json: "test:e2e:full": "playwright test --config=playwright.e2e.config.ts", "test:e2e:ci": "playwright test --config=playwright.e2e.config.ts --reporter=html,json". | Restrictions: Follow existing script naming conventions, ensure CI script outputs artifacts | Success: npm run test:e2e:full and test:e2e:ci work correctly. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [ ] 16. Add CI workflow job for E2E tests
  - File: .github/workflows/ci.yml
  - Add e2e-playwright-tests job
  - Install Playwright browsers
  - Upload test artifacts
  - Purpose: Automated E2E testing in CI
  - _Leverage: .github/workflows/ci.yml (existing jobs)_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec e2e-playwright-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps/CI Engineer | Task: Add new job 'e2e-playwright-tests' to ci.yml: depends on build-and-verify, builds daemon, installs Playwright browsers (chromium only), runs test:e2e:ci, uploads playwright-report as artifact on success/failure. | Restrictions: Use ubuntu-latest, install only chromium to save time, ensure daemon starts before tests, upload artifacts always | Success: CI runs E2E tests on every PR, artifacts available for debugging. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

## Task Dependencies

```
1 (config) ─┬─► 2 (daemon fixture)
            │
            ├─► 3 (network monitor)
            │
            └─► 4 (API helpers)
                    │
                    ▼
            5 (global setup/teardown)
                    │
                    ▼
    ┌───────────────┼───────────────┐
    │               │               │
    ▼               ▼               ▼
6-11 (page tests)  12 (API tests)  13-14 (flow/network tests)
                    │
                    ▼
                15 (npm scripts)
                    │
                    ▼
                16 (CI integration)
```

## Notes

- Tasks 1-5 are infrastructure, must complete first
- Tasks 6-14 are tests, can be done in parallel after infrastructure
- Tasks 15-16 are integration, complete last
- All tests should use fixtures from tasks 2-4
- Network efficiency tests (task 14) specifically catch the rapid PATCH bug pattern
