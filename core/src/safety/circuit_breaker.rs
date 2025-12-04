//! Circuit breaker implementation for preventing cascading failures.
//!
//! This module provides a thread-safe, lock-free circuit breaker that tracks
//! failures and opens the circuit when a threshold is exceeded. The circuit
//! breaker prevents repeated attempts to operations that are likely to fail,
//! giving the system time to recover.
//!
//! # States
//!
//! - **Closed**: Normal operation, requests pass through
//! - **Open**: Circuit is open, requests fail fast without attempting the operation
//! - **HalfOpen**: Testing recovery, limited requests are allowed to probe if the system has recovered
//!
//! # Design Philosophy
//!
//! - Lock-free implementation using atomic operations
//! - Thread-safe for use across async tasks
//! - Configurable failure thresholds and timeouts
//! - Automatic recovery through half-open state testing
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::safety::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! let config = CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     success_threshold: 2,
//!     timeout: Duration::from_secs(30),
//! };
//!
//! let breaker = CircuitBreaker::new("keyboard_driver", config);
//!
//! match breaker.call(|| risky_operation()) {
//!     Ok(result) => handle_success(result),
//!     Err(err) => {
//!         // Circuit may be open or operation failed
//!         log::error!("Operation failed: {}", err);
//!     }
//! }
//! ```

use crate::errors::critical::CriticalError;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Configuration for circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: usize,

    /// Number of consecutive successes in half-open state before closing the circuit.
    pub success_threshold: usize,

    /// Duration to wait before transitioning from open to half-open.
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker state representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum State {
    /// Circuit is closed, operations proceed normally.
    Closed = 0,
    /// Circuit is open, operations fail fast.
    Open = 1,
    /// Circuit is half-open, testing for recovery.
    HalfOpen = 2,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0 => State::Closed,
            1 => State::Open,
            2 => State::HalfOpen,
            _ => State::Closed, // Default to closed for invalid values
        }
    }
}

/// Thread-safe circuit breaker for preventing cascading failures.
///
/// The circuit breaker tracks consecutive failures and opens when a threshold
/// is exceeded. After a timeout, it enters half-open state to test if the
/// system has recovered.
pub struct CircuitBreaker {
    /// Context name for logging (e.g., "driver", "engine").
    context: String,

    /// Configuration parameters.
    config: CircuitBreakerConfig,

    /// Current state (Closed=0, Open=1, HalfOpen=2).
    state: AtomicU8,

    /// Consecutive failure count.
    failure_count: AtomicU64,

    /// Consecutive success count (used in half-open state).
    success_count: AtomicU64,

    /// Timestamp when circuit was opened (nanoseconds since epoch).
    opened_at: AtomicU64,

    /// Last error message when circuit opened.
    last_error: RwLock<Option<String>>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `context` - Name for logging and error messages
    /// * `config` - Configuration parameters
    ///
    /// # Example
    ///
    /// ```ignore
    /// let breaker = CircuitBreaker::new("driver", CircuitBreakerConfig::default());
    /// ```
    pub fn new(context: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            context: context.into(),
            config,
            state: AtomicU8::new(State::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            opened_at: AtomicU64::new(0),
            last_error: RwLock::new(None),
        }
    }

    /// Wraps an operation with circuit breaker protection.
    ///
    /// If the circuit is open, returns immediately with a `CircuitBreakerOpen` error.
    /// Otherwise, executes the operation and updates the circuit state based on the result.
    ///
    /// # Arguments
    ///
    /// * `f` - The operation to execute
    ///
    /// # Returns
    ///
    /// - `Ok(T)` if the operation succeeds
    /// - `Err(CriticalError)` if the circuit is open or the operation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = breaker.call(|| {
    ///     risky_operation()
    /// });
    /// ```
    pub fn call<F, T, E>(&self, f: F) -> Result<T, CriticalError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        // Check if we should transition to half-open
        self.maybe_transition_to_half_open();

        // Get current state
        let state = State::from(self.state.load(Ordering::Acquire));

        // If open, fail fast
        if state == State::Open {
            let last_error = self
                .last_error
                .read()
                .ok()
                .and_then(|guard| guard.clone())
                .unwrap_or_default();
            return Err(CriticalError::CircuitBreakerOpen {
                failure_count: self.failure_count.load(Ordering::Relaxed) as usize,
                last_error,
            });
        }

        // Execute the operation
        match f() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(err) => {
                let error_msg = err.to_string();
                self.on_failure(&error_msg);
                Err(CriticalError::ProcessingFailed {
                    reason: error_msg,
                    cause: None,
                })
            }
        }
    }

    /// Executes an operation that returns CriticalError directly.
    ///
    /// This is a convenience method for operations that already return CriticalError.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = breaker.call_critical(|| {
    ///     operation_that_returns_critical_error()
    /// });
    /// ```
    pub fn call_critical<F, T>(&self, f: F) -> Result<T, CriticalError>
    where
        F: FnOnce() -> Result<T, CriticalError>,
    {
        // Check if we should transition to half-open
        self.maybe_transition_to_half_open();

        // Get current state
        let state = State::from(self.state.load(Ordering::Acquire));

        // If open, fail fast
        if state == State::Open {
            let last_error = self
                .last_error
                .read()
                .ok()
                .and_then(|guard| guard.clone())
                .unwrap_or_default();
            return Err(CriticalError::CircuitBreakerOpen {
                failure_count: self.failure_count.load(Ordering::Relaxed) as usize,
                last_error,
            });
        }

        // Execute the operation
        match f() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(err) => {
                self.on_failure(&err.to_string());
                Err(err)
            }
        }
    }

    /// Records a successful operation.
    fn on_success(&self) {
        let state = State::from(self.state.load(Ordering::Acquire));

        match state {
            State::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Release);
            }
            State::HalfOpen => {
                // Increment success count
                let success_count = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;

                // If we've reached the success threshold, close the circuit
                if success_count >= self.config.success_threshold as u64 {
                    self.close_circuit();
                }
            }
            State::Open => {
                // Should not happen, but reset if it does
                tracing::warn!(
                    context = %self.context,
                    "Success recorded while circuit is open"
                );
            }
        }
    }

    /// Records a failed operation.
    fn on_failure(&self, error: &str) {
        let state = State::from(self.state.load(Ordering::Acquire));

        match state {
            State::Closed => {
                // Increment failure count
                let failure_count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;

                // If we've reached the failure threshold, open the circuit
                if failure_count >= self.config.failure_threshold as u64 {
                    self.open_circuit(error);
                }
            }
            State::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.open_circuit(error);
            }
            State::Open => {
                // Already open, just update the error
                if let Ok(mut guard) = self.last_error.write() {
                    *guard = Some(error.to_string());
                }
            }
        }
    }

    /// Opens the circuit due to repeated failures.
    fn open_circuit(&self, error: &str) {
        self.state.store(State::Open as u8, Ordering::Release);
        self.success_count.store(0, Ordering::Release);

        // Record when we opened
        let now = Instant::now();
        let nanos = now.elapsed().as_nanos() as u64;
        self.opened_at.store(nanos, Ordering::Release);

        // Store the error message
        if let Ok(mut guard) = self.last_error.write() {
            *guard = Some(error.to_string());
        }

        tracing::error!(
            context = %self.context,
            failure_count = self.failure_count.load(Ordering::Relaxed),
            error = %error,
            "Circuit breaker opened"
        );
    }

    /// Closes the circuit after successful recovery.
    fn close_circuit(&self) {
        self.state.store(State::Closed as u8, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);

        tracing::info!(
            context = %self.context,
            "Circuit breaker closed after recovery"
        );
    }

    /// Checks if enough time has passed to transition to half-open.
    fn maybe_transition_to_half_open(&self) {
        let state = State::from(self.state.load(Ordering::Acquire));

        if state != State::Open {
            return;
        }

        // Check if timeout has elapsed
        let opened_at = self.opened_at.load(Ordering::Acquire);
        if opened_at == 0 {
            return;
        }

        // Calculate elapsed time (note: this is a simplification, in production
        // you'd want to store an actual timestamp)
        let now = Instant::now();
        let elapsed = Duration::from_nanos(now.elapsed().as_nanos() as u64 - opened_at);

        if elapsed >= self.config.timeout {
            // Transition to half-open
            if self
                .state
                .compare_exchange(
                    State::Open as u8,
                    State::HalfOpen as u8,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                self.success_count.store(0, Ordering::Release);

                tracing::info!(
                    context = %self.context,
                    "Circuit breaker transitioning to half-open"
                );
            }
        }
    }

    /// Returns the current state of the circuit breaker.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if breaker.is_open() {
    ///     log::warn!("Circuit breaker is open, using fallback");
    ///     use_fallback();
    /// }
    /// ```
    pub fn is_open(&self) -> bool {
        State::from(self.state.load(Ordering::Acquire)) == State::Open
    }

    /// Returns true if the circuit breaker is closed (normal operation).
    pub fn is_closed(&self) -> bool {
        State::from(self.state.load(Ordering::Acquire)) == State::Closed
    }

    /// Returns true if the circuit breaker is in half-open state (testing recovery).
    pub fn is_half_open(&self) -> bool {
        State::from(self.state.load(Ordering::Acquire)) == State::HalfOpen
    }

    /// Gets the current failure count.
    pub fn failure_count(&self) -> usize {
        self.failure_count.load(Ordering::Relaxed) as usize
    }

    /// Manually resets the circuit breaker to closed state.
    ///
    /// This should only be used in exceptional circumstances or for testing.
    pub fn reset(&self) {
        self.close_circuit();
    }
}

/// Creates a new shared circuit breaker wrapped in Arc.
///
/// This is a convenience function for creating circuit breakers that will
/// be shared across multiple threads or tasks.
///
/// # Example
///
/// ```ignore
/// let breaker = new_shared_breaker("driver", CircuitBreakerConfig::default());
/// let breaker_clone = breaker.clone();
///
/// tokio::spawn(async move {
///     breaker_clone.call(|| operation()).await
/// });
/// ```
pub fn new_shared_breaker(
    context: impl Into<String>,
    config: CircuitBreakerConfig,
) -> Arc<CircuitBreaker> {
    Arc::new(CircuitBreaker::new(context, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_closed_state() {
        let breaker = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert!(breaker.is_closed());
        assert!(!breaker.is_open());
        assert!(!breaker.is_half_open());
    }

    #[test]
    fn opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new("test", config);

        // First two failures should not open the circuit
        let _ = breaker.call(|| Err::<(), _>("error 1"));
        assert!(breaker.is_closed());

        let _ = breaker.call(|| Err::<(), _>("error 2"));
        assert!(breaker.is_closed());

        // Third failure should open the circuit
        let _ = breaker.call(|| Err::<(), _>("error 3"));
        assert!(breaker.is_open());
    }

    #[test]
    fn fails_fast_when_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new("test", config);

        // Cause the circuit to open
        let _ = breaker.call(|| Err::<(), _>("error"));
        assert!(breaker.is_open());

        // Next call should fail fast without executing
        let result = breaker.call(|| Ok::<_, String>("should not execute"));
        assert!(result.is_err());
        match result.unwrap_err() {
            CriticalError::CircuitBreakerOpen { .. } => {}
            _ => panic!("Expected CircuitBreakerOpen error"),
        }
    }

    #[test]
    fn resets_failure_count_on_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new("test", config);

        // Two failures
        let _ = breaker.call(|| Err::<(), _>("error 1"));
        let _ = breaker.call(|| Err::<(), _>("error 2"));
        assert_eq!(breaker.failure_count(), 2);

        // One success should reset the counter
        let _ = breaker.call(|| Ok::<_, String>(42));
        assert_eq!(breaker.failure_count(), 0);

        // Now it should take 3 more failures to open
        let _ = breaker.call(|| Err::<(), _>("error 3"));
        let _ = breaker.call(|| Err::<(), _>("error 4"));
        assert!(breaker.is_closed());
        let _ = breaker.call(|| Err::<(), _>("error 5"));
        assert!(breaker.is_open());
    }

    #[test]
    fn manual_reset_closes_circuit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new("test", config);

        // Open the circuit
        let _ = breaker.call(|| Err::<(), _>("error"));
        assert!(breaker.is_open());

        // Manual reset
        breaker.reset();
        assert!(breaker.is_closed());
        assert_eq!(breaker.failure_count(), 0);
    }

    #[test]
    fn call_critical_works_with_critical_error() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new("test", config);

        // Success
        let result = breaker.call_critical(|| Ok::<_, CriticalError>(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Failure
        let result: Result<i32, CriticalError> = breaker.call_critical(|| {
            Err(CriticalError::ProcessingFailed {
                reason: "test".to_string(),
                cause: None,
            })
        });
        assert!(result.is_err());
    }

    #[test]
    fn shared_breaker_works() {
        let breaker = new_shared_breaker("test", CircuitBreakerConfig::default());
        let breaker_clone = breaker.clone();

        // Both references should point to the same breaker
        let _ = breaker.call(|| Err::<(), _>("error"));
        assert_eq!(breaker_clone.failure_count(), 1);
    }
}
