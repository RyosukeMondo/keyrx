import React, { useState, useEffect, useRef } from 'react';
import type { SimKeyEvent, EventSequence } from '../../wasm/core';
import './EventSequenceEditor.css';

interface EventSequenceEditorProps {
  onSubmit: (sequence: EventSequence) => void;
  disabled?: boolean;
}

interface EventForm {
  id: string;
  keyCode: string;
  eventType: 'press' | 'release';
  timestamp: number;
}

const COMMON_KEY_CODES = [
  { code: 'VK_A', name: 'A' },
  { code: 'VK_B', name: 'B' },
  { code: 'VK_C', name: 'C' },
  { code: 'VK_D', name: 'D' },
  { code: 'VK_E', name: 'E' },
  { code: 'VK_F', name: 'F' },
  { code: 'VK_G', name: 'G' },
  { code: 'VK_H', name: 'H' },
  { code: 'VK_I', name: 'I' },
  { code: 'VK_J', name: 'J' },
  { code: 'VK_K', name: 'K' },
  { code: 'VK_L', name: 'L' },
  { code: 'VK_M', name: 'M' },
  { code: 'VK_N', name: 'N' },
  { code: 'VK_O', name: 'O' },
  { code: 'VK_P', name: 'P' },
  { code: 'VK_Q', name: 'Q' },
  { code: 'VK_R', name: 'R' },
  { code: 'VK_S', name: 'S' },
  { code: 'VK_T', name: 'T' },
  { code: 'VK_U', name: 'U' },
  { code: 'VK_V', name: 'V' },
  { code: 'VK_W', name: 'W' },
  { code: 'VK_X', name: 'X' },
  { code: 'VK_Y', name: 'Y' },
  { code: 'VK_Z', name: 'Z' },
  { code: 'VK_LShift', name: 'Left Shift' },
  { code: 'VK_LCtrl', name: 'Left Ctrl' },
  { code: 'VK_LAlt', name: 'Left Alt' },
  { code: 'VK_Space', name: 'Space' },
  { code: 'VK_Enter', name: 'Enter' },
  { code: 'VK_Esc', name: 'Esc' },
];

export const EventSequenceEditor: React.FC<EventSequenceEditorProps> = ({
  onSubmit,
  disabled = false,
}) => {
  const [events, setEvents] = useState<EventForm[]>([]);
  const [nextId, setNextId] = useState(1);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const createNewEvent = (): EventForm => ({
    id: `event-${nextId}`,
    keyCode: 'VK_A', // Default to 'A'
    eventType: 'press',
    timestamp: events.length > 0 ? events[events.length - 1].timestamp + 100 : 0,
  });

  const addEvent = () => {
    const newEvent = createNewEvent();
    setEvents([...events, newEvent]);
    setNextId(nextId + 1);
    setEditingId(newEvent.id);
  };

  const removeEvent = (id: string) => {
    setEvents(events.filter((e) => e.id !== id));
    setValidationErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[id];
      return newErrors;
    });
  };

  const updateEvent = (id: string, field: keyof EventForm, value: string | number) => {
    setEvents(
      events.map((e) => (e.id === id ? { ...e, [field]: value } : e))
    );
    // Clear validation error for this field
    if (validationErrors[id]) {
      setValidationErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[id];
        return newErrors;
      });
    }
  };

  const validateSequence = (): boolean => {
    const errors: Record<string, string> = {};
    let isValid = true;

    // Check each event
    events.forEach((event, index) => {
      // Validate timestamp is positive
      if (event.timestamp < 0) {
        errors[event.id] = 'Timestamp must be positive';
        isValid = false;
        return;
      }

      // Validate timestamps are in ascending order
      if (index > 0 && event.timestamp <= events[index - 1].timestamp) {
        errors[event.id] = 'Timestamp must be greater than previous event';
        isValid = false;
        return;
      }

      // Validate key code is not empty
      if (!event.keyCode || event.keyCode.trim() === '') {
        errors[event.id] = 'Key code is required';
        isValid = false;
        return;
      }
    });

    setValidationErrors(errors);
    return isValid;
  };

  const handleSubmit = () => {
    if (!validateSequence()) {
      return;
    }

    if (events.length === 0) {
      alert('Please add at least one event to the sequence');
      return;
    }

    // Convert EventForm[] to EventSequence
    const keyEvents: SimKeyEvent[] = events.map((e) => ({
      keycode: e.keyCode,
      event_type: e.eventType,
      timestamp_us: e.timestamp,
    }));

    const sequence: EventSequence = {
      events: keyEvents,
    };

    onSubmit(sequence);
  };

  const handleExport = () => {
    if (events.length === 0) {
      alert('No events to export. Please add at least one event.');
      return;
    }

    // Convert EventForm[] to EventSequence
    const keyEvents: SimKeyEvent[] = events.map((e) => ({
      keycode: e.keyCode,
      event_type: e.eventType,
      timestamp_us: e.timestamp,
    }));

    const sequence: EventSequence = {
      events: keyEvents,
    };

    // Create JSON blob
    const json = JSON.stringify(sequence, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    // Create download link
    const link = document.createElement('a');
    link.href = url;
    link.download = `event-sequence-${Date.now()}.json`;
    document.body.appendChild(link);
    link.click();

    // Cleanup
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Validate file size (max 1MB)
    const MAX_FILE_SIZE = 1024 * 1024; // 1MB
    if (file.size > MAX_FILE_SIZE) {
      alert('File is too large. Maximum file size is 1MB.');
      e.target.value = ''; // Reset input
      return;
    }

    // Read file
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const json = event.target?.result as string;
        const sequence = JSON.parse(json) as EventSequence;

        // Validate JSON structure
        if (!sequence || typeof sequence !== 'object') {
          throw new Error('Invalid JSON structure');
        }

        if (!Array.isArray(sequence.events)) {
          throw new Error('Missing or invalid "events" array');
        }

        // Validate each event
        const importedEvents: EventForm[] = [];
        for (let i = 0; i < sequence.events.length; i++) {
          const event = sequence.events[i];

          if (!event || typeof event !== 'object') {
            throw new Error(`Event ${i + 1} is not a valid object`);
          }

          if (typeof event.keycode !== 'string' || !event.keycode.trim()) {
            throw new Error(`Event ${i + 1} has invalid or missing keycode`);
          }

          if (event.event_type !== 'press' && event.event_type !== 'release') {
            throw new Error(`Event ${i + 1} has invalid event_type (must be "press" or "release")`);
          }

          if (typeof event.timestamp_us !== 'number' || event.timestamp_us < 0) {
            throw new Error(`Event ${i + 1} has invalid timestamp (must be a non-negative number)`);
          }

          // Check timestamps are in ascending order
          if (i > 0 && event.timestamp_us <= sequence.events[i - 1].timestamp_us) {
            throw new Error(
              `Event ${i + 1} timestamp (${event.timestamp_us}μs) must be greater than previous event (${sequence.events[i - 1].timestamp_us}μs)`
            );
          }

          importedEvents.push({
            id: `imported-${i}`,
            keyCode: event.keycode,
            eventType: event.event_type,
            timestamp: event.timestamp_us,
          });
        }

        // Import successful - replace current events
        setEvents(importedEvents);
        setNextId(importedEvents.length + 1);
        setValidationErrors({});
        setSelectedEventId(null);
        setEditingId(null);

        alert(`Successfully imported ${importedEvents.length} events.`);
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error occurred';
        alert(`Failed to import sequence:\n\n${message}\n\nPlease check the file format and try again.`);
      }

      // Reset file input
      e.target.value = '';
    };

    reader.onerror = () => {
      alert('Failed to read file. Please try again.');
      e.target.value = '';
    };

    reader.readAsText(file);
  };

  const adjustTimestamp = (id: string, delta: number) => {
    setEvents(
      events.map((e) =>
        e.id === id ? { ...e, timestamp: Math.max(0, e.timestamp + delta) } : e
      )
    );
  };

  // Keyboard shortcut handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (disabled) return;

      // Don't trigger shortcuts when typing in input fields
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT'
      ) {
        // Allow arrow keys for timestamp adjustment in number inputs
        if (target.tagName === 'INPUT' && (target as HTMLInputElement).type === 'number') {
          if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
            // Let browser handle number input arrows
            return;
          }
        }
        // Allow Ctrl+Enter in input fields
        if (e.ctrlKey && e.key === 'Enter') {
          e.preventDefault();
          addEvent();
          return;
        }
        return;
      }

      // Ctrl+Enter: Add new event
      if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        addEvent();
        return;
      }

      // Delete: Remove selected event
      if (e.key === 'Delete' && selectedEventId) {
        e.preventDefault();
        removeEvent(selectedEventId);
        setSelectedEventId(null);
        return;
      }

      // Arrow keys: Adjust timestamp of selected event
      if (selectedEventId && !e.ctrlKey && !e.shiftKey && !e.altKey) {
        if (e.key === 'ArrowUp') {
          e.preventDefault();
          adjustTimestamp(selectedEventId, 10); // Increase by 10μs
          return;
        }
        if (e.key === 'ArrowDown') {
          e.preventDefault();
          adjustTimestamp(selectedEventId, -10); // Decrease by 10μs
          return;
        }
        if (e.key === 'ArrowRight') {
          e.preventDefault();
          adjustTimestamp(selectedEventId, 100); // Increase by 100μs
          return;
        }
        if (e.key === 'ArrowLeft') {
          e.preventDefault();
          adjustTimestamp(selectedEventId, -100); // Decrease by 100μs
          return;
        }
      }
    };

    const container = containerRef.current;
    if (container) {
      container.addEventListener('keydown', handleKeyDown);
      return () => {
        container.removeEventListener('keydown', handleKeyDown);
      };
    }
  }, [disabled, events, selectedEventId, nextId]);

  return (
    <div className="event-sequence-editor" ref={containerRef} tabIndex={0}>
      <div className="editor-header">
        <h3>Custom Event Sequence</h3>
        <div className="header-buttons">
          <button
            onClick={addEvent}
            disabled={disabled}
            className="btn-add"
            title="Add new event (Ctrl+Enter)"
          >
            + Add Event
          </button>
          <button
            onClick={handleExport}
            disabled={disabled || events.length === 0}
            className="btn-export"
            title="Export sequence as JSON file"
          >
            Export
          </button>
          <label className="btn-import" title="Import sequence from JSON file">
            Import
            <input
              type="file"
              accept=".json,application/json"
              onChange={handleImport}
              disabled={disabled}
              style={{ display: 'none' }}
            />
          </label>
        </div>
      </div>

      <div className="keyboard-shortcuts-hint">
        <strong>Keyboard Shortcuts:</strong> Ctrl+Enter to add event, Delete to remove selected,
        Arrow keys to adjust timestamp (↑↓: ±10μs, ←→: ±100μs)
      </div>

      {events.length === 0 && (
        <div className="empty-state">
          <p>No events yet. Add events to create a custom sequence.</p>
          <p className="hint">
            Events must have ascending timestamps (in microseconds).
          </p>
        </div>
      )}

      <div className="events-list">
        {events.map((event, index) => (
          <div
            key={event.id}
            className={`event-item ${editingId === event.id ? 'editing' : ''} ${
              validationErrors[event.id] ? 'error' : ''
            } ${selectedEventId === event.id ? 'selected' : ''}`}
            onClick={() => setSelectedEventId(event.id)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                setSelectedEventId(event.id);
              }
            }}
            aria-label={`Event ${index + 1}: ${event.keyCode} ${event.eventType} at ${event.timestamp}μs`}
          >
            <div className="event-number">{index + 1}</div>

            <div className="event-fields">
              <div className="field">
                <label htmlFor={`keycode-${event.id}`}>Key Code:</label>
                <select
                  id={`keycode-${event.id}`}
                  value={event.keyCode}
                  onChange={(e) =>
                    updateEvent(event.id, 'keyCode', e.target.value)
                  }
                  disabled={disabled}
                >
                  {COMMON_KEY_CODES.map((key) => (
                    <option key={key.code} value={key.code}>
                      {key.name} ({key.code})
                    </option>
                  ))}
                </select>
              </div>

              <div className="field">
                <label htmlFor={`type-${event.id}`}>Type:</label>
                <select
                  id={`type-${event.id}`}
                  value={event.eventType}
                  onChange={(e) =>
                    updateEvent(event.id, 'eventType', e.target.value as 'press' | 'release')
                  }
                  disabled={disabled}
                >
                  <option value="press">Press</option>
                  <option value="release">Release</option>
                </select>
              </div>

              <div className="field">
                <label htmlFor={`timestamp-${event.id}`}>
                  Timestamp (μs):
                </label>
                <input
                  id={`timestamp-${event.id}`}
                  type="number"
                  value={event.timestamp}
                  onChange={(e) =>
                    updateEvent(event.id, 'timestamp', parseInt(e.target.value, 10) || 0)
                  }
                  disabled={disabled}
                  min="0"
                  step="100"
                />
              </div>
            </div>

            <button
              onClick={() => removeEvent(event.id)}
              disabled={disabled}
              className="btn-remove"
              title="Remove event"
            >
              ×
            </button>

            {validationErrors[event.id] && (
              <div className="validation-error">{validationErrors[event.id]}</div>
            )}
          </div>
        ))}
      </div>

      {events.length > 0 && (
        <div className="editor-footer">
          <div className="event-summary">
            <span className="summary-item">
              <strong>{events.length}</strong> events
            </span>
            <span className="summary-item">
              <strong>{events[events.length - 1]?.timestamp || 0}</strong> μs duration
            </span>
          </div>
          <button
            onClick={handleSubmit}
            disabled={disabled || events.length === 0}
            className="btn-simulate"
          >
            Simulate Custom Sequence
          </button>
        </div>
      )}
    </div>
  );
};
