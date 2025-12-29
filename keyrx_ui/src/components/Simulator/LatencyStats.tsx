/**
 * LatencyStats component displays simulation performance metrics.
 *
 * Shows latency statistics (min, avg, max, p95, p99) with warnings for
 * values exceeding performance thresholds.
 */

import React from 'react';
import type { LatencyStats as LatencyStatsType } from '../../wasm/core';
import './LatencyStats.css';

interface LatencyStatsProps {
  /** Latency statistics from simulation */
  stats: LatencyStatsType;
}

/**
 * Formats microsecond value to a human-readable string.
 */
function formatLatency(us: number): string {
  if (us < 1000) {
    return `${us.toFixed(2)}μs`;
  } else if (us < 1000000) {
    return `${(us / 1000).toFixed(2)}ms`;
  } else {
    return `${(us / 1000000).toFixed(2)}s`;
  }
}

/**
 * Determines performance level based on latency value.
 */
function getPerformanceLevel(us: number): 'excellent' | 'good' | 'warning' {
  if (us < 1000) return 'excellent'; // <1ms
  if (us < 5000) return 'good'; // 1-5ms
  return 'warning'; // >5ms
}

/**
 * LatencyStats displays performance metrics for simulation.
 */
export function LatencyStats({ stats }: LatencyStatsProps): React.JSX.Element {
  const maxLevel = getPerformanceLevel(stats.max_us);
  const avgLevel = getPerformanceLevel(stats.avg_us);

  // Check if all metrics are excellent (<1ms)
  const allExcellent = stats.max_us < 1000;

  return (
    <div className="latency-stats" role="region" aria-label="Latency Statistics">
      <div className="latency-stats-header">
        <h3>Performance Metrics</h3>
        {allExcellent && (
          <span className="performance-badge excellent" aria-label="Excellent performance">
            ✓ All metrics &lt;1ms
          </span>
        )}
        {maxLevel === 'warning' && (
          <span className="performance-badge warning" aria-label="Performance warning">
            ⚠ Max latency exceeds 5ms
          </span>
        )}
      </div>

      <table className="latency-table">
        <thead>
          <tr>
            <th scope="col">Metric</th>
            <th scope="col">Value</th>
            <th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Minimum</td>
            <td className={`latency-value ${getPerformanceLevel(stats.min_us)}`}>
              {formatLatency(stats.min_us)}
            </td>
            <td className="status-cell">
              {getStatusIcon(getPerformanceLevel(stats.min_us))}
            </td>
          </tr>
          <tr>
            <td>Average</td>
            <td className={`latency-value ${avgLevel}`}>
              {formatLatency(stats.avg_us)}
            </td>
            <td className="status-cell">
              {getStatusIcon(avgLevel)}
            </td>
          </tr>
          <tr>
            <td>Maximum</td>
            <td className={`latency-value ${maxLevel}`}>
              {formatLatency(stats.max_us)}
            </td>
            <td className="status-cell">
              {getStatusIcon(maxLevel)}
            </td>
          </tr>
          <tr>
            <td>
              <abbr title="95th percentile: 95% of events processed faster than this">
                P95
              </abbr>
            </td>
            <td className={`latency-value ${getPerformanceLevel(stats.p95_us)}`}>
              {formatLatency(stats.p95_us)}
            </td>
            <td className="status-cell">
              {getStatusIcon(getPerformanceLevel(stats.p95_us))}
            </td>
          </tr>
          <tr>
            <td>
              <abbr title="99th percentile: 99% of events processed faster than this">
                P99
              </abbr>
            </td>
            <td className={`latency-value ${getPerformanceLevel(stats.p99_us)}`}>
              {formatLatency(stats.p99_us)}
            </td>
            <td className="status-cell">
              {getStatusIcon(getPerformanceLevel(stats.p99_us))}
            </td>
          </tr>
        </tbody>
      </table>

      <div className="latency-footer">
        <p className="latency-info">
          <strong>Target:</strong> All events should complete in &lt;1ms for optimal performance.
        </p>
        <p className="latency-info">
          <strong>Warning threshold:</strong> Max latency &gt;5ms may cause noticeable delays.
        </p>
      </div>
    </div>
  );
}

/**
 * Returns status icon for performance level.
 */
function getStatusIcon(level: 'excellent' | 'good' | 'warning'): string {
  switch (level) {
    case 'excellent':
      return '✓';
    case 'good':
      return '○';
    case 'warning':
      return '⚠';
  }
}
