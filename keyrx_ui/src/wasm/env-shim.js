/**
 * Stub module for WASM env imports
 * This provides empty stubs for any runtime imports WASM might need.
 * Required because wasm-bindgen sometimes generates code that imports from 'env'.
 */

// Export empty object - the WASM module will use this for any env imports
export default {};
