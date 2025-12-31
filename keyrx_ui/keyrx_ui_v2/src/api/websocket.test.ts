import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { WebSocketManager, getWebSocketInstance, closeWebSocketInstance } from './websocket';
import type { EventRecord, DaemonState, LatencyStats } from '../types';

// Mock WebSocket
class MockWebSocket {
  public readyState: number = WebSocket.CONNECTING;
  public onopen: ((event: Event) => void) | null = null;
  public onclose: ((event: CloseEvent) => void) | null = null;
  public onerror: ((event: Event) => void) | null = null;
  public onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(public url: string) {
    // Simulate async connection
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 10);
  }

  send(data: string): void {
    if (this.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not open');
    }
  }

  close(): void {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }
}

// Mock global WebSocket
global.WebSocket = MockWebSocket as unknown as typeof WebSocket;

describe('WebSocketManager', () => {
  let wsManager: WebSocketManager;

  beforeEach(() => {
    vi.clearAllTimers();
    vi.useFakeTimers();
  });

  afterEach(() => {
    if (wsManager) {
      wsManager.close();
    }
    vi.restoreAllMocks();
  });

  describe('Connection', () => {
    it('should connect to WebSocket server', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({}, { onOpen });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      expect(onOpen).toHaveBeenCalled();
      expect(wsManager.isConnected()).toBe(true);
      expect(wsManager.getConnectionState()).toBe('connected');
    });

    it('should not create duplicate connections', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({}, { onOpen });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      // Try to connect again
      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      // Should only call onOpen once
      expect(onOpen).toHaveBeenCalledTimes(1);
    });

    it('should disconnect cleanly', async () => {
      const onClose = vi.fn();
      wsManager = new WebSocketManager({}, { onClose });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      wsManager.disconnect();

      expect(onClose).toHaveBeenCalled();
      expect(wsManager.isConnected()).toBe(false);
      expect(wsManager.getConnectionState()).toBe('disconnected');
    });

    it('should not reconnect after close()', async () => {
      const onOpen = vi.fn();
      wsManager = new WebSocketManager({}, { onOpen });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      wsManager.close();

      // Try to reconnect
      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      // Should only connect once
      expect(onOpen).toHaveBeenCalledTimes(1);
    });
  });

  describe('Message Handling', () => {
    it('should handle event messages', async () => {
      const onEvent = vi.fn();
      wsManager = new WebSocketManager({}, { onEvent });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      // Simulate incoming event message
      const mockEvent: EventRecord = {
        id: '1',
        timestamp: '2024-01-01T00:00:00Z',
        type: 'key_press',
        keyCode: 'KEY_A',
        layer: 'base',
        latencyUs: 100,
      };

      const messageEvent = new MessageEvent('message', {
        data: JSON.stringify({
          type: 'event',
          payload: mockEvent,
        }),
      });

      // Access internal ws to trigger onmessage
      const ws = (wsManager as any).ws as MockWebSocket;
      ws.onmessage!(messageEvent);

      expect(onEvent).toHaveBeenCalledWith(mockEvent);
    });

    it('should handle state messages', async () => {
      const onState = vi.fn();
      wsManager = new WebSocketManager({}, { onState });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      const mockState: DaemonState = {
        activeLayer: 'base',
        modifiers: ['shift'],
        locks: [],
        tapHoldPending: false,
        uptime: 1000,
      };

      const messageEvent = new MessageEvent('message', {
        data: JSON.stringify({
          type: 'state',
          payload: mockState,
        }),
      });

      const ws = (wsManager as any).ws as MockWebSocket;
      ws.onmessage!(messageEvent);

      expect(onState).toHaveBeenCalledWith(mockState);
    });

    it('should handle latency messages', async () => {
      const onLatency = vi.fn();
      wsManager = new WebSocketManager({}, { onLatency });

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

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

      const messageEvent = new MessageEvent('message', {
        data: JSON.stringify({
          type: 'latency',
          payload: mockLatency,
        }),
      });

      const ws = (wsManager as any).ws as MockWebSocket;
      ws.onmessage!(messageEvent);

      expect(onLatency).toHaveBeenCalledWith(mockLatency);
    });

    it('should handle invalid JSON gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      wsManager = new WebSocketManager({});

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      const messageEvent = new MessageEvent('message', {
        data: 'invalid json',
      });

      const ws = (wsManager as any).ws as MockWebSocket;
      ws.onmessage!(messageEvent);

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('Reconnection', () => {
    it('should schedule reconnection after disconnect', async () => {
      const onConnectionStateChange = vi.fn();

      wsManager = new WebSocketManager(
        {
          reconnectInterval: 1000,
          reconnectDecay: 2,
          maxReconnectInterval: 10000,
        },
        { onConnectionStateChange }
      );

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      // Close the connection
      const ws = (wsManager as any).ws as MockWebSocket;
      ws.close();

      // Should have transitioned to disconnected
      expect(onConnectionStateChange).toHaveBeenCalledWith('disconnected');

      // Verify reconnection timeout is scheduled (check internal state)
      const reconnectTimeoutId = (wsManager as any).reconnectTimeoutId;
      expect(reconnectTimeoutId).not.toBeNull();
    });

    it('should stop reconnecting after max attempts', async () => {
      const onConnectionStateChange = vi.fn();
      const states: string[] = [];
      wsManager = new WebSocketManager(
        {
          maxReconnectAttempts: 2,
          reconnectInterval: 100,
        },
        {
          onConnectionStateChange: (state) => {
            states.push(state);
            onConnectionStateChange(state);
          }
        }
      );

      // Mock WebSocket to fail immediately
      const OriginalWebSocket = global.WebSocket;
      global.WebSocket = class {
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

      wsManager.connect();

      // First attempt fails
      await vi.advanceTimersByTimeAsync(10);

      // Wait for first reconnect
      await vi.advanceTimersByTimeAsync(110);

      // Wait for second reconnect
      await vi.advanceTimersByTimeAsync(200);

      // Should reach max attempts
      expect(states).toContain('error');

      global.WebSocket = OriginalWebSocket;
    });

    it('should not reconnect if disabled', async () => {
      const onConnectionStateChange = vi.fn();
      wsManager = new WebSocketManager(
        { reconnect: false },
        { onConnectionStateChange }
      );

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      const ws = (wsManager as any).ws as MockWebSocket;
      ws.close();

      // Reset the spy to only count reconnection attempts
      onConnectionStateChange.mockClear();

      // Wait for potential reconnect
      await vi.advanceTimersByTimeAsync(5000);

      // Should not attempt to reconnect (no new 'connecting' state)
      const connectingStates = onConnectionStateChange.mock.calls.filter(
        call => call[0] === 'connecting'
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

      wsManager = new WebSocketManager({}, { onConnectionStateChange, onOpen });

      wsManager.connect();
      expect(states).toContain('connecting');

      await vi.advanceTimersByTimeAsync(20);

      // Wait for onOpen to be called to ensure connection is established
      expect(onOpen).toHaveBeenCalled();
      expect(states).toContain('connected');

      wsManager.disconnect();
      expect(states).toContain('disconnected');
    });
  });

  describe('Send Messages', () => {
    it('should send string messages', async () => {
      wsManager = new WebSocketManager({});

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      const ws = (wsManager as any).ws as MockWebSocket;
      const sendSpy = vi.spyOn(ws, 'send');

      wsManager.send('test message');

      expect(sendSpy).toHaveBeenCalledWith('test message');
    });

    it('should send object messages as JSON', async () => {
      wsManager = new WebSocketManager({});

      wsManager.connect();
      await vi.advanceTimersByTimeAsync(20);

      const ws = (wsManager as any).ws as MockWebSocket;
      const sendSpy = vi.spyOn(ws, 'send');

      const message = { type: 'test', data: 'value' };
      wsManager.send(message);

      expect(sendSpy).toHaveBeenCalledWith(JSON.stringify(message));
    });

    it('should not send when disconnected', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      wsManager = new WebSocketManager({});

      wsManager.send('test');

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Cannot send message')
      );
      consoleSpy.mockRestore();
    });
  });
});
