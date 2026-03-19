/* tslint:disable */
/* eslint-disable */

export class ConfigHandle {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}

/**
 * Get current simulation state.
 *
 * Returns the state from the most recent simulation for the given configuration.
 * This matches the DaemonStateResponse format from the daemon API.
 *
 * # Arguments
 * * `config` - Handle to a loaded configuration
 *
 * # Returns
 * * `Ok(JsValue)` - DaemonStateResponse as JSON with:
 *   - active_layer: Optional<String> - Current active layer (if any)
 *   - modifiers: Vec<String> - Active modifier IDs as strings
 *   - locks: Vec<String> - Active lock IDs as strings
 *   - raw_state: Vec<bool> - 255-bit state vector
 *   - active_modifier_count: usize - Number of active modifiers
 *   - active_lock_count: usize - Number of active locks
 * * `Err(JsValue)` - Error message if no simulation has been run
 *
 * # Errors
 * Returns an error if:
 * - ConfigHandle is invalid
 * - No simulation has been run yet (no state available)
 *
 * # Example (JavaScript)
 * ```javascript
 * // After running a simulation
 * const state = get_state(configHandle);
 * console.log(`Active modifiers: ${state.active_modifier_count}`);
 * console.log(`Active locks: ${state.active_lock_count}`);
 * ```
 */
export function get_state(config: ConfigHandle): any;

/**
 * Load a Rhai configuration from source text.
 *
 * Parses the Rhai source, compiles it to a ConfigRoot structure, and stores
 * it in the global CONFIG_STORE. Returns an opaque handle for future operations.
 *
 * # Arguments
 * * `rhai_source` - Rhai DSL source code as a string
 *
 * # Returns
 * * `Ok(ConfigHandle)` - Handle to the loaded configuration
 * * `Err(JsValue)` - Parse error with line number and description
 *
 * # Errors
 * Returns an error if:
 * - Rhai syntax is invalid (returns parse error with line number)
 * - Configuration size exceeds 1MB
 * - Parser fails to generate valid ConfigRoot
 *
 * # Example (JavaScript)
 * ```javascript
 * const handle = load_config(`
 *   device("*") {
 *     map("VK_A", "VK_B");
 *   }
 * `);
 * ```
 */
export function load_config(rhai_source: string): ConfigHandle;

/**
 * Load a pre-compiled .krx binary configuration.
 *
 * Deserializes a .krx binary file using rkyv with validation to ensure
 * integrity. The configuration is then stored in the global CONFIG_STORE.
 *
 * # Arguments
 * * `binary` - Raw bytes of a .krx binary file
 *
 * # Returns
 * * `Ok(ConfigHandle)` - Handle to the loaded configuration
 * * `Err(JsValue)` - Validation or deserialization error
 *
 * # Errors
 * Returns an error if:
 * - Binary format is invalid or corrupted
 * - Binary size exceeds 10MB limit
 * - Validation fails (corrupted data, invalid structure)
 * - rkyv deserialization fails
 *
 * # Example (JavaScript)
 * ```javascript
 * const response = await fetch('config.krx');
 * const binary = new Uint8Array(await response.arrayBuffer());
 * const handle = load_krx(binary);
 * ```
 */
export function load_krx(binary: Uint8Array): ConfigHandle;

/**
 * Simulate keyboard event sequence.
 *
 * Processes a sequence of keyboard events through the remapping configuration,
 * tracking state changes and performance metrics.
 *
 * # Arguments
 * * `config` - Handle to a loaded configuration
 * * `events_json` - JSON string containing EventSequence
 *
 * # Returns
 * * `Ok(JsValue)` - SimulationResult as JSON
 * * `Err(JsValue)` - Error message
 *
 * # Errors
 * Returns an error if:
 * - ConfigHandle is invalid
 * - JSON is malformed or doesn't match EventSequence schema
 * - Event keycodes are invalid
 * - Simulation exceeds 1000 events
 *
 * # Example (JavaScript)
 * ```javascript
 * const events = {
 *   events: [
 *     { keycode: "A", event_type: "press", timestamp_us: 0 },
 *     { keycode: "A", event_type: "release", timestamp_us: 100000 }
 *   ]
 * };
 * const result = simulate(configHandle, JSON.stringify(events));
 * ```
 */
export function simulate(config: ConfigHandle, events_json: string): any;

/**
 * Validate a Rhai configuration source.
 *
 * Parses the Rhai source using keyrx_compiler and returns validation errors as a JSON array.
 * Returns an empty array if the configuration is valid.
 *
 * This function uses the same parser as the daemon to ensure validation results are
 * deterministic and match the daemon's behavior exactly.
 *
 * # Arguments
 * * `rhai_source` - Rhai DSL source code as a string
 *
 * # Returns
 * * `Ok(JsValue)` - JSON array of validation errors. Each error has:
 *   - `line`: number - Line number where the error occurred
 *   - `column`: number - Column number where the error occurred
 *   - `length`: number - Length of the error span
 *   - `message`: string - Error description
 * * `Err(JsValue)` - Internal error during validation
 *
 * # Example (JavaScript)
 * ```javascript
 * const errors = validate_config(`
 *   device("*") {
 *     map("A", "B");
 *   }
 * `);
 * console.log(errors); // [] for valid config
 * ```
 */
export function validate_config(rhai_source: string): any;

/**
 * Initialize the WASM module.
 *
 * This sets up the panic hook to provide better error messages in the
 * browser console. Call this before using any other WASM functions.
 *
 * # Example (JavaScript)
 * ```javascript
 * import init, { wasm_init } from './pkg/keyrx_core.js';
 *
 * await init();
 * wasm_init();
 * ```
 */
export function wasm_init(): void;

export type InitInput =
  | RequestInfo
  | URL
  | Response
  | BufferSource
  | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_confighandle_free: (a: number, b: number) => void;
  readonly get_state: (a: number, b: number) => void;
  readonly load_config: (a: number, b: number, c: number) => void;
  readonly load_krx: (a: number, b: number, c: number) => void;
  readonly simulate: (a: number, b: number, c: number, d: number) => void;
  readonly validate_config: (a: number, b: number, c: number) => void;
  readonly wasm_init: () => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (
    a: number,
    b: number,
    c: number,
    d: number
  ) => number;
  readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export4: (a: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(
  module: { module: SyncInitInput } | SyncInitInput
): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init(
  module_or_path?:
    | { module_or_path: InitInput | Promise<InitInput> }
    | InitInput
    | Promise<InitInput>
): Promise<InitOutput>;
