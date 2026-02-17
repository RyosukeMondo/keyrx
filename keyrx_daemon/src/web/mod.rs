pub mod api;
pub mod error;
pub mod events;
pub mod handlers;
pub mod middleware;
pub mod rpc_types;
pub mod static_files;
pub mod subscriptions;
pub mod ws;
pub mod ws_rpc;

#[cfg(test)]
mod ws_test;

use axum::{middleware as axum_middleware, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

pub use events::{DaemonEvent, ErrorData};
pub use middleware::{
    AuthMiddleware, InputValidationLayer, RateLimitLayer, SecurityHeadersLayer, SecurityLayer,
    TimeoutLayer,
};

use crate::container::ServiceContainer;
use crate::daemon::DaemonSharedState;
use crate::daemon_config::DaemonConfig;
use crate::macro_recorder::MacroRecorder;
use crate::services::{
    ConfigService, DaemonQueryService, DeviceService, ProfileService, SettingsService,
    SimulationService,
};
use crate::web::subscriptions::SubscriptionManager;

use crate::web::rpc_types::ServerMessage;

/// Application state shared across all web handlers
///
/// This struct contains all dependencies needed by the web API and is injected
/// via axum's State extraction pattern. This enables testability by allowing
/// mock implementations to be injected during tests.
#[derive(Clone)]
pub struct AppState {
    /// Macro recorder for capturing keyboard event sequences
    pub macro_recorder: Arc<MacroRecorder>,
    /// Profile service for profile management operations
    pub profile_service: Arc<ProfileService>,
    /// Device service for device management operations
    pub device_service: Arc<DeviceService>,
    /// Config service for configuration management operations
    pub config_service: Arc<ConfigService>,
    /// Settings service for daemon settings operations
    pub settings_service: Arc<SettingsService>,
    /// Simulation service for event simulation operations
    pub simulation_service: Arc<SimulationService>,
    /// Subscription manager for WebSocket pub/sub
    pub subscription_manager: Arc<SubscriptionManager>,
    /// Event broadcaster for sending events to WebSocket clients
    pub event_broadcaster: broadcast::Sender<ServerMessage>,
    /// Test mode IPC socket path (None in production mode)
    pub test_mode_socket: Option<std::path::PathBuf>,
    /// Shared daemon state for Windows IPC replacement (None on Linux/macOS)
    ///
    /// On Windows, where Unix domain sockets are not available, this provides
    /// direct access to daemon state from the web server thread. This enables
    /// the status API to query daemon state without IPC.
    ///
    /// This is `Some(state)` on Windows in single-process mode, `None` on Linux/macOS
    /// where IPC is used instead.
    pub daemon_state: Option<Arc<DaemonSharedState>>,
    /// Query service for daemon metrics (latency, events, status).
    /// Available when running with a real daemon (not test mode).
    pub daemon_query: Option<Arc<DaemonQueryService>>,
}

impl AppState {
    /// Creates a new AppState with the given dependencies
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        macro_recorder: Arc<MacroRecorder>,
        profile_service: Arc<ProfileService>,
        device_service: Arc<DeviceService>,
        config_service: Arc<ConfigService>,
        settings_service: Arc<SettingsService>,
        simulation_service: Arc<SimulationService>,
        subscription_manager: Arc<SubscriptionManager>,
        event_broadcaster: broadcast::Sender<ServerMessage>,
        daemon_state: Option<Arc<DaemonSharedState>>,
    ) -> Self {
        Self {
            macro_recorder,
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
            test_mode_socket: None,
            daemon_state,
            daemon_query: None,
        }
    }

    /// Creates a new AppState with test mode enabled
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_test_mode(
        macro_recorder: Arc<MacroRecorder>,
        profile_service: Arc<ProfileService>,
        device_service: Arc<DeviceService>,
        config_service: Arc<ConfigService>,
        settings_service: Arc<SettingsService>,
        simulation_service: Arc<SimulationService>,
        subscription_manager: Arc<SubscriptionManager>,
        event_broadcaster: broadcast::Sender<ServerMessage>,
        test_mode_socket: std::path::PathBuf,
        daemon_state: Option<Arc<DaemonSharedState>>,
    ) -> Self {
        Self {
            macro_recorder,
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
            test_mode_socket: Some(test_mode_socket),
            daemon_state,
            daemon_query: None,
        }
    }

    /// Creates a minimal AppState for testing with default services using ServiceContainer
    ///
    /// This method uses ServiceContainer for dependency injection, providing a cleaner
    /// approach to test setup that matches production initialization patterns.
    ///
    /// Note: This is public for integration tests but gated with cfg(test).
    pub fn new_for_testing(config_dir: std::path::PathBuf) -> Self {
        use crate::container::ServiceContainerBuilder;

        // Use ServiceContainer for consistent dependency injection
        let container = ServiceContainerBuilder::new(config_dir)
            .build()
            .expect("Failed to build ServiceContainer for testing");

        Self::from_container(container, None)
    }

    /// Create AppState from ServiceContainer
    ///
    /// This is the recommended way to create AppState, as it uses the ServiceContainer
    /// for proper dependency injection following the Dependency Inversion Principle.
    ///
    /// # Arguments
    ///
    /// * `container` - ServiceContainer with all dependencies wired
    /// * `test_mode_socket` - Optional IPC socket path for test mode
    pub fn from_container(
        container: ServiceContainer,
        test_mode_socket: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            macro_recorder: container.macro_recorder(),
            profile_service: container.profile_service(),
            device_service: container.device_service(),
            config_service: container.config_service(),
            settings_service: container.settings_service(),
            simulation_service: container.simulation_service(),
            subscription_manager: container.subscription_manager(),
            event_broadcaster: container.event_broadcaster(),
            test_mode_socket,
            daemon_state: None,
            daemon_query: None,
        }
    }

    /// Create AppState from ServiceContainer with daemon shared state
    ///
    /// This is the Windows-specific constructor that includes daemon shared state
    /// for direct daemon-to-web-server communication without IPC.
    ///
    /// # Arguments
    ///
    /// * `container` - ServiceContainer with all dependencies wired
    /// * `test_mode_socket` - Optional IPC socket path for test mode
    /// * `daemon_state` - Shared daemon state for Windows IPC replacement
    ///
    /// # Platform Support
    ///
    /// This method is primarily for Windows, where Unix domain sockets don't exist.
    /// On Linux/macOS, pass `None` for `daemon_state` and use IPC instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use keyrx_daemon::daemon::DaemonSharedState;
    /// use keyrx_daemon::container::ServiceContainer;
    /// use keyrx_daemon::web::AppState;
    ///
    /// # fn example(container: ServiceContainer, daemon_state: Arc<DaemonSharedState>) {
    /// let app_state = Arc::new(AppState::from_container_with_daemon(
    ///     container,
    ///     None, // No test mode
    ///     daemon_state,
    /// ));
    /// # }
    /// ```
    pub fn from_container_with_daemon(
        container: ServiceContainer,
        test_mode_socket: Option<std::path::PathBuf>,
        daemon_state: Arc<DaemonSharedState>,
        daemon_query: Option<Arc<DaemonQueryService>>,
    ) -> Self {
        Self {
            macro_recorder: container.macro_recorder(),
            profile_service: container.profile_service(),
            device_service: container.device_service(),
            config_service: container.config_service(),
            settings_service: container.settings_service(),
            simulation_service: container.simulation_service(),
            subscription_manager: container.subscription_manager(),
            event_broadcaster: container.event_broadcaster(),
            test_mode_socket,
            daemon_state: Some(daemon_state),
            daemon_query,
        }
    }

    /// Returns whether daemon state is available (Windows single-process mode)
    ///
    /// This is `true` on Windows where daemon state is shared directly,
    /// `false` on Linux/macOS where IPC is used instead.
    ///
    /// API endpoints can use this to determine whether to query shared state
    /// or fall back to IPC communication.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use keyrx_daemon::web::AppState;
    /// # fn example(state: AppState) {
    /// if state.has_daemon_state() {
    ///     // Use shared state (Windows)
    ///     println!("Using shared daemon state");
    /// } else {
    ///     // Use IPC (Linux/macOS)
    ///     println!("Using IPC communication");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn has_daemon_state(&self) -> bool {
        self.daemon_state.is_some()
    }
}

/// Build a CORS layer from DaemonConfig origins.
///
/// This is the single source of truth for CORS configuration (allowed methods,
/// headers, and origin parsing). Both `create_app_with_config` and `create_router`
/// use this to avoid duplicating the allowed-methods/headers list.
fn build_cors_layer(config: &DaemonConfig) -> CorsLayer {
    use tower_http::cors::AllowOrigin;

    let cors_origins = config.cors_origins();
    let allowed_origins: Vec<_> = cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    if allowed_origins.is_empty() {
        log::warn!(
            "No valid CORS origins configured. Check KEYRX_ALLOWED_ORIGINS environment variable."
        );
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ACCEPT_LANGUAGE,
            axum::http::header::ACCEPT_ENCODING,
            axum::http::header::USER_AGENT,
            axum::http::header::REFERER,
            axum::http::header::ORIGIN,
            // Browser User-Agent Client Hints headers
            axum::http::HeaderName::from_static("sec-ch-ua"),
            axum::http::HeaderName::from_static("sec-ch-ua-mobile"),
            axum::http::HeaderName::from_static("sec-ch-ua-platform"),
            axum::http::HeaderName::from_static("sec-fetch-site"),
            axum::http::HeaderName::from_static("sec-fetch-mode"),
            axum::http::HeaderName::from_static("sec-fetch-dest"),
        ])
}

#[allow(dead_code)]
pub async fn create_app(event_tx: broadcast::Sender<DaemonEvent>, state: Arc<AppState>) -> Router {
    create_app_with_config(event_tx, state, false).await
}

/// Creates an application router for testing with relaxed rate limits
///
/// This function is identical to `create_app` but uses test-friendly rate limiting
/// (1000 req/sec instead of 10 req/sec) to allow stress testing without hitting limits.
///
/// # Arguments
///
/// * `event_tx` - Channel for broadcasting daemon events
/// * `state` - Application state
///
/// # Returns
///
/// Configured router with test-friendly middleware
///
/// Note: This is public for integration tests but should only be used in test environments
pub async fn create_test_app(
    event_tx: broadcast::Sender<DaemonEvent>,
    state: Arc<AppState>,
) -> Router {
    create_app_with_config(event_tx, state, true).await
}

async fn create_app_with_config(
    event_tx: broadcast::Sender<DaemonEvent>,
    state: Arc<AppState>,
    test_mode: bool,
) -> Router {
    use crate::auth::AuthMode;
    use crate::web::middleware::rate_limit::RateLimitConfig;

    // Load configuration for CORS and security settings
    let config = DaemonConfig::from_env().unwrap_or_default();
    let cors = build_cors_layer(&config).allow_credentials(true);

    // Create security middleware layers (order matters: outer layers run first)
    let auth_mode = AuthMode::from_env();
    let auth_middleware = AuthMiddleware::new(auth_mode);
    let rate_limiter = if test_mode {
        RateLimitLayer::with_config(RateLimitConfig::test_mode())
    } else {
        RateLimitLayer::new()
    };
    let input_validator = InputValidationLayer::new();
    let security_layer = SecurityLayer::new();
    let timeout_layer = TimeoutLayer::new();

    // Security headers middleware (dev or production based on config)
    let security_headers_layer = if config.is_production {
        SecurityHeadersLayer::production()
    } else {
        SecurityHeadersLayer::dev()
    };

    Router::new()
        .nest("/api", api::create_router(Arc::clone(&state)))
        .nest("/ws", ws::create_router(event_tx))
        .nest("/ws-rpc", ws_rpc::create_router(Arc::clone(&state)))
        .fallback_service(static_files::serve_static())
        // Security layers (innermost to outermost):
        // Note: Middleware order is LIFO - last layer added runs first
        .layer(axum_middleware::from_fn_with_state(
            security_headers_layer,
            middleware::security_headers::security_headers_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            timeout_layer,
            middleware::timeout::timeout_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            security_layer,
            middleware::security::security_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            input_validator,
            middleware::input_validation::input_validation_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware,
            middleware::auth::auth_middleware,
        ))
        .layer(cors)
}

#[allow(dead_code)]
pub async fn serve(
    addr: SocketAddr,
    event_tx: broadcast::Sender<DaemonEvent>,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(event_tx, state).await;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Use into_make_service_with_connect_info to provide ConnectInfo extension
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Creates a router for testing without WebSocket event broadcasting
///
/// This is a simplified router creation for tests that don't need
/// full WebSocket functionality.
///
/// Note: This is public for integration tests but gated with cfg(test).
/// For production use, use create_app() instead.
pub fn create_router(state: Arc<AppState>) -> Router {
    use crate::auth::AuthMode;

    let config = DaemonConfig::from_env().unwrap_or_default();
    let cors = build_cors_layer(&config);

    let auth_mode = AuthMode::from_env();
    let auth_middleware = AuthMiddleware::new(auth_mode);

    Router::new()
        .nest("/api", api::create_router(Arc::clone(&state)))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware,
            middleware::auth::auth_middleware,
        ))
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn test_app_state_has_daemon_state_with_none() {
        let config_dir = std::env::temp_dir().join("keyrx-test-no-daemon-state");
        let state = AppState::new_for_testing(config_dir);

        // Without daemon state, should return false
        assert!(!state.has_daemon_state());
        assert!(state.daemon_state.is_none());
    }

    #[test]
    fn test_app_state_has_daemon_state_with_some() {
        use crate::container::ServiceContainerBuilder;

        let config_dir = std::env::temp_dir().join("keyrx-test-with-daemon-state");
        let container = ServiceContainerBuilder::new(config_dir)
            .build()
            .expect("Failed to build ServiceContainer");

        // Create a minimal daemon state for testing
        let running = Arc::new(AtomicBool::new(true));
        let daemon_state = Arc::new(DaemonSharedState::new(
            running,
            Some("test-profile".to_string()),
            PathBuf::from("/test/config.krx"),
            2,
        ));

        let state = AppState::from_container_with_daemon(container, None, daemon_state, None);

        // With daemon state, should return true
        assert!(state.has_daemon_state());
        assert!(state.daemon_state.is_some());

        // Verify we can access daemon state
        let daemon_state_ref = state.daemon_state.as_ref().unwrap();
        assert!(daemon_state_ref.is_running());
        assert_eq!(
            daemon_state_ref.get_active_profile(),
            Some("test-profile".to_string())
        );
    }

    #[test]
    fn test_app_state_from_container_without_daemon_state() {
        use crate::container::ServiceContainerBuilder;

        let config_dir = std::env::temp_dir().join("keyrx-test-no-daemon");
        let container = ServiceContainerBuilder::new(config_dir)
            .build()
            .expect("Failed to build ServiceContainer");

        let state = AppState::from_container(container, None);

        // Should have no daemon state
        assert!(!state.has_daemon_state());
        assert!(state.daemon_state.is_none());
    }

    #[test]
    fn test_app_state_new_includes_daemon_state_parameter() {
        use crate::container::ServiceContainerBuilder;

        let config_dir = std::env::temp_dir().join("keyrx-test-new-with-daemon");
        let container = ServiceContainerBuilder::new(config_dir)
            .build()
            .expect("Failed to build ServiceContainer");

        // Test with None
        let state_none = AppState::new(
            container.macro_recorder(),
            container.profile_service(),
            container.device_service(),
            container.config_service(),
            container.settings_service(),
            container.simulation_service(),
            container.subscription_manager(),
            container.event_broadcaster(),
            None,
        );
        assert!(!state_none.has_daemon_state());

        // Test with Some
        let running = Arc::new(AtomicBool::new(false));
        let daemon_state = Arc::new(DaemonSharedState::new(
            running,
            None,
            PathBuf::from("/test.krx"),
            0,
        ));
        let state_some = AppState::new(
            container.macro_recorder(),
            container.profile_service(),
            container.device_service(),
            container.config_service(),
            container.settings_service(),
            container.simulation_service(),
            container.subscription_manager(),
            container.event_broadcaster(),
            Some(daemon_state),
        );
        assert!(state_some.has_daemon_state());
    }
}
