//! Hardware detection and classification utilities.
//!
//! This module provides lightweight device fingerprinting that extracts
//! vendor/product identifiers and infers a coarse device class that can be
//! used for profile lookup and calibration defaults.

pub mod detector;

pub use detector::{DeviceClass, HardwareDetector, HardwareInfo};
