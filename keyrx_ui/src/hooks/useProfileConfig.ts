import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import { useUnifiedApi } from './useUnifiedApi';
import { RpcClient } from '../api/rpc';
import type { ProfileConfig } from '../api/types';

/**
 * Fetch profile configuration source code with React Query caching
 *
 * @param name - Profile name to fetch configuration for
 * @returns Query result with profile configuration
 */
export function useGetProfileConfig(name: string) {
  const api = useUnifiedApi();
  const client = new RpcClient(api);

  return useQuery({
    queryKey: queryKeys.config(name),
    queryFn: () => client.getProfileConfig(name),
    enabled: !!name && api.isConnected, // Only fetch if name is provided and WebSocket connected
    staleTime: 30000, // Consider data fresh for 30 seconds
    gcTime: 300000, // Keep in cache for 5 minutes
    retry: 1, // Only retry once on failure
    retryDelay: 1000, // Wait 1 second before retry
  });
}

/**
 * Update profile configuration source code with optimistic updates
 *
 * @returns Mutation function and status
 */
export function useSetProfileConfig() {
  const queryClient = useQueryClient();
  const api = useUnifiedApi();
  const client = new RpcClient(api);

  return useMutation({
    mutationFn: ({ name, source }: { name: string; source: string }) =>
      client.setProfileConfig(name, source),

    onMutate: async ({ name, source }) => {
      // Cancel outgoing queries for this profile config
      await queryClient.cancelQueries({ queryKey: queryKeys.config(name) });

      // Snapshot previous value
      const previousConfig = queryClient.getQueryData<ProfileConfig>(
        queryKeys.config(name)
      );

      // Optimistically update to new value
      queryClient.setQueryData<ProfileConfig>(queryKeys.config(name), {
        name,
        source,
      });

      return { previousConfig };
    },

    onError: (_error, { name }, context) => {
      // Rollback to previous value on error
      if (context?.previousConfig) {
        queryClient.setQueryData(
          queryKeys.config(name),
          context.previousConfig
        );
      }
    },

    onSuccess: (_data, { name }) => {
      // Invalidate related queries
      queryClient.invalidateQueries({ queryKey: queryKeys.config(name) });
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
    },
  });
}
