/**
 * Daemon control RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';

/**
 * Daemon control RPC methods
 */
export class DaemonControlRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Restart the daemon process.
   */
  async restartDaemon(): Promise<{ success: boolean; message: string }> {
    return this.api.command<{ success: boolean; message: string }>(
      'restart_daemon',
      {}
    );
  }

  /**
   * Activate a profile and restart the daemon to apply changes.
   */
  async activateProfileAndRestart(name: string): Promise<void> {
    await this.api.command<void>('activate_profile', { name });
    await new Promise((resolve) => setTimeout(resolve, 100));
    await this.restartDaemon();
  }
}