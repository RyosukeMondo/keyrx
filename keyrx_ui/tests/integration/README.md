# Integration Tests

This directory contains integration tests that verify the complete interaction between the UI and the daemon.

## Overview

Integration tests verify the full stack functionality including:
- **RPC Communication**: Complete WebSocket RPC flow from React to Rust
- **Config Editor**: Tab switching, validation, and saving workflows
- **Dashboard Updates**: Real-time state updates via WebSocket subscriptions
- **Profile Workflow**: Complete profile CRUD operations

## Prerequisites

The integration tests require a running daemon instance. You have two options:

### Option 1: Manual Daemon Start (Recommended for Development)

Start the daemon manually before running tests:

```bash
# Terminal 1: Start daemon in test mode
cd keyrx_daemon
cargo run -- --port 13030 --headless

# Terminal 2: Run integration tests
cd keyrx_ui
npm run test:integration
```

### Option 2: Auto-Start (Automated Testing)

The test harness can automatically start and stop the daemon. This is useful for CI/CD:

```bash
# Daemon will be started automatically
cd keyrx_ui
npm run test:integration
```

Note: Auto-start requires cargo to be available in PATH and may take longer due to daemon compilation.

## Running Tests

### Run all integration tests

```bash
npm run test:integration
```

### Run integration tests in watch mode

```bash
npm run test:integration:watch
```

### Run a specific test file

```bash
npm test tests/integration/config-editor.test.tsx
npm test tests/integration/dashboard-updates.test.tsx
npm test tests/integration/profile-workflow.test.tsx
npm test tests/integration/rpc-communication.test.ts
```

### Run with debug output

To see daemon output during tests, set the DEBUG_DAEMON environment variable:

```bash
DEBUG_DAEMON=1 npm run test:integration
```

## Test Files

### `test-harness.ts`
Utility functions for starting/stopping the daemon and common test helpers:
- `setupDaemon()` - Start daemon for tests
- `teardownDaemon()` - Stop daemon after tests
- `createTestProfileName()` - Generate unique test profile names
- `isDaemonRunning()` - Check if daemon is reachable
- Constants for daemon URLs and test configurations

### `rpc-communication.test.ts`
End-to-end RPC communication test covering all REQ-1 acceptance criteria:
- WebSocket connection and handshake
- Profile creation, activation, and deletion
- Real-time state updates via subscriptions
- Request timeout handling
- Error handling
- Concurrent request correlation

### `config-editor.test.tsx`
Configuration editor workflow tests (REQ-4):
- Visual tab active by default
- Tab switching between Visual and Code modes
- State persistence across tab switches
- Validation status display
- Save functionality via button and Ctrl+S
- Validation error prevention

### `dashboard-updates.test.tsx`
Dashboard real-time update tests (REQ-3):
- WebSocket connection
- Subscription to all channels (daemon-state, events, latency)
- Real-time component updates
- Pause/resume functionality
- Clear functionality
- FIFO limits enforcement
- Cleanup on unmount

### `profile-workflow.test.tsx`
Profile management workflow tests:
- Create profile
- Activate profile
- Duplicate profile
- Rename profile
- Delete profile
- Update profile configuration
- Error handling for invalid operations
- Complete profile lifecycle

## Configuration

### Test Daemon Port

Integration tests use port `13030` by default. This can be changed in `test-harness.ts`:

```typescript
export const DAEMON_TEST_PORT = 13030;
```

### Daemon Auto-Start

By default, tests check if the daemon is already running and use it. To require manual start:

```typescript
// In test file
beforeAll(async () => {
  await setupDaemon({ autoStart: false }); // Throws if daemon not running
});
```

To always auto-start:

```typescript
beforeAll(async () => {
  await setupDaemon({ autoStart: true }); // Starts daemon automatically
});
```

## Troubleshooting

### Error: "Daemon is not running"

Start the daemon manually:
```bash
cd keyrx_daemon
cargo run -- --port 13030 --headless
```

### Tests timeout

The daemon may take a while to start. Increase timeout in test files:
```typescript
await waitFor(() => {
  expect(result.current.isConnected).toBe(true);
}, { timeout: 30000 }); // 30 second timeout
```

### Port already in use

Kill any existing daemon process:
```bash
lsof -ti:13030 | xargs kill -9
```

Or change the test port in `test-harness.ts`.

### Tests fail on CI

Ensure the CI environment has:
- Rust toolchain installed
- Cargo available in PATH
- Port 13030 available
- Sufficient timeout for daemon startup

## CI Integration

Integration tests should run after unit tests in CI:

```yaml
# .github/workflows/ci.yml
- name: Run unit tests
  run: npm test

- name: Run integration tests
  run: npm run test:integration
```

For faster CI, consider:
1. Building the daemon once and reusing the binary
2. Starting the daemon before tests and keeping it running
3. Running tests in parallel groups

## Best Practices

1. **Cleanup**: Always clean up test data (profiles, configs) in `afterAll`
2. **Unique Names**: Use `createTestProfileName()` for unique test profiles
3. **Timeouts**: Set reasonable timeouts for async operations
4. **Isolation**: Tests should not depend on each other
5. **Manual Start**: Prefer manual daemon start for faster iteration during development

## Related Documentation

- [Daemon RPC API Documentation](../../keyrx_daemon/src/web/README.md)
- [WebSocket RPC Types](../../keyrx_daemon/src/web/rpc_types.rs)
- [Test Requirements](../../.spec-workflow/specs/keyrx-ui-integration/requirements.md)
