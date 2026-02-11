//! Rhai DSL parser for keyrx configuration.
//!
//! The `error` and `validators` submodules are always available (no external deps).
//! The full `Parser` type requires the `wasm` feature (rhai, sha2, spin).
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::parser::Parser;
//!
//! let source = r#"
//!     device_start("*");
//!     map("VK_A", "VK_B");
//!     device_end();
//! "#;
//!
//! let parser = Parser::new();
//! let config = parser.parse_string(source)?;
//! ```

// Always available - no external dependencies
pub mod error;
pub mod validators;

// Full parser - requires rhai, sha2, spin
#[cfg(feature = "wasm")]
pub mod functions;
#[cfg(feature = "wasm")]
pub mod state;

#[cfg(feature = "wasm")]
use alloc::format;
#[cfg(feature = "wasm")]
use alloc::string::{String, ToString};
#[cfg(feature = "wasm")]
use alloc::sync::Arc;
#[cfg(feature = "wasm")]
use alloc::vec::Vec;
#[cfg(feature = "wasm")]
use rhai::{Engine, Scope};
#[cfg(feature = "wasm")]
use sha2::{Digest, Sha256};
#[cfg(feature = "wasm")]
use spin::Mutex;

#[cfg(feature = "wasm")]
use crate::config::{ConfigRoot, Metadata, Version};
#[cfg(feature = "wasm")]
use state::ParserState;

/// Main parser for Rhai DSL.
#[cfg(feature = "wasm")]
pub struct Parser {
    engine: Engine,
    state: Arc<Mutex<ParserState>>,
}

#[cfg(feature = "wasm")]
impl Parser {
    /// Create a new parser with all functions registered.
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let state = Arc::new(Mutex::new(ParserState::new()));

        // Set resource limits
        engine.set_max_operations(100_000);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(100);

        // Register all DSL functions
        functions::device::register_device_functions(&mut engine, Arc::clone(&state));
        functions::map::register_map_functions(&mut engine, Arc::clone(&state));
        functions::tap_hold::register_tap_hold_function(&mut engine, Arc::clone(&state));
        functions::conditional::register_when_functions(&mut engine, Arc::clone(&state));
        functions::modifiers::register_modifier_functions(&mut engine);

        Self { engine, state }
    }

    /// Parse a Rhai script string into a ConfigRoot.
    pub fn parse_string(&self, script: &str) -> Result<ConfigRoot, String> {
        // Reset state for new parse
        {
            let mut state = self.state.lock();
            *state = ParserState::new();
        }

        // Run the script
        let mut scope = Scope::new();
        self.engine
            .run_with_scope(&mut scope, script)
            .map_err(|e| format!("Parse error: {}", e))?;

        // Finalize the configuration
        self.finalize_config(script)
    }

    /// Finalize the parsed configuration.
    fn finalize_config(&self, source: &str) -> Result<ConfigRoot, String> {
        let state = self.state.lock();

        // Check for unclosed device block
        if state.current_device.is_some() {
            return Err("Unclosed device_start() block - missing device_end()".to_string());
        }

        // Check for unclosed conditional blocks
        if !state.conditional_stack.is_empty() {
            return Err("Unclosed when_start() block - missing when_end()".to_string());
        }

        // Calculate SHA256 hash of source script
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let hash_result = hasher.finalize();
        let source_hash = format!("{:x}", hash_result);

        // Get current timestamp (0 for WASM/no_std since we don't have reliable time)
        let compilation_timestamp = 0u64;

        let metadata = Metadata {
            compilation_timestamp,
            compiler_version: "keyrx-core-0.1.0".to_string(),
            source_hash,
        };

        Ok(ConfigRoot {
            version: Version::current(),
            devices: state.devices.clone(),
            metadata,
        })
    }

    /// Validate a Rhai script without returning the configuration.
    /// Returns a list of validation errors (empty if valid).
    pub fn validate(&self, script: &str) -> Vec<error::ParseError> {
        match self.parse_string(script) {
            Ok(_) => Vec::new(),
            Err(msg) => alloc::vec![error::ParseError::Other(msg)],
        }
    }
}

#[cfg(feature = "wasm")]
impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
