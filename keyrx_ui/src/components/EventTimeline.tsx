/**
 * EventTimeline - Visual timeline editor for macro events.
 *
 * Features:
 * - Timeline visualization of key events
 * - Drag events to adjust timing
 * - Edit event timestamps
 * - Delete events
 */

import { useState, useRef, useEffect } from 'react';
import type { MacroEvent } from '../hooks/useMacroRecorder';
import './EventTimeline.css';

interface EventTimelineProps {
  /** Array of macro events to display */
  events: MacroEvent[];
  /** Callback when events are modified */
  onEventsChange: (events: MacroEvent[]) => void;
  /** Whether the timeline is editable */
  editable?: boolean;
}

/**
 * Formats a timestamp in microseconds to display string.
 */
function formatTime(timestampUs: number): string {
  const ms = timestampUs / 1000;
  if (ms < 1000) {
    return `${ms.toFixed(0)}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}

/**
 * Formats a key code to a human-readable key name.
 */
function formatKeyCode(code: number): string {
  const keyMap: Record<number, string> = {
    1: 'ESC', 16: 'Q', 17: 'W', 18: 'E', 19: 'R', 20: 'T',
    21: 'Y', 22: 'U', 23: 'I', 24: 'O', 25: 'P',
    30: 'A', 31: 'S', 32: 'D', 33: 'F', 34: 'G',
    35: 'H', 36: 'J', 37: 'K', 38: 'L',
    28: 'ENTER', 29: 'LCTRL', 42: 'LSHIFT', 56: 'LALT', 57: 'SPACE',
  };
  return keyMap[code] || `KEY_${code}`;
}

/**
 * EventTimeline component for editing macro event sequences.
 */
export function EventTimeline({
  events,
  onEventsChange,
  editable = true,
}: EventTimelineProps) {
  const [selectedEventIndex, setSelectedEventIndex] = useState<number | null>(null);
  const [editingTimestamp, setEditingTimestamp] = useState<number | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const timelineRef = useRef<HTMLDivElement>(null);
  const [timelineWidth, setTimelineWidth] = useState(800);

  // Calculate timeline scale based on total duration
  const maxTimestamp = events.length > 0
    ? Math.max(...events.map(e => e.relative_timestamp_us))
    : 100000;

  const scale = timelineWidth / maxTimestamp;

  // Update timeline width on mount and resize
  useEffect(() => {
    const updateWidth = () => {
      if (timelineRef.current) {
        setTimelineWidth(timelineRef.current.clientWidth - 40);
      }
    };

    updateWidth();
    window.addEventListener('resize', updateWidth);
    return () => window.removeEventListener('resize', updateWidth);
  }, []);

  /**
   * Handle event deletion.
   */
  const handleDeleteEvent = (index: number) => {
    if (!editable) return;

    const newEvents = events.filter((_, i) => i !== index);
    onEventsChange(newEvents);
    setSelectedEventIndex(null);
  };

  /**
   * Handle timestamp edit.
   */
  const handleTimestampEdit = (index: number, newTimestampMs: number) => {
    if (!editable) return;

    const newTimestampUs = Math.max(0, newTimestampMs * 1000);
    const newEvents = [...events];
    newEvents[index] = {
      ...newEvents[index],
      relative_timestamp_us: newTimestampUs,
    };

    // Re-sort events by timestamp
    newEvents.sort((a, b) => a.relative_timestamp_us - b.relative_timestamp_us);

    onEventsChange(newEvents);
    setEditingTimestamp(null);
  };

  /**
   * Handle drag start.
   */
  const handleDragStart = (index: number) => {
    if (!editable) return;
    setDraggedIndex(index);
  };

  /**
   * Handle drag over timeline.
   */
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  /**
   * Handle drop on timeline.
   */
  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();

    if (draggedIndex === null || !editable || !timelineRef.current) return;

    // Calculate new timestamp based on drop position
    const rect = timelineRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left - 20; // Account for padding
    const newTimestampUs = Math.max(0, x / scale);

    const newEvents = [...events];
    newEvents[draggedIndex] = {
      ...newEvents[draggedIndex],
      relative_timestamp_us: newTimestampUs,
    };

    // Re-sort events by timestamp
    newEvents.sort((a, b) => a.relative_timestamp_us - b.relative_timestamp_us);

    onEventsChange(newEvents);
    setDraggedIndex(null);
  };

  return (
    <div className="event-timeline">
      <div className="timeline-header">
        <h3>Event Timeline</h3>
        <div className="timeline-info">
          {events.length} events • {formatTime(maxTimestamp)} total
        </div>
      </div>

      <div
        ref={timelineRef}
        className="timeline-canvas"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        {/* Time markers */}
        <div className="time-markers">
          {[0, 0.25, 0.5, 0.75, 1].map((fraction) => (
            <div
              key={fraction}
              className="time-marker"
              style={{ left: `${fraction * 100}%` }}
            >
              <div className="marker-line" />
              <div className="marker-label">
                {formatTime(maxTimestamp * fraction)}
              </div>
            </div>
          ))}
        </div>

        {/* Event markers */}
        <div className="event-markers">
          {events.map((event, index) => {
            const position = (event.relative_timestamp_us / maxTimestamp) * 100;
            const isPress = event.event.value === 1;
            const isSelected = selectedEventIndex === index;

            return (
              <div
                key={index}
                className={`event-marker ${isPress ? 'press' : 'release'} ${
                  isSelected ? 'selected' : ''
                }`}
                style={{ left: `${position}%` }}
                draggable={editable}
                onDragStart={() => handleDragStart(index)}
                onClick={() => setSelectedEventIndex(index)}
              >
                <div className="event-marker-icon">
                  {isPress ? '▼' : '▲'}
                </div>
                <div className="event-marker-label">
                  {formatKeyCode(event.event.code)}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Event details panel */}
      {selectedEventIndex !== null && events[selectedEventIndex] && (
        <div className="event-details">
          <div className="details-header">
            <h4>Event #{selectedEventIndex + 1}</h4>
            {editable && (
              <button
                className="btn-delete"
                onClick={() => handleDeleteEvent(selectedEventIndex)}
              >
                Delete
              </button>
            )}
          </div>

          <div className="details-content">
            <div className="detail-row">
              <label>Key:</label>
              <span className="detail-value">
                {formatKeyCode(events[selectedEventIndex].event.code)}
              </span>
            </div>

            <div className="detail-row">
              <label>Action:</label>
              <span className="detail-value">
                {events[selectedEventIndex].event.value === 1 ? 'Press' : 'Release'}
              </span>
            </div>

            <div className="detail-row">
              <label>Timestamp:</label>
              {editingTimestamp === selectedEventIndex ? (
                <input
                  type="number"
                  className="timestamp-input"
                  defaultValue={events[selectedEventIndex].relative_timestamp_us / 1000}
                  onBlur={(e) =>
                    handleTimestampEdit(selectedEventIndex, parseFloat(e.target.value))
                  }
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleTimestampEdit(
                        selectedEventIndex,
                        parseFloat(e.currentTarget.value)
                      );
                    }
                  }}
                  autoFocus
                />
              ) : (
                <span
                  className={`detail-value ${editable ? 'editable' : ''}`}
                  onClick={() => editable && setEditingTimestamp(selectedEventIndex)}
                >
                  {formatTime(events[selectedEventIndex].relative_timestamp_us)}
                  {editable && <span className="edit-hint"> (click to edit)</span>}
                </span>
              )}
            </div>
          </div>
        </div>
      )}

      {events.length === 0 && (
        <div className="timeline-empty">
          <p>No events to display</p>
          <p className="timeline-hint">Record some events to see them on the timeline</p>
        </div>
      )}
    </div>
  );
}
