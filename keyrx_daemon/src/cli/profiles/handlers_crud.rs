//! Profile set mutation subcommand handlers: `create`, `delete`, `duplicate`.

use super::output::{ProfileCreatedOutput, SuccessOutput};
use crate::cli::common::output_error;
use crate::cli::logging;
use crate::config::profile_manager::{ProfileError, ProfileTemplate};
use crate::error::{CliError, DaemonResult};
use crate::services::ProfileService;

/// Handle the `create` subcommand.
pub(super) async fn handle_create(
    service: &ProfileService,
    name: &str,
    template: ProfileTemplate,
    json: bool,
) -> DaemonResult<()> {
    logging::log_command_start("profiles create", name);

    match service.create_profile(name, template).await {
        Ok(profile) => {
            logging::log_profile_create(name, profile.layer_count);
            logging::log_command_success("profiles create", 0);

            if json {
                let output = ProfileCreatedOutput {
                    success: true,
                    name: profile.name.clone(),
                    rhai_path: profile.rhai_path.display().to_string(),
                    layer_count: profile.layer_count,
                };
                if let Ok(json) = serde_json::to_string_pretty(&output) {
                    println!("{}", json);
                }
            } else {
                println!("✓ Profile '{}' created", name);
                println!("  Layers: {}", profile.layer_count);
                println!();
                println!("Edit the profile:");
                println!("  $EDITOR {}", profile.rhai_path.display());
                println!();
                println!("Activate the profile:");
                println!("  keyrx_daemon profiles activate {}", name);
            }
            Ok(())
        }
        Err(ProfileError::InvalidName(msg)) => {
            logging::log_command_error("profiles create", &format!("Invalid name: {}", msg));
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::ProfileLimitExceeded) => {
            logging::log_command_error("profiles create", "Profile limit exceeded (max 100)");
            output_error("Profile limit exceeded (max 100)", 1014, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::AlreadyExists(name)) => {
            logging::log_command_error(
                "profiles create",
                &format!("Profile '{}' already exists", name),
            );
            output_error(&format!("Profile '{}' already exists", name), 1015, json);
            Err(CliError::Reported.into())
        }
        Err(e) => {
            logging::log_command_error(
                "profiles create",
                &format!("Failed to create profile: {}", e),
            );
            output_error(&format!("Failed to create profile: {}", e), 3001, json);
            Err(CliError::Reported.into())
        }
    }
}

/// Handle the `delete` subcommand.
pub(super) async fn handle_delete(
    service: &ProfileService,
    name: &str,
    confirm: bool,
    json: bool,
) -> DaemonResult<()> {
    // Confirmation prompt if not using --confirm flag
    if !confirm && !json {
        use std::io::{self, Write};
        print!("Delete profile '{}'? This cannot be undone. [y/N]: ", name);
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    logging::log_command_start("profiles delete", name);

    match service.delete_profile(name).await {
        Ok(()) => {
            logging::log_profile_delete(name);
            logging::log_command_success("profiles delete", 0);

            if json {
                let output = SuccessOutput {
                    success: true,
                    message: format!("Profile '{}' deleted", name),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Profile '{}' deleted", name);
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            logging::log_command_error("profiles delete", &format!("Profile '{}' not found", name));
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(CliError::Reported.into())
        }
        Err(e) => {
            logging::log_command_error(
                "profiles delete",
                &format!("Failed to delete profile: {}", e),
            );
            output_error(&format!("Failed to delete profile: {}", e), 3001, json);
            Err(CliError::Reported.into())
        }
    }
}

/// Handle the `duplicate` subcommand.
pub(super) async fn handle_duplicate(
    service: &ProfileService,
    src: &str,
    dest: &str,
    json: bool,
) -> DaemonResult<()> {
    match service.duplicate_profile(src, dest).await {
        Ok(profile) => {
            if json {
                let output = ProfileCreatedOutput {
                    success: true,
                    name: profile.name.clone(),
                    rhai_path: profile.rhai_path.display().to_string(),
                    layer_count: profile.layer_count,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Profile '{}' duplicated to '{}'", src, dest);
                println!("  Layers: {}", profile.layer_count);
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::InvalidName(msg)) => {
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::ProfileLimitExceeded) => {
            output_error("Profile limit exceeded (max 100)", 1014, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::AlreadyExists(name)) => {
            output_error(&format!("Profile '{}' already exists", name), 1015, json);
            Err(CliError::Reported.into())
        }
        Err(e) => {
            output_error(&format!("Failed to duplicate profile: {}", e), 3001, json);
            Err(CliError::Reported.into())
        }
    }
}
