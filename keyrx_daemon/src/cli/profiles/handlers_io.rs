//! Read and file-transfer subcommand handlers: `list`, `export`, `import`.

use super::output::{ProfileCreatedOutput, ProfileInfo, ProfileListOutput, SuccessOutput};
use crate::cli::common::output_error;
use crate::config::profile_manager::ProfileError;
use crate::error::{CliError, DaemonResult};
use crate::services::ProfileService;
use std::path::Path;

/// Handle the `list` subcommand.
pub(super) async fn handle_list(service: &ProfileService, json: bool) -> DaemonResult<()> {
    let profiles = service
        .list_profiles()
        .await
        .map_err(|e| CliError::CommandFailed {
            command: "list".to_string(),
            reason: format!("Failed to list profiles: {}", e),
        })?;
    let active = service.get_active_profile().await;

    if json {
        let profile_infos: Vec<ProfileInfo> = profiles
            .iter()
            .map(|p| ProfileInfo {
                name: p.name.clone(),
                layer_count: p.layer_count,
                modified_at: p.modified_at,
            })
            .collect();

        let output = ProfileListOutput {
            profiles: profile_infos,
            active,
        };
        if let Ok(json) = serde_json::to_string_pretty(&output) {
            println!("{}", json);
        }
    } else {
        if profiles.is_empty() {
            println!("No profiles found.");
            println!();
            println!("Create a new profile with:");
            println!("  keyrx_daemon profiles create <name>");
            return Ok(());
        }

        println!("Profiles:");
        println!();
        println!("{:<32} {:<15} {:<10} STATUS", "NAME", "LAYERS", "MODIFIED");
        println!("{}", "-".repeat(80));

        for profile in &profiles {
            let status = if profile.active { "active" } else { "-" };

            let modified = format_time(&profile.modified_at);

            println!(
                "{:<32} {:<15} {:<10} {}",
                truncate(&profile.name, 32),
                profile.layer_count,
                modified,
                status
            );
        }

        println!();
        println!("Total: {} profile(s)", profiles.len());

        if let Some(active_name) = active {
            println!("Active: {}", active_name);
        } else {
            println!("Active: None");
        }
    }

    Ok(())
}

/// Handle the `export` subcommand.
pub(super) async fn handle_export(
    service: &ProfileService,
    name: &str,
    output: &Path,
    json: bool,
) -> DaemonResult<()> {
    match service.export_profile(name, output).await {
        Ok(()) => {
            if json {
                let output_msg = SuccessOutput {
                    success: true,
                    message: format!("Profile '{}' exported to {}", name, output.display()),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output_msg).map_err(CliError::from)?
                );
            } else {
                println!("✓ Profile '{}' exported to {}", name, output.display());
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(CliError::Reported.into())
        }
        Err(e) => {
            output_error(&format!("Failed to export profile: {}", e), 3001, json);
            Err(CliError::Reported.into())
        }
    }
}

/// Handle the `import` subcommand.
pub(super) async fn handle_import(
    service: &ProfileService,
    input: &Path,
    name: &str,
    json: bool,
) -> DaemonResult<()> {
    match service.import_profile(input, name).await {
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
                println!("✓ Profile '{}' imported from {}", name, input.display());
                println!("  Layers: {}", profile.layer_count);
            }
            Ok(())
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
            output_error(&format!("Failed to import profile: {}", e), 3001, json);
            Err(CliError::Reported.into())
        }
    }
}

/// Truncate a string to a maximum length.
pub(super) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format system time as a relative time string.
pub(super) fn format_time(time: &std::time::SystemTime) -> String {
    use std::time::SystemTime;

    let duration = SystemTime::now().duration_since(*time).unwrap_or_default();

    let secs = duration.as_secs();

    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 604800 {
        format!("{}d ago", secs / 86400)
    } else {
        format!("{}w ago", secs / 604800)
    }
}
