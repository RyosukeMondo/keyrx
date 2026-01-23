import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from '@/components/Card';
import { type Device } from '@/components/DeviceSelector';
import { KeyConfigPanel } from '@/components/KeyConfigPanel';
import {
  useGetProfileConfig,
  useSetProfileConfig,
} from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { useConfigStore } from '@/stores/configStore';
import type { KeyMapping } from '@/types';

// Import custom hooks
import { useProfileSelection } from '@/hooks/useProfileSelection';
import { useCodePanel } from '@/hooks/useCodePanel';
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';
import { useConfigSync } from '@/hooks/useConfigSync';
import { useASTRebuild } from '@/hooks/useASTRebuild';
import { useDeviceMerging } from '@/hooks/useDeviceMerging';
import { useASTSync } from '@/hooks/useASTSync';

// Import container components
import { CodePanelContainer } from '@/components/config/CodePanelContainer';
import { ProfileSelector } from '@/components/config/ProfileSelector';
import { ConfigurationLayout } from '@/components/config/ConfigurationLayout';
import { SyncStatusIndicator } from '@/components/config/SyncStatusIndicator';
import { DeviceSelectionPanel } from '@/components/config/DeviceSelectionPanel';
import { NotificationBanners } from '@/components/config/NotificationBanners';
import { ConfigScopeTabs } from '@/components/config/ConfigScopeTabs';
import { GlobalKeyboardPanel } from '@/components/config/GlobalKeyboardPanel';
import { DeviceKeyboardPanel } from '@/components/config/DeviceKeyboardPanel';

interface ConfigPageProps {
  profileName?: string;
}

const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName: propProfileName,
}) => {
  const navigate = useNavigate();
  const api = useUnifiedApi();

  // Custom hooks for state management
  const { selectedProfileName, setSelectedProfileName } =
    useProfileSelection(propProfileName);
  const {
    isOpen: isCodePanelOpen,
    height: _codePanelHeight,
    toggleOpen: toggleCodePanel,
  } = useCodePanel();
  const {
    layout: keyboardLayout,
    setLayout: _setLayout,
    layoutKeys,
  } = useKeyboardLayout('ANSI_104');
  const {
    syncEngine,
    syncStatus,
    lastSaveTime,
    setSyncStatus,
    setLastSaveTime,
  } = useConfigSync(selectedProfileName);

  // Fetch available profiles
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { mutateAsync: createProfile } = useCreateProfile();

  // Visual editor state - now using Zustand store for layer-aware mappings
  const configStore = useConfigStore();
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(
    null
  );

  // Computed: Get current layer's mappings for display
  const keyMappings = configStore.getLayerMappings(configStore.activeLayer);
  const activeLayer = configStore.activeLayer;
  const globalSelected = configStore.globalSelected;
  const selectedDevices = configStore.selectedDevices;

  // Available layers
  const availableLayers = [
    'base',
    'md-00',
    'md-01',
    'md-02',
    'md-03',
    'md-04',
    'md-05',
  ];

  // Responsive layout state: 'global' or 'device' for mobile/tablet views
  const [activePane, setActivePane] = useState<'global' | 'device'>('global');

  // Query for profile config - doesn't block rendering
  const {
    data: profileConfig,
    isLoading,
    error,
  } = useGetProfileConfig(selectedProfileName);
  const { mutateAsync: setProfileConfig } = useSetProfileConfig();

  // Merged device list: connected devices + devices from Rhai (even if disconnected)
  const mergedDevices = useDeviceMerging({
    syncEngine,
    configStore,
  });

  // Auto-select first profile if "Default" doesn't exist and profiles are loaded
  useEffect(() => {
    if (
      profiles &&
      profiles.length > 0 &&
      !profiles.some((p) => p.name === selectedProfileName)
    ) {
      setSelectedProfileName(profiles[0].name);
    }
  }, [profiles, selectedProfileName, setSelectedProfileName]);

  // Check if selected profile exists
  const profileExists = profiles?.some((p) => p.name === selectedProfileName) ?? false;

  // Check if config file exists
  const configMissing =
    !isLoading && !error && profileExists && !profileConfig?.source;

  // Track last loaded profile to prevent unnecessary re-initialization
  const lastProfileRef = useRef<string>(selectedProfileName);

  // Update sync engine when profile config loads
  useEffect(() => {
    // Only update if profile changed or config loaded for first time
    const profileChanged = lastProfileRef.current !== selectedProfileName;

    if (profileChanged) {
      lastProfileRef.current = selectedProfileName;

      if (profileConfig?.source) {
        // Initialize sync engine with loaded config
        syncEngine.onCodeChange(profileConfig.source);
        setSyncStatus('saved');
      } else if (configMissing) {
        // Default config template when config file doesn't exist
        const defaultTemplate = `// Configuration for profile: ${selectedProfileName}
// This is a new configuration file

// Example: Simple key remapping
// map("A", "B");  // Press A → outputs B

// Example: Tap/Hold behavior
// tap_hold("CapsLock", "Escape", "LCtrl", 200);

// Add your key mappings here...
`;
        syncEngine.onCodeChange(defaultTemplate);
        setSyncStatus('unsaved');
      }
    }
  }, [profileConfig, configMissing, selectedProfileName, syncEngine, setSyncStatus]);

  // Track code changes to update sync status
  useEffect(() => {
    // Mark as unsaved when code changes (except during save)
    // Note: syncStatus is intentionally NOT in deps to prevent infinite re-render.
    // This effect should only run when syncEngine.state or profileConfig changes,
    // not when syncStatus itself changes (which would create a loop).
    if (syncStatus === 'saved' && syncEngine.state === 'idle') {
      const currentCode = syncEngine.getCode();
      const originalCode = profileConfig?.source || '';
      if (currentCode !== originalCode) {
        setSyncStatus('unsaved');
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [syncEngine.state, syncEngine.getCode, profileConfig?.source, setSyncStatus]);


  // Sync visual editor state from parsed AST - LAYER-AWARE VERSION
  useASTSync({
    syncEngine,
    configStore,
    globalSelected,
    selectedDevices,
  });

  // Handle profile selection change
  const handleProfileChange = (newProfileName: string) => {
    setSelectedProfileName(newProfileName);
    navigate(`/config?profile=${newProfileName}`);
  };

  // Handle profile creation
  const handleCreateProfile = async () => {
    try {
      await createProfile({ name: selectedProfileName, template: 'blank' });
    } catch (err) {
      console.error('Failed to create profile:', err);
    }
  };

  const handleSaveConfig = async () => {
    try {
      setSyncStatus('saving');
      await setProfileConfig({
        name: selectedProfileName,
        source: syncEngine.getCode(),
      });
      setSyncStatus('saved');
      setLastSaveTime(new Date());
    } catch (err) {
      console.error('Failed to save config:', err);
      setSyncStatus('unsaved');
    }
  };

  // Handle key click: select key for inline configuration
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
  };

  // Handle clear mapping from summary - LAYER-AWARE
  const handleClearMapping = (keyCode: string) => {
    configStore.deleteKeyMapping(keyCode, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };

  // Handle save from modal - LAYER-AWARE
  const handleSaveMapping = (mapping: KeyMapping) => {
    if (!selectedPhysicalKey) return;

    // Save to active layer in store
    configStore.setKeyMapping(selectedPhysicalKey, mapping, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };


  // Use merged device list (connected + disconnected from Rhai)
  const devices: Device[] = mergedDevices;

  // Use AST rebuild hook
  const rebuildAndSyncAST = useASTRebuild({
    configStore,
    syncEngine,
    globalSelected,
    selectedDevices,
    devices,
  });

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Streamlined Header */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pb-4 border-b border-slate-700">
        {/* Left: Profile Selector */}
        <ProfileSelector
          value={selectedProfileName}
          onChange={handleProfileChange}
          profiles={profiles}
          isLoading={isLoadingProfiles}
          disabled={!api.isConnected}
        />

        {/* Right: Sync Status and Save Button */}
        <div className="flex items-center gap-3">
          <SyncStatusIndicator
            syncStatus={syncStatus}
            lastSaveTime={lastSaveTime}
            isConnected={api.isConnected}
          />

          {/* Code Panel Toggle and Save Button */}
          <button
            onClick={toggleCodePanel}
            className="px-4 py-2 bg-slate-700 text-slate-200 text-sm font-medium rounded-md hover:bg-slate-600 transition-colors whitespace-nowrap border border-slate-600"
            title={isCodePanelOpen ? 'Hide Code' : 'Show Code'}
          >
            {isCodePanelOpen ? '▼ Hide Code' : '▲ Show Code'}
          </button>

          <button
            onClick={handleSaveConfig}
            disabled={
              !api.isConnected || !profileExists || syncStatus === 'saving'
            }
            className="px-4 py-2 bg-primary-500 text-white text-sm font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
          >
            {configMissing ? 'Create' : 'Save'}
          </button>
        </div>
      </div>

      {/* Error/Info Messages */}
      <NotificationBanners
        profileName={selectedProfileName}
        profileExists={profileExists}
        configMissing={configMissing}
        error={error}
        isLoading={isLoading}
        isConnected={api.isConnected}
        onCreateProfile={handleCreateProfile}
      />

      {/* Visual Editor Content (Always visible) */}
      <ConfigurationLayout profileName={selectedProfileName}>
        {/* Device Selection Panel */}
        <DeviceSelectionPanel
          devices={devices}
          globalSelected={globalSelected}
          selectedDevices={selectedDevices}
          onToggleGlobal={(selected) => configStore.setGlobalSelected(selected)}
          onToggleDevice={(deviceId, selected) => {
            if (selected) {
              configStore.setSelectedDevices([...selectedDevices, deviceId]);
            } else {
              configStore.setSelectedDevices(
                selectedDevices.filter((id) => id !== deviceId)
              );
            }
          }}
        />

        {/* Tab Navigation - Accessible tabs for Global/Device switching */}
        {globalSelected && selectedDevices.length > 0 && (
          <ConfigScopeTabs
            activePane={activePane}
            onPaneChange={setActivePane}
          />
        )}

        {/* Single-Pane Layout: Show one pane at a time (tabs control visibility) */}
        <div className="flex flex-col gap-4">
          {/* Global Keyboard Panel */}
          <GlobalKeyboardPanel
            profileName={selectedProfileName}
            activeLayer={activeLayer}
            availableLayers={availableLayers}
            onLayerChange={configStore.setActiveLayer}
            globalSelected={globalSelected}
            onToggleGlobal={configStore.setGlobalSelected}
            keyMappings={keyMappings}
            onKeyClick={handlePhysicalKeyClick}
            selectedKeyCode={selectedPhysicalKey}
            initialLayout={keyboardLayout}
            isVisible={
              selectedDevices.length === 0 || activePane === 'global'
            }
          />

          {/* Device-Specific Keyboard Panel */}
          <DeviceKeyboardPanel
            profileName={selectedProfileName}
            activeLayer={activeLayer}
            availableLayers={availableLayers}
            onLayerChange={configStore.setActiveLayer}
            devices={devices}
            selectedDevices={selectedDevices}
            onDeviceChange={(oldDeviceId, newDeviceId) => {
              const updatedDevices = selectedDevices.filter(
                (id) => id !== oldDeviceId
              );
              configStore.setSelectedDevices([...updatedDevices, newDeviceId]);
            }}
            keyMappings={keyMappings}
            onKeyClick={handlePhysicalKeyClick}
            selectedKeyCode={selectedPhysicalKey}
            initialLayout={keyboardLayout}
            isVisible={!globalSelected || activePane === 'device'}
          />

          {/* Warning if no selection */}
          {!globalSelected && selectedDevices.length === 0 && (
            <Card
              className="bg-yellow-900/20 border border-yellow-700/50 flex-1 block"
              aria-label="Configuration Warning"
            >
              <div className="text-center py-8">
                <p className="text-yellow-200 text-lg mb-2">
                  ⚠️ No devices selected
                </p>
                <p className="text-yellow-300 text-sm">
                  Select at least one device or enable "Global Keys" to
                  configure key mappings
                </p>
              </div>
            </Card>
          )}
        </div>

        {/* Legend - Color coding */}
        <div className="flex gap-4 flex-wrap text-xs text-slate-400 px-2">
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-green-500"></div>
            <span>Simple</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-primary-500"></div>
            <span>Modifier</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-purple-500"></div>
            <span>Lock</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-red-500"></div>
            <span>Tap/Hold</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-yellow-500"></div>
            <span>Layer Active</span>
          </div>
        </div>

        {/* Current Mappings Summary - Shows active mappings with edit/delete */}
        {/* Inline Key Configuration Panel */}
        <KeyConfigPanel
          physicalKey={selectedPhysicalKey}
          currentMapping={
            selectedPhysicalKey
              ? keyMappings.get(selectedPhysicalKey)
              : undefined
          }
          onSave={handleSaveMapping}
          onClearMapping={handleClearMapping}
          onEditMapping={handlePhysicalKeyClick}
          activeLayer={activeLayer}
          keyMappings={keyMappings}
          layoutKeys={layoutKeys}
        />
      </ConfigurationLayout>

      {/* Collapsible Code Panel */}
      <CodePanelContainer
        profileName={selectedProfileName}
        rhaiCode={syncEngine.getCode()}
        onChange={(value) => syncEngine.onCodeChange(value)}
        syncEngine={syncEngine}
        isOpen={isCodePanelOpen}
        onToggle={toggleCodePanel}
      />
    </div>
  );
};

export default ConfigPage;
