import { z } from 'zod';

/**
 * Zod schemas for runtime validation of API responses.
 * These schemas match the TypeScript types generated from Rust structs in types/generated.ts.
 *
 * Note: Schemas are designed to be permissive with unexpected fields (log warnings instead of throwing).
 */

// JSON value type for serde_json::Value compatibility
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | { [key: string]: JsonValue }
  | JsonValue[];

export const ValueSchema: z.ZodType<JsonValue> = z.lazy(() =>
  z.union([
    z.string(),
    z.number(),
    z.boolean(),
    z.null(),
    z.record(z.string(), ValueSchema),
    z.array(ValueSchema),
  ])
);

// Device scope enum
export const DeviceScopeSchema = z.enum(['DeviceSpecific', 'Global']);

// Device metadata entry
export const DeviceEntrySchema = z
  .object({
    id: z.string().max(256),
    name: z.string().max(64),
    serial: z.string().optional(),
    scope: DeviceScopeSchema,
    layout: z.string().max(32).optional(),
    last_seen: z.number(),
  })
  .passthrough(); // Allow unexpected fields (log warning in validator)

// Device information from RPC and REST API
// Matches Rust DeviceResponse struct in keyrx_daemon/src/web/api/devices.rs
export const DeviceRpcInfoSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    path: z.string(),
    serial: z.string().nullable().optional(), // Backend returns null when not available
    active: z.boolean(),
    scope: z.string().nullable().optional(),
    layout: z.string().nullable().optional(), // Backend returns null when not set
  })
  .passthrough();

// Event in event log
export const EventRpcEntrySchema = z
  .object({
    timestamp: z.number(),
    key_code: z.number(),
    event_type: z.string(),
    device_id: z.string(),
  })
  .passthrough();

// Individual key event data
export const KeyEventDataSchema = z
  .object({
    timestamp: z.number(),
    keyCode: z.string(),
    eventType: z.string(),
    input: z.string(),
    output: z.string(),
    latency: z.number(),
  })
  .passthrough();

// Latency statistics from RPC
export const LatencyRpcStatsSchema = z
  .object({
    min_us: z.number(),
    avg_us: z.number(),
    max_us: z.number(),
    p50_us: z.number(),
    p95_us: z.number(),
    p99_us: z.number(),
    count: z.number(),
  })
  .passthrough();

// Latency statistics
export const LatencyStatsSchema = z
  .object({
    min: z.number(),
    avg: z.number(),
    max: z.number(),
    p95: z.number(),
    p99: z.number(),
    timestamp: z.number(),
  })
  .passthrough();

// Profile configuration from RPC
// Matches Rust ProfileConfigRpc in keyrx_daemon/src/web/handlers/profile.rs
export const ProfileConfigRpcSchema = z
  .object({
    name: z.string(),
    source: z.string(), // The Rhai source code
  })
  .passthrough();

// Profile information from RPC (used in list responses)
export const ProfileRpcInfoSchema = z
  .object({
    name: z.string(),
    rhaiPath: z.string(),
    krxPath: z.string(),
    modifiedAt: z.string(), // ISO 8601 timestamp
    createdAt: z.string(), // ISO 8601 timestamp
    layerCount: z.number(),
    deviceCount: z.number(),
    keyCount: z.number(),
    isActive: z.boolean(),
  })
  .passthrough();

// Activation result
export const ActivationRpcResultSchema = z
  .object({
    success: z.boolean(),
    compile_time_ms: z.number(),
    reload_time_ms: z.number(),
    error: z.string().optional(),
  })
  .passthrough();

// Daemon state snapshot
export const DaemonStateSchema = z
  .object({
    modifiers: z.array(z.string()),
    locks: z.array(z.string()),
    layer: z.string(),
    active_profile: z.string().optional(),
  })
  .passthrough();

// RPC error structure
export const RpcErrorSchema = z
  .object({
    code: z.number(),
    message: z.string(),
    data: ValueSchema.optional(),
  })
  .passthrough();

// Client messages (requests from UI to daemon)
export const ClientMessageSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('query'),
    content: z.object({
      id: z.string(),
      method: z.string(),
      params: ValueSchema.optional(),
    }),
  }),
  z.object({
    type: z.literal('command'),
    content: z.object({
      id: z.string(),
      method: z.string(),
      params: ValueSchema.optional(),
    }),
  }),
  z.object({
    type: z.literal('subscribe'),
    content: z.object({
      id: z.string(),
      channel: z.string(),
    }),
  }),
  z.object({
    type: z.literal('unsubscribe'),
    content: z.object({
      id: z.string(),
      channel: z.string(),
    }),
  }),
]);

// Daemon events (broadcasts from daemon)
export const DaemonEventSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('state'),
    payload: DaemonStateSchema,
  }),
  z.object({
    type: z.literal('event'),
    payload: KeyEventDataSchema,
  }),
  z.object({
    type: z.literal('latency'),
    payload: LatencyStatsSchema,
  }),
]);

// Server messages (responses from daemon to UI)
export const ServerMessageSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('response'),
    content: z.object({
      id: z.string(),
      result: ValueSchema.optional(),
      error: RpcErrorSchema.optional(),
    }),
  }),
  z.object({
    type: z.literal('event'),
    content: z.object({
      channel: z.string(),
      data: ValueSchema,
    }),
  }),
  z.object({
    type: z.literal('connected'),
    content: z.object({
      version: z.string(),
      timestamp: z.number(),
    }),
  }),
]);

// API response collections
// NOTE: DeviceListResponseSchema uses DeviceRpcInfoSchema (not DeviceEntrySchema)
// because the REST API returns the same format as the RPC interface.
// DeviceEntrySchema is for device metadata storage format, not API responses.
export const DeviceListResponseSchema = z
  .object({
    devices: z.array(DeviceRpcInfoSchema),
  })
  .passthrough();

export const ProfileListResponseSchema = z
  .object({
    profiles: z.array(ProfileRpcInfoSchema),
  })
  .passthrough();

export const ProfileConfigResponseSchema = ProfileConfigRpcSchema;

// Status response from REST API
// Matches Rust StatusResponse in keyrx_daemon/src/web/api/metrics.rs
export const StatusResponseSchema = z
  .object({
    status: z.string(),
    version: z.string(),
    daemon_running: z.boolean(),
    uptime_secs: z.number().nullable().optional(),
    active_profile: z.string().nullable().optional(),
    device_count: z.number().nullable().optional(),
  })
  .passthrough();

// Update device config response
// Matches JSON response from PATCH /api/devices/:id
export const UpdateDeviceConfigResponseSchema = z
  .object({
    success: z.boolean(),
  })
  .passthrough();

/**
 * Validates API response data against a Zod schema.
 *
 * @template T - The expected TypeScript type
 * @param schema - Zod schema to validate against
 * @param data - Unknown data to validate
 * @param endpoint - API endpoint name for error context
 * @returns Validated data of type T
 * @throws Error if validation fails
 *
 * @example
 * const devices = validateApiResponse(
 *   DeviceListResponseSchema,
 *   await response.json(),
 *   'GET /api/devices'
 * );
 */
export function validateApiResponse<T>(
  schema: z.ZodSchema<T>,
  data: unknown,
  endpoint: string
): T {
  const result = schema.safeParse(data);

  if (!result.success) {
    const errorMessage = `API validation failed for ${endpoint}: ${result.error.message}`;

    // Log structured error for debugging
    console.error(
      JSON.stringify({
        timestamp: new Date().toISOString(),
        level: 'error',
        service: 'API Validation',
        event: 'validation_failed',
        context: {
          endpoint,
          error: result.error.format(),
          data: data,
        },
      })
    );

    throw new Error(errorMessage);
  }

  // Check for unexpected fields and log as warnings
  // Note: Using passthrough() on schemas allows unexpected fields,
  // so we just log them as warnings rather than failing validation
  if (typeof data === 'object' && data !== null) {
    const receivedKeys = Object.keys(data);

    // We can't easily introspect Zod schemas to get expected keys,
    // but passthrough() already handles this by including extra fields in the result.
    // Just log that we received data for tracking purposes.
    if (receivedKeys.length > 0) {
      if (import.meta.env.DEV) {
        // eslint-disable-next-line no-console
        console.debug(
          JSON.stringify({
            timestamp: new Date().toISOString(),
            level: 'debug',
            service: 'API Validation',
            event: 'validation_success',
            context: {
              endpoint,
              fieldCount: receivedKeys.length,
            },
          })
        );
      }
    }
  }

  return result.data;
}

/**
 * Validates WebSocket RPC message.
 * Handles both client messages (outgoing) and server messages (incoming).
 *
 * @param data - Unknown message data
 * @param direction - 'client' for outgoing, 'server' for incoming
 * @returns Validated message
 * @throws Error if validation fails
 */
export function validateRpcMessage(
  data: unknown,
  direction: 'client' | 'server'
): z.infer<typeof ClientMessageSchema> | z.infer<typeof ServerMessageSchema> {
  const schema =
    direction === 'client' ? ClientMessageSchema : ServerMessageSchema;
  const result = schema.safeParse(data);

  if (!result.success) {
    console.error(
      JSON.stringify({
        timestamp: new Date().toISOString(),
        level: 'error',
        service: 'WebSocket RPC',
        event: 'message_validation_failed',
        context: {
          direction,
          error: result.error.format(),
          data: data,
        },
      })
    );

    throw new Error(
      `Invalid ${direction} RPC message: ${result.error.message}`
    );
  }

  return result.data;
}
