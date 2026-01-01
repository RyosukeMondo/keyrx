/**
 * MetricsChart - Visualize latency metrics over time
 *
 * This component renders a line chart showing average, P95, and P99 latency
 * metrics with a 5ms reference line. Will be fully implemented in Task 20.
 *
 * @placeholder This is a temporary stub for Task 18 integration
 */

import type { LatencyMetrics } from '../types/rpc';

interface MetricsChartProps {
  data: LatencyMetrics[];
}

/**
 * MetricsChart component - Displays latency metrics
 * @placeholder Full implementation in Task 20
 */
export function MetricsChart({ data }: MetricsChartProps) {
  const latestMetrics = data[data.length - 1];

  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold mb-4">Latency Metrics</h2>
      {data.length === 0 ? (
        <p className="text-slate-400">No data yet...</p>
      ) : (
        <div className="space-y-2 text-sm">
          <p>
            <span className="text-slate-400">Samples:</span> {data.length}
          </p>
          {latestMetrics && (
            <>
              <p>
                <span className="text-slate-400">Average:</span>{' '}
                {(latestMetrics.avg / 1000).toFixed(2)} ms
              </p>
              <p>
                <span className="text-slate-400">P95:</span>{' '}
                {(latestMetrics.p95 / 1000).toFixed(2)} ms
              </p>
              <p>
                <span className="text-slate-400">P99:</span>{' '}
                {(latestMetrics.p99 / 1000).toFixed(2)} ms
              </p>
            </>
          )}
          <p className="text-slate-500 mt-4">
            Chart visualization will be implemented in Task 20
          </p>
        </div>
      )}
    </div>
  );
}
