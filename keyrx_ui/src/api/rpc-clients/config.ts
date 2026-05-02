/**
 * Configuration RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { Config, Layer } from '../types';

/**
 * Configuration RPC methods
 */
export class ConfigRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Get current configuration.
   */
  async getConfig(): Promise<Config> {
    return this.api.query<Config>('get_config');
  }

  /**
   * Update configuration.
   */
  async updateConfig(code: string): Promise<void> {
    return this.api.command<void>('update_config', { code });
  }

  /**
   * Set a single key mapping.
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
   */
  async deleteKeyMapping(layer: string, keyCode: string): Promise<void> {
    return this.api.command<void>('delete_key_mapping', {
      layer,
      key_code: keyCode,
    });
  }

  /**
   * Get all layers.
   */
  async getLayers(): Promise<Layer[]> {
    return this.api.query<Layer[]>('get_layers');
  }
}