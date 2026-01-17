/**
 * EventList - Virtualized event list component
 *
 * Displays a list of key events with virtualization for performance when
 * rendering large numbers of events (1000+). Uses react-window for efficient
 * rendering and includes auto-scroll to latest event.
 *
 * @example
 * ```tsx
 * <EventList
 *   events={events}
 *   maxEvents={1000}
 *   onClear={handleClear}
 *   virtualizeThreshold={100}
 * />
 * ```
 */

import React, { useEffect, useRef, useMemo } from 'react';
import { FixedSizeList as List } from 'react-window';
import type { KeyEvent } from '@/types/rpc';
import { Button } from '../Button';

/**
 * Props for EventList component
 */
export interface EventListProps {
  /** Array of key events to display (newest first) */
  events: KeyEvent[];
  /** Maximum number of events that can be stored */
  maxEvents: number;
  /** Callback to clear all events */
  onClear: () => void;
  /** Threshold for enabling virtualization (default: 100) */
  virtualizeThreshold?: number;
}

/**
 * Format timestamp from microseconds to HH:MM:SS
 */
function formatTimestamp(timestampMicros: number): string {
  const date = new Date(timestampMicros / 1000);
  return date.toLocaleTimeString('en-US', { hour12: false });
}

/**
 * Get color class for event type
 */
function getEventTypeColor(eventType: 'press' | 'release'): string {
  return eventType === 'press' ? 'text-green-400' : 'text-red-400';
}

/**
 * Format event message
 */
function formatEventMessage(event: KeyEvent): string {
  return `${event.input} → ${event.output} (${event.latency}μs)`;
}

/**
 * Individual event row component
 */
interface EventRowProps {
  event: KeyEvent;
  index: number;
}

const EventRow: React.FC<EventRowProps> = ({ event, index }) => {
  return (
    <div
      className="flex items-start gap-3 text-xs"
      role="row"
      aria-rowindex={index + 1}
    >
      <span className="text-slate-500 shrink-0 w-20" role="cell">
        {formatTimestamp(event.timestamp)}
      </span>
      <span
        className={`shrink-0 w-16 font-medium ${getEventTypeColor(
          event.eventType
        )}`}
        role="cell"
      >
        {event.eventType.toUpperCase()}
      </span>
      <span className="text-slate-300 flex-1" role="cell">
        {formatEventMessage(event)}
      </span>
    </div>
  );
};

/**
 * EventList component with virtualization
 *
 * Displays a list of key events with optional virtualization for performance.
 * Automatically switches to virtualized rendering when event count exceeds
 * the threshold.
 */
export const EventList: React.FC<EventListProps> = ({
  events,
  maxEvents,
  onClear,
  virtualizeThreshold = 100,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<List>(null);

  // Determine if virtualization should be enabled
  const shouldVirtualize = events.length > virtualizeThreshold;

  // Auto-scroll to top when new events arrive
  useEffect(() => {
    if (!shouldVirtualize && containerRef.current) {
      // Non-virtualized: scroll container to top
      containerRef.current.scrollTop = 0;
    } else if (shouldVirtualize && listRef.current) {
      // Virtualized: scroll list to top (index 0)
      listRef.current.scrollToItem(0, 'start');
    }
  }, [events.length, shouldVirtualize]);

  // Memoize row renderer for react-window
  const Row = useMemo(
    () =>
      ({ index, style }: { index: number; style: React.CSSProperties }) => {
        const event = events[index];
        return (
          <div style={style}>
            <EventRow event={event} index={index} />
          </div>
        );
      },
    [events]
  );

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-base md:text-lg font-semibold text-slate-100">
          Event Log
          <span className="text-xs md:text-sm font-normal text-slate-400 ml-2">
            ({Math.min(events.length, maxEvents)} / {maxEvents} events)
          </span>
        </h2>
        <Button
          variant="secondary"
          size="sm"
          onClick={onClear}
          disabled={events.length === 0}
          aria-label="Clear event log"
        >
          Clear
        </Button>
      </div>

      {/* Event list container */}
      <div
        className="bg-slate-900 rounded-md p-3 md:p-4 flex-1 min-h-0"
        role="region"
        aria-label="Key event log"
      >
        {events.length === 0 ? (
          // Empty state
          <div
            className="flex items-center justify-center h-full text-slate-500 text-sm"
            role="status"
          >
            No events yet. Press a key to start.
          </div>
        ) : shouldVirtualize ? (
          // Virtualized list (for many events)
          <List
            ref={listRef}
            height={400}
            itemCount={events.length}
            itemSize={32}
            width="100%"
            className="font-mono"
          >
            {Row}
          </List>
        ) : (
          // Non-virtualized list (for few events)
          <div
            ref={containerRef}
            className="h-full overflow-y-auto font-mono space-y-1"
          >
            {events.map((event, index) => (
              <EventRow
                key={`${event.timestamp}-${index}`}
                event={event}
                index={index}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
