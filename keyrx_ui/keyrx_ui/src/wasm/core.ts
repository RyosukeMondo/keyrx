/**
 * TypeScript API wrapper for keyrx_core WASM module.
 *
 * This module provides a type-safe, Promise-based API for loading configurations,
 * running simulations, and querying state from the WASM module.
 *
 * @module wasm/core
 */

import init, {
  wasm_init,
  load_config as wasmLoadConfig,
  load_krx as wasmLoadKrx,
  simulate as wasmSimulate,
  get_state as wasmGetState,
  type ConfigHandle
} from './pkg/keyrx_core';

// Re-export ConfigHandle for use in other modules
export type { ConfigHandle };

// ============================================================================
// Type Definitions
// ============================================================================

/**
 * A single keyboard event for simulation.
 */
export interface SimKeyEvent {
  /** Key code (e.g., "A", "B", "LeftShift") */
  keycode: string;
  /** Event type: "press" or "release" */
  event_type: 'press' | 'release';
  /** Timestamp in microseconds */
  timestamp_us: number;
}

/**
 * Input event sequence for simulation.
 */
export interface EventSequence {
  /** List of events to simulate */
  events: SimKeyEvent[];
}

/**
 * State snapshot during simulation.
 */
export interface SimulationState {
  /** Active modifiers (list of modifier IDs) */
  active_modifiers: number[];
  /** Active locks (list of lock IDs) */
  active_locks: number[];
  /** Current active layer (if any) */
  active_layer: string | null;
}

/**
 * Entry in the simulation timeline.
 */
export interface TimelineEntry {
  /** Timestamp in microseconds */
  timestamp_us: number;
  /** Input event (if this was an input) */
  input: SimKeyEvent | null;
  /** Output events generated from this input */
  outputs: SimKeyEvent[];
  /** State snapshot after processing this event */
  state: SimulationState;
  /** Processing latency for this event in microseconds */
  latency_us: number;
}

/**
 * Latency statistics for the simulation.
 */
export interface LatencyStats {
  /** Minimum latency in microseconds */
  min_us: number;
  /** Average latency in microseconds */
  avg_us: number;
  /** Maximum latency in microseconds */
  max_us: number;
  /** 95th percentile latency in microseconds */
  p95_us: number;
  /** 99th percentile latency in microseconds */
  p99_us: number;
}

/**
 * Result of a simulation run.
 */
export interface SimulationResult {
  /** Timeline of all events (input and output) */
  timeline: TimelineEntry[];
  /** Latency statistics in microseconds */
  latency_stats: LatencyStats;
  /** Final state after simulation */
  final_state: SimulationState;
}

/**
 * Daemon state response format (matches IPC format).
 */
export interface DaemonState {
  /** Active modifiers */
  active_modifiers: number[];
  /** Active locks */
  active_locks: number[];
  /** Active layer */
  active_layer: string | null;
  /** Raw 255-bit state vector */
  raw_state: number[];
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Error class for WASM-related errors.
 */
export class WasmError extends Error {
  public readonly cause?: unknown;

  constructor(message: string, cause?: unknown) {
    super(message);
    this.name = 'WasmError';
    this.cause = cause;
  }
}

// ============================================================================
// WasmCore Class
// ============================================================================

/**
 * Type-safe wrapper for the keyrx_core WASM module.
 *
 * This class provides Promise-based APIs for all WASM functions with
 * input validation and error handling.
 */
export class WasmCore {
  private initialized = false;

  /**
   * Initialize the WASM module.
   *
   * This must be called before using any other methods. It loads the WASM
   * binary and sets up the panic hook for better error messages.
   *
   * @throws {WasmError} If initialization fails
   */
  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      await init();
      wasm_init();
      this.initialized = true;
    } catch (error) {
      throw new WasmError('Failed to initialize WASM module', error);
    }
  }

  /**
   * Load a Rhai configuration from source text.
   *
   * @param rhaiSource - The Rhai configuration source code
   * @returns A handle to the loaded configuration
   * @throws {WasmError} If the configuration is invalid or exceeds size limits
   */
  async loadConfig(rhaiSource: string): Promise<ConfigHandle> {
    this.ensureInitialized();

    // Validate input
    if (!rhaiSource || rhaiSource.trim().length === 0) {
      throw new WasmError('Configuration source cannot be empty');
    }

    const MAX_CONFIG_SIZE = 1024 * 1024; // 1MB
    if (rhaiSource.length > MAX_CONFIG_SIZE) {
      throw new WasmError(`Configuration exceeds maximum size of ${MAX_CONFIG_SIZE} bytes`);
    }

    try {
      return wasmLoadConfig(rhaiSource);
    } catch (error) {
      throw this.convertWasmError('Failed to load configuration', error);
    }
  }

  /**
   * Load a pre-compiled .krx binary configuration.
   *
   * @param binary - The .krx binary data
   * @returns A handle to the loaded configuration
   * @throws {WasmError} If the binary is invalid or exceeds size limits
   */
  async loadKrx(binary: Uint8Array): Promise<ConfigHandle> {
    this.ensureInitialized();

    // Validate input
    if (!binary || binary.length === 0) {
      throw new WasmError('Binary data cannot be empty');
    }

    const MAX_BINARY_SIZE = 10 * 1024 * 1024; // 10MB
    if (binary.length > MAX_BINARY_SIZE) {
      throw new WasmError(`Binary exceeds maximum size of ${MAX_BINARY_SIZE} bytes`);
    }

    try {
      return wasmLoadKrx(binary);
    } catch (error) {
      throw this.convertWasmError('Failed to load binary configuration', error);
    }
  }

  /**
   * Simulate an event sequence with a loaded configuration.
   *
   * @param config - Handle to the loaded configuration
   * @param eventSequence - The event sequence to simulate
   * @returns Simulation result with timeline and statistics
   * @throws {WasmError} If simulation fails or input is invalid
   */
  async simulate(
    config: ConfigHandle,
    eventSequence: EventSequence
  ): Promise<SimulationResult> {
    this.ensureInitialized();

    // Validate input
    if (!eventSequence.events || eventSequence.events.length === 0) {
      throw new WasmError('Event sequence cannot be empty');
    }

    // Validate each event
    for (let i = 0; i < eventSequence.events.length; i++) {
      const event = eventSequence.events[i];

      if (!event.keycode || event.keycode.trim().length === 0) {
        throw new WasmError(`Event ${i}: keycode cannot be empty`);
      }

      if (event.event_type !== 'press' && event.event_type !== 'release') {
        throw new WasmError(`Event ${i}: event_type must be 'press' or 'release'`);
      }

      if (event.timestamp_us < 0) {
        throw new WasmError(`Event ${i}: timestamp_us must be non-negative`);
      }

      // Validate timestamps are in ascending order
      if (i > 0 && event.timestamp_us < eventSequence.events[i - 1].timestamp_us) {
        throw new WasmError(`Event ${i}: timestamps must be in ascending order`);
      }
    }

    try {
      const eventsJson = JSON.stringify(eventSequence);
      const resultJson = wasmSimulate(config, eventsJson);
      return JSON.parse(resultJson as string) as SimulationResult;
    } catch (error) {
      throw this.convertWasmError('Simulation failed', error);
    }
  }

  /**
   * Get the current state for a loaded configuration.
   *
   * @param config - Handle to the loaded configuration
   * @returns The current daemon state
   * @throws {WasmError} If the configuration handle is invalid
   */
  async getState(config: ConfigHandle): Promise<DaemonState> {
    this.ensureInitialized();

    try {
      const stateJson = wasmGetState(config);
      return JSON.parse(stateJson as string) as DaemonState;
    } catch (error) {
      throw this.convertWasmError('Failed to get state', error);
    }
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  /**
   * Ensure the WASM module has been initialized.
   * @throws {WasmError} If not initialized
   */
  private ensureInitialized(): void {
    if (!this.initialized) {
      throw new WasmError('WASM module not initialized. Call init() first.');
    }
  }

  /**
   * Convert a WASM error to a WasmError with better messaging.
   */
  private convertWasmError(message: string, error: unknown): WasmError {
    if (error instanceof Error) {
      return new WasmError(`${message}: ${error.message}`, error);
    } else if (typeof error === 'string') {
      return new WasmError(`${message}: ${error}`);
    } else {
      return new WasmError(message, error);
    }
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

/**
 * Singleton instance of WasmCore for application-wide use.
 */
export const wasmCore = new WasmCore();
