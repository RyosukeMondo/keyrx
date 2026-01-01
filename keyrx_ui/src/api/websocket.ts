/**
 * WebSocket Connection Manager
 *
 * Provides robust WebSocket connection management with:
 * - Automatic reconnection with exponential backoff
 * - Connection state monitoring
 * - Type-safe message handling
 * - Resource cleanup
 */

import type { WSMessage, EventRecord, DaemonState, LatencyStats } from '../types';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

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

const DEFAULT_CONFIG: Required<WebSocketConfig> = {
  url: '', // Will be computed from window.location
  reconnect: true,
  reconnectInterval: 1000, // Start at 1 second
  maxReconnectInterval: 30000, // Max 30 seconds
  reconnectDecay: 1.5, // Exponential backoff multiplier
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

  constructor(config: WebSocketConfig = {}, callbacks: WebSocketCallbacks = {}) {
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
      console.warn('WebSocketManager: Cannot connect after close() was called');
      return;
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      console.warn('WebSocketManager: Already connected');
      return;
    }

    if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
      console.warn('WebSocketManager: Connection already in progress');
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
      console.error('WebSocketManager: Failed to create WebSocket:', error);
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
      console.warn('WebSocketManager: Cannot send message, not connected');
      return;
    }

    try {
      const message = typeof data === 'string' ? data : JSON.stringify(data);
      this.ws.send(message);
    } catch (error) {
      console.error('WebSocketManager: Failed to send message:', error);
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
    console.log('WebSocketManager: Connected');
    this.reconnectAttempts = 0;
    this.currentReconnectInterval = this.config.reconnectInterval;
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
            this.callbacks.onEvent(message.payload as EventRecord);
          }
          break;

        case 'state':
          if (this.callbacks.onState) {
            this.callbacks.onState(message.payload as DaemonState);
          }
          break;

        case 'latency':
          if (this.callbacks.onLatency) {
            this.callbacks.onLatency(message.payload as LatencyStats);
          }
          break;

        case 'error':
          console.error('WebSocketManager: Server error:', message.payload);
          break;

        default:
          console.warn('WebSocketManager: Unknown message type:', message);
      }
    } catch (error) {
      console.error('WebSocketManager: Failed to parse message:', error);
    }
  }

  /**
   * Handle WebSocket error event
   */
  private handleError(error: Event): void {
    console.error('WebSocketManager: Connection error:', error);
    this.setConnectionState('error');

    if (this.callbacks.onError) {
      this.callbacks.onError(error);
    }
  }

  /**
   * Handle WebSocket close event
   */
  private handleClose(): void {
    console.log('WebSocketManager: Connection closed');
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
   * Schedule a reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.isClosed) {
      return;
    }

    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      console.error(
        'WebSocketManager: Max reconnection attempts reached, giving up'
      );
      this.setConnectionState('error');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.currentReconnectInterval;

    console.log(
      `WebSocketManager: Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.config.maxReconnectAttempts})`
    );

    this.reconnectTimeoutId = window.setTimeout(() => {
      this.connect();
    }, delay);

    // Exponential backoff
    this.currentReconnectInterval = Math.min(
      this.currentReconnectInterval * this.config.reconnectDecay,
      this.config.maxReconnectInterval
    );
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
