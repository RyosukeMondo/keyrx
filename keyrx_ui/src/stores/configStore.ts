import { create } from 'zustand';
import type { KeyMapping } from '../types';

/**
 * Layer-aware Configuration Store
 *
 * Manages key mappings per layer, enabling proper layer-specific configuration.
 *
 * Structure:
 * - layerMappings: Map<layerId, Map<keyCode, KeyMapping>>
 *   - Outer map: layer identifier ('base', 'md-00', 'md-01', etc.)
 *   - Inner map: key code -> mapping configuration
 *
 * Example:
 * {
 *   'base': Map({ 'VK_A' -> { type: 'simple', tapAction: 'Space' } }),
 *   'md-00': Map({ 'VK_A' -> { type: 'simple', tapAction: 'Enter' } }),
 * }
 */

interface ConfigStore {
  // State
  currentProfile: string | null;
  activeLayer: string;
  layerMappings: Map<string, Map<string, KeyMapping>>; // Layer-aware structure
  globalSelected: boolean;
  selectedDevices: string[];
  loading: boolean;
  error: string | null;

  // Actions
  setActiveLayer: (layerId: string) => void;
  setKeyMapping: (key: string, mapping: KeyMapping, layerId?: string) => void;
  deleteKeyMapping: (key: string, layerId?: string) => void;
  getLayerMappings: (layerId: string) => Map<string, KeyMapping>;
  getAllLayers: () => string[];
  clearLayer: (layerId: string) => void;
  setGlobalSelected: (selected: boolean) => void;
  setSelectedDevices: (deviceIds: string[]) => void;
  loadLayerMappings: (mappings: Map<string, Map<string, KeyMapping>>) => void;
  clearError: () => void;
  reset: () => void;
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  // Initial state
  currentProfile: null,
  activeLayer: 'base',
  layerMappings: new Map([['base', new Map()]]), // Initialize with base layer
  globalSelected: true,
  selectedDevices: [],
  loading: false,
  error: null,

  // Set active layer
  setActiveLayer: (layerId: string) => {
    const { layerMappings } = get();

    // Ensure layer exists
    if (!layerMappings.has(layerId)) {
      const newLayerMappings = new Map(layerMappings);
      newLayerMappings.set(layerId, new Map());
      set({ layerMappings: newLayerMappings, activeLayer: layerId });
    } else {
      set({ activeLayer: layerId });
    }
  },

  // Set or update a key mapping in a specific layer
  setKeyMapping: (key: string, mapping: KeyMapping, layerId?: string) => {
    const { activeLayer, layerMappings } = get();
    const targetLayer = layerId || activeLayer;

    // Get or create layer
    const newLayerMappings = new Map(layerMappings);
    if (!newLayerMappings.has(targetLayer)) {
      newLayerMappings.set(targetLayer, new Map());
    }

    // Update mapping in the layer
    const layerMap = new Map(newLayerMappings.get(targetLayer)!);
    layerMap.set(key, mapping);
    newLayerMappings.set(targetLayer, layerMap);

    set({ layerMappings: newLayerMappings, error: null });
  },

  // Delete a key mapping from a specific layer
  deleteKeyMapping: (key: string, layerId?: string) => {
    const { activeLayer, layerMappings } = get();
    const targetLayer = layerId || activeLayer;

    if (!layerMappings.has(targetLayer)) return;

    const newLayerMappings = new Map(layerMappings);
    const layerMap = new Map(newLayerMappings.get(targetLayer)!);
    layerMap.delete(key);
    newLayerMappings.set(targetLayer, layerMap);

    set({ layerMappings: newLayerMappings, error: null });
  },

  // Get mappings for a specific layer
  getLayerMappings: (layerId: string) => {
    const { layerMappings } = get();
    return layerMappings.get(layerId) || new Map();
  },

  // Get all layer IDs
  getAllLayers: () => {
    const { layerMappings } = get();
    return Array.from(layerMappings.keys());
  },

  // Clear all mappings in a layer
  clearLayer: (layerId: string) => {
    const { layerMappings } = get();
    const newLayerMappings = new Map(layerMappings);
    newLayerMappings.set(layerId, new Map());
    set({ layerMappings: newLayerMappings });
  },

  // Set global scope selection
  setGlobalSelected: (selected: boolean) => {
    set({ globalSelected: selected });
  },

  // Set selected devices
  setSelectedDevices: (deviceIds: string[]) => {
    set({ selectedDevices: deviceIds });
  },

  // Load layer mappings from external source (e.g., parsed Rhai AST)
  loadLayerMappings: (mappings: Map<string, Map<string, KeyMapping>>) => {
    set({ layerMappings: new Map(mappings) });
  },

  // Clear error state
  clearError: () => set({ error: null }),

  // Reset store to initial state
  reset: () =>
    set({
      currentProfile: null,
      activeLayer: 'base',
      layerMappings: new Map([['base', new Map()]]),
      globalSelected: true,
      selectedDevices: [],
      loading: false,
      error: null,
    }),
}));
