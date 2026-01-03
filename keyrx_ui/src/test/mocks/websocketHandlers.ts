/**
 * MSW WebSocket Handlers for Test Infrastructure
 *
 * Provides comprehensive WebSocket mock server for testing using MSW v2 ws.link() API.
 * Handles connection lifecycle, message routing, state management, and subscriptions.
 *
 * Key features:
 * - Automatic connection management (no fake timers needed)
 * - Type-safe message handling
 * - Channel subscription support (daemon-state, events, latency)
 * - RPC query/command handling
 * - Test isolation with resetWebSocketState()
 */

import { ws } from 'msw';
import type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  DaemonState,
  KeyEvent,
  LatencyMetrics,
  RpcMethod,
} from '../../types/rpc';
import type { DeviceEntry, ProfileMetadata } from '../../types';

/**
 * WebSocket connection state
 */
interface ConnectionState {
  /** WebSocket client instance */
  client: any;
  /** Channels this connection is subscribed to */
  subscriptions: Set<SubscriptionChannel>;
}

/**
 * Mock daemon state for testing
 */
interface MockDaemonState {
  activeProfile: string | null;
  modifiers: string[];
  locks: string[];
  layer: string;
}

/**
 * Active WebSocket connections
 */
const connections = new Map<string, ConnectionState>();

/**
 * Mock daemon state
 */
let daemonState: MockDaemonState = {
  activeProfile: 'default',
  modifiers: [],
  locks: [],
  layer: 'base',
};

/**
 * Mock profiles for RPC queries
 */
let mockProfiles: ProfileMetadata[] = [
  {
    name: 'default',
    createdAt: '2024-01-01T00:00:00Z',
    modifiedAt: '2024-01-01T00:00:00Z',
    deviceCount: 1,
    keyCount: 10,
    isActive: true,
  },
  {
    name: 'gaming',
    createdAt: '2024-01-02T00:00:00Z',
    modifiedAt: '2024-01-02T00:00:00Z',
    deviceCount: 1,
    keyCount: 15,
    isActive: false,
  },
];

/**
 * Mock devices for RPC queries
 */
let mockDevices: DeviceEntry[] = [
  {
    id: 'device-1',
    name: 'Test Keyboard',
    path: '/dev/input/event0',
    serial: null,
    active: true,
    scope: 'global',
    layout: 'ANSI_104',
  },
];

/**
 * Generate unique connection ID
 */
let connectionIdCounter = 0;
function generateConnectionId(): string {
  return `conn-${++connectionIdCounter}`;
}

/**
 * Handle subscribe message
 */
function handleSubscribe(
  connectionId: string,
  id: string,
  channel: SubscriptionChannel,
  client: any
): void {
  const conn = connections.get(connectionId);
  if (!conn) {
    console.warn(`[MSW WebSocket] Connection ${connectionId} not found for subscribe`);
    return;
  }

  conn.subscriptions.add(channel);

  // Send success response
  const response: ServerMessage = {
    type: 'response',
    id,
    result: { success: true, channel },
  };
  client.send(JSON.stringify(response));

  // Immediately send current state for the subscribed channel
  switch (channel) {
    case 'daemon-state':
      broadcastToConnection(connectionId, 'daemon-state', daemonState);
      break;
    case 'latency':
      broadcastToConnection(connectionId, 'latency', {
        min: 500,
        avg: 1200,
        max: 3000,
        p95: 2500,
        p99: 2800,
        timestamp: Date.now() * 1000,
      });
      break;
    case 'events':
      // No initial event broadcast for events channel
      break;
  }
}

/**
 * Handle unsubscribe message
 */
function handleUnsubscribe(
  connectionId: string,
  id: string,
  channel: SubscriptionChannel,
  client: any
): void {
  const conn = connections.get(connectionId);
  if (!conn) {
    console.warn(`[MSW WebSocket] Connection ${connectionId} not found for unsubscribe`);
    return;
  }

  conn.subscriptions.delete(channel);

  // Send success response
  const response: ServerMessage = {
    type: 'response',
    id,
    result: { success: true, channel },
  };
  client.send(JSON.stringify(response));
}

/**
 * Handle query message (read-only RPC)
 */
function handleQuery(id: string, method: RpcMethod, params: unknown, client: any): void {
  console.debug(`[MSW WebSocket] Received query: ${method}`, params);
  let result: unknown;

  switch (method) {
    case 'get_profiles':
      result = { profiles: mockProfiles };
      break;
    case 'get_devices':
      result = { devices: mockDevices };
      break;
    case 'get_active_profile':
      result = { activeProfile: daemonState.activeProfile };
      break;
    case 'get_latency':
      result = {
        min: 500,
        avg: 1200,
        max: 3000,
        p95: 2500,
        p99: 2800,
      };
      break;
    case 'get_events':
      result = { events: [] };
      break;
    case 'get_profile_config':
      const profileName = (params as { name: string })?.name;
      const profile = mockProfiles.find((p) => p.name === profileName);
      if (profile) {
        result = {
          layers: [
            {
              id: 'base',
              name: 'Base Layer',
              mappings: {},
            },
          ],
        };
      } else {
        const errorResponse: ServerMessage = {
          type: 'response',
          id,
          error: {
            code: -32602,
            message: `Profile not found: ${profileName}`,
          },
        };
        client.send(JSON.stringify(errorResponse));
        return;
      }
      break;
    default:
      // Send error for unsupported methods
      const errorResponse: ServerMessage = {
        type: 'response',
        id,
        error: {
          code: -32601,
          message: `Method not found: ${method}`,
        },
      };
      client.send(JSON.stringify(errorResponse));
      return;
  }

  // Send success response
  const response: ServerMessage = {
    type: 'response',
    id,
    result,
  };
  client.send(JSON.stringify(response));
}

/**
 * Handle command message (state-modifying RPC)
 */
function handleCommand(id: string, method: RpcMethod, params: unknown, client: any): void {
  let result: unknown;

  switch (method) {
    case 'activate_profile':
      const profileName = (params as { name: string })?.name;
      if (profileName) {
        // Deactivate all profiles
        mockProfiles.forEach((p) => {
          p.isActive = false;
        });
        // Activate target profile
        const profile = mockProfiles.find((p) => p.name === profileName);
        if (profile) {
          profile.isActive = true;
          daemonState.activeProfile = profileName;
          result = { success: true };

          // Broadcast state change to all subscribed connections
          broadcastEvent('daemon-state', daemonState);
        } else {
          const errorResponse: ServerMessage = {
            type: 'response',
            id,
            error: {
              code: -32602,
              message: `Profile not found: ${profileName}`,
            },
          };
          client.send(JSON.stringify(errorResponse));
          return;
        }
      } else {
        const errorResponse: ServerMessage = {
          type: 'response',
          id,
          error: {
            code: -32602,
            message: 'Invalid params: name required',
          },
        };
        client.send(JSON.stringify(errorResponse));
        return;
      }
      break;

    default:
      // Send error for unsupported methods
      const errorResponse: ServerMessage = {
        type: 'response',
        id,
        error: {
          code: -32601,
          message: `Method not found: ${method}`,
        },
      };
      client.send(JSON.stringify(errorResponse));
      return;
  }

  // Send success response
  const response: ServerMessage = {
    type: 'response',
    id,
    result,
  };
  client.send(JSON.stringify(response));
}

/**
 * Broadcast event to a specific connection
 */
function broadcastToConnection(
  connectionId: string,
  channel: SubscriptionChannel,
  data: unknown
): void {
  const conn = connections.get(connectionId);
  if (!conn || !conn.subscriptions.has(channel)) {
    return;
  }

  const event: ServerMessage = {
    type: 'event',
    channel,
    data,
  };

  try {
    conn.client.send(JSON.stringify(event));
  } catch (error) {
    console.debug(`[MSW WebSocket] Failed to send to connection ${connectionId}:`, error);
  }
}

/**
 * Broadcast event to all subscribed connections
 */
export function broadcastEvent(channel: SubscriptionChannel, data: unknown): void {
  connections.forEach((conn, connectionId) => {
    if (conn.subscriptions.has(channel)) {
      broadcastToConnection(connectionId, channel, data);
    }
  });
}

/**
 * Reset WebSocket state for test isolation
 * Call this in afterEach to ensure tests don't interfere with each other
 */
export function resetWebSocketState(): void {
  // Clear all connections
  connections.clear();

  // Reset daemon state
  daemonState = {
    activeProfile: 'default',
    modifiers: [],
    locks: [],
    layer: 'base',
  };

  // Reset mock profiles
  mockProfiles = [
    {
      name: 'default',
      createdAt: '2024-01-01T00:00:00Z',
      modifiedAt: '2024-01-01T00:00:00Z',
      deviceCount: 1,
      keyCount: 10,
      isActive: true,
    },
    {
      name: 'gaming',
      createdAt: '2024-01-02T00:00:00Z',
      modifiedAt: '2024-01-02T00:00:00Z',
      deviceCount: 1,
      keyCount: 15,
      isActive: false,
    },
  ];

  // Reset mock devices
  mockDevices = [
    {
      id: 'device-1',
      name: 'Test Keyboard',
      path: '/dev/input/event0',
      serial: null,
      active: true,
      scope: 'global',
      layout: 'ANSI_104',
    },
  ];

  // Reset connection ID counter
  connectionIdCounter = 0;
}

/**
 * Create MSW WebSocket handlers
 */
export function createWebSocketHandlers() {
  return [
    ws.link('ws://localhost:9867/ws').addEventListener('connection', ({ client }) => {
      const connectionId = generateConnectionId();
      console.debug(`[MSW WebSocket] Connection opened: ${connectionId}`);

      // Register connection
      connections.set(connectionId, {
        client,
        subscriptions: new Set(),
      });

      // Send connected handshake immediately
      const connectedMessage: ServerMessage = {
        type: 'connected',
        version: '1.0.0',
        timestamp: Date.now() * 1000,
      };
      client.send(JSON.stringify(connectedMessage));

      // Handle incoming messages
      client.addEventListener('message', (event: MessageEvent) => {
        console.debug('[MSW WebSocket] Received message:', event.data);
        try {
          const message: ClientMessage = JSON.parse(event.data);
          console.debug('[MSW WebSocket] Parsed message:', message);

          switch (message.type) {
            case 'subscribe':
              handleSubscribe(connectionId, message.id, message.channel, client);
              break;

            case 'unsubscribe':
              handleUnsubscribe(connectionId, message.id, message.channel, client);
              break;

            case 'query':
              handleQuery(message.id, message.method, message.params, client);
              break;

            case 'command':
              handleCommand(message.id, message.method, message.params, client);
              break;

            default:
              console.warn('[MSW WebSocket] Unknown message type:', message);
          }
        } catch (error) {
          console.debug('[MSW WebSocket] Failed to parse message:', event.data, error);
          // Silently ignore non-JSON messages
        }
      });

      // Handle client disconnect
      client.addEventListener('close', () => {
        console.debug(`[MSW WebSocket] Connection closed: ${connectionId}`);
        connections.delete(connectionId);
      });
    }),
  ];
}
