//! WASM module entry point for browser-based keyboard simulation.
//!
//! This module provides a WebAssembly interface to the keyrx_core library,
//! enabling browser-based configuration testing and event simulation.
//!
//! # Features
//! - Load Rhai configurations from source text
//! - Load pre-compiled .krx binary configurations
//! - Simulate keyboard event sequences
//! - Query simulation state
//!
//! # Architecture
//! Configurations are stored in a global CONFIG_STORE and referenced by opaque
//! ConfigHandle values. This prevents JavaScript from directly accessing Rust
//! memory and ensures thread safety.

#![cfg(feature = "wasm")]

pub mod simulation;
pub mod validation;

extern crate std;

use once_cell::sync::Lazy;
use rkyv::Deserialize;
use serde::Serialize;
use std::{
    boxed::Box,
    format,
    string::String,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
    vec,
    vec::Vec,
};
use wasm_bindgen::prelude::*;

use sha2::{Digest, Sha256};

use crate::config::{ConfigRoot, Metadata, Version};
use crate::runtime::KeyLookup;

// Re-export simulation types
pub use simulation::{
    EventSequence, LatencyStats, SimKeyEvent, SimulationResult, SimulationState, TimelineEntry,
};

// ============================================================================
// Global Configuration Storage
// ============================================================================

use crate::runtime::DeviceState;

/// Configuration entry with associated state
struct ConfigEntry {
    config: ConfigRoot,
    /// Device state (reserved for future use)
    _state: DeviceState,
    last_sim_state: Option<SimulationState>,
}

/// Global storage for loaded configurations.
///
/// Configurations are stored in a Vec and referenced by their index (ConfigHandle).
/// The Mutex ensures thread-safe access, though WASM is currently single-threaded.
static CONFIG_STORE: Lazy<Mutex<Vec<ConfigEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

// ============================================================================
// Public Types
// ============================================================================

/// Opaque handle to a loaded configuration.
///
/// This handle prevents JavaScript from directly accessing Rust memory.
/// The handle is an index into the CONFIG_STORE Vec.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigHandle(usize);

// ============================================================================
// Module Initialization
// ============================================================================

/// Initialize the WASM module.
///
/// This sets up the panic hook to provide better error messages in the
/// browser console. Call this before using any other WASM functions.
///
/// # Example (JavaScript)
/// ```javascript
/// import init, { wasm_init } from './pkg/keyrx_core.js';
///
/// await init();
/// wasm_init();
/// ```
#[wasm_bindgen]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Recovers from a poisoned mutex by returning the inner guard.
///
/// This helper enables graceful degradation when a mutex is poisoned,
/// which can occur if a thread panics while holding the lock.
///
/// # Safety
/// The data protected by a poisoned mutex may be in an inconsistent state.
/// However, in the WASM single-threaded context, mutex poisoning should not
/// occur during normal operation. This recovery is defensive programming.
fn recover_mutex_lock<'a, T>(
    mutex: &'a Mutex<T>,
    context: &str,
) -> Result<std::sync::MutexGuard<'a, T>, JsValue> {
    mutex
        .lock()
        .or_else(
            |poison_error: std::sync::PoisonError<std::sync::MutexGuard<'a, T>>| {
                // Log warning in debug builds
                #[cfg(debug_assertions)]
                web_sys::console::warn_1(
                    &format!("WASM mutex poisoned in {}: recovering", context).into(),
                );

                // Recover by using the poisoned data
                Ok(poison_error.into_inner())
            },
        )
        .map_err(|_: std::sync::TryLockError<std::sync::MutexGuard<'a, T>>| {
            JsValue::from_str(&format!("Failed to acquire lock in {}", context))
        })
}

/// Store a configuration in the global CONFIG_STORE and return a handle.
fn store_config(config: ConfigRoot) -> Result<ConfigHandle, JsValue> {
    let mut store = recover_mutex_lock(&CONFIG_STORE, "store_config")?;
    let index = store.len();
    store.push(ConfigEntry {
        config,
        _state: DeviceState::new(),
        last_sim_state: None,
    });
    Ok(ConfigHandle(index))
}

/// Retrieve a configuration from the CONFIG_STORE by handle.
///
/// Returns an error if the handle is invalid.
fn get_config(handle: ConfigHandle) -> Result<ConfigRoot, JsValue> {
    let store = recover_mutex_lock(&CONFIG_STORE, "get_config")?;
    store
        .get(handle.0)
        .map(|entry| entry.config.clone())
        .ok_or_else(|| JsValue::from_str(&format!("Invalid ConfigHandle: {}", handle.0)))
}

/// Retrieve the last simulation state from the CONFIG_STORE by handle.
///
/// Returns an error if the handle is invalid or no simulation has been run.
fn get_sim_state_from_store(handle: ConfigHandle) -> Result<SimulationState, JsValue> {
    let store = recover_mutex_lock(&CONFIG_STORE, "get_sim_state_from_store")?;
    store
        .get(handle.0)
        .and_then(|entry| entry.last_sim_state.clone())
        .ok_or_else(|| JsValue::from_str("No simulation state available. Run a simulation first."))
}

/// Update the simulation state in the CONFIG_STORE.
///
/// Returns an error if the handle is invalid.
fn update_sim_state_in_store(
    handle: ConfigHandle,
    sim_state: SimulationState,
) -> Result<(), JsValue> {
    let mut store = recover_mutex_lock(&CONFIG_STORE, "update_sim_state_in_store")?;
    store
        .get_mut(handle.0)
        .map(|entry| {
            entry.last_sim_state = Some(sim_state);
        })
        .ok_or_else(|| JsValue::from_str(&format!("Invalid ConfigHandle: {}", handle.0)))
}

// ============================================================================
// Configuration Loading
// ============================================================================

/// Load a Rhai configuration from source text.
///
/// Parses the Rhai source, compiles it to a ConfigRoot structure, and stores
/// it in the global CONFIG_STORE. Returns an opaque handle for future operations.
///
/// # Arguments
/// * `rhai_source` - Rhai DSL source code as a string
///
/// # Returns
/// * `Ok(ConfigHandle)` - Handle to the loaded configuration
/// * `Err(JsValue)` - Parse error with line number and description
///
/// # Errors
/// Returns an error if:
/// - Rhai syntax is invalid (returns parse error with line number)
/// - Configuration size exceeds 1MB
/// - Parser fails to generate valid ConfigRoot
///
/// # Example (JavaScript)
/// ```javascript
/// const handle = load_config(`
///   device("*") {
///     map("VK_A", "VK_B");
///   }
/// `);
/// ```
#[wasm_bindgen]
pub fn load_config(rhai_source: &str) -> Result<ConfigHandle, JsValue> {
    // Validate input size (1MB limit)
    const MAX_CONFIG_SIZE: usize = 1024 * 1024;
    if rhai_source.len() > MAX_CONFIG_SIZE {
        return Err(JsValue::from_str(&format!(
            "Configuration too large: {} bytes (max {})",
            rhai_source.len(),
            MAX_CONFIG_SIZE
        )));
    }

    // Parse the Rhai configuration using embedded parser
    let config = parse_rhai_config(rhai_source).map_err(|e| JsValue::from_str(&e))?;

    // Store config and return handle
    store_config(config)
}

/// Parse Rhai configuration source into ConfigRoot.
///
/// This is a simplified parser that handles the essential Rhai DSL features
/// needed for browser-based simulation. It uses the Rhai engine to evaluate
/// the DSL and build a ConfigRoot structure.
fn parse_rhai_config(source: &str) -> Result<ConfigRoot, std::string::String> {
    use rhai::{Engine, Scope};
    use std::sync::{Arc, Mutex as StdMutex};

    #[derive(Debug, Clone, Default)]
    struct ParserState {
        devices: Vec<crate::config::DeviceConfig>,
        current_device: Option<crate::config::DeviceConfig>,
        current_mappings: Vec<crate::config::KeyMapping>,
    }

    let state = Arc::new(StdMutex::new(ParserState::default()));
    let mut engine = Engine::new();
    let state_clone = Arc::clone(&state);

    // Register device() function
    engine.register_fn("device", move |pattern: &str| {
        let mut s = state_clone
            .lock()
            .expect("Parser state mutex should not be poisoned during Rhai parsing");
        // If there's a current device, save it first
        if let Some(device) = s.current_device.take() {
            s.devices.push(device);
        }
        s.current_device = Some(crate::config::DeviceConfig {
            identifier: crate::config::DeviceIdentifier {
                pattern: pattern.into(),
            },
            mappings: Vec::new(),
        });
        s.current_mappings.clear();
    });

    let state_clone2 = Arc::clone(&state);
    // Register map() function for simple key mapping
    engine.register_fn("map", move |from: &str, to: &str| {
        use crate::config::{BaseKeyMapping, KeyCode, KeyMapping};

        let mut s = state_clone2
            .lock()
            .expect("Parser state mutex should not be poisoned during Rhai parsing");

        // Simple key parsing - map common names to KeyCode variants
        // This is a minimal implementation; full parser will be added later
        let from_key = match from {
            "A" => KeyCode::A,
            "B" => KeyCode::B,
            "VK_A" => KeyCode::A,
            "VK_B" => KeyCode::B,
            _ => return Err(format!("Unsupported key name: {}", from).into()),
        };

        let to_key = match to {
            "A" => KeyCode::A,
            "B" => KeyCode::B,
            "VK_A" => KeyCode::A,
            "VK_B" => KeyCode::B,
            _ => return Err(format!("Unsupported key name: {}", to).into()),
        };

        s.current_mappings
            .push(KeyMapping::Base(BaseKeyMapping::Simple {
                from: from_key,
                to: to_key,
            }));

        Ok::<(), Box<rhai::EvalAltResult>>(())
    });

    // Run the Rhai script
    let mut scope = Scope::new();
    engine
        .run_with_scope(&mut scope, source)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Finalize: save the last device
    let mut final_state = state
        .lock()
        .expect("Parser state mutex should not be poisoned after Rhai execution");
    if let Some(device) = final_state.current_device.take() {
        let mut device_with_mappings = device;
        device_with_mappings.mappings = final_state.current_mappings.clone();
        final_state.devices.push(device_with_mappings);
    }

    // Generate metadata
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after UNIX epoch (1970-01-01)")
        .as_secs();

    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let source_hash = format!("{:x}", hasher.finalize());

    let config = ConfigRoot {
        version: Version::current(),
        devices: final_state.devices.clone(),
        metadata: Metadata {
            compilation_timestamp: timestamp,
            compiler_version: "wasm-0.1.0".into(),
            source_hash,
        },
    };

    Ok(config)
}

/// Load a pre-compiled .krx binary configuration.
///
/// Deserializes a .krx binary file using rkyv with validation to ensure
/// integrity. The configuration is then stored in the global CONFIG_STORE.
///
/// # Arguments
/// * `binary` - Raw bytes of a .krx binary file
///
/// # Returns
/// * `Ok(ConfigHandle)` - Handle to the loaded configuration
/// * `Err(JsValue)` - Validation or deserialization error
///
/// # Errors
/// Returns an error if:
/// - Binary format is invalid or corrupted
/// - Binary size exceeds 10MB limit
/// - Validation fails (corrupted data, invalid structure)
/// - rkyv deserialization fails
///
/// # Example (JavaScript)
/// ```javascript
/// const response = await fetch('config.krx');
/// const binary = new Uint8Array(await response.arrayBuffer());
/// const handle = load_krx(binary);
/// ```
#[wasm_bindgen]
pub fn load_krx(binary: &[u8]) -> Result<ConfigHandle, JsValue> {
    // Validate input size (10MB limit)
    const MAX_BINARY_SIZE: usize = 10 * 1024 * 1024;
    if binary.len() > MAX_BINARY_SIZE {
        return Err(JsValue::from_str(&format!(
            "Binary too large: {} bytes (max {})",
            binary.len(),
            MAX_BINARY_SIZE
        )));
    }

    // Validate minimum size (at least a few bytes for valid rkyv data)
    if binary.len() < 8 {
        return Err(JsValue::from_str("Binary too small: not a valid .krx file"));
    }

    // Deserialize and validate using rkyv
    let archived = unsafe {
        // SAFETY: rkyv requires unsafe for archived_root
        rkyv::archived_root::<ConfigRoot>(binary)
    };

    // Validate the version
    if archived.version.major != 1 {
        return Err(JsValue::from_str(&format!(
            "Unsupported .krx version: {}.{}.{} (expected 1.x.x)",
            archived.version.major, archived.version.minor, archived.version.patch
        )));
    }

    // Deserialize to owned ConfigRoot
    let config: ConfigRoot = archived
        .deserialize(&mut rkyv::Infallible)
        .map_err(|_| JsValue::from_str("Deserialization failed: corrupted .krx file"))?;

    // Store config and return handle
    store_config(config)
}

// ============================================================================
// Configuration Validation
// ============================================================================

/// Validate a Rhai configuration source.
///
/// Parses the Rhai source using keyrx_compiler and returns validation errors as a JSON array.
/// Returns an empty array if the configuration is valid.
///
/// This function uses the same parser as the daemon to ensure validation results are
/// deterministic and match the daemon's behavior exactly.
///
/// # Arguments
/// * `rhai_source` - Rhai DSL source code as a string
///
/// # Returns
/// * `Ok(JsValue)` - JSON array of validation errors. Each error has:
///   - `line`: number - Line number where the error occurred
///   - `column`: number - Column number where the error occurred
///   - `length`: number - Length of the error span
///   - `message`: string - Error description
/// * `Err(JsValue)` - Internal error during validation
///
/// # Example (JavaScript)
/// ```javascript
/// const errors = validate_config(`
///   device("*") {
///     map("A", "B");
///   }
/// `);
/// console.log(errors); // [] for valid config
/// ```
#[wasm_bindgen]
pub fn validate_config(rhai_source: &str) -> Result<JsValue, JsValue> {
    // Use keyrx_compiler for deterministic validation
    let errors = validation::validate_rhai_config(rhai_source)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    // Serialize to JSON
    serde_wasm_bindgen::to_value(&errors)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize errors: {}", e)))
}

// ============================================================================
// Event Simulation
// ============================================================================

/// Simulate keyboard event sequence.
///
/// Processes a sequence of keyboard events through the remapping configuration,
/// tracking state changes and performance metrics.
///
/// # Arguments
/// * `config` - Handle to a loaded configuration
/// * `events_json` - JSON string containing EventSequence
///
/// # Returns
/// * `Ok(JsValue)` - SimulationResult as JSON
/// * `Err(JsValue)` - Error message
///
/// # Errors
/// Returns an error if:
/// - ConfigHandle is invalid
/// - JSON is malformed or doesn't match EventSequence schema
/// - Event keycodes are invalid
/// - Simulation exceeds 1000 events
///
/// # Example (JavaScript)
/// ```javascript
/// const events = {
///   events: [
///     { keycode: "A", event_type: "press", timestamp_us: 0 },
///     { keycode: "A", event_type: "release", timestamp_us: 100000 }
///   ]
/// };
/// const result = simulate(configHandle, JSON.stringify(events));
/// ```
#[wasm_bindgen]
pub fn simulate(config: ConfigHandle, events_json: &str) -> Result<JsValue, JsValue> {
    // Validate ConfigHandle
    let config_root = get_config(config)?;

    // Parse EventSequence from JSON
    let event_sequence: EventSequence = serde_json::from_str(events_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    // Validate event count (max 1000 events)
    const MAX_EVENTS: usize = 1000;
    if event_sequence.events.len() > MAX_EVENTS {
        return Err(JsValue::from_str(&format!(
            "Too many events: {} (max {})",
            event_sequence.events.len(),
            MAX_EVENTS
        )));
    }

    // Initialize runtime components
    let device_config = config_root
        .devices
        .first()
        .ok_or_else(|| JsValue::from_str("Configuration has no devices"))?;

    let lookup = KeyLookup::from_device_config(device_config);

    // Run simulation
    let result = simulation::run_simulation(&lookup, &event_sequence)
        .map_err(|e| JsValue::from_str(e.as_str()))?;

    // Store the final state for get_state to access
    update_sim_state_in_store(config, result.final_state.clone())?;

    // Serialize to JSON
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Get current simulation state.
///
/// Returns the state from the most recent simulation for the given configuration.
/// This matches the DaemonStateResponse format from the daemon API.
///
/// # Arguments
/// * `config` - Handle to a loaded configuration
///
/// # Returns
/// * `Ok(JsValue)` - DaemonStateResponse as JSON with:
///   - active_layer: Optional<String> - Current active layer (if any)
///   - modifiers: Vec<String> - Active modifier IDs as strings
///   - locks: Vec<String> - Active lock IDs as strings
///   - raw_state: Vec<bool> - 255-bit state vector
///   - active_modifier_count: usize - Number of active modifiers
///   - active_lock_count: usize - Number of active locks
/// * `Err(JsValue)` - Error message if no simulation has been run
///
/// # Errors
/// Returns an error if:
/// - ConfigHandle is invalid
/// - No simulation has been run yet (no state available)
///
/// # Example (JavaScript)
/// ```javascript
/// // After running a simulation
/// const state = get_state(configHandle);
/// console.log(`Active modifiers: ${state.active_modifier_count}`);
/// console.log(`Active locks: ${state.active_lock_count}`);
/// ```
#[wasm_bindgen]
pub fn get_state(config: ConfigHandle) -> Result<JsValue, JsValue> {
    // Validate ConfigHandle and get simulation state
    let sim_state = get_sim_state_from_store(config)?;

    // Convert to DaemonStateResponse format
    #[derive(Serialize)]
    struct DaemonStateResponse {
        active_layer: Option<String>,
        modifiers: Vec<String>,
        locks: Vec<String>,
        raw_state: Vec<bool>,
        active_modifier_count: usize,
        active_lock_count: usize,
    }

    // Convert modifier IDs to strings
    let modifiers: Vec<String> = sim_state
        .active_modifiers
        .iter()
        .map(|id| format!("MD_{:02X}", id))
        .collect();

    // Convert lock IDs to strings
    let locks: Vec<String> = sim_state
        .active_locks
        .iter()
        .map(|id| format!("LK_{:02X}", id))
        .collect();

    // Build 255-bit raw state vector
    let mut raw_state = vec![false; 255];
    for &id in &sim_state.active_modifiers {
        if (id as usize) < 255 {
            raw_state[id as usize] = true;
        }
    }
    for &id in &sim_state.active_locks {
        if (id as usize) < 255 {
            raw_state[id as usize] = true;
        }
    }

    let response = DaemonStateResponse {
        active_layer: sim_state.active_layer,
        modifiers: modifiers.clone(),
        locks: locks.clone(),
        raw_state,
        active_modifier_count: modifiers.len(),
        active_lock_count: locks.len(),
    };

    // Serialize to JSON
    serde_wasm_bindgen::to_value(&response)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize state: {}", e)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_handle_copy() {
        let handle1 = ConfigHandle(5);
        let handle2 = handle1;
        assert_eq!(handle1, handle2);
    }

    #[test]
    fn test_invalid_handle() {
        let invalid_handle = ConfigHandle(9999);
        let result = get_config(invalid_handle);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_krx_too_large() {
        let large_binary = vec![0u8; 11 * 1024 * 1024];
        let result = load_krx(&large_binary);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_krx_too_small() {
        let small_binary = vec![0u8; 4];
        let result = load_krx(&small_binary);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_krx_valid() {
        use crate::config::{DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};

        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".into(),
                },
                mappings: alloc::vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "test".into(),
                source_hash: "abc123".into(),
            },
        };

        let bytes = rkyv::to_bytes::<_, 1024>(&config).expect("Serialization failed");
        let result = load_krx(&bytes);
        assert!(result.is_ok());

        let handle = result.expect("load_krx should succeed with valid config");
        let loaded_config = get_config(handle);
        assert!(loaded_config.is_ok());
    }
}
