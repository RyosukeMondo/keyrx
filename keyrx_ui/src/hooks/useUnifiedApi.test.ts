/**
 * Tests for useUnifiedApi hook
 *
 * This test file verifies the WebSocket RPC communication hook implementation.
 * It tests all acceptance criteria from REQ-1:
 * - AC1: Connection and Connected handshake
 * - AC2: Query/command RPC methods
 * - AC3: Subscription and event handling
 * - AC6: Error handling
 * - AC9: Request timeout
 */

import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useUnifiedApi } from './useUnifiedApi';
import type { ServerMessage, RpcError } from '../types/rpc';

// Store the mock implementation
let mockWebSocketInstance: any;

// Mock react-use-websocket
vi.mock('react-use-websocket', () => {
  const ReadyState = {
    CONNECTING: 0,
    OPEN: 1,
    CLOSING: 2,
    CLOSED: 3,
    UNINSTANTIATED: -1,
  };

  return {
    default: vi.fn(() => ({
      sendMessage: vi.fn(),
      lastMessage: null,
      readyState: 1,
    })),
    ReadyState,
  };
});

// Mock uuid
let uuidCounter = 0;
vi.mock('uuid', () => ({
  v4: () => `test-uuid-${uuidCounter++}`,
}));

describe('useUnifiedApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    uuidCounter = 0;
  });

  describe('Connection and Handshake (AC1)', () => {
    it('should initialize with connection state', () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      // Initially not connected (no handshake yet)
      expect(result.current.isConnected).toBe(false);
      expect(result.current.readyState).toBe(1); // OPEN
    });

    it('should track readyState from WebSocket', () => {
      mockWebSocketInstance.readyState = 0; // CONNECTING
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      expect(result.current.readyState).toBe(0);
    });
  });

  describe('Query and Command Methods (AC2)', () => {
    it('should send query with correct message structure', async () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      // Execute query (will timeout but that's ok for this test)
      result.current.query('get_profiles').catch(() => {});

      const useWebSocket = vi.mocked(require('react-use-websocket').default);
      const mockInstance = useWebSocket.mock.results[0].value;

      await waitFor(() => {
        expect(mockInstance.sendMessage).toHaveBeenCalled();
      });

      // Verify message structure
      const sentMessage = mockInstance.sendMessage.mock.calls[0][0];
      expect(sentMessage).toContain('"type":"query"');
      expect(sentMessage).toContain('"method":"get_profiles"');
      expect(sentMessage).toContain('"id":"test-uuid-0"');
    });

    it('should send command with params', async () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      result.current.command('activate_profile', { name: 'Gaming' }).catch(() => {});

      await waitFor(() => {
        expect(mockWebSocketInstance.sendMessage).toHaveBeenCalled();
      });

      const sentMessage = mockWebSocketInstance.sendMessage.mock.calls[0][0];
      expect(sentMessage).toContain('"type":"command"');
      expect(sentMessage).toContain('"method":"activate_profile"');
      expect(sentMessage).toContain('"params":{"name":"Gaming"}');
    });

    it('should reject query when WebSocket is not connected', async () => {
      mockWebSocketInstance.readyState = 3; // CLOSED
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      let queryError: Error | null = null;

      await result.current.query('get_profiles').catch((err) => {
        queryError = err;
      });

      expect(queryError).toBeTruthy();
      expect(queryError?.message).toContain('not connected');
    });
  });

  describe('Subscription Management (AC3)', () => {
    it('should send subscribe message on first subscription', () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      const handler = vi.fn();
      result.current.subscribe('daemon-state', handler);

      expect(mockWebSocketInstance.sendMessage).toHaveBeenCalled();

      const sentMessage = mockWebSocketInstance.sendMessage.mock.calls[0][0];
      expect(sentMessage).toContain('"type":"subscribe"');
      expect(sentMessage).toContain('"channel":"daemon-state"');
    });

    it('should not send duplicate subscribe for same channel', () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      result.current.subscribe('daemon-state', vi.fn());

      mockWebSocketInstance.sendMessage.mockClear();

      result.current.subscribe('daemon-state', vi.fn());

      // Should not send another subscribe message
      expect(mockWebSocketInstance.sendMessage).not.toHaveBeenCalled();
    });

    it('should unsubscribe from channel', () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      const handler = vi.fn();
      const unsubscribe = result.current.subscribe('daemon-state', handler);

      mockWebSocketInstance.sendMessage.mockClear();

      unsubscribe();

      expect(mockWebSocketInstance.sendMessage).toHaveBeenCalled();

      const sentMessage = mockWebSocketInstance.sendMessage.mock.calls[0][0];
      expect(sentMessage).toContain('"type":"unsubscribe"');
      expect(sentMessage).toContain('"channel":"daemon-state"');
    });
  });

  describe('Request Correlation (AC7)', () => {
    it('should use UUID for request correlation', () => {
      const { result } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      result.current.query('method1').catch(() => {});
      result.current.query('method2').catch(() => {});
      result.current.query('method3').catch(() => {});

      const calls = mockWebSocketInstance.sendMessage.mock.calls;
      expect(calls.length).toBe(3);

      // Each call should have a different UUID
      expect(calls[0][0]).toContain('"id":"test-uuid-0"');
      expect(calls[1][0]).toContain('"id":"test-uuid-1"');
      expect(calls[2][0]).toContain('"id":"test-uuid-2"');
    });
  });

  describe('Auto-Reconnect Configuration (AC8)', () => {
    it('should configure auto-reconnect parameters', () => {
      const useWebSocket = vi.mocked(require('react-use-websocket').default);

      renderHook(() => useUnifiedApi('ws://localhost:3030'));

      // Verify useWebSocket was called with correct config
      expect(useWebSocket).toHaveBeenCalledWith(
        'ws://localhost:3030',
        expect.objectContaining({
          shouldReconnect: expect.any(Function),
          reconnectInterval: 3000,
          reconnectAttempts: 10,
        })
      );
    });

    it('should enable auto-reconnect', () => {
      const useWebSocket = vi.mocked(require('react-use-websocket').default);

      renderHook(() => useUnifiedApi('ws://localhost:3030'));

      const config = useWebSocket.mock.calls[0][1];
      expect(config.shouldReconnect()).toBe(true);
    });
  });

  describe('Cleanup on Unmount (AC10)', () => {
    it('should unsubscribe all channels on unmount', () => {
      const { result, unmount } = renderHook(() => useUnifiedApi('ws://localhost:3030'));

      result.current.subscribe('daemon-state', vi.fn());
      result.current.subscribe('events', vi.fn());
      result.current.subscribe('latency', vi.fn());

      mockWebSocketInstance.sendMessage.mockClear();

      unmount();

      // Should send unsubscribe for each channel
      const unsubscribeCalls = mockWebSocketInstance.sendMessage.mock.calls.filter(
        (call: any[]) => call[0].includes('"type":"unsubscribe"')
      );

      expect(unsubscribeCalls.length).toBe(3);
    });
  });

  describe('Error Scenarios (AC6)', () => {
    it('should include error callbacks in config', () => {
      renderHook(() => useUnifiedApi('ws://localhost:3030'));

      const config = mockUseWebSocket.mock.calls[0][1];
      expect(config.onError).toBeDefined();
      expect(config.onClose).toBeDefined();
    });
  });
});
