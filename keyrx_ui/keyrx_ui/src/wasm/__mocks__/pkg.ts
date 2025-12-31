/**
 * Mock implementation of the WASM module for testing.
 *
 * This file provides stub implementations of all WASM functions
 * to enable testing without loading actual WASM binary.
 */

import { vi } from 'vitest';

// Mock ConfigHandle type
export type ConfigHandle = { __brand: 'ConfigHandle' };

// Mock init function (async WASM initialization)
export default vi.fn().mockResolvedValue(undefined);

// Mock WASM functions
export const wasm_init = vi.fn();
export const load_config = vi.fn();
export const load_krx = vi.fn();
export const simulate = vi.fn();
export const get_state = vi.fn();
