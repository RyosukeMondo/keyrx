/**
 * WebSocket Connection Manager
 *
 * Manages the raw WebSocket connection lifecycle only.
 * Does not handle reconnection, parsing, or singleton management.
 */

import type { EventRecord, DaemonState, LatencyStats } from '../types';
import { buildWsUrl } from '../config/constants';
import { MessageParser } from './ws-parser';

export type ConnectionState =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'error';

export interface WebSocketConfig {
  url?: string;
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
};

export class WebSocketConnectionManager {
  private ws: WebSocket | null = null;
  private config: Required<WebSocketConfig>;
  private callbacks: WebSocketCallbacks;
  private connectionState: ConnectionState = 'disconnected';
  private isClosed = false;
  private parser = new MessageParser();

  constructor(
    config: WebSocketConfig = {},
    callbacks: WebSocketCallbacks = {}
  ) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.callbacks = callbacks;

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
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  public disconnect(): void {
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
    this.setConnectionState('connected');

    if (this.callbacks.onOpen) {
      this.callbacks.onOpen();
    }
  }

  /**
   * Handle WebSocket message event
   */
  private handleMessage(event: MessageEvent): void {
    this.parser.parse(event, {
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

  /**
   * Get the underlying WebSocket instance (for testing)
   */
  public getWs(): WebSocket | null {
    return this.ws;
  }
}
