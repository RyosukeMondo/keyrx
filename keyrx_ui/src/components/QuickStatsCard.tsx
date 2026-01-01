import React from 'react';
import { Card } from './Card';
import { LoadingSkeleton } from './LoadingSkeleton';

interface QuickStats {
  latencyAvg: number; // in milliseconds
  eventsToday: number;
  uptimeSeconds: number;
}

interface QuickStatsCardProps {
  stats?: QuickStats;
  loading?: boolean;
  className?: string;
}

/**
 * QuickStatsCard Component
 *
 * Displays real-time statistics:
 * - Average latency (ms)
 * - Number of events processed today
 * - Daemon uptime (formatted as hours:minutes)
 *
 * Used on: HomePage (dashboard)
 * Design: From design.md Layout 1 - Quick Stats section
 */
export const QuickStatsCard: React.FC<QuickStatsCardProps> = ({
  stats,
  loading = false,
  className = '',
}) => {
  const formatUptime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const formatNumber = (num: number): string => {
    return num.toLocaleString('en-US');
  };

  if (loading) {
    return (
      <Card className={className} aria-label="Quick statistics loading" aria-busy="true">
        <div className="flex flex-col gap-md">
          <LoadingSkeleton variant="text" width="150px" height="24px" />
          <div className="flex flex-wrap gap-md">
            <LoadingSkeleton variant="text" width="100px" height="16px" />
            <LoadingSkeleton variant="text" width="120px" height="16px" />
            <LoadingSkeleton variant="text" width="80px" height="16px" />
          </div>
        </div>
      </Card>
    );
  }

  if (!stats) {
    return (
      <Card className={className} aria-labelledby="quick-stats-title">
        <div className="flex flex-col gap-md">
          <h2 id="quick-stats-title" className="text-lg font-semibold text-slate-100">Quick Stats</h2>
          <p className="text-sm text-slate-400" role="status">
            Statistics unavailable. Ensure the daemon is running.
          </p>
        </div>
      </Card>
    );
  }

  return (
    <Card className={className} aria-labelledby="quick-stats-title">
      <div className="flex flex-col gap-md">
        <h2 id="quick-stats-title" className="text-lg font-semibold text-slate-100">Quick Stats</h2>

        <div className="flex flex-wrap items-center gap-md text-sm text-slate-400" role="list">
          <div className="flex items-center gap-xs" role="listitem">
            <span className="font-medium text-slate-100">Latency:</span>
            <span aria-label={`Average latency ${stats.latencyAvg.toFixed(1)} milliseconds`}>{stats.latencyAvg.toFixed(1)}ms avg</span>
          </div>
          <span aria-hidden="true">•</span>
          <div className="flex items-center gap-xs" role="listitem">
            <span className="font-medium text-slate-100">Events:</span>
            <span aria-label={`${formatNumber(stats.eventsToday)} events processed today`}>{formatNumber(stats.eventsToday)} today</span>
          </div>
          <span aria-hidden="true">•</span>
          <div className="flex items-center gap-xs" role="listitem">
            <span className="font-medium text-slate-100">Uptime:</span>
            <span aria-label={`Daemon uptime ${formatUptime(stats.uptimeSeconds)}`}>{formatUptime(stats.uptimeSeconds)}</span>
          </div>
        </div>
      </div>
    </Card>
  );
};
