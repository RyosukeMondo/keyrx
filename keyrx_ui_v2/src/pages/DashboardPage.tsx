/**
 * DashboardPage - Real-time monitoring dashboard
 *
 * This page provides real-time monitoring of the daemon's state, key events,
 * and latency metrics via WebSocket subscriptions.
 *
 * Features:
 * - Real-time daemon state updates (modifiers, locks, layer)
 * - Live key event stream (max 100 FIFO)
 * - Latency metrics visualization (max 60 samples)
 * - Connection status indicator
 * - Responsive layout (single column mobile, 2-column desktop)
 */

import { useEffect, useState } from 'react';
import { useUnifiedApi } from '../hooks/useUnifiedApi';
import { RpcClient } from '../api/rpc';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../types/rpc';
import { StateIndicatorPanel } from '../components/StateIndicatorPanel';
import { MetricsChart } from '../components/MetricsChart';
import { DashboardEventTimeline } from '../components/DashboardEventTimeline';

/**
 * DashboardPage component - Real-time monitoring dashboard
 */
export function DashboardPage() {
  const api = useUnifiedApi();
  const [client] = useState(() => new RpcClient(api));

  // State management with FIFO limits
  const [daemonState, setDaemonState] = useState<DaemonState | null>(null);
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [latencyHistory, setLatencyHistory] = useState<LatencyMetrics[]>([]);

  // Event stream control
  const [isPaused, setIsPaused] = useState(false);

  // Subscribe to all channels on mount
  useEffect(() => {
    // Subscribe to daemon state updates
    const unsubscribeState = client.onDaemonState((state) => {
      setDaemonState(state);
    });

    // Subscribe to key events with FIFO enforcement (max 100)
    const unsubscribeEvents = client.onKeyEvent((event) => {
      if (!isPaused) {
        setEvents((prev) => {
          const newEvents = [event, ...prev];
          // Enforce 100 max FIFO
          return newEvents.slice(0, 100);
        });
      }
    });

    // Subscribe to latency metrics with FIFO enforcement (max 60)
    const unsubscribeLatency = client.onLatencyUpdate((metrics) => {
      setLatencyHistory((prev) => {
        const newHistory = [...prev, metrics];
        // Enforce 60 max FIFO
        return newHistory.slice(-60);
      });
    });

    // Cleanup subscriptions on unmount
    return () => {
      unsubscribeState();
      unsubscribeEvents();
      unsubscribeLatency();
    };
  }, [client, isPaused]);

  // Toggle pause/resume for event stream
  const handleTogglePause = () => {
    setIsPaused((prev) => !prev);
  };

  // Clear event stream
  const handleClearEvents = () => {
    setEvents([]);
  };

  return (
    <div className="flex flex-col gap-4 p-4 md:p-6">
      {/* Connection Status Banner */}
      <div
        className={`px-4 py-2 rounded text-sm md:text-base font-medium text-center ${
          client.isConnected
            ? 'bg-green-600 text-white'
            : 'bg-red-600 text-white'
        }`}
      >
        {client.isConnected ? 'Connected' : 'Disconnected'}
      </div>

      {/* State Indicators and Metrics Chart - 2-column grid on desktop */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <StateIndicatorPanel state={daemonState} />
        <MetricsChart data={latencyHistory} />
      </div>

      {/* Event Timeline */}
      <DashboardEventTimeline
        events={events}
        isPaused={isPaused}
        onTogglePause={handleTogglePause}
        onClear={handleClearEvents}
      />
    </div>
  );
}
