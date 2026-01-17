/**
 * SimulationControls - Control panel for simulation operations
 *
 * This component provides UI controls for simulation management including
 * start/stop/clear buttons and statistics display. It groups all simulation
 * controls in a dedicated, reusable component.
 *
 * Features:
 * - Start/Stop/Clear button controls
 * - Statistics display (event counts, rates)
 * - Proper disabled states
 * - Accessible buttons with ARIA labels
 * - Visual feedback for running state
 *
 * @example
 * ```tsx
 * <SimulationControls
 *   isRunning={isRunning}
 *   eventCount={events.length}
 *   onStart={start}
 *   onStop={stop}
 *   onClear={clearEvents}
 *   statistics={statistics}
 * />
 * ```
 */

import React from 'react';
import { Button } from '../Button';

/**
 * Statistics data for the simulation
 */
export interface SimulationStatistics {
  /** Total number of events */
  total: number;
  /** Number of press events */
  pressCount: number;
  /** Number of release events */
  releaseCount: number;
  /** Events per second */
  eventsPerSecond: number;
}

/**
 * Props for SimulationControls component
 */
export interface SimulationControlsProps {
  /** Whether simulation is currently running */
  isRunning: boolean;
  /** Total event count */
  eventCount: number;
  /** Callback when start button is clicked */
  onStart: () => void;
  /** Callback when stop button is clicked */
  onStop: () => void;
  /** Callback when clear button is clicked */
  onClear: () => void;
  /** Statistics to display */
  statistics: SimulationStatistics;
  /** Optional CSS classes */
  className?: string;
}

/**
 * SimulationControls component
 *
 * Renders control buttons and statistics for simulation management
 */
export const SimulationControls: React.FC<SimulationControlsProps> = ({
  isRunning,
  eventCount,
  onStart,
  onStop,
  onClear,
  statistics,
  className = '',
}) => {
  return (
    <div className={`flex flex-col gap-4 ${className}`}>
      {/* Button Controls */}
      <div className="flex flex-col sm:flex-row gap-2">
        <Button
          variant={isRunning ? 'danger' : 'primary'}
          size="md"
          onClick={isRunning ? onStop : onStart}
          aria-label={isRunning ? 'Stop simulation' : 'Start simulation'}
          className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
        >
          {isRunning ? 'Stop' : 'Start'}
        </Button>
        <Button
          variant="secondary"
          size="md"
          onClick={onClear}
          aria-label="Clear all events"
          disabled={eventCount === 0}
          className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
        >
          Clear Events
        </Button>
      </div>

      {/* Statistics Display */}
      {eventCount > 0 && (
        <div
          className="bg-slate-800 rounded-md p-3 border border-slate-700"
          role="region"
          aria-label="Simulation statistics"
        >
          <h3 className="text-sm font-medium text-slate-300 mb-2">
            Statistics
          </h3>
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-slate-400">Total Events:</span>
              <span className="text-slate-100 font-medium ml-2">
                {statistics.total}
              </span>
            </div>
            <div>
              <span className="text-slate-400">Events/sec:</span>
              <span className="text-slate-100 font-medium ml-2">
                {statistics.eventsPerSecond}
              </span>
            </div>
            <div>
              <span className="text-slate-400">Press:</span>
              <span className="text-green-400 font-medium ml-2">
                {statistics.pressCount}
              </span>
            </div>
            <div>
              <span className="text-slate-400">Release:</span>
              <span className="text-red-400 font-medium ml-2">
                {statistics.releaseCount}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Running Indicator */}
      {isRunning && (
        <div className="flex items-center gap-2 text-sm text-green-400">
          <span
            className="inline-block w-2 h-2 bg-green-400 rounded-full animate-pulse"
            aria-hidden="true"
          />
          <span>Simulation Running</span>
        </div>
      )}
    </div>
  );
};
