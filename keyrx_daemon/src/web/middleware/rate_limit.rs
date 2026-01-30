//! Rate limiting middleware to prevent DoS attacks
//!
//! Implements per-IP rate limiting with configurable limits

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Rate limit configuration per IP
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window: Duration::from_secs(1),
        }
    }
}

impl RateLimitConfig {
    /// Create a test-friendly rate limit config with much higher limits
    ///
    /// This should only be used in test environments where we want to
    /// stress test the system without hitting rate limits.
    pub fn test_mode() -> Self {
        Self {
            max_requests: 1000,
            window: Duration::from_secs(1),
        }
    }
}

/// Rate limiter state
struct RateLimiterState {
    /// Request counts per IP
    counters: HashMap<SocketAddr, (Instant, usize)>,
    /// Configuration
    config: RateLimitConfig,
}

/// Rate limiting middleware layer
#[derive(Clone)]
pub struct RateLimitLayer {
    state: Arc<Mutex<RateLimiterState>>,
}

impl RateLimitLayer {
    /// Create new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create rate limiter with custom config
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                counters: HashMap::new(),
                config,
            })),
        }
    }

    /// Check if request is allowed
    pub fn check_rate_limit(&self, addr: SocketAddr) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let window = state.config.window;
        let max_requests = state.config.max_requests;

        // Clean up old entries
        state
            .counters
            .retain(|_, (timestamp, _)| now.duration_since(*timestamp) < window);

        // Check current IP
        let entry = state.counters.entry(addr).or_insert((now, 0));

        // Reset counter if window expired
        if now.duration_since(entry.0) >= window {
            *entry = (now, 0);
        }

        // Increment counter
        entry.1 += 1;

        // Check limit
        entry.1 <= max_requests
    }
}

impl Default for RateLimitLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting middleware handler
pub async fn rate_limit_middleware(
    connect_info: Option<ConnectInfo<SocketAddr>>,
    limiter: axum::extract::State<RateLimitLayer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    // Extract IP address from ConnectInfo if available, otherwise use localhost
    // This prevents 500 errors when ConnectInfo extension is not available
    let addr = connect_info
        .map(|ConnectInfo(addr)| addr)
        .unwrap_or_else(|| "127.0.0.1:0".parse().unwrap());

    if limiter.check_rate_limit(addr) {
        Ok(next.run(request).await)
    } else {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please slow down.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(1),
        };
        let limiter = RateLimitLayer::with_config(config);
        let addr = "127.0.0.1:8080".parse().unwrap();

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit(addr));
        assert!(limiter.check_rate_limit(addr));
        assert!(limiter.check_rate_limit(addr));

        // 4th request should fail
        assert!(!limiter.check_rate_limit(addr));
    }

    #[test]
    fn test_rate_limit_different_ips() {
        let limiter = RateLimitLayer::new();
        let addr1 = "127.0.0.1:8080".parse().unwrap();
        let addr2 = "127.0.0.2:8080".parse().unwrap();

        // Different IPs have independent limits
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(addr1));
        }
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(addr2));
        }
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
        };
        let limiter = RateLimitLayer::with_config(config);
        let addr = "127.0.0.1:8080".parse().unwrap();

        // Use up limit
        assert!(limiter.check_rate_limit(addr));
        assert!(limiter.check_rate_limit(addr));
        assert!(!limiter.check_rate_limit(addr));

        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be allowed again
        assert!(limiter.check_rate_limit(addr));
        assert!(limiter.check_rate_limit(addr));
    }
}
