//! REST API endpoints for the KeyRx daemon web interface.
//!
//! This module provides a complete REST API that exposes all CLI functionality
//! to the web UI. Endpoints are organized by domain into focused modules:
//! - `devices` - Device management
//! - `profiles` - Profile management
//! - `config` - Configuration and layer management
//! - `layouts` - Keyboard layout management
//! - `metrics` - Health checks, metrics, and monitoring
//! - `simulator` - Event simulation
//! - `macros` - Macro recorder
//!
//! Each module exports a `routes()` function that returns its router fragment.
//! The main router is assembled in `create_router()`.

use axum::Router;
use std::sync::Arc;

use crate::web::AppState;

pub mod config;
pub mod devices;
pub mod error;
pub mod layouts;
pub mod macros;
pub mod metrics;
pub mod profiles;
pub mod simulator;
pub mod validation;

// Re-export ApiError for convenience
pub use error::ApiError;

/// Creates the main API router by combining all domain-specific routers.
///
/// # Arguments
///
/// * `state` - Application state containing shared services
///
/// # Returns
///
/// Combined router with all API endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(metrics::routes())
        .merge(devices::routes())
        .merge(profiles::routes())
        .merge(config::routes())
        .merge(layouts::routes())
        .merge(simulator::routes())
        .merge(macros::routes())
        .with_state(state)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProfileManager;
    use crate::macro_recorder::MacroRecorder;
    use crate::services::ProfileService;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_create_router() {
        let config_dir = PathBuf::from("/tmp/keyrx-test");
        let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(crate::services::DeviceService::new(config_dir.clone()));
        let config_service = Arc::new(crate::services::ConfigService::new(profile_manager));
        let settings_service = Arc::new(crate::services::SettingsService::new(config_dir.clone()));
        let simulation_service = Arc::new(crate::services::SimulationService::new(
            config_dir.clone(),
            None,
        ));
        let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());
        let (event_broadcaster, _) = tokio::sync::broadcast::channel(1000);
        let state = Arc::new(AppState::new(
            Arc::new(MacroRecorder::new()),
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
        ));
        let router = create_router(state);
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_app_state_with_mock_recorder() {
        // This test demonstrates that we can inject a fresh MacroRecorder
        // instance for testing, proving dependency injection works
        let mock_recorder = Arc::new(MacroRecorder::new());
        let config_dir = PathBuf::from("/tmp/keyrx-test");
        let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(crate::services::DeviceService::new(config_dir.clone()));
        let config_service = Arc::new(crate::services::ConfigService::new(profile_manager));
        let settings_service = Arc::new(crate::services::SettingsService::new(config_dir.clone()));
        let simulation_service = Arc::new(crate::services::SimulationService::new(
            config_dir.clone(),
            None,
        ));
        let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());
        let (event_broadcaster, _) = tokio::sync::broadcast::channel(1000);
        let state = Arc::new(AppState::new(
            mock_recorder.clone(),
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
        ));

        // Verify state is accessible
        assert_eq!(state.macro_recorder.event_count(), 0);

        // Start recording using the mock
        mock_recorder
            .start_recording()
            .expect("Test: start_recording should succeed");
        assert!(state.macro_recorder.is_recording());
    }
}
