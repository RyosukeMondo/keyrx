/**
 * Tests for useUnifiedApi hook
 *
 * This test file verifies the WebSocket RPC communication hook implementation using jest-websocket-mock.
 * It tests key acceptance criteria from REQ-1:
 * - AC1: Connection and Connected handshake
 * - AC3: Subscription and event handling
 * - AC10: Cleanup on unmount
 *
 * Note: Query/command tests are in the contract test file (tests/hooks/useUnifiedApi.test.tsx)
 * as they test message format compatibility rather than hook behavior.
 */

import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useUnifiedApi } from './useUnifiedApi';
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  getMockWebSocket,
  simulateConnected,
  sendDaemonStateUpdate,
  sendLatencyUpdate,
  sendKeyEvent,
  WS_URL,
} from '../../tests/testUtils';

// Mock uuid for deterministic test IDs
let uuidCounter = 0;
vi.mock('uuid', () => ({
  v4: () => `test-uuid-${uuidCounter++}`,
}));

describe('useUnifiedApi', () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    uuidCounter = 0;
    await setupMockWebSocket();
  });

  afterEach(() => {
    vi.clearAllTimers();
    cleanupMockWebSocket();
  });

  describe('Connection and Handshake (AC1)', () => {
    it('should initialize with connection state', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Initially not connected (waiting for 'connected' message from MSW)
      expect(result.current.isConnected).toBe(false);

      // Wait for WebSocket to establish connection
      await waitFor(() => {
        expect(result.current.readyState).toBe(1); // OPEN
      }, { timeout: 1000 });
    });

    it('should become connected after receiving connected message', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for WebSocket to connect to mock server
      const server = getMockWebSocket();
      await server.connected;

      // Send 'connected' handshake message
      await simulateConnected();

      // Wait for isConnected to become true
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      }, { timeout: 2000 });
    });
  });

  describe('Subscription Management (AC3)', () => {
    it('should subscribe to channel and receive initial state', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      // Wait for connection
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const handler = vi.fn();
      result.current.subscribe('daemon-state', handler);

      // Send initial state for subscription
      sendDaemonStateUpdate({ layer: 'base', activeProfile: 'default' });

      await waitFor(() => {
        expect(handler).toHaveBeenCalled();
      }, { timeout: 2000 });

      const initialState = handler.mock.calls[0][0];
      expect(initialState).toBeDefined();
      expect((initialState as any).layer).toBeDefined();
    });

    it('should receive subscription events from WebSocket helpers', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const handler = vi.fn();
      result.current.subscribe('daemon-state', handler);

      // Send initial state
      sendDaemonStateUpdate({ layer: 'base', activeProfile: 'default' });

      // Wait for initial state
      await waitFor(() => {
        expect(handler).toHaveBeenCalled();
      });

      const callCountBefore = handler.mock.calls.length;

      // Simulate state change using jest-websocket-mock helper
      sendDaemonStateUpdate({ activeProfile: 'gaming', layer: 'fn' });

      // Wait for event to be received
      await waitFor(() => {
        expect(handler.mock.calls.length).toBeGreaterThan(callCountBefore);
      }, { timeout: 1000 });

      // Verify the event data
      const latestCall = handler.mock.calls[handler.mock.calls.length - 1][0];
      expect((latestCall as any).activeProfile).toBe('gaming');
      expect((latestCall as any).layer).toBe('fn');
    });

    it('should handle multiple handlers for same channel', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const handler1 = vi.fn();
      const handler2 = vi.fn();

      result.current.subscribe('latency', handler1);
      result.current.subscribe('latency', handler2);

      // Send initial latency data
      sendLatencyUpdate({ min: 50, avg: 100, max: 200, p95: 150, p99: 180 });

      // Clear any initial subscription events
      await waitFor(() => {
        expect(handler1).toHaveBeenCalled();
      });
      handler1.mockClear();
      handler2.mockClear();

      // Send latency update using jest-websocket-mock helper
      sendLatencyUpdate({ min: 100, avg: 200, max: 500, p95: 400, p99: 450 });

      // Both handlers should receive the event
      await waitFor(() => {
        expect(handler1).toHaveBeenCalled();
        expect(handler2).toHaveBeenCalled();
      }, { timeout: 1000 });

      // Verify both got the same data
      const data1 = handler1.mock.calls[0][0];
      const data2 = handler2.mock.calls[0][0];
      expect(data1).toEqual(data2);
    });

    it('should support all subscription channels', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const stateHandler = vi.fn();
      const eventsHandler = vi.fn();
      const latencyHandler = vi.fn();

      result.current.subscribe('daemon-state', stateHandler);
      result.current.subscribe('key-events', eventsHandler);
      result.current.subscribe('latency', latencyHandler);

      // Send initial messages to each channel
      sendDaemonStateUpdate({ layer: 'base', activeProfile: 'default' });
      sendLatencyUpdate({ min: 50, avg: 100, max: 200, p95: 150, p99: 180 });

      // Wait for initial subscription messages
      await waitFor(() => {
        expect(stateHandler).toHaveBeenCalled();
        expect(latencyHandler).toHaveBeenCalled();
      });

      // Send events to each channel
      sendDaemonStateUpdate({ layer: 'test' });
      sendKeyEvent({ keyCode: 'KEY_A', eventType: 'press', input: 'KEY_A', output: 'KEY_B', latency: 100 });
      sendLatencyUpdate({ min: 100, avg: 200, max: 300, p95: 250, p99: 280 });

      // All handlers should receive events
      await waitFor(() => {
        expect(stateHandler.mock.calls.length).toBeGreaterThan(1);
        expect(eventsHandler.mock.calls.length).toBeGreaterThan(0);
        expect(latencyHandler.mock.calls.length).toBeGreaterThan(1);
      }, { timeout: 1000 });
    });

    it('should unsubscribe from channel', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const handler = vi.fn();
      const unsubscribe = result.current.subscribe('daemon-state', handler);

      // Send initial state
      sendDaemonStateUpdate({ layer: 'base', activeProfile: 'default' });

      // Wait for initial state
      await waitFor(() => {
        expect(handler).toHaveBeenCalled();
      });

      const callCountBefore = handler.mock.calls.length;

      // Unsubscribe
      unsubscribe();

      // Wait to ensure unsubscribe has been processed
      await new Promise(resolve => setTimeout(resolve, 100));

      // Send state change
      sendDaemonStateUpdate({ activeProfile: 'test', layer: 'test' });

      // Wait a bit more
      await new Promise(resolve => setTimeout(resolve, 200));

      // Handler should not receive new events (call count should be unchanged)
      expect(handler.mock.calls.length).toBe(callCountBefore);
    });
  });

  describe('Cleanup on Unmount (AC10)', () => {
    it('should unsubscribe all channels on unmount', async () => {
      const { result, unmount } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const handler1 = vi.fn();
      const handler2 = vi.fn();
      const handler3 = vi.fn();

      result.current.subscribe('daemon-state', handler1);
      result.current.subscribe('key-events', handler2);
      result.current.subscribe('latency', handler3);

      // Send initial events
      sendDaemonStateUpdate({ layer: 'base', activeProfile: 'default' });
      sendLatencyUpdate({ min: 50, avg: 100, max: 200, p95: 150, p99: 180 });

      // Wait for initial events
      await waitFor(() => {
        expect(handler1).toHaveBeenCalled();
      });

      const callCounts = [
        handler1.mock.calls.length,
        handler2.mock.calls.length,
        handler3.mock.calls.length,
      ];

      // Unmount
      unmount();

      // Wait for unmount to complete
      await new Promise(resolve => setTimeout(resolve, 100));

      // Send events after unmount
      sendDaemonStateUpdate({ activeProfile: 'test' });
      sendKeyEvent({ keyCode: 'KEY_A', eventType: 'press', input: 'KEY_A', output: 'KEY_A', latency: 100 });
      sendLatencyUpdate({ min: 100, avg: 200, max: 300, p95: 250, p99: 280 });

      // Wait to ensure no events are received
      await new Promise(resolve => setTimeout(resolve, 200));

      // Handlers should not receive new events after unmount
      expect(handler1.mock.calls.length).toBe(callCounts[0]);
      expect(handler2.mock.calls.length).toBe(callCounts[1]);
      expect(handler3.mock.calls.length).toBe(callCounts[2]);
    });
  });

  describe('Error Handling (AC6)', () => {
    it('should track connection state', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Initially lastError should be null
      expect(result.current.lastError).toBeNull();

      // Wait for server connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      // Wait for connection
      await waitFor(() => {
        expect(result.current.readyState).toBe(1);
        expect(result.current.isConnected).toBe(true);
      });

      // After successful connection, error should still be null
      expect(result.current.lastError).toBeNull();
    });
  });
});
