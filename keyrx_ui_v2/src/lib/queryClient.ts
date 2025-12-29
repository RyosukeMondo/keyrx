import { QueryClient } from '@tanstack/react-query';

/**
 * React Query client configuration
 * Centralized configuration for caching, refetching, and error handling
 */
export const queryClient = new QueryClient({
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
 */
export const queryKeys = {
  // Device queries
  devices: ['devices'] as const,
  device: (id: string) => ['devices', id] as const,

  // Profile queries
  profiles: ['profiles'] as const,
  profile: (name: string) => ['profiles', name] as const,
  activeProfile: ['profiles', 'active'] as const,

  // Config queries
  config: (profile: string) => ['config', profile] as const,
  keyMapping: (profile: string, key: string) =>
    ['config', profile, 'key', key] as const,

  // Metrics queries
  latencyStats: ['metrics', 'latency'] as const,
  eventLog: ['metrics', 'events'] as const,
  daemonState: ['metrics', 'state'] as const,
} as const;
