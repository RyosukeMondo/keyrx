/**
 * Zustand store for the Visual Configuration Builder
 *
 * Manages the state of layers, mappings, modifiers, and locks.
 */

import { create } from 'zustand';
import {
  ConfigState,
  Layer,
  Mapping,
  Modifier,
  Lock,
  MappingType,
} from '../types/configBuilder';

/**
 * Actions available on the config builder store
 */
interface ConfigBuilderActions {
  // Layer management
  addLayer: (name?: string) => void;
  removeLayer: (layerId: string) => void;
  renameLayer: (layerId: string, newName: string) => void;
  setCurrentLayer: (layerId: string) => void;
  reorderLayers: (startIndex: number, endIndex: number) => void;

  // Mapping management
  addMapping: (layerId: string, sourceKey: string, targetKey: string, type?: MappingType) => void;
  removeMapping: (layerId: string, mappingId: string) => void;
  updateMapping: (layerId: string, mappingId: string, updates: Partial<Mapping>) => void;
  clearMappings: (layerId: string) => void;

  // Modifier management
  addModifier: (name: string, triggerKey: string) => void;
  removeModifier: (modifierId: string) => void;
  updateModifier: (modifierId: string, updates: Partial<Modifier>) => void;

  // Lock management
  addLock: (name: string, triggerKey: string) => void;
  removeLock: (lockId: string) => void;
  updateLock: (lockId: string, updates: Partial<Lock>) => void;

  // State management
  setConfig: (config: ConfigState) => void;
  resetConfig: () => void;
  markDirty: () => void;
  markClean: () => void;
}

/**
 * Complete store type
 */
type ConfigBuilderStore = ConfigState & ConfigBuilderActions;

/**
 * Generate a unique ID
 */
const generateId = (): string => {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
};

/**
 * Create the initial state with a base layer
 */
const createInitialState = (): ConfigState => ({
  layers: [
    {
      id: 'base',
      name: 'base',
      mappings: [],
      isBase: true,
    },
  ],
  modifiers: [],
  locks: [],
  currentLayerId: 'base',
  isDirty: false,
});

/**
 * Config builder store using Zustand
 */
export const useConfigBuilderStore = create<ConfigBuilderStore>((set) => ({
  ...createInitialState(),

  // Layer management
  addLayer: (name) =>
    set((state) => {
      const layerId = generateId();
      const layerName = name || `layer_${state.layers.length}`;
      const newLayer: Layer = {
        id: layerId,
        name: layerName,
        mappings: [],
        isBase: false,
      };
      return {
        layers: [...state.layers, newLayer],
        currentLayerId: layerId,
        isDirty: true,
      };
    }),

  removeLayer: (layerId) =>
    set((state) => {
      // Don't allow removing the base layer
      const layer = state.layers.find((l) => l.id === layerId);
      if (layer?.isBase) {
        console.warn('Cannot remove base layer');
        return state;
      }

      const newLayers = state.layers.filter((l) => l.id !== layerId);
      const newCurrentLayerId =
        state.currentLayerId === layerId
          ? state.layers.find((l) => l.isBase)?.id || state.layers[0]?.id
          : state.currentLayerId;

      return {
        layers: newLayers,
        currentLayerId: newCurrentLayerId,
        isDirty: true,
      };
    }),

  renameLayer: (layerId, newName) =>
    set((state) => ({
      layers: state.layers.map((layer) =>
        layer.id === layerId ? { ...layer, name: newName } : layer
      ),
      isDirty: true,
    })),

  setCurrentLayer: (layerId) =>
    set(() => ({
      currentLayerId: layerId,
    })),

  reorderLayers: (startIndex, endIndex) =>
    set((state) => {
      const newLayers = [...state.layers];
      const [removed] = newLayers.splice(startIndex, 1);
      newLayers.splice(endIndex, 0, removed);
      return {
        layers: newLayers,
        isDirty: true,
      };
    }),

  // Mapping management
  addMapping: (layerId, sourceKey, targetKey, type = 'simple') =>
    set((state) => {
      const mappingId = generateId();
      const newMapping: Mapping = {
        id: mappingId,
        sourceKey,
        targetKey,
        type,
      };

      return {
        layers: state.layers.map((layer) =>
          layer.id === layerId
            ? { ...layer, mappings: [...layer.mappings, newMapping] }
            : layer
        ),
        isDirty: true,
      };
    }),

  removeMapping: (layerId, mappingId) =>
    set((state) => ({
      layers: state.layers.map((layer) =>
        layer.id === layerId
          ? {
              ...layer,
              mappings: layer.mappings.filter((m) => m.id !== mappingId),
            }
          : layer
      ),
      isDirty: true,
    })),

  updateMapping: (layerId, mappingId, updates) =>
    set((state) => ({
      layers: state.layers.map((layer) =>
        layer.id === layerId
          ? {
              ...layer,
              mappings: layer.mappings.map((mapping) =>
                mapping.id === mappingId
                  ? { ...mapping, ...updates }
                  : mapping
              ),
            }
          : layer
      ),
      isDirty: true,
    })),

  clearMappings: (layerId) =>
    set((state) => ({
      layers: state.layers.map((layer) =>
        layer.id === layerId ? { ...layer, mappings: [] } : layer
      ),
      isDirty: true,
    })),

  // Modifier management
  addModifier: (name, triggerKey) =>
    set((state) => {
      const modifierId = generateId();
      const newModifier: Modifier = {
        id: modifierId,
        name,
        triggerKey,
        active: false,
      };
      return {
        modifiers: [...state.modifiers, newModifier],
        isDirty: true,
      };
    }),

  removeModifier: (modifierId) =>
    set((state) => ({
      modifiers: state.modifiers.filter((m) => m.id !== modifierId),
      isDirty: true,
    })),

  updateModifier: (modifierId, updates) =>
    set((state) => ({
      modifiers: state.modifiers.map((modifier) =>
        modifier.id === modifierId ? { ...modifier, ...updates } : modifier
      ),
      isDirty: true,
    })),

  // Lock management
  addLock: (name, triggerKey) =>
    set((state) => {
      const lockId = generateId();
      const newLock: Lock = {
        id: lockId,
        name,
        triggerKey,
        active: false,
      };
      return {
        locks: [...state.locks, newLock],
        isDirty: true,
      };
    }),

  removeLock: (lockId) =>
    set((state) => ({
      locks: state.locks.filter((l) => l.id !== lockId),
      isDirty: true,
    })),

  updateLock: (lockId, updates) =>
    set((state) => ({
      locks: state.locks.map((lock) =>
        lock.id === lockId ? { ...lock, ...updates } : lock
      ),
      isDirty: true,
    })),

  // State management
  setConfig: (config) =>
    set(() => ({
      ...config,
      isDirty: false,
    })),

  resetConfig: () =>
    set(() => ({
      ...createInitialState(),
    })),

  markDirty: () =>
    set(() => ({
      isDirty: true,
    })),

  markClean: () =>
    set(() => ({
      isDirty: false,
    })),
}));
