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
