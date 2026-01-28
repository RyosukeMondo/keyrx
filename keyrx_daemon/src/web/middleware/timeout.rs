//! Timeout middleware to prevent slow requests from tying up resources

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Duration;

/// Timeout configuration
#[derive(Clone)]
pub struct TimeoutConfig {
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(5),
        }
    }
}

/// Timeout middleware layer
#[derive(Clone)]
pub struct TimeoutLayer {
    config: TimeoutConfig,
}

impl TimeoutLayer {
    /// Create new timeout layer with default config
    pub fn new() -> Self {
        Self::with_config(TimeoutConfig::default())
    }

    /// Create timeout layer with custom config
    pub fn with_config(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }
}

impl Default for TimeoutLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeout middleware handler
pub async fn timeout_middleware(
    timeout: axum::extract::State<TimeoutLayer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    match tokio::time::timeout(timeout.config().request_timeout, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => Err((
            StatusCode::REQUEST_TIMEOUT,
            format!(
                "Request timeout after {:?}",
                timeout.config().request_timeout
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_timeout_fast_request() {
        async fn fast_handler() -> &'static str {
            "OK"
        }

        let timeout = TimeoutLayer::with_config(TimeoutConfig {
            request_timeout: Duration::from_secs(1),
        });

        let app = Router::new().route("/test", get(fast_handler)).layer(
            axum::middleware::from_fn_with_state(timeout, timeout_middleware),
        );

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_timeout_slow_request() {
        async fn slow_handler() -> &'static str {
            tokio::time::sleep(Duration::from_secs(2)).await;
            "OK"
        }

        let timeout = TimeoutLayer::with_config(TimeoutConfig {
            request_timeout: Duration::from_millis(100),
        });

        let app = Router::new().route("/test", get(slow_handler)).layer(
            axum::middleware::from_fn_with_state(timeout, timeout_middleware),
        );

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
