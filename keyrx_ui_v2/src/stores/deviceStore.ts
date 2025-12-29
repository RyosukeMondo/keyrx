import { create } from 'zustand';
import type { DeviceEntry, DeviceScope } from '../types';
import * as deviceApi from '../api/devices';
import { ApiError } from '../api/client';

interface DeviceStore {
  // State
  devices: DeviceEntry[];
  loading: boolean;
  error: string | null;

  // Actions
  fetchDevices: () => Promise<void>;
  renameDevice: (id: string, name: string) => Promise<void>;
  setScope: (id: string, scope: DeviceScope) => Promise<void>;
  forgetDevice: (id: string) => Promise<void>;
  clearError: () => void;
}

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  // Initial state
  devices: [],
  loading: false,
  error: null,

  // Fetch all devices
  fetchDevices: async () => {
    set({ loading: true, error: null });
    try {
      const devices = await deviceApi.fetchDevices();
      set({ devices, loading: false });
    } catch (error) {
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
    }
  },

  // Rename a device
  renameDevice: async (id: string, name: string) => {
    const { devices } = get();
    const oldDevices = [...devices];

    // Optimistic update
    const updatedDevices = devices.map((device) =>
      device.id === id ? { ...device, name } : device
    );
    set({ devices: updatedDevices, error: null });

    try {
      await deviceApi.renameDevice(id, name);
    } catch (error) {
      // Rollback on error
      set({ devices: oldDevices });
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Set device scope
  setScope: async (id: string, scope: DeviceScope) => {
    const { devices } = get();
    const oldDevices = [...devices];

    // Optimistic update
    const updatedDevices = devices.map((device) =>
      device.id === id ? { ...device, scope } : device
    );
    set({ devices: updatedDevices, error: null });

    try {
      await deviceApi.setDeviceScope(id, scope);
    } catch (error) {
      // Rollback on error
      set({ devices: oldDevices });
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Forget a device
  forgetDevice: async (id: string) => {
    const { devices } = get();
    const oldDevices = [...devices];

    // Optimistic update
    const updatedDevices = devices.filter((device) => device.id !== id);
    set({ devices: updatedDevices, error: null });

    try {
      await deviceApi.forgetDevice(id);
    } catch (error) {
      // Rollback on error
      set({ devices: oldDevices });
      const errorMessage =
        error instanceof ApiError ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Clear error state
  clearError: () => set({ error: null }),
}));
