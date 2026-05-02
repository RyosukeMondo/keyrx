/**
 * Metrics RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { LatencyMetrics } from '../../types/rpc';
import type { PaginatedEvents } from '../types';

/**
 * Metrics RPC methods
 */
export class MetricsRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Get current latency statistics.
   */
  async getLatency(): Promise<LatencyMetrics> {
    return this.api.query<LatencyMetrics>('get_latency');
  }

  /**
   * Get event history with pagination.
   */
  async getEvents(limit?: number, offset?: number): Promise<PaginatedEvents> {
    return this.api.query<PaginatedEvents>('get_events', { limit, offset });
  }

  /**
   * Clear event history.
   */
  async clearEvents(): Promise<void> {
    return this.api.command<void>('clear_events');
  }
}