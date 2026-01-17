import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { waitFor } from '@testing-library/react';
import {
  WebSocketManager,
  getWebSocketInstance,
  closeWebSocketInstance,
} from './websocket';
import type { EventRecord, DaemonState, LatencyStats } from '../types';
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  getMockWebSocket,
  simulateConnected,
  WS_URL,
} from '../../tests/testUtils';

describe('WebSocketManager', () => {
  let wsManager: WebSocketManager;

  beforeEach(async () => {
    // Set up jest-websocket-mock server for WebSocket testing
    await setupMockWebSocket();
  });

  afterEach(() => {
    if (wsManager) {
      wsManager.close();
    }
    cleanupMockWebSocket();
    vi.restoreAllMocks();
  });

  describe('Connection', () => {
    it('should connect to WebSocket server', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onOpen });

      wsManager.connect();

      // Wait for WebSocket connection to jest-websocket-mock server
      const server = getMockWebSocket();
      await server.connected;

      await waitFor(() => {
        expect(onOpen).toHaveBeenCalled();
        expect(wsManager.isConnected()).toBe(true);
        expect(wsManager.getConnectionState()).toBe('connected');
      });
    });

    it('should not create duplicate connections', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onOpen });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(onOpen).toHaveBeenCalled());

      // Try to connect again
      wsManager.connect();
      await waitFor(() => {
        // Should only call onOpen once
        expect(onOpen).toHaveBeenCalledTimes(1);
      });
    });

    it('should disconnect cleanly', async () => {
      const onClose = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onClose });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      wsManager.disconnect();

      await waitFor(() => {
        expect(onClose).toHaveBeenCalled();
        expect(wsManager.isConnected()).toBe(false);
        expect(wsManager.getConnectionState()).toBe('disconnected');
      });
    });

    it('should not reconnect after close()', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onOpen });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(onOpen).toHaveBeenCalled());

      wsManager.close();

      // Try to reconnect
      wsManager.connect();

      // Should only connect once
      await waitFor(() => {
        expect(onOpen).toHaveBeenCalledTimes(1);
      });
    });
  });

  describe('Message Handling', () => {
    // TODO: Fix - message handling timing issue
    it.skip('should handle event messages', async () => {
      const onEvent = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onEvent });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      // Send event message from mock server
      const mockEvent: EventRecord = {
        id: '1',
        timestamp: '2024-01-01T00:00:00Z',
        type: 'key_press',
        keyCode: 'KEY_A',
        layer: 'base',
        latencyUs: 100,
      };

      server.send({
        type: 'event',
        payload: mockEvent,
      });

      await waitFor(() => {
        expect(onEvent).toHaveBeenCalledWith(mockEvent);
      });
    });

    // TODO: Fix - message handling timing issue
    it.skip('should handle state messages', async () => {
      const onState = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onState });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      const mockState: DaemonState = {
        activeLayer: 'base',
        modifiers: ['shift'],
        locks: [],
        tapHoldPending: false,
        uptime: 1000,
      };

      server.send({
        type: 'state',
        payload: mockState,
      });

      await waitFor(() => {
        expect(onState).toHaveBeenCalledWith(mockState);
      });
    });

    // TODO: Fix - message handling timing issue
    it.skip('should handle latency messages', async () => {
      const onLatency = vi.fn();
      wsManager = new WebSocketManager({ url: WS_URL }, { onLatency });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      const mockLatency: LatencyStats = {
        min: 50,
        max: 200,
        avg: 100,
        p50: 95,
        p95: 180,
        p99: 195,
        samples: 1000,
        timestamp: '2024-01-01T00:00:00Z',
      };

      server.send({
        type: 'latency',
        payload: mockLatency,
      });

      await waitFor(() => {
        expect(onLatency).toHaveBeenCalledWith(mockLatency);
      });
    });

    it('should handle invalid JSON gracefully', async () => {
      const consoleSpy = vi
        .spyOn(console, 'error')
        .mockImplementation(() => {});
      wsManager = new WebSocketManager({ url: WS_URL });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      // Send invalid JSON from server (use sendText for non-JSON)
      const ws = (wsManager as any).ws as WebSocket;
      ws.onmessage!(
        new MessageEvent('message', {
          data: 'invalid json',
        })
      );

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalled();
      });
      consoleSpy.mockRestore();
    });
  });

  describe('Reconnection', () => {
    it('should schedule reconnection after disconnect', async () => {
      const onConnectionStateChange = vi.fn();

      wsManager = new WebSocketManager(
        {
          url: WS_URL,
          reconnectInterval: 1000,
          reconnectDecay: 2,
          maxReconnectInterval: 10000,
        },
        { onConnectionStateChange }
      );

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      // Close the connection
      const ws = (wsManager as any).ws as WebSocket;
      ws.close();

      // Should have transitioned to disconnected
      await waitFor(() => {
        expect(onConnectionStateChange).toHaveBeenCalledWith('disconnected');
      });

      // Verify reconnection timeout is scheduled (check internal state)
      const reconnectTimeoutId = (wsManager as any).reconnectTimeoutId;
      expect(reconnectTimeoutId).not.toBeNull();
    });

    it('should stop reconnecting after max attempts', async () => {
      const onConnectionStateChange = vi.fn();
      const states: string[] = [];
      wsManager = new WebSocketManager(
        {
          url: WS_URL,
          maxReconnectAttempts: 2,
          reconnectInterval: 100,
        },
        {
          onConnectionStateChange: (state) => {
            states.push(state);
            onConnectionStateChange(state);
          },
        }
      );

      // Mock WebSocket to fail immediately using vi.stubGlobal
      const MockFailingWebSocket = class {
        constructor() {
          setTimeout(() => {
            if (this.onerror) {
              this.onerror(new Event('error'));
            }
            if (this.onclose) {
              this.onclose(new CloseEvent('close'));
            }
          }, 0);
        }
        readyState = WebSocket.CONNECTING;
        onopen: any = null;
        onclose: any = null;
        onerror: any = null;
        onmessage: any = null;
        send() {}
        close() {}
      } as any;

      vi.stubGlobal('WebSocket', MockFailingWebSocket);

      wsManager.connect();

      // Wait for multiple reconnection attempts
      await waitFor(
        () => {
          // Should reach max attempts and have 'error' state
          expect(states).toContain('error');
        },
        { timeout: 1000 }
      );

      vi.unstubAllGlobals();
    });

    it('should not reconnect if disabled', async () => {
      const onConnectionStateChange = vi.fn();
      wsManager = new WebSocketManager(
        { url: WS_URL, reconnect: false },
        { onConnectionStateChange }
      );

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      const ws = (wsManager as any).ws as WebSocket;
      ws.close();

      // Reset the spy to only count reconnection attempts
      onConnectionStateChange.mockClear();

      // Wait for potential reconnect
      await new Promise((resolve) => setTimeout(resolve, 200));

      // Should not attempt to reconnect (no new 'connecting' state)
      const connectingStates = onConnectionStateChange.mock.calls.filter(
        (call) => call[0] === 'connecting'
      );
      expect(connectingStates.length).toBe(0);
    });
  });

  describe('Singleton Instance', () => {
    it('should return same instance', () => {
      const instance1 = getWebSocketInstance();
      const instance2 = getWebSocketInstance();

      expect(instance1).toBe(instance2);

      closeWebSocketInstance();
    });

    it('should create new instance after close', () => {
      const instance1 = getWebSocketInstance();
      closeWebSocketInstance();

      const instance2 = getWebSocketInstance();

      expect(instance1).not.toBe(instance2);

      closeWebSocketInstance();
    });
  });

  describe('Connection State', () => {
    it('should track connection state changes', async () => {
      const states: string[] = [];
      const onConnectionStateChange = vi.fn((state) => states.push(state));
      const onOpen = vi.fn();

      wsManager = new WebSocketManager(
        { url: WS_URL },
        { onConnectionStateChange, onOpen }
      );

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      expect(states).toContain('connecting');

      await waitFor(() => {
        // Wait for onOpen to be called to ensure connection is established
        expect(onOpen).toHaveBeenCalled();
        expect(states).toContain('connected');
      });

      wsManager.disconnect();

      await waitFor(() => {
        expect(states).toContain('disconnected');
      });
    });
  });

  describe('Send Messages', () => {
    it('should send string messages', async () => {
      wsManager = new WebSocketManager({ url: WS_URL });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      const ws = (wsManager as any).ws as WebSocket;
      const sendSpy = vi.spyOn(ws, 'send');

      wsManager.send('test message');

      expect(sendSpy).toHaveBeenCalledWith('test message');
    });

    it('should send object messages as JSON', async () => {
      wsManager = new WebSocketManager({ url: WS_URL });

      wsManager.connect();
      const server = getMockWebSocket();
      await server.connected;
      await waitFor(() => expect(wsManager.isConnected()).toBe(true));

      const ws = (wsManager as any).ws as WebSocket;
      const sendSpy = vi.spyOn(ws, 'send');

      const message = { type: 'test', data: 'value' };
      wsManager.send(message);

      expect(sendSpy).toHaveBeenCalledWith(JSON.stringify(message));
    });

    it('should not send when disconnected', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      wsManager = new WebSocketManager({ url: WS_URL });

      wsManager.send('test');

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Cannot send message')
      );
      consoleSpy.mockRestore();
    });
  });
});
