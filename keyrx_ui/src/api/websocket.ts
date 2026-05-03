/**
 * WebSocket Connection Manager
 *
 * Provides robust WebSocket connection management with:
 * - Automatic reconnection with exponential backoff
 * - Connection state monitoring
 * - Type-safe message handling
 * - Resource cleanup
 *
 * Responsibilities are delegated to specialized modules:
 * - WebSocketConnectionManager: raw WebSocket lifecycle
 * - ReconnectionScheduler: reconnection scheduling
 * - MessageParser: message parsing and transformation
 */

import type { EventRecord, DaemonState, LatencyStats } from '../types';
import { buildWsUrl, WS_RECONNECT_CONFIG } from '../config/constants';
import { WebSocketConnectionManager } from './ws-connection';
import { ReconnectionScheduler } from './ws-reconnection';
import { MessageParser } from './ws-parser';

// Re-export ConnectionState from ws-connection to avoid duplication
export type { ConnectionState } from './ws-connection';

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

// Re-export specialized classes for testing
export { WebSocketConnectionManager } from './ws-connection';
export { ReconnectionScheduler } from './ws-reconnection';
export { MessageParser } from './ws-parser';

// Use configuration from central constants
const DEFAULT_CONFIG: Required<WebSocketConfig> = {
  url: '', // Will be computed from buildWsUrl()
  reconnect: true,
  reconnectInterval: WS_RECONNECT_CONFIG.initialDelayMs,
  maxReconnectInterval: WS_RECONNECT_CONFIG.maxDelayMs,
  reconnectDecay: WS_RECONNECT_CONFIG.backoffMultiplier,
  maxReconnectAttempts: WS_RECONNECT_CONFIG.maxRetries,
};

export class WebSocketManager {
  private connectionManager: WebSocketConnectionManager;
  private reconnectScheduler: ReconnectionScheduler;
  private messageParser: MessageParser;
  private config: Required<WebSocketConfig>;
  private callbacks: WebSocketCallbacks;
  private connectionState: ConnectionState = 'disconnected';
  private isClosed = false;

  constructor(
    config: WebSocketConfig = {},
    callbacks: WebSocketCallbacks = {}
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.callbacks = callbacks;

    // Initialize message parser
    this.messageParser = new MessageParser();

    // Initialize reconnection scheduler with exponential backoff
    this.reconnectScheduler = new ReconnectionScheduler(
      (delay, attempt) => {
        window.setTimeout(attempt, delay);
      },
      [100, 200, 400, 800, 1600],
      5000,
      this.config.maxReconnectAttempts
    );

    // Initialize connection manager
    this.connectionManager = new WebSocketConnectionManager(
      { url: this.config.url || buildWsUrl() },
      {
        onOpen: () => this.handleOpen(),
        onClose: () => this.handleClose(),
        onError: (error) => this.handleError(error),
        onEvent: (event) => {
          if (this.callbacks.onEvent) {
            this.callbacks.onEvent(event);
          }
        },
        onState: (state) => {
          if (this.callbacks.onState) {
            this.callbacks.onState(state);
          }
        },
        onLatency: (stats) => {
          if (this.callbacks.onLatency) {
            this.callbacks.onLatency(stats);
          }
        },
        onConnectionStateChange: (state) => this.setConnectionState(state),
      }
    );
  }

  /**
   * Connect to the WebSocket server
   */
  public connect(): void {
    if (this.isClosed) {
      return;
    }

    if (this.connectionManager.isConnected()) {
      return;
    }

    this.connectionManager.connect();
  }

  /**
   * Disconnect from the WebSocket server
   */
  public disconnect(): void {
    this.reconnectScheduler.clear();
    this.connectionManager.disconnect();
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
    this.connectionManager.send(data);
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
    this.reconnectScheduler.reset();
    this.setConnectionState('connected');

    if (this.callbacks.onOpen) {
      this.callbacks.onOpen();
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
   * Schedule a reconnection attempt with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.isClosed) {
      return;
    }

    const scheduled = this.reconnectScheduler.schedule(() => {
      this.connect();
    });

    if (!scheduled) {
      this.setConnectionState('error');
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
