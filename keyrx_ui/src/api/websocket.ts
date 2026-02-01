/**
 * WebSocket Connection Manager
 *
 * Provides robust WebSocket connection management with:
 * - Automatic reconnection with exponential backoff
 * - Connection state monitoring
 * - Type-safe message handling
 * - Resource cleanup
 */

import type {
  WSMessage,
  EventRecord,
  DaemonState,
  LatencyStats,
  KeyEventPayload,
} from '../types';

/**
 * Transform daemon's KeyEventPayload to frontend's EventRecord
 */
function transformKeyEvent(payload: KeyEventPayload): EventRecord {
  return {
    id: `evt-${payload.timestamp}-${Math.random().toString(36).slice(2, 8)}`,
    timestamp: new Date(payload.timestamp / 1000).toISOString(),
    type: payload.eventType === 'press' ? 'press' : 'release',
    keyCode: payload.keyCode.replace(/^KEY_/, ''),
    layer: 'Base',
    latencyUs: payload.latency,
    action: payload.mappingTriggered ? payload.output : undefined,
    input: payload.input,
    output: payload.output,
    deviceId: payload.deviceId,
    deviceName: payload.deviceName,
    mappingType: payload.mappingType,
    mappingTriggered: payload.mappingTriggered,
  };
}

export type ConnectionState =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'error';

export interface WebSocketConfig {
  url?: string;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectInterval?: number;
  reconnectDecay?: number;
  maxReconnectAttempts?: number;
}

export interface WebSocketCallbacks {
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  onEvent?: (event: EventRecord) => void;
  onState?: (state: DaemonState) => void;
  onLatency?: (stats: LatencyStats) => void;
  onConnectionStateChange?: (state: ConnectionState) => void;
}

// WS-002: Exponential backoff configuration
const RECONNECT_INTERVALS = [100, 200, 400, 800, 1600]; // ms
const MAX_RECONNECT_INTERVAL = 5000; // 5 seconds max

const DEFAULT_CONFIG: Required<WebSocketConfig> = {
  url: '', // Will be computed from window.location
  reconnect: true,
  reconnectInterval: 100, // Start at 100ms (WS-002)
  maxReconnectInterval: MAX_RECONNECT_INTERVAL,
  reconnectDecay: 2.0, // Double each time (WS-002)
  maxReconnectAttempts: 10,
};

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private config: Required<WebSocketConfig>;
  private callbacks: WebSocketCallbacks;
  private reconnectAttempts = 0;
  private reconnectTimeoutId: number | null = null;
  private currentReconnectInterval: number;
  private connectionState: ConnectionState = 'disconnected';
  private isClosed = false;

  constructor(
    config: WebSocketConfig = {},
    callbacks: WebSocketCallbacks = {}
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.callbacks = callbacks;
    this.currentReconnectInterval = this.config.reconnectInterval;

    // Compute WebSocket URL if not provided
    if (!this.config.url) {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.config.url = `${protocol}//${window.location.host}/ws`;
    }
  }

  /**
   * Connect to the WebSocket server
   */
  public connect(): void {
    if (this.isClosed) {
      return;
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return;
    }

    if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
      return;
    }

    this.setConnectionState('connecting');

    try {
      this.ws = new WebSocket(this.config.url);

      this.ws.onopen = () => {
        this.handleOpen();
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(event);
      };

      this.ws.onerror = (error) => {
        this.handleError(error);
      };

      this.ws.onclose = () => {
        this.handleClose();
      };
    } catch (error) {
      this.setConnectionState('error');
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  public disconnect(): void {
    this.clearReconnectTimeout();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.setConnectionState('disconnected');
  }

  /**
   * Close the WebSocket connection permanently (no reconnection)
   */
  public close(): void {
    this.isClosed = true;
    this.disconnect();
  }

  /**
   * Send a message to the server
   */
  public send(data: string | object): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      return;
    }

    try {
      const message = typeof data === 'string' ? data : JSON.stringify(data);
      this.ws.send(message);
    } catch (error) {
      // Silently handle send errors
    }
  }

  /**
   * Get current connection state
   */
  public getConnectionState(): ConnectionState {
    return this.connectionState;
  }

  /**
   * Check if connected
   */
  public isConnected(): boolean {
    return this.connectionState === 'connected';
  }

  /**
   * Handle WebSocket open event
   */
  private handleOpen(): void {
    // WS-002: Clear reconnection state on successful connect
    this.reconnectAttempts = 0;
    this.currentReconnectInterval = this.config.reconnectInterval;
    this.clearReconnectTimeout();
    this.setConnectionState('connected');

    if (this.callbacks.onOpen) {
      this.callbacks.onOpen();
    }
  }

  /**
   * Handle WebSocket message event
   */
  private handleMessage(event: MessageEvent): void {
    try {
      const message: WSMessage = JSON.parse(event.data);

      switch (message.type) {
        case 'event':
          if (this.callbacks.onEvent) {
            // Transform KeyEventPayload to EventRecord
            this.callbacks.onEvent(transformKeyEvent(message.payload));
          }
          break;

        case 'state':
          if (this.callbacks.onState) {
            this.callbacks.onState(message.payload);
          }
          break;

        case 'latency':
          if (this.callbacks.onLatency) {
            this.callbacks.onLatency(message.payload);
          }
          break;

        case 'error':
          // Server error received
          break;

        default:
          // Unknown message type received
      }
    } catch (error) {
      // Failed to parse message
    }
  }

  /**
   * Handle WebSocket error event
   */
  private handleError(error: Event): void {
    this.setConnectionState('error');

    if (this.callbacks.onError) {
      this.callbacks.onError(error);
    }
  }

  /**
   * Handle WebSocket close event
   */
  private handleClose(): void {
    this.setConnectionState('disconnected');

    if (this.callbacks.onClose) {
      this.callbacks.onClose();
    }

    // Attempt to reconnect if enabled and not manually closed
    if (this.config.reconnect && !this.isClosed) {
      this.scheduleReconnect();
    }
  }

  /**
   * Schedule a reconnection attempt (WS-002: with exponential backoff)
   */
  private scheduleReconnect(): void {
    if (this.isClosed) {
      return;
    }

    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.setConnectionState('error');
      return;
    }

    // WS-002: Use exponential backoff (100ms, 200ms, 400ms, 800ms, 1600ms, max 5s)
    const delay =
      this.reconnectAttempts < RECONNECT_INTERVALS.length
        ? RECONNECT_INTERVALS[this.reconnectAttempts]
        : MAX_RECONNECT_INTERVAL;

    this.reconnectTimeoutId = window.setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  /**
   * Clear reconnection timeout
   */
  private clearReconnectTimeout(): void {
    if (this.reconnectTimeoutId !== null) {
      window.clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
  }

  /**
   * Set connection state and notify callback
   */
  private setConnectionState(state: ConnectionState): void {
    if (this.connectionState !== state) {
      this.connectionState = state;

      if (this.callbacks.onConnectionStateChange) {
        this.callbacks.onConnectionStateChange(state);
      }
    }
  }
}

/**
 * Create and manage a singleton WebSocket connection
 */
let wsInstance: WebSocketManager | null = null;

export function getWebSocketInstance(
  config?: WebSocketConfig,
  callbacks?: WebSocketCallbacks
): WebSocketManager {
  if (!wsInstance) {
    wsInstance = new WebSocketManager(config, callbacks);
  }
  return wsInstance;
}

export function closeWebSocketInstance(): void {
  if (wsInstance) {
    wsInstance.close();
    wsInstance = null;
  }
}
