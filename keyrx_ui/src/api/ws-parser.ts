/**
 * Message parser for WebSocket messages
 *
 * Handles parsing and transforming daemon messages to frontend types.
 */
import type {
  WSMessage,
  EventRecord,
  DaemonState,
  LatencyStats,
  KeyEventPayload,
} from '../types';

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
