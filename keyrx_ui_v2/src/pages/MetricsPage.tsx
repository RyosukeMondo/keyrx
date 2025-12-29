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
  // Mock data - in production, this would come from WebSocket
  const [latencyData, setLatencyData] = useState<LatencyDataPoint[]>([]);
  const [eventLog, setEventLog] = useState<EventLogEntry[]>([]);
  const [currentState, setCurrentState] = useState<StateSnapshot>({
    activeLayer: 'Base',
    modifiers: [],
    locks: [],
    tapHoldTimers: 0,
    queuedEvents: 0,
  });

  // Generate mock latency data
  useEffect(() => {
    const generateMockData = () => {
      const now = Date.now();
      const data: LatencyDataPoint[] = [];
      for (let i = 60; i >= 0; i--) {
        data.push({
          timestamp: now - i * 1000,
          latency: Math.random() * 5 + 0.5, // Random latency between 0.5-5.5ms
        });
      }
      setLatencyData(data);
    };

    generateMockData();

    // Update every second with new data point
    const interval = setInterval(() => {
      setLatencyData((prev) => {
        const newData = [
          ...prev.slice(1),
          {
            timestamp: Date.now(),
            latency: Math.random() * 5 + 0.5,
          },
        ];
        return newData;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  // Generate mock event log
  useEffect(() => {
    const generateMockEvent = (): EventLogEntry => {
      const types: EventLogEntry['type'][] = [
        'press',
        'release',
        'tap',
        'hold',
        'macro',
        'layer_switch',
      ];
      const keyCodes = [
        'KEY_A',
        'KEY_S',
        'KEY_D',
        'KEY_F',
        'KEY_SPACE',
        'KEY_ENTER',
        'KEY_LEFTSHIFT',
      ];

      return {
        id: `${Date.now()}-${Math.random()}`,
        timestamp: Date.now(),
        type: types[Math.floor(Math.random() * types.length)],
        keyCode: keyCodes[Math.floor(Math.random() * keyCodes.length)],
        action: Math.random() > 0.5 ? 'KEY_B' : undefined,
        latency: Math.random() * 3 + 0.2,
      };
    };

    // Add initial events
    const initialEvents: EventLogEntry[] = [];
    for (let i = 0; i < 100; i++) {
      initialEvents.push(generateMockEvent());
    }
    setEventLog(initialEvents);

    // Add new events periodically
    const interval = setInterval(() => {
      const newEvent = generateMockEvent();
      setEventLog((prev) => [newEvent, ...prev].slice(0, 1000)); // Keep last 1000 events
    }, 500);

    return () => clearInterval(interval);
  }, []);

  // Calculate statistics
  const stats = useMemo(() => {
    if (latencyData.length === 0)
      return { avg: 0, min: 0, max: 0, current: 0 };

    const latencies = latencyData.map((d) => d.latency);
    const current = latencies[latencies.length - 1];
    const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    const min = Math.min(...latencies);
    const max = Math.max(...latencies);

    return { avg, min, max, current };
  }, [latencyData]);

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

  return (
    <div className="p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-xl md:text-2xl lg:text-3xl font-bold text-slate-100">
          Performance Metrics
        </h1>
        <p className="text-sm md:text-base text-slate-400 mt-2">
          Real-time monitoring and debugging tools
        </p>
      </div>

      {/* Latency Statistics Cards - responsive grid */}
      <div className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4">
        <Card padding="md">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-blue-500/10 rounded-lg">
              <Activity className="w-5 h-5 sm:w-6 sm:h-6 text-blue-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Current</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100">
                {formatLatency(stats.current)}
              </p>
            </div>
          </div>
        </Card>

        <Card padding="md">
          <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
            <div className="p-2 sm:p-3 bg-green-500/10 rounded-lg">
              <Clock className="w-5 h-5 sm:w-6 sm:h-6 text-green-500" />
            </div>
            <div>
              <p className="text-xs sm:text-sm text-slate-400">Average</p>
              <p className="text-lg sm:text-2xl font-bold text-slate-100">
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
      </div>

      {/* Latency Chart */}
      <Card>
        <div className="mb-4">
          <h2 className="text-lg md:text-xl font-semibold text-slate-100">
            Latency Over Time
          </h2>
          <p className="text-xs md:text-sm text-slate-400 mt-1">Last 60 seconds</p>
        </div>

        <ResponsiveContainer width="100%" height={250} className="md:h-[300px]">
          <LineChart data={latencyData}>
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
    </div>
  );
};
