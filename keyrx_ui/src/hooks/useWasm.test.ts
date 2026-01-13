/**
 * Tests for useWasm hook
 *
 * Tests the WASM integration hook that handles module loading, validation, and simulation.
 * Covers initialization, retry logic, validation, and simulation scenarios.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useWasm } from './useWasm';

// Mock the WASM module
const mockWasmInit = vi.fn();
const mockLoadConfig = vi.fn();
const mockSimulate = vi.fn();
const mockValidateConfig = vi.fn();

const mockWasmModule = {
  wasm_init: mockWasmInit,
  load_config: mockLoadConfig,
  simulate: mockSimulate,
  validate_config: mockValidateConfig,
};

// Mock the dynamic import
vi.mock('@/wasm/pkg/keyrx_core.js', () => ({
  default: mockWasmModule,
  wasm_init: mockWasmInit,
  load_config: mockLoadConfig,
  simulate: mockSimulate,
  validate_config: mockValidateConfig,
}));

describe('useWasm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset timers for retry logic testing
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Initialization', () => {
    it('initializes WASM module successfully', async () => {
      mockWasmInit.mockImplementation(() => {});

      const { result } = renderHook(() => useWasm());

      // Initially loading
      expect(result.current.isLoading).toBe(true);
      expect(result.current.isWasmReady).toBe(false);
      expect(result.current.error).toBe(null);

      // Wait for initialization to complete
      await waitFor(
        () => {
          expect(result.current.isWasmReady).toBe(true);
        },
        { timeout: 5000 }
      );

      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe(null);
      expect(mockWasmInit).toHaveBeenCalledTimes(1);
    });

    it('sets error state when WASM module fails to load', async () => {
      // Mock import failure
      vi.doMock('@/wasm/pkg/keyrx_core.js', () => {
        throw new Error('Module not found');
      });

      const { result } = renderHook(() => useWasm());

      // Wait for all retry attempts to complete
      await vi.advanceTimersByTimeAsync(10000);

      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 15000 }
      );

      expect(result.current.isWasmReady).toBe(false);
      expect(result.current.error).not.toBe(null);
      expect(result.current.error?.message).toContain('WASM module not found');
    });

    it('retries initialization on failure', async () => {
      let attemptCount = 0;
      mockWasmInit.mockImplementation(() => {
        attemptCount++;
        if (attemptCount < 2) {
          throw new Error('Initialization failed');
        }
        // Succeed on second attempt
      });

      const { result } = renderHook(() => useWasm());

      // Advance time for first retry
      await vi.advanceTimersByTimeAsync(1000);

      // Wait for successful initialization
      await waitFor(
        () => {
          expect(result.current.isWasmReady).toBe(true);
        },
        { timeout: 10000 }
      );

      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe(null);
      expect(attemptCount).toBe(2);
    });

    it('fails after max retry attempts', async () => {
      mockWasmInit.mockImplementation(() => {
        throw new Error('Persistent initialization failure');
      });

      const { result } = renderHook(() => useWasm());

      // Advance time through all retry attempts
      for (let i = 0; i < 3; i++) {
        await vi.advanceTimersByTimeAsync(1000);
      }

      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 15000 }
      );

      expect(result.current.isWasmReady).toBe(false);
      expect(result.current.error).not.toBe(null);
      expect(result.current.error?.message).toContain('Persistent initialization failure');
    });
  });

  describe('validateConfig', () => {
    it('validates correct Rhai configuration', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1); // Return config handle

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('map("A", "B");');

      expect(errors).toEqual([]);
      expect(mockLoadConfig).toHaveBeenCalledWith('map("A", "B");');
    });

    it('returns validation errors for invalid configuration', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation((code: string) => {
        throw new Error('Syntax error at line 5 column 10');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('invalid code');

      expect(errors).toHaveLength(1);
      expect(errors[0]).toMatchObject({
        line: 5,
        column: 10,
        message: expect.stringContaining('Syntax error'),
      });
    });

    it('returns empty array when WASM is not ready', async () => {
      // Don't initialize WASM
      mockWasmInit.mockImplementation(() => {
        throw new Error('Not initialized');
      });

      const { result } = renderHook(() => useWasm());

      // Advance timers to fail initialization
      await vi.advanceTimersByTimeAsync(10000);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      const errors = await result.current.validateConfig('map("A", "B");');

      expect(errors).toEqual([]);
      expect(mockLoadConfig).not.toHaveBeenCalled();
    });

    it('extracts line and column from error message', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation(() => {
        throw new Error('Parse error at line 12 column 5: unexpected token');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('bad syntax');

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(12);
      expect(errors[0].column).toBe(5);
    });

    it('defaults to line 1, column 1 when no position in error', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation(() => {
        throw new Error('Generic validation error');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('bad syntax');

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(1);
      expect(errors[0].column).toBe(1);
    });
  });

  describe('runSimulation', () => {
    it('runs simulation with valid configuration', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1); // Return config handle
      mockSimulate.mockReturnValue({
        states: [
          {
            timestamp_us: 1000,
            active_modifiers: [],
            active_locks: [],
            active_layer: 'base',
          },
        ],
        outputs: [
          {
            keycode: 'B',
            event_type: 'press',
            timestamp_us: 1500,
          },
        ],
        latency: [500],
        final_state: {
          active_modifiers: [],
          active_locks: [],
          active_layer: 'base',
        },
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
        ],
      };

      const simulationResult = await result.current.runSimulation('map("A", "B");', simulationInput);

      expect(simulationResult).not.toBe(null);
      expect(simulationResult?.outputs).toHaveLength(1);
      expect(simulationResult?.outputs[0].keycode).toBe('B');
      expect(mockLoadConfig).toHaveBeenCalledWith('map("A", "B");');
      expect(mockSimulate).toHaveBeenCalled();
    });

    it('returns null when WASM is not ready', async () => {
      // Don't initialize WASM
      mockWasmInit.mockImplementation(() => {
        throw new Error('Not initialized');
      });

      const { result } = renderHook(() => useWasm());

      // Advance timers to fail initialization
      await vi.advanceTimersByTimeAsync(10000);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
        ],
      };

      const simulationResult = await result.current.runSimulation('map("A", "B");', simulationInput);

      expect(simulationResult).toBe(null);
      expect(mockLoadConfig).not.toHaveBeenCalled();
      expect(mockSimulate).not.toHaveBeenCalled();
    });

    it('returns null when simulation fails', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1);
      mockSimulate.mockImplementation(() => {
        throw new Error('Simulation error');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
        ],
      };

      const simulationResult = await result.current.runSimulation('map("A", "B");', simulationInput);

      expect(simulationResult).toBe(null);
    });

    it('returns null when config loading fails during simulation', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation(() => {
        throw new Error('Invalid configuration');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
        ],
      };

      const simulationResult = await result.current.runSimulation('invalid code', simulationInput);

      expect(simulationResult).toBe(null);
      expect(mockSimulate).not.toHaveBeenCalled();
    });

    it('serializes input events as JSON', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1);
      mockSimulate.mockReturnValue({
        states: [],
        outputs: [],
        latency: [],
        final_state: {
          active_modifiers: [],
          active_locks: [],
          active_layer: null,
        },
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
          {
            keycode: 'A',
            event_type: 'release' as const,
            timestamp_us: 2000,
          },
        ],
      };

      await result.current.runSimulation('map("A", "B");', simulationInput);

      expect(mockSimulate).toHaveBeenCalledWith(1, JSON.stringify(simulationInput));
    });
  });

  describe('Error Handling', () => {
    it('handles non-Error objects thrown during initialization', async () => {
      mockWasmInit.mockImplementation(() => {
        throw 'String error';
      });

      const { result } = renderHook(() => useWasm());

      // Advance time through all retry attempts
      await vi.advanceTimersByTimeAsync(10000);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error).not.toBe(null);
      expect(result.current.error?.message).toContain('String error');
    });

    it('handles non-Error objects thrown during validation', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation(() => {
        throw 'Validation string error';
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('bad code');

      expect(errors).toHaveLength(1);
      expect(errors[0].message).toContain('Validation string error');
    });

    it('handles non-Error objects thrown during simulation', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1);
      mockSimulate.mockImplementation(() => {
        throw 'Simulation string error';
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simulationInput = {
        events: [
          {
            keycode: 'A',
            event_type: 'press' as const,
            timestamp_us: 1000,
          },
        ],
      };

      const result2 = await result.current.runSimulation('map("A", "B");', simulationInput);

      expect(result2).toBe(null);
    });
  });

  describe('Edge Cases', () => {
    it('handles empty configuration string', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockImplementation((code: string) => {
        if (!code) {
          throw new Error('Empty configuration');
        }
        return 1;
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('');

      expect(errors).toHaveLength(1);
      expect(errors[0].message).toContain('Empty configuration');
    });

    it('handles empty simulation input', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1);
      mockSimulate.mockReturnValue({
        states: [],
        outputs: [],
        latency: [],
        final_state: {
          active_modifiers: [],
          active_locks: [],
          active_layer: null,
        },
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const emptyInput = {
        events: [],
      };

      const simulationResult = await result.current.runSimulation('map("A", "B");', emptyInput);

      expect(simulationResult).not.toBe(null);
      expect(simulationResult?.outputs).toEqual([]);
    });

    it('handles very long configuration strings', async () => {
      mockWasmInit.mockImplementation(() => {});
      mockLoadConfig.mockReturnValue(1);

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const longConfig = 'map("A", "B");'.repeat(1000);
      const errors = await result.current.validateConfig(longConfig);

      expect(errors).toEqual([]);
      expect(mockLoadConfig).toHaveBeenCalledWith(longConfig);
    });
  });
});
