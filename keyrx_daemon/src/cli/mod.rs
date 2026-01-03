//! CLI command implementations.
//!
//! This module contains all CLI commands for device management, profile management,
//! configuration, layers, layouts, simulation, and monitoring.

pub mod common;
pub mod config;
pub mod config_handlers;
pub mod config_helpers;
pub mod devices;
pub mod error;
pub mod layers;
pub mod layouts;
pub mod logging;
pub mod metrics;
pub mod profiles;
pub mod simulate;
pub mod state;
pub mod status;
pub mod test;
