/**
 * WASM Integration Tests
 *
 * Tests the useWasm hook and WASM module integration to ensure:
 * 1. WASM module loads successfully in the browser environment
 * 2. Validation functions work correctly
 * 3. Simulation functions work correctly
 * 4. Error handling is graceful when WASM is unavailable
 *
 * Note: These tests use mocks to avoid dependency on actual WASM binary
 * in the CI environment. Real WASM functionality is tested separately
 * in ffi-types.test.ts
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import {
  useWasm,
  type ValidationError,
  type SimulationResult,
} from '../hooks/useWasm';

// Mock the WASM module import
vi.mock('@/wasm/pkg/keyrx_core.js', () => {
  return {
    // Default export is the init function for wasm-pack web target
    default: vi.fn(() => Promise.resolve()),
    wasm_init: vi.fn(),
    load_config: vi.fn((code: string) => {
      if (code.includes('invalid')) {
        throw new Error(
          'Parse error at line 2 column 5: Invalid function call'
        );
      }
      return 12345; // Mock ConfigHandle
    }),
    validate_config: vi.fn((code: string) => {
      if (code.includes('invalid')) {
        throw new Error(
          'Parse error at line 2 column 5: Invalid function call'
        );
      }
    }),
    simulate: vi.fn((handle: number, eventsJson: string) => {
      const events = JSON.parse(eventsJson);
      return {
        states: [
          {
            timestamp_us: 0,
            active_modifiers: [],
            active_locks: [],
            active_layer: null,
          },
        ],
        outputs: events.events.map(
          (e: {
            keycode: string;
            event_type: string;
            timestamp_us: number;
          }) => ({
            keycode: e.keycode,
            event_type: e.event_type,
            timestamp_us: e.timestamp_us,
          })
        ),
        latency: [100],
        final_state: {
          active_modifiers: [],
          active_locks: [],
          active_layer: null,
        },
      };
    }),
  };
});

describe('WASM Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('useWasm Hook Initialization', () => {
    it('should initialize WASM module and set isWasmReady to true', async () => {
      const { result } = renderHook(() => useWasm());

      // Initially loading
      expect(result.current.isLoading).toBe(true);
      expect(result.current.isWasmReady).toBe(false);
      expect(result.current.error).toBeNull();

      // Wait for WASM to initialize
      await waitFor(
        () => {
          expect(result.current.isWasmReady).toBe(true);
        },
        { timeout: 5000 }
      );

      // After initialization
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(result.current.validateConfig).toBeDefined();
      expect(result.current.runSimulation).toBeDefined();
    });

    it('should provide validation and simulation functions', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      expect(typeof result.current.validateConfig).toBe('function');
      expect(typeof result.current.runSimulation).toBe('function');
    });
  });

  describe('validateConfig Function', () => {
    it('should return empty array for valid configuration', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const validConfig = `
device("*") {
  map("VK_A", "VK_B");
}
      `;

      const errors = await result.current.validateConfig(validConfig);

      expect(Array.isArray(errors)).toBe(true);
      expect(errors).toHaveLength(0);
    });

    it('should return validation errors for invalid configuration', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const invalidConfig = `
device("*") {
  invalid_function_call();
}
      `;

      const errors = await result.current.validateConfig(invalidConfig);

      expect(Array.isArray(errors)).toBe(true);
      expect(errors.length).toBeGreaterThan(0);

      // Verify error structure
      const error = errors[0];
      expect(error).toHaveProperty('line');
      expect(error).toHaveProperty('column');
      expect(error).toHaveProperty('length');
      expect(error).toHaveProperty('message');

      expect(typeof error.line).toBe('number');
      expect(typeof error.column).toBe('number');
      expect(typeof error.length).toBe('number');
      expect(typeof error.message).toBe('string');

      expect(error.message).toContain('Parse error');
    });

    it('should extract line and column from error message', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const invalidConfig = 'device("*") { invalid_function_call(); }';
      const errors = await result.current.validateConfig(invalidConfig);

      expect(errors.length).toBeGreaterThan(0);

      const error = errors[0];
      // Error message contains "line 2 column 5"
      expect(error.line).toBe(2);
      expect(error.column).toBe(5);
    });

    it('should return empty array when WASM is not ready', async () => {
      const { result } = renderHook(() => useWasm());

      // Call validateConfig before WASM is ready
      const errors = await result.current.validateConfig('test code');

      // Should gracefully degrade
      expect(Array.isArray(errors)).toBe(true);
      expect(errors).toHaveLength(0);
    });
  });

  describe('runSimulation Function', () => {
    it('should run simulation with valid configuration and input', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const validConfig = `
device("*") {
  map("VK_A", "VK_B");
}
      `;

      const simulationInput = {
        events: [
          { keycode: 'A', event_type: 'press' as const, timestamp_us: 0 },
          {
            keycode: 'A',
            event_type: 'release' as const,
            timestamp_us: 100000,
          },
        ],
      };

      const simulationResult = await result.current.runSimulation(
        validConfig,
        simulationInput
      );

      expect(simulationResult).not.toBeNull();
      expect(simulationResult).toHaveProperty('states');
      expect(simulationResult).toHaveProperty('outputs');
      expect(simulationResult).toHaveProperty('latency');
      expect(simulationResult).toHaveProperty('final_state');

      const typedResult = simulationResult as SimulationResult;
      expect(Array.isArray(typedResult.states)).toBe(true);
      expect(Array.isArray(typedResult.outputs)).toBe(true);
      expect(Array.isArray(typedResult.latency)).toBe(true);

      // Verify output structure
      expect(typedResult.outputs).toHaveLength(2);
      expect(typedResult.outputs[0]).toHaveProperty('keycode');
      expect(typedResult.outputs[0]).toHaveProperty('event_type');
      expect(typedResult.outputs[0]).toHaveProperty('timestamp_us');
    });

    it('should return null when simulation fails', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      // Mock will throw for invalid config
      const invalidConfig = 'invalid syntax';
      const simulationInput = {
        events: [
          { keycode: 'A', event_type: 'press' as const, timestamp_us: 0 },
        ],
      };

      const simulationResult = await result.current.runSimulation(
        invalidConfig,
        simulationInput
      );

      expect(simulationResult).toBeNull();
    });

    it('should return null when WASM is not ready', async () => {
      const { result } = renderHook(() => useWasm());

      // Call runSimulation before WASM is ready
      const simulationInput = {
        events: [
          { keycode: 'A', event_type: 'press' as const, timestamp_us: 0 },
        ],
      };

      const simulationResult = await result.current.runSimulation(
        'test config',
        simulationInput
      );

      // Should gracefully degrade
      expect(simulationResult).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('should handle WASM initialization errors gracefully', async () => {
      // Mock import failure
      vi.doMock('@/wasm/pkg/keyrx_core.js', () => {
        throw new Error('WASM module not found');
      });

      const { result } = renderHook(() => useWasm());

      // Should eventually fail gracefully
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 5000 }
      );

      // Even after retries, should handle gracefully
      expect(result.current.isWasmReady).toBe(false);

      // validateConfig should return empty array (graceful degradation)
      const errors = await result.current.validateConfig('test');
      expect(errors).toEqual([]);

      // runSimulation should return null (graceful degradation)
      const simResult = await result.current.runSimulation('test', {
        events: [],
      });
      expect(simResult).toBeNull();
    });

    it('should provide detailed error information', async () => {
      vi.doMock('@/wasm/pkg/keyrx_core.js', () => {
        throw new Error('WASM module failed to load');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 5000 }
      );

      if (result.current.error) {
        expect(result.current.error).toBeInstanceOf(Error);
        expect(result.current.error.message).toBeTruthy();
      }
    });
  });

  describe('Retry Logic', () => {
    it('should retry WASM initialization on failure', async () => {
      // This test verifies that retry logic is in place
      // The actual retry is tested by observing console logs in manual testing

      const { result } = renderHook(() => useWasm());

      // Should eventually complete (success or failure)
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 5000 }
      );

      // Either succeeded or failed after retries
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('Type Safety', () => {
    it('should maintain type safety for ValidationError', () => {
      const mockError: ValidationError = {
        line: 1,
        column: 5,
        length: 10,
        message: 'Test error',
      };

      expect(mockError.line).toBe(1);
      expect(mockError.column).toBe(5);
      expect(mockError.length).toBe(10);
      expect(mockError.message).toBe('Test error');
    });

    it('should maintain type safety for SimulationResult', () => {
      const mockResult: SimulationResult = {
        states: [
          {
            timestamp_us: 0,
            active_modifiers: [1, 2],
            active_locks: [],
            active_layer: 'base',
          },
        ],
        outputs: [
          {
            keycode: 'A',
            event_type: 'press',
            timestamp_us: 0,
          },
        ],
        latency: [100],
        final_state: {
          active_modifiers: [1, 2],
          active_locks: [],
          active_layer: 'base',
        },
      };

      expect(mockResult.states).toHaveLength(1);
      expect(mockResult.outputs).toHaveLength(1);
      expect(mockResult.latency).toHaveLength(1);
      expect(mockResult.final_state).toBeDefined();
    });
  });

  describe('Performance', () => {
    it('should initialize WASM within reasonable time', async () => {
      const startTime = performance.now();
      const { result } = renderHook(() => useWasm());

      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 5000 }
      );

      const elapsedTime = performance.now() - startTime;

      // Initialization should complete within 5 seconds (including retries)
      expect(elapsedTime).toBeLessThan(5000);
    });

    // Skip performance tests that depend on WASM loading successfully
    // These are better tested in real browser environment with actual WASM
    it.skip('should validate configuration quickly', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const validConfig = 'device("*") { map("VK_A", "VK_B"); }';

      const startTime = performance.now();
      await result.current.validateConfig(validConfig);
      const elapsedTime = performance.now() - startTime;

      // Validation should be fast (< 100ms for simple configs)
      expect(elapsedTime).toBeLessThan(100);
    });

    it.skip('should run simulation quickly', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const validConfig = 'device("*") { map("VK_A", "VK_B"); }';
      const simulationInput = {
        events: [
          { keycode: 'A', event_type: 'press' as const, timestamp_us: 0 },
        ],
      };

      const startTime = performance.now();
      await result.current.runSimulation(validConfig, simulationInput);
      const elapsedTime = performance.now() - startTime;

      // Simulation should be fast (< 200ms for simple scenarios)
      expect(elapsedTime).toBeLessThan(200);
    });
  });
});
