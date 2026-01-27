pub mod api;
pub mod error;
pub mod events;
pub mod handlers;
pub mod rpc_types;
pub mod static_files;
pub mod subscriptions;
pub mod ws;
pub mod ws_rpc;

#[cfg(test)]
mod ws_test;

use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

pub use events::DaemonEvent;

use crate::macro_recorder::MacroRecorder;
use crate::services::{
    ConfigService, DeviceService, ProfileService, SettingsService, SimulationService,
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
        }
    }

    /// Creates a minimal AppState for testing with default services
    ///
    /// This method creates all required services with minimal configuration,
    /// suitable for integration tests that don't need full daemon functionality.
    ///
    /// Note: This is public for integration tests but gated with cfg(test).
    pub fn new_for_testing(config_dir: std::path::PathBuf) -> Self {
        use crate::config::profile_manager::ProfileManager;

        let macro_recorder = Arc::new(MacroRecorder::new());

        // Create ProfileManager first
        let profile_manager = Arc::new(
            ProfileManager::new(config_dir.clone())
                .expect("Failed to create ProfileManager for testing")
        );

        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(DeviceService::new(config_dir.clone()));
        let config_service = Arc::new(ConfigService::new(Arc::clone(&profile_manager)));
        let settings_service = Arc::new(SettingsService::new(config_dir.clone()));

        let simulation_service = Arc::new(SimulationService::new(config_dir, None));
        let subscription_manager = Arc::new(SubscriptionManager::new());
        let (event_broadcaster, _) = broadcast::channel(100);

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
        }
    }
}

#[allow(dead_code)]
pub async fn create_app(event_tx: broadcast::Sender<DaemonEvent>, state: Arc<AppState>) -> Router {
    // Configure CORS to allow requests from Vite dev server (localhost:5173)
    // and any other origins for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api", api::create_router(Arc::clone(&state)))
        .nest("/ws", ws::create_router(event_tx))
        .nest("/ws-rpc", ws_rpc::create_router(Arc::clone(&state)))
        .fallback_service(static_files::serve_static())
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
    axum::serve(listener, app).await?;
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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api", api::create_router(Arc::clone(&state)))
        .layer(cors)
}
