/**
 * DashboardEventTimeline - Real-time event log
 *
 * This component displays a virtualized list of key events with pause/resume
 * and clear functionality. Will be fully implemented in Task 21.
 *
 * @placeholder This is a temporary stub for Task 18 integration
 */

import type { KeyEvent } from '../types/rpc';

interface DashboardEventTimelineProps {
  events: KeyEvent[];
  isPaused: boolean;
  onTogglePause: () => void;
  onClear: () => void;
}

/**
 * DashboardEventTimeline component - Displays real-time event stream
 * @placeholder Full implementation in Task 21
 */
export function DashboardEventTimeline({
  events,
  isPaused,
  onTogglePause,
  onClear,
}: DashboardEventTimelineProps) {
  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 mb-4">
        <h2 className="text-lg font-semibold">Event Timeline</h2>
        <div className="flex flex-col sm:flex-row gap-2">
          <button
            onClick={onTogglePause}
            className="min-h-[44px] md:min-h-0 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white text-sm"
          >
            {isPaused ? 'Resume' : 'Pause'}
          </button>
          <button
            onClick={onClear}
            className="min-h-[44px] md:min-h-0 px-4 py-2 bg-red-600 hover:bg-red-700 rounded text-white text-sm"
          >
            Clear
          </button>
        </div>
      </div>

      <div className="text-sm text-slate-400">
        {events.length === 0 ? (
          <p>No events yet...</p>
        ) : (
          <>
            <p className="mb-2">
              {events.length} event{events.length !== 1 ? 's' : ''}{' '}
              {isPaused && '(Paused)'}
            </p>
            <div className="max-h-96 overflow-y-auto space-y-1">
              {events.slice(0, 10).map((event, index) => (
                <div
                  key={`${event.timestamp}-${index}`}
                  className="p-2 bg-slate-700 rounded"
                >
                  <span className="font-mono">{event.keyCode}</span>
                  <span className="mx-2 text-slate-500">→</span>
                  <span className="text-slate-300">{event.eventType}</span>
                  <span className="mx-2 text-slate-500">|</span>
                  <span className="text-slate-400">
                    {(event.latency / 1000).toFixed(2)} ms
                  </span>
                </div>
              ))}
              {events.length > 10 && (
                <p className="text-slate-500 text-center pt-2">
                  ...and {events.length - 10} more events
                </p>
              )}
            </div>
            <p className="text-slate-500 mt-4">
              Virtualized list will be implemented in Task 21
            </p>
          </>
        )}
      </div>
    </div>
  );
}
