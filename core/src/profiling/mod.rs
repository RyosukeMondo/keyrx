//! Advanced profiling support for KeyRx
//!
//! This module provides comprehensive profiling capabilities including:
//! - Stack sampling for flame graph generation
//! - Allocation tracking for memory profiling
//! - Low-overhead collection (< 10% overhead)
//!
//! # Examples
//!
//! ```no_run
//! use keyrx_core::profiling::{Profiler, ProfilerConfig};
//! use std::time::Duration;
//!
//! let config = ProfilerConfig {
//!     stack_sampling: true,
//!     sample_rate: Duration::from_millis(10),
//!     allocation_tracking: false,
//!     allocation_threshold: 1024,
//! };
//!
//! let mut profiler = Profiler::new(config);
//! profiler.start().expect("Failed to start profiler");
//!
//! // ... run code to profile ...
//!
//! let result = profiler.stop().expect("Failed to stop profiler");
//! println!("Collected {} samples", result.sample_count);
//! ```

pub mod profiler;
pub mod sampler;

// Re-export main types
pub use profiler::{ProfileResult, Profiler, ProfilerConfig, StackSample};
pub use sampler::StackSampler;
