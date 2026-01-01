/**
 * React Query hooks for KeyRx UI
 * Centralized exports for all data fetching and mutation hooks
 */

// Device hooks
export {
  useDevices,
  useRenameDevice,
  useSetDeviceScope,
  useForgetDevice,
} from './useDevices';

// Profile hooks
export {
  useProfiles,
  useActiveProfile,
  useCreateProfile,
  useActivateProfile,
  useDeleteProfile,
} from './useProfiles';

// Config hooks
export {
  useConfig,
  useSetKeyMapping,
  useDeleteKeyMapping,
} from './useConfig';

// Metrics hooks
export {
  useLatencyStats,
  useEventLog,
  useWebSocketMetrics,
  useDaemonState,
} from './useMetrics';

// WebSocket RPC hooks
export { useUnifiedApi, type UseUnifiedApiReturn } from './useUnifiedApi';
