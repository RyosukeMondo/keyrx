import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '../lib/queryClient';
import { fetchDaemonStatus } from '../api/status';

/** Poll the daemon status (active profile, grabbed keyboards). */
export function useDaemonStatus() {
  return useQuery({
    queryKey: queryKeys.daemonStatus,
    queryFn: fetchDaemonStatus,
    refetchInterval: 5000,
    staleTime: 2000,
  });
}
