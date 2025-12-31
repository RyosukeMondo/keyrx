/**
 * ScenarioSelector - Component for selecting and running built-in test scenarios.
 *
 * Provides a dropdown menu of pre-configured test scenarios (tap-hold, layer-switch,
 * modifier combinations) with descriptions and a "Run" button.
 */

import { useState, useCallback } from 'react';
import { BUILT_IN_SCENARIOS } from '../../utils/scenarios';
import type { EventSequence } from '../../wasm/core';
import './ScenarioSelector.css';

interface ScenarioSelectorProps {
  /** Callback when user runs a scenario */
  onRunScenario: (eventSequence: EventSequence) => void;
  /** Whether the component is disabled (no config loaded) */
  disabled?: boolean;
  /** Whether a simulation is currently running */
  isLoading?: boolean;
}

export function ScenarioSelector({
  onRunScenario,
  disabled = false,
  isLoading = false
}: ScenarioSelectorProps) {
  const [selectedScenarioId, setSelectedScenarioId] = useState<string>(
    BUILT_IN_SCENARIOS[0]?.id || ''
  );

  // Get the currently selected scenario
  const selectedScenario = BUILT_IN_SCENARIOS.find(
    s => s.id === selectedScenarioId
  );

  /**
   * Handle scenario selection from dropdown.
   */
  const handleScenarioChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => {
      setSelectedScenarioId(event.target.value);
    },
    []
  );

  /**
   * Handle running the selected scenario.
   */
  const handleRunScenario = useCallback(() => {
    if (!selectedScenario || disabled || isLoading) {
      return;
    }

    try {
      // Generate the event sequence for the selected scenario
      const eventSequence = selectedScenario.generator();
      onRunScenario(eventSequence);
    } catch (error) {
      console.error('Failed to generate scenario:', error);
    }
  }, [selectedScenario, disabled, isLoading, onRunScenario]);

  /**
   * Handle keyboard shortcuts (Enter to run).
   */
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'Enter' && !disabled && !isLoading) {
        handleRunScenario();
      }
    },
    [disabled, isLoading, handleRunScenario]
  );

  return (
    <div className="scenario-selector">
      <div className="selector-container">
        <label htmlFor="scenario-dropdown" className="selector-label">
          Select Scenario:
        </label>
        <select
          id="scenario-dropdown"
          className="scenario-dropdown"
          value={selectedScenarioId}
          onChange={handleScenarioChange}
          onKeyDown={handleKeyDown}
          disabled={disabled || isLoading}
          aria-label="Select a test scenario"
        >
          {BUILT_IN_SCENARIOS.map(scenario => (
            <option key={scenario.id} value={scenario.id}>
              {scenario.name}
            </option>
          ))}
        </select>
      </div>

      {selectedScenario && (
        <div className="scenario-description" aria-live="polite">
          <strong>Description:</strong>
          <p>{selectedScenario.description}</p>
        </div>
      )}

      <button
        className="run-scenario-button"
        onClick={handleRunScenario}
        disabled={disabled || isLoading}
        aria-label={`Run ${selectedScenario?.name || 'selected scenario'}`}
        title={
          disabled
            ? 'Load a configuration first to enable scenarios'
            : isLoading
            ? 'Simulation in progress...'
            : 'Run the selected scenario (Enter)'
        }
      >
        {isLoading ? 'Running...' : 'Run Scenario'}
      </button>
    </div>
  );
}

export default ScenarioSelector;
