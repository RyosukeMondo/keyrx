/**
 * Central configuration for keyrx UI
 *
 * This is the single source of truth for all configuration values including
 * ports, URLs, environment-based overrides, and magic numbers.
 *
 * Configuration hierarchy (highest to lowest priority):
 * 1. Environment variables (VITE_*)
 * 2. .env files (.env.production, .env.development)
 * 3. Defaults defined in this file
 */

// ============================================================================
// Server Configuration (Ports & Addresses)
// ============================================================================

/** Default daemon API port */
export const DEFAULT_DAEMON_PORT = 9867;

/** Default daemon hostname */
export const DEFAULT_DAEMON_HOST = 'localhost';

/** Default daemon IP address */
export const DEFAULT_DAEMON_IP = '127.0.0.1';

// ============================================================================
// API Configuration (URLs and Endpoints)
// ============================================================================

/**
 * Determine API base URL from environment or defaults
 *
 * Priority:
 * 1. VITE_API_URL environment variable (set in .env files)
 * 2. Production: same host as UI (empty string, uses relative URLs)
 * 3. Development: http://localhost:9867
 */
export const API_BASE_URL =
  import.meta.env.VITE_API_URL ??
  (import.meta.env.PROD
    ? '' // Production: use relative URLs (same host as UI)
    : `http://${DEFAULT_DAEMON_HOST}:${DEFAULT_DAEMON_PORT}`);

/**
 * Determine WebSocket URL from environment or defaults
 *
 * Priority:
 * 1. VITE_WS_URL environment variable (set in .env files)
 * 2. Production: ws://[same-host]/ws-rpc (same host as UI)
 * 3. Development: ws://localhost:9867/ws-rpc
 */
export const WS_BASE_URL =
  import.meta.env.VITE_WS_URL ??
  (import.meta.env.PROD
    ? `ws://${window.location.host}/ws-rpc` // Production: same host as UI
    : `ws://${DEFAULT_DAEMON_HOST}:${DEFAULT_DAEMON_PORT}/ws-rpc`);

// ============================================================================
// API Endpoints (relative paths)
// ============================================================================

/** API endpoint prefixes */
export const API_ENDPOINTS = {
  // Health & Status
  health: '/api/health',
  diagnostics: '/api/diagnostics',
  status: '/api/status',

  // Profiles
  profiles: '/api/profiles',
  profileActivate: (profileId: string) => `/api/profiles/${profileId}/activate`,
  profileDeactivate: (profileId: string) => `/api/profiles/${profileId}/deactivate`,
  profileDelete: (profileId: string) => `/api/profiles/${profileId}`,

  // Configuration
  config: (profile: string) => `/api/config/${profile}`,
  configKey: (profile: string) => `/api/config/${profile}/key`,
  configExport: (profile: string) => `/api/config/${profile}/export`,
  configImport: (profile: string) => `/api/config/${profile}/import`,

  // Devices
  devices: '/api/devices',
  deviceRename: (deviceId: string) => `/api/devices/${deviceId}/rename`,
  deviceScope: (deviceId: string) => `/api/devices/${deviceId}/scope`,
  deviceLayout: (deviceId: string) => `/api/devices/${deviceId}/layout`,

  // Keyboard Simulator
  simulatorEvents: '/api/simulator/events',

  // Metrics & Monitoring
  metrics: '/api/metrics',
  metricsEvents: '/api/metrics/events',
  metricsEventsClear: '/api/metrics/events',
} as const;

// ============================================================================
// WebSocket Configuration
// ============================================================================

/** WebSocket RPC endpoint path */
export const WS_RPC_PATH = '/ws-rpc';

/** WebSocket reconnection options */
export const WS_RECONNECT_CONFIG = {
  maxRetries: 5,
  initialDelayMs: 1000,
  maxDelayMs: 30000,
  backoffMultiplier: 1.5,
} as const;

// ============================================================================
// Environment Configuration
// ============================================================================

/** Current environment */
export const ENVIRONMENT = import.meta.env.VITE_ENV ?? 'development';

/** Is production build */
export const IS_PRODUCTION = import.meta.env.PROD || ENVIRONMENT === 'production';

/** Is development build */
export const IS_DEVELOPMENT = !IS_PRODUCTION || ENVIRONMENT === 'development';

/** Debug logging enabled */
export const DEBUG_ENABLED =
  import.meta.env.VITE_DEBUG === 'true' || IS_DEVELOPMENT;

// ============================================================================
// UI Configuration (Magic Numbers)
// ============================================================================

/** Request timeout in milliseconds */
export const REQUEST_TIMEOUT_MS = 30000;

/** WebSocket heartbeat interval in milliseconds */
export const WS_HEARTBEAT_INTERVAL_MS = 30000;

/** Default polling interval for metrics (ms) */
export const METRICS_POLL_INTERVAL_MS = 5000;

/** Maximum metrics events to fetch at once */
export const METRICS_MAX_EVENTS = 1000;

/** Key mapping search debounce delay (ms) */
export const SEARCH_DEBOUNCE_MS = 300;

/** Toast notification duration (ms) */
export const TOAST_DURATION_MS = 5000;

/** Modal animation duration (ms) */
export const MODAL_ANIMATION_MS = 300;

/** Auto-save debounce delay (ms) */
export const AUTOSAVE_DEBOUNCE_MS = 1000;

/** Profile list refresh interval (ms) */
export const PROFILE_REFRESH_INTERVAL_MS = 10000;

/** Device list refresh interval (ms) */
export const DEVICE_REFRESH_INTERVAL_MS = 5000;

// ============================================================================
// Feature Flags & Capabilities
// ============================================================================

/** Features available in this build */
export const FEATURES = {
  /** Enable WebSocket RPC functionality */
  wsRpc: true,

  /** Enable macro recording */
  macroRecording: true,

  /** Enable device management */
  deviceManagement: true,

  /** Enable keyboard simulator */
  simulator: true,

  /** Enable profile analytics */
  analytics: !IS_PRODUCTION,

  /** Enable development tools */
  devTools: IS_DEVELOPMENT,
} as const;

// ============================================================================
// Validation & Constraints
// ============================================================================

/** Validation constraints */
export const VALIDATION = {
  /** Max profile name length */
  maxProfileNameLength: 255,

  /** Max key mapping per profile */
  maxKeyMappings: 10000,

  /** Max layers per profile */
  maxLayersPerProfile: 50,

  /** Min port number */
  minPort: 1024,

  /** Max port number */
  maxPort: 65535,
} as const;

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Build a full API URL from an endpoint path
 *
 * @param endpoint - Relative API endpoint path
 * @returns Full URL
 *
 * @example
 * buildApiUrl('/api/profiles') // => 'http://localhost:9867/api/profiles'
 * buildApiUrl('/api/health')   // => 'http://localhost:9867/api/health'
 */
export function buildApiUrl(endpoint: string): string {
  // Remove leading slash if present to avoid double slashes
  const cleanEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;

  // If API_BASE_URL is empty (production), use relative URLs
  if (!API_BASE_URL) {
    return cleanEndpoint;
  }

  // Remove trailing slash from base URL
  const cleanBase = API_BASE_URL.endsWith('/')
    ? API_BASE_URL.slice(0, -1)
    : API_BASE_URL;

  return `${cleanBase}${cleanEndpoint}`;
}

/**
 * Build WebSocket URL for RPC endpoint
 *
 * @returns Full WebSocket URL
 *
 * @example
 * buildWsUrl() // => 'ws://localhost:9867/ws-rpc'
 */
export function buildWsUrl(): string {
  // If WS_BASE_URL already includes the path, return as-is
  if (WS_BASE_URL.includes('/ws-rpc')) {
    return WS_BASE_URL;
  }

  // Remove trailing slash
  const cleanBase = WS_BASE_URL.endsWith('/')
    ? WS_BASE_URL.slice(0, -1)
    : WS_BASE_URL;

  return `${cleanBase}${WS_RPC_PATH}`;
}

/**
 * Get daemon connection info for display purposes
 *
 * @returns Connection info string
 *
 * @example
 * getDaemonConnectionInfo() // => 'localhost:9867'
 */
export function getDaemonConnectionInfo(): string {
  try {
    const url = new URL(API_BASE_URL || `http://${DEFAULT_DAEMON_HOST}:${DEFAULT_DAEMON_PORT}`);
    return `${url.hostname}:${url.port || DEFAULT_DAEMON_PORT}`;
  } catch {
    return `${DEFAULT_DAEMON_HOST}:${DEFAULT_DAEMON_PORT}`;
  }
}
