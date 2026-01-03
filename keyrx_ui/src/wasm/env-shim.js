/**
 * Shim for wasm-pack generated 'env' import.
 *
 * wasm-pack generates code that imports from 'env', which is a placeholder
 * that should be resolved by the bundler. This module provides the required
 * functions that WASM modules may need.
 *
 * See: https://github.com/rustwasm/wasm-pack/issues/835
 */

// Provide timing function for WASM modules
export function now() {
  return Date.now();
}

export default { now };
