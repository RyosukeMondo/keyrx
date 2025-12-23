//! Runtime module for keyboard event processing
//!
//! This module provides the core runtime components for processing keyboard events:
//! - `DeviceState`: Tracks modifier and lock state (255-bit vectors)
//! - `KeyLookup`: O(1) key-to-mapping resolution using HashMap
//! - `KeyEvent`: Type-safe keyboard event representation (Press/Release)
//! - `process_event`: Core event processing logic
//!
//! # Example
//!
//! ```rust,ignore
//! use keyrx_core::runtime::{DeviceState, KeyLookup, KeyEvent, process_event};
//! use keyrx_core::config::DeviceConfig;
//!
//! // Load configuration
//! let config: DeviceConfig = /* ... */;
//!
//! // Initialize runtime components
//! let lookup = KeyLookup::from_device_config(&config);
//! let mut state = DeviceState::new();
//!
//! // Process events
//! let input = KeyEvent::Press(KeyCode::A);
//! let outputs = process_event(input, &lookup, &mut state);
//! ```

pub mod event;
pub mod lookup;
pub mod state;

// Re-export public API
pub use event::{process_event, KeyEvent, KeyEventType};
pub use lookup::KeyLookup;
pub use state::DeviceState;
