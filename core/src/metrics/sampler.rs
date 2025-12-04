//! Periodic memory sampler for automatic tracking.
//!
//! This module provides a background task that periodically samples process memory
//! usage and records it to a MetricsCollector. The sampler runs on a separate
//! tokio task and can be started/stopped as needed.
//!
//! # Architecture
//!
//! The sampler uses the `memory-stats` crate to get cross-platform process memory
//! usage (physical RSS and virtual memory). It samples at a configurable interval
//! and records to the provided MetricsCollector.
//!
//! # Performance
//!
//! - Sampling overhead: < 1ms per sample (depends on OS)
//! - Default interval: 1 second (configurable)
//! - Runs on separate tokio task (non-blocking)
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::{MetricsCollector, FullMetricsCollector};
//! use keyrx_core::metrics::sampler::MemorySampler;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! let collector = Arc::new(FullMetricsCollector::new());
//!
//! // Start sampling every second
//! let sampler = MemorySampler::start(
//!     Arc::clone(&collector) as Arc<dyn MetricsCollector>,
//!     Duration::from_secs(1)
//! );
//!
//! // ... run application ...
//!
//! // Stop sampling
//! sampler.stop().await;
//! ```

use super::collector::MetricsCollector;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Default sampling interval (1 second).
pub const DEFAULT_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

/// Memory sampler that periodically records process memory usage.
///
/// The sampler runs on a background tokio task and can be stopped via the
/// returned handle.
pub struct MemorySampler {
    /// Handle to stop the sampling task.
    stop_tx: Option<oneshot::Sender<()>>,

    /// Join handle for the sampling task.
    task_handle: Option<JoinHandle<()>>,
}

impl MemorySampler {
    /// Start periodic memory sampling.
    ///
    /// Spawns a background tokio task that samples process memory at the
    /// specified interval and records it to the provided collector.
    ///
    /// # Arguments
    ///
    /// * `collector` - MetricsCollector to record memory samples to
    /// * `interval` - Time between samples
    ///
    /// # Returns
    ///
    /// A `MemorySampler` handle that can be used to stop sampling.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sampler = MemorySampler::start(collector, Duration::from_secs(1));
    /// ```
    pub fn start(collector: Arc<dyn MetricsCollector>, interval: Duration) -> Self {
        let (stop_tx, mut stop_rx) = oneshot::channel();

        let task_handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // Sample memory and record
                        if let Some(usage) = memory_stats::memory_stats() {
                            // Use physical memory (RSS) as the primary metric
                            collector.record_memory(usage.physical_mem);
                        }
                    }
                    _ = &mut stop_rx => {
                        // Stop signal received
                        break;
                    }
                }
            }
        });

        Self {
            stop_tx: Some(stop_tx),
            task_handle: Some(task_handle),
        }
    }

    /// Start sampling with default interval (1 second).
    ///
    /// Convenience method for starting with the default 1-second interval.
    ///
    /// # Arguments
    ///
    /// * `collector` - MetricsCollector to record memory samples to
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sampler = MemorySampler::start_default(collector);
    /// ```
    pub fn start_default(collector: Arc<dyn MetricsCollector>) -> Self {
        Self::start(collector, DEFAULT_SAMPLE_INTERVAL)
    }

    /// Stop the memory sampler and wait for it to finish.
    ///
    /// Sends a stop signal to the background task and waits for it to
    /// complete gracefully.
    ///
    /// # Example
    ///
    /// ```ignore
    /// sampler.stop().await;
    /// ```
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            // Send stop signal (ignore errors if already stopped)
            let _ = tx.send(());

            // Wait for task to complete
            if let Some(handle) = self.task_handle.take() {
                let _ = handle.await;
            }
        }
    }

    /// Stop the sampler without waiting.
    ///
    /// Sends the stop signal but doesn't wait for the task to complete.
    /// The task will still finish gracefully in the background.
    pub fn stop_nowait(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for MemorySampler {
    fn drop(&mut self) {
        // Send stop signal on drop
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::full_collector::FullMetricsCollector;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sampler_starts_and_stops() {
        let collector = Arc::new(FullMetricsCollector::new());
        let sampler = MemorySampler::start(
            Arc::clone(&collector) as Arc<dyn MetricsCollector>,
            Duration::from_millis(100),
        );

        // Let it sample a few times
        tokio::time::sleep(Duration::from_millis(350)).await;

        // Stop sampler
        sampler.stop().await;

        // Check that memory was recorded
        let stats = collector.get_memory_monitor().stats();
        assert!(stats.sample_count >= 3, "Expected at least 3 samples");
        assert!(stats.current > 0, "Expected non-zero memory usage");
    }

    #[tokio::test]
    async fn test_sampler_default_interval() {
        let collector = Arc::new(FullMetricsCollector::new());
        let sampler =
            MemorySampler::start_default(Arc::clone(&collector) as Arc<dyn MetricsCollector>);

        // Let it sample once
        tokio::time::sleep(Duration::from_millis(1100)).await;

        sampler.stop().await;

        let stats = collector.get_memory_monitor().stats();
        assert!(stats.sample_count >= 1);
    }

    #[tokio::test]
    async fn test_sampler_records_increasing_samples() {
        let collector = Arc::new(FullMetricsCollector::new());
        let sampler = MemorySampler::start(
            Arc::clone(&collector) as Arc<dyn MetricsCollector>,
            Duration::from_millis(50),
        );

        // Initial count
        tokio::time::sleep(Duration::from_millis(75)).await;
        let count1 = collector.get_memory_monitor().stats().sample_count;

        // Wait for more samples
        tokio::time::sleep(Duration::from_millis(150)).await;
        let count2 = collector.get_memory_monitor().stats().sample_count;

        sampler.stop().await;

        assert!(count2 > count1, "Sample count should increase over time");
    }

    #[tokio::test]
    async fn test_sampler_stop_nowait() {
        let collector = Arc::new(FullMetricsCollector::new());
        let sampler = MemorySampler::start(
            Arc::clone(&collector) as Arc<dyn MetricsCollector>,
            Duration::from_millis(100),
        );

        tokio::time::sleep(Duration::from_millis(250)).await;

        // Stop without waiting
        sampler.stop_nowait();

        // Should have recorded samples
        let stats = collector.get_memory_monitor().stats();
        assert!(stats.sample_count >= 2);
    }

    #[tokio::test]
    async fn test_sampler_drop_stops_task() {
        let collector = Arc::new(FullMetricsCollector::new());
        {
            let _sampler = MemorySampler::start(
                Arc::clone(&collector) as Arc<dyn MetricsCollector>,
                Duration::from_millis(100),
            );

            tokio::time::sleep(Duration::from_millis(250)).await;
        } // sampler dropped here

        // Give task time to stop
        tokio::time::sleep(Duration::from_millis(50)).await;

        let count1 = collector.get_memory_monitor().stats().sample_count;

        // Wait more, count shouldn't increase
        tokio::time::sleep(Duration::from_millis(300)).await;
        let count2 = collector.get_memory_monitor().stats().sample_count;

        // Samples might still increase by 1-2 due to timing, but shouldn't be significantly more
        assert!(count2 - count1 <= 2, "Task should have stopped after drop");
    }
}
