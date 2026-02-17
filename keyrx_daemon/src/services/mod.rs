//! Service layer for business logic.
//!
//! This module provides service-layer abstractions that act as a single source
//! of truth for business operations, shared between CLI and Web API.

pub mod config_service;
pub mod daemon_query;
pub mod device_service;
pub mod profile_service;
pub mod settings_service;
pub mod simulation_service;

pub use config_service::ConfigService;
pub use daemon_query::DaemonQueryService;
pub use device_service::DeviceService;
pub use profile_service::ProfileService;
pub use settings_service::{DaemonSettings, SettingsService, DEFAULT_PORT};
pub use simulation_service::SimulationService;
