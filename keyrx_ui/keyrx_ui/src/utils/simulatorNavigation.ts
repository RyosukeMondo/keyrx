/**
 * Utilities for navigating to the simulator with pre-loaded configurations.
 *
 * This module provides functions for passing configuration data between components,
 * particularly from a config editor to the simulator panel.
 */

export const CONFIG_STORAGE_KEY = 'keyrx_simulator_config';
export const CONFIG_PARAM_KEY = 'config';

/**
 * Navigate to the simulator with a pre-loaded configuration.
 * The configuration will be automatically loaded when the simulator mounts.
 *
 * @param config - The Rhai configuration source code to pre-load
 * @param method - The method to use for passing the config: 'url' (via URL params) or 'storage' (via sessionStorage)
 *
 * URL method: Encodes config in URL as base64 (cleaner URLs, shareable links)
 * Storage method: Stores in sessionStorage (better for large configs, not shareable)
 */
export function navigateToSimulatorWithConfig(
  config: string,
  method: 'url' | 'storage' = 'storage'
): void {
  if (method === 'url') {
    // Encode config as base64 to handle special characters in URL
    const encodedConfig = btoa(config);
    const url = new URL(window.location.origin);
    url.searchParams.set(CONFIG_PARAM_KEY, encodedConfig);
    url.hash = '#simulator'; // Assuming the app uses hash routing or will be updated to do so

    window.location.href = url.toString();
  } else {
    // Use sessionStorage (better for large configs)
    sessionStorage.setItem(CONFIG_STORAGE_KEY, config);

    // Navigate to simulator view
    // This assumes the app has some way to change views - adjust as needed
    window.location.hash = '#simulator';
  }
}

/**
 * Check if there's a pending config to load from URL or sessionStorage.
 * Used by SimulatorPanel on mount to detect auto-load scenarios.
 *
 * @returns The config string if found, null otherwise
 */
export function getPendingConfig(): string | null {
  // Check URL params first
  const urlParams = new URLSearchParams(window.location.search);
  const configFromUrl = urlParams.get(CONFIG_PARAM_KEY);

  if (configFromUrl) {
    try {
      return atob(configFromUrl);
    } catch {
      console.error('Failed to decode config from URL');
      return null;
    }
  }

  // Fallback to sessionStorage
  return sessionStorage.getItem(CONFIG_STORAGE_KEY);
}

/**
 * Clear any pending config from URL and sessionStorage.
 * Called by SimulatorPanel after successfully loading a config.
 */
export function clearPendingConfig(): void {
  // Clear URL param
  const url = new URL(window.location.href);
  if (url.searchParams.has(CONFIG_PARAM_KEY)) {
    url.searchParams.delete(CONFIG_PARAM_KEY);
    window.history.replaceState({}, '', url.toString());
  }

  // Clear sessionStorage
  sessionStorage.removeItem(CONFIG_STORAGE_KEY);
}
