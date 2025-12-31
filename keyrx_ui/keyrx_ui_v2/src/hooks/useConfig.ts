import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import * as configApi from '../api/config';
import type { KeyMapping } from '../types';

/**
 * Fetch configuration for a specific profile
 */
export function useConfig(profile: string | null) {
  return useQuery({
    queryKey: queryKeys.config(profile ?? ''),
    queryFn: () => {
      if (!profile) {
        throw new Error('No profile specified');
      }
      return configApi.fetchConfig(profile);
    },
    enabled: !!profile, // Only run query if profile is provided
  });
}

/**
 * Set or update a key mapping with optimistic updates
 */
export function useSetKeyMapping(profile: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ key, mapping }: { key: string; mapping: KeyMapping }) =>
      configApi.setKeyMapping(profile, key, mapping),

    onMutate: async ({ key, mapping }) => {
      const queryKey = queryKeys.config(profile);

      // Cancel outgoing queries
      await queryClient.cancelQueries({ queryKey });

      // Snapshot previous value
      const previousConfig = queryClient.getQueryData<{
        keyMappings: Record<string, KeyMapping>;
        activeLayer: string;
      }>(queryKey);

      // Optimistically update cache
      queryClient.setQueryData(queryKey, (old) => {
        if (!old) return old;
        return {
          ...old,
          keyMappings: {
            ...old.keyMappings,
            [key]: mapping,
          },
        };
      });

      return { previousConfig };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousConfig) {
        queryClient.setQueryData(
          queryKeys.config(profile),
          context.previousConfig
        );
      }
    },

    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.config(profile) });
    },
  });
}

/**
 * Delete a key mapping (restore to default) with optimistic updates
 */
export function useDeleteKeyMapping(profile: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (key: string) => configApi.deleteKeyMapping(profile, key),

    onMutate: async (key) => {
      const queryKey = queryKeys.config(profile);

      await queryClient.cancelQueries({ queryKey });

      const previousConfig = queryClient.getQueryData<{
        keyMappings: Record<string, KeyMapping>;
        activeLayer: string;
      }>(queryKey);

      // Optimistically remove key mapping
      queryClient.setQueryData(queryKey, (old) => {
        if (!old) return old;
        const { [key]: removed, ...remainingMappings } = old.keyMappings;
        return {
          ...old,
          keyMappings: remainingMappings,
        };
      });

      return { previousConfig };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousConfig) {
        queryClient.setQueryData(
          queryKeys.config(profile),
          context.previousConfig
        );
      }
    },

    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.config(profile) });
    },
  });
}
