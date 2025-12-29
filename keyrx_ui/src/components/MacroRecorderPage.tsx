/**
 * MacroRecorderPage - Main component for macro recording functionality.
 *
 * This component provides:
 * - Record/stop buttons for macro capture
 * - Real-time display of captured events
 * - Rhai code generation preview
 * - Export and clear functionality
 */

import { useState, useEffect } from 'react';
import { useMacroRecorder, type MacroEvent } from '../hooks/useMacroRecorder';
import { EventTimeline } from './EventTimeline';
import './MacroRecorderPage.css';

/**
 * Formats a timestamp in microseconds to a human-readable string.
 */
function formatTimestamp(timestampUs: number): string {
  const ms = timestampUs / 1000;
  if (ms < 1000) {
    return `${ms.toFixed(2)}ms`;
  }
  const seconds = ms / 1000;
  return `${seconds.toFixed(3)}s`;
}

/**
 * Formats a key code to a human-readable key name.
 */
function formatKeyCode(code: number): string {
  // Common Linux input event codes
  const keyMap: Record<number, string> = {
    1: 'ESC',
    2: '1',
    3: '2',
    4: '3',
    5: '4',
    6: '5',
    7: '6',
    8: '7',
    9: '8',
    10: '9',
    11: '0',
    14: 'BACKSPACE',
    15: 'TAB',
    16: 'Q',
    17: 'W',
    18: 'E',
    19: 'R',
    20: 'T',
    21: 'Y',
    22: 'U',
    23: 'I',
    24: 'O',
    25: 'P',
    28: 'ENTER',
    29: 'LCTRL',
    30: 'A',
    31: 'S',
    32: 'D',
    33: 'F',
    34: 'G',
    35: 'H',
    36: 'J',
    37: 'K',
    38: 'L',
    42: 'LSHIFT',
    54: 'RSHIFT',
    56: 'LALT',
    57: 'SPACE',
    97: 'RCTRL',
    100: 'RALT',
  };

  return keyMap[code] || `KEY_${code}`;
}

/**
 * Generates Rhai macro code from recorded events.
 */
function generateRhaiCode(events: MacroEvent[]): string {
  if (events.length === 0) {
    return '// No events recorded yet';
  }

  const lines: string[] = [
    '// Generated macro from recorded events',
    'macro("recorded_macro", || {',
  ];

  for (const event of events) {
    const keyName = formatKeyCode(event.event.code);
    const action = event.event.value === 1 ? 'press' : 'release';
    const delayMs = event.relative_timestamp_us / 1000;

    if (delayMs > 0) {
      lines.push(`    delay(${Math.round(delayMs)});`);
    }
    lines.push(`    ${action}(${keyName});`);
  }

  lines.push('});');

  return lines.join('\n');
}

/**
 * MacroRecorderPage component.
 */
export function MacroRecorderPage() {
  const { state, startRecording, stopRecording, clearEvents, clearError } =
    useMacroRecorder();

  const [rhaiCode, setRhaiCode] = useState<string>('');
  const [editedEvents, setEditedEvents] = useState<MacroEvent[]>([]);

  // Sync edited events with recorded events
  useEffect(() => {
    setEditedEvents(state.events);
  }, [state.events]);

  // Update Rhai code preview when events change
  useEffect(() => {
    setRhaiCode(generateRhaiCode(editedEvents));
  }, [editedEvents]);

  const handleStartRecording = async () => {
    await startRecording();
  };

  const handleStopRecording = async () => {
    await stopRecording();
  };

  const handleClearEvents = async () => {
    await clearEvents();
  };

  const handleCopyCode = () => {
    navigator.clipboard.writeText(rhaiCode);
  };

  const handleExportEvents = () => {
    const dataStr = JSON.stringify(editedEvents, null, 2);
    const dataUri = `data:application/json;charset=utf-8,${encodeURIComponent(dataStr)}`;
    const exportFileDefaultName = `macro_${Date.now()}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
  };

  return (
    <div className="macro-recorder-page">
      <div className="recorder-header">
        <h2>Macro Recorder</h2>
        <p>Record keyboard events and generate Rhai macro code</p>
      </div>

      {state.error && (
        <div className="error-banner">
          <span>{state.error}</span>
          <button onClick={clearError} className="error-close">
            ×
          </button>
        </div>
      )}

      <div className="recorder-controls">
        <div className="control-group">
          <button
            onClick={handleStartRecording}
            disabled={state.recordingState === 'recording' || state.isLoading}
            className="btn btn-primary"
          >
            {state.recordingState === 'recording' ? 'Recording...' : 'Start Recording'}
          </button>

          <button
            onClick={handleStopRecording}
            disabled={state.recordingState !== 'recording' || state.isLoading}
            className="btn btn-secondary"
          >
            Stop Recording
          </button>

          <button
            onClick={handleClearEvents}
            disabled={state.events.length === 0 || state.isLoading}
            className="btn btn-danger"
          >
            Clear Events
          </button>
        </div>

        <div className="status-indicator">
          {state.recordingState === 'recording' && (
            <span className="status-recording">
              <span className="recording-dot"></span>
              Recording...
            </span>
          )}
          {state.recordingState === 'stopped' && (
            <span className="status-stopped">Stopped</span>
          )}
          {state.recordingState === 'idle' && <span className="status-idle">Ready</span>}
        </div>
      </div>

      {/* Event Timeline Editor */}
      {editedEvents.length > 0 && (
        <EventTimeline
          events={editedEvents}
          onEventsChange={setEditedEvents}
          editable={state.recordingState !== 'recording'}
        />
      )}

      <div className="recorder-content">
        <div className="events-panel">
          <div className="panel-header">
            <h3>Recorded Events ({editedEvents.length})</h3>
            <button
              onClick={handleExportEvents}
              disabled={editedEvents.length === 0}
              className="btn-export"
            >
              Export JSON
            </button>
          </div>

          <div className="events-list">
            {editedEvents.length === 0 ? (
              <div className="events-empty">
                <p>No events recorded yet</p>
                <p className="events-hint">Click "Start Recording" to begin capturing events</p>
              </div>
            ) : (
              <table className="events-table">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>Timestamp</th>
                    <th>Key</th>
                    <th>Action</th>
                  </tr>
                </thead>
                <tbody>
                  {editedEvents.map((event, index) => (
                    <tr key={index}>
                      <td>{index + 1}</td>
                      <td>{formatTimestamp(event.relative_timestamp_us)}</td>
                      <td className="key-code">{formatKeyCode(event.event.code)}</td>
                      <td className={event.event.value === 1 ? 'action-press' : 'action-release'}>
                        {event.event.value === 1 ? 'Press' : 'Release'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>

        <div className="code-panel">
          <div className="panel-header">
            <h3>Rhai Code Preview</h3>
            <button
              onClick={handleCopyCode}
              disabled={state.events.length === 0}
              className="btn-copy"
            >
              Copy Code
            </button>
          </div>

          <div className="code-preview">
            <pre>
              <code>{rhaiCode}</code>
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}
