import React, { useState, useEffect, useMemo } from 'react';
import { Link } from 'react-router-dom';
import { Card } from '../components/Card';
import { Activity, FileCode } from 'lucide-react';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import { useMetricsStore } from '../stores/metricsStore';
import { useActiveProfile } from '../hooks/useProfiles';
import { MetricsStatsCards } from '../components/metrics/MetricsStatsCards';
import {
  LatencyChart,
  type LatencyDataPoint,
} from '../components/metrics/LatencyChart';
import {
  EventLogList,
  type EventLogEntry,
} from '../components/metrics/EventLogList';
import { StateSnapshot } from '../components/metrics/StateSnapshot';

// Types are now imported from components

export const MetricsPage: React.FC = () => {
  // Connect to metrics store (real WebSocket data)
  const {
    latencyStats,
    eventLog: storeEventLog,
    currentState: storeState,
    connected,
    loading,
    error,
    subscribeToEvents,
    unsubscribeFromEvents,
  } = useMetricsStore();

  // Get the active profile data
  const activeProfile = useActiveProfile();

  // Track latency history for the chart (last 60 data points)
  const [latencyHistory, setLatencyHistory] = useState<LatencyDataPoint[]>([]);

  // Subscribe to WebSocket on mount
  useEffect(() => {
    subscribeToEvents();
    return () => unsubscribeFromEvents();
  }, [subscribeToEvents, unsubscribeFromEvents]);

  // Update latency history when new stats arrive
  useEffect(() => {
    if (latencyStats) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setLatencyHistory((prev) => {
        const newPoint: LatencyDataPoint = {
          timestamp: Date.now(),
          latency: latencyStats.avg / 1000, // Convert microseconds to milliseconds
        };
        const updated = [...prev, newPoint];
        // Keep last 60 seconds
        return updated.slice(-60);
      });
    }
  }, [latencyStats]);

  // Transform store event log to component format
  const eventLog: EventLogEntry[] = useMemo(() => {
    return storeEventLog.map((event) => ({
      id: event.id,
      timestamp: new Date(event.timestamp).getTime(),
      type:
        event.type === 'key_press'
          ? 'press'
          : event.type === 'key_release'
            ? 'release'
            : (event.type as EventLogEntry['type']),
      keyCode: event.keyCode,
      action: event.action,
      latency: event.latencyUs / 1000, // Convert microseconds to milliseconds
      input: event.input,
      output: event.output,
      deviceId: event.deviceId,
      deviceName: event.deviceName,
      mappingType: event.mappingType,
      mappingTriggered: event.mappingTriggered,
    }));
  }, [storeEventLog]);

  // Transform daemon state to component format
  const currentState = useMemo(() => {
    if (!storeState) {
      return {
        activeLayer: 'Base',
        modifiers: [],
        locks: [],
        tapHoldTimers: 0,
        queuedEvents: 0,
      };
    }
    return {
      activeLayer: storeState.activeLayer,
      modifiers: storeState.modifiers,
      locks: storeState.locks,
      tapHoldTimers: storeState.tapHoldPending ? 1 : 0,
      queuedEvents: 0, // Not tracked yet
    };
  }, [storeState]);

  if (loading) {
    return (
      <div className="p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6">
        <div>
          <LoadingSkeleton variant="text" width="250px" height="32px" />
          <LoadingSkeleton
            variant="text"
            width="300px"
            height="20px"
            className="mt-2"
          />
        </div>

        <Card padding="md">
          <LoadingSkeleton variant="rectangular" height="60px" />
        </Card>

        <div className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4">
          <Card padding="md">
            <LoadingSkeleton variant="rectangular" height="60px" />
          </Card>
          <Card padding="md">
            <LoadingSkeleton variant="rectangular" height="60px" />
          </Card>
          <Card padding="md">
            <LoadingSkeleton variant="rectangular" height="60px" />
          </Card>
          <Card padding="md">
            <LoadingSkeleton variant="rectangular" height="60px" />
          </Card>
        </div>

        <Card padding="lg">
          <LoadingSkeleton
            variant="text"
            width="150px"
            height="24px"
            className="mb-4"
          />
          <LoadingSkeleton variant="rectangular" height="300px" />
        </Card>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6">
          <Card padding="lg">
            <LoadingSkeleton
              variant="text"
              width="120px"
              height="24px"
              className="mb-4"
            />
            <LoadingSkeleton variant="rectangular" height="400px" />
          </Card>
          <Card padding="lg">
            <LoadingSkeleton
              variant="text"
              width="140px"
              height="24px"
              className="mb-4"
            />
            <LoadingSkeleton variant="rectangular" height="400px" />
          </Card>
        </div>
      </div>
    );
  }

  return (
    <main
      className="p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6"
      role="main"
      aria-label="Performance Metrics"
    >
      {/* Page Header */}
      <header>
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl md:text-2xl lg:text-3xl font-bold text-slate-100">
              Performance Metrics
            </h1>
            <p className="text-sm md:text-base text-slate-400 mt-2">
              Real-time monitoring and debugging tools
            </p>
          </div>
          {/* Connection Status Indicator */}
          <div className="flex items-center gap-2">
            <div
              className={`w-3 h-3 rounded-full ${
                connected ? 'bg-green-500' : 'bg-red-500'
              }`}
              role="status"
              aria-label={connected ? 'Connected' : 'Disconnected'}
            />
            <span className="text-sm text-slate-400">
              {connected ? 'Live' : 'Disconnected'}
            </span>
          </div>
        </div>
        {/* Error Display */}
        {error && (
          <div className="mt-4 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
            <p className="text-sm text-red-400">{error}</p>
          </div>
        )}
      </header>

      {/* Active Profile Header */}
      <Card padding="md" aria-label="Active profile information">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-blue-500/10 rounded-lg">
            <Activity className="w-5 h-5 text-blue-500" aria-hidden="true" />
          </div>
          <div className="flex-1">
            <p className="text-sm text-slate-400">Active Profile</p>
            {activeProfile ? (
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <Link
                    to={`/config?profile=${encodeURIComponent(
                      activeProfile.name
                    )}`}
                    className="text-lg font-semibold text-blue-400 hover:text-blue-300 transition-colors underline"
                    aria-label={`Edit ${activeProfile.name} configuration`}
                  >
                    {activeProfile.name}
                  </Link>
                  <FileCode
                    className="w-4 h-4 text-slate-400"
                    aria-hidden="true"
                  />
                  <span className="text-sm text-slate-400 font-mono">
                    {activeProfile.name}.rhai
                  </span>
                </div>
                {activeProfile.modifiedAt && (
                  <p className="text-xs text-slate-500">
                    Last modified:{' '}
                    {new Date(activeProfile.modifiedAt).toLocaleString(
                      'en-US',
                      {
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      }
                    )}
                  </p>
                )}
              </div>
            ) : (
              <div className="space-y-1">
                <p className="text-lg font-semibold text-slate-400">
                  {connected ? 'None' : 'Daemon offline'}
                </p>
                {connected && (
                  <Link
                    to="/profiles"
                    className="inline-block text-sm text-blue-400 hover:text-blue-300 transition-colors underline"
                  >
                    Go to Profiles to activate one
                  </Link>
                )}
              </div>
            )}
          </div>
        </div>
      </Card>

      {/* Latency Statistics Cards */}
      <MetricsStatsCards
        latencyStats={latencyStats}
        eventCount={storeEventLog.length}
        connected={connected}
      />

      {/* Latency Chart */}
      <Card aria-labelledby="latency-chart-heading">
        <div className="mb-4">
          <h2
            id="latency-chart-heading"
            className="text-lg md:text-xl font-semibold text-slate-100"
          >
            Latency Over Time
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Last 60 seconds
          </p>
        </div>

        <LatencyChart data={latencyHistory} maxDataPoints={60} height={250} />
      </Card>

      {/* Event Log */}
      <Card aria-labelledby="event-log-heading">
        <div className="mb-4">
          <h2
            id="event-log-heading"
            className="text-lg md:text-xl font-semibold text-slate-100"
          >
            Event Log
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Recent keyboard events ({eventLog.length} total)
          </p>
        </div>

        <EventLogList events={eventLog} height={300} autoScroll={true} />
      </Card>

      {/* State Inspector */}
      <Card aria-labelledby="state-inspector-heading">
        <div className="mb-4">
          <h2
            id="state-inspector-heading"
            className="text-lg md:text-xl font-semibold text-slate-100"
          >
            State Inspector
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Current daemon internal state
          </p>
        </div>

        <StateSnapshot state={currentState} />
      </Card>
    </main>
  );
};

export default MetricsPage;
