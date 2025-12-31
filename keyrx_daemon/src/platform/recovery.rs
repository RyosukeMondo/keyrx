//! Mutex poison recovery utilities.
//!
//! This module provides helper functions for safe mutex access with poison recovery,
//! enabling graceful degradation instead of panics when a mutex is poisoned.
//!
//! # Poison Recovery Strategy
//!
//! When a thread panics while holding a mutex lock, the mutex becomes "poisoned" to
//! indicate that the data it protects may be in an inconsistent state. The standard
//! library's default behavior is to return a `PoisonError` on subsequent lock attempts.
//!
//! This module takes the approach of recovering the inner guard from poisoned mutexes,
//! accepting that the data may be inconsistent but still accessible. This allows the
//! program to continue running in a degraded mode rather than cascading the panic.
//!
//! # Examples
//!
//! ```
//! use std::sync::Mutex;
//! use keyrx_daemon::platform::recovery::recover_lock;
//!
//! let mutex = Mutex::new(42);
//! let guard = recover_lock(&mutex).unwrap();
//! assert_eq!(*guard, 42);
//! ```

use crate::platform::common::PlatformError;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Attempts to acquire a mutex lock, recovering from poisoned state.
///
/// If the mutex is poisoned (another thread panicked while holding the lock),
/// this function will log a warning and recover the inner guard, allowing
/// access to potentially inconsistent data.
///
/// # Examples
///
/// ```
/// use std::sync::Mutex;
/// use keyrx_daemon::platform::recovery::recover_lock;
///
/// let mutex = Mutex::new(vec![1, 2, 3]);
/// let guard = recover_lock(&mutex).unwrap();
/// assert_eq!(*guard, vec![1, 2, 3]);
/// ```
///
/// # Errors
///
/// Returns `PlatformError::Poisoned` if the mutex cannot be accessed.
/// In practice, this function will recover from poison and return `Ok`,
/// but the error variant exists for consistency and future extensibility.
pub fn recover_lock<'a, T>(mutex: &'a Mutex<T>) -> Result<MutexGuard<'a, T>, PlatformError> {
    mutex
        .lock()
        .or_else(|poison_error: PoisonError<MutexGuard<T>>| {
            log::warn!("Mutex poisoned, attempting recovery");
            // Use poisoned guard (data may be inconsistent but accessible)
            Ok(poison_error.into_inner())
        })
}

/// Attempts to acquire a mutex lock with context for error messages.
///
/// This is similar to [`recover_lock`] but includes a context string in the
/// log message to help identify which mutex was poisoned.
///
/// # Examples
///
/// ```
/// use std::sync::Mutex;
/// use keyrx_daemon::platform::recovery::recover_lock_with_context;
///
/// let mutex = Mutex::new("important data");
/// let guard = recover_lock_with_context(&mutex, "config loader").unwrap();
/// assert_eq!(*guard, "important data");
/// ```
///
/// # Errors
///
/// Returns `PlatformError::Poisoned` if the mutex cannot be accessed.
pub fn recover_lock_with_context<'a, T>(
    mutex: &'a Mutex<T>,
    context: &str,
) -> Result<MutexGuard<'a, T>, PlatformError> {
    mutex
        .lock()
        .or_else(|poison_error: PoisonError<MutexGuard<T>>| {
            log::error!("Mutex poisoned in {}: recovering", context);
            Ok(poison_error.into_inner())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_normal_lock_acquisition() {
        let mutex = Mutex::new(42);
        let guard = recover_lock(&mutex).unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_normal_lock_with_context() {
        let mutex = Mutex::new("test data");
        let guard = recover_lock_with_context(&mutex, "test context").unwrap();
        assert_eq!(*guard, "test data");
    }

    #[test]
    fn test_poisoned_mutex_recovery() {
        let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex by panicking while holding the lock
        let result = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("intentional panic to poison mutex");
        })
        .join();

        assert!(result.is_err(), "Thread should have panicked");

        // Verify mutex is poisoned (normal lock would return Err)
        assert!(mutex.lock().is_err(), "Mutex should be poisoned");

        // Verify recovery succeeds
        let guard = recover_lock(&mutex).expect("Recovery should succeed");
        assert_eq!(*guard, vec![1, 2, 3]);
    }

    #[test]
    fn test_poisoned_mutex_recovery_with_context() {
        let mutex = Arc::new(Mutex::new(100));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex
        let result = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("poisoning mutex");
        })
        .join();

        assert!(result.is_err());
        assert!(mutex.lock().is_err(), "Mutex should be poisoned");

        // Verify recovery with context
        let guard =
            recover_lock_with_context(&mutex, "test mutex").expect("Recovery should succeed");
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_subsequent_operations_after_recovery() {
        let mutex = Arc::new(Mutex::new(0));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex
        let _ = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("poison");
        })
        .join();

        // First recovery
        {
            let mut guard = recover_lock(&mutex).unwrap();
            *guard = 42;
        }

        // Second access should work
        {
            let guard = recover_lock(&mutex).unwrap();
            assert_eq!(*guard, 42);
        }

        // Third access should also work
        {
            let mut guard = recover_lock(&mutex).unwrap();
            *guard = 100;
        }

        assert_eq!(*recover_lock(&mutex).unwrap(), 100);
    }

    #[test]
    fn test_concurrent_access_after_recovery() {
        let mutex = Arc::new(Mutex::new(vec![]));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex
        let _ = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("poison");
        })
        .join();

        // Recover and modify
        {
            let mut guard = recover_lock(&mutex).unwrap();
            guard.push(1);
        }

        // Spawn multiple threads to access the recovered mutex
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let mutex = Arc::clone(&mutex);
                thread::spawn(move || {
                    let mut guard = recover_lock(&mutex).unwrap();
                    guard.push(i);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let guard = recover_lock(&mutex).unwrap();
        assert_eq!(guard.len(), 6); // 1 initial + 5 from threads
    }

    #[test]
    fn test_context_string_variations() {
        let mutex = Mutex::new(String::from("data"));

        // Test various context strings
        let contexts = vec![
            "Windows message handler",
            "Linux event processor",
            "config loader",
            "device registry",
        ];

        for context in contexts {
            let guard = recover_lock_with_context(&mutex, context).unwrap();
            assert_eq!(*guard, "data");
        }
    }

    #[test]
    fn test_recover_lock_type_inference() {
        // Verify the function works with type inference
        let mutex = Mutex::new(42);
        let guard = recover_lock(&mutex).unwrap();
        let value: i32 = *guard;
        assert_eq!(value, 42);
    }

    #[test]
    fn test_recover_lock_with_complex_types() {
        #[derive(Debug, PartialEq)]
        struct ComplexType {
            id: usize,
            data: Vec<String>,
        }

        let complex = ComplexType {
            id: 1,
            data: vec!["a".to_string(), "b".to_string()],
        };

        let mutex = Mutex::new(complex);
        let guard = recover_lock(&mutex).unwrap();
        assert_eq!(guard.id, 1);
        assert_eq!(guard.data, vec!["a".to_string(), "b".to_string()]);
    }
}
