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
  validate_config: (source: string) => void; // Throws on validation errors
}

// Configuration for retry logic
const RETRY_CONFIG = {
  maxAttempts: 3,
  delayMs: 1000,
};

// Helper function to sleep for a specified duration
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Hook for integrating with WASM module for validation and simulation
 *
 * @returns Object containing WASM initialization status and validation/simulation functions
 */
export function useWasm() {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [wasmModule, setWasmModule] = useState<WasmModule | null>(null);

  useEffect(() => {
    // Initialize WASM module with retry logic
    async function initWasm() {
      const startTime = performance.now();
      if (import.meta.env.DEV) {
        // eslint-disable-next-line no-console
        console.info('[WASM] Starting initialization...');
      }
      setIsLoading(true);

      let lastError: Error | null = null;

      for (let attempt = 1; attempt <= RETRY_CONFIG.maxAttempts; attempt++) {
        try {
          if (attempt > 1) {
            if (import.meta.env.DEV) {
              // eslint-disable-next-line no-console
              console.info(
                `[WASM] Retry attempt ${attempt}/${RETRY_CONFIG.maxAttempts}...`
              );
            }
            await sleep(RETRY_CONFIG.delayMs);
          }

          // Try to dynamically import the WASM module
          if (import.meta.env.DEV) {
            // eslint-disable-next-line no-console
            console.info('[WASM] Fetching module...');
          }
          const module = await import('@/wasm/pkg/keyrx_core.js').catch(
            (importErr) => {
              throw new Error(
                `WASM module not found at @/wasm/pkg/keyrx_core.js. ` +
                  `Run 'npm run build:wasm' to compile the WASM module. ` +
                  `Import error: ${
                    importErr instanceof Error
                      ? importErr.message
                      : String(importErr)
                  }`
              );
            }
          );

          if (import.meta.env.DEV) {
            // eslint-disable-next-line no-console
            console.info('[WASM] Module loaded, initializing WASM binary...');
          }
          // For wasm-pack web target, must call default init() first to load WASM binary
          if (module.default && typeof module.default === 'function') {
            await module.default();
            if (import.meta.env.DEV) {
              // eslint-disable-next-line no-console
              console.info('[WASM] Binary loaded, setting up panic hook...');
            }
          }
          // Initialize WASM with panic hook
          module.wasm_init();

          const loadTime = performance.now() - startTime;
          setWasmModule(module as unknown as WasmModule);
          setIsWasmReady(true);
          setIsLoading(false);
          setError(null);
          if (import.meta.env.DEV) {
            // eslint-disable-next-line no-console
            console.info(
              `[WASM] Initialized successfully in ${loadTime.toFixed(0)}ms` +
                (attempt > 1 ? ` (succeeded on attempt ${attempt})` : '')
            );
          }
          return; // Success - exit the retry loop
        } catch (err) {
          lastError = err instanceof Error ? err : new Error(String(err));
          const loadTime = performance.now() - startTime;

          if (attempt < RETRY_CONFIG.maxAttempts) {
            console.warn(
              `[WASM] Attempt ${attempt}/${
                RETRY_CONFIG.maxAttempts
              } failed after ${loadTime.toFixed(0)}ms:`,
              lastError.message,
              `- Retrying in ${RETRY_CONFIG.delayMs}ms...`
            );
          } else {
            console.error(
              `[WASM] All ${
                RETRY_CONFIG.maxAttempts
              } initialization attempts failed after ${loadTime.toFixed(0)}ms:`,
              lastError.message
            );
          }
        }
      }

      // All attempts failed
      if (lastError) {
        setError(lastError);
        setIsWasmReady(false);
        setIsLoading(false);
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
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug(
            '[WASM] Validation skipped: WASM not ready.',
            isLoading
              ? 'Still loading...'
              : error
                ? `Error: ${error.message}`
                : 'Not initialized'
          );
        }
        return [];
      }

      try {
        // Use load_config to validate - it will throw if invalid
        wasmModule.load_config(code);
        // If we get here, the config is valid
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug('[WASM] Validation passed');
        }
        return [];
      } catch (err) {
        // Parse error message to extract line/column information
        const errorMessage = err instanceof Error ? err.message : String(err);
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug('[WASM] Validation error:', errorMessage);
        }

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
    [isWasmReady, wasmModule, isLoading, error]
  );

  /**
   * Run simulation with Rhai configuration
   *
   * @param code - Rhai configuration code
   * @param input - Input events for simulation
   * @returns Simulation results or null if WASM not ready or simulation fails
   */
  const runSimulation = useCallback(
    async (
      code: string,
      input: SimulationInput
    ): Promise<SimulationResult | null> => {
      if (!isWasmReady || !wasmModule) {
        // Return null if WASM not ready - graceful degradation
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug(
            '[WASM] Simulation skipped: WASM not ready.',
            isLoading
              ? 'Still loading...'
              : error
                ? `Error: ${error.message}`
                : 'Not initialized'
          );
        }
        return null;
      }

      try {
        // Load the configuration
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug('[WASM] Loading config for simulation...');
        }
        const configHandle = wasmModule.load_config(code);

        // Run simulation
        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug(
            '[WASM] Running simulation with',
            input.events.length,
            'events...'
          );
        }
        const inputJson = JSON.stringify(input);
        const result = wasmModule.simulate(configHandle, inputJson);

        if (import.meta.env.DEV) {
          // eslint-disable-next-line no-console
          console.debug('[WASM] Simulation completed successfully');
        }
        // Parse and return the result
        return result as SimulationResult;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.error('[WASM] Simulation error:', errorMessage);
        return null;
      }
    },
    [isWasmReady, wasmModule, isLoading, error]
  );

  return {
    isWasmReady,
    isLoading,
    error,
    validateConfig,
    runSimulation,
  };
}
