/**
 * Tests for useWasm hook
 *
 * Tests the WASM integration hook that handles module loading, validation, and simulation.
 * Covers initialization, retry logic, validation, and simulation scenarios.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useWasm } from './useWasm';

// Create mock functions that we can reference and configure
const mockDefaultInit = vi.fn(() => Promise.resolve());
const mockWasmInit = vi.fn();
const mockLoadConfig = vi.fn();
const mockSimulate = vi.fn();
const mockValidateConfig = vi.fn();

// Mock the WASM module with our controllable mock functions
vi.mock('@/wasm/pkg/keyrx_core.js', () => ({
  default: () => mockDefaultInit(),
  wasm_init: (...args: unknown[]) => mockWasmInit(...args),
  load_config: (...args: unknown[]) => mockLoadConfig(...args),
  simulate: (...args: unknown[]) => mockSimulate(...args),
  validate_config: (...args: unknown[]) => mockValidateConfig(...args),
}));

describe('useWasm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
    // Reset mocks to default behavior
    mockDefaultInit.mockResolvedValue(undefined);
    mockWasmInit.mockImplementation(() => {});
    mockLoadConfig.mockReturnValue(12345);
    mockSimulate.mockReturnValue({
      states: [],
      outputs: [],
      latency: [],
      final_state: { active_modifiers: [], active_locks: [], active_layer: null },
    });
    mockValidateConfig.mockImplementation(() => {});
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Initialization', () => {
    it('initializes WASM module successfully', async () => {
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

    it('sets error state when WASM initialization fails', async () => {
      mockWasmInit.mockImplementation(() => {
        throw new Error('Initialization failed');
      });

      const { result } = renderHook(() => useWasm());

      // Wait for all retry attempts to fail (3 attempts * 1s delay = ~3s minimum)
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 10000 }
      );

      expect(result.current.isWasmReady).toBe(false);
      expect(result.current.error).not.toBe(null);
      expect(result.current.error?.message).toContain('Initialization failed');
    });

    it('retries initialization on transient failure', async () => {
      let attemptCount = 0;
      mockWasmInit.mockImplementation(() => {
        attemptCount++;
        if (attemptCount < 2) {
          throw new Error('Transient failure');
        }
        // Succeed on second attempt
      });

      const { result } = renderHook(() => useWasm());

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
  });

  describe('validateConfig', () => {
    it('returns empty array for valid configuration', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('map("A", "B");');
      expect(errors).toEqual([]);
      expect(mockLoadConfig).toHaveBeenCalledWith('map("A", "B");');
    });

    it('returns validation errors for invalid configuration', async () => {
      mockLoadConfig.mockImplementation(() => {
        throw new Error('Parse error at line 2, column 5: Unexpected token');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('invalid syntax');
      expect(errors).toHaveLength(1);
      expect(errors[0].message).toContain('Parse error');
      expect(errors[0].line).toBe(2);
      expect(errors[0].column).toBe(5);
    });

    it('returns empty array when WASM is not ready', async () => {
      // Keep the init failing so WASM never becomes ready
      mockWasmInit.mockImplementation(() => {
        throw new Error('Init failed');
      });

      const { result } = renderHook(() => useWasm());

      // Wait for all retry attempts to fail
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 10000 }
      );

      // Should return empty array since WASM not ready
      const errors = await result.current.validateConfig('test');
      expect(errors).toEqual([]);
    });
  });

  describe('runSimulation', () => {
    it('runs simulation with valid configuration', async () => {
      mockSimulate.mockReturnValue({
        states: [{ timestamp_us: 0, active_modifiers: [], active_locks: [], active_layer: null }],
        outputs: [{ keycode: 'B', event_type: 'press', timestamp_us: 0 }],
        latency: [100],
        final_state: { active_modifiers: [], active_locks: [], active_layer: null },
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simResult = await result.current.runSimulation('map("A", "B");', {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: 0 }],
      });

      expect(simResult).not.toBeNull();
      expect(simResult?.outputs).toHaveLength(1);
      expect(simResult?.outputs[0].keycode).toBe('B');
    });

    it('returns null when WASM is not ready', async () => {
      mockWasmInit.mockImplementation(() => {
        throw new Error('Init failed');
      });

      const { result } = renderHook(() => useWasm());

      // Wait for all retry attempts to fail
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 10000 }
      );

      const simResult = await result.current.runSimulation('test', { events: [] });
      expect(simResult).toBeNull();
    });

    it('returns null when simulation fails', async () => {
      mockSimulate.mockImplementation(() => {
        throw new Error('Simulation error');
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const simResult = await result.current.runSimulation('map("A", "B");', { events: [] });
      expect(simResult).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('handles non-Error objects thrown during initialization', async () => {
      mockWasmInit.mockImplementation(() => {
        throw 'String error';
      });

      const { result } = renderHook(() => useWasm());

      // Wait for all retry attempts to fail
      await waitFor(
        () => {
          expect(result.current.isLoading).toBe(false);
        },
        { timeout: 10000 }
      );

      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toContain('String error');
    });

    it('handles non-Error objects thrown during validation', async () => {
      mockLoadConfig.mockImplementation(() => {
        throw { message: 'Object error' };
      });

      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('test');
      expect(errors).toHaveLength(1);
      expect(errors[0].message).toBeDefined();
    });
  });

  describe('Edge Cases', () => {
    it('handles empty configuration string', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      const errors = await result.current.validateConfig('');
      expect(mockLoadConfig).toHaveBeenCalledWith('');
    });

    it('handles empty simulation input', async () => {
      const { result } = renderHook(() => useWasm());

      await waitFor(() => {
        expect(result.current.isWasmReady).toBe(true);
      });

      await result.current.runSimulation('map("A", "B");', { events: [] });
      expect(mockSimulate).toHaveBeenCalled();
    });
  });
});
