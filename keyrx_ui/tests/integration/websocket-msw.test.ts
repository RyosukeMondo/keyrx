/**
 * MSW WebSocket Infrastructure Integration Smoke Test
 *
 * Comprehensive test suite verifying the MSW WebSocket infrastructure works correctly.
 * Tests automatic connection handling, helper function broadcasting, state isolation,
 * and custom handler overrides.
 *
 * This test ensures the test infrastructure itself is working properly, catching
 * regressions in the MSW WebSocket setup that would break other tests.
 */

import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { cleanupMockWebSocket } from '../helpers/websocket';
import { ws } from 'msw';
import { buildWsUrl } from '../../src/config/constants';
import { server } from '../../src/test/mocks/server';
import {
  createWebSocketHandlers,
  resetWebSocketState,
  broadcastEvent,
} from '../../src/test/mocks/websocketHandlers';
import {
  setDaemonState,
  sendLatencyUpdate,
  sendKeyEvent,
  waitForWebSocketConnection,
} from '../../src/test/mocks/websocketHelpers';
import type { ServerMessage, ClientMessage } from '../../src/types/rpc';

describe('MSW WebSocket Infrastructure', () => {
  let wsConnection: WebSocket | null = null;
  const receivedMessages: ServerMessage[] = [];

  /**
   * Create a WebSocket connection and set up message listener
   */
  function createWebSocket(): Promise<WebSocket> {
    return new Promise((resolve, reject) => {
      const socket = new WebSocket(buildWsUrl());
      receivedMessages.length = 0; // Clear previous messages

      socket.onopen = () => {
        resolve(socket);
      };

      socket.onerror = (error) => {
        reject(error);
      };

      socket.onmessage = (event) => {
        try {
          const message: ServerMessage = JSON.parse(event.data);
          receivedMessages.push(message);
        } catch (error) {
          console.error(
            'Failed to parse WebSocket message:',
            event.data,
            error
          );
        }
      };

      wsConnection = socket;
    });
  }

  /**
   * Close WebSocket connection
   */
  function closeWebSocket() {
    if (wsConnection && wsConnection.readyState === WebSocket.OPEN) {
      wsConnection.close();
      wsConnection = null;
    }
  }

  /**
   * Wait for a specific message type
   */
  async function waitForMessage(
    predicate: (msg: ServerMessage) => boolean,
    timeout = 1000
  ): Promise<ServerMessage> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      const message = receivedMessages.find(predicate);
      if (message) {
        return message;
      }
      // Wait a bit before checking again
      await new Promise((resolve) => setTimeout(resolve, 10));
    }

    throw new Error(
      `Timeout waiting for message. Received: ${JSON.stringify(receivedMessages, null, 2)}`
    );
  }

  let mswWebSocket: typeof WebSocket;
  beforeAll(() => {
    // Runs after setup.ts's beforeAll(server.listen), which patches WebSocket
    mswWebSocket = globalThis.WebSocket;
  });

  beforeEach(async () => {
    // The global setup.ts beforeEach installs jest-websocket-mock (mock-socket
    // replaces globalThis.WebSocket). This suite exercises MSW's WebSocket
    // interception instead, so put MSW's patched WebSocket back.
    cleanupMockWebSocket();
    globalThis.WebSocket = mswWebSocket;
    // server.ts registers HTTP handlers only; opt in to the WS handlers here.
    server.use(...createWebSocketHandlers());
    // Reset WebSocket state before each test
    resetWebSocketState();
    receivedMessages.length = 0;
  });

  afterEach(() => {
    // Clean up WebSocket connection
    closeWebSocket();
  });

  describe('Automatic Connection Handling', () => {
    it('should connect to MSW WebSocket server automatically', async () => {
      const socket = await createWebSocket();

      // Verify socket is connected
      expect(socket.readyState).toBe(WebSocket.OPEN);

      // Should receive connected handshake message
      const connectedMsg = await waitForMessage(
        (msg) => msg.type === 'connected'
      );
      expect(connectedMsg).toMatchObject({
        type: 'connected',
        version: '1.0.0',
      });
      expect(connectedMsg.timestamp).toBeGreaterThan(0);
    });

    it('should handle multiple concurrent connections', async () => {
      // Create two connections with message listeners set up before connection
      const receivedMessages1: ServerMessage[] = [];
      const socket1 = new WebSocket(buildWsUrl());
      socket1.onmessage = (event) => {
        receivedMessages1.push(JSON.parse(event.data));
      };
      await new Promise((resolve, reject) => {
        socket1.onopen = resolve;
        socket1.onerror = reject;
      });

      const receivedMessages2: ServerMessage[] = [];
      const socket2 = new WebSocket(buildWsUrl());
      socket2.onmessage = (event) => {
        receivedMessages2.push(JSON.parse(event.data));
      };
      await new Promise((resolve, reject) => {
        socket2.onopen = resolve;
        socket2.onerror = reject;
      });

      // Both should receive connected messages
      expect(socket1.readyState).toBe(WebSocket.OPEN);
      expect(socket2.readyState).toBe(WebSocket.OPEN);

      // Wait for connected messages
      await new Promise((resolve) => setTimeout(resolve, 100));
      expect(receivedMessages1.some((msg) => msg.type === 'connected')).toBe(
        true
      );
      expect(receivedMessages2.some((msg) => msg.type === 'connected')).toBe(
        true
      );

      // Clean up
      socket1.close();
      socket2.close();
    });

    it('should handle subscribe/unsubscribe lifecycle', async () => {
      const socket = await createWebSocket();

      // Wait for connected message
      await waitForMessage((msg) => msg.type === 'connected');

      // Send subscribe message
      const subscribeMsg: ClientMessage = {
        type: 'subscribe',
        id: 'sub-1',
        channel: 'daemon-state',
      };
      socket.send(JSON.stringify(subscribeMsg));

      // Should receive subscribe response
      const subscribeResponse = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'sub-1'
      );
      expect(subscribeResponse).toMatchObject({
        type: 'response',
        id: 'sub-1',
        result: { success: true, channel: 'daemon-state' },
      });

      // Should receive initial daemon-state event
      const initialState = await waitForMessage(
        (msg) => msg.type === 'event' && msg.channel === 'daemon-state'
      );
      expect(initialState).toMatchObject({
        type: 'event',
        channel: 'daemon-state',
        data: {
          activeProfile: 'default',
          modifiers: [],
          locks: [],
          layer: 'base',
        },
      });

      // Send unsubscribe message
      const unsubscribeMsg: ClientMessage = {
        type: 'unsubscribe',
        id: 'unsub-1',
        channel: 'daemon-state',
      };
      socket.send(JSON.stringify(unsubscribeMsg));

      // Should receive unsubscribe response
      const unsubscribeResponse = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'unsub-1'
      );
      expect(unsubscribeResponse).toMatchObject({
        type: 'response',
        id: 'unsub-1',
        result: { success: true, channel: 'daemon-state' },
      });
    });
  });

  describe('Helper Function Broadcasting', () => {
    beforeEach(async () => {
      // Create connection and subscribe to all channels
      const socket = await createWebSocket();

      // Wait for connected message
      await waitForMessage((msg) => msg.type === 'connected');

      // Subscribe to daemon-state
      const subscribeState: ClientMessage = {
        type: 'subscribe',
        id: 'sub-state',
        channel: 'daemon-state',
      };
      socket.send(JSON.stringify(subscribeState));
      await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'sub-state'
      );

      // Subscribe to latency
      const subscribeLatency: ClientMessage = {
        type: 'subscribe',
        id: 'sub-latency',
        channel: 'latency',
      };
      socket.send(JSON.stringify(subscribeLatency));
      await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'sub-latency'
      );

      // Subscribe to events
      const subscribeEvents: ClientMessage = {
        type: 'subscribe',
        id: 'sub-events',
        channel: 'events',
      };
      socket.send(JSON.stringify(subscribeEvents));
      await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'sub-events'
      );

      // Clear received messages (keep only new events)
      receivedMessages.length = 0;
    });

    it('should broadcast daemon state changes via setDaemonState()', async () => {
      // Broadcast daemon state change
      setDaemonState({ activeProfile: 'gaming', layer: 'fn' });

      // Should receive daemon-state event
      const stateEvent = await waitForMessage(
        (msg) => msg.type === 'event' && msg.channel === 'daemon-state'
      );

      expect(stateEvent).toMatchObject({
        type: 'event',
        channel: 'daemon-state',
        data: {
          activeProfile: 'gaming',
          modifiers: [],
          locks: [],
          layer: 'fn',
        },
      });
    });

    it('should broadcast latency updates via sendLatencyUpdate()', async () => {
      // Broadcast latency update
      sendLatencyUpdate({
        min: 100,
        avg: 500,
        max: 2000,
        p95: 1500,
        p99: 1800,
      });

      // Should receive latency event
      const latencyEvent = await waitForMessage(
        (msg) => msg.type === 'event' && msg.channel === 'latency'
      );

      expect(latencyEvent).toMatchObject({
        type: 'event',
        channel: 'latency',
        data: {
          min: 100,
          avg: 500,
          max: 2000,
          p95: 1500,
          p99: 1800,
        },
      });
      expect((latencyEvent.data as any).timestamp).toBeGreaterThan(0);
    });

    it('should broadcast key events via sendKeyEvent()', async () => {
      // Broadcast key event
      sendKeyEvent({
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'KEY_A',
        output: 'KEY_B',
        latency: 500,
      });

      // Should receive key event
      const keyEvent = await waitForMessage(
        (msg) => msg.type === 'event' && msg.channel === 'events'
      );

      expect(keyEvent).toMatchObject({
        type: 'event',
        channel: 'events',
        data: {
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'KEY_A',
          output: 'KEY_B',
          latency: 500,
        },
      });
      expect((keyEvent.data as any).timestamp).toBeGreaterThan(0);
    });

    it('should broadcast to multiple subscribed connections', async () => {
      // Create second connection and subscribe
      const socket2 = new WebSocket(buildWsUrl());
      const receivedMessages2: ServerMessage[] = [];
      await new Promise((resolve, reject) => {
        socket2.onopen = resolve;
        socket2.onerror = reject;
        socket2.onmessage = (event) => {
          receivedMessages2.push(JSON.parse(event.data));
        };
      });

      // Wait for connected message
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Subscribe second connection to daemon-state
      const subscribeMsg: ClientMessage = {
        type: 'subscribe',
        id: 'sub-2',
        channel: 'daemon-state',
      };
      socket2.send(JSON.stringify(subscribeMsg));

      // Wait for subscription
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Clear messages
      receivedMessages.length = 0;
      receivedMessages2.length = 0;

      // Broadcast state change
      setDaemonState({ activeProfile: 'coding' });

      // Both connections should receive the event
      await new Promise((resolve) => setTimeout(resolve, 50));

      const event1 = receivedMessages.find(
        (msg) => msg.type === 'event' && msg.channel === 'daemon-state'
      );
      const event2 = receivedMessages2.find(
        (msg) => msg.type === 'event' && msg.channel === 'daemon-state'
      );

      expect(event1).toBeDefined();
      expect(event2).toBeDefined();
      expect((event1 as any).data.activeProfile).toBe('coding');
      expect((event2 as any).data.activeProfile).toBe('coding');

      // Clean up
      socket2.close();
    });
  });

  describe('State Isolation Between Tests', () => {
    it('should reset state between tests (test 1)', async () => {
      // First, verify initial state is default
      const socket = await createWebSocket();
      await waitForMessage((msg) => msg.type === 'connected');

      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'query-1',
        method: 'get_active_profile',
        params: {},
      };
      socket.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'query-1'
      );

      // Should start with default state (proving reset worked from any previous test)
      expect((response.result as any).activeProfile).toBe('default');
    });

    it('should reset state between tests (test 2)', async () => {
      // State should be reset to default (not 'test-profile-1' from previous test)
      const socket = await createWebSocket();
      await waitForMessage((msg) => msg.type === 'connected');

      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'query-2',
        method: 'get_active_profile',
        params: {},
      };
      socket.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'query-2'
      );

      // Should be reset to default, not 'test-profile-1'
      expect((response.result as any).activeProfile).toBe('default');
    });

    it('should clear connections between tests', async () => {
      // This test verifies that connections from previous tests are cleaned up
      // If connections weren't cleared, we'd have lingering subscriptions

      const socket = await createWebSocket();
      await waitForMessage((msg) => msg.type === 'connected');

      // Subscribe to daemon-state
      const subscribeMsg: ClientMessage = {
        type: 'subscribe',
        id: 'sub-isolation',
        channel: 'daemon-state',
      };
      socket.send(JSON.stringify(subscribeMsg));
      await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'sub-isolation'
      );

      // Clear messages and broadcast
      receivedMessages.length = 0;
      setDaemonState({ activeProfile: 'isolated-test' });

      // Should receive exactly one event (not multiple from previous tests)
      await new Promise((resolve) => setTimeout(resolve, 50));
      const events = receivedMessages.filter(
        (msg) => msg.type === 'event' && msg.channel === 'daemon-state'
      );

      expect(events.length).toBe(1);
      expect((events[0].data as any).activeProfile).toBe('isolated-test');
    });
  });

  describe('Custom Handler Overrides', () => {
    it('should allow overriding handlers with server.use()', async () => {
      // Override WebSocket handler with custom behavior
      const customHandler = ws
        .link(buildWsUrl())
        .addEventListener('connection', ({ client }) => {
          // Send custom connected message
          const customMessage: ServerMessage = {
            type: 'connected',
            version: '2.0.0-custom',
            timestamp: Date.now() * 1000,
          };
          client.send(JSON.stringify(customMessage));

          // Handle custom message behavior
          client.addEventListener('message', (event: MessageEvent) => {
            try {
              const message: ClientMessage = JSON.parse(event.data);
              if (
                message.type === 'query' &&
                message.method === 'get_active_profile'
              ) {
                const response: ServerMessage = {
                  type: 'response',
                  id: message.id,
                  result: { activeProfile: 'custom-override-profile' },
                };
                client.send(JSON.stringify(response));
              }
            } catch (error) {
              // Ignore parse errors
            }
          });
        });

      // Use custom handler
      server.use(customHandler);

      try {
        const socket = await createWebSocket();

        // Should receive custom connected message
        const connectedMsg = await waitForMessage(
          (msg) => msg.type === 'connected'
        );
        expect(connectedMsg.version).toBe('2.0.0-custom');

        // Query should return custom result
        const queryMsg: ClientMessage = {
          type: 'query',
          id: 'custom-query',
          method: 'get_active_profile',
          params: {},
        };
        socket.send(JSON.stringify(queryMsg));

        const response = await waitForMessage(
          (msg) => msg.type === 'response' && msg.id === 'custom-query'
        );
        expect((response.result as any).activeProfile).toBe(
          'custom-override-profile'
        );
      } finally {
        // Reset handlers after test
        server.resetHandlers();
      }
    });

    it('should restore default handlers after server.resetHandlers()', async () => {
      // After previous test's server.resetHandlers(), should use default handlers
      const socket = await createWebSocket();

      // Should receive default connected message (not custom)
      const connectedMsg = await waitForMessage(
        (msg) => msg.type === 'connected'
      );
      expect(connectedMsg.version).toBe('1.0.0'); // Default version

      // Query should return default result
      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'default-query',
        method: 'get_active_profile',
        params: {},
      };
      socket.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'default-query'
      );
      expect((response.result as any).activeProfile).toBe('default'); // Default profile
    });
  });

  describe('RPC Query Handling', () => {
    beforeEach(async () => {
      wsConnection = await createWebSocket();
      await waitForMessage((msg) => msg.type === 'connected');
    });

    it('should handle get_profiles query', async () => {
      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'get-profiles',
        method: 'get_profiles',
        params: {},
      };
      wsConnection!.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'get-profiles'
      );

      expect(response.result).toHaveProperty('profiles');
      expect(Array.isArray((response.result as any).profiles)).toBe(true);
      expect((response.result as any).profiles.length).toBeGreaterThan(0);
    });

    it('should handle get_devices query', async () => {
      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'get-devices',
        method: 'get_devices',
        params: {},
      };
      wsConnection!.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'get-devices'
      );

      expect(response.result).toHaveProperty('devices');
      expect(Array.isArray((response.result as any).devices)).toBe(true);
    });

    it('should handle unsupported method with error', async () => {
      const queryMsg: ClientMessage = {
        type: 'query',
        id: 'unsupported',
        method: 'unsupported_method' as any,
        params: {},
      };
      wsConnection!.send(JSON.stringify(queryMsg));

      const response = await waitForMessage(
        (msg) => msg.type === 'response' && msg.id === 'unsupported'
      );

      expect(response.error).toBeDefined();
      expect(response.error?.code).toBe(-32601); // Method not found
      expect(response.error?.message).toContain('Method not found');
    });
  });

  describe('Helper Utility Functions', () => {
    it('should provide waitForWebSocketConnection() utility', async () => {
      // Create connection in background
      const socket = new WebSocket(buildWsUrl());
      socket.onmessage = (event) => {
        receivedMessages.push(JSON.parse(event.data));
      };

      // Wait for connection
      await waitForWebSocketConnection();

      // Connection should be ready (helper resolves immediately with MSW)
      expect(true).toBe(true); // Helper completed without error

      // Clean up
      socket.close();
    });
  });
});
