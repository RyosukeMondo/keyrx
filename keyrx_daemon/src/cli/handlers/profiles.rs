//! Profile management command handler.

use crate::cli::dispatcher::exit_codes;
use crate::cli::profiles::ProfilesArgs;
use crate::config::ProfileManager;
use crate::services::ProfileService;
use std::path::PathBuf;
use std::sync::Arc;

/// Handle the profiles command.
///
/// Initializes the ProfileManager and ProfileService, then executes the
/// profiles subcommand.
///
/// # Arguments
///
/// * `args` - Profile command arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn handle_profiles(args: ProfilesArgs) -> Result<(), (i32, String)> {
    // Determine config directory (KEYRX_CONFIG_DIR > dirs::config_dir)
    let config_dir = if let Ok(dir) = std::env::var("KEYRX_CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

    // Initialize ProfileManager
    let manager = match ProfileManager::new(config_dir) {
        Ok(mgr) => Arc::new(mgr),
        Err(e) => {
            return Err((
                exit_codes::CONFIG_ERROR,
                format!("Failed to initialize profile manager: {}", e),
            ));
        }
    };

    // Create ProfileService
    let service = ProfileService::new(manager);

    // Create async runtime for CLI commands
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to create async runtime: {}", e),
        )
    })?;

    // Execute command
    rt.block_on(async {
        match crate::cli::profiles::execute(args, &service).await {
            Ok(()) => Ok(()),
            Err(err) => Err((exit_codes::CONFIG_ERROR, err.to_string())),
        }
    })
}
