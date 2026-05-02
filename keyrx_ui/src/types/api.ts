/**
 * Shared API types for RPC communication.
 *
 * These types define the interface for the RPC layer without depending on
 * any React or UI code. This allows RpcClient to be used in non-React contexts.
 */

import type { ReadyState } from 'react-use-websocket';
import type { RpcMethod, SubscriptionChannel } from './rpc';

/**
 * RPC API interface for daemon communication.
 *
 * This interface defines the contract for WebSocket-based RPC communication,
 * independent of any React hooks. It can be implemented by:
 * - The useUnifiedApi hook (React)
 * - A mock implementation (testing)
 * - A standalone WebSocket client (non-React usage)
 */
export interface RpcApi {
  /** Execute a query (read-only) RPC method */
  query: <T = unknown>(method: RpcMethod, params?: unknown) => Promise<T>;
  /** Execute a command (state-modifying) RPC method */
  command: <T = unknown>(method: RpcMethod, params?: unknown) => Promise<T>;
  /** Subscribe to a channel for real-time updates */
  subscribe: (
    channel: SubscriptionChannel,
    handler: (data: unknown) => void
  ) => () => void;
  /** Unsubscribe from a channel */
  unsubscribe: (channel: SubscriptionChannel) => void;
  /** Current WebSocket connection state */
  readyState: ReadyState;
  /** Whether the WebSocket is connected and handshake completed */
  isConnected: boolean;
  /** Last error that occurred */
  lastError: Error | null;
}
