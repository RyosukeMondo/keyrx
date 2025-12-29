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
      <Card className={className}>
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
      <Card className={className}>
        <div className="flex flex-col gap-md">
          <h2 className="text-lg font-semibold text-slate-100">Quick Stats</h2>
          <p className="text-sm text-slate-400">
            Statistics unavailable. Ensure the daemon is running.
          </p>
        </div>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <div className="flex flex-col gap-md">
        <h2 className="text-lg font-semibold text-slate-100">Quick Stats</h2>

        <div className="flex flex-wrap items-center gap-md text-sm text-slate-400">
          <div className="flex items-center gap-xs">
            <span className="font-medium text-slate-100">Latency:</span>
            <span>{stats.latencyAvg.toFixed(1)}ms avg</span>
          </div>
          <span>•</span>
          <div className="flex items-center gap-xs">
            <span className="font-medium text-slate-100">Events:</span>
            <span>{formatNumber(stats.eventsToday)} today</span>
          </div>
          <span>•</span>
          <div className="flex items-center gap-xs">
            <span className="font-medium text-slate-100">Uptime:</span>
            <span>{formatUptime(stats.uptimeSeconds)}</span>
          </div>
        </div>
      </div>
    </Card>
  );
};
