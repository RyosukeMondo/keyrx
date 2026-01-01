/**
 * StateIndicatorPanel - Visual display of daemon state
 *
 * This component displays the current daemon state including active modifiers,
 * locks, and the current layer. Will be fully implemented in Task 19.
 *
 * @placeholder This is a temporary stub for Task 18 integration
 */

import type { DaemonState } from '../types/rpc';

interface StateIndicatorPanelProps {
  state: DaemonState | null;
}

/**
 * StateIndicatorPanel component - Displays daemon state
 * @placeholder Full implementation in Task 19
 */
export function StateIndicatorPanel({ state }: StateIndicatorPanelProps) {
  if (!state) {
    return (
      <div className="bg-slate-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4">Daemon State</h2>
        <p className="text-slate-400">Loading...</p>
      </div>
    );
  }

  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold mb-4">Daemon State</h2>
      <div className="space-y-2 text-sm">
        <p>
          <span className="text-slate-400">Modifiers:</span>{' '}
          {state.modifiers.length > 0 ? state.modifiers.join(', ') : 'None'}
        </p>
        <p>
          <span className="text-slate-400">Locks:</span>{' '}
          {state.locks.length > 0 ? state.locks.join(', ') : 'None'}
        </p>
        <p>
          <span className="text-slate-400">Layer:</span> {state.layer}
        </p>
      </div>
    </div>
  );
}
