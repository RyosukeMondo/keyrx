/**
 * TypeScript RPC types matching the Rust WebSocket RPC protocol.
 *
 * This module defines type-safe message types for WebSocket RPC communication
 * between the frontend and daemon. All types match their Rust counterparts
 * defined in keyrx_daemon/src/web/rpc_types.rs exactly.
 */

// JSON-RPC 2.0 standard error codes
export const PARSE_ERROR = -32700;
export const INVALID_REQUEST = -32600;
export const METHOD_NOT_FOUND = -32601;
export const INVALID_PARAMS = -32602;
export const INTERNAL_ERROR = -32603;

/**
 * All available RPC method names.
 * Query methods are read-only operations.
 * Command methods modify state.
 */
export type RpcMethod =
  // Query methods (read-only)
  | "get_profiles"
  | "get_devices"
  | "get_config"
  | "get_layers"
  | "get_latency"
  | "get_events"
  // Command methods (state-modifying)
  | "create_profile"
  | "activate_profile"
  | "delete_profile"
  | "duplicate_profile"
  | "rename_profile"
  | "rename_device"
  | "set_scope_device"
  | "forget_device"
  | "update_config"
  | "set_key_mapping"
  | "delete_key_mapping"
  | "clear_events"
  | "simulate"
  | "reset_simulator";

/**
 * Available subscription channels for real-time updates.
 */
export type SubscriptionChannel = "daemon-state" | "events" | "latency";

/**
 * Messages sent from client to server.
 * Uses discriminated union with 'type' field for type safety.
 */
export type ClientMessage =
  | {
      /** Message type discriminator */
      type: "query";
      /** Unique identifier for request/response correlation */
      id: string;
      /** Method name to invoke */
      method: RpcMethod;
      /** Optional parameters for the method */
      params?: unknown;
    }
  | {
      /** Message type discriminator */
      type: "command";
      /** Unique identifier for request/response correlation */
      id: string;
      /** Method name to invoke */
      method: RpcMethod;
      /** Optional parameters for the method */
      params?: unknown;
    }
  | {
      /** Message type discriminator */
      type: "subscribe";
      /** Unique identifier for request/response correlation */
      id: string;
      /** Channel name to subscribe to */
      channel: SubscriptionChannel;
    }
  | {
      /** Message type discriminator */
      type: "unsubscribe";
      /** Unique identifier for request/response correlation */
      id: string;
      /** Channel name to unsubscribe from */
      channel: SubscriptionChannel;
    };

/**
 * Messages sent from server to client.
 * Uses discriminated union with 'type' field for type safety.
 */
export type ServerMessage =
  | {
      /** Message type discriminator */
      type: "response";
      /** Request ID this response corresponds to */
      id: string;
      /** Result data (success) - only present if no error */
      result?: unknown;
      /** Error information (failure) - only present if error occurred */
      error?: RpcError;
    }
  | {
      /** Message type discriminator */
      type: "event";
      /** Channel this event was published on */
      channel: SubscriptionChannel;
      /** Event data */
      data: unknown;
    }
  | {
      /** Message type discriminator */
      type: "connected";
      /** Protocol version */
      version: string;
      /** Server timestamp in microseconds since UNIX epoch */
      timestamp: number;
    };

/**
 * RPC error structure following JSON-RPC 2.0 conventions.
 */
export interface RpcError {
  /** Numeric error code */
  code: number;
  /** Human-readable error message */
  message: string;
  /** Optional additional error data */
  data?: unknown;
}

/**
 * Current daemon state snapshot.
 * Broadcast on "daemon-state" channel when state changes.
 */
export interface DaemonState {
  /** Active modifier IDs (e.g., ["MD_00", "MD_01"]) */
  modifiers: string[];
  /** Active lock IDs (e.g., ["LK_00"]) */
  locks: string[];
  /** Current active layer name */
  layer: string;
}

/**
 * Individual key event data.
 * Broadcast on "events" channel for each key press/release.
 */
export interface KeyEvent {
  /** Timestamp in microseconds since UNIX epoch */
  timestamp: number;
  /** Key code (e.g., "KEY_A") */
  keyCode: string;
  /** Event type ("press" or "release") */
  eventType: "press" | "release";
  /** Input key (before mapping) */
  input: string;
  /** Output key (after mapping) */
  output: string;
  /** Processing latency in microseconds */
  latency: number;
}

/**
 * Latency statistics.
 * Broadcast on "latency" channel periodically (every 1 second).
 */
export interface LatencyMetrics {
  /** Minimum latency in microseconds */
  min: number;
  /** Average latency in microseconds */
  avg: number;
  /** Maximum latency in microseconds */
  max: number;
  /** 95th percentile latency in microseconds */
  p95: number;
  /** 99th percentile latency in microseconds */
  p99: number;
  /** Timestamp of this stats snapshot (microseconds since UNIX epoch) */
  timestamp: number;
}

/**
 * Type guard to check if a ServerMessage is a Response.
 */
export function isResponse(msg: ServerMessage): msg is Extract<ServerMessage, { type: "response" }> {
  return msg.type === "response";
}

/**
 * Type guard to check if a ServerMessage is an Event.
 */
export function isEvent(msg: ServerMessage): msg is Extract<ServerMessage, { type: "event" }> {
  return msg.type === "event";
}

/**
 * Type guard to check if a ServerMessage is a Connected handshake.
 */
export function isConnected(msg: ServerMessage): msg is Extract<ServerMessage, { type: "connected" }> {
  return msg.type === "connected";
}
