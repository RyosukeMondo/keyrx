import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
} from 'react';
import type {
  ValidationError,
  SimulationResult,
  SimulationInput,
} from '../hooks/useWasm';

// Type definitions for WASM module
interface WasmModule {
  wasm_init: () => void;
  load_config: (source: string) => number; // Returns ConfigHandle
  simulate: (configHandle: number, eventsJson: string) => unknown;
}

interface WasmContextValue {
  isWasmReady: boolean;
  isLoading: boolean;
  error: Error | null;
  validateConfig: (code: string) => Promise<ValidationError[]>;
  runSimulation: (
    code: string,
    input: SimulationInput
  ) => Promise<SimulationResult | null>;
}

const WasmContext = createContext<WasmContextValue | null>(null);

/**
 * Global WASM Provider - initializes WASM once at app startup
 * Prevents re-initialization on page navigation
 */
export function WasmProvider({ children }: { children: React.ReactNode }) {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [wasmModule, setWasmModule] = useState<WasmModule | null>(null);

  useEffect(() => {
    // Initialize WASM module ONCE for entire app lifecycle
    async function initWasm() {
      const startTime = performance.now();
      console.info('[WASM] Starting global initialization...');
      setIsLoading(true);

      try {
        console.info('[WASM] Fetching module...');
        const module = await import('@/wasm/pkg/keyrx_core.js').catch(() => {
          throw new Error(
            'WASM module not found. Run build:wasm to compile the WASM module.'
          );
        });

        console.info('[WASM] Module loaded, initializing...');
        module.wasm_init();

        const loadTime = performance.now() - startTime;
        setWasmModule(module as unknown as WasmModule);
        setIsWasmReady(true);
        setIsLoading(false);
        console.info(
          `[WASM] ✓ Global initialization complete in ${loadTime.toFixed(0)}ms`
        );
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        const loadTime = performance.now() - startTime;
        console.warn(
          `[WASM] Initialization failed after ${loadTime.toFixed(0)}ms:`,
          errorMessage
        );
        setError(err instanceof Error ? err : new Error(errorMessage));
        setIsWasmReady(false);
        setIsLoading(false);
      }
    }

    initWasm();
  }, []); // Empty deps = initialize once on app mount

  const validateConfig = useCallback(
    async (code: string): Promise<ValidationError[]> => {
      if (!isWasmReady || !wasmModule) {
        console.debug('WASM not ready, skipping validation');
        return [];
      }

      try {
        wasmModule.load_config(code);
        return [];
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.debug('Validation error:', errorMessage);

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

  const runSimulation = useCallback(
    async (
      code: string,
      input: SimulationInput
    ): Promise<SimulationResult | null> => {
      if (!isWasmReady || !wasmModule) {
        console.debug('WASM not ready, skipping simulation');
        return null;
      }

      try {
        const configHandle = wasmModule.load_config(code);
        const inputJson = JSON.stringify(input);
        const result = wasmModule.simulate(configHandle, inputJson);
        return result as SimulationResult;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        console.error('Simulation error:', errorMessage);
        return null;
      }
    },
    [isWasmReady, wasmModule]
  );

  const value: WasmContextValue = {
    isWasmReady,
    isLoading,
    error,
    validateConfig,
    runSimulation,
  };

  return <WasmContext.Provider value={value}>{children}</WasmContext.Provider>;
}

/**
 * Hook to access WASM context
 * Use this instead of useWasm() for cached WASM access
 */
export function useWasmContext() {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error('useWasmContext must be used within WasmProvider');
  }
  return context;
}
