import { create } from 'zustand';
import type { KeyMapping } from '../types';
import * as configApi from '../api/config';
import { ApiError } from '../api/client';

interface ConfigStore {
  // State
  currentProfile: string | null;
  activeLayer: string;
  keyMappings: Map<string, KeyMapping>;
  loading: boolean;
  error: string | null;

  // Actions
  fetchConfig: (profile: string) => Promise<void>;
  setKeyMapping: (key: string, mapping: KeyMapping) => Promise<void>;
  deleteKeyMapping: (key: string) => Promise<void>;
  switchLayer: (layerId: string) => void;
  clearError: () => void;
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  // Initial state
  currentProfile: null,
  activeLayer: 'base',
  keyMappings: new Map(),
  loading: false,
  error: null,

  // Fetch configuration for a profile
  fetchConfig: async (profile: string) => {
    set({ loading: true, error: null });
    try {
      const data = await configApi.fetchConfig(profile);

      // Convert key mappings object to Map
      const keyMappings = new Map<string, KeyMapping>(
        Object.entries(data.keyMappings || {})
      );

      set({
        currentProfile: profile,
        keyMappings,
        activeLayer: data.activeLayer || 'base',
        loading: false,
      });
    } catch (error) {
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
    }
  },

  // Set or update a key mapping
  setKeyMapping: async (key: string, mapping: KeyMapping) => {
    const { currentProfile, keyMappings } = get();

    if (!currentProfile) {
      const errorMessage = 'No profile loaded';
      set({ error: errorMessage });
      throw new Error(errorMessage);
    }

    // Store old mapping for rollback
    const oldMappings = new Map(keyMappings);

    // Optimistic update
    const updatedMappings = new Map(keyMappings);
    updatedMappings.set(key, mapping);
    set({ keyMappings: updatedMappings, error: null });

    try {
      await configApi.setKeyMapping(currentProfile, key, mapping);
    } catch (error) {
      // Rollback on error
      set({ keyMappings: oldMappings });
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Delete a key mapping (restore to default)
  deleteKeyMapping: async (key: string) => {
    const { currentProfile, keyMappings } = get();

    if (!currentProfile) {
      const errorMessage = 'No profile loaded';
      set({ error: errorMessage });
      throw new Error(errorMessage);
    }

    // Store old mapping for rollback
    const oldMappings = new Map(keyMappings);

    // Optimistic update
    const updatedMappings = new Map(keyMappings);
    updatedMappings.delete(key);
    set({ keyMappings: updatedMappings, error: null });

    try {
      await configApi.deleteKeyMapping(currentProfile, key);
    } catch (error) {
      // Rollback on error
      set({ keyMappings: oldMappings });
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Switch active layer (local state only, doesn't persist)
  switchLayer: (layerId: string) => {
    set({ activeLayer: layerId });
  },

  // Clear error state
  clearError: () => set({ error: null }),
}));
