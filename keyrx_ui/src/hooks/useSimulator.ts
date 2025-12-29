/**
 * useSimulator - React hook for WASM-based keyboard simulation.
 *
 * This hook wraps the WasmCore API with React state management, providing
 * a reusable interface for loading configurations and running simulations.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { wasmCore, WasmError } from '../wasm/core';
import type { ConfigHandle, EventSequence, SimulationResult } from '../wasm/core';

/**
 * Loading state for async operations.
 */
type LoadingState = 'idle' | 'loading' | 'success' | 'error';

/**
 * State management for the simulator hook.
 */
interface SimulatorState {
  /** Currently loaded configuration handle */
  config: ConfigHandle | null;
  /** Most recent simulation result */
  result: SimulationResult | null;
  /** Loading state for async operations */
  loadingState: LoadingState;
  /** Error message if operation failed */
  error: string | null;
  /** Whether WASM module is initialized */
  isInitialized: boolean;
}

/**
 * Return value of useSimulator hook.
 */
export interface UseSimulatorReturn {
  /** Current simulator state */
  state: SimulatorState;
  /** Load a Rhai configuration */
  loadConfig: (rhaiSource: string) => Promise<void>;
  /** Load a pre-compiled .krx binary */
  loadKrx: (binary: Uint8Array) => Promise<void>;
  /** Run simulation with event sequence */
  simulate: (eventSequence: EventSequence) => Promise<void>;
  /** Clear error message */
  clearError: () => void;
  /** Reset all state */
  reset: () => void;
}

/**
 * Custom hook for WASM-based keyboard simulation.
 *
 * Manages configuration loading, simulation execution, and result state
 * with proper error handling and cleanup.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { state, loadConfig, simulate } = useSimulator();
 *
 *   const handleLoad = async () => {
 *     await loadConfig(rhaiSource);
 *   };
 *
 *   return (
 *     <div>
 *       {state.isInitialized ? 'Ready' : 'Initializing...'}
 *       {state.error && <div>Error: {state.error}</div>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useSimulator(): UseSimulatorReturn {
  // State management
  const [state, setState] = useState<SimulatorState>({
    config: null,
    result: null,
    loadingState: 'idle',
    error: null,
    isInitialized: false,
  });

  // Track mounted state to prevent updates after unmount
  const isMountedRef = useRef(true);

  /**
   * Initialize WASM module on mount.
   */
  useEffect(() => {
    let cancelled = false;

    const initWasm = async () => {
      try {
        await wasmCore.init();
        if (!cancelled && isMountedRef.current) {
          setState((prev) => ({ ...prev, isInitialized: true }));
        }
      } catch (err) {
        if (!cancelled && isMountedRef.current) {
          const message = err instanceof Error ? err.message : 'Failed to initialize WASM';
          setState((prev) => ({
            ...prev,
            error: message,
            loadingState: 'error',
          }));
        }
      }
    };

    initWasm();

    // Cleanup on unmount
    return () => {
      cancelled = true;
      isMountedRef.current = false;
    };
  }, []);

  /**
   * Load a Rhai configuration.
   */
  const loadConfig = useCallback(async (rhaiSource: string) => {
    // Validate input
    if (!rhaiSource.trim()) {
      setState((prev) => ({
        ...prev,
        error: 'Configuration source cannot be empty',
        loadingState: 'error',
      }));
      return;
    }

    if (rhaiSource.length > 1024 * 1024) {
      setState((prev) => ({
        ...prev,
        error: 'Configuration source exceeds 1MB limit',
        loadingState: 'error',
      }));
      return;
    }

    try {
      setState((prev) => ({
        ...prev,
        loadingState: 'loading',
        error: null,
      }));

      const configHandle = await wasmCore.loadConfig(rhaiSource);

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        config: configHandle,
        loadingState: 'success',
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof WasmError
        ? err.message
        : err instanceof Error
        ? err.message
        : 'Failed to load configuration';

      setState((prev) => ({
        ...prev,
        config: null,
        loadingState: 'error',
        error: message,
      }));
    }
  }, []);

  /**
   * Load a pre-compiled .krx binary.
   */
  const loadKrx = useCallback(async (binary: Uint8Array) => {
    // Validate input
    if (binary.length === 0) {
      setState((prev) => ({
        ...prev,
        error: 'Binary data cannot be empty',
        loadingState: 'error',
      }));
      return;
    }

    if (binary.length > 10 * 1024 * 1024) {
      setState((prev) => ({
        ...prev,
        error: 'Binary data exceeds 10MB limit',
        loadingState: 'error',
      }));
      return;
    }

    try {
      setState((prev) => ({
        ...prev,
        loadingState: 'loading',
        error: null,
      }));

      const configHandle = await wasmCore.loadKrx(binary);

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        config: configHandle,
        loadingState: 'success',
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof WasmError
        ? err.message
        : err instanceof Error
        ? err.message
        : 'Failed to load binary configuration';

      setState((prev) => ({
        ...prev,
        config: null,
        loadingState: 'error',
        error: message,
      }));
    }
  }, []);

  /**
   * Run simulation with event sequence.
   */
  const simulate = useCallback(async (eventSequence: EventSequence) => {
    // Validate config is loaded
    if (!state.config) {
      setState((prev) => ({
        ...prev,
        error: 'No configuration loaded. Load a configuration first.',
        loadingState: 'error',
      }));
      return;
    }

    // Validate event sequence
    if (!eventSequence.events || eventSequence.events.length === 0) {
      setState((prev) => ({
        ...prev,
        error: 'Event sequence cannot be empty',
        loadingState: 'error',
      }));
      return;
    }

    // Validate timestamps are positive and increasing
    for (let i = 0; i < eventSequence.events.length; i++) {
      const event = eventSequence.events[i];
      if (event.timestamp_us < 0) {
        setState((prev) => ({
          ...prev,
          error: `Event ${i + 1} has negative timestamp: ${event.timestamp_us}`,
          loadingState: 'error',
        }));
        return;
      }
      if (i > 0 && event.timestamp_us < eventSequence.events[i - 1].timestamp_us) {
        setState((prev) => ({
          ...prev,
          error: `Event ${i + 1} timestamp is not in ascending order`,
          loadingState: 'error',
        }));
        return;
      }
    }

    try {
      setState((prev) => ({
        ...prev,
        loadingState: 'loading',
        error: null,
      }));

      const result = await wasmCore.simulate(state.config, eventSequence);

      if (!isMountedRef.current) return;

      setState((prev) => ({
        ...prev,
        result,
        loadingState: 'success',
        error: null,
      }));
    } catch (err) {
      if (!isMountedRef.current) return;

      const message = err instanceof WasmError
        ? err.message
        : err instanceof Error
        ? err.message
        : 'Simulation failed';

      setState((prev) => ({
        ...prev,
        result: null,
        loadingState: 'error',
        error: message,
      }));
    }
  }, [state.config]);

  /**
   * Clear error message.
   */
  const clearError = useCallback(() => {
    setState((prev) => ({
      ...prev,
      error: null,
      loadingState: prev.loadingState === 'error' ? 'idle' : prev.loadingState,
    }));
  }, []);

  /**
   * Reset all state.
   */
  const reset = useCallback(() => {
    setState((prev) => ({
      ...prev,
      config: null,
      result: null,
      loadingState: 'idle',
      error: null,
    }));
  }, []);

  return {
    state,
    loadConfig,
    loadKrx,
    simulate,
    clearError,
    reset,
  };
}
