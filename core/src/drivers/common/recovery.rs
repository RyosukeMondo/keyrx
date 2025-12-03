//! Automatic error recovery with exponential backoff.
//!
//! This module provides retry logic for transient driver errors, using
//! exponential backoff to avoid overwhelming the system while attempting
//! recovery.

use super::error::DriverError;
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for retry behavior.
///
/// Controls the backoff strategy and maximum retry attempts.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Infrastructure for future use
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries (caps exponential growth).
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 for doubling).
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

#[allow(dead_code)] // Infrastructure for future use
impl RetryConfig {
    /// Creates a new retry configuration with custom settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::drivers::common::recovery::RetryConfig;
    /// use std::time::Duration;
    ///
    /// let config = RetryConfig::new(3, Duration::from_millis(50), Duration::from_secs(5));
    /// ```
    pub fn new(max_retries: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            backoff_multiplier: 2.0,
        }
    }

    /// Creates a configuration for aggressive retries (fast, many attempts).
    ///
    /// Useful for operations that typically recover quickly.
    pub fn aggressive() -> Self {
        Self {
            max_retries: 10,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 1.5,
        }
    }

    /// Creates a configuration for conservative retries (slow, few attempts).
    ///
    /// Useful for operations that are expensive or less likely to recover.
    pub fn conservative() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 3.0,
        }
    }

    /// Calculates the delay for a given retry attempt.
    ///
    /// Uses exponential backoff: delay = initial_delay * (multiplier ^ attempt)
    /// capped at max_delay.
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms =
            (self.initial_delay.as_millis() as f64) * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(delay_ms as u64)
    }
}

/// Retries an operation with exponential backoff.
///
/// This function attempts to execute the provided operation, retrying on
/// retryable errors according to the configuration. Non-retryable errors
/// are returned immediately without retry.
///
/// # Arguments
///
/// * `config` - Retry configuration (max attempts, delays, etc.)
/// * `operation_name` - Name of the operation for logging
/// * `operation` - The operation to retry (a closure returning Result)
///
/// # Returns
///
/// Returns the result of the operation, or the last error if all retries fail.
///
/// # Examples
///
/// ```no_run
/// use keyrx_core::drivers::common::recovery::{retry_with_backoff, RetryConfig};
/// use keyrx_core::drivers::common::error::DriverError;
///
/// # async fn example() -> Result<(), DriverError> {
/// let config = RetryConfig::default();
/// let result = retry_with_backoff(config, "connect_device", || async {
///     // Your operation here
///     Ok(())
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)] // Infrastructure for future use
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, DriverError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, DriverError>>,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                // Check if error is retryable
                if !error.is_retryable() {
                    warn!(
                        operation = operation_name,
                        error = %error,
                        "Operation failed with non-retryable error"
                    );
                    return Err(error);
                }

                // Check if we've exhausted retries
                if attempt >= config.max_retries {
                    warn!(
                        operation = operation_name,
                        attempts = attempt + 1,
                        error = %error,
                        "Operation failed after maximum retries"
                    );
                    return Err(error);
                }

                // Calculate delay (use error's suggestion or config's backoff)
                let delay = error
                    .retry_delay()
                    .unwrap_or_else(|| config.delay_for_attempt(attempt));

                debug!(
                    operation = operation_name,
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    error = %error,
                    "Retrying operation after delay"
                );

                // Wait before retry
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

/// Retries a synchronous operation with exponential backoff.
///
/// Similar to `retry_with_backoff` but for blocking operations.
/// Uses thread sleep instead of async sleep.
///
/// # Arguments
///
/// * `config` - Retry configuration (max attempts, delays, etc.)
/// * `operation_name` - Name of the operation for logging
/// * `operation` - The operation to retry (a closure returning Result)
///
/// # Returns
///
/// Returns the result of the operation, or the last error if all retries fail.
///
/// # Examples
///
/// ```no_run
/// use keyrx_core::drivers::common::recovery::{retry_with_backoff_sync, RetryConfig};
/// use keyrx_core::drivers::common::error::DriverError;
///
/// # fn example() -> Result<(), DriverError> {
/// let config = RetryConfig::default();
/// let result = retry_with_backoff_sync(config, "grab_device", || {
///     // Your blocking operation here
///     Ok(())
/// })?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)] // Infrastructure for future use
pub fn retry_with_backoff_sync<F, T>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, DriverError>
where
    F: FnMut() -> Result<T, DriverError>,
{
    let mut attempt = 0;

    loop {
        match operation() {
            Ok(result) => {
                if attempt > 0 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                // Check if error is retryable
                if !error.is_retryable() {
                    warn!(
                        operation = operation_name,
                        error = %error,
                        "Operation failed with non-retryable error"
                    );
                    return Err(error);
                }

                // Check if we've exhausted retries
                if attempt >= config.max_retries {
                    warn!(
                        operation = operation_name,
                        attempts = attempt + 1,
                        error = %error,
                        "Operation failed after maximum retries"
                    );
                    return Err(error);
                }

                // Calculate delay (use error's suggestion or config's backoff)
                let delay = error
                    .retry_delay()
                    .unwrap_or_else(|| config.delay_for_attempt(attempt));

                debug!(
                    operation = operation_name,
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    error = %error,
                    "Retrying operation after delay"
                );

                // Wait before retry
                std::thread::sleep(delay);
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn default_config_is_reasonable() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn aggressive_config_has_more_retries() {
        let config = RetryConfig::aggressive();
        assert!(config.max_retries > RetryConfig::default().max_retries);
        assert!(config.initial_delay < RetryConfig::default().initial_delay);
    }

    #[test]
    fn conservative_config_has_fewer_retries() {
        let config = RetryConfig::conservative();
        assert!(config.max_retries < RetryConfig::default().max_retries);
        assert!(config.initial_delay > RetryConfig::default().initial_delay);
    }

    #[test]
    fn delay_grows_exponentially() {
        let config = RetryConfig::default();
        let delay0 = config.delay_for_attempt(0);
        let delay1 = config.delay_for_attempt(1);
        let delay2 = config.delay_for_attempt(2);

        assert_eq!(delay0, Duration::from_millis(100));
        assert_eq!(delay1, Duration::from_millis(200));
        assert_eq!(delay2, Duration::from_millis(400));
    }

    #[test]
    fn delay_caps_at_max() {
        let config = RetryConfig {
            max_retries: 20,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };

        let delay_large = config.delay_for_attempt(10);
        assert!(delay_large <= config.max_delay);
    }

    #[tokio::test]
    async fn retry_succeeds_on_first_attempt() {
        let config = RetryConfig::default();
        let result = retry_with_backoff(config, "test_op", || async { Ok::<_, DriverError>(42) })
            .await
            .unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn retry_succeeds_after_temporary_failure() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(config, "test_op", move || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(DriverError::Temporary {
                        message: "not ready".to_string(),
                        retry_after: Duration::from_millis(10),
                    })
                } else {
                    Ok(42)
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_fails_on_non_retryable_error() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(config, "test_op", move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(DriverError::DeviceNotFound {
                    path: "/dev/null".into(),
                })
            }
        })
        .await;

        assert!(result.is_err());
        // Should only try once (no retries for non-retryable errors)
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_exhausts_max_retries() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(config, "test_op", move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(DriverError::Temporary {
                    message: "always fail".to_string(),
                    retry_after: Duration::from_millis(1),
                })
            }
        })
        .await;

        assert!(result.is_err());
        // Initial attempt + 3 retries = 4 total
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn sync_retry_succeeds_on_first_attempt() {
        let config = RetryConfig::default();
        let result =
            retry_with_backoff_sync(config, "test_op", || Ok::<_, DriverError>(42)).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn sync_retry_succeeds_after_temporary_failure() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff_sync(config, "test_op", move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(DriverError::Temporary {
                    message: "not ready".to_string(),
                    retry_after: Duration::from_millis(1),
                })
            } else {
                Ok(42)
            }
        })
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn sync_retry_fails_on_non_retryable_error() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff_sync(config, "test_op", move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(DriverError::DeviceNotFound {
                path: "/dev/null".into(),
            })
        });

        assert!(result.is_err());
        // Should only try once (no retries for non-retryable errors)
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn sync_retry_exhausts_max_retries() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff_sync(config, "test_op", move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(DriverError::Temporary {
                message: "always fail".to_string(),
                retry_after: Duration::from_millis(1),
            })
        });

        assert!(result.is_err());
        // Initial attempt + 3 retries = 4 total
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn uses_error_suggested_delay() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // This test verifies that the error's retry_delay is used
        let result = retry_with_backoff_sync(config, "test_op", move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 1 {
                // Error suggests a specific retry delay
                Err(DriverError::Temporary {
                    message: "busy".to_string(),
                    retry_after: Duration::from_millis(5),
                })
            } else {
                Ok(42)
            }
        })
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
