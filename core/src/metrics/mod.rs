//! Performance metrics collection system.
//!
//! This module provides a comprehensive metrics collection infrastructure for KeyRx,
//! enabling latency tracking, memory monitoring, and hot path profiling with minimal
//! overhead (< 1 microsecond per recording).
//!
//! # Architecture
//!
//! The metrics system is built around the `MetricsCollector` trait, which allows
//! for pluggable implementations:
//!
//! - **NoOpCollector**: Zero-overhead collector for release builds (inlined to nothing)
//! - **FullCollector**: Complete metrics collection with histograms and profiling
//!
//! # Design Principles
//!
//! - **Zero Allocation**: Hot path metrics use pre-allocated storage
//! - **Thread Safe**: All collectors are `Send + Sync`
//! - **Bounded Memory**: Metrics use fixed-size buffers and histograms
//! - **Pluggable**: Trait-based design allows different implementations
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::{MetricsCollector, Operation};
//!
//! // Record latency manually
//! collector.record_latency(Operation::EventProcess, 150);
//!
//! // Or use RAII guards for automatic timing
//! {
//!     let _guard = collector.start_profile("expensive_function");
//!     // ... code to profile ...
//! } // Automatically records elapsed time on drop
//!
//! // Get snapshot for export
//! let snapshot = collector.snapshot();
//! ```

pub mod collector;
pub mod full_collector;
pub mod latency;
pub mod memory;
pub mod noop_collector;
pub mod operation;
pub mod profile;
pub mod sampler;
pub mod snapshot;

// Re-export commonly used types
pub use collector::{MetricsCollector, OtelMetricsCollector, ProfileGuard};
pub use full_collector::FullMetricsCollector;
pub use latency::LatencyHistogram;
pub use memory::{MemoryMonitor, MemoryStats};
pub use noop_collector::NoOpCollector;
pub use operation::Operation;
pub use profile::{ProfilePointGuard, ProfilePoints, ProfileStats};
pub use sampler::{MemorySampler, DEFAULT_SAMPLE_INTERVAL};
pub use snapshot::{LatencyStats, MemorySnapshot, MetricsSnapshot, ProfileSnapshot};

use std::sync::Arc;

/// Create a default no-op metrics collector for test/non-production use.
///
/// This returns a zero-overhead collector wrapped in Arc for easy injection
/// into components that require metrics but don't need actual tracking.
pub fn default_noop_collector() -> Arc<dyn MetricsCollector> {
    Arc::new(NoOpCollector::new())
}
