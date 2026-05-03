import React, { createContext, useContext, type ReactNode } from 'react';
import type { KeyMapping } from '@/types';
import type { Device } from '@/components/DeviceSelector';
import type { PaletteKey } from '@/components/KeyPalette';
import type { SVGKey } from '@/components/SVGKeyboard';

/**
 * Configuration Context - Provides shared configuration state to sub-components
 *
 * Breaks the god object pattern by allowing sub-components to consume
 * configuration state directly via sub-context hooks instead of receiving
 * all props passed through the aggregate ConfigurationPanel.
 *
 * Sub-contexts:
 * - useDeviceConfig: device selection state
 * - useLayerConfig: layer switching state
 * - useKeySelectionConfig: key palette and physical key selection
 * - useKeyMappingsConfig: key mappings state
 */

// Device Config
interface DeviceConfigContextValue {
  devices: Device[];
  selectedDevices: string[];
  globalSelected: boolean;
  onDeviceSelectionChange: (devices: string[], global: boolean) => void;
}

const DeviceConfigContext = createContext<DeviceConfigContextValue | null>(null);

export function useDeviceConfig(): DeviceConfigContextValue {
  const ctx = useContext(DeviceConfigContext);
  if (!ctx) {
    throw new Error('useDeviceConfig must be used within DeviceConfigProvider');
  }
  return ctx;
}

interface DeviceConfigProviderProps {
  devices: Device[];
  selectedDevices: string[];
  globalSelected: boolean;
  onDeviceSelectionChange: (devices: string[], global: boolean) => void;
  children: ReactNode;
}

export function DeviceConfigProvider({
  devices,
  selectedDevices,
  globalSelected,
  onDeviceSelectionChange,
  children,
}: DeviceConfigProviderProps) {
  return (
    <DeviceConfigContext.Provider
      value={{ devices, selectedDevices, globalSelected, onDeviceSelectionChange }}
    >
      {children}
    </DeviceConfigContext.Provider>
  );
}

// Layer Config
interface LayerConfigContextValue {
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
}

const LayerConfigContext = createContext<LayerConfigContextValue | null>(null);

export function useLayerConfig(): LayerConfigContextValue {
  const ctx = useContext(LayerConfigContext);
  if (!ctx) {
    throw new Error('useLayerConfig must be used within LayerConfigProvider');
  }
  return ctx;
}

interface LayerConfigProviderProps {
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
  children: ReactNode;
}

export function LayerConfigProvider({
  activeLayer,
  availableLayers,
  onLayerChange,
  children,
}: LayerConfigProviderProps) {
  return (
    <LayerConfigContext.Provider value={{ activeLayer, availableLayers, onLayerChange }}>
      {children}
    </LayerConfigContext.Provider>
  );
}

// Key Selection Config
interface KeySelectionConfigContextValue {
  selectedPhysicalKey: string | null;
  selectedPaletteKey: PaletteKey | null;
  onPaletteKeySelect: (key: PaletteKey) => void;
  onEditMapping: (keyCode: string) => void;
}

const KeySelectionConfigContext = createContext<KeySelectionConfigContextValue | null>(null);

export function useKeySelectionConfig(): KeySelectionConfigContextValue {
  const ctx = useContext(KeySelectionConfigContext);
  if (!ctx) {
    throw new Error('useKeySelectionConfig must be used within KeySelectionConfigProvider');
  }
  return ctx;
}

interface KeySelectionConfigProviderProps {
  selectedPhysicalKey: string | null;
  selectedPaletteKey: PaletteKey | null;
  onPaletteKeySelect: (key: PaletteKey) => void;
  onEditMapping: (keyCode: string) => void;
  children: ReactNode;
}

export function KeySelectionConfigProvider({
  selectedPhysicalKey,
  selectedPaletteKey,
  onPaletteKeySelect,
  onEditMapping,
  children,
}: KeySelectionConfigProviderProps) {
  return (
    <KeySelectionConfigContext.Provider
      value={{ selectedPhysicalKey, selectedPaletteKey, onPaletteKeySelect, onEditMapping }}
    >
      {children}
    </KeySelectionConfigContext.Provider>
  );
}

// Key Mappings Config
interface KeyMappingsConfigContextValue {
  keyMappings: Map<string, KeyMapping>;
  onSaveMapping: (mapping: KeyMapping) => void;
  onClearMapping: (keyCode: string) => void;
  layoutKeys?: SVGKey[];
}

const KeyMappingsConfigContext = createContext<KeyMappingsConfigContextValue | null>(null);

export function useKeyMappingsConfig(): KeyMappingsConfigContextValue {
  const ctx = useContext(KeyMappingsConfigContext);
  if (!ctx) {
    throw new Error('useKeyMappingsConfig must be used within KeyMappingsConfigProvider');
  }
  return ctx;
}

interface KeyMappingsConfigProviderProps {
  keyMappings: Map<string, KeyMapping>;
  onSaveMapping: (mapping: KeyMapping) => void;
  onClearMapping: (keyCode: string) => void;
  layoutKeys?: SVGKey[];
  children: ReactNode;
}

export function KeyMappingsConfigProvider({
  keyMappings,
  onSaveMapping,
  onClearMapping,
  layoutKeys,
  children,
}: KeyMappingsConfigProviderProps) {
  return (
    <KeyMappingsConfigContext.Provider
      value={{ keyMappings, onSaveMapping, onClearMapping, layoutKeys }}
    >
      {children}
    </KeyMappingsConfigContext.Provider>
  );
}

/**
 * Combined configuration context provider
 * Composes all sub-contexts for use by ConfigurationPanel
 */
interface ConfigurationContextProviderProps {
  // Device
  devices: Device[];
  selectedDevices: string[];
  globalSelected: boolean;
  onDeviceSelectionChange: (devices: string[], global: boolean) => void;
  // Layer
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
  // Key Selection
  selectedPhysicalKey: string | null;
  selectedPaletteKey: PaletteKey | null;
  onPaletteKeySelect: (key: PaletteKey) => void;
  onEditMapping: (keyCode: string) => void;
  // Key Mappings
  keyMappings: Map<string, KeyMapping>;
  onSaveMapping: (mapping: KeyMapping) => void;
  onClearMapping: (keyCode: string) => void;
  layoutKeys?: SVGKey[];
  children: ReactNode;
}

export function ConfigurationContextProvider({
  devices,
  selectedDevices,
  globalSelected,
  onDeviceSelectionChange,
  activeLayer,
  availableLayers,
  onLayerChange,
  selectedPhysicalKey,
  selectedPaletteKey,
  onPaletteKeySelect,
  onEditMapping,
  keyMappings,
  onSaveMapping,
  onClearMapping,
  layoutKeys,
  children,
}: ConfigurationContextProviderProps) {
  return (
    <DeviceConfigProvider
      devices={devices}
      selectedDevices={selectedDevices}
      globalSelected={globalSelected}
      onDeviceSelectionChange={onDeviceSelectionChange}
    >
      <LayerConfigProvider
        activeLayer={activeLayer}
        availableLayers={availableLayers}
        onLayerChange={onLayerChange}
      >
        <KeySelectionConfigProvider
          selectedPhysicalKey={selectedPhysicalKey}
          selectedPaletteKey={selectedPaletteKey}
          onPaletteKeySelect={onPaletteKeySelect}
          onEditMapping={onEditMapping}
        >
          <KeyMappingsConfigProvider
            keyMappings={keyMappings}
            onSaveMapping={onSaveMapping}
            onClearMapping={onClearMapping}
            layoutKeys={layoutKeys}
          >
            {children}
          </KeyMappingsConfigProvider>
        </KeySelectionConfigProvider>
      </LayerConfigProvider>
    </DeviceConfigProvider>
  );
}
