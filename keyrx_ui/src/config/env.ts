/**
 * Environment Configuration
 * Type-safe access to Vite environment variables
 */

/**
 * Get the API base URL
 * In production, uses relative URL (same origin)
 * In development, uses configured URL or localhost
 */
export function getApiUrl(): string {
  const configuredUrl = import.meta.env.VITE_API_URL;

  // In production, if no URL configured, use same origin
  if (import.meta.env.PROD && !configuredUrl) {
    return window.location.origin;
  }

  // In development or if explicitly configured, use the configured URL
  return configuredUrl || 'http://localhost:9867';
}

/**
 * Get the WebSocket URL
 * In production, uses relative WebSocket URL (same origin)
 * In development, uses configured URL or localhost
 */
export function getWsUrl(): string {
  const configuredUrl = import.meta.env.VITE_WS_URL;

  // In production, if no URL configured, use same origin with ws/wss protocol
  if (import.meta.env.PROD && !configuredUrl) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${window.location.host}/ws-rpc`;
  }

  // In development or if explicitly configured, use the configured URL
  return configuredUrl || 'ws://localhost:9867/ws-rpc';
}

/**
 * Check if debug mode is enabled
 */
export function isDebugMode(): boolean {
  return import.meta.env.VITE_DEBUG === 'true' || import.meta.env.DEV;
}

/**
 * Get the current environment
 */
export function getEnvironment(): 'development' | 'production' {
  return import.meta.env.PROD ? 'production' : 'development';
}

/**
 * Environment configuration object
 */
export const env = {
  apiUrl: getApiUrl(),
  wsUrl: getWsUrl(),
  debug: isDebugMode(),
  environment: getEnvironment(),
  isDev: import.meta.env.DEV,
  isProd: import.meta.env.PROD,
} as const;

// Log configuration in debug mode
if (import.meta.env.DEV && isDebugMode()) {
  // eslint-disable-next-line no-console
  console.log('[ENV] Configuration:', {
    apiUrl: env.apiUrl,
    wsUrl: env.wsUrl,
    environment: env.environment,
    debug: env.debug,
  });
}
