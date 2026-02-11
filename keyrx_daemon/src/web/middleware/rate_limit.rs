//! Rate limiting middleware to prevent DoS attacks
//!
//! Implements per-IP rate limiting with configurable limits and endpoint-specific limits

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
    /// Maximum requests per time window (general API endpoints)
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
    /// Maximum login attempts per time window
    pub max_login_attempts: usize,
    /// Login attempt window duration
    pub login_window: Duration,
    /// Maximum WebSocket connections per IP
    pub max_ws_connections: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // General API: 100 requests/minute
            max_requests: 100,
            window: Duration::from_secs(60),
            // Login endpoint: 5 attempts/minute
            max_login_attempts: 5,
            login_window: Duration::from_secs(60),
            // WebSocket: 10 connections per IP
            max_ws_connections: 10,
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
            max_login_attempts: 100,
            login_window: Duration::from_secs(1),
            max_ws_connections: 100,
        }
    }
}

/// Rate limiter state
struct RateLimiterState {
    /// General request counts per IP
    counters: HashMap<SocketAddr, (Instant, usize)>,
    /// Login attempt counts per IP
    login_counters: HashMap<SocketAddr, (Instant, usize)>,
    /// WebSocket connection counts per IP
    ws_connections: HashMap<SocketAddr, usize>,
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
                login_counters: HashMap::new(),
                ws_connections: HashMap::new(),
                config,
            })),
        }
    }

    /// Check if general request is allowed
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

    /// Check if login attempt is allowed
    pub fn check_login_limit(&self, addr: SocketAddr) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let window = state.config.login_window;
        let max_attempts = state.config.max_login_attempts;

        // Clean up old entries
        state
            .login_counters
            .retain(|_, (timestamp, _)| now.duration_since(*timestamp) < window);

        // Check current IP
        let entry = state.login_counters.entry(addr).or_insert((now, 0));

        // Reset counter if window expired
        if now.duration_since(entry.0) >= window {
            *entry = (now, 0);
        }

        // Increment counter
        entry.1 += 1;

        // Check limit
        entry.1 <= max_attempts
    }

    /// Check if WebSocket connection is allowed
    pub fn check_ws_connection_limit(&self, addr: SocketAddr) -> bool {
        let state = self.state.lock().unwrap();
        let count = state.ws_connections.get(&addr).copied().unwrap_or(0);
        count < state.config.max_ws_connections
    }

    /// Register new WebSocket connection
    pub fn register_ws_connection(&self, addr: SocketAddr) {
        let mut state = self.state.lock().unwrap();
        *state.ws_connections.entry(addr).or_insert(0) += 1;
    }

    /// Unregister WebSocket connection
    pub fn unregister_ws_connection(&self, addr: SocketAddr) {
        let mut state = self.state.lock().unwrap();
        if let Some(count) = state.ws_connections.get_mut(&addr) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.ws_connections.remove(&addr);
            }
        }
    }
}

impl Default for RateLimitLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting middleware handler with endpoint-specific limits
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

    let path = request.uri().path();

    // Apply endpoint-specific rate limits
    let is_allowed = if is_login_endpoint(path) {
        // Login endpoint: stricter limit (5/minute)
        if !limiter.check_login_limit(addr) {
            log::warn!(
                "Login rate limit exceeded for IP: {} (5 attempts/minute)",
                addr
            );
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "Too many login attempts. Please try again later.",
            ));
        }
        true
    } else if is_ws_endpoint(path) {
        // WebSocket endpoint: connection limit check
        if !limiter.check_ws_connection_limit(addr) {
            log::warn!(
                "WebSocket connection limit exceeded for IP: {} (10 connections max)",
                addr
            );
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "Maximum WebSocket connections reached for your IP.",
            ));
        }
        true
    } else {
        // General API endpoints: 100/minute
        limiter.check_rate_limit(addr)
    };

    if is_allowed {
        Ok(next.run(request).await)
    } else {
        log::warn!(
            "API rate limit exceeded for IP: {} (100 requests/minute)",
            addr
        );
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Maximum 100 requests per minute.",
        ))
    }
}

/// Check if path is a login endpoint
fn is_login_endpoint(path: &str) -> bool {
    path.contains("/login") || path.contains("/auth")
}

/// Check if path is a WebSocket endpoint
fn is_ws_endpoint(path: &str) -> bool {
    path.starts_with("/ws") || path.contains("/websocket")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(1),
            max_login_attempts: 5,
            login_window: Duration::from_secs(60),
            max_ws_connections: 10,
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
            max_login_attempts: 5,
            login_window: Duration::from_secs(60),
            max_ws_connections: 10,
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

    #[test]
    fn test_login_rate_limit() {
        let config = RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60),
            max_login_attempts: 3,
            login_window: Duration::from_secs(60),
            max_ws_connections: 10,
        };
        let limiter = RateLimitLayer::with_config(config);
        let addr = "127.0.0.1:8080".parse().unwrap();

        // First 3 login attempts should succeed
        assert!(limiter.check_login_limit(addr));
        assert!(limiter.check_login_limit(addr));
        assert!(limiter.check_login_limit(addr));

        // 4th attempt should fail
        assert!(!limiter.check_login_limit(addr));
    }

    #[test]
    fn test_ws_connection_limit() {
        let limiter = RateLimitLayer::new();
        let addr = "127.0.0.1:8080".parse().unwrap();

        // Register connections up to limit
        for _ in 0..10 {
            assert!(limiter.check_ws_connection_limit(addr));
            limiter.register_ws_connection(addr);
        }

        // 11th connection should be rejected
        assert!(!limiter.check_ws_connection_limit(addr));

        // Unregister one connection
        limiter.unregister_ws_connection(addr);

        // Now should allow one more
        assert!(limiter.check_ws_connection_limit(addr));
    }

    #[tokio::test]
    async fn test_login_limit_window_reset() {
        let config = RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60),
            max_login_attempts: 2,
            login_window: Duration::from_millis(100),
            max_ws_connections: 10,
        };
        let limiter = RateLimitLayer::with_config(config);
        let addr = "127.0.0.1:8080".parse().unwrap();

        // Use up login attempts
        assert!(limiter.check_login_limit(addr));
        assert!(limiter.check_login_limit(addr));
        assert!(!limiter.check_login_limit(addr));

        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be allowed again
        assert!(limiter.check_login_limit(addr));
    }
}
