import React from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
} from 'recharts';

/**
 * Data point for latency chart
 */
export interface LatencyDataPoint {
  timestamp: number;
  latency: number;
}

/**
 * Props for LatencyChart component
 */
export interface LatencyChartProps {
  /** Array of latency data points to display */
  data: LatencyDataPoint[];
  /** Maximum number of data points to display (default: 60) */
  maxDataPoints?: number;
  /** Height of the chart in pixels (default: 250) */
  height?: number;
}

/**
 * Format timestamp for display on X axis
 * @param timestamp - Unix timestamp in milliseconds
 * @returns Formatted time string (HH:MM:SS)
 */
export const formatTimestamp = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
};

/**
 * Format latency value for display
 * @param latency - Latency value in milliseconds
 * @returns Formatted latency string with unit (e.g., "1.23ms")
 */
export const formatLatency = (latency: number): string => {
  return `${latency.toFixed(2)}ms`;
};

/**
 * LatencyChart component displays a line chart of latency measurements over time.
 *
 * @example
 * ```tsx
 * <LatencyChart
 *   data={[
 *     { timestamp: 1234567890, latency: 1.23 },
 *     { timestamp: 1234567891, latency: 1.45 },
 *   ]}
 *   maxDataPoints={60}
 * />
 * ```
 */
export const LatencyChart: React.FC<LatencyChartProps> = ({
  data,
  maxDataPoints = 60,
  height = 250,
}) => {
  // Limit data to maxDataPoints (most recent)
  const chartData = React.useMemo(() => {
    return data.slice(-maxDataPoints);
  }, [data, maxDataPoints]);

  // Handle empty data case
  if (chartData.length === 0) {
    return (
      <div
        className="flex items-center justify-center bg-slate-800 rounded-lg"
        style={{ height }}
        role="img"
        aria-label="No latency data available"
      >
        <p className="text-slate-500 text-sm">No data available</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={height} className="md:h-[300px]">
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
        <XAxis
          dataKey="timestamp"
          tickFormatter={formatTimestamp}
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
          labelFormatter={(timestamp) => formatTimestamp(Number(timestamp))}
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
  );
};

export default LatencyChart;
