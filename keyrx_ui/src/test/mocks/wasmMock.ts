/**
 * Global WASM Module Mock
 *
 * Provides a mock implementation of the WASM module for unit tests.
 * This prevents tests from attempting to load actual WASM binaries
 * which fail in the jsdom test environment.
 *
 * The mock provides:
 * - Successful initialization (default export returns a resolved promise)
 * - Valid implementations of all exported functions
 * - Configurable behavior through the mock functions
 */

import { vi } from 'vitest';

/**
 * Mock implementation of wasm_init
 * Sets up the WASM panic hook (no-op in tests)
 */
export const mockWasmInit = vi.fn(() => {});

/**
 * Mock implementation of load_config
 * Returns a mock ConfigHandle (number) on success
 * Throws on invalid configurations containing 'invalid' or 'syntax error'
 */
export const mockLoadConfig = vi.fn((code: string) => {
  if (code.includes('invalid') || code.includes('syntax error')) {
    throw new Error('Parse error at line 2 column 5: Invalid syntax');
  }
  // Return a mock config handle
  return 12345;
});

/**
 * Mock implementation of validate_config
 * Returns an empty array for valid configurations
 * Returns validation errors for invalid configurations
 */
export const mockValidateConfig = vi.fn((code: string) => {
  if (code.includes('invalid') || code.includes('syntax error')) {
    return [
      {
        line: 2,
        column: 5,
        length: 10,
        message: 'Parse error: Invalid syntax',
      },
    ];
  }
  return [];
});

/**
 * Mock implementation of simulate
 * Returns a mock simulation result
 */
export const mockSimulate = vi.fn((_handle: number, eventsJson: string) => {
  const input = JSON.parse(eventsJson);
  return {
    states: [
      {
        timestamp_us: 0,
        active_modifiers: [],
        active_locks: [],
        active_layer: null,
      },
    ],
    outputs: input.events.map(
      (e: { keycode: string; event_type: string; timestamp_us: number }) => ({
        keycode: e.keycode,
        event_type: e.event_type,
        timestamp_us: e.timestamp_us,
      })
    ),
    latency: input.events.map(() => 100),
    final_state: {
      active_modifiers: [],
      active_locks: [],
      active_layer: null,
    },
  };
});

/**
 * Mock implementation of get_state
 * Returns the current state of a configuration
 */
export const mockGetState = vi.fn((_handle: number) => ({
  active_modifiers: [],
  active_locks: [],
  active_layer: null,
}));

/**
 * Mock implementation of load_krx
 * Returns a mock ConfigHandle
 */
export const mockLoadKrx = vi.fn((_binary: Uint8Array) => 12345);

/**
 * Mock default export (init function for wasm-pack web target)
 * Returns a resolved promise to simulate successful WASM binary loading
 */
export const mockDefault = vi.fn(() => Promise.resolve());

/**
 * Reset all mock function states
 * Call this in beforeEach to ensure clean test state
 */
export function resetWasmMocks() {
  mockWasmInit.mockClear();
  mockLoadConfig.mockClear();
  mockValidateConfig.mockClear();
  mockSimulate.mockClear();
  mockGetState.mockClear();
  mockLoadKrx.mockClear();
  mockDefault.mockClear();
}

/**
 * Complete mock module object
 * This matches the shape of the actual WASM module exports
 */
export const wasmModuleMock = {
  default: mockDefault,
  wasm_init: mockWasmInit,
  load_config: mockLoadConfig,
  validate_config: mockValidateConfig,
  simulate: mockSimulate,
  get_state: mockGetState,
  load_krx: mockLoadKrx,
};
