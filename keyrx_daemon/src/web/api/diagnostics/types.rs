//! Response and status types shared across the diagnostics endpoints.

use serde::{Deserialize, Serialize};

/// Comprehensive diagnostics information
#[derive(Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    /// Daemon version from Cargo.toml
    pub version: String,
    /// Build timestamp
    pub build_time: String,
    /// Git commit hash
    pub git_hash: String,
    /// Binary file modification timestamp (if available)
    pub binary_timestamp: Option<String>,
    /// Whether running with administrator privileges
    pub admin_status: bool,
    /// Key blocker hook installation status
    pub hook_status: HookStatus,
    /// Platform information
    pub platform_info: PlatformInfo,
    /// Memory usage information
    pub memory_usage: MemoryUsage,
    /// Configuration validation status
    pub config_validation_status: ConfigStatus,
}

/// Hook installation status
#[derive(Serialize, Deserialize)]
pub struct HookStatus {
    /// Whether the hook is installed
    pub installed: bool,
    /// Number of keys currently being remapped
    pub remapped_keys_count: usize,
}

/// Platform information
#[derive(Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system name
    pub os: String,
    /// System architecture
    pub arch: String,
}

/// Memory usage information
#[derive(Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Process memory usage in bytes
    pub process_memory_bytes: u64,
    /// Process memory usage in human-readable format
    pub process_memory_human: String,
}

/// Configuration validation status
#[derive(Serialize, Deserialize)]
pub struct ConfigStatus {
    /// Whether configuration is valid
    pub valid: bool,
    /// Validation message or error
    pub message: String,
}

/// Simple build information for quick verification
#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub build_time: String,
    pub git_hash: String,
    pub binary_timestamp: Option<String>,
}

/// Response for keyboard label detection
#[derive(Serialize, Deserialize)]
pub struct KeyboardLabelsResponse {
    /// Detected keyboard layout name
    pub detected_layout: String,
    /// Map of KeyCode name -> display label
    pub labels: std::collections::HashMap<String, String>,
}
