/**
 * useDaemonWebSocket Hook
 *
 * Manages WebSocket connection to the daemon and updates the dashboard store.
 * Provides auto-reconnect, message parsing, and connection state tracking.
 */

import { useEffect, useCallback } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';
import { useDashboardStore } from '../store/dashboardStore';
import type { DaemonState, KeyEvent, LatencyStats } from '../store/dashboardStore';

/**
 * WebSocket message types from daemon
 */
type WebSocketMessage =
  | { type: 'state'; payload: DaemonState }
  | { type: 'event'; payload: KeyEvent }
  | { type: 'latency'; payload: LatencyStats }
  | { type: 'connected'; payload: unknown }
  | { type: 'heartbeat'; payload: unknown };

/**
 * WebSocket connection options
 */
interface UseDaemonWebSocketOptions {
  /** WebSocket URL (default: ws://localhost:9867/ws) */
  url?: string;
  /** Enable auto-reconnect (default: true) */
  shouldReconnect?: boolean;
  /** Reconnect interval in milliseconds (default: 3000) */
  reconnectInterval?: number;
  /** Maximum reconnect attempts (default: Infinity) */
  reconnectAttempts?: number;
}

/**
 * Default WebSocket URL (daemon web server)
 */
const DEFAULT_WS_URL = 'ws://localhost:9867/ws';

/**
 * Hook for managing daemon WebSocket connection
 *
 * @param options - WebSocket connection options
 * @returns Connection ready state and send message function
 *
 * @example
 * ```tsx
 * function Dashboard() {
 *   const { readyState } = useDaemonWebSocket();
 *
 *   const connectionStatus = {
 *     [ReadyState.CONNECTING]: 'Connecting...',
 *     [ReadyState.OPEN]: 'Connected',
 *     [ReadyState.CLOSING]: 'Closing...',
 *     [ReadyState.CLOSED]: 'Disconnected',
 *     [ReadyState.UNINSTANTIATED]: 'Uninstantiated',
 *   }[readyState];
 *
 *   return <div>Status: {connectionStatus}</div>;
 * }
 * ```
 */
export function useDaemonWebSocket(options: UseDaemonWebSocketOptions = {}) {
  const {
    url = DEFAULT_WS_URL,
    shouldReconnect = true,
    reconnectInterval = 3000,
    reconnectAttempts = Infinity,
  } = options;

  // Store actions
  const { updateState, addEvent, updateMetrics, setConnectionStatus } = useDashboardStore();

  // Handle incoming WebSocket messages
  const handleMessage = useCallback(
    (event: MessageEvent) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'state':
            updateState(message.payload);
            break;

          case 'event':
            addEvent(message.payload);
            break;

          case 'latency':
            updateMetrics(message.payload);
            break;

          case 'connected':
            console.log('WebSocket connected to daemon', message.payload);
            break;

          case 'heartbeat':
            // Heartbeat received - connection is alive
            break;

          default:
            console.warn('Unknown WebSocket message type:', message);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    },
    [updateState, addEvent, updateMetrics]
  );

  // WebSocket connection with auto-reconnect
  const { sendMessage, lastMessage, readyState } = useWebSocket(url, {
    onMessage: handleMessage,
    shouldReconnect: () => shouldReconnect,
    reconnectInterval,
    reconnectAttempts,
    share: false, // Don't share connection across hook instances
  });

  // Update connection status in store based on ready state
  useEffect(() => {
    switch (readyState) {
      case ReadyState.CONNECTING:
        setConnectionStatus('connecting');
        break;
      case ReadyState.OPEN:
        setConnectionStatus('connected');
        break;
      case ReadyState.CLOSING:
      case ReadyState.CLOSED:
      case ReadyState.UNINSTANTIATED:
        setConnectionStatus('disconnected');
        break;
    }
  }, [readyState, setConnectionStatus]);

  return {
    /** WebSocket ready state */
    readyState,
    /** Send message to WebSocket server */
    sendMessage,
    /** Last received message */
    lastMessage,
    /** Is WebSocket connected */
    isConnected: readyState === ReadyState.OPEN,
    /** Is WebSocket connecting */
    isConnecting: readyState === ReadyState.CONNECTING,
    /** Is WebSocket disconnected */
    isDisconnected: readyState === ReadyState.CLOSED || readyState === ReadyState.UNINSTANTIATED,
  };
}

/**
 * Re-export ReadyState for convenience
 */
export { ReadyState };
