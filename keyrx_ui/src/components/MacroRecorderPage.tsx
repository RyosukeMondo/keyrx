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
import { useSimulator } from '../hooks/useSimulator';
import { EventTimeline } from './EventTimeline';
import { TemplateLibrary } from './TemplateLibrary';
import {
  eventCodeToVK,
  generateRhaiMacro,
  generateMacroJSON,
  getMacroStats,
} from '../utils/macroGenerator';
import {
  textToMacroEvents,
  getTextSnippetStats,
  TEXT_SNIPPET_TEMPLATES,
} from '../utils/textSnippetTemplate';
import { formatTimestampMs } from '../utils/timeFormatting';
import { formatKeyCode } from '../utils/keyCodeMapping';
import type { EventSequence, SimKeyEvent } from '../wasm/core';
import './MacroRecorderPage.css';

/**
 * Convert MacroEvent array to EventSequence for simulator.
 */
function macroEventsToEventSequence(events: MacroEvent[]): EventSequence {
  const simEvents: SimKeyEvent[] = events.map((event) => ({
    keycode: eventCodeToVK(event.event.code),
    event_type: event.event.value === 1 ? 'press' : 'release',
    timestamp_us: event.relative_timestamp_us,
  }));

  return { events: simEvents };
}

/**
 * MacroRecorderPage component.
 */
export function MacroRecorderPage() {
  const { state, startRecording, stopRecording, clearEvents, clearError } =
    useMacroRecorder();

  const simulator = useSimulator();

  const [rhaiCode, setRhaiCode] = useState<string>('');
  const [editedEvents, setEditedEvents] = useState<MacroEvent[]>([]);
  const [triggerKey, setTriggerKey] = useState<string>('VK_F13');
  const [showTestPanel, setShowTestPanel] = useState<boolean>(false);
  const [textSnippet, setTextSnippet] = useState<string>('');
  const [showTextSnippet, setShowTextSnippet] = useState<boolean>(false);
  const [showTemplateLibrary, setShowTemplateLibrary] = useState<boolean>(false);

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

  const handleTestMacro = async () => {
    if (editedEvents.length === 0) return;

    // Load the generated Rhai code into the simulator
    await simulator.loadConfig(rhaiCode);

    // Convert macro events to simulation format
    const eventSequence = macroEventsToEventSequence(editedEvents);

    // Run the simulation
    await simulator.simulate(eventSequence);

    // Show the test panel
    setShowTestPanel(true);
  };

  const handleLoadTextSnippet = () => {
    if (!textSnippet.trim()) return;

    // Convert text to macro events
    const events = textToMacroEvents(textSnippet, {
      keyDelay: 10,
      optimize: true,
    });

    // Update the edited events
    setEditedEvents(events);
  };

  const handleLoadTemplate = (templateKey: keyof typeof TEXT_SNIPPET_TEMPLATES) => {
    const template = TEXT_SNIPPET_TEMPLATES[templateKey];
    setTextSnippet(template.template);
  };

  const handleSelectFromLibrary = (events: MacroEvent[], templateName: string) => {
    setEditedEvents(events);
    setShowTemplateLibrary(false);
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

      {/* Text Snippet Panel */}
      <div className="text-snippet-section">
        <div className="section-header">
          <h3>
            Text Snippet Template
            <button
              onClick={() => setShowTextSnippet(!showTextSnippet)}
              className="btn-toggle"
            >
              {showTextSnippet ? '▼' : '▶'}
            </button>
          </h3>
          <p className="section-hint">Convert text into keyboard macro events</p>
        </div>

        {showTextSnippet && (
          <div className="text-snippet-panel">
            <div className="snippet-controls">
              <div className="template-buttons">
                <span className="template-label">Templates:</span>
                {(Object.keys(TEXT_SNIPPET_TEMPLATES) as Array<keyof typeof TEXT_SNIPPET_TEMPLATES>).map((key) => (
                  <button
                    key={key}
                    onClick={() => handleLoadTemplate(key)}
                    className="btn-template"
                  >
                    {TEXT_SNIPPET_TEMPLATES[key].name}
                  </button>
                ))}
              </div>
            </div>

            <div className="snippet-input">
              <textarea
                value={textSnippet}
                onChange={(e) => setTextSnippet(e.target.value)}
                placeholder="Enter text to convert to macro (e.g., 'Hello, World!')"
                rows={4}
                className="snippet-textarea"
              />
            </div>

            <div className="snippet-actions">
              {textSnippet && (
                <div className="snippet-stats">
                  {(() => {
                    const stats = getTextSnippetStats(textSnippet);
                    return (
                      <>
                        <span>{stats.characters} chars</span>
                        <span>{stats.supportedCharacters} supported</span>
                        {stats.unsupportedCharacters > 0 && (
                          <span className="warning">
                            {stats.unsupportedCharacters} unsupported
                          </span>
                        )}
                        <span>{stats.steps} steps</span>
                        <span>~{stats.estimatedDurationMs}ms</span>
                      </>
                    );
                  })()}
                </div>
              )}

              <button
                onClick={handleLoadTextSnippet}
                disabled={!textSnippet.trim()}
                className="btn btn-primary"
              >
                Load as Macro
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Template Library Button */}
      <div className="library-section">
        <button
          onClick={() => setShowTemplateLibrary(!showTemplateLibrary)}
          className="btn btn-primary"
        >
          {showTemplateLibrary ? 'Close' : 'Open'} Template Library
        </button>
      </div>

      {/* Template Library */}
      {showTemplateLibrary && (
        <TemplateLibrary
          onSelectTemplate={handleSelectFromLibrary}
          isOpen={showTemplateLibrary}
          onClose={() => setShowTemplateLibrary(false)}
        />
      )}

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
                      <td>{formatTimestampMs(event.relative_timestamp_us)}</td>
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
            <div className="panel-header-actions">
              <button
                onClick={handleTestMacro}
                disabled={editedEvents.length === 0 || simulator.state.loadingState === 'loading'}
                className="btn-test"
              >
                {simulator.state.loadingState === 'loading' ? 'Testing...' : 'Test Macro'}
              </button>
              <button
                onClick={handleCopyCode}
                disabled={state.events.length === 0}
                className="btn-copy"
              >
                Copy Code
              </button>
            </div>
          </div>

          <div className="code-preview">
            <pre>
              <code>{rhaiCode}</code>
            </pre>
          </div>
        </div>
      </div>

      {/* Test Results Panel */}
      {showTestPanel && simulator.state.result && (
        <div className="test-panel">
          <div className="panel-header">
            <h3>Test Results</h3>
            <button onClick={() => setShowTestPanel(false)} className="btn-close">
              ×
            </button>
          </div>

          {simulator.state.error && (
            <div className="test-error">
              <strong>Error:</strong> {simulator.state.error}
            </div>
          )}

          {!simulator.state.error && (
            <div className="test-results">
              <div className="test-stats">
                <h4>Latency Statistics</h4>
                <table className="stats-table">
                  <tbody>
                    <tr>
                      <td>Min Latency:</td>
                      <td>{(simulator.state.result.latency_stats.min_us / 1000).toFixed(3)} ms</td>
                    </tr>
                    <tr>
                      <td>Avg Latency:</td>
                      <td>{(simulator.state.result.latency_stats.avg_us / 1000).toFixed(3)} ms</td>
                    </tr>
                    <tr>
                      <td>Max Latency:</td>
                      <td>{(simulator.state.result.latency_stats.max_us / 1000).toFixed(3)} ms</td>
                    </tr>
                    <tr>
                      <td>P95 Latency:</td>
                      <td>{(simulator.state.result.latency_stats.p95_us / 1000).toFixed(3)} ms</td>
                    </tr>
                    <tr>
                      <td>P99 Latency:</td>
                      <td>{(simulator.state.result.latency_stats.p99_us / 1000).toFixed(3)} ms</td>
                    </tr>
                  </tbody>
                </table>
              </div>

              <div className="test-timeline">
                <h4>Event Timeline ({simulator.state.result.timeline.length} events)</h4>
                <div className="timeline-list">
                  {simulator.state.result.timeline.slice(0, 20).map((entry, index) => (
                    <div key={index} className="timeline-entry">
                      <span className="timeline-timestamp">
                        {formatTimestampMs(entry.timestamp_us)}
                      </span>
                      {entry.input && (
                        <span className="timeline-input">
                          Input: {entry.input.keycode} ({entry.input.event_type})
                        </span>
                      )}
                      {entry.outputs.length > 0 && (
                        <span className="timeline-outputs">
                          Outputs: {entry.outputs.map(o => `${o.keycode}(${o.event_type})`).join(', ')}
                        </span>
                      )}
                      <span className="timeline-latency">
                        {(entry.latency_us / 1000).toFixed(3)} ms
                      </span>
                    </div>
                  ))}
                  {simulator.state.result.timeline.length > 20 && (
                    <div className="timeline-more">
                      ... and {simulator.state.result.timeline.length - 20} more events
                    </div>
                  )}
                </div>
              </div>

              <div className="test-final-state">
                <h4>Final State</h4>
                <div className="state-info">
                  <div>
                    <strong>Active Modifiers:</strong>{' '}
                    {simulator.state.result.final_state.active_modifiers.length > 0
                      ? simulator.state.result.final_state.active_modifiers.join(', ')
                      : 'None'}
                  </div>
                  <div>
                    <strong>Active Locks:</strong>{' '}
                    {simulator.state.result.final_state.active_locks.length > 0
                      ? simulator.state.result.final_state.active_locks.join(', ')
                      : 'None'}
                  </div>
                  <div>
                    <strong>Active Layer:</strong>{' '}
                    {simulator.state.result.final_state.active_layer || 'None'}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
