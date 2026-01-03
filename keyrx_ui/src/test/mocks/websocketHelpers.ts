/**
 * WebSocket Test Helper Functions
 *
 * Convenient utility functions for simulating daemon events in tests.
 * These helpers provide type-safe, well-documented APIs for common test scenarios.
 *
 * @example
 * ```typescript
 * import { setDaemonState, sendLatencyUpdate } from '@/test/mocks/websocketHelpers';
 *
 * test('component updates on daemon state change', async () => {
 *   const { getByText } = renderWithProviders(<MyComponent />);
 *
 *   // Simulate daemon state change
 *   setDaemonState({ activeProfile: 'gaming', layer: 'fn' });
 *
 *   // Assert component updated
 *   await waitFor(() => {
 *     expect(getByText('gaming')).toBeInTheDocument();
 *   });
 * });
 * ```
 */

import { broadcastEvent } from './websocketHandlers';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../../types/rpc';

/**
 * Set daemon state and broadcast to all subscribed connections.
 *
 * Simulates a daemon state change event (e.g., profile activation, layer change,
 * modifier/lock state update). All connections subscribed to the "daemon-state"
 * channel will receive this update.
 *
 * @param state - Partial daemon state to broadcast (only provide fields that changed)
 *
 * @example
 * ```typescript
 * // Simulate profile activation
 * setDaemonState({ activeProfile: 'gaming' });
 *
 * // Simulate layer change
 * setDaemonState({ layer: 'fn' });
 *
 * // Simulate modifier press
 * setDaemonState({
 *   modifiers: ['MD_00', 'MD_01'],
 *   layer: 'shift'
 * });
 *
 * // Simulate lock toggle
 * setDaemonState({
 *   locks: ['LK_00'],
 *   layer: 'caps'
 * });
 * ```
 */
export function setDaemonState(state: Partial<DaemonState> & { activeProfile?: string }): void {
  // Merge with default state to ensure all required fields are present
  const fullState: DaemonState & { activeProfile?: string } = {
    modifiers: state.modifiers ?? [],
    locks: state.locks ?? [],
    layer: state.layer ?? 'base',
    ...(state.activeProfile !== undefined && { activeProfile: state.activeProfile }),
  };

  broadcastEvent('daemon-state', fullState);
}

/**
 * Send latency statistics update to all subscribed connections.
 *
 * Simulates a latency metrics broadcast event. The daemon sends these
 * periodically (every 1 second) on the "latency" channel.
 *
 * @param stats - Latency statistics (all values in microseconds)
 *
 * @example
 * ```typescript
 * // Simulate good performance (sub-millisecond latency)
 * sendLatencyUpdate({
 *   min: 200,
 *   avg: 500,
 *   max: 1000,
 *   p95: 800,
 *   p99: 950
 * });
 *
 * // Simulate degraded performance
 * sendLatencyUpdate({
 *   min: 5000,
 *   avg: 15000,
 *   max: 50000,
 *   p95: 30000,
 *   p99: 45000
 * });
 *
 * // Simulate latency spike
 * sendLatencyUpdate({
 *   min: 500,
 *   avg: 2000,
 *   max: 100000, // 100ms spike
 *   p95: 5000,
 *   p99: 50000
 * });
 * ```
 */
export function sendLatencyUpdate(stats: Omit<LatencyMetrics, 'timestamp'>): void {
  const metrics: LatencyMetrics = {
    ...stats,
    timestamp: Date.now() * 1000, // Convert to microseconds
  };

  broadcastEvent('latency', metrics);
}

/**
 * Send key event to all subscribed connections.
 *
 * Simulates a key press/release event broadcast. The daemon sends these
 * on the "events" channel for each key event processed.
 *
 * @param event - Key event data
 *
 * @example
 * ```typescript
 * // Simulate key press (A key)
 * sendKeyEvent({
 *   keyCode: 'KEY_A',
 *   eventType: 'press',
 *   input: 'KEY_A',
 *   output: 'KEY_B', // Remapped to B
 *   latency: 500
 * });
 *
 * // Simulate key release
 * sendKeyEvent({
 *   keyCode: 'KEY_A',
 *   eventType: 'release',
 *   input: 'KEY_A',
 *   output: 'KEY_B',
 *   latency: 300
 * });
 *
 * // Simulate tap-hold (A tap → B, A hold → Ctrl)
 * sendKeyEvent({
 *   keyCode: 'KEY_A',
 *   eventType: 'press',
 *   input: 'KEY_A',
 *   output: 'KEY_LEFTCTRL', // Hold threshold exceeded
 *   latency: 200
 * });
 * ```
 */
export function sendKeyEvent(event: Omit<KeyEvent, 'timestamp'>): void {
  const fullEvent: KeyEvent = {
    ...event,
    timestamp: Date.now() * 1000, // Convert to microseconds
  };

  broadcastEvent('events', fullEvent);
}

/**
 * Simulate WebSocket disconnect scenario.
 *
 * Currently not fully implemented as MSW WebSocket lifecycle is managed
 * automatically by the test framework. Use this in integration tests to
 * verify offline/reconnection behavior.
 *
 * TODO: Implement disconnect simulation once MSW WebSocket supports
 * programmatic connection control.
 *
 * @example
 * ```typescript
 * test('handles disconnection gracefully', async () => {
 *   const { getByText } = renderWithProviders(<MyComponent />);
 *
 *   // Simulate disconnect
 *   simulateDisconnect();
 *
 *   // Verify UI shows offline state
 *   await waitFor(() => {
 *     expect(getByText('Offline')).toBeInTheDocument();
 *   });
 * });
 * ```
 */
export function simulateDisconnect(): void {
  // TODO: Implement disconnect simulation
  // MSW WebSocket doesn't currently provide a way to programmatically
  // close connections from the server side in tests.
  // For now, tests should use the WebSocket instance directly if needed.
  console.warn(
    '[WebSocket Test Helpers] simulateDisconnect() is not yet implemented. ' +
      'MSW WebSocket lifecycle is managed automatically by the test framework.'
  );
}

/**
 * Wait for WebSocket connection to establish.
 *
 * Returns a promise that resolves when a WebSocket connection is established.
 * Useful for ensuring WebSocket is ready before sending events in tests.
 *
 * @param timeout - Maximum time to wait in milliseconds (default: 5000ms)
 * @returns Promise that resolves when connected or rejects on timeout
 *
 * @example
 * ```typescript
 * test('waits for WebSocket before testing', async () => {
 *   const { getByText } = renderWithProviders(<MyComponent />);
 *
 *   // Wait for WebSocket connection
 *   await waitForWebSocketConnection();
 *
 *   // Now safe to send events
 *   setDaemonState({ activeProfile: 'gaming' });
 *
 *   await waitFor(() => {
 *     expect(getByText('gaming')).toBeInTheDocument();
 *   });
 * });
 * ```
 */
export function waitForWebSocketConnection(timeout = 5000): Promise<void> {
  return new Promise((resolve, reject) => {
    const startTime = Date.now();

    const checkConnection = () => {
      // In MSW, connections are established synchronously when WebSocket is created
      // This function exists primarily for API compatibility and can be used
      // in integration tests that need to wait for async connection setup
      if (Date.now() - startTime > timeout) {
        reject(new Error(`WebSocket connection timeout after ${timeout}ms`));
        return;
      }

      // MSW WebSocket connections are established immediately
      // No need to poll - resolve immediately
      resolve();
    };

    checkConnection();
  });
}

/**
 * Send a custom server message to all connections.
 *
 * Low-level utility for sending arbitrary messages on any channel.
 * Use the higher-level helpers (setDaemonState, sendLatencyUpdate, sendKeyEvent)
 * for common scenarios.
 *
 * @param channel - Subscription channel to broadcast on
 * @param data - Message data (must match channel's expected type)
 *
 * @example
 * ```typescript
 * // Send custom daemon state
 * sendServerMessage('daemon-state', {
 *   modifiers: ['MD_CUSTOM'],
 *   locks: [],
 *   layer: 'custom-layer'
 * });
 *
 * // Send custom latency metrics
 * sendServerMessage('latency', {
 *   min: 100,
 *   avg: 200,
 *   max: 500,
 *   p95: 400,
 *   p99: 480,
 *   timestamp: Date.now() * 1000
 * });
 * ```
 */
export function sendServerMessage(channel: 'daemon-state' | 'events' | 'latency', data: unknown): void {
  broadcastEvent(channel, data);
}
