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
 * Get the currently active profile from the profiles list
 * @deprecated Use useActiveProfileQuery for direct API access
 */
export function useActiveProfile() {
  const { data: profiles } = useProfiles();
  return profiles?.find((p) => p.isActive) ?? null;
}

/**
 * Fetch active profile name directly from the daemon
 * This provides a dedicated query that can be invalidated independently
 */
export function useActiveProfileQuery() {
  return useQuery({
    queryKey: queryKeys.activeProfile,
    queryFn: profileApi.fetchActiveProfile,
    staleTime: 10 * 1000, // 10 seconds - more frequent checks for active profile
  });
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

    // Also refetch on error - profile might exist but cache is stale
    onError: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
    },
  });
}

/**
 * Activate a profile with optimistic updates.
 *
 * The REST API compiles the profile and sets it active. The daemon's message
 * loop detects the change via shared state and hot-reloads automatically.
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

    onSuccess: async (result, _variables, context) => {
      // Only proceed if there are no compilation errors
      if (result.errors && result.errors.length > 0) {
        // Rollback optimistic update on compilation error
        if (context?.previousProfiles) {
          queryClient.setQueryData(
            queryKeys.profiles,
            context.previousProfiles
          );
        }
        return;
      }

      // Profile activated successfully — the REST API already compiled and
      // set the active profile. The daemon's message loop detects the change
      // via shared state and hot-reloads automatically. No restart needed.
      queryClient.invalidateQueries({ queryKey: queryKeys.profiles });
      queryClient.invalidateQueries({ queryKey: ['config'] });
      queryClient.invalidateQueries({ queryKey: queryKeys.daemonState });
      queryClient.invalidateQueries({ queryKey: queryKeys.activeProfile });
    },
  });
}

/**
 * Update a profile with optimistic updates
 */
export function useUpdateProfile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      originalName,
      updates,
    }: {
      originalName: string;
      updates: { name?: string; description?: string };
    }) => profileApi.updateProfile(originalName, updates),

    onMutate: async ({ originalName, updates }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.profiles });

      const previousProfiles = queryClient.getQueryData<ProfileMetadata[]>(
        queryKeys.profiles
      );

      // Optimistically update profile
      queryClient.setQueryData<ProfileMetadata[]>(
        queryKeys.profiles,
        (old) =>
          old?.map((p) => {
            if (p.name === originalName) {
              return {
                ...p,
                ...(updates.name && { name: updates.name }),
                // Note: description is not in ProfileMetadata, so we can't update it here
                // But the API call will succeed
              };
            }
            return p;
          })
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
      queryClient.setQueryData<ProfileMetadata[]>(
        queryKeys.profiles,
        (old) => old?.filter((p) => p.name !== name)
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
