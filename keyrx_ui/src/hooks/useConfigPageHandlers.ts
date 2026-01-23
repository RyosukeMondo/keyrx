import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import type { KeyMapping } from '@/types';

interface UseConfigPageHandlersProps {
  selectedProfileName: string;
  setSelectedProfileName: (name: string) => void;
  createProfile: (data: { name: string }) => Promise<void>;
  setProfileConfig: (data: {
    name: string;
    source: string;
  }) => Promise<void>;
  configStore: {
    setMapping: (keyCode: string, mapping: KeyMapping) => void;
    deleteMapping: (keyCode: string) => void;
    activeLayer: string;
    generateRhaiSource: () => string;
  };
  syncEngine: {
    syncToRhai: () => void;
  };
  setSelectedPhysicalKey: (key: string | null) => void;
  setSyncStatus: (status: 'saved' | 'unsaved' | 'saving') => void;
  setLastSaveTime: (time: Date) => void;
}

export function useConfigPageHandlers({
  selectedProfileName,
  setSelectedProfileName,
  createProfile,
  setProfileConfig,
  configStore,
  syncEngine,
  setSelectedPhysicalKey,
  setSyncStatus,
  setLastSaveTime,
}: UseConfigPageHandlersProps) {
  const navigate = useNavigate();

  const handleProfileChange = useCallback(
    (newProfileName: string) => {
      setSelectedProfileName(newProfileName);
      navigate(`/config/${encodeURIComponent(newProfileName)}`);
    },
    [setSelectedProfileName, navigate]
  );

  const handleCreateProfile = useCallback(async () => {
    try {
      await createProfile({ name: selectedProfileName });
    } catch (err) {
      console.error('Failed to create profile:', err);
    }
  }, [createProfile, selectedProfileName]);

  const handleSaveConfig = useCallback(async () => {
    try {
      setSyncStatus('saving');
      const rhaiSource = configStore.generateRhaiSource();
      await setProfileConfig({
        name: selectedProfileName,
        source: rhaiSource,
      });
      setSyncStatus('saved');
      setLastSaveTime(new Date());
    } catch (err) {
      console.error('Failed to save config:', err);
      setSyncStatus('unsaved');
    }
  }, [
    selectedProfileName,
    configStore,
    setProfileConfig,
    setSyncStatus,
    setLastSaveTime,
  ]);

  const handlePhysicalKeyClick = useCallback(
    (keyCode: string) => {
      setSelectedPhysicalKey(keyCode);
    },
    [setSelectedPhysicalKey]
  );

  const handleClearMapping = useCallback(
    (keyCode: string) => {
      configStore.deleteMapping(keyCode);
      syncEngine.syncToRhai();
      setSyncStatus('unsaved');
    },
    [configStore, syncEngine, setSyncStatus]
  );

  const handleSaveMapping = useCallback(
    (keyCode: string, mapping: KeyMapping) => {
      configStore.setMapping(keyCode, mapping);
      syncEngine.syncToRhai();
      setSyncStatus('unsaved');
    },
    [configStore, syncEngine, setSyncStatus]
  );

  const rebuildAndSyncAST = useCallback(() => {
    syncEngine.syncToRhai();
  }, [syncEngine]);

  return {
    handleProfileChange,
    handleCreateProfile,
    handleSaveConfig,
    handlePhysicalKeyClick,
    handleClearMapping,
    handleSaveMapping,
    rebuildAndSyncAST,
  };
}
