# Design: REST API Comprehensive E2E Testing

## 1. Overview

Pure REST API testing system that validates ALL daemon features via JSON-based communication. No browser/JavaScript required - tests exercise features through HTTP endpoints only.

## 2. Architecture

### 2.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Orchestrator                         │
│  - CLI entry point (scripts/automated-e2e-test.ts)          │
│  - Start/stop daemon lifecycle                               │
│  - Execute test suite sequentially                           │
│  - Generate reports (console, JSON, HTML)                    │
└─────────────┬───────────────────────────────────────────────┘
              │
              ├─► DaemonFixture (daemon lifecycle management)
              ├─► ApiClient (typed REST calls with retry)
              ├─► TestSuite (60+ test cases organized by feature)
              ├─► WebSocketClient (real-time event testing)
              └─► Reporter (progress, results, artifacts)
```

### 2.2 Component Diagram

```
┌────────────────────────────────────────────────────────────┐
│                     Test Runner Flow                        │
│                                                             │
│  1. Parse CLI args (--daemon-path, --fix, --verbose)      │
│  2. Install dependencies (check zod, axios, ws)            │
│  3. Start daemon (DaemonFixture)                           │
│  4. Wait for health (poll GET /api/health until 200)       │
│  5. Execute tests sequentially (avoid race conditions)     │
│  6. Collect results (pass/fail, timing, diffs)             │
│  7. Generate reports (console, JSON, HTML)                 │
│  8. Stop daemon gracefully                                  │
│  9. Exit with code (0=success, 1=failure)                  │
└────────────────────────────────────────────────────────────┘

┌─────────────────┐
│  DaemonFixture  │
├─────────────────┤
│ + start()       │ ─► spawn keyrx_daemon process
│ + stop()        │ ─► SIGTERM → wait 5s → SIGKILL
│ + isHealthy()   │ ─► poll GET /api/health
│ + getLogs()     │ ─► capture stdout/stderr
└─────────────────┘

┌─────────────────┐
│   ApiClient     │
├─────────────────┤
│ + get(path)     │ ─► HTTP GET with retry
│ + post(path)    │ ─► HTTP POST with validation
│ + put(path)     │ ─► HTTP PUT with validation
│ + patch(path)   │ ─► HTTP PATCH with validation
│ + delete(path)  │ ─► HTTP DELETE with retry
└─────────────────┘

┌─────────────────┐
│  TestCase       │
├─────────────────┤
│ id: string      │ ─► Unique test ID
│ name: string    │ ─► Human-readable name
│ category: str   │ ─► Feature category
│ setup()         │ ─► Pre-test preparation
│ execute()       │ ─► API call
│ assert()        │ ─► Validation
│ cleanup()       │ ─► Resource cleanup
└─────────────────┘

┌─────────────────┐
│ WebSocketClient │
├─────────────────┤
│ + connect()     │ ─► ws://localhost:9867/ws
│ + subscribe()   │ ─► Subscribe to channel
│ + waitForEvent()│ ─► Wait for specific event
│ + disconnect()  │ ─► Close connection
└─────────────────┘
```

### 2.3 Test Organization

```
scripts/
├── automated-e2e-test.ts              # Main CLI entry point
├── fixtures/
│   ├── daemon-fixture.ts              # Daemon lifecycle
│   └── expected-results.json          # Expected API responses
├── api-client/
│   ├── client.ts                      # REST API client
│   └── websocket-client.ts            # WebSocket client
├── test-cases/
│   ├── types.ts                       # Test interfaces
│   ├── health-metrics.tests.ts        # Health & metrics (7 tests)
│   ├── device-management.tests.ts     # Devices (10 tests)
│   ├── profile-management.tests.ts    # Profiles (15 tests)
│   ├── config-layers.tests.ts         # Config & layers (8 tests)
│   ├── layouts.tests.ts               # Layouts (3 tests)
│   ├── macros.tests.ts                # Macro recorder (6 tests)
│   ├── simulator.tests.ts             # Simulator (5 tests)
│   ├── websocket.tests.ts             # WebSocket (5 tests)
│   └── workflows.tests.ts             # Integration workflows (6 tests)
├── executor/
│   └── test-executor.ts               # Test suite execution
├── comparator/
│   └── response-comparator.ts         # Deep diff comparison
└── reporters/
    ├── console-reporter.ts            # Console output
    ├── json-reporter.ts               # JSON artifact
    └── html-reporter.ts               # Visual HTML report
```

## 3. Test Suite Structure

### 3.1 Test Case Template

```typescript
interface TestCase {
  id: string;                // Unique ID (e.g., "health-001")
  name: string;              // Description (e.g., "GET /api/health - healthy")
  category: string;          // Feature category
  endpoint: string;          // API endpoint
  priority: 1 | 2 | 3;       // 1=critical, 2=important, 3=nice-to-have

  setup(client: ApiClient): Promise<void>;
  execute(client: ApiClient): Promise<Response>;
  assert(response: Response): AssertionResult;
  cleanup(client: ApiClient): Promise<void>;
}
```

### 3.2 Test Categories

| Category | Tests | Endpoints | Examples |
|----------|-------|-----------|----------|
| Health & Metrics | 7 | 7 | health, version, status, latency, events, daemon-state |
| Device Management | 10 | 7 | list, update, rename, layout, disable, forget |
| Profile Management | 15 | 10 | CRUD, activate, duplicate, rename, validate |
| Config & Layers | 8 | 5 | get config, update config, key mappings, layers |
| Layouts | 3 | 2 | list layouts, get layout |
| Macro Recorder | 6 | 4 | start, stop, get events, clear |
| Simulator | 5 | 2 | simulate events, reset |
| WebSocket | 5 | 1 | connect, subscribe, events, reconnect, disconnect |
| Workflows | 6 | Multiple | Profile lifecycle, device workflow, macro recording |
| **Total** | **65** | **40+** | - |

### 3.3 Test Scenarios Per Endpoint

Each endpoint has multiple scenarios:

**Example: POST /api/profiles**
1. ✅ Success - Create new profile with valid name
2. ✅ Conflict - Create profile with existing name (409)
3. ✅ Invalid - Create profile with empty name (400)
4. ✅ Invalid - Create profile with special chars (400)

**Example: PATCH /api/devices/:id**
1. ✅ Success - Enable/disable device
2. ✅ Not Found - Update non-existent device (404)
3. ✅ Invalid - Update with invalid payload (400)

## 4. Detailed Component Design

### 4.1 DaemonFixture

```typescript
class DaemonFixture {
  private process: ChildProcess | null = null;
  private logs: string[] = [];

  constructor(
    private binaryPath: string,
    private port: number = 9867
  ) {}

  async start(config?: string): Promise<void> {
    // 1. Check if port is available
    // 2. Spawn daemon process with config
    // 3. Capture stdout/stderr
    // 4. Wait for process to start
  }

  async waitUntilHealthy(timeoutMs: number = 30000): Promise<void> {
    // Poll GET /api/health every 100ms until:
    // - Response is 200 OK
    // - Timeout is reached (throw error)
  }

  async stop(): Promise<void> {
    // 1. Send SIGTERM
    // 2. Wait 5 seconds
    // 3. If still running, send SIGKILL (Linux) or taskkill (Windows)
    // 4. Cleanup resources
  }

  getLogs(): string[] {
    return this.logs;
  }

  isRunning(): boolean {
    return this.process !== null && !this.process.killed;
  }
}
```

### 4.2 ApiClient

```typescript
class ApiClient {
  constructor(
    private baseUrl: string,
    private timeout: number = 5000,
    private maxRetries: number = 3
  ) {}

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    schema?: z.ZodSchema<T>
  ): Promise<ApiResponse<T>> {
    // 1. Build full URL
    // 2. Set timeout
    // 3. Make request with fetch/axios
    // 4. Retry on network errors (exponential backoff)
    // 5. Validate response with Zod schema
    // 6. Return typed response
  }

  // Convenience methods for each endpoint
  async getHealth(): Promise<HealthResponse> { ... }
  async getVersion(): Promise<VersionResponse> { ... }
  async getStatus(): Promise<StatusResponse> { ... }
  async getDevices(): Promise<DeviceResponse[]> { ... }
  async patchDevice(id: string, updates: DeviceUpdate): Promise<void> { ... }
  async getProfiles(): Promise<ProfileResponse[]> { ... }
  async createProfile(name: string): Promise<void> { ... }
  // ... all 40+ endpoints
}
```

### 4.3 WebSocketClient

```typescript
class WebSocketClient {
  private ws: WebSocket | null = null;
  private eventQueue: Map<string, Event[]> = new Map();

  async connect(url: string): Promise<void> {
    // 1. Create WebSocket connection
    // 2. Set up event handlers
    // 3. Wait for 'open' event
  }

  async subscribe(channel: string): Promise<void> {
    // Send subscription message:
    // { "type": "subscribe", "channel": "devices" }
  }

  async waitForEvent(
    channel: string,
    predicate: (event: Event) => boolean,
    timeoutMs: number = 5000
  ): Promise<Event> {
    // 1. Wait for event matching predicate
    // 2. Timeout if not received
    // 3. Return matched event
  }

  async disconnect(): Promise<void> {
    // Close WebSocket connection
  }
}
```

### 4.4 TestExecutor

```typescript
class TestExecutor {
  async runAll(
    client: ApiClient,
    cases: TestCase[]
  ): Promise<TestSuiteResult> {
    const results: TestResult[] = [];
    const startTime = performance.now();

    for (const testCase of cases) {
      const result = await this.runSingle(client, testCase);
      results.push(result);

      // Log progress
      console.log(result.status === 'pass' ? '✓' : '✗', testCase.name);
    }

    const duration = performance.now() - startTime;

    return {
      total: results.length,
      passed: results.filter(r => r.status === 'pass').length,
      failed: results.filter(r => r.status === 'fail').length,
      duration,
      results
    };
  }

  async runSingle(
    client: ApiClient,
    testCase: TestCase
  ): Promise<TestResult> {
    const startTime = performance.now();

    try {
      // 1. Setup
      await testCase.setup(client);

      // 2. Execute
      const response = await testCase.execute(client);

      // 3. Assert
      const assertResult = testCase.assert(response);

      // 4. Cleanup (always runs)
      await testCase.cleanup(client);

      return {
        id: testCase.id,
        name: testCase.name,
        status: assertResult.passed ? 'pass' : 'fail',
        duration: performance.now() - startTime,
        error: assertResult.error,
        actual: assertResult.actual,
        expected: assertResult.expected,
        diff: assertResult.diff
      };
    } catch (error) {
      // Ensure cleanup runs even on error
      try {
        await testCase.cleanup(client);
      } catch {}

      return {
        id: testCase.id,
        name: testCase.name,
        status: 'fail',
        duration: performance.now() - startTime,
        error: error.message
      };
    }
  }
}
```

## 5. Test Implementation Examples

### 5.1 Simple Endpoint Test

```typescript
// Test: GET /api/health
{
  id: 'health-001',
  name: 'GET /api/health - daemon healthy',
  category: 'health',
  endpoint: '/api/health',
  priority: 1,

  setup: async () => {
    // No setup needed
  },

  execute: async (client) => {
    return await client.getHealth();
  },

  assert: (response) => {
    const expected = { status: 'ok', version: expect.any(String) };
    const passed = response.status === 'ok' && response.version.length > 0;

    return {
      passed,
      actual: response,
      expected,
      error: passed ? undefined : 'Health check failed'
    };
  },

  cleanup: async () => {
    // No cleanup needed
  }
}
```

### 5.2 CRUD Test

```typescript
// Test: POST /api/profiles - Create profile
{
  id: 'profiles-003',
  name: 'POST /api/profiles - create new profile',
  category: 'profiles',
  endpoint: '/api/profiles',
  priority: 1,

  setup: async (client) => {
    // Ensure test profile doesn't exist
    try {
      await client.deleteProfile('test-profile-create');
    } catch {}
  },

  execute: async (client) => {
    return await client.createProfile('test-profile-create', 'empty');
  },

  assert: (response) => {
    const passed = response.profile?.name === 'test-profile-create';

    return {
      passed,
      actual: response,
      expected: { profile: { name: 'test-profile-create' } },
      error: passed ? undefined : 'Profile creation failed'
    };
  },

  cleanup: async (client) => {
    // Delete test profile
    try {
      await client.deleteProfile('test-profile-create');
    } catch {}
  }
}
```

### 5.3 Workflow Test

```typescript
// Test: Profile lifecycle workflow
{
  id: 'workflow-001',
  name: 'Profile lifecycle - Create → Activate → Delete',
  category: 'workflows',
  endpoint: 'multiple',
  priority: 1,

  setup: async (client) => {
    // Clean state
    try {
      await client.deleteProfile('test-lifecycle');
    } catch {}
  },

  execute: async (client) => {
    // Step 1: Create
    await client.createProfile('test-lifecycle', 'empty');

    // Step 2: Activate
    await client.activateProfile('test-lifecycle');

    // Step 3: Verify active
    const active = await client.getActiveProfile();

    // Step 4: Delete
    await client.deleteProfile('test-lifecycle');

    return { active };
  },

  assert: (response) => {
    const passed = response.active === 'test-lifecycle';

    return {
      passed,
      actual: response,
      expected: { active: 'test-lifecycle' },
      error: passed ? undefined : 'Profile lifecycle failed'
    };
  },

  cleanup: async (client) => {
    // Already deleted in execute
  }
}
```

### 5.4 WebSocket Test

```typescript
// Test: WebSocket device events
{
  id: 'websocket-002',
  name: 'WebSocket - Receive device update events',
  category: 'websocket',
  endpoint: '/ws',
  priority: 2,

  setup: async () => {
    // No setup
  },

  execute: async (client) => {
    const wsClient = new WebSocketClient();

    // 1. Connect
    await wsClient.connect('ws://localhost:9867/ws');

    // 2. Subscribe to device events
    await wsClient.subscribe('devices');

    // 3. Trigger device update via REST API
    const devices = await client.getDevices();
    if (devices.length > 0) {
      await client.patchDevice(devices[0].id, { enabled: false });
    }

    // 4. Wait for WebSocket event
    const event = await wsClient.waitForEvent(
      'devices',
      (e) => e.type === 'device_updated',
      5000
    );

    // 5. Disconnect
    await wsClient.disconnect();

    return event;
  },

  assert: (event) => {
    const passed = event.type === 'device_updated';

    return {
      passed,
      actual: event,
      expected: { type: 'device_updated' },
      error: passed ? undefined : 'Device update event not received'
    };
  },

  cleanup: async (client) => {
    // Re-enable device if needed
    const devices = await client.getDevices();
    if (devices.length > 0) {
      try {
        await client.patchDevice(devices[0].id, { enabled: true });
      } catch {}
    }
  }
}
```

## 6. Reporting

### 6.1 Console Output

```
===========================================
KeyRx REST API E2E Tests
===========================================

Starting daemon...
  Daemon started on port 9867
  Waiting for health check... OK (2.1s)

Running 65 tests...

Health & Metrics (7 tests)
  ✓ GET /api/health - daemon healthy (45ms)
  ✓ GET /api/version - get version info (32ms)
  ✓ GET /api/status - daemon running (28ms)
  ✓ GET /api/metrics/latency - latency stats (41ms)
  ✓ GET /api/metrics/events - event log (37ms)
  ✓ DELETE /api/metrics/events - clear log (29ms)
  ✓ GET /api/daemon/state - daemon state (51ms)

Device Management (10 tests)
  ✓ GET /api/devices - list devices (34ms)
  ✓ PATCH /api/devices/:id - update config (58ms)
  ✓ PUT /api/devices/:id/name - rename device (47ms)
  ...

Summary:
  Total:   65 tests
  Passed:  65 tests (100%)
  Failed:  0 tests
  Duration: 2m 34s

All tests passed! ✅
```

### 6.2 JSON Report

```json
{
  "version": "1.0",
  "timestamp": "2026-01-21T12:00:00Z",
  "daemon": {
    "binaryPath": "/path/to/keyrx_daemon",
    "port": 9867,
    "startupTime": 2100
  },
  "summary": {
    "total": 65,
    "passed": 65,
    "failed": 0,
    "duration": 154000,
    "passRate": 100.0
  },
  "results": [
    {
      "id": "health-001",
      "name": "GET /api/health - daemon healthy",
      "category": "health",
      "endpoint": "/api/health",
      "status": "pass",
      "duration": 45,
      "error": null
    }
  ],
  "categories": {
    "health": { "total": 7, "passed": 7, "failed": 0 },
    "devices": { "total": 10, "passed": 10, "failed": 0 },
    "profiles": { "total": 15, "passed": 15, "failed": 0 },
    "config": { "total": 8, "passed": 8, "failed": 0 },
    "layouts": { "total": 3, "passed": 3, "failed": 0 },
    "macros": { "total": 6, "passed": 6, "failed": 0 },
    "simulator": { "total": 5, "passed": 5, "failed": 0 },
    "websocket": { "total": 5, "passed": 5, "failed": 0 },
    "workflows": { "total": 6, "passed": 6, "failed": 0 }
  }
}
```

### 6.3 HTML Report

Interactive HTML report with:
- Summary dashboard (pass rate, duration, categories)
- Test list (filterable by category, status)
- Detailed test results (request, response, diff)
- Syntax-highlighted JSON
- Search functionality

## 7. Performance Optimization

### 7.1 Sequential Execution
- Tests run sequentially to avoid race conditions
- Each test is isolated (independent state)
- Cleanup ensures no state leakage

### 7.2 Fast Startup
- Daemon starts once for entire suite
- Health check polls every 100ms
- Timeout after 30s if unhealthy

### 7.3 Efficient API Calls
- HTTP/1.1 keep-alive (connection pooling)
- Timeouts per request (5s default)
- Retry on network errors only (3 attempts max)

### 7.4 Minimal Fixtures
- Use existing daemon state where possible
- Create test data only when needed
- Clean up immediately after use

## 8. Error Handling

### 8.1 Graceful Degradation
- If daemon fails to start: exit with error code 2
- If test fails: continue with remaining tests
- If cleanup fails: log warning, continue

### 8.2 Resource Cleanup
- Always stop daemon on exit (even on SIGINT)
- Always run test cleanup (even on error)
- Always close WebSocket connections

### 8.3 Clear Error Messages
```
✗ POST /api/profiles - create new profile (123ms)

  Expected:
    { "profile": { "name": "test-profile" } }

  Actual:
    { "error": "Profile already exists", "code": "PROFILE_EXISTS" }

  Error:
    Profile creation failed - PROFILE_EXISTS
```

## 9. Future Enhancements

### 9.1 Parallel Execution
- Run independent tests in parallel (worker threads)
- Reduce total execution time from 3min to < 1min

### 9.2 Snapshot Testing
- Auto-generate expected results from actual responses
- Detect API changes automatically

### 9.3 Performance Benchmarking
- Track response times over time
- Alert on slowdowns
- Identify performance regressions

### 9.4 Contract Testing
- Generate tests from OpenAPI spec
- Ensure API contract compliance
- Auto-update tests when spec changes

## 10. Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Runtime | Node.js 18+ | Existing ecosystem, fast |
| Language | TypeScript 5+ | Type safety, IDE support |
| HTTP Client | `axios` | Retry logic, interceptors |
| WebSocket | `ws` | Lightweight, reliable |
| Validation | `zod` | Already used in keyrx_ui |
| CLI Parsing | `commander` | Standard, feature-rich |
| Comparison | `deep-diff` | Detailed diffs |
| Colors | `chalk` | Readable console output |
| Execution | `tsx` | Fast TS execution |

## 11. File Size Budget

| File | Max Lines | Actual (Estimated) |
|------|-----------|---------------------|
| automated-e2e-test.ts | 500 | 350 |
| daemon-fixture.ts | 500 | 200 |
| api-client.ts | 500 | 450 |
| websocket-client.ts | 500 | 250 |
| test-executor.ts | 500 | 300 |
| *.tests.ts (each) | 500 | 200-400 |
| reporters/*.ts (each) | 500 | 200-300 |

All files within 500-line limit ✅
