/**
 * useUpdateDevice - React hook for updating device configuration
 *
 * This hook provides a unified mutation for updating device layout and scope settings.
 * It replaces separate PUT endpoints with a single PATCH endpoint for atomic updates.
 *
 * Features:
 * - PATCH /api/devices/:id for unified config updates
 * - Optimistic updates with rollback on error
 * - Automatic cache invalidation on success
 * - Loading and error state management
 *
 * @example
 * ```tsx
 * function DeviceSettings() {
 *   const { mutate: updateDevice, isPending } = useUpdateDevice();
 *
 *   const handleLayoutChange = (layout: string) => {
 *     updateDevice({ id: deviceId, layout });
 *   };
 *
 *   const handleScopeChange = (scope: DeviceScope) => {
 *     updateDevice({ id: deviceId, scope });
 *   };
 * }
 * ```
 */

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import { apiClient } from '../api/client';
import type { DeviceEntry, DeviceScope } from '../types';

/**
 * Request body for PATCH /api/devices/:id
 * Both fields are optional - send only what needs updating
 */
interface UpdateDeviceRequest {
  layout?: string | null;
  scope?: DeviceScope | null;
}

/**
 * Mutation variables for the hook
 */
interface UpdateDeviceVariables {
  id: string;
  layout?: string | null;
  scope?: DeviceScope | null;
}

/**
 * Backend response (currently not used, but follows existing patterns)
 */
interface DeviceResponse {
  success: boolean;
}

/**
 * Update device configuration (layout, scope, or both)
 *
 * Uses React Query mutation with optimistic updates for instant UI feedback.
 * Automatically invalidates device cache on success to ensure consistency.
 *
 * @returns Mutation object with:
 *   - mutate/mutateAsync: Update function (accepts { id, layout?, scope? })
 *   - isPending: Loading state
 *   - isError: Error state
 *   - error: Error details if mutation failed
 */
export function useUpdateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, layout, scope }: UpdateDeviceVariables) => {
      const request: UpdateDeviceRequest = {};

      // Only include fields that are provided
      if (layout !== undefined) {
        request.layout = layout;
      }
      if (scope !== undefined) {
        request.scope = scope;
      }

      return apiClient.patch<DeviceResponse>(`/api/devices/${id}`, request);
    },

    // Optimistic update: immediately update cache before API call
    onMutate: async ({ id, layout, scope }) => {
      // Cancel outgoing queries to avoid overwriting optimistic update
      await queryClient.cancelQueries({ queryKey: queryKeys.devices });

      // Snapshot previous value for rollback
      const previousDevices = queryClient.getQueryData<DeviceEntry[]>(
        queryKeys.devices
      );

      // Optimistically update cache
      queryClient.setQueryData<DeviceEntry[]>(
        queryKeys.devices,
        (old) =>
          old?.map((device) => {
            if (device.id !== id) return device;

            // Update only the fields that were provided
            const updated = { ...device };
            if (layout !== undefined) {
              updated.layout = layout;
            }
            if (scope !== undefined) {
              updated.scope = scope;
            }
            return updated;
          })
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
