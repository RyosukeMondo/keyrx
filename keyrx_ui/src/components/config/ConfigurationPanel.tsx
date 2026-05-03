import React, { type ReactNode } from 'react';
import { DeviceSelector } from '@/components/DeviceSelector';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { KeyConfigPanel } from '@/components/KeyConfigPanel';
import { CurrentMappingsSummary } from '@/components/CurrentMappingsSummary';
import { KeyPalette, type PaletteKey } from '@/components/KeyPalette';
import type { KeyMapping } from '@/types';
import type { SVGKey } from '@/components/SVGKeyboard';
import {
  ConfigurationContextProvider,
  useDeviceConfig,
  useLayerConfig,
  useKeySelectionConfig,
  useKeyMappingsConfig,
} from './ConfigurationContext';

/**
 * ConfigurationPanel - Configuration controls component
 *
 * Composes configuration UI components into a unified panel.
 * State is provided via context so sub-components can access
 * their respective config slices directly.
 *
 * @component
 * @example
 * ```tsx
 * <ConfigurationPanel
 *   selectedPhysicalKey="VK_A"
 *   onSaveMapping={(mapping) => console.log(mapping)}
 *   onClearMapping={(key) => console.log('Clear', key)}
 *   activeLayer="base"
 *   availableLayers={['base', 'md-00']}
 *   onLayerChange={(layer) => console.log(layer)}
 *   devices={[{ id: '1', name: 'Keyboard' }]}
 *   selectedDevices={['1']}
 *   globalSelected={true}
 *   onDeviceSelectionChange={(devices, global) => console.log(devices, global)}
 *   keyMappings={new Map()}
 *   onEditMapping={(key) => console.log('Edit', key)}
 * />
 * ```
 *
 * Sub-components (DeviceSelector, LayerSwitcher, KeyPalette, KeyConfigPanel,
 * CurrentMappingsSummary) access their config via context hooks directly,
 * breaking the god object that previously passed all props through
 * ConfigurationPanel.
 */

interface ConfigurationPanelProps {
  /** Currently selected physical key on keyboard */
  selectedPhysicalKey: string | null;
  /** Currently selected palette key */
  selectedPaletteKey: PaletteKey | null;
  /** Callback when a palette key is selected */
  onPaletteKeySelect: (key: PaletteKey) => void;
  /** Callback when saving a key mapping */
  onSaveMapping: (mapping: KeyMapping) => void;
  /** Callback when clearing a key mapping */
  onClearMapping: (keyCode: string) => void;
  /** Active layer ID */
  activeLayer: string;
  /** Available layer IDs */
  availableLayers: string[];
  /** Callback when layer changes */
  onLayerChange: (layer: string) => void;
  /** Available devices */
  devices: Device[];
  /** Selected device IDs */
  selectedDevices: string[];
  /** Whether global is selected */
  globalSelected: boolean;
  /** Callback when device selection changes */
  onDeviceSelectionChange: (devices: string[], global: boolean) => void;
  /** Current key mappings for active layer */
  keyMappings: Map<string, KeyMapping>;
  /** Callback when editing a mapping */
  onEditMapping: (keyCode: string) => void;
  /** Layout keys for keyboard visualization */
  layoutKeys?: SVGKey[];
}

/**
 * @deprecated Use sub-context hooks directly (useDeviceConfig, useLayerConfig,
 * useKeySelectionConfig, useKeyMappingsConfig) to access configuration state.
 * This aggregate hook is kept for migration compatibility only.
 */
export function useConfigurationPanel() {
  return {
    device: useDeviceConfig(),
    layer: useLayerConfig(),
    keySelection: useKeySelectionConfig(),
    keyMappings: useKeyMappingsConfig(),
  };
}

/** Section wrapper for layer layout */
function LayerSection({ children }: { children: ReactNode }) {
  return <div className="flex gap-2">{children}</div>;
}

export const ConfigurationPanel: React.FC<ConfigurationPanelProps> = ({
  selectedPhysicalKey,
  selectedPaletteKey,
  onPaletteKeySelect,
  onSaveMapping,
  onClearMapping,
  activeLayer,
  availableLayers,
  onLayerChange,
  devices,
  selectedDevices,
  globalSelected,
  onDeviceSelectionChange,
  keyMappings,
  onEditMapping,
  layoutKeys,
}) => {
  return (
    <ConfigurationContextProvider
      devices={devices}
      selectedDevices={selectedDevices}
      globalSelected={globalSelected}
      onDeviceSelectionChange={onDeviceSelectionChange}
      activeLayer={activeLayer}
      availableLayers={availableLayers}
      onLayerChange={onLayerChange}
      selectedPhysicalKey={selectedPhysicalKey}
      selectedPaletteKey={selectedPaletteKey}
      onPaletteKeySelect={onPaletteKeySelect}
      onEditMapping={onEditMapping}
      keyMappings={keyMappings}
      onSaveMapping={onSaveMapping}
      onClearMapping={onClearMapping}
      layoutKeys={layoutKeys}
    >
      <div className="flex flex-col gap-4">
        {/* Device Selector - Select global or specific devices */}
        <DeviceSelector
          devices={devices}
          selectedDevices={selectedDevices}
          globalSelected={globalSelected}
          onSelectionChange={onDeviceSelectionChange}
          showGlobalOption={true}
          multiSelect={true}
        />

        {/* Layer Switcher - Select active layer */}
        <LayerSection>
          <LayerSwitcher
            activeLayer={activeLayer}
            availableLayers={availableLayers}
            onLayerChange={onLayerChange}
          />
        </LayerSection>

        {/* Key Palette - Select keys for mapping */}
        <KeyPalette
          onKeySelect={onPaletteKeySelect}
          selectedKey={selectedPaletteKey}
          compact={false}
        />

        {/* Key Configuration Panel - Configure selected key */}
        <KeyConfigPanel
          physicalKey={selectedPhysicalKey}
          currentMapping={
            selectedPhysicalKey ? keyMappings.get(selectedPhysicalKey) : undefined
          }
          onSave={onSaveMapping}
          onClearMapping={onClearMapping}
          onEditMapping={onEditMapping}
          activeLayer={activeLayer}
          keyMappings={keyMappings}
          layoutKeys={layoutKeys}
        />

        {/* Current Mappings Summary - Show all mappings */}
        <CurrentMappingsSummary
          keyMappings={keyMappings}
          onEditMapping={onEditMapping}
          onClearMapping={onClearMapping}
        />
      </div>
    </ConfigurationContextProvider>
  );
};
