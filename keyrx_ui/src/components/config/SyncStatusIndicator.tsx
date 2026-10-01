import React from 'react';
import { t } from '@/i18n';

export type SyncStatus = 'saved' | 'unsaved' | 'saving';

interface SyncStatusIndicatorProps {
  syncStatus: SyncStatus;
  lastSaveTime: Date | null;
  isConnected: boolean;
}

/**
 * Displays the current sync status with visual indicators
 */
export const SyncStatusIndicator: React.FC<SyncStatusIndicatorProps> = ({
  syncStatus,
  lastSaveTime,
  isConnected,
}) => {
  const getTimeAgo = () => {
    if (!lastSaveTime) return '';
    const msAgo = new Date().getTime() - lastSaveTime.getTime();
    if (msAgo < 60000) return t('sync.justNow');
    return t('sync.minutesAgo', { count: Math.floor(msAgo / 60000) });
  };

  return (
    <div className="flex items-center gap-2">
      {syncStatus === 'saved' && (
        <div
          className="flex items-center gap-2 text-xs text-green-400"
          title={t('sync.savedTitle')}
        >
          <span className="w-2 h-2 rounded-full bg-green-400"></span>
          <span>{t('sync.saved')}</span>
          {lastSaveTime && (
            <span className="text-slate-400 hidden md:inline">
              {getTimeAgo()}
            </span>
          )}
        </div>
      )}
      {syncStatus === 'unsaved' && (
        <div
          className="flex items-center gap-2 text-xs text-yellow-400"
          title={t('sync.unsavedTitle')}
        >
          <span className="w-2 h-2 rounded-full bg-yellow-400"></span>
          <span>{t('sync.unsaved')}</span>
        </div>
      )}
      {syncStatus === 'saving' && (
        <div
          className="flex items-center gap-2 text-xs text-blue-400"
          title={t('sync.saving')}
        >
          <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse"></span>
          <span>{t('sync.saving')}</span>
        </div>
      )}
      {!isConnected && (
        <div
          className="flex items-center gap-2 text-xs text-red-400"
          title={t('sync.disconnectedTitle')}
        >
          <span className="w-2 h-2 rounded-full bg-red-400"></span>
          <span>{t('sync.disconnected')}</span>
        </div>
      )}
    </div>
  );
};
