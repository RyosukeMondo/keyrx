import { useState, useEffect } from 'react';
import {
  useRhaiSyncEngine,
  type RhaiSyncEngineResult,
} from '@/components/RhaiSyncEngine';

export type SyncStatus = 'saved' | 'unsaved' | 'saving';

export interface UseConfigSyncReturn {
  syncEngine: RhaiSyncEngineResult;
  syncStatus: SyncStatus;
  lastSaveTime: Date | null;
  setSyncStatus: (status: SyncStatus) => void;
  setLastSaveTime: (time: Date | null) => void;
}

/**
 * Custom hook for managing config synchronization with RhaiSyncEngine.
 *
 * Encapsulates RhaiSyncEngine initialization and sync status management,
 * providing a unified interface for tracking configuration save state.
 *
 * @param profileName - Name of the profile to sync
 * @returns Object containing syncEngine, syncStatus state, and setters
 *
 * @example
 * ```tsx
 * const { syncEngine, syncStatus, lastSaveTime, setSyncStatus } = useConfigSync('MyProfile');
 *
 * // Use syncEngine for code/visual sync
 * <MonacoEditor value={syncEngine.getCode()} onChange={syncEngine.onCodeChange} />
 *
 * // Track save status
 * {syncStatus === 'saving' && <Spinner />}
 * {syncStatus === 'saved' && <CheckIcon />}
 * ```
 */
export function useConfigSync(profileName: string): UseConfigSyncReturn {
  // Sync status tracking
  const [syncStatus, setSyncStatus] = useState<SyncStatus>('saved');
  const [lastSaveTime, setLastSaveTime] = useState<Date | null>(null);

  // Initialize RhaiSyncEngine for bidirectional sync
  const syncEngine = useRhaiSyncEngine({
    storageKey: `profile-${profileName}`,
    debounceMs: 500,
    onStateChange: () => {
      // Sync state changed
    },
    onError: (error, direction) => {
      // Log error to console in development
      if (process.env.NODE_ENV === 'development') {
        // eslint-disable-next-line no-console
        console.error('Sync error:', { error, direction });
      }
    },
  });

  // Reset sync status when profile changes
  // This is intentional - we want to reset state synchronously when the profile changes
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setSyncStatus('saved');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLastSaveTime(null);
  }, [profileName]);

  return {
    syncEngine,
    syncStatus,
    lastSaveTime,
    setSyncStatus,
    setLastSaveTime,
  };
}
