/**
 * Shared transform from the daemon's KeyEventData wire type to the frontend's
 * EventRecord view type.
 *
 * This used to be copy-pasted in stores/metricsStore.ts, hooks/useMetrics.ts,
 * and api/websocket.ts. Keep it in one place.
 */

import type { KeyEventData, EventRecord } from '../types';

/**
 * Transform a daemon KeyEventData (WebSocket event payload or REST event log
 * entry) into the frontend's EventRecord shape.
 */
export function toEventRecord(data: KeyEventData): EventRecord {
  return {
    id: `evt-${data.timestamp}-${Math.random().toString(36).slice(2, 8)}`,
    timestamp: new Date(data.timestamp / 1000).toISOString(), // microseconds -> ISO string
    type: data.eventType === 'press' ? 'press' : 'release',
    keyCode: data.keyCode.replace(/^KEY_/, ''), // Remove KEY_ prefix for display
    layer: 'Base', // TODO: Get from daemon state
    latencyUs: data.latency,
    action: data.mappingTriggered ? data.output : undefined,
    input: data.input,
    output: data.output,
    deviceId: data.deviceId,
    deviceName: data.deviceName,
    mappingType: data.mappingType,
    mappingTriggered: data.mappingTriggered,
  };
}
