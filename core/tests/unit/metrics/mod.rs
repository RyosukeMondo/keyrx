//! Integration tests for the metrics system.
//!
//! This module contains comprehensive tests verifying:
//! - Histogram accuracy and percentile calculations
//! - Memory tracking and leak detection
//! - Profile points and hot spot identification
//! - Full collector integration and performance

mod collector_tests;
mod latency_tests;
mod memory_tests;
mod profile_tests;
