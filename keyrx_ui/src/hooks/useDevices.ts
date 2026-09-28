import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import * as deviceApi from '../api/devices';
import type { DeviceEntry } from '../types';

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
 * Fetch the global default keyboard layout
 */
export function useGlobalLayout() {
  return useQuery({
    queryKey: queryKeys.globalLayout,
    queryFn: deviceApi.fetchGlobalLayout,
  });
}

/**
 * Set the global default keyboard layout with cache update
 */
export function useSetGlobalLayout() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (layout: string) => deviceApi.setGlobalLayout(layout),
    onMutate: async (layout) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.globalLayout });
      const previous = queryClient.getQueryData<string>(queryKeys.globalLayout);
      queryClient.setQueryData(queryKeys.globalLayout, layout);
      return { previous };
    },
    onError: (_error, _variables, context) => {
      if (context?.previous) {
        queryClient.setQueryData(queryKeys.globalLayout, context.previous);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.globalLayout });
    },
    meta: { successMessage: 'Global layout saved' },
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
