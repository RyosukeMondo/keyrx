use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Configuration for resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_instructions: u64,
    pub max_recursion: u32,
    pub max_memory: usize,
    pub timeout: Duration,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_instructions: 100_000,
            max_recursion: 64,
            max_memory: 1024 * 1024, // 1MB
            #[cfg(not(test))]
            timeout: Duration::from_millis(100),
            #[cfg(test)]
            timeout: Duration::from_secs(5), // More lenient for tests
        }
    }
}

/// Resource budget for script execution.
///
/// Uses atomic counters for thread-safe, low-overhead tracking of:
/// - Instruction count (operations executed)
/// - Recursion depth (call stack depth)
/// - Memory usage (bytes allocated)
/// - Execution time (elapsed duration)
pub struct ResourceBudget {
    instruction_limit: u64,
    instruction_count: AtomicU64,
    recursion_limit: u32,
    recursion_depth: AtomicU32,
    memory_limit: usize,
    memory_used: AtomicUsize,
    timeout: Duration,
    start_time: Instant,
}

impl ResourceBudget {
    /// Create a new resource budget with the given configuration.
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            instruction_limit: config.max_instructions,
            instruction_count: AtomicU64::new(0),
            recursion_limit: config.max_recursion,
            recursion_depth: AtomicU32::new(0),
            memory_limit: config.max_memory,
            memory_used: AtomicUsize::new(0),
            timeout: config.timeout,
            start_time: Instant::now(),
        }
    }

    /// Check if instruction budget is exhausted.
    pub fn check_instructions(&self) -> Result<(), ResourceExhausted> {
        let count = self.instruction_count.load(Ordering::Relaxed);
        if count >= self.instruction_limit {
            Err(ResourceExhausted::Instructions {
                count,
                limit: self.instruction_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Increment instruction count and check limit.
    pub fn increment_instructions(&self, count: u64) -> Result<(), ResourceExhausted> {
        let new_count = self.instruction_count.fetch_add(count, Ordering::Relaxed) + count;
        if new_count >= self.instruction_limit {
            Err(ResourceExhausted::Instructions {
                count: new_count,
                limit: self.instruction_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Enter a recursion level, returning a guard that will decrement on drop.
    pub fn enter_recursion(&self) -> Result<RecursionGuard<'_>, ResourceExhausted> {
        let new_depth = self.recursion_depth.fetch_add(1, Ordering::Relaxed) + 1;
        if new_depth > self.recursion_limit {
            self.recursion_depth.fetch_sub(1, Ordering::Relaxed);
            Err(ResourceExhausted::Recursion {
                depth: new_depth,
                limit: self.recursion_limit,
            })
        } else {
            Ok(RecursionGuard {
                budget: self,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Allocate memory from the budget.
    pub fn allocate(&self, bytes: usize) -> Result<(), ResourceExhausted> {
        let new_used = self.memory_used.fetch_add(bytes, Ordering::Relaxed) + bytes;
        if new_used > self.memory_limit {
            self.memory_used.fetch_sub(bytes, Ordering::Relaxed);
            Err(ResourceExhausted::Memory {
                used: new_used,
                limit: self.memory_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Deallocate memory from the budget.
    pub fn deallocate(&self, bytes: usize) {
        self.memory_used.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Check if execution timeout has been exceeded.
    pub fn check_timeout(&self) -> Result<(), ResourceExhausted> {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.timeout {
            Err(ResourceExhausted::Timeout {
                elapsed,
                timeout: self.timeout,
            })
        } else {
            Ok(())
        }
    }

    /// Get current resource usage.
    pub fn usage(&self) -> ResourceUsage {
        ResourceUsage {
            instructions: self.instruction_count.load(Ordering::Relaxed),
            max_instructions: self.instruction_limit,
            recursion_depth: self.recursion_depth.load(Ordering::Relaxed),
            max_recursion: self.recursion_limit,
            memory_used: self.memory_used.load(Ordering::Relaxed),
            max_memory: self.memory_limit,
            elapsed: self.start_time.elapsed(),
            timeout: self.timeout,
        }
    }
}

/// RAII guard for recursion depth tracking.
///
/// Automatically decrements recursion depth when dropped.
pub struct RecursionGuard<'a> {
    budget: &'a ResourceBudget,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Drop for RecursionGuard<'_> {
    fn drop(&mut self) {
        self.budget.recursion_depth.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Current resource usage snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceUsage {
    pub instructions: u64,
    pub max_instructions: u64,
    pub recursion_depth: u32,
    pub max_recursion: u32,
    pub memory_used: usize,
    pub max_memory: usize,
    pub elapsed: Duration,
    pub timeout: Duration,
}

/// Resource exhaustion errors.
#[derive(Debug, Error)]
pub enum ResourceExhausted {
    #[error("Instruction limit exceeded ({count}/{limit})")]
    Instructions { count: u64, limit: u64 },
    #[error("Recursion limit exceeded ({depth}/{limit})")]
    Recursion { depth: u32, limit: u32 },
    #[error("Memory limit exceeded ({used}/{limit} bytes)")]
    Memory { used: usize, limit: usize },
    #[error("Script timeout ({elapsed:?}/{timeout:?})")]
    Timeout {
        elapsed: Duration,
        timeout: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ResourceConfig::default();
        assert_eq!(config.max_instructions, 100_000);
        assert_eq!(config.max_recursion, 64);
        assert_eq!(config.max_memory, 1024 * 1024);
    }

    #[test]
    fn test_instruction_tracking() {
        let budget = ResourceBudget::new(ResourceConfig {
            max_instructions: 10,
            ..Default::default()
        });

        assert!(budget.check_instructions().is_ok());
        assert!(budget.increment_instructions(5).is_ok());
        assert!(budget.increment_instructions(4).is_ok());
        assert!(budget.increment_instructions(1).is_err());

        let usage = budget.usage();
        assert_eq!(usage.instructions, 10);
    }

    #[test]
    fn test_recursion_tracking() {
        let budget = ResourceBudget::new(ResourceConfig {
            max_recursion: 3,
            ..Default::default()
        });

        let _guard1 = budget.enter_recursion().unwrap();
        let _guard2 = budget.enter_recursion().unwrap();
        let _guard3 = budget.enter_recursion().unwrap();
        assert!(budget.enter_recursion().is_err());

        drop(_guard3);
        assert!(budget.enter_recursion().is_ok());
    }

    #[test]
    fn test_memory_tracking() {
        let budget = ResourceBudget::new(ResourceConfig {
            max_memory: 100,
            ..Default::default()
        });

        assert!(budget.allocate(50).is_ok());
        assert!(budget.allocate(40).is_ok());
        assert!(budget.allocate(20).is_err());

        budget.deallocate(50);
        assert!(budget.allocate(20).is_ok());

        let usage = budget.usage();
        assert_eq!(usage.memory_used, 60);
    }

    #[test]
    fn test_timeout_not_exceeded() {
        let budget = ResourceBudget::new(ResourceConfig {
            timeout: Duration::from_secs(10),
            ..Default::default()
        });

        assert!(budget.check_timeout().is_ok());
    }

    #[test]
    fn test_timeout_exceeded() {
        let budget = ResourceBudget::new(ResourceConfig {
            timeout: Duration::from_nanos(1),
            ..Default::default()
        });

        std::thread::sleep(Duration::from_millis(1));
        assert!(budget.check_timeout().is_err());
    }

    #[test]
    fn test_resource_usage() {
        let budget = ResourceBudget::new(ResourceConfig::default());

        budget.increment_instructions(42).unwrap();
        budget.allocate(1024).unwrap();

        let usage = budget.usage();
        assert_eq!(usage.instructions, 42);
        assert_eq!(usage.memory_used, 1024);
        assert!(usage.elapsed < usage.timeout);
    }

    #[test]
    fn test_recursion_guard_drop() {
        let budget = ResourceBudget::new(ResourceConfig::default());

        {
            let _guard = budget.enter_recursion().unwrap();
            assert_eq!(budget.recursion_depth.load(Ordering::Relaxed), 1);
        }
        assert_eq!(budget.recursion_depth.load(Ordering::Relaxed), 0);
    }
}
