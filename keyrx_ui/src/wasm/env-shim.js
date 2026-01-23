/**
 * Stub module for WASM env imports
 * This provides runtime imports that WASM might need.
 * Required because wasm-bindgen sometimes generates code that imports from 'env'.
 */

/**
 * Returns high-resolution timestamp in nanoseconds.
 * Used by Rust's Instant::now() when compiled to WASM.
 *
 * @returns {bigint} Current time in nanoseconds since an arbitrary epoch
 */
export function now() {
  // performance.now() returns milliseconds with microsecond precision
  // Convert to nanoseconds by multiplying by 1,000,000
  return BigInt(Math.floor(performance.now() * 1_000_000));
}

// Export as default object for compatibility
export default {
  now
};
