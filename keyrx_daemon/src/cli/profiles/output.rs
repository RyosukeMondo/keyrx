//! JSON output structures for profile management CLI commands.

use serde::Serialize;

/// JSON output structure for profile list.
#[derive(Serialize)]
pub(super) struct ProfileListOutput {
    pub(super) profiles: Vec<ProfileInfo>,
    pub(super) active: Option<String>,
}

/// Profile info for JSON serialization.
#[derive(Serialize)]
pub(super) struct ProfileInfo {
    pub(super) name: String,
    pub(super) layer_count: usize,
    pub(super) modified_at: std::time::SystemTime,
}

/// JSON output structure for activation.
#[derive(Serialize)]
pub(super) struct ActivationOutput {
    pub(super) success: bool,
    pub(super) compile_time_ms: u64,
    pub(super) reload_time_ms: u64,
    pub(super) error: Option<String>,
    /// What the running daemon did with the activation.
    pub(super) daemon: Option<String>,
}

/// JSON output structure for profile creation.
#[derive(Serialize)]
pub(super) struct ProfileCreatedOutput {
    pub(super) success: bool,
    pub(super) name: String,
    pub(super) rhai_path: String,
    pub(super) layer_count: usize,
}

/// JSON output structure for success operations.
#[derive(Serialize)]
pub(super) struct SuccessOutput {
    pub(super) success: bool,
    pub(super) message: String,
}
