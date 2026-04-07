import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { Card } from '@/components/Card';
import { Activity, FileCode, Pause, Play, Download, Search } from 'lucide-react';
import { LoadingSkeleton } from '@/components/LoadingSkeleton';
import { useMetricsStore } from '@/stores/metricsStore';
import { useActiveProfile } from '@/hooks/useProfiles';
import { MetricsStatsCards } from '@/components/metrics/MetricsStatsCards';
import {
  LatencyChart,
  type LatencyDataPoint,
} from '@/components/metrics/LatencyChart';
import {
  EventLogList,
  type EventLogEntry,
} from '@/components/metrics/EventLogList';
import { StateSnapshot } from '@/components/metrics/StateSnapshot';

/** Trigger a file download in the browser */
function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export const MonitorPage: React.FC = () => {
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

  // Pause / filter state
  const [isPaused, setIsPaused] = useState(false);
  const [keyFilter, setKeyFilter] = useState('');
  const pausedEventsRef = useRef<EventLogEntry[]>([]);

  // Subscribe to WebSocket on mount
  useEffect(() => {
    subscribeToEvents();
    return () => unsubscribeFromEvents();
  }, [subscribeToEvents, unsubscribeFromEvents]);

  // Update latency history when new stats arrive (skip when paused)
  useEffect(() => {
    if (latencyStats && !isPaused) {
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
  }, [latencyStats, isPaused]);

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

  // Snapshot events when pausing
  useEffect(() => {
    if (isPaused) {
      pausedEventsRef.current = eventLog;
    }
  }, [isPaused, eventLog]);

  // Use paused snapshot or live feed, then apply key filter
  const filteredEventLog: EventLogEntry[] = useMemo(() => {
    const source = isPaused ? pausedEventsRef.current : eventLog;
    if (!keyFilter) return source;
    const lowerFilter = keyFilter.toLowerCase();
    return source.filter(
      (e) =>
        e.keyCode.toLowerCase().includes(lowerFilter) ||
        (e.input?.toLowerCase().includes(lowerFilter) ?? false) ||
        (e.output?.toLowerCase().includes(lowerFilter) ?? false),
    );
  }, [isPaused, eventLog, keyFilter]);

  // Export handlers
  const handleExportCSV = useCallback(() => {
    const header = 'timestamp,type,keyCode,input,output,latency_ms,device,mappingType';
    const rows = filteredEventLog.map((e) =>
      [
        new Date(e.timestamp).toISOString(),
        e.type,
        e.keyCode,
        e.input ?? '',
        e.output ?? '',
        e.latency.toFixed(2),
        e.deviceName ?? e.deviceId ?? '',
        e.mappingType ?? '',
      ].join(','),
    );
    downloadFile([header, ...rows].join('\n'), 'keyrx-events.csv', 'text/csv');
  }, [filteredEventLog]);

  const handleExportJSON = useCallback(() => {
    const json = JSON.stringify(filteredEventLog, null, 2);
    downloadFile(json, 'keyrx-events.json', 'application/json');
  }, [filteredEventLog]);

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
      activeLayer: storeState.layer ?? storeState.activeLayer ?? 'Base',
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
      aria-label="Monitor"
    >
      {/* Page Header */}
      <header>
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl md:text-2xl lg:text-3xl font-bold text-slate-100">
              Monitor
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
                    to={`/profiles/${encodeURIComponent(
                      activeProfile.name
                    )}/config`}
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
                    to="/"
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

      {/* Event Log + State Inspector — side-by-side on large screens */}
      <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto] gap-4 md:gap-6">
        <Card aria-labelledby="event-log-heading">
          <div className="mb-4">
            <div className="flex items-center justify-between flex-wrap gap-2">
              <div>
                <h2
                  id="event-log-heading"
                  className="text-lg md:text-xl font-semibold text-slate-100"
                >
                  Event Log
                </h2>
                <p className="text-xs md:text-sm text-slate-400 mt-1">
                  {isPaused ? 'Paused' : 'Live'} &middot;{' '}
                  {filteredEventLog.length} event{filteredEventLog.length !== 1 ? 's' : ''}
                  {keyFilter && ` (filtered)`}
                </p>
              </div>

              {/* Toolbar */}
              <div className="flex items-center gap-2 flex-wrap">
                <button
                  type="button"
                  onClick={() => setIsPaused((p) => !p)}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200 transition-colors"
                  aria-label={isPaused ? 'Resume event stream' : 'Pause event stream'}
                >
                  {isPaused ? (
                    <Play className="w-4 h-4" aria-hidden="true" />
                  ) : (
                    <Pause className="w-4 h-4" aria-hidden="true" />
                  )}
                  {isPaused ? 'Resume' : 'Pause'}
                </button>

                <div className="relative">
                  <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" aria-hidden="true" />
                  <input
                    type="text"
                    value={keyFilter}
                    onChange={(e) => setKeyFilter(e.target.value)}
                    placeholder="Filter by key..."
                    className="pl-8 pr-3 py-1.5 text-sm rounded-lg bg-slate-800 border border-slate-600 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-blue-500 w-40"
                    aria-label="Filter events by key"
                  />
                </div>

                <button
                  type="button"
                  onClick={handleExportCSV}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200 transition-colors"
                  aria-label="Export events as CSV"
                >
                  <Download className="w-4 h-4" aria-hidden="true" />
                  CSV
                </button>
                <button
                  type="button"
                  onClick={handleExportJSON}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200 transition-colors"
                  aria-label="Export events as JSON"
                >
                  <Download className="w-4 h-4" aria-hidden="true" />
                  JSON
                </button>
              </div>
            </div>
          </div>

          <EventLogList events={filteredEventLog} height={300} autoScroll={!isPaused} />
        </Card>

        {/* State Inspector */}
        <Card aria-labelledby="state-inspector-heading" className="lg:w-80">
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
      </div>
    </main>
  );
};

export default MonitorPage;
