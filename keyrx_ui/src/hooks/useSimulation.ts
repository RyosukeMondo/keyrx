/**
 * useSimulation - React hook for simulation state management
 *
 * This hook manages simulation state including events collection, WebSocket
 * subscription, and statistics computation. It provides a clean interface
 * for components to interact with the simulation without managing state directly.
 *
 * Features:
 * - Event collection with FIFO queue (max events limit)
 * - WebSocket subscription management with cleanup
 * - Start/stop simulation control
 * - Statistics computation (event counts by type)
 * - Memoized callbacks for performance
 *
 * @example
 * ```tsx
 * function SimulatorComponent() {
 *   const { events, isRunning, addEvent, clearEvents, start, stop, statistics } =
 *     useSimulation({ maxEvents: 1000, autoStart: false });
 *
 *   return (
 *     <div>
 *       <button onClick={isRunning ? stop : start}>
 *         {isRunning ? 'Stop' : 'Start'}
 *       </button>
 *       <button onClick={clearEvents}>Clear</button>
 *       <div>Total Events: {statistics.total}</div>
 *     </div>
 *   );
 * }
 * ```
 */

import { useState, useCallback, useEffect, useMemo } from 'react';
import { useUnifiedApi } from './useUnifiedApi';
import type { KeyEvent } from '../types/rpc';

/**
 * Options for useSimulation hook
 */
export interface UseSimulationOptions {
  /** Maximum number of events to keep (default: 1000) */
  maxEvents?: number;
  /** Whether to auto-start simulation on mount (default: false) */
  autoStart?: boolean;
}

/**
 * Statistics computed from simulation events
 */
export interface SimulationStatistics {
  /** Total number of events */
  total: number;
  /** Number of press events */
  pressCount: number;
  /** Number of release events */
  releaseCount: number;
  /** Events per second (computed from recent events) */
  eventsPerSecond: number;
}

/**
 * Return type for useSimulation hook
 */
export interface UseSimulationReturn {
  /** Array of key events (newest first) */
  events: KeyEvent[];
  /** Whether simulation is running (receiving events) */
  isRunning: boolean;
  /** Add a single event to the collection */
  addEvent: (event: KeyEvent) => void;
  /** Clear all events */
  clearEvents: () => void;
  /** Start simulation (subscribe to WebSocket) */
  start: () => void;
  /** Stop simulation (unsubscribe from WebSocket) */
  stop: () => void;
  /** Computed statistics from events */
  statistics: SimulationStatistics;
}

/**
 * React hook for simulation state management
 *
 * @param options - Configuration options for the simulation
 * @returns Simulation state and control methods
 */
export function useSimulation(
  options?: UseSimulationOptions
): UseSimulationReturn {
  const { maxEvents = 1000, autoStart = false } = options || {};

  // State
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [isRunning, setIsRunning] = useState(false);

  // WebSocket API
  const api = useUnifiedApi();

  /**
   * Add a single event to the collection
   * Enforces FIFO queue with maxEvents limit
   */
  const addEvent = useCallback(
    (event: KeyEvent) => {
      setEvents((prev) => {
        // Add new event at the beginning (newest first)
        const newEvents = [event, ...prev];
        // Enforce max events limit (FIFO)
        return newEvents.slice(0, maxEvents);
      });
    },
    [maxEvents]
  );

  /**
   * Clear all events
   */
  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  /**
   * Start simulation
   * Sets running state to true, which triggers WebSocket subscription
   */
  const start = useCallback(() => {
    setIsRunning(true);
  }, []);

  /**
   * Stop simulation
   * Sets running state to false, which triggers WebSocket unsubscription
   */
  const stop = useCallback(() => {
    setIsRunning(false);
  }, []);

  /**
   * WebSocket subscription management
   * Subscribes to 'events' channel when running, unsubscribes when stopped
   */
  useEffect(() => {
    if (!isRunning) {
      return;
    }

    // Subscribe to events channel
    const unsubscribe = api.subscribe('events', (data) => {
      // Type guard: ensure data is a KeyEvent
      if (
        data &&
        typeof data === 'object' &&
        'timestamp' in data &&
        'keyCode' in data &&
        'eventType' in data
      ) {
        addEvent(data as KeyEvent);
      } else {
        console.warn('[useSimulation] Received invalid event data:', data);
      }
    });

    // Cleanup: unsubscribe on unmount or when isRunning changes
    return () => {
      unsubscribe();
    };
  }, [isRunning, api, addEvent]);

  /**
   * Auto-start on mount if enabled
   */
  useEffect(() => {
    if (autoStart) {
      start();
    }
    // Only run on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /**
   * Compute statistics from events
   * Memoized to avoid recomputation on every render
   */
  const statistics = useMemo<SimulationStatistics>(() => {
    const total = events.length;
    const pressCount = events.filter((e) => e.eventType === 'press').length;
    const releaseCount = events.filter((e) => e.eventType === 'release').length;

    // Compute events per second from recent events (last 1 second)
    let eventsPerSecond = 0;
    if (events.length > 0) {
      const now = Date.now() * 1000; // Convert to microseconds
      const oneSecondAgo = now - 1_000_000; // 1 second in microseconds
      const recentEvents = events.filter((e) => e.timestamp >= oneSecondAgo);
      eventsPerSecond = recentEvents.length;
    }

    return {
      total,
      pressCount,
      releaseCount,
      eventsPerSecond,
    };
  }, [events]);

  return {
    events,
    isRunning,
    addEvent,
    clearEvents,
    start,
    stop,
    statistics,
  };
}
