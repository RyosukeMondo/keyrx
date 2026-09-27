import { useCallback, useEffect, useState } from 'react';
import { logger } from '@/utils/logger';

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
      logger.debug('wasm_init_started');
      setIsLoading(true);

      let lastError: Error | null = null;

      for (let attempt = 1; attempt <= RETRY_CONFIG.maxAttempts; attempt++) {
        try {
          if (attempt > 1) {
            logger.debug('wasm_init_retry', {
              attempt,
              maxAttempts: RETRY_CONFIG.maxAttempts,
            });
            await sleep(RETRY_CONFIG.delayMs);
          }

          // Try to dynamically import the WASM module
          logger.debug('wasm_module_fetch_started');
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

          logger.debug('wasm_module_loaded');
          // For wasm-pack web target, must call default init() first to load WASM binary
          if (module.default && typeof module.default === 'function') {
            await module.default();
            logger.debug('wasm_binary_loaded');
          }
          // Initialize WASM with panic hook
          module.wasm_init();

          const loadTime = performance.now() - startTime;
          setWasmModule(module as unknown as WasmModule);
          setIsWasmReady(true);
          setIsLoading(false);
          setError(null);
          logger.debug('wasm_init_succeeded', {
            loadTimeMs: Math.round(loadTime),
            attempt,
          });
          return; // Success - exit the retry loop
        } catch (err) {
          lastError = err instanceof Error ? err : new Error(String(err));
          const loadTime = performance.now() - startTime;

          if (attempt < RETRY_CONFIG.maxAttempts) {
            logger.warn('wasm_init_attempt_failed', {
              attempt,
              maxAttempts: RETRY_CONFIG.maxAttempts,
              loadTimeMs: Math.round(loadTime),
              error: lastError.message,
              retryDelayMs: RETRY_CONFIG.delayMs,
            });
          } else {
            logger.error('wasm_init_failed', lastError, {
              attempts: RETRY_CONFIG.maxAttempts,
              loadTimeMs: Math.round(loadTime),
            });
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
        logger.debug('wasm_validation_skipped', {
          reason: isLoading
            ? 'loading'
            : error
              ? error.message
              : 'not_initialized',
        });
        return [];
      }

      try {
        // Use load_config to validate - it will throw if invalid
        wasmModule.load_config(code);
        // If we get here, the config is valid
        logger.debug('wasm_validation_passed');
        return [];
      } catch (err) {
        // Parse error message to extract line/column information
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.debug('wasm_validation_error', { error: errorMessage });

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
        logger.debug('wasm_simulation_skipped', {
          reason: isLoading
            ? 'loading'
            : error
              ? error.message
              : 'not_initialized',
        });
        return null;
      }

      try {
        // Load the configuration
        logger.debug('wasm_simulation_config_load_started');
        const configHandle = wasmModule.load_config(code);

        // Run simulation
        logger.debug('wasm_simulation_started', {
          eventCount: input.events.length,
        });
        const inputJson = JSON.stringify(input);
        const result = wasmModule.simulate(configHandle, inputJson);

        logger.debug('wasm_simulation_succeeded');
        // Parse and return the result
        return result as SimulationResult;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.error('wasm_simulation_failed', undefined, {
          error: errorMessage,
        });
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
