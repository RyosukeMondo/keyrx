//! Read and file-transfer subcommand handlers: `list`, `export`, `import`.

use super::output::{ProfileCreatedOutput, ProfileInfo, ProfileListOutput, SuccessOutput};
use crate::cli::common::output_error;
use crate::config::profile_manager::ProfileError;
use crate::error::{CliError, DaemonResult};
use crate::services::{LayoutFormat, ProfileService, MAX_LAYOUT_BYTES};
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

/// Reads a layout file, refusing oversized ones before loading them.
fn read_layout(input: &Path) -> Result<Vec<u8>, ProfileError> {
    let len = std::fs::metadata(input)?.len();
    if len > MAX_LAYOUT_BYTES as u64 {
        return Err(ProfileError::InvalidLayout(format!(
            "the file is {len} bytes; the limit is {MAX_LAYOUT_BYTES} bytes"
        )));
    }
    Ok(std::fs::read(input)?)
}

/// Handle the `import` subcommand.
pub(super) async fn handle_import(
    service: &ProfileService,
    input: &Path,
    name: Option<&str>,
    activate: bool,
    json: bool,
) -> DaemonResult<()> {
    let name = name
        .map(str::to_string)
        .or_else(|| {
            input
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let imported = async {
        let format = LayoutFormat::from_path(input)?;
        let bytes = read_layout(input)?;
        service.import_layout(&name, format, bytes).await
    }
    .await;
    match imported {
        Ok(imported) => {
            let profile = &imported.profile;
            if activate {
                if let Err(e) = service.activate_profile(&profile.name).await {
                    output_error(
                        &format!("Imported '{}' but could not activate it: {}", name, e),
                        3001,
                        json,
                    );
                    return Err(CliError::Reported.into());
                }
            }
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
                if imported.converted {
                    println!("  Converted from .krx to editable Rhai (comments are not stored in a .krx)");
                }
                for warning in &imported.warnings {
                    println!("  Warning: {warning}");
                }
                println!("  Layers: {}", profile.layer_count);
                if activate {
                    println!("  Activated");
                }
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
            output_error(
                &format!("Profile '{}' already exists; pass a different name", name),
                1015,
                json,
            );
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
