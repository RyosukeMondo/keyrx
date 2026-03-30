import { QueryClient, MutationCache } from '@tanstack/react-query';
import { toast } from 'sonner';
import { getErrorMessage } from '../utils/errorUtils';

// Type-safe mutation meta for global toast control
declare module '@tanstack/react-query' {
  interface Register {
    mutationMeta: {
      /** Skip global error toast (for mutations with inline error UI) */
      suppressGlobalError?: boolean;
      /** Show success toast with this message */
      successMessage?: string;
    };
  }
}

/**
 * Global mutation cache — SSOT for mutation error/success notifications.
 *
 * MutationCache.onError fires for ALL mutations, alongside per-mutation
 * onError handlers (e.g., optimistic rollback). This means existing
 * mutations with onError for cache rollback need zero changes.
 */
const mutationCache = new MutationCache({
  onError: (error, _variables, _context, mutation) => {
    if (mutation.meta?.suppressGlobalError) return;
    toast.error(getErrorMessage(error, 'Operation failed'));
  },
  onSuccess: (_data, _variables, _context, mutation) => {
    if (mutation.meta?.successMessage) {
      toast.success(mutation.meta.successMessage);
    }
  },
});

/**
 * React Query client configuration
 * Centralized configuration for caching, refetching, and error handling
 */
export const queryClient = new QueryClient({
  mutationCache,
  defaultOptions: {
    queries: {
      // Stale time: data considered fresh for 30 seconds
      staleTime: 30 * 1000,

      // Cache time: unused data kept in cache for 5 minutes
      gcTime: 5 * 60 * 1000,

      // Refetch on window focus for real-time updates
      refetchOnWindowFocus: true,

      // Retry failed requests 3 times with exponential backoff
      retry: 3,
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),

      // Don't refetch on mount if data is fresh
      refetchOnMount: 'always',
    },
    mutations: {
      // Retry mutations once on failure
      retry: 1,
    },
  },
});

/**
 * Query keys for cache management
 * Centralized query key definitions for type safety and consistency
 *
 * Invalidation patterns:
 * - Profile activation: Invalidates activeProfile, daemonState, and all profileConfig queries
 * - Profile creation/deletion: Invalidates profiles list and activeProfile
 * - Config updates: Invalidates specific profileConfig and related config queries
 * - Device layout changes: Invalidates specific deviceLayout query
 */
export const queryKeys = {
  // Device queries
  devices: ['devices'] as const,
  device: (id: string) => ['devices', id] as const,
  deviceLayout: (serial: string) => ['devices', serial, 'layout'] as const,

  // Profile queries
  profiles: ['profiles'] as const,
  profile: (name: string) => ['profiles', name] as const,
  activeProfile: ['profiles', 'active'] as const,
  profileConfig: (name: string) => ['profiles', name, 'config'] as const,

  // Config queries (legacy - prefer profileConfig for new code)
  config: (profile: string) => ['config', profile] as const,
  keyMapping: (profile: string, key: string) =>
    ['config', profile, 'key', key] as const,

  // Metrics queries
  latencyStats: ['metrics', 'latency'] as const,
  eventLog: ['metrics', 'events'] as const,
  daemonState: ['metrics', 'state'] as const,
} as const;
