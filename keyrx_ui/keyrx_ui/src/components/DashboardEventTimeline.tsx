/**
 * DashboardEventTimeline Component
 *
 * Virtualized event timeline displaying the last 100 key events from the daemon.
 * Features pause/resume functionality and tooltips on hover.
 */

import React, { useState, useCallback, useMemo } from 'react';
import { FixedSizeList as List } from 'react-window';
import { useDashboardStore, KeyEvent } from '../store/dashboardStore';
import './DashboardEventTimeline.css';

/**
 * Format microseconds to human-readable time
 */
const formatTime = (microseconds: number): string => {
  const date = new Date(microseconds / 1000);
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  const seconds = date.getSeconds().toString().padStart(2, '0');
  const ms = date.getMilliseconds().toString().padStart(3, '0');
  return `${hours}:${minutes}:${seconds}.${ms}`;
};

/**
 * Format latency in microseconds to milliseconds with 2 decimal places
 */
const formatLatency = (microseconds: number): string => {
  const ms = microseconds / 1000;
  return `${ms.toFixed(2)}ms`;
};

/**
 * DashboardEventTimeline component props
 */
interface DashboardEventTimelineProps {
  /** Height of the timeline container in pixels */
  height?: number;
  /** Width of the timeline container (default: 100%) */
  width?: string | number;
}

/**
 * DashboardEventTimeline component
 *
 * Displays a virtualized list of key events with pause/resume functionality.
 * Uses react-window for efficient rendering of large lists.
 */
export const DashboardEventTimeline: React.FC<DashboardEventTimelineProps> = ({
  height = 400,
  width = '100%',
}) => {
  const events = useDashboardStore((state) => state.events);
  const [isPaused, setIsPaused] = useState(false);
  const [pausedEvents, setPausedEvents] = useState<KeyEvent[]>([]);
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  /**
   * Toggle pause/resume
   * When paused, events are buffered but not displayed
   */
  const togglePause = useCallback(() => {
    if (!isPaused) {
      // Pausing: capture current events
      setPausedEvents([...events]);
    } else {
      // Resuming: clear paused buffer
      setPausedEvents([]);
    }
    setIsPaused(!isPaused);
  }, [isPaused, events]);

  /**
   * Events to display (paused snapshot or live)
   */
  const displayEvents = useMemo(() => {
    return isPaused ? pausedEvents : events;
  }, [isPaused, pausedEvents, events]);

  /**
   * Reversed events for display (newest first)
   */
  const reversedEvents = useMemo(() => {
    return [...displayEvents].reverse();
  }, [displayEvents]);

  /**
   * Row renderer for react-window
   */
  const Row = useCallback(
    ({ index, style }: { index: number; style: React.CSSProperties }) => {
      const event = reversedEvents[index];
      if (!event) return null;

      const isHovered = hoveredIndex === index;
      const isHighLatency = event.latency > 5000; // >5ms

      return (
        <div
          style={style}
          className={`event-row ${isHighLatency ? 'high-latency' : ''}`}
          onMouseEnter={() => setHoveredIndex(index)}
          onMouseLeave={() => setHoveredIndex(null)}
          role="row"
          aria-label={`Event ${index + 1}: ${event.eventType} ${event.keyCode}`}
        >
          <span className="event-time">{formatTime(event.timestamp)}</span>
          <span className={`event-type ${event.eventType}`}>
            {event.eventType === 'press' ? '↓' : '↑'}
          </span>
          <span className="event-key">{event.keyCode}</span>
          <span className="event-mapping">
            {event.input} → {event.output}
          </span>
          <span className={`event-latency ${isHighLatency ? 'high' : ''}`}>
            {formatLatency(event.latency)}
          </span>

          {/* Tooltip on hover */}
          {isHovered && (
            <div className="event-tooltip" role="tooltip">
              <div className="tooltip-row">
                <strong>Time:</strong> {formatTime(event.timestamp)}
              </div>
              <div className="tooltip-row">
                <strong>Type:</strong> {event.eventType}
              </div>
              <div className="tooltip-row">
                <strong>Key:</strong> {event.keyCode}
              </div>
              <div className="tooltip-row">
                <strong>Mapping:</strong> {event.input} → {event.output}
              </div>
              <div className="tooltip-row">
                <strong>Latency:</strong> {formatLatency(event.latency)}
              </div>
            </div>
          )}
        </div>
      );
    },
    [reversedEvents, hoveredIndex]
  );

  return (
    <div className="dashboard-event-timeline" role="region" aria-label="Event timeline">
      <div className="timeline-header">
        <h3>Event Timeline</h3>
        <div className="timeline-controls">
          <button
            onClick={togglePause}
            className={`pause-button ${isPaused ? 'paused' : ''}`}
            aria-pressed={isPaused}
            aria-label={isPaused ? 'Resume event timeline' : 'Pause event timeline'}
          >
            {isPaused ? '▶ Resume' : '⏸ Pause'}
          </button>
          <span className="event-count" aria-live="polite">
            {displayEvents.length} / 100 events
          </span>
        </div>
      </div>

      <div className="timeline-legend">
        <span className="legend-item">
          <span className="legend-icon press">↓</span> Press
        </span>
        <span className="legend-item">
          <span className="legend-icon release">↑</span> Release
        </span>
        <span className="legend-item">
          <span className="legend-dot high-latency"></span> High Latency (&gt;5ms)
        </span>
      </div>

      {displayEvents.length === 0 ? (
        <div className="timeline-empty" role="status">
          <p>No events yet. Start typing to see events appear here.</p>
        </div>
      ) : (
        <List
          height={height}
          itemCount={reversedEvents.length}
          itemSize={40}
          width={width}
          className="event-list"
          role="list"
          aria-label="Event list"
        >
          {Row}
        </List>
      )}

      {isPaused && events.length !== pausedEvents.length && (
        <div className="paused-indicator" aria-live="polite">
          Timeline paused. {events.length - pausedEvents.length} new events buffered.
        </div>
      )}
    </div>
  );
};
