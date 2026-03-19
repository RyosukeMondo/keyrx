import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import * as deviceApi from '../api/devices';
import type { DeviceEntry, DeviceScope } from '../types';

/**
 * Fetch all devices with React Query caching
 */
export function useDevices() {
  return useQuery({
    queryKey: queryKeys.devices,
    queryFn: deviceApi.fetchDevices,
  });
}

/**
 * Rename a device with optimistic updates and cache invalidation
 */
export function useRenameDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      deviceApi.renameDevice(id, name),

    // Optimistic update: immediately update cache before API call
    onMutate: async ({ id, name }) => {
      // Cancel outgoing queries to avoid overwriting optimistic update
      await queryClient.cancelQueries({ queryKey: queryKeys.devices });

      // Snapshot previous value for rollback
      const previousDevices = queryClient.getQueryData<DeviceEntry[]>(
        queryKeys.devices
      );

      // Optimistically update cache
      queryClient.setQueryData<DeviceEntry[]>(queryKeys.devices, (old) =>
        old?.map((device) => (device.id === id ? { ...device, name } : device))
      );

      // Return context for rollback
      return { previousDevices };
    },

    // Rollback on error
    onError: (_error, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKeys.devices, context.previousDevices);
      }
    },

    // Refetch on success to ensure data consistency
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.devices });
    },
  });
}

/**
 * Set device scope with optimistic updates
 */
export function useSetDeviceScope() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, scope }: { id: string; scope: DeviceScope }) =>
      deviceApi.setDeviceScope(id, scope),

    onMutate: async ({ id, scope }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.devices });

      const previousDevices = queryClient.getQueryData<DeviceEntry[]>(
        queryKeys.devices
      );

      queryClient.setQueryData<DeviceEntry[]>(queryKeys.devices, (old) =>
        old?.map((device) => (device.id === id ? { ...device, scope } : device))
      );

      return { previousDevices };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKeys.devices, context.previousDevices);
      }
    },

    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.devices });
    },
  });
}

/**
 * Forget a device with optimistic updates
 */
export function useForgetDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deviceApi.forgetDevice(id),

    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.devices });

      const previousDevices = queryClient.getQueryData<DeviceEntry[]>(
        queryKeys.devices
      );

      queryClient.setQueryData<DeviceEntry[]>(queryKeys.devices, (old) =>
        old?.filter((device) => device.id !== id)
      );

      return { previousDevices };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKeys.devices, context.previousDevices);
      }
    },

    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.devices });
    },
  });
}

/**
 * Set device enabled/disabled state with optimistic updates
 *
 * This manages client-side enabled state (persisted to localStorage).
 * Disabled devices are hidden from the UI but not forgotten.
 */
export function useSetDeviceEnabled() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      deviceApi.setDeviceEnabled(id, enabled),

    onMutate: async ({ id, enabled }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.devices });

      const previousDevices = queryClient.getQueryData<DeviceEntry[]>(
        queryKeys.devices
      );

      // Optimistically update the enabled state
      queryClient.setQueryData<DeviceEntry[]>(queryKeys.devices, (old) =>
        old?.map((device) =>
          device.id === id ? { ...device, enabled } : device
        )
      );

      return { previousDevices };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKeys.devices, context.previousDevices);
      }
    },

    // No need to invalidate since this is client-side only and optimistic update is accurate
    onSuccess: () => {
      // Could add a success toast here if desired
    },
  });
}
