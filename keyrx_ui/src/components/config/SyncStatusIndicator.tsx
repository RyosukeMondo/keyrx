import React from 'react';

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
    if (msAgo < 60000) return 'just now';
    return `${Math.floor(msAgo / 60000)}m ago`;
  };

  return (
    <div className="flex items-center gap-2">
      {syncStatus === 'saved' && (
        <div
          className="flex items-center gap-2 text-xs text-green-400"
          title="All changes saved"
        >
          <span className="w-2 h-2 rounded-full bg-green-400"></span>
          <span className="hidden sm:inline">Saved</span>
          {lastSaveTime && (
            <span className="text-slate-500 hidden md:inline">
              {getTimeAgo()}
            </span>
          )}
        </div>
      )}
      {syncStatus === 'unsaved' && (
        <div
          className="flex items-center gap-2 text-xs text-yellow-400"
          title="Unsaved changes"
        >
          <span className="w-2 h-2 rounded-full bg-yellow-400"></span>
          <span className="hidden sm:inline">Unsaved</span>
        </div>
      )}
      {syncStatus === 'saving' && (
        <div
          className="flex items-center gap-2 text-xs text-blue-400"
          title="Saving..."
        >
          <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse"></span>
          <span className="hidden sm:inline">Saving...</span>
        </div>
      )}
      {!isConnected && (
        <div
          className="flex items-center gap-2 text-xs text-red-400"
          title="Disconnected from daemon"
        >
          <span className="w-2 h-2 rounded-full bg-red-400"></span>
          <span className="hidden sm:inline">Disconnected</span>
        </div>
      )}
    </div>
  );
};
