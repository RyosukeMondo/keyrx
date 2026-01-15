/**
 * Vitest Mock Setup for Unit Tests
 *
 * This file configures global mocks that need to be in place before any
 * component imports are resolved.
 *
 * IMPORTANT: This file must be listed BEFORE setup.ts in the vitest config's
 * setupFiles array to ensure mocks are in place before any component imports.
 */

import { vi } from 'vitest';

/**
 * Mock the WASM module globally for unit tests
 *
 * This prevents all unit tests from attempting to load the actual WASM binary,
 * which fails in the jsdom test environment. Components that use the
 * useWasm hook will receive this mock instead.
 */
vi.mock('@/wasm/pkg/keyrx_core.js', () => ({
  // Default export is the init function for wasm-pack web target
  default: vi.fn(() => Promise.resolve()),

  // wasm_init sets up the panic hook (no-op in tests)
  wasm_init: vi.fn(() => {}),

  // load_config returns a mock ConfigHandle on success
  load_config: vi.fn((code: string) => {
    if (code.includes('invalid') || code.includes('syntax error')) {
      throw new Error('Parse error at line 2 column 5: Invalid syntax');
    }
    return 12345;
  }),

  // validate_config returns validation errors for invalid configs
  validate_config: vi.fn((code: string) => {
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
  }),

  // simulate returns a mock simulation result
  simulate: vi.fn((_handle: number, eventsJson: string) => {
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
  }),

  // get_state returns the current state
  get_state: vi.fn(() => ({
    active_modifiers: [],
    active_locks: [],
    active_layer: null,
  })),

  // load_krx loads a compiled .krx binary
  load_krx: vi.fn(() => 12345),
}));

/**
 * Also mock the .wasm binary file import to prevent fetch attempts
 */
vi.mock('@/wasm/pkg/keyrx_core_bg.wasm', () => ({
  default: new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]), // Minimal valid WASM header
}));
