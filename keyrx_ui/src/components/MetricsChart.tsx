/**
 * MetricsChart Component
 *
 * Visualizes latency metrics over time using a line chart.
 * Displays avg, p95, and p99 latencies with a 5ms performance target reference line.
 *
 * @component
 * @example
 * ```tsx
 * <MetricsChart data={latencyHistory} />
 * ```
 */

import React, { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
} from "recharts";
import type { LatencyMetrics } from "../types/rpc";

interface MetricsChartProps {
  /** Array of latency metrics to display */
  data: LatencyMetrics[];
}

/**
 * MetricsChart displays latency metrics over time with avg, p95, and p99 lines.
 *
 * Features:
 * - Converts microseconds to milliseconds for readability
 * - Shows 5ms reference line as performance target
 * - Dark theme matching TailwindCSS slate palette
 * - Responsive width with fixed 300px height
 * - Tooltip shows values on hover
 *
 * @param props - Component props
 * @param props.data - Array of LatencyMetrics to visualize
 */
export function MetricsChart({ data }: MetricsChartProps) {
  /**
   * Transform latency data from microseconds to milliseconds.
   * Creates chart-friendly format with index, avg, p95, p99.
   */
  const chartData = useMemo(() => {
    return data.map((metrics, index) => ({
      index,
      avg: metrics.avg / 1000, // Convert μs to ms
      p95: metrics.p95 / 1000, // Convert μs to ms
      p99: metrics.p99 / 1000, // Convert μs to ms
    }));
  }, [data]);

  // Dark theme colors matching TailwindCSS slate palette
  const colors = {
    grid: "#334155", // slate-700
    text: "#cbd5e1", // slate-300
    avg: "#3b82f6", // blue-500
    p95: "#f97316", // orange-500
    p99: "#ef4444", // red-500
    target: "#dc2626", // red-600
    background: "#1e293b", // slate-800
  };

  return (
    <div className="rounded-lg bg-slate-800 p-4">
      <h3 className="mb-4 text-lg font-semibold text-slate-200">Latency Metrics</h3>
      {data.length === 0 ? (
        <div className="flex h-[300px] items-center justify-center">
          <p className="text-slate-400">No latency data available yet...</p>
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={300}>
          <LineChart
            data={chartData}
            margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
          >
            <CartesianGrid strokeDasharray="3 3" stroke={colors.grid} />
            <XAxis
              dataKey="index"
              stroke={colors.text}
              tick={{ fill: colors.text }}
              label={{ value: "Time", position: "insideBottom", offset: -5, fill: colors.text }}
            />
            <YAxis
              stroke={colors.text}
              tick={{ fill: colors.text }}
              label={{ value: "Latency (ms)", angle: -90, position: "insideLeft", fill: colors.text }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: colors.background,
                border: `1px solid ${colors.grid}`,
                borderRadius: "0.5rem",
                color: colors.text,
              }}
              formatter={(value: number) => `${value.toFixed(2)} ms`}
            />
            <Legend wrapperStyle={{ color: colors.text }} />
            <ReferenceLine
              y={5}
              stroke={colors.target}
              strokeDasharray="3 3"
              label={{
                value: "Target (5ms)",
                position: "right",
                fill: colors.target,
                fontSize: 12,
              }}
            />
            <Line
              type="monotone"
              dataKey="avg"
              stroke={colors.avg}
              name="Average"
              dot={false}
              strokeWidth={2}
            />
            <Line
              type="monotone"
              dataKey="p95"
              stroke={colors.p95}
              name="P95"
              dot={false}
              strokeWidth={2}
            />
            <Line
              type="monotone"
              dataKey="p99"
              stroke={colors.p99}
              name="P99"
              dot={false}
              strokeWidth={2}
            />
          </LineChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}
