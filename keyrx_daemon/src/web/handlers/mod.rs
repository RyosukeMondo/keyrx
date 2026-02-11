//! RPC method handlers.
//!
//! This module contains all RPC method implementations organized by domain.
//! Each handler module provides methods that accept parameters as serde_json::Value,
//! validate inputs, and delegate to service layer for business logic.

pub mod config;
pub mod daemon;
pub mod device;
pub mod metric;
pub mod profile;
pub mod setting;
