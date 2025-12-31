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
            const eventRecord = message.payload as EventRecord;
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
            const state = message.payload as DaemonState;
            set({ currentState: state });
            break;
          }

          case 'latency': {
            const stats = message.payload as LatencyStats;
            set({ latencyStats: stats });
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
