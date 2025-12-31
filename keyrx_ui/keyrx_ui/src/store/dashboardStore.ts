/**
 * Dashboard Store
 *
 * Zustand store for real-time daemon state, events, and metrics.
 * Manages WebSocket connection state and provides actions for updates.
 */

import { create } from 'zustand';

/**
 * Current daemon state snapshot
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
 * Individual key event data
 */
export interface KeyEvent {
  /** Timestamp in microseconds since UNIX epoch */
  timestamp: number;
  /** Key code (e.g., "KEY_A") */
  keyCode: string;
  /** Event type ("press" or "release") */
  eventType: 'press' | 'release';
  /** Input key (before mapping) */
  input: string;
  /** Output key (after mapping) */
  output: string;
  /** Processing latency in microseconds */
  latency: number;
}

/**
 * Latency statistics
 */
export interface LatencyStats {
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
 * WebSocket connection status
 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected';

/**
 * Dashboard store state
 */
interface DashboardStore {
  /** Current daemon state */
  currentState: DaemonState;
  /** Recent events (last 100, FIFO) */
  events: KeyEvent[];
  /** Current latency statistics */
  metrics: LatencyStats | null;
  /** WebSocket connection status */
  connectionStatus: ConnectionStatus;

  /** Update daemon state */
  updateState: (state: DaemonState) => void;
  /** Add a new event (maintains FIFO with 100 max) */
  addEvent: (event: KeyEvent) => void;
  /** Update latency metrics */
  updateMetrics: (metrics: LatencyStats) => void;
  /** Update connection status */
  setConnectionStatus: (status: ConnectionStatus) => void;
  /** Clear all events */
  clearEvents: () => void;
  /** Reset store to initial state */
  reset: () => void;
}

/**
 * Maximum number of events to keep in the store
 */
const MAX_EVENTS = 100;

/**
 * Initial daemon state
 */
const initialState: DaemonState = {
  modifiers: [],
  locks: [],
  layer: 'base',
};

/**
 * Dashboard store hook
 */
export const useDashboardStore = create<DashboardStore>((set) => ({
  currentState: initialState,
  events: [],
  metrics: null,
  connectionStatus: 'disconnected',

  updateState: (state: DaemonState) =>
    set(() => ({
      currentState: state,
    })),

  addEvent: (event: KeyEvent) =>
    set((state) => {
      const newEvents = [...state.events, event];
      // Maintain FIFO: remove oldest if exceeding MAX_EVENTS
      if (newEvents.length > MAX_EVENTS) {
        newEvents.shift();
      }
      return { events: newEvents };
    }),

  updateMetrics: (metrics: LatencyStats) =>
    set(() => ({
      metrics,
    })),

  setConnectionStatus: (status: ConnectionStatus) =>
    set(() => ({
      connectionStatus: status,
    })),

  clearEvents: () =>
    set(() => ({
      events: [],
    })),

  reset: () =>
    set(() => ({
      currentState: initialState,
      events: [],
      metrics: null,
      connectionStatus: 'disconnected',
    })),
}));
