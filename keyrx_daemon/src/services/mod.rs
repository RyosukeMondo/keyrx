//! Service layer for business logic.
//!
//! This module provides service-layer abstractions that act as a single source
//! of truth for business operations, shared between CLI and Web API.

pub mod profile_service;

pub use profile_service::ProfileService;
