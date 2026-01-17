/**
 * StateIndicatorPanel - Visual display of daemon state
 *
 * Displays the current daemon state including active modifiers, locks, layer,
 * and connected devices with virtual/physical indicators.
 * Color-coded badges: blue for modifiers, orange for locks, green for layer,
 * purple for virtual devices, gray for physical devices.
 */

import React from 'react';
import type { DaemonState } from '../types/rpc';
import { useDevices } from '../hooks/useDevices';

interface StateIndicatorPanelProps {
  state: DaemonState | null;
}

/**
 * StateIndicatorPanel component - Displays daemon state with color-coded badges
 *
 * @param state - Current daemon state (null while loading)
 * @returns Visual panel with modifiers (blue), locks (orange), layer (green), and devices (purple/gray)
 */
export function StateIndicatorPanel({ state }: StateIndicatorPanelProps) {
  const { data: devices, isLoading: devicesLoading } = useDevices();

  if (!state) {
    return (
      <div className="p-4 bg-slate-800 rounded-lg">
        <p className="text-slate-400 text-sm">Loading daemon state...</p>
      </div>
    );
  }

  return (
    <div
      className="p-4 bg-slate-800 rounded-lg"
      data-testid="state-indicator-panel"
    >
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Modifiers Section */}
        <div>
          <h3
            className="text-sm font-medium text-slate-300 mb-2"
            aria-label="Active Modifiers"
          >
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
              <span
                className="text-slate-500 text-sm"
                aria-label="No modifiers active"
              >
                None
              </span>
            )}
          </div>
        </div>

        {/* Locks Section */}
        <div>
          <h3
            className="text-sm font-medium text-slate-300 mb-2"
            aria-label="Active Locks"
          >
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
              <span
                className="text-slate-500 text-sm"
                aria-label="No locks active"
              >
                None
              </span>
            )}
          </div>
        </div>

        {/* Layer Section */}
        <div>
          <h3
            className="text-sm font-medium text-slate-300 mb-2"
            aria-label="Current Layer"
          >
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

        {/* Devices Section */}
        <div>
          <h3
            className="text-sm font-medium text-slate-300 mb-2"
            aria-label="Connected Devices"
          >
            Devices
          </h3>
          <div className="flex flex-wrap gap-2">
            {devicesLoading ? (
              <span
                className="text-slate-500 text-sm"
                aria-label="Loading devices"
              >
                Loading...
              </span>
            ) : devices && devices.length > 0 ? (
              devices
                .filter((device) => device.active)
                .map((device) => (
                  <div
                    key={device.id}
                    className="flex items-center gap-1.5 px-3 py-1 bg-slate-700 text-white text-sm rounded-full font-medium"
                    aria-label={`Device ${device.name} - ${
                      device.isVirtual ? 'Virtual' : 'Hardware'
                    }`}
                  >
                    {device.isVirtual ? (
                      <svg
                        className="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                        />
                      </svg>
                    ) : (
                      <svg
                        className="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
                        />
                      </svg>
                    )}
                    <span
                      className={
                        device.isVirtual ? 'text-purple-300' : 'text-slate-300'
                      }
                    >
                      {device.name}
                    </span>
                    <span
                      className={`px-2 py-0.5 text-xs rounded ${
                        device.isVirtual
                          ? 'bg-purple-600 text-white'
                          : 'bg-slate-600 text-slate-200'
                      }`}
                    >
                      {device.isVirtual ? 'Virtual' : 'Hardware'}
                    </span>
                  </div>
                ))
            ) : (
              <span
                className="text-slate-500 text-sm"
                aria-label="No devices connected"
              >
                None
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
