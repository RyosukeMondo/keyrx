//! Key name and prefix validation - delegates to keyrx_core (SSOT).
//!
//! All validator functions are defined in keyrx_core::parser::validators.
//! This module re-exports them so compiler code can use them unchanged.

pub use keyrx_core::parser::validators::{
    parse_condition_string, parse_lock_id, parse_modifier_id, parse_physical_key, parse_virtual_key,
};
