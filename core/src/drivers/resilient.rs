//! Resilient driver wrapper with circuit breaker and fallback support.
//!
//! This module provides a wrapper around platform-specific input drivers that
//! adds circuit breaker protection and automatic fallback to passthrough mode
//! when the circuit opens due to repeated failures.
//!
//! # Design
//!
//! - **Circuit Breaker**: Tracks consecutive failures and opens after threshold
//! - **Fallback Engine**: Activates when circuit opens, passing all input through
//! - **Automatic Recovery**: Circuit transitions through half-open state to test recovery
//! - **Thread-Safe**: Can be shared across async tasks
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::drivers::{ResilientDriver, PlatformInput};
//! use keyrx_core::safety::circuit_breaker::CircuitBreakerConfig;
//!
//! let driver = PlatformInput::new()?;
//! let resilient = ResilientDriver::new(driver, CircuitBreakerConfig::default());
//!
//! // Use normally - circuit breaker is transparent when closed
//! resilient.start().await?;
//! let events = resilient.poll_events().await?;
//!
//! // If repeated failures occur, circuit opens and fallback activates
//! // Input continues working in passthrough mode
//! ```

use crate::bail_keyrx;
use crate::engine::fallback::{FallbackEngine, FallbackReason};
use crate::engine::{InputEvent, OutputAction};
use crate::errors::{critical::CriticalError, driver::*, KeyrxError};
use crate::safety::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::traits::InputSource;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Resilient wrapper around an input driver with circuit breaker protection.
///
/// This struct wraps any `InputSource` implementation and adds:
/// - Circuit breaker that opens after repeated failures
/// - Fallback engine that activates when circuit opens
/// - Automatic recovery testing through half-open state
///
/// # Type Parameters
///
/// * `T` - The underlying input source type (e.g., `WindowsInput`, `LinuxInput`)
pub struct ResilientDriver<T: InputSource> {
    /// The underlying platform-specific driver.
    inner: T,
    /// Circuit breaker for tracking and preventing cascading failures.
    circuit_breaker: Arc<CircuitBreaker>,
    /// Fallback engine that activates when circuit opens.
    fallback: FallbackEngine,
}

impl<T: InputSource> ResilientDriver<T> {
    /// Create a new resilient driver wrapper.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying input driver
    /// * `config` - Circuit breaker configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// let driver = WindowsInput::new()?;
    /// let config = CircuitBreakerConfig {
    ///     failure_threshold: 5,
    ///     success_threshold: 2,
    ///     timeout: Duration::from_secs(30),
    /// };
    /// let resilient = ResilientDriver::new(driver, config);
    /// ```
    pub fn new(inner: T, config: CircuitBreakerConfig) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new("driver", config));
        let fallback = FallbackEngine::new();

        debug!(
            service = "keyrx",
            event = "resilient_driver_created",
            component = "resilient_driver",
            "ResilientDriver created with circuit breaker protection"
        );

        Self {
            inner,
            circuit_breaker,
            fallback,
        }
    }

    /// Check if fallback mode is currently active.
    pub fn is_fallback_active(&self) -> bool {
        self.fallback.is_active()
    }

    /// Check if the circuit breaker is open.
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_breaker.is_open()
    }

    /// Get the current failure count from the circuit breaker.
    pub fn failure_count(&self) -> usize {
        self.circuit_breaker.failure_count()
    }

    /// Manually reset the circuit breaker.
    ///
    /// This should only be used in exceptional circumstances or for testing.
    pub fn reset_circuit(&self) {
        self.circuit_breaker.reset();
        if self.fallback.is_active() {
            self.fallback.deactivate();
            info!(
                service = "keyrx",
                event = "resilient_driver_reset",
                component = "resilient_driver",
                "Circuit breaker manually reset, fallback deactivated"
            );
        }
    }

    /// Activate fallback mode due to circuit breaker opening.
    fn activate_fallback(&self) {
        if !self.fallback.is_active() {
            self.fallback.activate(FallbackReason::CircuitBreakerOpen);
            error!(
                service = "keyrx",
                event = "resilient_driver_fallback_activated",
                component = "resilient_driver",
                failure_count = self.failure_count(),
                "Circuit breaker opened - activating fallback mode"
            );
        }
    }

    /// Attempt to deactivate fallback mode if circuit has recovered.
    fn maybe_deactivate_fallback(&self) {
        if self.fallback.is_active() && self.circuit_breaker.is_closed() {
            self.fallback.deactivate();
            info!(
                service = "keyrx",
                event = "resilient_driver_fallback_deactivated",
                component = "resilient_driver",
                events_processed = self.fallback.event_count(),
                "Circuit breaker closed - deactivating fallback mode"
            );
        }
    }

    /// Convert KeyrxError to CriticalError for circuit breaker.
    fn to_critical_error(err: &KeyrxError) -> CriticalError {
        CriticalError::ProcessingFailed {
            reason: err.to_string(),
            cause: None,
        }
    }
}

#[async_trait]
impl<T: InputSource> InputSource for ResilientDriver<T> {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        // Check if circuit is open
        if self.circuit_breaker.is_open() {
            self.activate_fallback();
            // In fallback mode, we don't poll the underlying driver
            // Return empty vec as we can't generate synthetic events here
            return Ok(Vec::new());
        }

        // Call the underlying driver's poll_events
        let result = self.inner.poll_events().await;

        match result {
            Ok(events) => {
                // Success - record it with the circuit breaker
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Ok::<_, CriticalError>(()));

                // Check if we should deactivate fallback
                self.maybe_deactivate_fallback();
                Ok(events)
            }
            Err(err) => {
                // Record the failure with the circuit breaker
                let critical_err = Self::to_critical_error(&err);
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Err::<(), _>(critical_err.clone()));

                // Check if circuit just opened
                if self.circuit_breaker.is_open() {
                    self.activate_fallback();
                    warn!(
                        service = "keyrx",
                        event = "resilient_driver_circuit_open",
                        component = "resilient_driver",
                        failure_count = self.failure_count(),
                        error = %err,
                        "Circuit breaker opened during poll_events"
                    );
                    // Return empty vec instead of error to keep system running
                    return Ok(Vec::new());
                }

                // Circuit still closed - propagate the error
                error!(
                    service = "keyrx",
                    event = "resilient_driver_poll_error",
                    component = "resilient_driver",
                    error = %err,
                    "Error during poll_events"
                );
                Err(err)
            }
        }
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError> {
        // If in fallback mode, process through fallback engine
        if self.fallback.is_active() {
            // In fallback mode, we convert the action to an event for logging
            // but we don't actually send anything - fallback is passthrough
            debug!(
                service = "keyrx",
                event = "resilient_driver_fallback_output",
                component = "resilient_driver",
                action = ?action,
                "send_output called in fallback mode (no-op)"
            );
            return Ok(());
        }

        // Check if circuit is open
        if self.circuit_breaker.is_open() {
            self.activate_fallback();
            debug!(
                service = "keyrx",
                event = "resilient_driver_output_blocked",
                component = "resilient_driver",
                action = ?action,
                "send_output blocked - circuit open"
            );
            return Ok(());
        }

        // Store action string for error logging
        let action_str = format!("{:?}", action);

        // Call the underlying driver's send_output
        let result = self.inner.send_output(action).await;

        match result {
            Ok(()) => {
                // Success - record it with the circuit breaker
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Ok::<_, CriticalError>(()));

                // Check if we should deactivate fallback
                self.maybe_deactivate_fallback();
                Ok(())
            }
            Err(err) => {
                // Record the failure with the circuit breaker
                let critical_err = Self::to_critical_error(&err);
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Err::<(), _>(critical_err.clone()));

                // Check if circuit just opened
                if self.circuit_breaker.is_open() {
                    self.activate_fallback();
                    warn!(
                        service = "keyrx",
                        event = "resilient_driver_circuit_open",
                        component = "resilient_driver",
                        failure_count = self.failure_count(),
                        action = %action_str,
                        "Circuit breaker opened during send_output"
                    );
                    // Return Ok to keep system running (action is dropped)
                    return Ok(());
                }

                // Circuit still closed - propagate the error
                error!(
                    service = "keyrx",
                    event = "resilient_driver_output_error",
                    component = "resilient_driver",
                    error = %err,
                    action = %action_str,
                    "Error during send_output"
                );
                Err(err)
            }
        }
    }

    async fn start(&mut self) -> Result<(), KeyrxError> {
        // Check if circuit is open before starting
        if self.circuit_breaker.is_open() {
            self.activate_fallback();
            error!(
                service = "keyrx",
                event = "resilient_driver_start_blocked",
                component = "resilient_driver",
                "Cannot start driver - circuit breaker is open"
            );
            bail_keyrx!(
                DRIVER_INIT_FAILED,
                reason = "Circuit breaker is open - driver unavailable".to_string()
            );
        }

        // Call the underlying driver's start
        let result = self.inner.start().await;

        match result {
            Ok(()) => {
                // Success - record it with the circuit breaker
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Ok::<_, CriticalError>(()));

                debug!(
                    service = "keyrx",
                    event = "resilient_driver_started",
                    component = "resilient_driver",
                    "Driver started successfully"
                );
                Ok(())
            }
            Err(err) => {
                // Record the failure with the circuit breaker
                let critical_err = Self::to_critical_error(&err);
                let _ = self
                    .circuit_breaker
                    .call_critical(|| Err::<(), _>(critical_err.clone()));

                // Check if circuit just opened
                if self.circuit_breaker.is_open() {
                    self.activate_fallback();
                }

                error!(
                    service = "keyrx",
                    event = "resilient_driver_start_error",
                    component = "resilient_driver",
                    error = %err,
                    circuit_open = self.circuit_breaker.is_open(),
                    "Error during start"
                );
                Err(err)
            }
        }
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        // Stop doesn't go through circuit breaker - we always want to be able to stop
        let result = self.inner.stop().await;

        // Deactivate fallback if active
        if self.fallback.is_active() {
            self.fallback.deactivate();
        }

        debug!(
            service = "keyrx",
            event = "resilient_driver_stopped",
            component = "resilient_driver",
            success = result.is_ok(),
            "Driver stopped"
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockInput;
    use crate::KeyCode;
    use std::time::Duration;

    #[tokio::test]
    async fn resilient_driver_starts_with_closed_circuit() {
        let mock = MockInput::new();
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let driver = ResilientDriver::new(mock, config);

        assert!(!driver.is_circuit_open());
        assert!(!driver.is_fallback_active());
        assert_eq!(driver.failure_count(), 0);
    }

    #[tokio::test]
    async fn successful_operations_keep_circuit_closed() {
        let mut mock = MockInput::new();
        mock.queue_event(InputEvent::key_down(KeyCode::A, 1000));

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let mut driver = ResilientDriver::new(mock, config);

        let result = driver.start().await;
        assert!(result.is_ok());

        let events = driver.poll_events().await.unwrap();
        assert_eq!(events.len(), 1);

        assert!(!driver.is_circuit_open());
        assert!(!driver.is_fallback_active());
    }

    #[tokio::test]
    async fn manual_reset_works() {
        let mock = MockInput::new();
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };
        let mut driver = ResilientDriver::new(mock, config);

        // Cause failure by starting a mock that fails
        let _ = driver.start().await;

        // Reset
        driver.reset_circuit();

        assert!(!driver.is_circuit_open());
        assert!(!driver.is_fallback_active());
    }
}
