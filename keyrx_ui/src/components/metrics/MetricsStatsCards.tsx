import React from 'react';
import { Card } from '../Card';
import { Activity, Clock, Zap, Cpu } from 'lucide-react';
import type { LatencyStats } from '../../types';

/**
 * Props for the MetricsStatsCards component
 */
export interface MetricsStatsCardsProps {
  /** Latency statistics from the daemon */
  latencyStats: LatencyStats | null;
  /** Total number of events processed */
  eventCount: number;
  /** Whether the WebSocket is connected */
  connected: boolean;
}

/**
 * Displays 4 metric stat cards showing latency statistics
 *
 * Renders a responsive grid (2x2 on desktop, stacked on mobile) of cards displaying:
 * - Current latency (Activity icon, blue)
 * - Average latency (Clock icon, green)
 * - Minimum latency (Zap icon, yellow)
 * - Maximum latency (Cpu icon, red)
 *
 * @example
 * ```tsx
 * <MetricsStatsCards
 *   latencyStats={latencyStats}
 *   eventCount={eventLog.length}
 *   connected={connected}
 * />
 * ```
 */
export const MetricsStatsCards: React.FC<MetricsStatsCardsProps> = ({
  latencyStats,
  eventCount: _eventCount,
  connected: _connected,
}) => {
  // Calculate statistics from latency stats (convert microseconds to milliseconds)
  const stats = React.useMemo(() => {
    if (!latencyStats) {
      return { current: 0, avg: 0, min: 0, max: 0 };
    }

    return {
      current: latencyStats.avg / 1000,
      avg: latencyStats.avg / 1000,
      min: latencyStats.min / 1000,
      max: latencyStats.max / 1000,
    };
  }, [latencyStats]);

  // Format latency value with units
  const formatLatency = (latency: number): string => {
    return `${latency.toFixed(2)}ms`;
  };

  return (
    <section
      className="grid grid-cols-2 md:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4"
      aria-label="Latency statistics"
    >
      {/* Current Latency Card */}
      <Card padding="md" aria-label="Current latency">
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
          <div
            className="p-2 sm:p-3 bg-blue-500/10 rounded-lg"
            aria-hidden="true"
          >
            <Activity className="w-5 h-5 sm:w-6 sm:h-6 text-blue-500" />
          </div>
          <div>
            <p className="text-xs sm:text-sm text-slate-400">Current</p>
            <p
              className="text-lg sm:text-2xl font-bold text-slate-100"
              aria-label={`Current latency: ${formatLatency(stats.current)}`}
            >
              {formatLatency(stats.current)}
            </p>
          </div>
        </div>
      </Card>

      {/* Average Latency Card */}
      <Card padding="md" aria-label="Average latency">
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
          <div
            className="p-2 sm:p-3 bg-green-500/10 rounded-lg"
            aria-hidden="true"
          >
            <Clock className="w-5 h-5 sm:w-6 sm:h-6 text-green-500" />
          </div>
          <div>
            <p className="text-xs sm:text-sm text-slate-400">Average</p>
            <p
              className="text-lg sm:text-2xl font-bold text-slate-100"
              aria-label={`Average latency: ${formatLatency(stats.avg)}`}
            >
              {formatLatency(stats.avg)}
            </p>
          </div>
        </div>
      </Card>

      {/* Minimum Latency Card */}
      <Card padding="md" aria-label="Minimum latency">
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

      {/* Maximum Latency Card */}
      <Card padding="md" aria-label="Maximum latency">
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
  );
};
