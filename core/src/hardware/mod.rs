//! Hardware detection and classification utilities.
//!
//! This module provides lightweight device fingerprinting that extracts
//! vendor/product identifiers and infers a coarse device class that can be
//! used for profile lookup and calibration defaults.

pub mod classification;
pub mod cloud_sync;
pub mod database;
pub mod detector;
pub mod profile;

pub use classification::{DeviceClass, DeviceClassifier};
pub use cloud_sync::{
    CloudProfileBundle, CloudProfileSync, CloudSyncOutcome, HttpProfileClient, ProfileClient,
};
pub use database::ProfileDatabase;
pub use detector::{HardwareDetector, HardwareInfo};
pub use profile::{HardwareProfile, ProfileSource, TimingConfig};
