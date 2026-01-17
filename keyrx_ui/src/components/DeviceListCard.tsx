import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from './Card';
import { Button } from './Button';
import { LoadingSkeleton } from './LoadingSkeleton';
import { ErrorState } from './ErrorState';
import { useDevices } from '../hooks/useDevices';

interface DeviceListCardProps {
  className?: string;
}

/**
 * DeviceListCard Component
 *
 * Displays connected keyboard devices with:
 * - Device name and icon
 * - USB identifier
 * - Scope (global/device-specific)
 * - Layout preset
 * - Active status indicator
 * - Manage Devices button to navigate to devices page
 *
 * Fetches device data automatically using useDevices hook.
 * Shows loading skeleton while fetching, error state with retry on failure.
 *
 * Used on: HomePage (dashboard)
 * Design: From design.md Layout 1 - Connected Devices section
 */
export const DeviceListCard: React.FC<DeviceListCardProps> = ({
  className = '',
}) => {
  const navigate = useNavigate();
  const { data: devices = [], isLoading, error, refetch } = useDevices();

  // Error state with retry
  if (error) {
    return (
      <Card className={className}>
        <div className="flex flex-col gap-md">
          <h2 className="text-lg font-semibold text-slate-100">
            Connected Devices
          </h2>
          <ErrorState
            title="Failed to load devices"
            message={
              error instanceof Error
                ? error.message
                : 'An error occurred while fetching devices'
            }
            onRetry={() => refetch()}
            retryLabel="Retry"
          />
        </div>
      </Card>
    );
  }

  // Loading state
  if (isLoading) {
    return (
      <Card className={className}>
        <div className="flex flex-col gap-md">
          <div className="flex items-center justify-between">
            <LoadingSkeleton variant="text" width="180px" height="24px" />
            <LoadingSkeleton
              variant="rectangular"
              width="140px"
              height="36px"
            />
          </div>
          <div className="flex flex-col gap-md">
            <LoadingSkeleton variant="rectangular" height="80px" />
            <LoadingSkeleton variant="rectangular" height="80px" />
          </div>
        </div>
      </Card>
    );
  }

  // Loaded state
  return (
    <Card className={className}>
      <div className="flex flex-col gap-md">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-100">
            Connected Devices ({devices.length})
          </h2>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => navigate('/devices')}
            aria-label="Manage devices"
          >
            Manage Devices
          </Button>
        </div>

        {devices.length === 0 ? (
          <div className="py-lg text-center">
            <p className="text-sm text-slate-400">
              No devices connected. Connect a keyboard to get started.
            </p>
          </div>
        ) : (
          <div className="flex flex-col gap-md">
            {devices.map((device) => (
              <div
                key={device.id}
                className="rounded-md border border-slate-700 bg-slate-800 p-md transition-colors hover:border-slate-600"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-sm">
                    <span className="text-xl" role="img" aria-label="Keyboard">
                      🖮
                    </span>
                    <div className="flex flex-col">
                      <div className="flex items-center gap-sm">
                        <span className="text-base font-medium text-slate-100">
                          {device.name}
                        </span>
                        {device.active && (
                          <span
                            className="text-xs text-green-500"
                            aria-label="Active device"
                          >
                            ✓ Active
                          </span>
                        )}
                      </div>
                      <span className="text-xs font-mono text-slate-500">
                        {device.serial || device.path}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="mt-sm flex flex-wrap items-center gap-md text-xs text-slate-400">
                  <span>
                    Scope:{' '}
                    {device.scope === 'global'
                      ? 'Global'
                      : device.scope === 'device-specific'
                        ? 'Device-Specific'
                        : 'Not Set'}
                  </span>
                  <span>•</span>
                  <span>Layout: {device.layout || 'Not Set'}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </Card>
  );
};
