# Design: E2E Playwright Testing

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    E2E Test Architecture                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │ Playwright       │     │  Test Fixtures   │                  │
│  │ Test Runner      │────▶│  & Helpers       │                  │
│  └────────┬─────────┘     └──────────────────┘                  │
│           │                                                      │
│           │ Browser automation                                   │
│           ▼                                                      │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │  keyrx_ui        │────▶│  keyrx_daemon    │                  │
│  │  (localhost:5173)│     │  (localhost:9867)│                  │
│  └──────────────────┘     └──────────────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Test Structure

```
keyrx_ui/
├── tests/
│   └── e2e/
│       ├── fixtures/
│       │   ├── daemon.ts          # Daemon lifecycle management
│       │   ├── api.ts             # API request helpers
│       │   └── network-monitor.ts # Request tracking
│       ├── pages/
│       │   ├── home.spec.ts       # HomePage tests
│       │   ├── devices.spec.ts    # DevicesPage tests
│       │   ├── profiles.spec.ts   # ProfilesPage tests
│       │   ├── config.spec.ts     # ConfigPage tests
│       │   ├── metrics.spec.ts    # MetricsPage tests
│       │   └── simulator.spec.ts  # SimulatorPage tests
│       ├── api/
│       │   ├── status.spec.ts     # Status/health endpoints
│       │   ├── devices.spec.ts    # Device API tests
│       │   ├── profiles.spec.ts   # Profile API tests
│       │   ├── config.spec.ts     # Config API tests
│       │   └── metrics.spec.ts    # Metrics API tests
│       ├── flows/
│       │   ├── profile-crud.spec.ts    # Profile CRUD flow
│       │   ├── device-config.spec.ts   # Device config flow
│       │   └── navigation.spec.ts      # Navigation flow
│       └── network/
│           └── request-efficiency.spec.ts # Network efficiency tests
├── playwright.config.ts           # Playwright configuration
└── playwright.e2e.config.ts       # E2E-specific config
```

## Components

### 1. Daemon Fixture
**File:** `tests/e2e/fixtures/daemon.ts`

Manages daemon lifecycle for tests:
```typescript
export const daemonFixture = {
  async startDaemon(): Promise<void>;
  async stopDaemon(): Promise<void>;
  async waitForReady(): Promise<void>;
  async createTestProfile(): Promise<void>;
}
```

### 2. Network Monitor
**File:** `tests/e2e/fixtures/network-monitor.ts`

Tracks API requests to detect issues:
```typescript
export class NetworkMonitor {
  requests: Map<string, number>;  // endpoint → count

  start(page: Page): void;
  stop(): void;
  getRequestCount(endpoint: string): number;
  assertNoExcessiveRequests(max: number): void;
  assertNoDuplicates(endpoint: string): void;
}
```

### 3. API Test Helpers
**File:** `tests/e2e/fixtures/api.ts`

Reusable API request helpers:
```typescript
export const apiHelpers = {
  async getStatus(request: APIRequestContext): Promise<StatusResponse>;
  async getDevices(request: APIRequestContext): Promise<DeviceListResponse>;
  async createProfile(request: APIRequestContext, name: string): Promise<void>;
  async activateProfile(request: APIRequestContext, name: string): Promise<void>;
}
```

## Technical Decisions

### D1: Use Playwright API Testing + Browser Testing
**Decision:** Use both Playwright's API testing and browser automation
**Rationale:** API tests are fast and verify backend; browser tests verify UI integration

### D2: Daemon Runs in Same Process Space
**Decision:** Start daemon as subprocess during test setup
**Rationale:** Isolated test environment, no dependency on external services

### D3: Network Request Monitoring via CDP
**Decision:** Use Chrome DevTools Protocol to monitor network
**Rationale:** Accurate request tracking without modifying application code

### D4: Test Data Isolation
**Decision:** Each test creates/deletes its own test data
**Rationale:** Tests don't interfere with each other, can run in parallel

### D5: Screenshot/Video on Failure Only
**Decision:** Only capture artifacts on test failure
**Rationale:** Reduces CI artifact size while preserving debugging info

## Playwright Configuration

```typescript
// playwright.e2e.config.ts
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,  // Sequential for daemon dependency
  retries: 1,
  workers: 1,
  reporter: [
    ['html', { open: 'never' }],
    ['json', { outputFile: 'e2e-results.json' }],
  ],
  use: {
    baseURL: 'http://localhost:9867',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
  globalSetup: './tests/e2e/global-setup.ts',
  globalTeardown: './tests/e2e/global-teardown.ts',
});
```

## Test Categories

### 1. Page Load Tests (Smoke Tests)
Fast tests that verify each page loads without errors:
- Navigate to page
- Assert no console errors
- Assert no failed requests
- Assert key elements present

### 2. API Endpoint Tests
Direct API testing without browser:
- Test each endpoint with valid/invalid data
- Verify response structure
- Verify status codes

### 3. User Flow Tests
End-to-end user journey tests:
- Multi-step interactions
- State persistence verification
- Cross-page navigation

### 4. Network Efficiency Tests
Verify network request patterns:
- Count requests per page load
- Detect duplicate requests
- Detect rapid-fire requests

## CI Integration

```yaml
e2e-playwright-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Setup Node.js
      uses: actions/setup-node@v4
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
    - name: Build daemon
      run: cargo build --release -p keyrx_daemon
    - name: Install dependencies
      run: cd keyrx_ui && npm ci
    - name: Install Playwright browsers
      run: cd keyrx_ui && npx playwright install --with-deps chromium
    - name: Run E2E tests
      run: cd keyrx_ui && npm run test:e2e:ci
    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: playwright-report
        path: keyrx_ui/playwright-report/
```

## Error Handling

### Console Error Detection
```typescript
page.on('console', msg => {
  if (msg.type() === 'error') {
    consoleErrors.push(msg.text());
  }
});
```

### Network Failure Detection
```typescript
page.on('requestfailed', request => {
  failedRequests.push({
    url: request.url(),
    failure: request.failure()?.errorText,
  });
});
```

### Request Rate Detection
```typescript
const requestCounts = new Map<string, number>();
page.on('request', request => {
  const key = `${request.method()} ${new URL(request.url()).pathname}`;
  requestCounts.set(key, (requestCounts.get(key) || 0) + 1);
});
```

## Test Execution Order

1. **Global Setup**: Start daemon, create test profile
2. **API Tests**: Fast, no browser needed
3. **Page Load Tests**: Quick smoke tests
4. **Flow Tests**: Full user journey tests
5. **Network Tests**: Request efficiency verification
6. **Global Teardown**: Stop daemon, cleanup

## Security Considerations

- Tests run against localhost only
- No real keyboard capture in CI (uinput permissions)
- Test profiles use harmless config (no key remapping)
- Credentials never stored in test code
