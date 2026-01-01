import { useCallback, useEffect, useState } from 'react';

/**
 * Validation error structure returned by WASM validator
 */
export interface ValidationError {
  line: number;
  column: number;
  length: number;
  message: string;
}

/**
 * Simulation result structure returned by WASM simulator
 */
export interface SimulationResult {
  states: unknown[];
  outputs: unknown[];
  latency: number[];
}

/**
 * Hook for integrating with WASM module for validation and simulation
 *
 * @returns Object containing WASM initialization status and validation/simulation functions
 */
export function useWasm() {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    // Initialize WASM module
    async function initWasm() {
      try {
        // TODO: Implement WASM initialization in Task 16
        // For now, WASM is not available
        console.warn('WASM module not yet implemented. Validation will be unavailable.');
        setIsWasmReady(false);
      } catch (err) {
        console.error('Failed to initialize WASM:', err);
        setError(err instanceof Error ? err : new Error(String(err)));
        setIsWasmReady(false);
      }
    }

    initWasm();
  }, []);

  /**
   * Validate Rhai configuration code
   *
   * @param code - Rhai configuration code to validate
   * @returns Array of validation errors, empty if valid
   */
  const validateConfig = useCallback(
    async (code: string): Promise<ValidationError[]> => {
      if (!isWasmReady) {
        // Return empty array if WASM not ready - graceful degradation
        return [];
      }

      try {
        // TODO: Implement WASM validation in Task 27
        // const result = await wasmModule.validate_config(code);
        // return JSON.parse(result);
        return [];
      } catch (err) {
        console.error('Validation error:', err);
        return [];
      }
    },
    [isWasmReady]
  );

  /**
   * Run simulation with Rhai configuration
   *
   * @param code - Rhai configuration code
   * @param input - Input events for simulation
   * @returns Simulation results
   */
  const runSimulation = useCallback(
    async (code: string, input: unknown): Promise<SimulationResult | null> => {
      if (!isWasmReady) {
        // Return null if WASM not ready - graceful degradation
        return null;
      }

      try {
        // TODO: Implement WASM simulation in Task 28
        // const inputJson = JSON.stringify(input);
        // const result = await wasmModule.simulate(code, inputJson);
        // return JSON.parse(result);
        return null;
      } catch (err) {
        console.error('Simulation error:', err);
        return null;
      }
    },
    [isWasmReady]
  );

  return {
    isWasmReady,
    error,
    validateConfig,
    runSimulation,
  };
}
