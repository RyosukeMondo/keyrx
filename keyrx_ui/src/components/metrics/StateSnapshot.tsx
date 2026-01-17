import React from 'react';

/**
 * Props for the StateSnapshot component
 */
export interface StateSnapshotProps {
  /** Current daemon state snapshot */
  state: {
    /** Currently active layer name */
    activeLayer: string;
    /** List of active modifier keys */
    modifiers: string[];
    /** List of active lock states */
    locks: string[];
    /** Number of pending tap/hold timers */
    tapHoldTimers: number;
    /** Number of events in the queue */
    queuedEvents: number;
  };
}

/**
 * Displays current daemon internal state snapshot
 *
 * Renders a grid showing:
 * - Active layer (blue)
 * - Tap/Hold timers count (yellow)
 * - Active modifiers as comma-separated list (green)
 * - Active locks as comma-separated list (purple)
 * - Queued events count (red)
 *
 * @example
 * ```tsx
 * <StateSnapshot
 *   state={{
 *     activeLayer: 'Base',
 *     modifiers: ['Ctrl', 'Shift'],
 *     locks: ['CapsLock'],
 *     tapHoldTimers: 1,
 *     queuedEvents: 0
 *   }}
 * />
 * ```
 */
export const StateSnapshot: React.FC<StateSnapshotProps> = ({ state }) => {
  return (
    <div
      className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 md:gap-4"
      role="group"
      aria-label="Daemon state information"
    >
      {/* Active Layer */}
      <div
        className="bg-slate-800 rounded-lg p-4"
        role="status"
        aria-label={`Active layer: ${state.activeLayer}`}
      >
        <h3 className="text-sm font-semibold text-slate-300 mb-2">
          Active Layer
        </h3>
        <p className="text-lg font-mono text-blue-400">{state.activeLayer}</p>
      </div>

      {/* Tap/Hold Timers */}
      <div
        className="bg-slate-800 rounded-lg p-4"
        role="status"
        aria-label={`${state.tapHoldTimers} active tap/hold timers`}
      >
        <h3 className="text-sm font-semibold text-slate-300 mb-2">
          Tap/Hold Timers
        </h3>
        <p className="text-lg font-mono text-yellow-400">
          {state.tapHoldTimers} active
        </p>
      </div>

      {/* Active Modifiers */}
      <div
        className="bg-slate-800 rounded-lg p-4"
        role="status"
        aria-label={`Active modifiers: ${
          state.modifiers.length > 0 ? state.modifiers.join(', ') : 'None'
        }`}
      >
        <h3 className="text-sm font-semibold text-slate-300 mb-2">
          Active Modifiers
        </h3>
        <p className="text-lg font-mono text-green-400">
          {state.modifiers.length > 0 ? state.modifiers.join(', ') : 'None'}
        </p>
      </div>

      {/* Active Locks */}
      <div
        className="bg-slate-800 rounded-lg p-4"
        role="status"
        aria-label={`Active locks: ${
          state.locks.length > 0 ? state.locks.join(', ') : 'None'
        }`}
      >
        <h3 className="text-sm font-semibold text-slate-300 mb-2">
          Active Locks
        </h3>
        <p className="text-lg font-mono text-purple-400">
          {state.locks.length > 0 ? state.locks.join(', ') : 'None'}
        </p>
      </div>

      {/* Queued Events */}
      <div
        className="bg-slate-800 rounded-lg p-4"
        role="status"
        aria-label={`${state.queuedEvents} queued events`}
      >
        <h3 className="text-sm font-semibold text-slate-300 mb-2">
          Queued Events
        </h3>
        <p className="text-lg font-mono text-red-400">{state.queuedEvents}</p>
      </div>
    </div>
  );
};
