/**
 * SimulatorPanel - Main container component for WASM-based keyboard simulation.
 *
 * This component orchestrates the configuration loading, scenario selection,
 * event sequence simulation, and results visualization.
 */

import { useState, useCallback } from 'react';
import { wasmCore, WasmError } from '../../wasm/core';
import type { ConfigHandle, EventSequence, SimulationResult } from '../../wasm/core';
import './SimulatorPanel.css';

type LoadingState = 'idle' | 'loading' | 'success' | 'error';

export function SimulatorPanel() {
  // State management
  const [loadedConfig, setLoadedConfig] = useState<ConfigHandle | null>(null);
  const [simulationResult, setSimulationResult] = useState<SimulationResult | null>(null);
  const [loadingState, setLoadingState] = useState<LoadingState>('idle');
  const [error, setError] = useState<string | null>(null);

  /**
   * Handle configuration loading from Rhai source.
   * TODO: Wire to ConfigLoader component (Task 13)
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const handleLoadConfig = useCallback(async (rhaiSource: string) => {
    try {
      setLoadingState('loading');
      setError(null);

      // Initialize WASM if not already initialized
      await wasmCore.init();

      // Load the configuration
      const configHandle = await wasmCore.loadConfig(rhaiSource);
      setLoadedConfig(configHandle);
      setLoadingState('success');
    } catch (err) {
      const message = err instanceof WasmError
        ? err.message
        : err instanceof Error
        ? err.message
        : 'Unknown error occurred';

      setError(`Failed to load configuration: ${message}`);
      setLoadingState('error');
      setLoadedConfig(null);
    }
  }, []);

  /**
   * Handle simulation execution with the loaded configuration.
   * TODO: Wire to ScenarioSelector and EventSequenceEditor (Tasks 14-15)
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const handleSimulate = useCallback(async (eventSequence: EventSequence) => {
    if (!loadedConfig) {
      setError('No configuration loaded. Please load a configuration first.');
      return;
    }

    try {
      setLoadingState('loading');
      setError(null);

      // Run the simulation
      const result = await wasmCore.simulate(loadedConfig, eventSequence);
      setSimulationResult(result);
      setLoadingState('success');
    } catch (err) {
      const message = err instanceof WasmError
        ? err.message
        : err instanceof Error
        ? err.message
        : 'Unknown error occurred';

      setError(`Simulation failed: ${message}`);
      setLoadingState('error');
      setSimulationResult(null);
    }
  }, [loadedConfig]);

  return (
    <div className="simulator-panel">
      <header className="simulator-header">
        <h2>Configuration Simulator</h2>
        <p>Test your keyboard remapping configurations in the browser</p>
      </header>

      {/* Error display */}
      {error && (
        <div className="error-banner">
          <strong>Error:</strong> {error}
        </div>
      )}

      {/* Config Loader Section */}
      <section className="simulator-section config-loader-section">
        <h3>1. Load Configuration</h3>
        <div className="placeholder-message">
          ConfigLoader component will be inserted here (Task 13)
        </div>
        {loadingState === 'loading' && (
          <div className="loading-spinner">Loading configuration...</div>
        )}
      </section>

      {/* Scenario Selector Section */}
      <section className="simulator-section scenario-selector-section">
        <h3>2. Select or Create Event Sequence</h3>
        {!loadedConfig && (
          <div className="disabled-message">
            Load a configuration first to enable simulation
          </div>
        )}
        {loadedConfig && (
          <>
            <div className="placeholder-message">
              ScenarioSelector component will be inserted here (Task 14)
            </div>
            <div className="placeholder-message">
              EventSequenceEditor component will be inserted here (Task 15)
            </div>
          </>
        )}
      </section>

      {/* Simulation Results Section */}
      {simulationResult && (
        <>
          <section className="simulator-section results-section">
            <h3>3. Simulation Results</h3>
            <div className="placeholder-message">
              SimulationResults component will be inserted here (Task 16)
            </div>
          </section>

          <section className="simulator-section latency-section">
            <h3>4. Performance Metrics</h3>
            <div className="placeholder-message">
              LatencyStats component will be inserted here (Task 17)
            </div>
          </section>
        </>
      )}

      {/* Development Info */}
      <div className="dev-info">
        <p>
          <strong>Status:</strong>{' '}
          {loadedConfig ? '✓ Configuration loaded' : '○ No configuration'}
          {' | '}
          {simulationResult ? `✓ Simulation complete (${simulationResult.timeline.length} events)` : '○ No simulation run'}
        </p>
      </div>
    </div>
  );
}

export default SimulatorPanel;
