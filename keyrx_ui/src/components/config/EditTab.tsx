import React, { useState } from 'react';
import { Card } from '@/components/Card';
import { type Device } from '@/components/DeviceSelector';
import { KeyConfigPanel } from '@/components/KeyConfigPanel';
import type { KeyMapping } from '@/types';
import type { LayoutType } from '@/components/KeyboardVisualizer';
import type { RhaiSyncEngineResult } from '@/components/RhaiSyncEngine';
import type { SyncStatus } from '@/hooks/useConfigSync';
import type { SVGKeyData } from '@/utils/kle-parser';
import { useDeviceMerging } from '@/hooks/useDeviceMerging';
import { useASTRebuild } from '@/hooks/useASTRebuild';
import { NotificationBanners } from '@/components/config/NotificationBanners';
import { ConfigurationLayout } from '@/components/config/ConfigurationLayout';
import { DeviceSelectionPanel } from '@/components/config/DeviceSelectionPanel';
import { ConfigScopeTabs } from '@/components/config/ConfigScopeTabs';
import { GlobalKeyboardPanel } from '@/components/config/GlobalKeyboardPanel';
import { DeviceKeyboardPanel } from '@/components/config/DeviceKeyboardPanel';

const AVAILABLE_LAYERS = [
  'base',
  'md-00',
  'md-01',
  'md-02',
  'md-03',
  'md-04',
  'md-05',
];

interface EditTabProps {
  selectedProfileName: string;
  profileConfig: { source: string } | undefined;
  isLoading: boolean;
  error: Error | null;
  profileExists: boolean;
  configMissing: boolean;
  isConnected: boolean;
  syncEngine: RhaiSyncEngineResult;
  syncStatus: SyncStatus;
  setSyncStatus: (status: SyncStatus) => void;
  configStore: {
    activeLayer: string;
    globalSelected: boolean;
    selectedDevices: string[];
    getLayerMappings: (layerId: string) => Map<string, KeyMapping>;
    getAllLayers: () => string[];
    setActiveLayer: (layerId: string) => void;
    setGlobalSelected: (selected: boolean) => void;
    setSelectedDevices: (deviceIds: string[]) => void;
    setKeyMapping: (
      key: string,
      mapping: KeyMapping,
      layerId?: string
    ) => void;
    deleteKeyMapping: (key: string, layerId?: string) => void;
  };
  keyboardLayout: LayoutType;
  layoutKeys: SVGKeyData[];
  onCreateProfile: () => void;
}

/**
 * Visual editor tab for the ConfigPage.
 *
 * Renders the keyboard visualizer, device selection, layer switcher,
 * key configuration panel, and notification banners. Owns the
 * selectedPhysicalKey and activePane local state, and internally
 * calls useDeviceMerging / useASTRebuild hooks.
 */
export const EditTab: React.FC<EditTabProps> = ({
  selectedProfileName,
  profileConfig: _profileConfig,
  isLoading,
  error,
  profileExists,
  configMissing,
  isConnected,
  syncEngine,
  syncStatus: _syncStatus,
  setSyncStatus,
  configStore,
  keyboardLayout,
  layoutKeys,
  onCreateProfile,
}) => {
  // Local state owned by EditTab
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(
    null
  );
  const [activePane, setActivePane] = useState<'global' | 'device'>('global');

  // Derived values from configStore
  const keyMappings = configStore.getLayerMappings(configStore.activeLayer);
  const { activeLayer, globalSelected, selectedDevices } = configStore;

  // Merged device list: connected devices + devices from Rhai config
  const mergedDevices = useDeviceMerging({ syncEngine, configStore });
  const devices: Device[] = mergedDevices;

  // AST rebuild hook for syncing visual editor changes back to code
  const rebuildAndSyncAST = useASTRebuild({
    configStore,
    syncEngine,
    globalSelected,
    selectedDevices,
    devices,
  });

  // Handlers
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
  };

  const handleClearMapping = (keyCode: string) => {
    configStore.deleteKeyMapping(keyCode, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };

  const handleSaveMapping = (mapping: KeyMapping) => {
    if (!selectedPhysicalKey) return;
    configStore.setKeyMapping(selectedPhysicalKey, mapping, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };

  return (
    <div className="flex flex-col gap-4 md:gap-6">
      {/* Error/Info Messages */}
      <NotificationBanners
        profileName={selectedProfileName}
        profileExists={profileExists}
        configMissing={configMissing}
        error={error}
        isLoading={isLoading}
        isConnected={isConnected}
        onCreateProfile={onCreateProfile}
      />

      {/* Visual Editor Content */}
      <ConfigurationLayout profileName={selectedProfileName}>
        {/* Device Selection Panel */}
        <DeviceSelectionPanel
          devices={devices}
          globalSelected={globalSelected}
          selectedDevices={selectedDevices}
          onToggleGlobal={(selected) =>
            configStore.setGlobalSelected(selected)
          }
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

        {/* Tab Navigation for Global/Device switching */}
        {globalSelected && selectedDevices.length > 0 && (
          <ConfigScopeTabs
            activePane={activePane}
            onPaneChange={setActivePane}
          />
        )}

        {/* Single-Pane Layout: tabs control visibility */}
        <div className="flex flex-col gap-4">
          {/* Global Keyboard Panel */}
          <GlobalKeyboardPanel
            profileName={selectedProfileName}
            activeLayer={activeLayer}
            availableLayers={AVAILABLE_LAYERS}
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
            availableLayers={AVAILABLE_LAYERS}
            onLayerChange={configStore.setActiveLayer}
            devices={devices}
            selectedDevices={selectedDevices}
            onDeviceChange={(oldDeviceId, newDeviceId) => {
              const updatedDevices = selectedDevices.filter(
                (id) => id !== oldDeviceId
              );
              configStore.setSelectedDevices([
                ...updatedDevices,
                newDeviceId,
              ]);
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
                  No devices selected
                </p>
                <p className="text-yellow-300 text-sm">
                  Select at least one device or enable &quot;Global
                  Keys&quot; to configure key mappings
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
    </div>
  );
};
