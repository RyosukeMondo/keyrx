/**
 * Unit tests for WasmCore TypeScript wrapper.
 *
 * These tests mock the underlying WASM module to test error handling,
 * input validation, and Promise-based API behavior without requiring
 * actual WASM compilation.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { WasmCore, WasmError, type EventSequence, type ConfigHandle } from './core';

// ============================================================================
// Mocks
// ============================================================================

// The WASM module is automatically mocked via vitest.config.ts alias
// Import mocked functions for manipulation in tests
import init, {
  wasm_init,
  load_config,
  load_krx,
  simulate,
  get_state,
} from './pkg/keyrx_core';

// ============================================================================
// Test Suite
// ============================================================================

describe('WasmCore', () => {
  let wasmCore: WasmCore;

  beforeEach(() => {
    // Create fresh instance for each test
    wasmCore = new WasmCore();

    // Reset all mocks
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // ==========================================================================
  // Initialization Tests
  // ==========================================================================

  describe('init()', () => {
    it('should initialize the WASM module successfully', async () => {
      await wasmCore.init();

      expect(init).toHaveBeenCalledOnce();
      expect(wasm_init).toHaveBeenCalledOnce();
    });

    it('should be idempotent (safe to call multiple times)', async () => {
      await wasmCore.init();
      await wasmCore.init();
      await wasmCore.init();

      // init() and wasm_init() should only be called once
      expect(init).toHaveBeenCalledOnce();
      expect(wasm_init).toHaveBeenCalledOnce();
    });

    it('should throw WasmError if initialization fails', async () => {
      // Create a new instance for this test to avoid state pollution
      const failingCore = new WasmCore();
      vi.mocked(init).mockRejectedValueOnce(new Error('WASM load failed'));

      try {
        await failingCore.init();
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('Failed to initialize WASM module');
      }
    });
  });

  // ==========================================================================
  // loadConfig() Tests
  // ==========================================================================

  describe('loadConfig()', () => {
    beforeEach(async () => {
      // Initialize before each test
      await wasmCore.init();
    });

    it('should load a valid Rhai configuration', async () => {
      const mockHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
      vi.mocked(load_config).mockReturnValueOnce(mockHandle);

      const rhaiSource = 'device("*") { map("A", "B"); }';
      const result = await wasmCore.loadConfig(rhaiSource);

      expect(load_config).toHaveBeenCalledWith(rhaiSource);
      expect(result).toBe(mockHandle);
    });

    it('should throw WasmError if not initialized', async () => {
      const uninitializedCore = new WasmCore();

      await expect(uninitializedCore.loadConfig('device("*") {}')).rejects.toThrow(WasmError);
      await expect(uninitializedCore.loadConfig('device("*") {}')).rejects.toThrow('not initialized');
    });

    it('should throw WasmError if config source is empty', async () => {
      await expect(wasmCore.loadConfig('')).rejects.toThrow(WasmError);
      await expect(wasmCore.loadConfig('')).rejects.toThrow('cannot be empty');
    });

    it('should throw WasmError if config source is only whitespace', async () => {
      await expect(wasmCore.loadConfig('   \n\t  ')).rejects.toThrow(WasmError);
      await expect(wasmCore.loadConfig('   \n\t  ')).rejects.toThrow('cannot be empty');
    });

    it('should throw WasmError if config exceeds 1MB size limit', async () => {
      // Create a string larger than 1MB
      const largeConfig = 'x'.repeat(1024 * 1024 + 1);

      await expect(wasmCore.loadConfig(largeConfig)).rejects.toThrow(WasmError);
      await expect(wasmCore.loadConfig(largeConfig)).rejects.toThrow('exceeds maximum size');
    });

    it('should accept config exactly at 1MB limit', async () => {
      const mockHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
      vi.mocked(load_config).mockReturnValueOnce(mockHandle);

      // Create a string exactly 1MB
      const maxConfig = 'x'.repeat(1024 * 1024);
      const result = await wasmCore.loadConfig(maxConfig);

      expect(result).toBe(mockHandle);
    });

    it('should convert WASM errors to WasmError with context', async () => {
      vi.mocked(load_config).mockImplementationOnce(() => {
        throw new Error('Parse error at line 5');
      });

      try {
        await wasmCore.loadConfig('invalid config');
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('Parse error at line 5');
      }
    });

    it('should handle string errors from WASM', async () => {
      vi.mocked(load_config).mockImplementationOnce(() => {
        throw 'String error from WASM';
      });

      try {
        await wasmCore.loadConfig('config');
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('String error from WASM');
      }
    });
  });

  // ==========================================================================
  // loadKrx() Tests
  // ==========================================================================

  describe('loadKrx()', () => {
    beforeEach(async () => {
      await wasmCore.init();
    });

    it('should load a valid .krx binary', async () => {
      const mockHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
      vi.mocked(load_krx).mockReturnValueOnce(mockHandle);

      const binary = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
      const result = await wasmCore.loadKrx(binary);

      expect(load_krx).toHaveBeenCalledWith(binary);
      expect(result).toBe(mockHandle);
    });

    it('should throw WasmError if not initialized', async () => {
      const uninitializedCore = new WasmCore();
      const binary = new Uint8Array([1, 2, 3]);

      await expect(uninitializedCore.loadKrx(binary)).rejects.toThrow(WasmError);
      await expect(uninitializedCore.loadKrx(binary)).rejects.toThrow('not initialized');
    });

    it('should throw WasmError if binary is empty', async () => {
      const emptyBinary = new Uint8Array([]);

      await expect(wasmCore.loadKrx(emptyBinary)).rejects.toThrow(WasmError);
      await expect(wasmCore.loadKrx(emptyBinary)).rejects.toThrow('cannot be empty');
    });

    it('should throw WasmError if binary exceeds 10MB size limit', async () => {
      // Create a binary larger than 10MB
      const largeBinary = new Uint8Array(10 * 1024 * 1024 + 1);

      await expect(wasmCore.loadKrx(largeBinary)).rejects.toThrow(WasmError);
      await expect(wasmCore.loadKrx(largeBinary)).rejects.toThrow('exceeds maximum size');
    });

    it('should accept binary exactly at 10MB limit', async () => {
      const mockHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
      vi.mocked(load_krx).mockReturnValueOnce(mockHandle);

      // Create a binary exactly 10MB
      const maxBinary = new Uint8Array(10 * 1024 * 1024);
      const result = await wasmCore.loadKrx(maxBinary);

      expect(result).toBe(mockHandle);
    });

    it('should convert WASM errors to WasmError', async () => {
      vi.mocked(load_krx).mockImplementationOnce(() => {
        throw new Error('Invalid binary format');
      });

      try {
        await wasmCore.loadKrx(new Uint8Array([1, 2, 3]));
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('Invalid binary format');
      }
    });
  });

  // ==========================================================================
  // simulate() Tests
  // ==========================================================================

  describe('simulate()', () => {
    let mockConfigHandle: ConfigHandle;

    beforeEach(async () => {
      await wasmCore.init();
      mockConfigHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
    });

    it('should simulate a valid event sequence', async () => {
      const mockResult = {
        timeline: [
          {
            timestamp_us: 0,
            input: { keycode: 'A', event_type: 'press', timestamp_us: 0 },
            outputs: [{ keycode: 'B', event_type: 'press', timestamp_us: 0 }],
            state: { active_modifiers: [], active_locks: [], active_layer: null },
            latency_us: 100,
          },
        ],
        latency_stats: {
          min_us: 100,
          avg_us: 100,
          max_us: 100,
          p95_us: 100,
          p99_us: 100,
        },
        final_state: { active_modifiers: [], active_locks: [], active_layer: null },
      };

      vi.mocked(simulate).mockReturnValueOnce(JSON.stringify(mockResult));

      const eventSequence: EventSequence = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = await wasmCore.simulate(mockConfigHandle, eventSequence);

      expect(simulate).toHaveBeenCalled();
      expect(result).toEqual(mockResult);
    });

    it('should throw WasmError if not initialized', async () => {
      const uninitializedCore = new WasmCore();
      const eventSequence: EventSequence = {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: 0 }],
      };

      await expect(uninitializedCore.simulate(mockConfigHandle, eventSequence)).rejects.toThrow(WasmError);
      await expect(uninitializedCore.simulate(mockConfigHandle, eventSequence)).rejects.toThrow('not initialized');
    });

    it('should throw WasmError if event sequence is empty', async () => {
      const emptySequence: EventSequence = { events: [] };

      await expect(wasmCore.simulate(mockConfigHandle, emptySequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, emptySequence)).rejects.toThrow('cannot be empty');
    });

    it('should throw WasmError if keycode is empty', async () => {
      const invalidSequence: EventSequence = {
        events: [{ keycode: '', event_type: 'press', timestamp_us: 0 }],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('keycode cannot be empty');
    });

    it('should throw WasmError if keycode is whitespace only', async () => {
      const invalidSequence: EventSequence = {
        events: [{ keycode: '   ', event_type: 'press', timestamp_us: 0 }],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('keycode cannot be empty');
    });

    it('should throw WasmError if event_type is invalid', async () => {
      const invalidSequence: EventSequence = {
        events: [{ keycode: 'A', event_type: 'invalid' as 'press', timestamp_us: 0 }],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('must be \'press\' or \'release\'');
    });

    it('should throw WasmError if timestamp is negative', async () => {
      const invalidSequence: EventSequence = {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: -100 }],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('must be non-negative');
    });

    it('should throw WasmError if timestamps are not in ascending order', async () => {
      const invalidSequence: EventSequence = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 1000 },
          { keycode: 'A', event_type: 'release', timestamp_us: 500 }, // Earlier than previous
        ],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow(WasmError);
      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('ascending order');
    });

    it('should allow equal timestamps (same time)', async () => {
      vi.mocked(simulate).mockReturnValueOnce(
        JSON.stringify({
          timeline: [],
          latency_stats: { min_us: 0, avg_us: 0, max_us: 0, p95_us: 0, p99_us: 0 },
          final_state: { active_modifiers: [], active_locks: [], active_layer: null },
        })
      );

      const validSequence: EventSequence = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 1000 },
          { keycode: 'B', event_type: 'press', timestamp_us: 1000 }, // Same time is OK
        ],
      };

      await expect(wasmCore.simulate(mockConfigHandle, validSequence)).resolves.toBeDefined();
    });

    it('should include event index in error messages', async () => {
      const invalidSequence: EventSequence = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: '', event_type: 'press', timestamp_us: 100 }, // Event 1 is invalid
        ],
      };

      await expect(wasmCore.simulate(mockConfigHandle, invalidSequence)).rejects.toThrow('Event 1:');
    });

    it('should convert WASM errors to WasmError', async () => {
      vi.mocked(simulate).mockImplementationOnce(() => {
        throw new Error('Simulation panic');
      });

      const eventSequence: EventSequence = {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: 0 }],
      };

      try {
        await wasmCore.simulate(mockConfigHandle, eventSequence);
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('Simulation panic');
      }
    });
  });

  // ==========================================================================
  // getState() Tests
  // ==========================================================================

  describe('getState()', () => {
    let mockConfigHandle: ConfigHandle;

    beforeEach(async () => {
      await wasmCore.init();
      mockConfigHandle = { __brand: 'ConfigHandle' } as ConfigHandle;
    });

    it('should get state successfully', async () => {
      const mockState = {
        active_modifiers: [1, 2],
        active_locks: [3],
        active_layer: 'layer1',
        raw_state: Array(255).fill(false),
      };

      vi.mocked(get_state).mockReturnValueOnce(JSON.stringify(mockState));

      const result = await wasmCore.getState(mockConfigHandle);

      expect(get_state).toHaveBeenCalledWith(mockConfigHandle);
      expect(result).toEqual(mockState);
    });

    it('should throw WasmError if not initialized', async () => {
      const uninitializedCore = new WasmCore();

      await expect(uninitializedCore.getState(mockConfigHandle)).rejects.toThrow(WasmError);
      await expect(uninitializedCore.getState(mockConfigHandle)).rejects.toThrow('not initialized');
    });

    it('should convert WASM errors to WasmError', async () => {
      vi.mocked(get_state).mockImplementationOnce(() => {
        throw new Error('No simulation run yet');
      });

      try {
        await wasmCore.getState(mockConfigHandle);
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('No simulation run yet');
      }
    });

    it('should handle invalid handle error', async () => {
      vi.mocked(get_state).mockImplementationOnce(() => {
        throw 'Invalid ConfigHandle: 9999';
      });

      try {
        await wasmCore.getState(mockConfigHandle);
        expect.fail('Should have thrown WasmError');
      } catch (error) {
        expect(error).toBeInstanceOf(WasmError);
        expect((error as WasmError).message).toContain('Invalid ConfigHandle');
      }
    });
  });

  // ==========================================================================
  // Error Handling Tests
  // ==========================================================================

  describe('Error Handling', () => {
    it('should create WasmError with message and cause', () => {
      const cause = new Error('Original error');
      const error = new WasmError('Wrapped error', cause);

      expect(error.message).toBe('Wrapped error');
      expect(error.name).toBe('WasmError');
      expect(error.cause).toBe(cause);
    });

    it('should create WasmError without cause', () => {
      const error = new WasmError('Simple error');

      expect(error.message).toBe('Simple error');
      expect(error.name).toBe('WasmError');
      expect(error.cause).toBeUndefined();
    });

    it('should be instanceof Error', () => {
      const error = new WasmError('Test');
      expect(error instanceof Error).toBe(true);
    });
  });
});
