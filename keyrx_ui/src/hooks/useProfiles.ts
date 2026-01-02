import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import * as profileApi from '../api/profiles';
import type { ProfileMetadata, Template } from '../types';

/**
 * Fetch all profiles with React Query caching
 */
export function useProfiles() {
  return useQuery({
    queryKey: queryKeys.profiles,
    queryFn: profileApi.fetchProfiles,
  });
}

/**
 * Get the currently active profile
 */
export function useActiveProfile() {
  const { data: profiles } = useProfiles();
  return profiles?.find((p) => p.isActive) ?? null;
}

/**
 * Create a new profile with cache invalidation
 */
export function useCreateProfile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ name, template }: { name: string; template: Template }) =>
      profileApi.createProfile(name, template),

    // Invalidate and refetch profiles list after creation
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
    },
  });
}

/**
 * Activate a profile with optimistic updates
 */
export function useActivateProfile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => profileApi.activateProfile(name),

    onMutate: async (name) => {
      // Cancel outgoing queries
      await queryClient.cancelQueries({ queryKey: queryKeys.profiles });

      // Snapshot previous value
      const previousProfiles = queryClient.getQueryData<ProfileMetadata[]>(
        queryKeys.profiles
      );

      // Optimistically update: set all to inactive, target to active
      queryClient.setQueryData<ProfileMetadata[]>(
        queryKeys.profiles,
        (old) =>
          old?.map((p) => ({
            ...p,
            isActive: p.name === name,
          }))
      );

      return { previousProfiles };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousProfiles) {
        queryClient.setQueryData(queryKeys.profiles, context.previousProfiles);
      }
    },

    onSuccess: () => {
      // Invalidate profiles, config, daemon state, and active profile queries
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
      queryClient.invalidateQueries({ queryKey: ['config'] });
      queryClient.invalidateQueries({ queryKey: queryKeys.daemonState });
      queryClient.invalidateQueries({ queryKey: queryKeys.activeProfile });
    },
  });
}

/**
 * Delete a profile with optimistic updates
 */
export function useDeleteProfile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (name: string) => {
      // Check if profile is active before attempting deletion
      const profiles = queryClient.getQueryData<ProfileMetadata[]>(
        queryKeys.profiles
      );
      const profile = profiles?.find((p) => p.name === name);

      if (profile?.isActive) {
        throw new Error('Cannot delete the active profile');
      }

      return profileApi.deleteProfile(name);
    },

    onMutate: async (name) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.profiles });

      const previousProfiles = queryClient.getQueryData<ProfileMetadata[]>(
        queryKeys.profiles
      );

      // Optimistically remove profile
      queryClient.setQueryData<ProfileMetadata[]>(queryKeys.profiles, (old) =>
        old?.filter((p) => p.name !== name)
      );

      return { previousProfiles };
    },

    onError: (_error, _variables, context) => {
      if (context?.previousProfiles) {
        queryClient.setQueryData(queryKeys.profiles, context.previousProfiles);
      }
    },

    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
    },
  });
}
