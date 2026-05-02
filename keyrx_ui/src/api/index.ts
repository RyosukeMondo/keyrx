/**
 * API client exports
 * Single entry point for all API interactions
 */

// Base client and error handling
export { apiClient, ApiError } from './client';

// Device management
export {
  fetchDevices,
  renameDevice,
  setDeviceScope,
  forgetDevice,
} from './devices';

// Profile management
export {
  fetchProfiles,
  createProfile,
  activateProfile,
  deleteProfile,
  duplicateProfile,
} from './profiles';

// Configuration management
export {
  fetchConfig,
  setKeyMapping,
  deleteKeyMapping,
  exportConfig,
  importConfig,
} from './config';

// Metrics and monitoring
export {
  fetchLatencyStats,
  fetchEventLog,
  fetchDaemonState,
  clearEventLog,
  fetchHealthStatus,
} from './metrics';

// WebSocket connection management
export {
  WebSocketManager,
  getWebSocketInstance,
  closeWebSocketInstance,
  type ConnectionState,
  type WebSocketConfig,
  type WebSocketCallbacks,
} from './websocket';

// RPC client (unified WebSocket API)
export { RpcClient } from './rpc';
export { ProfileRpcMethods } from './rpc-clients/profiles';
export { DeviceRpcMethods } from './rpc-clients/devices';
export { ConfigRpcMethods } from './rpc-clients/config';
export { MetricsRpcMethods } from './rpc-clients/metrics';
export { SimulationRpcMethods } from './rpc-clients/simulation';
export { SubscriptionRpcMethods } from './rpc-clients/subscriptions';
export { DaemonControlRpcMethods } from './rpc-clients/daemon';
export type {
  Profile,
  Device,
  Config,
  Layer,
  SimulationInput,
  SimulationResult,
  PaginatedEvents,
} from './types';
