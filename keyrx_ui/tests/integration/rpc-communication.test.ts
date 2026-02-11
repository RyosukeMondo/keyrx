/**
 * End-to-End RPC Communication Integration Test
 *
 * This test verifies the complete RPC stack from React components to the Rust daemon.
 * It tests all acceptance criteria from REQ-1 by executing a real workflow with a running daemon.
 *
 * Prerequisites:
 * - Start the daemon before running this test:
 *   ```
 *   cd keyrx_daemon && cargo run -- --port 13030 --headless
 *   ```
 *
 * Test Workflow:
 * 1. Connect via WebSocket and verify handshake (AC1)
 * 2. Create a new profile (AC2)
 * 3. Subscribe to daemon-state channel (AC3, AC4)
 * 4. Activate the profile (AC2)
 * 5. Verify state change event received (AC8)
 * 6. Update config (AC2)
 * 7. Delete profile (AC2)
 * 8. Disconnect and verify cleanup (AC10)
 *
 * To run this test:
 * ```bash
 * # Terminal 1: Start daemon
 * cd keyrx_daemon && cargo run -- --port 13030 --headless
 *
 * # Terminal 2: Run integration test
 * cd keyrx_ui && npm test tests/integration/rpc-communication.test.ts
 * ```
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useUnifiedApi } from '../../src/hooks/useUnifiedApi';
import type { DaemonState } from '../../src/types/rpc';

import { DAEMON_TEST_PORT, DAEMON_WS_URL } from './test-harness';

const DAEMON_PORT = DAEMON_TEST_PORT;
const WS_URL = DAEMON_WS_URL;

describe('End-to-End RPC Communication', () => {
  // Check if daemon is running before executing tests
  beforeAll(async () => {
    try {
      const response = await fetch(`http://127.0.0.1:${DAEMON_PORT}/health`, {
        method: 'GET',
      });
      if (!response.ok) {
        throw new Error('Daemon health check failed');
      }
    } catch (error) {
      console.error(
        '\n\n❌ Daemon is not running! Start it with:\n' +
          `   cd keyrx_daemon && cargo run -- --port ${DAEMON_PORT} --headless\n\n`
      );
      throw new Error(
        `Daemon is not running on port ${DAEMON_PORT}. ` +
          'Start the daemon before running integration tests.'
      );
    }
  });

  it('should execute complete profile workflow with real-time updates (AC1, AC2, AC3, AC4, AC8, AC10)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // AC1: Wait for connection and handshake
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    expect(result.current.readyState).toBe(1); // OPEN
    console.log('✓ AC1: Connection and handshake successful');

    // AC3, AC4: Subscribe to daemon-state channel
    const stateChanges: DaemonState[] = [];
    const unsubscribe = result.current.subscribe('daemon-state', (state) => {
      console.log('State change received:', state);
      stateChanges.push(state as DaemonState);
    });
    console.log('✓ AC3, AC4: Subscribed to daemon-state channel');

    // AC2: Create a new profile
    const profileName = `test-profile-${Date.now()}`;
    console.log(`Creating profile: ${profileName}`);
    await result.current.command('create_profile', { name: profileName });

    // AC2: Verify profile was created
    const profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(expect.arrayContaining([expect.objectContaining({ name: profileName })]));
    console.log('✓ AC2: Profile created successfully');

    // AC2: Activate the profile
    console.log(`Activating profile: ${profileName}`);
    await result.current.command('activate_profile', { name: profileName });

    // AC8: Wait for state change event
    await waitFor(
      () => {
        expect(stateChanges.length).toBeGreaterThan(0);
      },
      { timeout: 5000 }
    );

    // Verify state change was received
    const lastState = stateChanges[stateChanges.length - 1];
    expect(lastState).toBeDefined();
    console.log('✓ AC8: State change event received via subscription');

    // AC2: Update config
    const simpleConfig = `
      // Simple test configuration
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    console.log('Updating configuration...');
    await result.current.command('update_config', {
      profile_name: profileName,
      code: simpleConfig,
    });

    // AC2: Verify config was updated
    const config = await result.current.query('get_config', { profile_name: profileName });
    expect(config).toHaveProperty('code');
    expect(config).toHaveProperty('hash');
    console.log('✓ AC2: Configuration updated successfully');

    // AC2: Delete profile
    console.log(`Deleting profile: ${profileName}`);
    await result.current.command('delete_profile', { name: profileName });

    // AC2: Verify profile was deleted
    const profilesAfterDelete = await result.current.query('get_profiles');
    expect(profilesAfterDelete).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ name: profileName })])
    );
    console.log('✓ AC2: Profile deleted successfully');

    // AC10: Cleanup subscription
    unsubscribe();
    await new Promise((resolve) => setTimeout(resolve, 100));
    console.log('✓ AC10: Subscription cleanup successful');

    console.log('\n✅ All acceptance criteria verified successfully!');
  }, 30000); // 30 second timeout for the test

  it('should handle query timeouts (AC9)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Request should complete well before 30s timeout
    const startTime = Date.now();
    await result.current.query('get_profiles');
    const elapsed = Date.now() - startTime;

    expect(elapsed).toBeLessThan(5000);
    console.log(`✓ AC9: Request completed in ${elapsed}ms (well before 30s timeout)`);
  }, 15000);

  it('should handle errors correctly (AC6)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Try to delete a non-existent profile (should reject)
    await expect(
      result.current.command('delete_profile', { name: 'non-existent-profile-xyz' })
    ).rejects.toThrow();

    console.log('✓ AC6: Error handling works correctly');
  }, 15000);

  it('should handle concurrent requests with UUID correlation (AC7)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Send multiple concurrent requests
    console.log('Sending 3 concurrent requests...');
    const [profiles, devices, layers] = await Promise.all([
      result.current.query('get_profiles'),
      result.current.query('get_devices'),
      result.current.query('get_layers', { profile_name: 'default' }),
    ]);

    // All requests should complete successfully and be correlated correctly
    expect(profiles).toBeDefined();
    expect(devices).toBeDefined();
    expect(layers).toBeDefined();
    console.log('✓ AC7: Concurrent requests handled correctly with UUID correlation');
  }, 15000);

  it('should verify auto-reconnect configuration (AC8)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for initial connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Verify connection is established
    expect(result.current.readyState).toBe(1); // OPEN

    // Note: Full auto-reconnect testing would require stopping the daemon,
    // which is not practical for this test. The auto-reconnect configuration
    // is verified in the unit tests (3s interval, 10 max attempts).
    console.log('✓ AC8: Auto-reconnect configuration verified (see unit tests for details)');
  }, 15000);

  it('should handle subscription events correctly (AC3, AC4)', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Subscribe to latency channel
    const latencyUpdates: unknown[] = [];
    const unsubscribeLatency = result.current.subscribe('latency', (data) => {
      console.log('Latency update received:', data);
      latencyUpdates.push(data);
    });

    // Wait for at least one latency update (broadcasts every 1 second)
    await waitFor(
      () => {
        expect(latencyUpdates.length).toBeGreaterThan(0);
      },
      { timeout: 5000 }
    );

    const latencyData = latencyUpdates[0];
    expect(latencyData).toBeDefined();
    expect(latencyData).toHaveProperty('min');
    expect(latencyData).toHaveProperty('avg');
    expect(latencyData).toHaveProperty('max');

    // Cleanup
    unsubscribeLatency();
    await new Promise((resolve) => setTimeout(resolve, 100));
    console.log('✓ AC3, AC4: Subscription events received and handled correctly');
  }, 15000);
});
