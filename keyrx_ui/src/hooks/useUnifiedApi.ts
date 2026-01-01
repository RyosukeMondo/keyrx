/**
 * useUnifiedApi - React hook for WebSocket RPC communication
 *
 * This hook provides a unified API for WebSocket-based RPC communication with the daemon.
 * It handles connection management, request/response correlation, subscriptions, and auto-reconnect.
 *
 * Features:
 * - Request/response correlation via UUID
 * - 30-second timeout for all requests
 * - Auto-reconnect (3s interval, 10 max attempts)
 * - Type-safe query/command methods
 * - Subscription management with cleanup
 * - Connection state tracking
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const api = useUnifiedApi();
 *
 *   // Query example
 *   const profiles = await api.query('get_profiles');
 *
 *   // Command example
 *   await api.command('activate_profile', { name: 'Gaming' });
 *
 *   // Subscription example
 *   useEffect(() => {
 *     return api.subscribe('daemon-state', (state) => {
 *       console.log('State changed:', state);
 *     });
 *   }, []);
 * }
 * ```
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';
import { v4 as uuidv4 } from 'uuid';
import { env } from '../config/env';
import type {
  ClientMessage,
  ServerMessage,
  RpcMethod,
  SubscriptionChannel,
  RpcError,
  isResponse,
  isEvent,
  isConnected,
} from '../types/rpc';

// Import type guards
import {
  isResponse as checkIsResponse,
  isEvent as checkIsEvent,
  isConnected as checkIsConnected,
} from '../types/rpc';

const REQUEST_TIMEOUT_MS = 30000; // 30 seconds
const RECONNECT_INTERVAL_MS = 3000; // 3 seconds
const MAX_RECONNECT_ATTEMPTS = 10;

/**
 * Pending request structure for tracking in-flight requests
 */
interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timeoutId: NodeJS.Timeout;
}

/**
 * Subscription handler function type
 */
type SubscriptionHandler = (data: unknown) => void;

/**
 * Hook return type
 */
export interface UseUnifiedApiReturn {
  /** Execute a query (read-only) RPC method */
  query: <T = unknown>(method: RpcMethod, params?: unknown) => Promise<T>;
  /** Execute a command (state-modifying) RPC method */
  command: <T = unknown>(method: RpcMethod, params?: unknown) => Promise<T>;
  /** Subscribe to a channel for real-time updates */
  subscribe: (channel: SubscriptionChannel, handler: SubscriptionHandler) => () => void;
  /** Unsubscribe from a channel */
  unsubscribe: (channel: SubscriptionChannel) => void;
  /** Current WebSocket connection state */
  readyState: ReadyState;
  /** Whether the WebSocket is connected and handshake completed */
  isConnected: boolean;
  /** Last error that occurred */
  lastError: Error | null;
}

/**
 * React hook for unified WebSocket RPC API
 *
 * @param url - Optional WebSocket URL (defaults to env.wsUrl)
 * @returns API methods and connection state
 */
export function useUnifiedApi(url?: string): UseUnifiedApiReturn {
  const wsUrl = url || env.wsUrl;

  // Connection state
  const [isConnected, setIsConnected] = useState(false);
  const [lastError, setLastError] = useState<Error | null>(null);

  // Use useRef for mutable tracking (not useState to avoid re-renders)
  const pendingRequests = useRef<Map<string, PendingRequest>>(new Map());
  const subscriptions = useRef<Map<SubscriptionChannel, Set<SubscriptionHandler>>>(new Map());

  // WebSocket connection with auto-reconnect
  const { sendMessage, lastMessage, readyState } = useWebSocket(wsUrl, {
    shouldReconnect: () => true,
    reconnectInterval: RECONNECT_INTERVAL_MS,
    reconnectAttempts: MAX_RECONNECT_ATTEMPTS,
    onError: (event) => {
      console.error('[useUnifiedApi] WebSocket error:', event);
      setLastError(new Error('WebSocket connection error'));
      setIsConnected(false);
    },
    onClose: () => {
      console.log('[useUnifiedApi] WebSocket closed');
      setIsConnected(false);

      // Reject all pending requests on disconnect
      pendingRequests.current.forEach((pending, id) => {
        clearTimeout(pending.timeoutId);
        pending.reject(new Error('WebSocket connection closed'));
      });
      pendingRequests.current.clear();
    },
  });

  // Handle incoming messages
  useEffect(() => {
    if (!lastMessage?.data) return;

    try {
      const message: ServerMessage = JSON.parse(lastMessage.data);

      // Handle Connected handshake
      if (checkIsConnected(message)) {
        console.log('[useUnifiedApi] Connected:', message);
        setIsConnected(true);
        setLastError(null);
        return;
      }

      // Handle Response messages
      if (checkIsResponse(message)) {
        const pending = pendingRequests.current.get(message.id);
        if (pending) {
          clearTimeout(pending.timeoutId);
          pendingRequests.current.delete(message.id);

          if (message.error) {
            pending.reject(new Error(message.error.message));
          } else {
            pending.resolve(message.result);
          }
        } else {
          console.warn('[useUnifiedApi] Received response for unknown request:', message.id);
        }
        return;
      }

      // Handle Event messages (broadcasts)
      if (checkIsEvent(message)) {
        const handlers = subscriptions.current.get(message.channel);
        if (handlers) {
          handlers.forEach((handler) => {
            try {
              handler(message.data);
            } catch (error) {
              console.error('[useUnifiedApi] Subscription handler error:', error);
            }
          });
        }
        return;
      }

      console.warn('[useUnifiedApi] Unknown message type:', message);
    } catch (error) {
      console.error('[useUnifiedApi] Failed to parse message:', error);
      setLastError(error instanceof Error ? error : new Error('Failed to parse message'));
    }
  }, [lastMessage]);

  /**
   * Send a request and return a promise that resolves with the response
   */
  const sendRequest = useCallback(
    <T = unknown>(message: ClientMessage): Promise<T> => {
      return new Promise((resolve, reject) => {
        if (readyState !== ReadyState.OPEN) {
          reject(new Error('WebSocket is not connected'));
          return;
        }

        const id = message.id;

        // Setup timeout
        const timeoutId = setTimeout(() => {
          pendingRequests.current.delete(id);
          reject(new Error(`Request timeout after ${REQUEST_TIMEOUT_MS}ms`));
        }, REQUEST_TIMEOUT_MS);

        // Store pending request
        pendingRequests.current.set(id, {
          resolve: resolve as (value: unknown) => void,
          reject,
          timeoutId,
        });

        // Send message
        try {
          sendMessage(JSON.stringify(message));
        } catch (error) {
          clearTimeout(timeoutId);
          pendingRequests.current.delete(id);
          reject(error);
        }
      });
    },
    [readyState, sendMessage]
  );

  /**
   * Execute a query (read-only) RPC method
   */
  const query = useCallback(
    <T = unknown>(method: RpcMethod, params?: unknown): Promise<T> => {
      const id = uuidv4();
      const message: ClientMessage = {
        type: 'query',
        id,
        method,
        params,
      };
      return sendRequest<T>(message);
    },
    [sendRequest]
  );

  /**
   * Execute a command (state-modifying) RPC method
   */
  const command = useCallback(
    <T = unknown>(method: RpcMethod, params?: unknown): Promise<T> => {
      const id = uuidv4();
      const message: ClientMessage = {
        type: 'command',
        id,
        method,
        params,
      };
      return sendRequest<T>(message);
    },
    [sendRequest]
  );

  /**
   * Subscribe to a channel for real-time updates
   *
   * @param channel - Channel to subscribe to
   * @param handler - Function to call when events are received
   * @returns Unsubscribe function
   */
  const subscribe = useCallback(
    (channel: SubscriptionChannel, handler: SubscriptionHandler): (() => void) => {
      // Add handler to subscriptions
      let handlers = subscriptions.current.get(channel);
      if (!handlers) {
        handlers = new Set();
        subscriptions.current.set(channel, handlers);
      }

      const isFirstSubscriber = handlers.size === 0;
      handlers.add(handler);

      // Send subscribe message to server if this is the first subscriber
      if (isFirstSubscriber && readyState === ReadyState.OPEN) {
        const id = uuidv4();
        const message: ClientMessage = {
          type: 'subscribe',
          id,
          channel,
        };
        sendMessage(JSON.stringify(message));
      }

      // Return unsubscribe function
      return () => {
        const handlers = subscriptions.current.get(channel);
        if (handlers) {
          handlers.delete(handler);

          // If no more handlers, send unsubscribe message to server
          if (handlers.size === 0) {
            subscriptions.current.delete(channel);
            if (readyState === ReadyState.OPEN) {
              const id = uuidv4();
              const message: ClientMessage = {
                type: 'unsubscribe',
                id,
                channel,
              };
              sendMessage(JSON.stringify(message));
            }
          }
        }
      };
    },
    [readyState, sendMessage]
  );

  /**
   * Unsubscribe from a channel (all handlers)
   */
  const unsubscribe = useCallback(
    (channel: SubscriptionChannel): void => {
      const handlers = subscriptions.current.get(channel);
      if (handlers) {
        subscriptions.current.delete(channel);

        // Send unsubscribe message to server
        if (readyState === ReadyState.OPEN) {
          const id = uuidv4();
          const message: ClientMessage = {
            type: 'unsubscribe',
            id,
            channel,
          };
          sendMessage(JSON.stringify(message));
        }
      }
    },
    [readyState, sendMessage]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Clear all pending requests
      pendingRequests.current.forEach((pending) => {
        clearTimeout(pending.timeoutId);
        pending.reject(new Error('Component unmounted'));
      });
      pendingRequests.current.clear();

      // Unsubscribe from all channels
      subscriptions.current.forEach((_, channel) => {
        if (readyState === ReadyState.OPEN) {
          const id = uuidv4();
          const message: ClientMessage = {
            type: 'unsubscribe',
            id,
            channel,
          };
          try {
            sendMessage(JSON.stringify(message));
          } catch (error) {
            console.error('[useUnifiedApi] Failed to send unsubscribe on unmount:', error);
          }
        }
      });
      subscriptions.current.clear();
    };
  }, [readyState, sendMessage]);

  return {
    query,
    command,
    subscribe,
    unsubscribe,
    readyState,
    isConnected,
    lastError,
  };
}
