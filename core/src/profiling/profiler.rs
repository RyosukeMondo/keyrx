//! Core profiling infrastructure for performance analysis
//!
//! This module provides the main `Profiler` struct that coordinates
//! stack sampling and allocation tracking.

use std::time::Duration;

use super::sampler::StackSampler;
use crate::error::KeyRxError;

/// Configuration for the profiler
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Enable stack sampling for flame graphs
    pub stack_sampling: bool,
    /// Sample rate for stack sampling (e.g., every 10ms)
    pub sample_rate: Duration,
    /// Enable allocation tracking for memory profiling
    pub allocation_tracking: bool,
    /// Minimum allocation size to track (in bytes)
    pub allocation_threshold: usize,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            stack_sampling: false,
            sample_rate: Duration::from_millis(10),
            allocation_tracking: false,
            allocation_threshold: 1024, // 1KB
        }
    }
}

/// Result of a profiling session
#[derive(Debug, Clone)]
pub struct ProfileResult {
    /// Stack samples collected during profiling
    pub stack_samples: Vec<StackSample>,
    /// Duration of the profiling session
    pub duration: Duration,
    /// Number of samples collected
    pub sample_count: usize,
}

/// A single stack sample captured during profiling
#[derive(Debug, Clone)]
pub struct StackSample {
    /// Timestamp when this sample was taken
    pub timestamp: Duration,
    /// Stack frames from bottom to top
    pub frames: Vec<String>,
}

/// Core profiling coordinator
///
/// The `Profiler` manages the lifecycle of profiling operations,
/// including starting/stopping sampling and collecting results.
pub struct Profiler {
    config: ProfilerConfig,
    sampler: Option<StackSampler>,
    start_time: Option<std::time::Instant>,
}

impl Profiler {
    /// Create a new profiler with the given configuration
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            config,
            sampler: None,
            start_time: None,
        }
    }

    /// Start profiling
    ///
    /// This initializes the stack sampler (if enabled) and begins
    /// collecting performance data.
    pub fn start(&mut self) -> Result<(), KeyRxError> {
        self.start_time = Some(std::time::Instant::now());

        if self.config.stack_sampling {
            let mut sampler = StackSampler::new(self.config.sample_rate);
            sampler.start()?;
            self.sampler = Some(sampler);
        }

        Ok(())
    }

    /// Stop profiling and return results
    ///
    /// This stops all sampling and returns the collected data.
    pub fn stop(&mut self) -> Result<ProfileResult, KeyRxError> {
        let duration = self
            .start_time
            .map(|start| start.elapsed())
            .ok_or_else(|| KeyRxError::platform("Profiler was not started"))?;

        let stack_samples = if let Some(sampler) = &mut self.sampler {
            sampler.stop()?
        } else {
            Vec::new()
        };

        let sample_count = stack_samples.len();

        self.sampler = None;
        self.start_time = None;

        Ok(ProfileResult {
            stack_samples,
            duration,
            sample_count,
        })
    }

    /// Check if profiling is currently active
    pub fn is_active(&self) -> bool {
        self.start_time.is_some()
    }

    /// Get the current configuration
    pub fn config(&self) -> &ProfilerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_lifecycle() {
        let config = ProfilerConfig {
            stack_sampling: false,
            ..Default::default()
        };

        let mut profiler = Profiler::new(config);
        assert!(!profiler.is_active());

        profiler.start().expect("Failed to start profiler");
        assert!(profiler.is_active());

        let result = profiler.stop().expect("Failed to stop profiler");
        assert!(!profiler.is_active());
        assert!(result.duration > Duration::from_nanos(0));
    }

    #[test]
    fn test_stop_without_start() {
        let config = ProfilerConfig::default();
        let mut profiler = Profiler::new(config);

        let result = profiler.stop();
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config() {
        let config = ProfilerConfig::default();
        assert!(!config.stack_sampling);
        assert!(!config.allocation_tracking);
        assert_eq!(config.sample_rate, Duration::from_millis(10));
        assert_eq!(config.allocation_threshold, 1024);
    }
}
