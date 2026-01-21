/**
 * WebSocket API Test Cases
 *
 * Tests for WebSocket real-time event communication:
 * - WebSocket connection lifecycle (connect, disconnect)
 * - Channel subscription (devices, profiles, metrics, state)
 * - Event notifications (device changes, profile changes)
 * - Connection resilience (reconnection)
 */

import { ApiClient } from '../api-client/client.js';
import {
  WebSocketClient,
  ConnectionState,
  ConnectionError,
  TimeoutError,
  SubscriptionError,
} from '../api-client/websocket-client.js';
import type { TestCase, TestResult } from './api-tests.js';
import { z } from 'zod';

/**
 * No-op setup function for tests that don't need preparation
 */
const noOpSetup = async (): Promise<void> => {
  // No setup needed
};

/**
 * No-op cleanup function for tests that don't modify state
 */
const noOpCleanup = async (): Promise<void> => {
  // No cleanup needed
};

/**
 * WebSocket URL for tests (derived from API base URL)
 */
const getWebSocketUrl = (apiBaseUrl: string): string => {
  // Convert http://host:port/api to ws://host:port/ws-rpc
  const url = new URL(apiBaseUrl);
  const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${url.host}/ws-rpc`;
};

/**
 * WebSocket test cases
 */
export const websocketTestCases: TestCase[] = [
  // =================================================================
  // Connection Tests
  // =================================================================
  {
    id: 'websocket-001',
    name: 'WebSocket - Connect and disconnect lifecycle',
    endpoint: '/ws-rpc',
    scenario: 'connection',
    category: 'websocket',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });

      try {
        // Connect
        await wsClient.connect(5000);
        const connectedState = wsClient.getState();
        const isConnected = wsClient.isConnected();

        // Disconnect
        wsClient.disconnect();
        const disconnectedState = wsClient.getState();

        return {
          status: 200,
          data: {
            success: true,
            connectedState,
            isConnected,
            disconnectedState,
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        connectedState?: string;
        isConnected?: boolean;
        disconnectedState?: string;
        error?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Connection failed: ${actualData.error}`,
        };
      }

      // Verify connected state
      if (actualData.connectedState !== ConnectionState.CONNECTED) {
        return {
          passed: false,
          actual,
          expected: { connectedState: ConnectionState.CONNECTED },
          error: `Expected state 'connected', got '${actualData.connectedState}'`,
        };
      }

      // Verify isConnected flag
      if (!actualData.isConnected) {
        return {
          passed: false,
          actual,
          expected: { isConnected: true },
          error: 'Expected isConnected to be true',
        };
      }

      // Verify disconnected state
      if (actualData.disconnectedState !== ConnectionState.CLOSED) {
        return {
          passed: false,
          actual,
          expected: { disconnectedState: ConnectionState.CLOSED },
          error: `Expected state 'closed', got '${actualData.disconnectedState}'`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        connectedState: ConnectionState.CONNECTED,
        isConnected: true,
        disconnectedState: ConnectionState.CLOSED,
      },
    },
  },

  // =================================================================
  // Subscription Tests
  // =================================================================
  {
    id: 'websocket-002',
    name: 'WebSocket - Subscribe to channel and receive acknowledgment',
    endpoint: '/ws-rpc',
    scenario: 'subscription',
    category: 'websocket',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });

      try {
        // Connect
        await wsClient.connect(5000);

        // Subscribe to 'devices' channel
        await wsClient.subscribe('devices', 5000);

        // Verify subscription
        const subscriptions = wsClient.getSubscriptions();

        // Disconnect
        wsClient.disconnect();

        return {
          status: 200,
          data: {
            success: true,
            subscribed: subscriptions.has('devices'),
            subscriptions: Array.from(subscriptions),
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
            errorType: error instanceof Error ? error.constructor.name : 'Unknown',
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        subscribed?: boolean;
        subscriptions?: string[];
        error?: string;
        errorType?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Subscription failed: ${actualData.error} (${actualData.errorType})`,
        };
      }

      // Verify subscription successful
      if (!actualData.subscribed) {
        return {
          passed: false,
          actual,
          expected: { subscribed: true },
          error: 'Expected to be subscribed to devices channel',
        };
      }

      // Verify subscriptions list
      if (!actualData.subscriptions?.includes('devices')) {
        return {
          passed: false,
          actual,
          expected: { subscriptions: ['devices'] },
          error: `Expected subscriptions to include 'devices', got: ${actualData.subscriptions?.join(', ')}`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        subscribed: true,
        subscriptions: ['devices'],
      },
    },
  },

  // =================================================================
  // Event Notification Tests
  // =================================================================
  {
    id: 'websocket-003',
    name: 'WebSocket - Receive device event notification',
    endpoint: '/ws-rpc',
    scenario: 'device_event',
    category: 'websocket',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });

      try {
        // Connect and subscribe to devices channel
        await wsClient.connect(5000);
        await wsClient.subscribe('devices', 5000);

        // Get list of devices
        const devicesResponse = await client.customRequest(
          'GET',
          '/api/devices',
          z.object({
            devices: z.array(
              z.object({
                id: z.string(),
                name: z.string(),
              })
            ),
          })
        );

        if (!devicesResponse.data.devices.length) {
          return {
            status: 500,
            data: {
              success: false,
              error: 'No devices available for testing',
            },
          };
        }

        const testDevice = devicesResponse.data.devices[0];
        const originalName = testDevice.name;
        const newName = `${originalName}-ws-test`;

        // Set up event listener before making change
        const eventPromise = wsClient.waitForEvent(
          (event) => event.content.channel === 'devices',
          5000
        );

        // Update device name via REST API
        await client.customRequest(
          'PUT',
          `/api/devices/${testDevice.id}/name`,
          z.object({ success: z.boolean() }),
          { name: newName }
        );

        // Wait for WebSocket event
        const event = await eventPromise;

        // Restore original name
        await client.customRequest(
          'PUT',
          `/api/devices/${testDevice.id}/name`,
          z.object({ success: z.boolean() }),
          { name: originalName }
        );

        // Disconnect
        wsClient.disconnect();

        return {
          status: 200,
          data: {
            success: true,
            receivedEvent: true,
            eventChannel: event.content.channel,
            eventData: event.content.data,
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
            errorType: error instanceof Error ? error.constructor.name : 'Unknown',
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        receivedEvent?: boolean;
        eventChannel?: string;
        eventData?: unknown;
        error?: string;
        errorType?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Event notification failed: ${actualData.error} (${actualData.errorType})`,
        };
      }

      // Verify event received
      if (!actualData.receivedEvent) {
        return {
          passed: false,
          actual,
          expected: { receivedEvent: true },
          error: 'Expected to receive device event',
        };
      }

      // Verify event channel
      if (actualData.eventChannel !== 'devices') {
        return {
          passed: false,
          actual,
          expected: { eventChannel: 'devices' },
          error: `Expected channel 'devices', got '${actualData.eventChannel}'`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        receivedEvent: true,
        eventChannel: 'devices',
      },
    },
  },

  {
    id: 'websocket-004',
    name: 'WebSocket - Receive profile event notification',
    endpoint: '/ws-rpc',
    scenario: 'profile_event',
    category: 'websocket',
    priority: 1,
    setup: async () => {
      // No setup needed - cleanup will handle test profile
    },
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });
      const testProfileName = 'ws-test-profile';

      try {
        // Connect and subscribe to profiles channel
        await wsClient.connect(5000);
        await wsClient.subscribe('profiles', 5000);

        // Create test profile
        await client.createProfile(testProfileName, 'blank');

        // Set up event listener before activating profile
        const eventPromise = wsClient.waitForEvent(
          (event) => event.content.channel === 'profiles',
          5000
        );

        // Activate profile via REST API
        await client.customRequest(
          'PUT',
          `/api/profiles/${testProfileName}/activate`,
          z.object({ success: z.boolean() })
        );

        // Wait for WebSocket event
        const event = await eventPromise;

        // Get original active profile to restore
        const profilesResponse = await client.customRequest(
          'GET',
          '/api/profiles',
          z.object({
            profiles: z.array(
              z.object({
                name: z.string(),
                active: z.boolean(),
              })
            ),
          })
        );

        const originalProfile = profilesResponse.data.profiles.find(
          (p) => p.name !== testProfileName && !p.name.startsWith('ws-test')
        );

        // Restore original profile if one exists
        if (originalProfile) {
          await client.customRequest(
            'PUT',
            `/api/profiles/${originalProfile.name}/activate`,
            z.object({ success: z.boolean() })
          );
        }

        // Disconnect
        wsClient.disconnect();

        return {
          status: 200,
          data: {
            success: true,
            receivedEvent: true,
            eventChannel: event.content.channel,
            eventData: event.content.data,
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
            errorType: error instanceof Error ? error.constructor.name : 'Unknown',
          },
        };
      } finally {
        wsClient.disconnect();

        // Cleanup test profile
        try {
          await client.customRequest(
            'DELETE',
            `/api/profiles/${testProfileName}`,
            z.object({ success: z.boolean() })
          );
        } catch {
          // Ignore cleanup errors
        }
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        receivedEvent?: boolean;
        eventChannel?: string;
        eventData?: unknown;
        error?: string;
        errorType?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Profile event notification failed: ${actualData.error} (${actualData.errorType})`,
        };
      }

      // Verify event received
      if (!actualData.receivedEvent) {
        return {
          passed: false,
          actual,
          expected: { receivedEvent: true },
          error: 'Expected to receive profile event',
        };
      }

      // Verify event channel
      if (actualData.eventChannel !== 'profiles') {
        return {
          passed: false,
          actual,
          expected: { eventChannel: 'profiles' },
          error: `Expected channel 'profiles', got '${actualData.eventChannel}'`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        receivedEvent: true,
        eventChannel: 'profiles',
      },
    },
  },

  // =================================================================
  // Resilience Tests
  // =================================================================
  {
    id: 'websocket-005',
    name: 'WebSocket - Reconnection and subscription restoration',
    endpoint: '/ws-rpc',
    scenario: 'reconnection',
    category: 'websocket',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({
        url: wsUrl,
        reconnect: true,
        reconnectDelayMs: 500,
        maxReconnectAttempts: 3,
      });

      try {
        // Initial connection
        await wsClient.connect(5000);
        const initialState = wsClient.getState();

        // Subscribe to channels
        await wsClient.subscribe('devices', 5000);
        await wsClient.subscribe('profiles', 5000);
        const initialSubscriptions = Array.from(wsClient.getSubscriptions());

        // Simulate disconnect by closing connection
        wsClient.disconnect();
        const disconnectedState = wsClient.getState();

        // Reconnect
        await wsClient.connect(5000);
        const reconnectedState = wsClient.getState();

        // Verify we can re-subscribe (subscriptions don't auto-restore)
        await wsClient.subscribe('devices', 5000);
        const restoredSubscriptions = Array.from(wsClient.getSubscriptions());

        // Disconnect cleanly
        wsClient.disconnect();

        return {
          status: 200,
          data: {
            success: true,
            initialState,
            disconnectedState,
            reconnectedState,
            initialSubscriptions,
            restoredSubscriptions,
            canResubscribe: restoredSubscriptions.includes('devices'),
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
            errorType: error instanceof Error ? error.constructor.name : 'Unknown',
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        initialState?: string;
        disconnectedState?: string;
        reconnectedState?: string;
        initialSubscriptions?: string[];
        restoredSubscriptions?: string[];
        canResubscribe?: boolean;
        error?: string;
        errorType?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Reconnection test failed: ${actualData.error} (${actualData.errorType})`,
        };
      }

      // Verify initial connection
      if (actualData.initialState !== ConnectionState.CONNECTED) {
        return {
          passed: false,
          actual,
          expected: { initialState: ConnectionState.CONNECTED },
          error: `Expected initial state 'connected', got '${actualData.initialState}'`,
        };
      }

      // Verify disconnected state
      if (actualData.disconnectedState !== ConnectionState.CLOSED) {
        return {
          passed: false,
          actual,
          expected: { disconnectedState: ConnectionState.CLOSED },
          error: `Expected disconnected state 'closed', got '${actualData.disconnectedState}'`,
        };
      }

      // Verify reconnected state
      if (actualData.reconnectedState !== ConnectionState.CONNECTED) {
        return {
          passed: false,
          actual,
          expected: { reconnectedState: ConnectionState.CONNECTED },
          error: `Expected reconnected state 'connected', got '${actualData.reconnectedState}'`,
        };
      }

      // Verify can resubscribe
      if (!actualData.canResubscribe) {
        return {
          passed: false,
          actual,
          expected: { canResubscribe: true },
          error: 'Expected to be able to resubscribe after reconnection',
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        initialState: ConnectionState.CONNECTED,
        disconnectedState: ConnectionState.CLOSED,
        reconnectedState: ConnectionState.CONNECTED,
        canResubscribe: true,
      },
    },
  },
];
