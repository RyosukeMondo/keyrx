/**
 * Subscription RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../../types/rpc';

/**
 * Subscription RPC methods
 */
export class SubscriptionRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Subscribe to daemon state changes.
   */
  onDaemonState(handler: (state: DaemonState) => void): () => void {
    return this.api.subscribe(
      'daemon-state',
      handler as (data: unknown) => void
    );
  }

  /**
   * Subscribe to key events.
   */
  onKeyEvent(handler: (event: KeyEvent) => void): () => void {
    return this.api.subscribe('events', handler as (data: unknown) => void);
  }

  /**
   * Subscribe to latency metric updates.
   */
  onLatencyUpdate(handler: (metrics: LatencyMetrics) => void): () => void {
    return this.api.subscribe('latency', handler as (data: unknown) => void);
  }
}