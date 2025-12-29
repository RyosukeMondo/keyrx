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
import {
  eventCodeToVK,
  generateRhaiMacro,
  generateMacroJSON,
  getMacroStats,
} from '../utils/macroGenerator';
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
  const vkName = eventCodeToVK(code);
  // Extract just the key part (remove VK_ prefix)
  return vkName.replace('VK_', '').replace('Unknown', 'KEY_');
}

/**
 * MacroRecorderPage component.
 */
export function MacroRecorderPage() {
  const { state, startRecording, stopRecording, clearEvents, clearError } =
    useMacroRecorder();

  const [rhaiCode, setRhaiCode] = useState<string>('');
  const [editedEvents, setEditedEvents] = useState<MacroEvent[]>([]);
  const [triggerKey, setTriggerKey] = useState<string>('VK_F13');

  // Sync edited events with recorded events
  useEffect(() => {
    setEditedEvents(state.events);
  }, [state.events]);

  // Update Rhai code preview when events change
  useEffect(() => {
    if (editedEvents.length === 0) {
      setRhaiCode('// No events recorded yet\n// Click "Start Recording" to begin');
    } else {
      setRhaiCode(
        generateRhaiMacro(editedEvents, triggerKey, {
          macroName: 'Recorded Macro',
          includeComments: true,
          deviceId: '*',
        })
      );
    }
  }, [editedEvents, triggerKey]);

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
    const stats = getMacroStats(editedEvents);
    const metadata = {
      macroName: 'Recorded Macro',
      triggerKey,
      recordedAt: new Date().toISOString(),
      stats,
    };
    const dataStr = generateMacroJSON(editedEvents, metadata);
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
            <div className="panel-header-actions">
              <label htmlFor="trigger-key-select" className="trigger-key-label">
                Trigger Key:
              </label>
              <select
                id="trigger-key-select"
                value={triggerKey}
                onChange={(e) => setTriggerKey(e.target.value)}
                className="trigger-key-select"
              >
                <option value="VK_F13">F13</option>
                <option value="VK_F14">F14</option>
                <option value="VK_F15">F15</option>
                <option value="VK_F16">F16</option>
                <option value="VK_F17">F17</option>
                <option value="VK_F18">F18</option>
              </select>
              <button
                onClick={handleExportEvents}
                disabled={editedEvents.length === 0}
                className="btn-export"
              >
                Export JSON
              </button>
            </div>
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
