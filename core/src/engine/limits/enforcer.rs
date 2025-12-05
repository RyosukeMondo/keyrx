//! Runtime resource enforcement for engine operations.
//!
//! Provides atomic tracking for execution timeout, memory usage, and queue
//! depth. Designed to be lightweight and reusable across the engine.

use super::config::ResourceLimits;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::warn;

/// Resource enforcement failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResourceLimitError {
    #[error("Execution timeout exceeded ({elapsed:?}/{timeout:?})")]
    Timeout {
        elapsed: Duration,
        timeout: Duration,
    },
    #[error("Memory limit exceeded ({used}/{limit} bytes)")]
    Memory { used: usize, limit: usize },
    #[error("Queue limit exceeded ({depth}/{limit} items)")]
    Queue { depth: usize, limit: usize },
}

/// Snapshot of current tracked usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceUsageSnapshot {
    pub memory_used: usize,
    pub memory_limit: usize,
    pub queue_depth: usize,
    pub queue_limit: usize,
    pub timeout: Duration,
}

/// Centralized resource enforcement.
#[derive(Debug)]
#[allow(dead_code)] // Will be used in subsequent integration tasks
pub struct ResourceEnforcer {
    timeout: Duration,
    memory_limit: usize,
    queue_limit: usize,
    current_memory: AtomicUsize,
    queue_depth: AtomicUsize,
}

impl ResourceEnforcer {
    /// Create a new enforcer with explicit limits.
    pub fn new(config: ResourceLimits) -> Self {
        Self {
            timeout: config.execution_timeout,
            memory_limit: config.memory_limit,
            queue_limit: config.queue_limit,
            current_memory: AtomicUsize::new(0),
            queue_depth: AtomicUsize::new(0),
        }
    }

    /// Start execution tracking; checks timeout on drop.
    pub fn start_execution(&self) -> ExecutionGuard<'_> {
        ExecutionGuard {
            enforcer: self,
            start: Instant::now(),
        }
    }

    /// Record a memory allocation, enforcing the configured limit.
    pub fn record_allocation(&self, bytes: usize) -> Result<(), ResourceLimitError> {
        if bytes == 0 {
            return Ok(());
        }

        let new_used = self.current_memory.fetch_add(bytes, Ordering::Relaxed) + bytes;
        if new_used >= self.memory_limit {
            self.current_memory.fetch_sub(bytes, Ordering::Relaxed);
            Err(ResourceLimitError::Memory {
                used: new_used,
                limit: self.memory_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Record a memory deallocation with saturating semantics.
    pub fn record_deallocation(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }

        let _ = self
            .current_memory
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(bytes))
            });
    }

    /// Increment queue depth and enforce bounds.
    pub fn increment_queue(&self) -> Result<(), ResourceLimitError> {
        let new_depth = self.queue_depth.fetch_add(1, Ordering::Relaxed) + 1;
        if new_depth > self.queue_limit {
            self.queue_depth.fetch_sub(1, Ordering::Relaxed);
            Err(ResourceLimitError::Queue {
                depth: new_depth,
                limit: self.queue_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Decrement queue depth with saturating semantics.
    pub fn decrement_queue(&self) {
        let _ = self
            .queue_depth
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(1))
            });
    }

    /// Validate current memory usage against the limit.
    pub fn check_memory(&self) -> Result<(), ResourceLimitError> {
        let used = self.current_memory.load(Ordering::Relaxed);
        if used >= self.memory_limit {
            Err(ResourceLimitError::Memory {
                used,
                limit: self.memory_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Validate current queue depth against the limit.
    pub fn check_queue(&self) -> Result<(), ResourceLimitError> {
        let depth = self.queue_depth.load(Ordering::Relaxed);
        if depth > self.queue_limit {
            Err(ResourceLimitError::Queue {
                depth,
                limit: self.queue_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Current usage snapshot for observability.
    pub fn snapshot(&self) -> ResourceUsageSnapshot {
        ResourceUsageSnapshot {
            memory_used: self.current_memory.load(Ordering::Relaxed),
            memory_limit: self.memory_limit,
            queue_depth: self.queue_depth.load(Ordering::Relaxed),
            queue_limit: self.queue_limit,
            timeout: self.timeout,
        }
    }

    /// Internal timeout validation helper.
    fn check_timeout(&self, start: Instant) -> Result<(), ResourceLimitError> {
        let elapsed = start.elapsed();
        if elapsed >= self.timeout {
            Err(ResourceLimitError::Timeout {
                elapsed,
                timeout: self.timeout,
            })
        } else {
            Ok(())
        }
    }
}

/// RAII guard for execution timing.
#[derive(Debug)]
#[allow(dead_code)] // Will be consumed by timeout enforcement in integration tasks
pub struct ExecutionGuard<'a> {
    enforcer: &'a ResourceEnforcer,
    start: Instant,
}

impl ExecutionGuard<'_> {
    /// Check timeout status manually.
    pub fn check_timeout(&self) -> Result<(), ResourceLimitError> {
        self.enforcer.check_timeout(self.start)
    }
}

impl Drop for ExecutionGuard<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.enforcer.check_timeout(self.start) {
            warn!(target: "resource_enforcer", %error, "Execution exceeded timeout");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::ResourceLimits;
    use super::*;
    use std::time::Duration;

    #[test]
    fn memory_enforcement_reverts_on_overflow() {
        let enforcer =
            ResourceEnforcer::new(ResourceLimits::new(Duration::from_millis(10), 100, 10));

        assert!(enforcer.record_allocation(60).is_ok());
        assert!(matches!(
            enforcer.record_allocation(50),
            Err(ResourceLimitError::Memory { .. })
        ));
        assert_eq!(
            enforcer.current_memory.load(Ordering::Relaxed),
            60,
            "failed allocation should revert"
        );
    }

    #[test]
    fn queue_enforcement_caps_depth() {
        let enforcer =
            ResourceEnforcer::new(ResourceLimits::new(Duration::from_millis(10), 100, 1));

        assert!(enforcer.increment_queue().is_ok());
        assert!(matches!(
            enforcer.increment_queue(),
            Err(ResourceLimitError::Queue { .. })
        ));

        enforcer.decrement_queue();
        assert_eq!(enforcer.queue_depth.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn timeout_check_triggers() {
        let enforcer =
            ResourceEnforcer::new(ResourceLimits::new(Duration::from_millis(1), 100, 10));

        let guard = enforcer.start_execution();
        std::thread::sleep(Duration::from_millis(3));
        assert!(matches!(
            guard.check_timeout(),
            Err(ResourceLimitError::Timeout { .. })
        ));
    }

    #[test]
    fn snapshot_reports_usage() {
        let enforcer = ResourceEnforcer::new(ResourceLimits::new(Duration::from_millis(5), 200, 5));

        enforcer.record_allocation(40).unwrap();
        enforcer.increment_queue().unwrap();

        let snapshot = enforcer.snapshot();
        assert_eq!(snapshot.memory_used, 40);
        assert_eq!(snapshot.queue_depth, 1);
        assert_eq!(snapshot.memory_limit, 200);
        assert_eq!(snapshot.queue_limit, 5);
        assert_eq!(snapshot.timeout, Duration::from_millis(5));
    }
}
