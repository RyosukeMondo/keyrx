/**
 * Device RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { Device } from '../types';

/**
 * Device RPC methods
 */
export class DeviceRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Get all devices.
   */
  async getDevices(): Promise<Device[]> {
    return this.api.query<Device[]>('get_devices');
  }

  /**
   * Rename a device.
   */
  async renameDevice(serial: string, newName: string): Promise<void> {
    return this.api.command<void>('rename_device', {
      serial,
      new_name: newName,
    });
  }

  /**
   * Set device scope.
   */
  async setScopeDevice(
    serial: string,
    scope: 'global' | 'profile'
  ): Promise<void> {
    return this.api.command<void>('set_scope_device', { serial, scope });
  }

  /**
   * Forget a device (remove from configuration).
   */
  async forgetDevice(serial: string): Promise<void> {
    return this.api.command<void>('forget_device', { serial });
  }

  /**
   * Set device keyboard layout preference.
   */
  async setDeviceLayout(serial: string, layout: string): Promise<void> {
    return this.api.command<void>('set_device_layout', { serial, layout });
  }
}