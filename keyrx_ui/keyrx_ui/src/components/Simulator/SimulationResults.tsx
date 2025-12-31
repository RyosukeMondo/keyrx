/**
 * SimulationResults Component
 *
 * Displays timeline visualization of simulation events with state changes,
 * input/output comparison, and interactive tooltips.
 */

import React, { useState, useMemo } from 'react';
import type { SimulationResult, TimelineEntry, SimKeyEvent } from '../../wasm/core';
import './SimulationResults.css';

interface SimulationResultsProps {
  result: SimulationResult | null;
}

export default function SimulationResults({ result }: SimulationResultsProps): React.JSX.Element {
  const [hoveredEntry, setHoveredEntry] = useState<TimelineEntry | null>(null);
  const [tooltipPosition, setTooltipPosition] = useState<{ x: number; y: number } | null>(null);
  const [showStateChanges, setShowStateChanges] = useState<boolean>(true);
  const [showDifferencesOnly, setShowDifferencesOnly] = useState<boolean>(false);

  if (!result) {
    return (
      <div className="simulation-results">
        <div className="empty-state">
          <p>No simulation results yet. Run a scenario or custom sequence to see results.</p>
        </div>
      </div>
    );
  }

  const { timeline } = result;

  // Check if entry has state changes
  const hasStateChange = (entry: TimelineEntry): boolean => {
    return (
      entry.state.active_modifiers.length > 0 ||
      entry.state.active_locks.length > 0 ||
      entry.state.active_layer !== null
    );
  };

  // Check if input and output differ
  const hasInputOutputDiff = (entry: TimelineEntry): boolean => {
    if (!entry.input) return false;
    if (entry.outputs.length !== 1) return true;

    const output = entry.outputs[0];
    return (
      entry.input.keycode !== output.keycode ||
      entry.input.event_type !== output.event_type
    );
  };

  // Filter timeline based on toggle states
  const filteredTimeline = useMemo(() => {
    let filtered = timeline;

    if (showDifferencesOnly) {
      filtered = filtered.filter((entry) => hasInputOutputDiff(entry));
    }

    if (!showStateChanges) {
      filtered = filtered.filter((entry) => !hasStateChange(entry));
    }

    return filtered;
  }, [timeline, showDifferencesOnly, showStateChanges]);

  if (timeline.length === 0) {
    return (
      <div className="simulation-results">
        <div className="empty-state">
          <p>Simulation completed with no events.</p>
        </div>
      </div>
    );
  }

  if (filteredTimeline.length === 0) {
    return (
      <div className="simulation-results">
        <h3 className="results-title">Simulation Results</h3>

        <div className="visualization-controls">
          <label className="control-toggle">
            <input
              type="checkbox"
              checked={showStateChanges}
              onChange={(e) => setShowStateChanges(e.target.checked)}
            />
            <span>Show State Changes</span>
          </label>
          <label className="control-toggle">
            <input
              type="checkbox"
              checked={showDifferencesOnly}
              onChange={(e) => setShowDifferencesOnly(e.target.checked)}
            />
            <span>Show Differences Only</span>
          </label>
        </div>

        <div className="empty-state">
          <p>No events match the current filters. Try adjusting the visualization controls.</p>
        </div>
      </div>
    );
  }

  // Calculate timeline scale using filtered events
  const minTime = Math.min(...filteredTimeline.map((entry) => entry.timestamp_us));
  const maxTime = Math.max(...filteredTimeline.map((entry) => entry.timestamp_us));
  const timeRange = maxTime - minTime || 1;

  // Event type colors
  const getEventTypeColor = (entry: TimelineEntry): string => {
    const hasModifierChange = entry.state.active_modifiers.length > 0;
    const hasLockChange = entry.state.active_locks.length > 0;
    const hasLayerChange = entry.state.active_layer !== null;

    if (hasLayerChange) return 'layer-change';
    if (hasLockChange) return 'lock-change';
    if (hasModifierChange) return 'modifier-change';
    return 'regular-event';
  };

  const handleMouseEnter = (entry: TimelineEntry, event: React.MouseEvent): void => {
    setHoveredEntry(entry);
    const rect = (event.target as HTMLElement).getBoundingClientRect();
    setTooltipPosition({
      x: rect.left + rect.width / 2,
      y: rect.top,
    });
  };

  const handleMouseLeave = (): void => {
    setHoveredEntry(null);
    setTooltipPosition(null);
  };

  const formatTimestamp = (timestampUs: number): string => {
    if (timestampUs < 1000) return `${timestampUs}μs`;
    if (timestampUs < 1000000) return `${(timestampUs / 1000).toFixed(1)}ms`;
    return `${(timestampUs / 1000000).toFixed(2)}s`;
  };

  const formatKeyEvent = (event: SimKeyEvent | null): string => {
    if (!event) return 'N/A';
    const action = event.event_type === 'press' ? '↓' : '↑';
    return `${event.keycode}${action}`;
  };

  return (
    <div className="simulation-results">
      <div className="results-header">
        <h3 className="results-title">Simulation Results</h3>
        <div className="event-count">
          {filteredTimeline.length} / {timeline.length} events
        </div>
      </div>

      <div className="visualization-controls">
        <label className="control-toggle">
          <input
            type="checkbox"
            checked={showStateChanges}
            onChange={(e) => setShowStateChanges(e.target.checked)}
          />
          <span>Show State Changes</span>
        </label>
        <label className="control-toggle">
          <input
            type="checkbox"
            checked={showDifferencesOnly}
            onChange={(e) => setShowDifferencesOnly(e.target.checked)}
          />
          <span>Show Differences Only</span>
        </label>
      </div>

      <div className="timeline-container">
        <div className="timeline-header">
          <div className="time-labels">
            <span>{formatTimestamp(minTime)}</span>
            <span>{formatTimestamp(minTime + timeRange / 2)}</span>
            <span>{formatTimestamp(maxTime)}</span>
          </div>
        </div>

        <div className="timeline-track">
          <div className="timeline-line" />
          {filteredTimeline.map((entry, index) => {
            const position = ((entry.timestamp_us - minTime) / timeRange) * 100;
            const eventType = getEventTypeColor(entry);
            const hasDiff = hasInputOutputDiff(entry);

            return (
              <div
                key={`event-${index}`}
                className={`timeline-event ${eventType} ${hasDiff ? 'has-diff' : ''}`}
                style={{ left: `${position}%` }}
                onMouseEnter={(e) => handleMouseEnter(entry, e)}
                onMouseLeave={handleMouseLeave}
                role="button"
                tabIndex={0}
                aria-label={`Event at ${formatTimestamp(entry.timestamp_us)}: ${formatKeyEvent(entry.input)}`}
              >
                <div className="event-marker" />
              </div>
            );
          })}
        </div>

        {hoveredEntry && tooltipPosition && (
          <div
            className="timeline-tooltip"
            style={{
              left: `${tooltipPosition.x}px`,
              top: `${tooltipPosition.y - 10}px`,
            }}
          >
            <div className="tooltip-content">
              <div className="tooltip-row">
                <span className="tooltip-label">Timestamp:</span>
                <span className="tooltip-value">{formatTimestamp(hoveredEntry.timestamp_us)}</span>
              </div>
              <div className="tooltip-row">
                <span className="tooltip-label">Input:</span>
                <span className="tooltip-value">{formatKeyEvent(hoveredEntry.input)}</span>
              </div>
              <div className="tooltip-row">
                <span className="tooltip-label">Outputs:</span>
                <span className="tooltip-value">
                  {hoveredEntry.outputs.length === 0
                    ? 'None'
                    : hoveredEntry.outputs.map((e) => formatKeyEvent(e)).join(', ')}
                </span>
              </div>
              <div className="tooltip-row">
                <span className="tooltip-label">Latency:</span>
                <span className="tooltip-value">{formatTimestamp(hoveredEntry.latency_us)}</span>
              </div>
              {hoveredEntry.state.active_modifiers.length > 0 && (
                <div className="tooltip-row">
                  <span className="tooltip-label">Modifiers:</span>
                  <span className="tooltip-value">
                    [{hoveredEntry.state.active_modifiers.join(', ')}]
                  </span>
                </div>
              )}
              {hoveredEntry.state.active_locks.length > 0 && (
                <div className="tooltip-row">
                  <span className="tooltip-label">Locks:</span>
                  <span className="tooltip-value">
                    [{hoveredEntry.state.active_locks.join(', ')}]
                  </span>
                </div>
              )}
              {hoveredEntry.state.active_layer !== null && (
                <div className="tooltip-row">
                  <span className="tooltip-label">Layer:</span>
                  <span className="tooltip-value">{hoveredEntry.state.active_layer}</span>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      <div className="legend">
        <div className="legend-title">Legend:</div>
        <div className="legend-items">
          <div className="legend-item">
            <div className="legend-marker regular-event" />
            <span>Regular Event</span>
          </div>
          <div className="legend-item">
            <div className="legend-marker modifier-change" />
            <span>Modifier Change</span>
          </div>
          <div className="legend-item">
            <div className="legend-marker lock-change" />
            <span>Lock Change</span>
          </div>
          <div className="legend-item">
            <div className="legend-marker layer-change" />
            <span>Layer Change</span>
          </div>
          <div className="legend-item">
            <div className="legend-marker has-diff" />
            <span>Input/Output Mismatch</span>
          </div>
        </div>
      </div>
    </div>
  );
}
