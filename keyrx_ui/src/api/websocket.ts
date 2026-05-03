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
import { buildWsUrl, WS_RECONNECT_CONFIG } from '../config/constants';

/**
 * Reconnection Scheduler with exponential backoff
 *
 * Handles scheduling reconnection attempts with configurable
 * backoff intervals and max retry limits.
 */
export class ReconnectionScheduler {
  private attempts = 0;
  private timeoutId: number | null = null;
  private readonly intervals: readonly number[];
  private readonly maxInterval: number;
  private readonly maxRetries: number;
  private onSchedule: (delay: number, attempt: () => void) => void;

  constructor(
    onSchedule: (delay: number, attempt: () => void) => void,
    intervals: readonly number[] = [100, 200, 400, 800, 1600],
    maxInterval = 5000,
    maxRetries = WS_RECONNECT_CONFIG.maxRetries
  ) {
    this.onSchedule = onSchedule;
    this.intervals = intervals;
    this.maxInterval = maxInterval;
    this.maxRetries = maxRetries;
  }

  /**
   * Schedule next reconnection attempt with exponential backoff
   */
  schedule(attempt: () => void): boolean {
    if (this.attempts >= this.maxRetries) {
      return false;
    }

    const delay =
      this.attempts < this.intervals.length
        ? this.intervals[this.attempts]
        : this.maxInterval;

    this.timeoutId = window.setTimeout(() => {
      this.attempts++;
      attempt();
    }, delay);

    return true;
  }

  /**
   * Clear any pending reconnection timeout
   */
  clear(): void {
    if (this.timeoutId !== null) {
      window.clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }
  }

  /**
   * Reset scheduler state (call on successful connect)
   */
  reset(): void {
    this.clear();
    this.attempts = 0;
  }

  /**
   * Check if max retries reached
   */
  isExhausted(): boolean {
    return this.attempts >= this.maxRetries;
  }

  get attemptCount(): number {
    return this.attempts;
  }
}

/**
 * Message parser for WebSocket messages
 *
 * Handles parsing and transforming daemon messages to frontend types.
 */
export class MessageParser {
  /**
   * Transform daemon's KeyEventPayload to frontend's EventRecord
   */
  transformKeyEvent(payload: KeyEventPayload): EventRecord {
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

  /**
   * Parse and dispatch message to appropriate callback
   */
  parse(
    event: MessageEvent,
    callbacks: {
      onEvent?: (event: EventRecord) => void;
      onState?: (state: DaemonState) => void;
      onLatency?: (stats: LatencyStats) => void;
    }
  ): void {
    try {
      const message: WSMessage = JSON.parse(event.data);

      switch (message.type) {
        case 'event':
          if (callbacks.onEvent) {
            callbacks.onEvent(this.transformKeyEvent(message.payload));
          }
          break;

        case 'state':
          if (callbacks.onState) {
            callbacks.onState(message.payload);
          }
          break;

        case 'latency':
          if (callbacks.onLatency) {
            callbacks.onLatency(message.payload);
          }
          break;

        case 'error':
          break;

        default:
        // Unknown message type received
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }
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
  private ws: WebSocket | null = null;
  private config: Required<WebSocketConfig>;
  private callbacks: WebSocketCallbacks;
  private connectionState: ConnectionState = 'disconnected';
  private isClosed = false;
  private reconnectScheduler: ReconnectionScheduler;
  private messageParser: MessageParser;

  constructor(
    config: WebSocketConfig = {},
    callbacks: WebSocketCallbacks = {}
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.callbacks = callbacks;

    // Initialize reconnection scheduler with exponential backoff
    this.reconnectScheduler = new ReconnectionScheduler(
      (delay, attempt) => {
        window.setTimeout(attempt, delay);
      },
      [100, 200, 400, 800, 1600],
      5000,
      this.config.maxReconnectAttempts
    );

    // Initialize message parser
    this.messageParser = new MessageParser();

    // Compute WebSocket URL if not provided
    if (!this.config.url) {
      this.config.url = buildWsUrl();
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
    this.reconnectScheduler.clear();

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
      console.warn('Cannot send message: WebSocket is not connected');
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
    this.reconnectScheduler.reset();
    this.setConnectionState('connected');

    if (this.callbacks.onOpen) {
      this.callbacks.onOpen();
    }
  }

  /**
   * Handle WebSocket message event
   */
  private handleMessage(event: MessageEvent): void {
    this.messageParser.parse(event, {
      onEvent: this.callbacks.onEvent,
      onState: this.callbacks.onState,
      onLatency: this.callbacks.onLatency,
    });
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
