/**
 * StateIndicatorPanel - Visual display of daemon state
 *
 * Displays the current daemon state including active modifiers, locks, and layer
 * with color-coded badges (blue for modifiers, orange for locks, green for layer).
 */

import React from 'react';
import type { DaemonState } from '../types/rpc';

interface StateIndicatorPanelProps {
  state: DaemonState | null;
}

/**
 * StateIndicatorPanel component - Displays daemon state with color-coded badges
 *
 * @param state - Current daemon state (null while loading)
 * @returns Visual panel with modifiers (blue), locks (orange), and layer (green)
 */
export function StateIndicatorPanel({ state }: StateIndicatorPanelProps) {
  if (!state) {
    return (
      <div className="p-4 bg-slate-800 rounded-lg">
        <p className="text-slate-400 text-sm">Loading daemon state...</p>
      </div>
    );
  }

  return (
    <div className="p-4 bg-slate-800 rounded-lg">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Modifiers Section */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2" aria-label="Active Modifiers">
            Modifiers
          </h3>
          <div className="flex flex-wrap gap-2">
            {state.modifiers && state.modifiers.length > 0 ? (
              state.modifiers.map((modId) => (
                <span
                  key={`mod-${modId}`}
                  className="px-3 py-1 bg-blue-600 text-white text-sm rounded-full font-medium"
                  aria-label={`Modifier ${modId} active`}
                >
                  MOD_{modId}
                </span>
              ))
            ) : (
              <span className="text-slate-500 text-sm" aria-label="No modifiers active">
                None
              </span>
            )}
          </div>
        </div>

        {/* Locks Section */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2" aria-label="Active Locks">
            Locks
          </h3>
          <div className="flex flex-wrap gap-2">
            {state.locks && state.locks.length > 0 ? (
              state.locks.map((lockId) => (
                <span
                  key={`lock-${lockId}`}
                  className="px-3 py-1 bg-orange-600 text-white text-sm rounded-full font-medium"
                  aria-label={`Lock ${lockId} active`}
                >
                  LOCK_{lockId}
                </span>
              ))
            ) : (
              <span className="text-slate-500 text-sm" aria-label="No locks active">
                None
              </span>
            )}
          </div>
        </div>

        {/* Layer Section */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2" aria-label="Current Layer">
            Layer
          </h3>
          <div className="flex flex-wrap gap-2">
            <span
              className="px-3 py-1 bg-green-600 text-white text-sm rounded-full font-medium"
              aria-label={`Layer ${state.layer} active`}
            >
              Layer {state.layer}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
