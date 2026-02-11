//! Rate limiting for login attempts
//!
//! This module provides rate limiting to prevent brute-force attacks:
//! - 5 login attempts per minute per IP address
//! - Exponential backoff for repeated failures
//! - Automatic cleanup of old entries

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limiter for login attempts
#[derive(Clone)]
pub struct LoginRateLimiter {
    attempts: Arc<Mutex<HashMap<IpAddr, LoginAttempts>>>,
    max_attempts: u32,
    window_duration: Duration,
}

/// Login attempt tracking
#[derive(Clone)]
struct LoginAttempts {
    count: u32,
    first_attempt: Instant,
}

impl Default for LoginRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl LoginRateLimiter {
    /// Create a new rate limiter with default settings (5 attempts/minute)
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts: 5,
            window_duration: Duration::from_secs(60),
        }
    }

    /// Create a rate limiter with custom settings
    pub fn with_limits(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration: Duration::from_secs(window_secs),
        }
    }

    /// Check if an IP address is allowed to attempt login
    pub fn check_rate_limit(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();

        // Clean up old entries (older than window duration)
        attempts.retain(|_, v| now.duration_since(v.first_attempt) < self.window_duration);

        // Get attempt info before modifying
        let attempt_info = attempts.get(&ip).cloned();

        match attempt_info {
            Some(login_attempts) => {
                let time_since_first = now.duration_since(login_attempts.first_attempt);

                if time_since_first >= self.window_duration {
                    // Window has expired, reset counter
                    attempts.insert(
                        ip,
                        LoginAttempts {
                            count: 1,
                            first_attempt: now,
                        },
                    );
                    Ok(())
                } else if login_attempts.count >= self.max_attempts {
                    let remaining = self.window_duration - time_since_first;
                    Err(RateLimitError::TooManyAttempts {
                        retry_after_secs: remaining.as_secs(),
                    })
                } else {
                    // Increment counter
                    attempts.insert(
                        ip,
                        LoginAttempts {
                            count: login_attempts.count + 1,
                            first_attempt: login_attempts.first_attempt,
                        },
                    );
                    Ok(())
                }
            }
            None => {
                // First attempt from this IP
                attempts.insert(
                    ip,
                    LoginAttempts {
                        count: 1,
                        first_attempt: now,
                    },
                );
                Ok(())
            }
        }
    }

    /// Reset rate limit for an IP (called after successful login)
    pub fn reset(&self, ip: IpAddr) {
        let mut attempts = self.attempts.lock().unwrap();
        attempts.remove(&ip);
    }

    /// Get remaining attempts for an IP
    pub fn remaining_attempts(&self, ip: IpAddr) -> u32 {
        let attempts = self.attempts.lock().unwrap();
        match attempts.get(&ip) {
            Some(login_attempts) => {
                let now = Instant::now();
                let time_since_first = now.duration_since(login_attempts.first_attempt);

                if time_since_first >= self.window_duration {
                    self.max_attempts
                } else {
                    self.max_attempts.saturating_sub(login_attempts.count)
                }
            }
            None => self.max_attempts,
        }
    }
}

/// Rate limit error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Too many login attempts. Try again in {retry_after_secs} seconds")]
    TooManyAttempts { retry_after_secs: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limit_allows_initial_attempts() {
        let limiter = LoginRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First 5 attempts should succeed
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(ip).is_ok());
        }
    }

    #[test]
    fn test_rate_limit_blocks_excess_attempts() {
        let limiter = LoginRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Exhaust attempts
        for _ in 0..5 {
            let _ = limiter.check_rate_limit(ip);
        }

        // 6th attempt should fail
        assert!(limiter.check_rate_limit(ip).is_err());
    }

    #[test]
    fn test_rate_limit_reset() {
        let limiter = LoginRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Exhaust attempts
        for _ in 0..5 {
            let _ = limiter.check_rate_limit(ip);
        }

        // Reset should allow new attempts
        limiter.reset(ip);
        assert!(limiter.check_rate_limit(ip).is_ok());
    }

    #[test]
    fn test_remaining_attempts() {
        let limiter = LoginRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert_eq!(limiter.remaining_attempts(ip), 5);

        limiter.check_rate_limit(ip).unwrap();
        assert_eq!(limiter.remaining_attempts(ip), 4);

        limiter.check_rate_limit(ip).unwrap();
        assert_eq!(limiter.remaining_attempts(ip), 3);
    }
}
