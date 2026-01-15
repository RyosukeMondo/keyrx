import { create } from 'zustand';
import type {
  LatencyStats,
  EventRecord,
  DaemonState,
  WSMessage,
} from '../types';
import * as metricsApi from '../api/metrics';
import { ApiError } from '../api/client';

interface MetricsStore {
  // State
  latencyStats: LatencyStats | null;
  eventLog: EventRecord[];
  currentState: DaemonState | null;
  connected: boolean;
  loading: boolean;
  error: string | null;

  // WebSocket
  ws: WebSocket | null;

  // Actions
  fetchMetrics: () => Promise<void>;
  subscribeToEvents: () => void;
  unsubscribeFromEvents: () => void;
  clearEventLog: () => void;
  clearError: () => void;
}

export const useMetricsStore = create<MetricsStore>((set, get) => ({
  // Initial state
  latencyStats: null,
  eventLog: [],
  currentState: null,
  connected: false,
  loading: false,
  error: null,
  ws: null,

  // Fetch current metrics
  fetchMetrics: async () => {
    set({ loading: true, error: null });
    try {
      // Fetch latency stats and event log in parallel
      const [latencyStats, eventLog] = await Promise.all([
        metricsApi.fetchLatencyStats(),
        metricsApi.fetchEventLog(),
      ]);

      set({ latencyStats, eventLog, loading: false });
    } catch (error) {
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
    }
  },

  // Subscribe to real-time events via WebSocket
  subscribeToEvents: () => {
    const { ws } = get();

    // Don't create duplicate connections
    if (ws && ws.readyState === WebSocket.OPEN) {
      return;
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    const websocket = new WebSocket(wsUrl);

    websocket.onopen = () => {
      set({ connected: true, error: null });
    };

    websocket.onmessage = (event) => {
      try {
        const message: WSMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'event': {
            // Transform daemon's KeyEventPayload to frontend's EventRecord
            // Type is automatically narrowed by discriminated union
            const payload = message.payload;

            const eventRecord: EventRecord = {
              id: `evt-${payload.timestamp}-${Math.random().toString(36).slice(2, 8)}`,
              timestamp: new Date(payload.timestamp / 1000).toISOString(), // Convert microseconds to ISO string
              type: payload.eventType === 'press' ? 'press' : 'release',
              keyCode: payload.keyCode.replace(/^KEY_/, ''), // Remove KEY_ prefix for display
              layer: 'Base', // TODO: Get from daemon state
              latencyUs: payload.latency,
              action: payload.mappingTriggered ? payload.output : undefined,
              input: payload.input,
              output: payload.output,
              deviceId: payload.deviceId,
              deviceName: payload.deviceName,
              mappingType: payload.mappingType,
              mappingTriggered: payload.mappingTriggered,
            };

            const { eventLog } = get();
            // Prepend new event (most recent first)
            const updatedLog = [eventRecord, ...eventLog];
            // Limit to 1000 events
            if (updatedLog.length > 1000) {
              updatedLog.pop();
            }
            set({ eventLog: updatedLog });
            break;
          }

          case 'state': {
            // Type is automatically narrowed to DaemonState
            set({ currentState: message.payload });
            break;
          }

          case 'latency': {
            // Type is automatically narrowed to LatencyStats
            set({ latencyStats: message.payload });
            break;
          }

          case 'error': {
            const errorPayload = message.payload as { message: string };
            set({ error: errorPayload.message });
            break;
          }
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    websocket.onerror = (error) => {
      console.error('WebSocket error:', error);
      set({ error: 'WebSocket connection error', connected: false });
    };

    websocket.onclose = () => {
      set({ connected: false });

      // Attempt to reconnect after 3 seconds
      setTimeout(() => {
        const { ws: currentWs } = get();
        if (!currentWs || currentWs.readyState === WebSocket.CLOSED) {
          get().subscribeToEvents();
        }
      }, 3000);
    };

    set({ ws: websocket });
  },

  // Unsubscribe from WebSocket events
  unsubscribeFromEvents: () => {
    const { ws } = get();
    if (ws) {
      ws.close();
      set({ ws: null, connected: false });
    }
  },

  // Clear event log
  clearEventLog: () => {
    set({ eventLog: [] });
  },

  // Clear error state
  clearError: () => set({ error: null }),
}));
