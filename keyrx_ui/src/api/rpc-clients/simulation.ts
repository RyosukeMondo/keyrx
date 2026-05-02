/**
 * Simulation RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { SimulationInput, SimulationResult } from '../types';

/**
 * Simulation RPC methods
 */
export class SimulationRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Run a simulation with the current configuration.
   */
  async simulate(input: SimulationInput[]): Promise<SimulationResult[]> {
    return this.api.command<SimulationResult[]>('simulate', { input });
  }

  /**
   * Reset the simulator state.
   */
  async resetSimulator(): Promise<void> {
    return this.api.command<void>('reset_simulator');
  }
}