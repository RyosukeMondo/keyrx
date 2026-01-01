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
  states: StateTransition[];
  outputs: KeyEvent[];
  latency: number[];
  final_state: {
    active_modifiers: number[];
    active_locks: number[];
    active_layer: string | null;
  };
}

interface StateTransition {
  timestamp_us: number;
  active_modifiers: number[];
  active_locks: number[];
  active_layer: string | null;
}

interface KeyEvent {
  keycode: string;
  event_type: 'press' | 'release';
  timestamp_us: number;
}

/**
 * Input event for simulation
 */
export interface SimulationInput {
  events: Array<{
    keycode: string;
    event_type: 'press' | 'release';
    timestamp_us: number;
  }>;
}

// Type definitions for WASM module
interface WasmModule {
  wasm_init: () => void;
  load_config: (source: string) => number; // Returns ConfigHandle
  simulate: (configHandle: number, eventsJson: string) => unknown;
}

/**
 * Hook for integrating with WASM module for validation and simulation
 *
 * @returns Object containing WASM initialization status and validation/simulation functions
 */
export function useWasm() {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [wasmModule, setWasmModule] = useState<WasmModule | null>(null);

  useEffect(() => {
    // Initialize WASM module
    async function initWasm() {
      try {
        // Try to dynamically import the WASM module
        // The path will be correct once WASM is built (Task 25-26)
        // @ts-expect-error - WASM module doesn't exist until build:wasm runs
        const module = await import('@/wasm/pkg/keyrx_core.js').catch(() => {
          throw new Error('WASM module not found. Run build:wasm to compile the WASM module.');
        });

        // Initialize WASM with panic hook
        module.wasm_init();

        setWasmModule(module as unknown as WasmModule);
        setIsWasmReady(true);
        console.info('WASM module initialized successfully');
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.warn('WASM module not available:', errorMessage);
        setError(err instanceof Error ? err : new Error(errorMessage));
        setIsWasmReady(false);
      }
    }

    initWasm();
  }, []);

  /**
   * Validate Rhai configuration code
   *
   * Uses the WASM load_config function which validates and parses the configuration.
   * If parsing fails, it returns validation errors with line/column information.
   *
   * @param code - Rhai configuration code to validate
   * @returns Array of validation errors, empty if valid
   */
  const validateConfig = useCallback(
    async (code: string): Promise<ValidationError[]> => {
      if (!isWasmReady || !wasmModule) {
        // Return empty array if WASM not ready - graceful degradation
        console.debug('WASM not ready, skipping validation');
        return [];
      }

      try {
        // Use load_config to validate - it will throw if invalid
        wasmModule.load_config(code);
        // If we get here, the config is valid
        return [];
      } catch (err) {
        // Parse error message to extract line/column information
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.debug('Validation error:', errorMessage);

        // Try to extract line number from error message
        // Rhai errors typically include line information
        const lineMatch = errorMessage.match(/line (\d+)/i);
        const columnMatch = errorMessage.match(/column (\d+)/i);

        const line = lineMatch ? parseInt(lineMatch[1], 10) : 1;
        const column = columnMatch ? parseInt(columnMatch[1], 10) : 1;

        return [
          {
            line,
            column,
            length: 1,
            message: errorMessage,
          },
        ];
      }
    },
    [isWasmReady, wasmModule]
  );

  /**
   * Run simulation with Rhai configuration
   *
   * @param code - Rhai configuration code
   * @param input - Input events for simulation
   * @returns Simulation results
   */
  const runSimulation = useCallback(
    async (code: string, input: SimulationInput): Promise<SimulationResult | null> => {
      if (!isWasmReady || !wasmModule) {
        // Return null if WASM not ready - graceful degradation
        console.debug('WASM not ready, skipping simulation');
        return null;
      }

      try {
        // Load the configuration
        const configHandle = wasmModule.load_config(code);

        // Run simulation
        const inputJson = JSON.stringify(input);
        const result = wasmModule.simulate(configHandle, inputJson);

        // Parse and return the result
        return result as SimulationResult;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.error('Simulation error:', errorMessage);
        return null;
      }
    },
    [isWasmReady, wasmModule]
  );

  return {
    isWasmReady,
    error,
    validateConfig,
    runSimulation,
  };
}
