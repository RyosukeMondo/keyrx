/**
 * Unit tests for useDaemonWebSocket hook.
 *
 * Tests cover:
 * - Auto-reconnect behavior on disconnect
 * - WebSocket message parsing and store updates
 * - Connection state tracking
 * - Error handling for malformed messages
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useDaemonWebSocket, ReadyState } from './useDaemonWebSocket';
import { useDashboardStore } from '../store/dashboardStore';
import type { DaemonState, KeyEvent, LatencyStats } from '../store/dashboardStore';

// Mock react-use-websocket
vi.mock('react-use-websocket', () => ({
  default: vi.fn(),
  ReadyState: {
    CONNECTING: 0,
    OPEN: 1,
    CLOSING: 2,
    CLOSED: 3,
    UNINSTANTIATED: -1,
  },
}));

import useWebSocket from 'react-use-websocket';

describe('useDaemonWebSocket', () => {
  // Mock WebSocket state
  let mockSendMessage: ReturnType<typeof vi.fn>;
  let mockOnMessage: ((event: MessageEvent) => void) | null = null;
  let mockReadyState: number;

  beforeEach(() => {
    vi.clearAllMocks();
    mockSendMessage = vi.fn();
    mockReadyState = ReadyState.OPEN;
    mockOnMessage = null;

    // Mock useWebSocket implementation
    vi.mocked(useWebSocket).mockImplementation((url, options) => {
      // Capture the onMessage callback
      mockOnMessage = options?.onMessage || null;

      return {
        sendMessage: mockSendMessage,
        lastMessage: null,
        readyState: mockReadyState,
      } as any;
    });

    // Reset store state
    useDashboardStore.getState().reset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('initialization', () => {
    it('should connect to default WebSocket URL', () => {
      renderHook(() => useDaemonWebSocket());

      expect(useWebSocket).toHaveBeenCalledWith(
        'ws://localhost:9867/ws',
        expect.objectContaining({
          onMessage: expect.any(Function),
          shouldReconnect: expect.any(Function),
          reconnectInterval: 3000,
          reconnectAttempts: Infinity,
          share: false,
        })
      );
    });

    it('should connect to custom WebSocket URL when provided', () => {
      renderHook(() => useDaemonWebSocket({ url: 'ws://custom-host:8080/ws' }));

      expect(useWebSocket).toHaveBeenCalledWith(
        'ws://custom-host:8080/ws',
        expect.any(Object)
      );
    });

    it('should use custom reconnect options when provided', () => {
      renderHook(() =>
        useDaemonWebSocket({
          reconnectInterval: 5000,
          reconnectAttempts: 10,
        })
      );

      expect(useWebSocket).toHaveBeenCalledWith(
        'ws://localhost:9867/ws',
        expect.objectContaining({
          reconnectInterval: 5000,
          reconnectAttempts: 10,
        })
      );
    });

    it('should disable reconnect when shouldReconnect is false', () => {
      renderHook(() => useDaemonWebSocket({ shouldReconnect: false }));

      const call = vi.mocked(useWebSocket).mock.calls[0];
      const options = call[1];
      const shouldReconnectFn = options?.shouldReconnect as () => boolean;

      expect(shouldReconnectFn()).toBe(false);
    });

    it('should enable reconnect by default', () => {
      renderHook(() => useDaemonWebSocket());

      const call = vi.mocked(useWebSocket).mock.calls[0];
      const options = call[1];
      const shouldReconnectFn = options?.shouldReconnect as () => boolean;

      expect(shouldReconnectFn()).toBe(true);
    });
  });

  describe('connection state tracking', () => {
    it('should update store connection status to connected when WebSocket opens', async () => {
      mockReadyState = ReadyState.OPEN;

      renderHook(() => useDaemonWebSocket());

      await waitFor(() => {
        expect(useDashboardStore.getState().connectionStatus).toBe('connected');
      });
    });

    it('should update store connection status to connecting when WebSocket is connecting', async () => {
      mockReadyState = ReadyState.CONNECTING;

      renderHook(() => useDaemonWebSocket());

      await waitFor(() => {
        expect(useDashboardStore.getState().connectionStatus).toBe('connecting');
      });
    });

    it('should update store connection status to disconnected when WebSocket closes', async () => {
      mockReadyState = ReadyState.CLOSED;

      renderHook(() => useDaemonWebSocket());

      await waitFor(() => {
        expect(useDashboardStore.getState().connectionStatus).toBe('disconnected');
      });
    });

    it('should update store connection status to disconnected when WebSocket is uninstantiated', async () => {
      mockReadyState = ReadyState.UNINSTANTIATED;

      renderHook(() => useDaemonWebSocket());

      await waitFor(() => {
        expect(useDashboardStore.getState().connectionStatus).toBe('disconnected');
      });
    });

    it('should provide correct isConnected flag', () => {
      mockReadyState = ReadyState.OPEN;
      const { result } = renderHook(() => useDaemonWebSocket());

      expect(result.current.isConnected).toBe(true);
    });

    it('should provide correct isConnecting flag', () => {
      mockReadyState = ReadyState.CONNECTING;
      const { result } = renderHook(() => useDaemonWebSocket());

      expect(result.current.isConnecting).toBe(true);
    });

    it('should provide correct isDisconnected flag', () => {
      mockReadyState = ReadyState.CLOSED;
      const { result } = renderHook(() => useDaemonWebSocket());

      expect(result.current.isDisconnected).toBe(true);
    });
  });

  describe('message parsing - state updates', () => {
    it('should parse and update store with state message', async () => {
      renderHook(() => useDaemonWebSocket());

      const statePayload: DaemonState = {
        modifiers: ['MD_00', 'MD_01'],
        locks: ['LK_00'],
        layer: 'function',
      };

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'state', payload: statePayload }),
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        const state = useDashboardStore.getState();
        expect(state.currentState).toEqual(statePayload);
      });
    });
  });

  describe('message parsing - key events', () => {
    it('should parse and add key event to store', async () => {
      renderHook(() => useDaemonWebSocket());

      const eventPayload: KeyEvent = {
        timestamp: 1234567890,
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'a',
        output: 'A',
        latency: 120,
      };

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'event', payload: eventPayload }),
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        const state = useDashboardStore.getState();
        expect(state.events).toHaveLength(1);
        expect(state.events[0]).toEqual(eventPayload);
      });
    });

    it('should add multiple events in sequence', async () => {
      renderHook(() => useDaemonWebSocket());

      const event1: KeyEvent = {
        timestamp: 1000,
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'a',
        output: 'A',
        latency: 100,
      };

      const event2: KeyEvent = {
        timestamp: 2000,
        keyCode: 'KEY_A',
        eventType: 'release',
        input: 'a',
        output: 'A',
        latency: 110,
      };

      act(() => {
        mockOnMessage?.(
          new MessageEvent('message', {
            data: JSON.stringify({ type: 'event', payload: event1 }),
          })
        );
        mockOnMessage?.(
          new MessageEvent('message', {
            data: JSON.stringify({ type: 'event', payload: event2 }),
          })
        );
      });

      await waitFor(() => {
        const state = useDashboardStore.getState();
        expect(state.events).toHaveLength(2);
        expect(state.events[0]).toEqual(event1);
        expect(state.events[1]).toEqual(event2);
      });
    });
  });

  describe('message parsing - latency statistics', () => {
    it('should parse and update metrics in store', async () => {
      renderHook(() => useDaemonWebSocket());

      const latencyPayload: LatencyStats = {
        min: 50,
        avg: 120,
        max: 250,
        p95: 180,
        p99: 230,
        timestamp: 1234567890,
      };

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'latency', payload: latencyPayload }),
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        const state = useDashboardStore.getState();
        expect(state.metrics).toEqual(latencyPayload);
      });
    });
  });

  describe('message parsing - special messages', () => {
    it('should handle connected message without error', async () => {
      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

      renderHook(() => useDaemonWebSocket());

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'connected', payload: { version: '1.0.0' } }),
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith(
          'WebSocket connected to daemon',
          expect.any(Object)
        );
      });

      consoleSpy.mockRestore();
    });

    it('should handle heartbeat message silently', async () => {
      renderHook(() => useDaemonWebSocket());

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'heartbeat', payload: {} }),
      });

      // Should not throw or log errors
      act(() => {
        mockOnMessage?.(message);
      });

      // No assertions needed - test passes if no errors thrown
    });
  });

  describe('error handling', () => {
    it('should handle malformed JSON gracefully', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      renderHook(() => useDaemonWebSocket());

      const message = new MessageEvent('message', {
        data: 'not valid json',
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          'Failed to parse WebSocket message:',
          expect.any(Error)
        );
      });

      consoleErrorSpy.mockRestore();
    });

    it('should warn about unknown message types', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      renderHook(() => useDaemonWebSocket());

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'unknown', payload: {} }),
      });

      act(() => {
        mockOnMessage?.(message);
      });

      await waitFor(() => {
        expect(consoleWarnSpy).toHaveBeenCalledWith(
          'Unknown WebSocket message type:',
          expect.any(Object)
        );
      });

      consoleWarnSpy.mockRestore();
    });

    it('should not crash on null payload', async () => {
      renderHook(() => useDaemonWebSocket());

      const message = new MessageEvent('message', {
        data: JSON.stringify({ type: 'state', payload: null }),
      });

      // Should not throw
      act(() => {
        mockOnMessage?.(message);
      });

      // Test passes if no error thrown
    });
  });

  describe('sendMessage function', () => {
    it('should expose sendMessage function', () => {
      const { result } = renderHook(() => useDaemonWebSocket());

      expect(result.current.sendMessage).toBeDefined();
      expect(typeof result.current.sendMessage).toBe('function');
    });

    it('should call WebSocket sendMessage when invoked', () => {
      const { result } = renderHook(() => useDaemonWebSocket());

      act(() => {
        result.current.sendMessage('test message');
      });

      expect(mockSendMessage).toHaveBeenCalledWith('test message');
    });
  });

  describe('readyState exposure', () => {
    it('should expose WebSocket readyState', () => {
      mockReadyState = ReadyState.OPEN;
      const { result } = renderHook(() => useDaemonWebSocket());

      expect(result.current.readyState).toBe(ReadyState.OPEN);
    });

    it('should update readyState when connection state changes', () => {
      mockReadyState = ReadyState.CONNECTING;
      const { result, rerender } = renderHook(() => useDaemonWebSocket());

      expect(result.current.readyState).toBe(ReadyState.CONNECTING);

      // Simulate state change
      mockReadyState = ReadyState.OPEN;
      vi.mocked(useWebSocket).mockImplementation((url, options) => {
        mockOnMessage = options?.onMessage || null;
        return {
          sendMessage: mockSendMessage,
          lastMessage: null,
          readyState: ReadyState.OPEN,
        } as any;
      });

      rerender();

      expect(result.current.readyState).toBe(ReadyState.OPEN);
    });
  });
});
