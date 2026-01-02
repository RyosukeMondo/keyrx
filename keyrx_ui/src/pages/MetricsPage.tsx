import React, { useState, useEffect, useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
} from 'recharts';
import { FixedSizeList as List } from 'react-window';
import { Card } from '../components/Card';
import { Activity, Clock, Cpu, Zap } from 'lucide-react';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import { useMetricsStore } from '../stores/metricsStore';

interface LatencyDataPoint {
  timestamp: number;
  latency: number;
}

interface EventLogEntry {
  id: string;
  timestamp: number;
  type: 'press' | 'release' | 'tap' | 'hold' | 'macro' | 'layer_switch';
  keyCode: string;
  action?: string;
  latency: number;
}

interface StateSnapshot {
  activeLayer: string;
  modifiers: string[];
  locks: string[];
  tapHoldTimers: number;
  queuedEvents: number;
}

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
      type: event.type === 'key_press' ? 'press' :
            event.type === 'key_release' ? 'release' :
            event.type as EventLogEntry['type'],
      keyCode: event.keyCode,
      action: event.action,
      latency: event.latencyUs / 1000, // Convert microseconds to milliseconds
    }));
  }, [storeEventLog]);

  // Transform daemon state to component format
  const currentState: StateSnapshot = useMemo(() => {
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

  // Calculate statistics from real latency stats
  const stats = useMemo(() => {
    if (!latencyStats) {
      return { avg: 0, min: 0, max: 0, current: 0 };
    }

    return {
      current: latencyStats.avg / 1000, // Convert microseconds to milliseconds
      avg: latencyStats.avg / 1000,
      min: latencyStats.min / 1000,
      max: latencyStats.max / 1000,
    };
  }, [latencyStats]);

  // Format timestamp for display
  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    });
  };

  // Format latency for display
  const formatLatency = (latency: number) => {
    return `${latency.toFixed(2)}ms`;
  };

  // Event log row renderer for react-window
  const EventRow = ({
    index,
    style,
  }: {
    index: number;
    style: React.CSSProperties;
  }) => {
    const event = eventLog[index];
    if (!event) return null;

    const typeColors = {
      press: 'text-green-400',
      release: 'text-red-400',
      tap: 'text-blue-400',
      hold: 'text-yellow-400',
      macro: 'text-purple-400',
      layer_switch: 'text-cyan-400',
    };

    return (
      <div
        style={style}
        className="flex items-center gap-4 px-4 text-sm font-mono border-b border-slate-700 hover:bg-slate-700/50"
      >
        <span className="w-32 text-slate-400">{formatTime(event.timestamp)}</span>
        <span className={`w-24 ${typeColors[event.type]}`}>{event.type}</span>
        <span className="w-32 text-slate-200">{event.keyCode}</span>
        {event.action && (
          <span className="w-32 text-slate-300">→ {event.action}</span>
        )}
        <span className="w-20 text-slate-400">{formatLatency(event.latency)}</span>
      </div>
    );
  };

  if (loading) {
    return (
      <div className="p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6">
        <div>
          <LoadingSkeleton variant="text" width="250px" height="32px" />
          <LoadingSkeleton variant="text" width="300px" height="20px" className="mt-2" />
        </div>

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
          <LoadingSkeleton variant="text" width="150px" height="24px" className="mb-4" />
          <LoadingSkeleton variant="rectangular" height="300px" />
        </Card>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6">
          <Card padding="lg">
            <LoadingSkeleton variant="text" width="120px" height="24px" className="mb-4" />
            <LoadingSkeleton variant="rectangular" height="400px" />
          </Card>
          <Card padding="lg">
            <LoadingSkeleton variant="text" width="140px" height="24px" className="mb-4" />
            <LoadingSkeleton variant="rectangular" height="400px" />
          </Card>
        </div>
      </div>
    );
  }

  return (
    <main className="p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6" role="main" aria-label="Performance Metrics">
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

      {/* Latency Statistics Cards - responsive grid */}
      <section className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4" aria-label="Latency statistics">
        <Card padding="md" aria-label="Current latency">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-blue-500/10 rounded-lg" aria-hidden="true">
              <Activity className="w-5 h-5 sm:w-6 sm:h-6 text-blue-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Current</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100" aria-label={`Current latency: ${formatLatency(stats.current)}`}>
                {formatLatency(stats.current)}
              </p>
            </div>
          </div>
        </Card>

        <Card padding="md" aria-label="Average latency">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-green-500/10 rounded-lg" aria-hidden="true">
              <Clock className="w-5 h-5 sm:w-6 sm:h-6 text-green-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Average</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100" aria-label={`Average latency: ${formatLatency(stats.avg)}`}>
                {formatLatency(stats.avg)}
              </p>
            </div>
          </div>
        </Card>

        <Card padding="md">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-yellow-500/10 rounded-lg">
              <Zap className="w-5 h-5 sm:w-6 sm:h-6 text-yellow-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Min</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100">
                {formatLatency(stats.min)}
              </p>
            </div>
          </div>
        </Card>

        <Card padding="md">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-red-500/10 rounded-lg">
              <Cpu className="w-5 h-5 sm:w-6 sm:h-6 text-red-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Max</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100">
                {formatLatency(stats.max)}
              </p>
            </div>
          </div>
        </Card>
      </section>

      {/* Latency Chart */}
      <Card>
        <div className="mb-4">
          <h2 className="text-lg md:text-xl font-semibold text-slate-100">
            Latency Over Time
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">Last 60 seconds</p>
        </div>

        <ResponsiveContainer width="100%" height={250} className="md:h-[300px]">
          <LineChart data={latencyHistory}>
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
            <XAxis
              dataKey="timestamp"
              tickFormatter={(ts) => {
                const date = new Date(ts);
                return date.toLocaleTimeString('en-US', {
                  hour12: false,
                  minute: '2-digit',
                  second: '2-digit',
                });
              }}
              stroke="#94A3B8"
              style={{ fontSize: '12px' }}
            />
            <YAxis
              stroke="#94A3B8"
              style={{ fontSize: '12px' }}
              label={{
                value: 'Latency (ms)',
                angle: -90,
                position: 'insideLeft',
                style: { fill: '#94A3B8', fontSize: '12px' },
              }}
            />
            <RechartsTooltip
              contentStyle={{
                backgroundColor: '#1E293B',
                border: '1px solid #334155',
                borderRadius: '8px',
                color: '#F1F5F9',
              }}
              labelFormatter={(ts) => formatTime(Number(ts))}
              formatter={(value: number) => [formatLatency(value), 'Latency']}
            />
            <Line
              type="monotone"
              dataKey="latency"
              stroke="#3B82F6"
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </Card>

      {/* Event Log */}
      <Card>
        <div className="mb-4">
          <h2 className="text-lg md:text-xl font-semibold text-slate-100">Event Log</h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Recent keyboard events ({eventLog.length} total)
          </p>
        </div>

        {/* Table Header - hide some columns on mobile */}
        <div className="hidden md:flex items-center gap-4 px-4 py-2 bg-slate-800 border-b border-slate-700 text-sm font-semibold text-slate-300">
          <span className="w-32">Timestamp</span>
          <span className="w-24">Type</span>
          <span className="w-32">Key Code</span>
          <span className="w-32">Action</span>
          <span className="w-20">Latency</span>
        </div>

        {/* Virtual Scrolling List */}
        <List
          height={300}
          itemCount={eventLog.length}
          itemSize={40}
          width="100%"
          className="bg-slate-900"
        >
          {EventRow}
        </List>
      </Card>

      {/* State Inspector */}
      <Card>
        <div className="mb-4">
          <h2 className="text-lg md:text-xl font-semibold text-slate-100">
            State Inspector
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">
            Current daemon internal state
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 md:gap-4">
          <div className="bg-slate-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">
              Active Layer
            </h3>
            <p className="text-lg font-mono text-blue-400">
              {currentState.activeLayer}
            </p>
          </div>

          <div className="bg-slate-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">
              Tap/Hold Timers
            </h3>
            <p className="text-lg font-mono text-yellow-400">
              {currentState.tapHoldTimers} active
            </p>
          </div>

          <div className="bg-slate-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">
              Active Modifiers
            </h3>
            <p className="text-lg font-mono text-green-400">
              {currentState.modifiers.length > 0
                ? currentState.modifiers.join(', ')
                : 'None'}
            </p>
          </div>

          <div className="bg-slate-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">
              Active Locks
            </h3>
            <p className="text-lg font-mono text-purple-400">
              {currentState.locks.length > 0
                ? currentState.locks.join(', ')
                : 'None'}
            </p>
          </div>

          <div className="bg-slate-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">
              Queued Events
            </h3>
            <p className="text-lg font-mono text-red-400">
              {currentState.queuedEvents}
            </p>
          </div>
        </div>
      </Card>
    </main>
  );
};

export default MetricsPage;
