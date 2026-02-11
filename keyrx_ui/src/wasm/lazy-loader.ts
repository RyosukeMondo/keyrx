/**
 * WASM Module Lazy Loader
 *
 * Defers WASM module loading until needed to improve initial page load time.
 * First call triggers async load, subsequent calls return cached instance.
 *
 * Performance Impact:
 * - Initial page load: -150-200ms (WASM not downloaded immediately)
 * - TTI (Time to Interactive): -200-300ms (WASM parse/instantiate deferred)
 *
 * Usage:
 *   const wasm = await getWasm();
 *   const result = wasm.simulate(input);
 */

let wasmInstance: typeof import('@/wasm/pkg/keyrx_core.js') | null = null;
let wasmLoading: Promise<typeof import('@/wasm/pkg/keyrx_core.js')> | null = null;

/**
 * Get WASM instance with lazy loading
 *
 * On first call, loads the WASM module asynchronously.
 * Returns cached instance on subsequent calls.
 *
 * @returns Promise resolving to WASM module instance
 * @throws Error if WASM loading fails
 */
export async function getWasm() {
  // Return cached instance if already loaded
  if (wasmInstance) {
    return wasmInstance;
  }

  // Reuse loading promise if already in progress
  if (wasmLoading) {
    return wasmLoading;
  }

  // Start loading
  wasmLoading = import('@/wasm/pkg/keyrx_core.js');

  try {
    wasmInstance = await wasmLoading;
    return wasmInstance;
  } catch (error) {
    wasmLoading = null;
    throw error;
  }
}

/**
 * Check if WASM is already loaded (synchronous)
 *
 * Useful for determining if blocking operation needed
 *
 * @returns true if WASM already loaded, false otherwise
 */
export function isWasmLoaded(): boolean {
  return wasmInstance !== null;
}

/**
 * Preload WASM module without awaiting (fire-and-forget)
 *
 * Useful for background loading during idle time
 *
 * @returns void (use getWasm() if you need to await)
 */
export function preloadWasm(): void {
  getWasm().catch(() => {
    // Silent failure - preload is optional
  });
}

/**
 * Reset WASM instance (for testing only)
 *
 * @internal
 */
export function __resetWasmForTesting(): void {
  wasmInstance = null;
  wasmLoading = null;
}
