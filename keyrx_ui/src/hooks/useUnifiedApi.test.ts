/**
 * useUnifiedApi - Hook Tests
 *
 * Tests the core API methods: query, command, subscribe
 * Uses jest-websocket-mock for WebSocket simulation.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useUnifiedApi } from './useUnifiedApi';
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
  sendRpcResponse,
  sendRpcError,
  sendServerMessage,
  waitForRpcRequest,
  WS_URL,
} from '../../tests/helpers/websocket';

describe('useUnifiedApi - Core API Tests', () => {
  beforeEach(async () => {
    // Setup mock WebSocket server at the URL the hook will use
    await setupMockWebSocket(WS_URL);
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  it('query sends RPC request and resolves with result', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Wait for connection
    await simulateConnected('test-session');

    // Call query
    const queryPromise = result.current.query('get_profiles');

    // Wait for the request to be sent and respond
    const request = await waitForRpcRequest('get_profiles');
    sendRpcResponse(request.id, { profiles: ['default', 'gaming'] });

    // Verify result
    const profiles = await queryPromise;
    expect(profiles).toEqual({ profiles: ['default', 'gaming'] });
  });

  it('query rejects on error response', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    await simulateConnected('test-session');

    const queryPromise = result.current.query('get_profiles');

    const request = await waitForRpcRequest('get_profiles');
    sendRpcError(request.id, -32600, 'Invalid request');

    await expect(queryPromise).rejects.toThrow('Invalid request');
  });

  it('command sends RPC request and resolves with result', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    await simulateConnected('test-session');

    const commandPromise = result.current.command('activate_profile', {
      name: 'gaming',
    });

    const request = await waitForRpcRequest('activate_profile');
    expect(request.params).toEqual({ name: 'gaming' });
    sendRpcResponse(request.id, { success: true });

    const response = await commandPromise;
    expect(response).toEqual({ success: true });
  });

  it('subscribe receives events from server', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    await simulateConnected('test-session');

    const events: unknown[] = [];
    result.current.subscribe('daemon-state', (data) => {
      events.push(data);
    });

    // Send state update event
    sendServerMessage({
      type: 'event',
      channel: 'daemon-state',
      data: { running: true, activeProfile: 'gaming' },
    });

    await waitFor(() => {
      expect(events).toContainEqual({ running: true, activeProfile: 'gaming' });
    });
  });

  it('isConnected reflects WebSocket connection state', async () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Initially not connected
    expect(result.current.isConnected).toBe(false);

    await simulateConnected('test-session');

    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
    });
  });

  it('readyState reflects WebSocket state', () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Should be a valid ReadyState number (0=CONNECTING, 1=OPEN, 2=CLOSING, 3=CLOSED)
    expect(result.current.readyState).toBeGreaterThanOrEqual(0);
    expect(result.current.readyState).toBeLessThanOrEqual(3);
  });

  it('returns unsubscribe function from subscribe', () => {
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    const unsubscribe = result.current.subscribe('daemon-state', () => {});
    expect(typeof unsubscribe).toBe('function');
    unsubscribe();
  });

  it('query rejects when WebSocket is not connected', async () => {
    // Don't connect - keep WebSocket in disconnected state
    const { result } = renderHook(() => useUnifiedApi(WS_URL));

    // Need to wait for the hook to initialize
    await waitFor(() => {
      // result.current should be available
      expect(result.current).toBeDefined();
    });

    // Query should reject since not connected
    await expect(result.current.query('test')).rejects.toThrow(
      'WebSocket is not connected'
    );
  });
});
