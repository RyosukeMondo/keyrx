use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use std::time::Instant;

/// Middleware to log HTTP requests and responses.
/// This logs details such that they can be captured by the LogBridge and sent to Dart.
pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let method = req.method().clone();
        let url = req.url().clone();
        let start = Instant::now();

        tracing::info!(
            target: "http_client",
            method = %method,
            url = %url,
            "Sending HTTP request"
        );

        // Run the request
        let response_result = next.run(req, extensions).await;

        let duration = start.elapsed();

        match &response_result {
            Ok(response) => {
                let status = response.status();
                tracing::info!(
                    target: "http_client",
                    method = %method,
                    url = %url,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    "HTTP request completed"
                );
            }
            Err(e) => {
                tracing::error!(
                    target: "http_client",
                    method = %method,
                    url = %url,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "HTTP request failed"
                );
            }
        }

        response_result
    }
}

/// Helper to create a client with the logging middleware.
pub fn wrap_client_with_logging(
    client: reqwest::Client,
) -> reqwest_middleware::ClientWithMiddleware {
    reqwest_middleware::ClientBuilder::new(client)
        // Add tracing middleware first to propagate context and create spans
        .with(reqwest_tracing::TracingMiddleware::default())
        .with(LoggingMiddleware)
        .build()
}
