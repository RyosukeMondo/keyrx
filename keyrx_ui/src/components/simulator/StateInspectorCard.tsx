import React from 'react';
import { Card } from '../Card';
import { StateIndicatorPanel } from '../StateIndicatorPanel';
import type { DaemonState, SimulatorState } from '@/types/rpc';

interface StateInspectorCardProps {
  isUsingProfileConfig: boolean;
  wasmState: DaemonState | null;
  state: SimulatorState;
}

/**
 * Card component for displaying simulator state (layers, modifiers, locks)
 */
export const StateInspectorCard: React.FC<StateInspectorCardProps> = ({
  isUsingProfileConfig,
  wasmState,
  state,
}) => {
  return (
    <Card className="lg:col-span-1" aria-labelledby="simulator-state-heading">
      <h2
        id="simulator-state-heading"
        className="text-base md:text-lg font-semibold text-slate-100 mb-3"
      >
        State Inspector
      </h2>
      {isUsingProfileConfig && wasmState ? (
        <StateIndicatorPanel state={wasmState} />
      ) : (
        <div className="space-y-3 md:space-y-4">
          <div>
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-slate-400">Active Layer:</span>
                <span className="text-sm font-mono text-slate-100">
                  {state.activeLayer}
                </span>
              </div>
            </div>
          </div>

          <div>
            <h3 className="text-sm font-medium text-slate-300 mb-2">
              Modifiers
            </h3>
            <div className="grid grid-cols-2 gap-2">
              {Object.entries(state.modifiers).map(([key, active]) => (
                <div
                  key={key}
                  className={`px-3 py-2 rounded text-xs font-mono text-center transition-colors ${
                    active
                      ? 'bg-green-500 text-white'
                      : 'bg-slate-700 text-slate-400'
                  }`}
                >
                  {key.charAt(0).toUpperCase() + key.slice(1)}{' '}
                  {active ? '✓' : ''}
                </div>
              ))}
            </div>
          </div>

          <div>
            <h3 className="text-sm font-medium text-slate-300 mb-2">Locks</h3>
            <div className="grid grid-cols-1 gap-2">
              {Object.entries(state.locks).map(([key, active]) => (
                <div
                  key={key}
                  className={`px-3 py-2 rounded text-xs font-mono text-center transition-colors ${
                    active
                      ? 'bg-blue-500 text-white'
                      : 'bg-slate-700 text-slate-400'
                  }`}
                >
                  {key
                    .replace(/([A-Z])/g, ' $1')
                    .trim()
                    .split(' ')
                    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
                    .join(' ')}{' '}
                  {active ? '✓' : ''}
                </div>
              ))}
            </div>
          </div>
          {!isUsingProfileConfig && (
            <p className="text-xs text-slate-500 mt-2">
              Using mock state. Select a valid profile to see WASM state.
            </p>
          )}
        </div>
      )}
    </Card>
  );
};
