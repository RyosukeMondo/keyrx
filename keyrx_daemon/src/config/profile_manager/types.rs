//! Data types for profile management.
//!
//! Metadata, creation templates, activation/reload results, and the error type shared
//! by every submodule of `profile_manager`.

use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use typeshare::typeshare;

use crate::config::profile_compiler::CompilationError;

use super::MAX_PROFILES;

/// Metadata for a single profile.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub name: String,
    #[typeshare(skip)]
    pub rhai_path: PathBuf,
    #[typeshare(skip)]
    pub krx_path: PathBuf,
    #[typeshare(skip)]
    pub modified_at: SystemTime,
    #[typeshare(serialized_as = "number")]
    pub layer_count: usize,
    /// `device_start` blocks in the compiled profile.
    #[typeshare(serialized_as = "number")]
    pub device_count: usize,
    /// Key mappings in the compiled profile (all blocks and layers).
    #[typeshare(serialized_as = "number")]
    pub key_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub activated_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_by: Option<String>,
}

/// Template for creating new profiles.
#[typeshare]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileTemplate {
    /// Empty configuration with minimal valid syntax
    Blank,
    /// Simple A→B key remapping example
    SimpleRemap,
    /// CapsLock→Escape mapping
    CapslockEscape,
    /// Vim navigation with HJKL layer
    VimNavigation,
    /// Gaming-optimized profile
    Gaming,
}

impl ProfileTemplate {
    /// Every template, in the order they are documented. THE list: the CLI
    /// help, REST, MCP and WS-RPC all derive their accepted names from it.
    pub const ALL: [ProfileTemplate; 5] = [
        ProfileTemplate::Blank,
        ProfileTemplate::SimpleRemap,
        ProfileTemplate::CapslockEscape,
        ProfileTemplate::VimNavigation,
        ProfileTemplate::Gaming,
    ];

    /// The template's name as typed by users (`snake_case`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::SimpleRemap => "simple_remap",
            Self::CapslockEscape => "capslock_escape",
            Self::VimNavigation => "vim_navigation",
            Self::Gaming => "gaming",
        }
    }

    /// The template's `.rhai` source, embedded at build time.
    pub fn source(self) -> &'static str {
        match self {
            Self::Blank => include_str!("../../../templates/blank.rhai"),
            Self::SimpleRemap => include_str!("../../../templates/simple_remap.rhai"),
            Self::CapslockEscape => include_str!("../../../templates/capslock_escape.rhai"),
            Self::VimNavigation => include_str!("../../../templates/vim_navigation.rhai"),
            Self::Gaming => include_str!("../../../templates/gaming.rhai"),
        }
    }

    /// `"blank, simple_remap, ..."` for help and error messages.
    pub fn names_list() -> String {
        Self::ALL.map(Self::name).join(", ")
    }

    /// Parses a user-typed name (case-insensitive; `-` or `_`).
    ///
    /// # Errors
    ///
    /// The message names the offending input and every valid template.
    pub fn from_name(name: &str) -> Result<Self, String> {
        let wanted = name.to_lowercase().replace('-', "_");
        Self::ALL
            .into_iter()
            .find(|t| t.name() == wanted)
            .ok_or_else(|| {
                format!(
                    "Invalid template '{name}'. Valid templates: {}",
                    Self::names_list()
                )
            })
    }
}

/// Result of profile activation.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationResult {
    #[typeshare(serialized_as = "number")]
    pub compile_time_ms: u64,
    #[typeshare(serialized_as = "number")]
    pub reload_time_ms: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of reloading the active profile.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadResult {
    pub recompiled: bool,
    #[typeshare(serialized_as = "number")]
    pub compile_time_ms: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Errors that can occur during profile operations.
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Invalid profile name: {0}")]
    InvalidName(String),

    #[error("Compilation error: {0}")]
    Compilation(#[from] CompilationError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Profile limit exceeded (max {MAX_PROFILES})")]
    ProfileLimitExceeded,

    #[error("Disk space exhausted")]
    DiskSpaceExhausted,

    #[error("Profile already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid template")]
    InvalidTemplate,

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Activation in progress for profile: {0}")]
    ActivationInProgress(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
}
