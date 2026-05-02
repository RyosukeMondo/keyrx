/**
 * Type-safe RPC client for WebSocket communication with the daemon.
 *
 * This client wraps the useUnifiedApi hook and provides typed methods for all
 * RPC operations. Uses composition to delegate to specialized RPC method classes:
 * - profiles: Profile CRUD operations
 * - devices: Device management
 * - config: Configuration management
 * - metrics: Latency and event metrics
 * - simulation: Simulation operations
 * - subscriptions: Real-time event subscriptions
 * - daemon: Daemon control operations
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const api = useUnifiedApi();
 *   const client = new RpcClient(api);
 *
 *   // Direct methods (backward compatible)
 *   const profiles = await client.getProfiles();
 *   await client.activateProfile('Gaming');
 *
 *   // Or use composed sub-clients (preferred for new code)
 *   await client.profiles.getProfiles();
 *   await client.profiles.activateProfile('Gaming');
 *   client.subscriptions.onDaemonState((state) => { ... });
 * }
 * ```
 */

import type { RpcApi } from '../types/api';
import type { SimulationInput, SimulationResult } from './types';
import { ProfileRpcMethods } from './rpc-clients/profiles';
import { DeviceRpcMethods } from './rpc-clients/devices';
import { ConfigRpcMethods } from './rpc-clients/config';
import { MetricsRpcMethods } from './rpc-clients/metrics';
import { SimulationRpcMethods } from './rpc-clients/simulation';
import { SubscriptionRpcMethods } from './rpc-clients/subscriptions';
import { DaemonControlRpcMethods } from './rpc-clients/daemon';

/**
 * Type-safe RPC client for daemon communication.
 */
export class RpcClient {
  readonly profiles: ProfileRpcMethods;
  readonly devices: DeviceRpcMethods;
  readonly config: ConfigRpcMethods;
  readonly metrics: MetricsRpcMethods;
  readonly simulation: SimulationRpcMethods;
  readonly subscriptions: SubscriptionRpcMethods;
  readonly daemon: DaemonControlRpcMethods;

  private api: RpcApi;

  /**
   * Create a new RPC client.
   *
   * @param api - The RPC API implementation (e.g., from useUnifiedApi hook)
   */
  constructor(api: RpcApi) {
    this.api = api;

    // Initialize composed sub-clients
    this.profiles = new ProfileRpcMethods(api);
    this.devices = new DeviceRpcMethods(api);
    this.config = new ConfigRpcMethods(api);
    this.metrics = new MetricsRpcMethods(api);
    this.simulation = new SimulationRpcMethods(api);
    this.subscriptions = new SubscriptionRpcMethods(api);
    this.daemon = new DaemonControlRpcMethods(api);
  }

  // ========================================
  // Profile Methods (delegated to profiles sub-client)
  // ========================================

  async getProfiles() {
    return this.profiles.getProfiles();
  }

  async createProfile(name: string, template?: string) {
    return this.profiles.createProfile(name, template);
  }

  async activateProfile(name: string) {
    return this.profiles.activateProfile(name);
  }

  async deleteProfile(name: string) {
    return this.profiles.deleteProfile(name);
  }

  async duplicateProfile(sourceName: string, newName: string) {
    return this.profiles.duplicateProfile(sourceName, newName);
  }

  async renameProfile(oldName: string, newName: string) {
    return this.profiles.renameProfile(oldName, newName);
  }

  async getProfileConfig(name: string) {
    return this.profiles.getProfileConfig(name);
  }

  async setProfileConfig(name: string, source: string) {
    return this.profiles.setProfileConfig(name, source);
  }

  async getActiveProfile() {
    return this.profiles.getActiveProfile();
  }

  // ========================================
  // Device Methods (delegated to devices sub-client)
  // ========================================

  async getDevices() {
    return this.devices.getDevices();
  }

  async renameDevice(serial: string, newName: string) {
    return this.devices.renameDevice(serial, newName);
  }

  async setScopeDevice(serial: string, scope: 'global' | 'profile') {
    return this.devices.setScopeDevice(serial, scope);
  }

  async forgetDevice(serial: string) {
    return this.devices.forgetDevice(serial);
  }

  async setDeviceLayout(serial: string, layout: string) {
    return this.devices.setDeviceLayout(serial, layout);
  }

  // ========================================
  // Configuration Methods (delegated to config sub-client)
  // ========================================

  async getConfig() {
    return this.config.getConfig();
  }

  async updateConfig(code: string) {
    return this.config.updateConfig(code);
  }

  async setKeyMapping(layer: string, keyCode: string, mapping: string) {
    return this.config.setKeyMapping(layer, keyCode, mapping);
  }

  async deleteKeyMapping(layer: string, keyCode: string) {
    return this.config.deleteKeyMapping(layer, keyCode);
  }

  async getLayers() {
    return this.config.getLayers();
  }

  // ========================================
  // Metrics Methods (delegated to metrics sub-client)
  // ========================================

  async getLatency() {
    return this.metrics.getLatency();
  }

  async getEvents(limit?: number, offset?: number) {
    return this.metrics.getEvents(limit, offset);
  }

  async clearEvents() {
    return this.metrics.clearEvents();
  }

  // ========================================
  // Simulation Methods (delegated to simulation sub-client)
  // ========================================

  async simulate(input: SimulationInput[]): Promise<SimulationResult[]> {
    return this.simulation.simulate(input);
  }

  async resetSimulator() {
    return this.simulation.resetSimulator();
  }

  // ========================================
  // Subscription Methods (delegated to subscriptions sub-client)
  // ========================================

  onDaemonState(handler: (state: import('../types/rpc').DaemonState) => void) {
    return this.subscriptions.onDaemonState(handler);
  }

  onKeyEvent(handler: (event: import('../types/rpc').KeyEvent) => void) {
    return this.subscriptions.onKeyEvent(handler);
  }

  onLatencyUpdate(
    handler: (metrics: import('../types/rpc').LatencyMetrics) => void
  ) {
    return this.subscriptions.onLatencyUpdate(handler);
  }

  // ========================================
  // Daemon Control Methods (delegated to daemon sub-client)
  // ========================================

  async restartDaemon() {
    return this.daemon.restartDaemon();
  }

  async activateProfileAndRestart(name: string) {
    return this.daemon.activateProfileAndRestart(name);
  }

  // ========================================
  // Connection State (delegated to api)
  // ========================================

  get isConnected(): boolean {
    return this.api.isConnected;
  }

  get readyState() {
    return this.api.readyState;
  }

  get lastError(): Error | null {
    return this.api.lastError;
  }
}