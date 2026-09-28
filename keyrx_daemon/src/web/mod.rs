pub mod api;
pub mod error;
pub mod events;
pub mod handlers;
pub mod mcp;
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
use crate::daemon::{DaemonSharedState, DaemonTelemetry};
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
    /// Control handle on the running keyboard daemon (suspend, reload).
    /// `None` when this process has no daemon (test mode, unit tests). When
    /// present it is the same state `daemon_query` reads.
    pub daemon_state: Option<Arc<DaemonSharedState>>,
    /// The single read model for status/state/latency/events. Always present:
    /// handlers never fall back to IPC or to a second source.
    pub daemon_query: Arc<DaemonQueryService>,
}

impl AppState {
    /// Creates AppState from individual services. With `daemon_state`, status
    /// reads that state; without it the process reports no running daemon.
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
        if let Some(ds) = &daemon_state {
            profile_service.attach_daemon_state(Arc::clone(ds));
        }
        let daemon_query = Arc::new(match &daemon_state {
            Some(ds) => DaemonQueryService::new(Arc::clone(ds), Arc::new(DaemonTelemetry::new())),
            None => DaemonQueryService::without_daemon(),
        });
        Self {
            macro_recorder,
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
            daemon_state,
            daemon_query,
        }
    }

    /// Creates AppState for test mode (no keyboard daemon). `daemon_query` must
    /// be the same instance the process's IPC handler uses.
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
        daemon_query: Arc<DaemonQueryService>,
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
            daemon_state: None,
            daemon_query,
        }
    }

    /// Creates AppState for tests with a temporary config directory.
    pub fn new_for_testing(config_dir: std::path::PathBuf) -> Self {
        use crate::container::ServiceContainerBuilder;

        let container = ServiceContainerBuilder::new(config_dir)
            .build()
            .expect("Failed to build ServiceContainer for testing");

        Self::from_container(container)
    }

    /// Creates AppState from a ServiceContainer, with no keyboard daemon.
    pub fn from_container(container: ServiceContainer) -> Self {
        Self::build(
            container,
            None,
            Arc::new(DaemonQueryService::without_daemon()),
        )
    }

    /// Creates AppState from a ServiceContainer for a running daemon. The
    /// control handle is taken from `daemon_query`, so reads and control
    /// operations always refer to the same daemon state.
    pub fn from_container_with_daemon(
        container: ServiceContainer,
        daemon_query: Arc<DaemonQueryService>,
    ) -> Self {
        let daemon_state = Some(Arc::clone(daemon_query.shared_state()));
        Self::build(container, daemon_state, daemon_query)
    }

    fn build(
        container: ServiceContainer,
        daemon_state: Option<Arc<DaemonSharedState>>,
        daemon_query: Arc<DaemonQueryService>,
    ) -> Self {
        if let Some(ds) = &daemon_state {
            container
                .profile_service()
                .attach_daemon_state(Arc::clone(ds));
        }
        Self {
            macro_recorder: container.macro_recorder(),
            profile_service: container.profile_service(),
            device_service: container.device_service(),
            config_service: container.config_service(),
            settings_service: container.settings_service(),
            simulation_service: container.simulation_service(),
            subscription_manager: container.subscription_manager(),
            event_broadcaster: container.event_broadcaster(),
            daemon_state,
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
        .nest_service("/mcp", mcp::create_service(Arc::clone(&state)))
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
        // Outermost: refuse foreign origins / DNS-rebound hosts first.
        .layer(axum_middleware::from_fn_with_state(
            middleware::origin_guard::OriginGuard::from_config(&config),
            middleware::origin_guard::origin_guard_middleware,
        ))
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

        let query = Arc::new(DaemonQueryService::new(
            Arc::clone(&daemon_state),
            Arc::new(DaemonTelemetry::new()),
        ));
        let state = AppState::from_container_with_daemon(container, query);

        // Invariant: the control handle and the read model are the same state.
        assert!(Arc::ptr_eq(
            state.daemon_state.as_ref().unwrap(),
            state.daemon_query.shared_state()
        ));
        assert_eq!(state.daemon_query.get_status().device_count, 2);

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

        let state = AppState::from_container(container);

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
