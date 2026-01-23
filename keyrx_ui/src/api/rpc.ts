/**
 * Type-safe RPC client for WebSocket communication with the daemon.
 *
 * This client wraps the useUnifiedApi hook and provides typed methods for all
 * RPC operations. All methods are thin wrappers that delegate to the underlying
 * query/command/subscribe methods with proper type safety.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const api = useUnifiedApi();
 *   const client = new RpcClient(api);
 *
 *   // Query profiles
 *   const profiles = await client.getProfiles();
 *
 *   // Activate a profile
 *   await client.activateProfile('Gaming');
 *
 *   // Subscribe to state changes
 *   client.onDaemonState((state) => {
 *     console.log('State changed:', state);
 *   });
 * }
 * ```
 */

import type { UseUnifiedApiReturn } from '../hooks/useUnifiedApi';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../types/rpc';
import type {
  Profile,
  ProfileConfig,
  Device,
  Config,
  Layer,
  SimulationInput,
  SimulationResult,
  PaginatedEvents,
} from './types';
import { validateApiResponse, ProfileConfigRpcSchema } from './schemas';
import type { z } from 'zod';

/**
 * Type-safe RPC client for daemon communication.
 */
export class RpcClient {
  private api: UseUnifiedApiReturn;

  /**
   * Create a new RPC client.
   *
   * @param api - The useUnifiedApi hook return value
   */
  constructor(api: UseUnifiedApiReturn) {
    this.api = api;
  }

  // ========================================
  // Profile Methods
  // ========================================

  /**
   * Get all profiles.
   *
   * @returns Array of profile metadata
   * @throws Error if the request fails
   */
  async getProfiles(): Promise<Profile[]> {
    return this.api.query<Profile[]>('get_profiles');
  }

  /**
   * Create a new profile.
   *
   * @param name - Name of the profile to create
   * @param template - Optional template to initialize from
   * @returns Success status
   * @throws Error if profile name is invalid or profile already exists
   */
  async createProfile(name: string, template?: string): Promise<void> {
    return this.api.command<void>('create_profile', { name, template });
  }

  /**
   * Activate a profile.
   *
   * @param name - Name of the profile to activate
   * @returns Success status
   * @throws Error if profile does not exist or activation fails
   */
  async activateProfile(name: string): Promise<void> {
    return this.api.command<void>('activate_profile', { name });
  }

  /**
   * Delete a profile.
   *
   * @param name - Name of the profile to delete
   * @returns Success status
   * @throws Error if profile does not exist or is currently active
   */
  async deleteProfile(name: string): Promise<void> {
    return this.api.command<void>('delete_profile', { name });
  }

  /**
   * Duplicate a profile.
   *
   * @param sourceName - Name of the profile to duplicate
   * @param newName - Name for the new profile
   * @returns Success status
   * @throws Error if source does not exist or new name is invalid
   */
  async duplicateProfile(sourceName: string, newName: string): Promise<void> {
    return this.api.command<void>('duplicate_profile', {
      source_name: sourceName,
      new_name: newName,
    });
  }

  /**
   * Rename a profile.
   *
   * @param oldName - Current name of the profile
   * @param newName - New name for the profile
   * @returns Success status
   * @throws Error if profile does not exist or new name is invalid
   */
  async renameProfile(oldName: string, newName: string): Promise<void> {
    return this.api.command<void>('rename_profile', {
      old_name: oldName,
      new_name: newName,
    });
  }

  /**
   * Get profile configuration source code.
   *
   * @param name - Name of the profile
   * @returns Profile configuration with name and source code
   * @throws Error if profile does not exist
   */
  async getProfileConfig(name: string): Promise<ProfileConfig> {
    const response = await this.api.query<
      z.infer<typeof ProfileConfigRpcSchema>
    >('get_profile_config', { name });
    const validated = validateApiResponse(
      ProfileConfigRpcSchema,
      response,
      'get_profile_config'
    );
    return {
      name: validated.name,
      source: validated.source, // Backend returns 'source' field
    };
  }

  /**
   * Set profile configuration source code.
   *
   * @param name - Name of the profile
   * @param source - Rhai configuration source code
   * @returns Success status
   * @throws Error if profile does not exist or source is invalid
   */
  async setProfileConfig(name: string, source: string): Promise<void> {
    await this.api.command<void>('set_profile_config', {
      name,
      source,
    });
    // For commands, the response is typically empty on success
    // Validation already happened at the WebSocket layer
    if (import.meta.env.DEV) {
      // eslint-disable-next-line no-console
      console.debug(
        JSON.stringify({
          timestamp: new Date().toISOString(),
          level: 'debug',
          service: 'RPC Client',
          event: 'set_profile_config_success',
          context: { profileName: name },
        })
      );
    }
  }

  /**
   * Get the currently active profile name.
   *
   * @returns Active profile name or null if no profile is active
   * @throws Error if the request fails
   */
  async getActiveProfile(): Promise<string | null> {
    return this.api.query<string | null>('get_active_profile');
  }

  // ========================================
  // Device Methods
  // ========================================

  /**
   * Get all devices.
   *
   * @returns Array of device information
   * @throws Error if the request fails
   */
  async getDevices(): Promise<Device[]> {
    return this.api.query<Device[]>('get_devices');
  }

  /**
   * Rename a device.
   *
   * @param serial - Device serial number
   * @param newName - New name for the device
   * @returns Success status
   * @throws Error if device does not exist or name is invalid
   */
  async renameDevice(serial: string, newName: string): Promise<void> {
    return this.api.command<void>('rename_device', {
      serial,
      new_name: newName,
    });
  }

  /**
   * Set device scope.
   *
   * @param serial - Device serial number
   * @param scope - Scope to set ('global' or 'profile')
   * @returns Success status
   * @throws Error if device does not exist or scope is invalid
   */
  async setScopeDevice(
    serial: string,
    scope: 'global' | 'profile'
  ): Promise<void> {
    return this.api.command<void>('set_scope_device', { serial, scope });
  }

  /**
   * Forget a device (remove from configuration).
   *
   * @param serial - Device serial number
   * @returns Success status
   * @throws Error if device does not exist
   */
  async forgetDevice(serial: string): Promise<void> {
    return this.api.command<void>('forget_device', { serial });
  }

  /**
   * Set device keyboard layout preference.
   *
   * @param serial - Device serial number
   * @param layout - Layout name (e.g., 'ansi', 'iso', 'jis')
   * @returns Success status
   * @throws Error if device does not exist or layout is invalid
   */
  async setDeviceLayout(serial: string, layout: string): Promise<void> {
    return this.api.command<void>('set_device_layout', { serial, layout });
  }

  // ========================================
  // Configuration Methods
  // ========================================

  /**
   * Get current configuration.
   *
   * @returns Configuration code and hash
   * @throws Error if the request fails
   */
  async getConfig(): Promise<Config> {
    return this.api.query<Config>('get_config');
  }

  /**
   * Update configuration.
   *
   * @param code - New configuration code
   * @returns Success status
   * @throws Error if configuration is invalid or exceeds size limit
   */
  async updateConfig(code: string): Promise<void> {
    return this.api.command<void>('update_config', { code });
  }

  /**
   * Set a single key mapping.
   *
   * @param layer - Layer name
   * @param keyCode - Key code to map
   * @param mapping - Mapping definition
   * @returns Success status
   * @throws Error if layer or mapping is invalid
   */
  async setKeyMapping(
    layer: string,
    keyCode: string,
    mapping: string
  ): Promise<void> {
    return this.api.command<void>('set_key_mapping', {
      layer,
      key_code: keyCode,
      mapping,
    });
  }

  /**
   * Delete a key mapping.
   *
   * @param layer - Layer name
   * @param keyCode - Key code to unmap
   * @returns Success status
   * @throws Error if layer does not exist
   */
  async deleteKeyMapping(layer: string, keyCode: string): Promise<void> {
    return this.api.command<void>('delete_key_mapping', {
      layer,
      key_code: keyCode,
    });
  }

  /**
   * Get all layers.
   *
   * @returns Array of layer names
   * @throws Error if the request fails
   */
  async getLayers(): Promise<Layer[]> {
    return this.api.query<Layer[]>('get_layers');
  }

  // ========================================
  // Metrics Methods
  // ========================================

  /**
   * Get current latency statistics.
   *
   * @returns Latency metrics
   * @throws Error if the request fails
   */
  async getLatency(): Promise<LatencyMetrics> {
    return this.api.query<LatencyMetrics>('get_latency');
  }

  /**
   * Get event history with pagination.
   *
   * @param limit - Maximum number of events to return (default 100, max 1000)
   * @param offset - Number of events to skip (default 0)
   * @returns Paginated events
   * @throws Error if the request fails
   */
  async getEvents(limit?: number, offset?: number): Promise<PaginatedEvents> {
    return this.api.query<PaginatedEvents>('get_events', { limit, offset });
  }

  /**
   * Clear event history.
   *
   * @returns Success status
   * @throws Error if the request fails
   */
  async clearEvents(): Promise<void> {
    return this.api.command<void>('clear_events');
  }

  // ========================================
  // Simulation Methods
  // ========================================

  /**
   * Run a simulation with the current configuration.
   *
   * @param input - Array of input events to simulate
   * @returns Simulation results
   * @throws Error if simulation fails
   */
  async simulate(input: SimulationInput[]): Promise<SimulationResult[]> {
    return this.api.command<SimulationResult[]>('simulate', { input });
  }

  /**
   * Reset the simulator state.
   *
   * @returns Success status
   * @throws Error if the request fails
   */
  async resetSimulator(): Promise<void> {
    return this.api.command<void>('reset_simulator');
  }

  // ========================================
  // Subscription Methods
  // ========================================

  /**
   * Subscribe to daemon state changes.
   *
   * @param handler - Function to call when state changes
   * @returns Unsubscribe function
   */
  onDaemonState(handler: (state: DaemonState) => void): () => void {
    return this.api.subscribe(
      'daemon-state',
      handler as (data: unknown) => void
    );
  }

  /**
   * Subscribe to key events.
   *
   * @param handler - Function to call for each key event
   * @returns Unsubscribe function
   */
  onKeyEvent(handler: (event: KeyEvent) => void): () => void {
    return this.api.subscribe('events', handler as (data: unknown) => void);
  }

  /**
   * Subscribe to latency metric updates.
   *
   * @param handler - Function to call when latency metrics are updated
   * @returns Unsubscribe function
   */
  onLatencyUpdate(handler: (metrics: LatencyMetrics) => void): () => void {
    return this.api.subscribe('latency', handler as (data: unknown) => void);
  }

  // ========================================
  // Daemon Control
  // ========================================

  /**
   * Restart the daemon process.
   *
   * This triggers a full process restart to apply configuration changes.
   * The WebSocket connection will be lost and the client should reconnect.
   *
   * @returns Success status with message
   * @throws Error if restart fails to initiate
   */
  async restartDaemon(): Promise<{ success: boolean; message: string }> {
    return this.api.command<{ success: boolean; message: string }>(
      'restart_daemon',
      {}
    );
  }

  /**
   * Activate a profile and restart the daemon to apply changes.
   *
   * This is a convenience method that activates a profile and then restarts
   * the daemon so the new configuration takes effect immediately.
   *
   * @param name - Name of the profile to activate
   * @returns Success status
   * @throws Error if activation or restart fails
   */
  async activateProfileAndRestart(name: string): Promise<void> {
    await this.activateProfile(name);
    // Brief delay to ensure activation is complete
    await new Promise((resolve) => setTimeout(resolve, 100));
    await this.restartDaemon();
  }

  // ========================================
  // Connection State
  // ========================================

  /**
   * Check if connected to the daemon.
   *
   * @returns True if WebSocket is connected and handshake completed
   */
  get isConnected(): boolean {
    return this.api.isConnected;
  }

  /**
   * Get the current WebSocket connection state.
   *
   * @returns ReadyState enum value
   */
  get readyState() {
    return this.api.readyState;
  }

  /**
   * Get the last error that occurred.
   *
   * @returns Error object or null if no error
   */
  get lastError(): Error | null {
    return this.api.lastError;
  }
}
