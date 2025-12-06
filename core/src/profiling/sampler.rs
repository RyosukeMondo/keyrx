//! Stack sampling infrastructure for low-overhead profiling
//!
//! This module implements a stack sampler that periodically captures
//! the call stack at configurable intervals.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::KeyRxError;

use super::profiler::StackSample;

/// Stack sampler for collecting call stacks at regular intervals
///
/// The sampler runs in a background thread and periodically captures
/// the current call stack. It's designed for low overhead (<10%).
#[allow(clippy::expect_used)] // Poisoned mutex is unrecoverable
pub struct StackSampler {
    sample_rate: Duration,
    samples: Arc<Mutex<Vec<StackSample>>>,
    stop_signal: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl StackSampler {
    /// Create a new stack sampler with the given sample rate
    pub fn new(sample_rate: Duration) -> Self {
        Self {
            sample_rate,
            samples: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(Mutex::new(false)),
            thread_handle: None,
        }
    }

    /// Start sampling in a background thread
    #[allow(clippy::expect_used)] // Poisoned mutex is unrecoverable
    pub fn start(&mut self) -> Result<(), KeyRxError> {
        if self.thread_handle.is_some() {
            return Err(KeyRxError::platform("Sampler already started"));
        }

        let sample_rate = self.sample_rate;
        let samples = Arc::clone(&self.samples);
        let stop_signal = Arc::clone(&self.stop_signal);

        let handle = thread::spawn(move || {
            let start_time = std::time::Instant::now();

            loop {
                // Check if we should stop
                {
                    let should_stop = stop_signal.lock().expect("stop_signal mutex poisoned");
                    if *should_stop {
                        break;
                    }
                }

                // Capture a stack sample
                let timestamp = start_time.elapsed();
                let frames = Self::capture_stack();
                let sample = StackSample { timestamp, frames };

                // Store the sample
                {
                    let mut samples_guard = samples.lock().expect("samples mutex poisoned");
                    samples_guard.push(sample);
                }

                // Sleep until next sample
                thread::sleep(sample_rate);
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop sampling and return collected samples
    #[allow(clippy::expect_used)] // Poisoned mutex is unrecoverable
    pub fn stop(&mut self) -> Result<Vec<StackSample>, KeyRxError> {
        // Signal the sampling thread to stop
        {
            let mut stop = self.stop_signal.lock().expect("stop_signal mutex poisoned");
            *stop = true;
        }

        // Wait for the thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| KeyRxError::platform("Failed to join sampler thread"))?;
        }

        // Collect samples
        let mut samples_guard = self.samples.lock().expect("samples mutex poisoned");
        let samples = samples_guard.drain(..).collect();

        // Reset stop signal for potential restart
        {
            let mut stop = self.stop_signal.lock().expect("stop_signal mutex poisoned");
            *stop = false;
        }

        Ok(samples)
    }

    /// Capture the current call stack
    ///
    /// This uses backtrace to capture the current stack.
    /// In a production implementation, this would use more efficient
    /// methods like unwinding or sampling-based profiling.
    fn capture_stack() -> Vec<String> {
        // For now, we'll use a simple backtrace approach
        // In production, this could use:
        // - DWARF unwinding for lower overhead
        // - Sampling-based profiling (perf_event_open on Linux)
        // - Platform-specific APIs (instruments on macOS, ETW on Windows)

        let trace_string = std::panic::catch_unwind(backtrace::Backtrace::new)
            .map(|bt| format!("{:?}", bt))
            .unwrap_or_else(|_| "<backtrace unavailable>".to_string());
        let frames: Vec<String> = trace_string
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Self::symbolicate_frame)
            .collect();

        frames
    }

    /// Symbolicate a stack frame for better readability
    fn symbolicate_frame(frame: &str) -> String {
        // Extract function name from backtrace line
        // Format is typically: "  at <function_name> (<file>:<line>)"
        let trimmed = frame.trim();

        // Simple extraction - in production, use proper symbolication
        if let Some(at_pos) = trimmed.find("at ") {
            trimmed[at_pos + 3..].to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Get the current number of collected samples
    #[allow(clippy::expect_used)] // Poisoned mutex is unrecoverable
    pub fn sample_count(&self) -> usize {
        self.samples.lock().expect("samples mutex poisoned").len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_lifecycle() {
        let mut sampler = StackSampler::new(Duration::from_millis(10));

        sampler.start().expect("Failed to start sampler");

        // Let it collect some samples
        thread::sleep(Duration::from_millis(50));

        let samples = sampler.stop().expect("Failed to stop sampler");

        // Should have collected at least a few samples
        assert!(samples.len() > 0, "Expected at least one sample");
        assert!(samples.len() < 10, "Expected fewer than 10 samples");
    }

    #[test]
    fn test_double_start() {
        let mut sampler = StackSampler::new(Duration::from_millis(10));

        sampler.start().expect("Failed to start sampler");
        let result = sampler.start();

        assert!(result.is_err());

        sampler.stop().expect("Failed to stop sampler");
    }

    #[test]
    fn test_stop_without_start() {
        let mut sampler = StackSampler::new(Duration::from_millis(10));
        let samples = sampler.stop().expect("Stop should work even without start");
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_capture_stack() {
        let frames = StackSampler::capture_stack();
        assert!(!frames.is_empty(), "Should capture at least one frame");
    }

    #[test]
    fn test_symbolicate_frame() {
        let frame = "  at keyrx_core::profiling::sampler::test";
        let symbolicated = StackSampler::symbolicate_frame(frame);
        assert!(symbolicated.contains("keyrx_core::profiling::sampler::test"));
    }
}
