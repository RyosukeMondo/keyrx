/**
 * MetricsChart Component
 *
 * Line chart displaying latency metrics over a 60-second rolling window.
 * Highlights latency values >5ms in red.
 */

import { useEffect, useState } from 'react';
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
} from 'recharts';
import { useDashboardStore } from '../store/dashboardStore';
import type { LatencyStats } from '../store/dashboardStore';
import './MetricsChart.css';

/**
 * Data point for the chart
 */
interface ChartDataPoint {
  /** Timestamp in seconds (for X-axis) */
  time: number;
  /** Formatted time string for display */
  timeLabel: string;
  /** Average latency in milliseconds */
  avg: number;
  /** P95 latency in milliseconds */
  p95: number;
  /** P99 latency in milliseconds */
  p99: number;
}

/**
 * Maximum number of data points to display (60 seconds of data)
 */
const MAX_DATA_POINTS = 60;

/**
 * Latency threshold in milliseconds (values above this are highlighted in red)
 */
const LATENCY_THRESHOLD_MS = 5;

/**
 * Metrics chart component with 60-second rolling window
 */
export function MetricsChart() {
  const metrics = useDashboardStore((state) => state.metrics);
  const [chartData, setChartData] = useState<ChartDataPoint[]>([]);

  // Update chart data when new metrics arrive
  useEffect(() => {
    if (!metrics) return;

    const now = Date.now();
    const newPoint: ChartDataPoint = {
      time: now / 1000, // Convert to seconds
      timeLabel: new Date(now).toLocaleTimeString(),
      avg: metrics.avg / 1000, // Convert microseconds to milliseconds
      p95: metrics.p95 / 1000,
      p99: metrics.p99 / 1000,
    };

    setChartData((prev) => {
      const updated = [...prev, newPoint];
      // Keep only last 60 data points (60-second window)
      if (updated.length > MAX_DATA_POINTS) {
        updated.shift();
      }
      return updated;
    });
  }, [metrics]);

  // Show empty state if no data
  if (chartData.length === 0) {
    return (
      <div className="metrics-chart-empty">
        <p>Waiting for latency data...</p>
      </div>
    );
  }

  return (
    <div className="metrics-chart">
      {/* Current stats summary */}
      <div className="metrics-summary">
        {metrics && (
          <>
            <div className="metric-stat">
              <span className="metric-label">Avg:</span>
              <span className={`metric-value ${metrics.avg / 1000 > LATENCY_THRESHOLD_MS ? 'high' : ''}`}>
                {(metrics.avg / 1000).toFixed(2)}ms
              </span>
            </div>
            <div className="metric-stat">
              <span className="metric-label">P95:</span>
              <span className={`metric-value ${metrics.p95 / 1000 > LATENCY_THRESHOLD_MS ? 'high' : ''}`}>
                {(metrics.p95 / 1000).toFixed(2)}ms
              </span>
            </div>
            <div className="metric-stat">
              <span className="metric-label">P99:</span>
              <span className={`metric-value ${metrics.p99 / 1000 > LATENCY_THRESHOLD_MS ? 'high' : ''}`}>
                {(metrics.p99 / 1000).toFixed(2)}ms
              </span>
            </div>
          </>
        )}
      </div>

      {/* Line chart */}
      <ResponsiveContainer width="100%" height={300}>
        <LineChart
          data={chartData}
          margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#e0e0e0" />
          <XAxis
            dataKey="timeLabel"
            stroke="#666"
            tick={{ fontSize: 12 }}
            interval="preserveStartEnd"
          />
          <YAxis
            stroke="#666"
            tick={{ fontSize: 12 }}
            label={{ value: 'Latency (ms)', angle: -90, position: 'insideLeft' }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#fff',
              border: '1px solid #ccc',
              borderRadius: '4px',
              padding: '8px',
            }}
            formatter={(value: number) => `${value.toFixed(2)}ms`}
          />
          <Legend />

          {/* Reference line at 5ms threshold */}
          <ReferenceLine
            y={LATENCY_THRESHOLD_MS}
            stroke="#f44336"
            strokeDasharray="3 3"
            label={{ value: '5ms threshold', position: 'right', fill: '#f44336' }}
          />

          {/* Latency lines */}
          <Line
            type="monotone"
            dataKey="avg"
            stroke="#2196f3"
            strokeWidth={2}
            name="Average"
            dot={false}
            isAnimationActive={false}
          />
          <Line
            type="monotone"
            dataKey="p95"
            stroke="#ff9800"
            strokeWidth={2}
            name="P95"
            dot={false}
            isAnimationActive={false}
          />
          <Line
            type="monotone"
            dataKey="p99"
            stroke="#f44336"
            strokeWidth={2}
            name="P99"
            dot={false}
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
