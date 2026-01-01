/**
 * DashboardEventTimeline - Virtualized real-time event log component.
 *
 * Displays a scrollable list of key events using react-window for performance
 * with large event lists. Supports pause/resume and clearing of events.
 */

import { useState } from "react";
import { FixedSizeList } from "react-window";
import type { KeyEvent } from "../types/rpc";
import { formatKeyCode } from "../utils/keyCodeMapping";
import { formatTimestampRelative } from "../utils/timeFormatting";

/**
 * Props for the DashboardEventTimeline component.
 */
export interface DashboardEventTimelineProps {
  /** Array of key events to display (newest first) */
  events: KeyEvent[];
  /** Whether event updates are paused */
  isPaused: boolean;
  /** Callback to toggle pause state */
  onTogglePause: () => void;
  /** Callback to clear all events */
  onClear: () => void;
}

/**
 * Props for individual event row.
 */
interface EventRowProps {
  /** Event data to display */
  event: KeyEvent;
  /** Row style from react-window */
  style: React.CSSProperties;
}

/**
 * Individual event row component with tooltip.
 */
function EventRow({ event, style }: EventRowProps) {
  const [showTooltip, setShowTooltip] = useState(false);

  // Convert timestamp from microseconds to milliseconds for relative time
  const timestampMs = Math.floor(event.timestamp / 1000);
  const relativeTime = formatTimestampRelative(timestampMs);
  const keyLabel = formatKeyCode(event.keyCode);

  return (
    <div
      style={style}
      className="relative flex items-center gap-4 px-4 py-2 border-b border-slate-700 hover:bg-slate-800 transition-colors"
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      {/* Key code */}
      <div className="w-20 font-mono text-sm text-blue-400 font-semibold">
        {keyLabel}
      </div>

      {/* Event type */}
      <div className="w-20 text-sm">
        <span
          className={`px-2 py-1 rounded text-xs font-medium ${
            event.eventType === "press"
              ? "bg-green-900 text-green-200"
              : "bg-red-900 text-red-200"
          }`}
        >
          {event.eventType}
        </span>
      </div>

      {/* Input -> Output */}
      <div className="flex-1 text-sm text-slate-400 font-mono">
        {formatKeyCode(event.input)} → {formatKeyCode(event.output)}
      </div>

      {/* Relative timestamp */}
      <div className="w-24 text-right text-xs text-slate-500">
        {relativeTime}
      </div>

      {/* Tooltip with full details */}
      {showTooltip && (
        <div className="absolute left-0 top-full z-50 mt-1 p-3 bg-slate-900 border border-slate-700 rounded shadow-lg text-xs font-mono whitespace-nowrap">
          <div className="grid grid-cols-2 gap-x-4 gap-y-1">
            <span className="text-slate-500">Timestamp:</span>
            <span className="text-slate-200">{event.timestamp}μs</span>

            <span className="text-slate-500">Input:</span>
            <span className="text-slate-200">{event.input}</span>

            <span className="text-slate-500">Output:</span>
            <span className="text-slate-200">{event.output}</span>

            <span className="text-slate-500">Latency:</span>
            <span className="text-slate-200">{event.latency}μs</span>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * DashboardEventTimeline component.
 *
 * Renders a virtualized list of key events with pause/resume and clear controls.
 * Uses react-window's FixedSizeList for efficient rendering of large event lists.
 *
 * @example
 * ```tsx
 * const [events, setEvents] = useState<KeyEvent[]>([]);
 * const [isPaused, setIsPaused] = useState(false);
 *
 * <DashboardEventTimeline
 *   events={events}
 *   isPaused={isPaused}
 *   onTogglePause={() => setIsPaused(!isPaused)}
 *   onClear={() => setEvents([])}
 * />
 * ```
 */
export function DashboardEventTimeline({
  events,
  isPaused,
  onTogglePause,
  onClear,
}: DashboardEventTimelineProps) {
  return (
    <div className="flex flex-col h-full">
      {/* Header with controls */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 p-4 border-b border-slate-700 bg-slate-900">
        <h2 className="text-lg font-semibold text-slate-200">Event Timeline</h2>
        <div className="flex flex-col sm:flex-row gap-2">
          <button
            onClick={onTogglePause}
            className="min-h-[44px] md:min-h-0 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded transition-colors text-sm font-medium"
            aria-label={isPaused ? "Resume event updates" : "Pause event updates"}
          >
            {isPaused ? "Resume" : "Pause"}
          </button>
          <button
            onClick={onClear}
            className="min-h-[44px] md:min-h-0 px-4 py-2 bg-red-900 hover:bg-red-800 text-red-200 rounded transition-colors text-sm font-medium"
            aria-label="Clear all events"
          >
            Clear
          </button>
        </div>
      </div>

      {/* Event list or empty state */}
      {events.length === 0 ? (
        <div className="flex items-center justify-center h-96 text-slate-500">
          No events yet. Start typing to see events appear.
        </div>
      ) : (
        <FixedSizeList
          height={400}
          itemCount={events.length}
          itemSize={50}
          width="100%"
          className="bg-slate-950"
        >
          {({ index, style }) => <EventRow event={events[index]} style={style} />}
        </FixedSizeList>
      )}
    </div>
  );
}
