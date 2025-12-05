//! Row-column to KeyCode resolution for position-based remapping.
//!
//! This module provides the `RowColResolver` which translates physical
//! keyboard positions (row, column) into `KeyCode` values by:
//! 1. Looking up the scan_code from the device profile
//! 2. Converting scan_code to KeyCode using platform-specific mappings

use crate::discovery::types::DeviceProfile;
use crate::engine::KeyCode;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[cfg(target_os = "linux")]
use crate::drivers::keycodes::evdev_to_keycode;

#[cfg(target_os = "windows")]
use crate::drivers::keycodes::vk_to_keycode;

/// Errors that can occur during row-col resolution.
#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("No device profile loaded. Row-column API requires device discovery.")]
    NoProfileLoaded,

    #[error(
        "Position r{row}_c{col} not found in device profile{device_hint}.\n\
         Hint: Run './scripts/show_key_position.sh all' to see available positions."
    )]
    PositionNotFound {
        row: u8,
        col: u8,
        device_hint: String,
    },

    #[error("scan_code {0} could not be converted to KeyCode")]
    ScanCodeConversionFailed(u16),
}

/// Resolves (row, col) positions to KeyCode using device profile.
///
/// The resolver performs a two-step translation:
/// 1. Device Profile Lookup: (row, col) → scan_code
/// 2. Platform Conversion: scan_code → KeyCode
///
/// This is done at script load time, not runtime, so performance is not critical.
#[derive(Debug, Clone)]
pub struct RowColResolver {
    device_profile: Arc<RwLock<Option<Arc<DeviceProfile>>>>,
}

impl RowColResolver {
    /// Create a new resolver with an optional device profile.
    pub fn new(device_profile: Option<Arc<DeviceProfile>>) -> Self {
        Self {
            device_profile: Arc::new(RwLock::new(device_profile)),
        }
    }

    /// Create a resolver without a device profile (will fail on all resolutions).
    pub fn without_profile() -> Self {
        Self {
            device_profile: Arc::new(RwLock::new(None)),
        }
    }

    /// Load a device profile into this resolver.
    /// This allows updating the profile after the resolver has been created and shared.
    pub fn load_profile(&self, profile: Arc<DeviceProfile>) {
        if let Ok(mut guard) = self.device_profile.write() {
            *guard = Some(profile);
        }
    }

    /// Resolve a (row, col) position to a KeyCode.
    ///
    /// # Arguments
    /// * `row` - 0-based row number
    /// * `col` - 0-based column number
    ///
    /// # Returns
    /// * `Ok(KeyCode)` - The resolved key code
    /// * `Err(ResolverError)` - If position not found or no profile loaded
    ///
    /// # Example
    /// ```ignore
    /// let resolver = RowColResolver::new(Some(profile));
    /// let key = resolver.resolve(3, 1)?;  // Home row, 2nd key
    /// assert_eq!(key, KeyCode::A);  // On QWERTY
    /// ```
    pub fn resolve(&self, row: u8, col: u8) -> Result<KeyCode, ResolverError> {
        // Step 1: Lookup scan_code from device profile
        let scan_code = self.lookup_scan_code(row, col)?;

        // Step 2: Convert scan_code to KeyCode (platform-specific)
        let key_code = self.scan_code_to_keycode(scan_code)?;

        Ok(key_code)
    }

    /// Look up the scan_code for a given (row, col) position in the device profile.
    fn lookup_scan_code(&self, row: u8, col: u8) -> Result<u16, ResolverError> {
        let profile_guard = self
            .device_profile
            .read()
            .map_err(|_| ResolverError::NoProfileLoaded)?;

        let profile = profile_guard
            .as_ref()
            .ok_or(ResolverError::NoProfileLoaded)?;

        // Search through keymap for matching row-col position
        profile
            .keymap
            .iter()
            .find(|(_, physical_key)| physical_key.row == row && physical_key.col == col)
            .map(|(scan_code, _)| *scan_code)
            .ok_or_else(|| {
                let device_hint = profile
                    .name
                    .as_ref()
                    .map(|name| format!(" for {}", name))
                    .unwrap_or_default();

                ResolverError::PositionNotFound {
                    row,
                    col,
                    device_hint,
                }
            })
    }

    /// Convert a scan_code to KeyCode using platform-specific mappings.
    ///
    /// On Linux: Uses evdev codes
    /// On Windows: Uses Virtual Key codes (note: scan codes and VK codes differ)
    fn scan_code_to_keycode(&self, scan_code: u16) -> Result<KeyCode, ResolverError> {
        #[cfg(target_os = "linux")]
        {
            // On Linux, scan_code IS the evdev code
            let key_code = evdev_to_keycode(scan_code);
            if matches!(key_code, KeyCode::Unknown(_)) {
                return Err(ResolverError::ScanCodeConversionFailed(scan_code));
            }
            Ok(key_code)
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we need to handle this differently
            // Note: Windows device profiles store scan codes differently
            // For now, we'll use a placeholder. This needs proper Windows implementation.
            // TODO: Implement proper Windows scan_code → VK → KeyCode conversion
            let key_code = vk_to_keycode(scan_code as u16);
            if matches!(key_code, KeyCode::Unknown(_)) {
                return Err(ResolverError::ScanCodeConversionFailed(scan_code));
            }
            Ok(key_code)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(ResolverError::ScanCodeConversionFailed(scan_code))
        }
    }

    /// Check if a device profile is loaded.
    pub fn has_profile(&self) -> bool {
        self.device_profile
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|_| true))
            .unwrap_or(false)
    }

    /// Get the device name from the loaded profile, if available.
    pub fn device_name(&self) -> Option<String> {
        self.device_profile
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().and_then(|p| p.name.clone()))
    }
}
