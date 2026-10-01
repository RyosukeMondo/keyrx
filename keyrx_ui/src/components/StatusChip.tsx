import React from 'react';
import { useDaemonStatus } from '@/hooks/useDaemonStatus';
import { t } from '@/i18n';

/**
 * App-wide daemon status: which profile is live and how many keyboards the
 * daemon has grabbed. Shown on every page so "is it working?" is answerable at
 * a glance, not from a 6px dot.
 */
export const StatusChip: React.FC = () => {
  const { data, isError, isLoading } = useDaemonStatus();

  if (isLoading) return null;

  const online = !isError && !!data && data.daemon_running !== false;
  const text =
    online && data
      ? `${
          data.active_profile
            ? t('status.active', { profile: data.active_profile })
            : t('status.noProfile')
        } · ${t('status.keyboards', { count: data.device_count })}`
      : t('status.offline');

  return (
    <div
      role="status"
      aria-label={t('status.label')}
      data-testid="status-chip"
      className={`inline-flex max-w-full items-center gap-2 rounded-full px-3 py-1 text-xs font-medium ring-1 ${
        online
          ? 'bg-emerald-400/10 text-emerald-300 ring-emerald-400/30'
          : 'bg-rose-400/10 text-rose-300 ring-rose-400/30'
      }`}
    >
      <span
        aria-hidden="true"
        className={`h-2 w-2 shrink-0 rounded-full ${online ? 'bg-emerald-400' : 'bg-rose-400'}`}
      />
      <span className="truncate">{text}</span>
    </div>
  );
};
