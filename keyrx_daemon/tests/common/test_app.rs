//! Test application fixture for integration testing.
//!
//! Provides TestApp struct with isolated configuration directory and HTTP helpers.

use keyrx_daemon::macro_recorder::MacroRecorder;
use keyrx_daemon::services::{ConfigService, DeviceService, ProfileService};
use keyrx_daemon::web::{create_app, AppState, DaemonEvent};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::broadcast;

/// Test application fixture with isolated configuration.
///
/// Provides an isolated test environment with:
/// - Temporary configuration directory
/// - In-memory web server
/// - HTTP request helpers
///
/// Each TestApp instance is isolated and can run in parallel with other tests.
pub struct TestApp {
    /// Temporary directory for configuration files (automatically cleaned up on drop)
    pub config_dir: TempDir,
    /// Base URL for HTTP requests (e.g., "http://127.0.0.1:3000")
    pub base_url: String,
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Server task handle (server runs in background)
    _server_handle: tokio::task::JoinHandle<()>,
}

impl TestApp {
    /// Creates a new test application with isolated config directory.
    ///
    /// # Example
    /// ```no_run
    /// mod common;
    /// use common::test_app::TestApp;
    ///
    /// #[tokio::test]
    /// async fn test_api() {
    ///     let app = TestApp::new().await;
    ///     let response = app.get("/api/status").await;
    ///     assert_eq!(response.status(), 200);
    /// }
    /// ```
    pub async fn new() -> Self {
        // Create isolated config directory with proper structure
        // We need HOME/.config/keyrx structure because device API uses get_config_dir()
        let temp_home = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_home.path().join(".config").join("keyrx");
        std::fs::create_dir_all(&config_path).expect("Failed to create config directory");

        // Set HOME environment variable so get_config_dir() works correctly
        std::env::set_var("HOME", temp_home.path());

        // Create services with isolated config directory
        let config_dir = temp_home;
        let profile_manager = Arc::new(
            keyrx_daemon::config::ProfileManager::new(config_path.clone())
                .expect("Failed to create ProfileManager"),
        );
        let profile_service = Arc::new(ProfileService::new(profile_manager.clone()));
        let device_service = Arc::new(DeviceService::new(config_path.clone()));
        let config_service = Arc::new(ConfigService::new(profile_manager));
        let macro_recorder = Arc::new(MacroRecorder::new());
        let subscription_manager =
            Arc::new(keyrx_daemon::web::subscriptions::SubscriptionManager::new());

        // Create app state
        let state = Arc::new(AppState::new(
            macro_recorder,
            profile_service,
            device_service,
            config_service,
            subscription_manager,
        ));

        // Create event channel
        let (event_tx, _event_rx) = broadcast::channel::<DaemonEvent>(100);

        // Create router
        let app = create_app(event_tx, state).await;

        // Bind to random available port (127.0.0.1:0 lets OS choose)
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        // Spawn server in background
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Server failed to start");
        });

        // Wait for server to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            config_dir,
            base_url,
            client: reqwest::Client::new(),
            _server_handle: server_handle,
        }
    }

    /// Returns the path to the isolated config directory (HOME/.config/keyrx).
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.path().join(".config").join("keyrx")
    }

    /// Sends a GET request to the specified path.
    ///
    /// # Arguments
    /// * `path` - URL path (e.g., "/api/profiles")
    ///
    /// # Example
    /// ```no_run
    /// let response = app.get("/api/profiles").await;
    /// assert_eq!(response.status(), 200);
    /// ```
    pub async fn get(&self, path: &str) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .get(&url)
            .send()
            .await
            .expect("Failed to send GET request")
    }

    /// Sends a POST request with JSON body to the specified path.
    ///
    /// # Arguments
    /// * `path` - URL path (e.g., "/api/profiles")
    /// * `body` - JSON-serializable body
    ///
    /// # Example
    /// ```no_run
    /// use serde_json::json;
    ///
    /// let body = json!({"name": "test-profile"});
    /// let response = app.post("/api/profiles", &body).await;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[allow(dead_code)]
    pub async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .expect("Failed to send POST request")
    }

    /// Sends a PATCH request with JSON body to the specified path.
    ///
    /// # Arguments
    /// * `path` - URL path (e.g., "/api/devices/ABC123")
    /// * `body` - JSON-serializable body
    ///
    /// # Example
    /// ```no_run
    /// use serde_json::json;
    ///
    /// let body = json!({"layout": "ansi"});
    /// let response = app.patch("/api/devices/ABC123", &body).await;
    /// assert_eq!(response.status(), 200);
    /// ```
    #[allow(dead_code)]
    pub async fn patch<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .patch(&url)
            .json(body)
            .send()
            .await
            .expect("Failed to send PATCH request")
    }

    /// Sends a PUT request with JSON body to the specified path.
    ///
    /// # Arguments
    /// * `path` - URL path (e.g., "/api/devices/ABC123/layout")
    /// * `body` - JSON-serializable body
    ///
    /// # Example
    /// ```no_run
    /// use serde_json::json;
    ///
    /// let body = json!({"layout": "ansi"});
    /// let response = app.put("/api/devices/ABC123/layout", &body).await;
    /// assert_eq!(response.status(), 200);
    /// ```
    #[allow(dead_code)]
    pub async fn put<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .put(&url)
            .json(body)
            .send()
            .await
            .expect("Failed to send PUT request")
    }

    /// Sends a DELETE request to the specified path.
    ///
    /// # Arguments
    /// * `path` - URL path (e.g., "/api/profiles/test-profile")
    ///
    /// # Example
    /// ```no_run
    /// let response = app.delete("/api/profiles/test-profile").await;
    /// assert_eq!(response.status(), 204);
    /// ```
    #[allow(dead_code)]
    pub async fn delete(&self, path: &str) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .delete(&url)
            .send()
            .await
            .expect("Failed to send DELETE request")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creates_isolated_config() {
        let app = TestApp::new().await;
        let config_path = app.config_path();

        // Verify temp directory exists
        assert!(config_path.exists());
        assert!(config_path.is_dir());
    }

    #[tokio::test]
    async fn test_app_http_helpers_work() {
        let app = TestApp::new().await;

        // Test GET request
        let response = app.get("/api/status").await;
        assert!(response.status().is_success() || response.status().is_client_error());

        // Response should be valid (server is running)
        assert!(response.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_multiple_apps_isolated() {
        // Create two test apps in parallel
        let app1 = TestApp::new().await;
        let app2 = TestApp::new().await;

        // Verify different config directories
        assert_ne!(app1.config_path(), app2.config_path());

        // Verify different ports
        assert_ne!(app1.base_url, app2.base_url);
    }
}
