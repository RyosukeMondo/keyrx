/**
 * LocalStorage utility for persisting device enabled/disabled state
 *
 * Since the backend doesn't currently support device enabled state,
 * we persist this state client-side using localStorage.
 */

const STORAGE_KEY = 'keyrx:device:enabled';

interface DeviceEnabledState {
  [deviceId: string]: boolean;
}

/**
 * Get enabled state for all devices from localStorage
 */
export function getDeviceEnabledStates(): DeviceEnabledState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) {
      return {};
    }
    return JSON.parse(stored) as DeviceEnabledState;
  } catch (err) {
    console.error(
      'Failed to parse device enabled states from localStorage:',
      err
    );
    return {};
  }
}

/**
 * Get enabled state for a specific device
 * @returns true if enabled (or not found - default enabled), false if disabled
 */
export function isDeviceEnabled(deviceId: string): boolean {
  const states = getDeviceEnabledStates();
  // Default to enabled if not found
  return states[deviceId] !== false;
}

/**
 * Set enabled state for a specific device
 */
export function setDeviceEnabled(deviceId: string, enabled: boolean): void {
  try {
    const states = getDeviceEnabledStates();
    states[deviceId] = enabled;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(states));
  } catch (err) {
    console.error('Failed to persist device enabled state:', err);
    throw err;
  }
}

/**
 * Remove enabled state for a device (useful when device is forgotten)
 */
export function removeDeviceEnabledState(deviceId: string): void {
  try {
    const states = getDeviceEnabledStates();
    delete states[deviceId];
    localStorage.setItem(STORAGE_KEY, JSON.stringify(states));
  } catch (err) {
    console.error('Failed to remove device enabled state:', err);
  }
}

/**
 * Clear all device enabled states (useful for testing or reset)
 */
export function clearDeviceEnabledStates(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (err) {
    console.error('Failed to clear device enabled states:', err);
  }
}
