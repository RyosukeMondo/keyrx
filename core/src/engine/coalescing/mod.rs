//! Event coalescing layer for batching and optimizing input processing.
//!
//! This module provides an event buffer that batches input events to reduce
//! FFI overhead and improve throughput during rapid typing. Events are flushed
//! based on time windows, batch size limits, or modifier state changes.

mod buffer;
mod config;

pub use buffer::EventBuffer;
pub use config::CoalescingConfig;
