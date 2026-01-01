//! Profile management CLI commands.
//!
//! This module implements the `keyrx profiles` command and all its subcommands
//! for managing Rhai configuration profiles, including creation, activation,
//! deletion, duplication, import, and export.

use crate::cli::common::output_error;
use crate::cli::logging;
use crate::config::profile_manager::{ProfileError, ProfileTemplate};
use crate::services::ProfileService;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Profile management subcommands.
#[derive(Args)]
pub struct ProfilesArgs {
    #[command(subcommand)]
    command: ProfilesCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum ProfilesCommands {
    /// List all profiles.
    List,

    /// Create a new profile from a template.
    Create {
        /// Profile name (max 32 chars).
        name: String,

        /// Template to use: "blank" (default) or "qmk-layers".
        #[arg(long, default_value = "blank", value_parser = parse_template)]
        template: ProfileTemplate,
    },

    /// Activate a profile (hot-reload with compilation).
    Activate {
        /// Profile name to activate.
        name: String,
    },

    /// Delete a profile.
    Delete {
        /// Profile name to delete.
        name: String,

        /// Skip confirmation prompt.
        #[arg(long)]
        confirm: bool,
    },

    /// Duplicate a profile.
    Duplicate {
        /// Source profile name.
        src: String,

        /// Destination profile name.
        dest: String,
    },

    /// Export a profile to a file.
    Export {
        /// Profile name to export.
        name: String,

        /// Output file path.
        output: PathBuf,
    },

    /// Import a profile from a file.
    Import {
        /// Input file path.
        input: PathBuf,

        /// Profile name.
        name: String,
    },
}

/// JSON output structure for profile list.
#[derive(Serialize)]
struct ProfileListOutput {
    profiles: Vec<ProfileInfo>,
    active: Option<String>,
}

/// Profile info for JSON serialization.
#[derive(Serialize)]
struct ProfileInfo {
    name: String,
    layer_count: usize,
    modified_at: std::time::SystemTime,
}

/// JSON output structure for activation.
#[derive(Serialize)]
struct ActivationOutput {
    success: bool,
    compile_time_ms: u64,
    reload_time_ms: u64,
    error: Option<String>,
}

/// JSON output structure for profile creation.
#[derive(Serialize)]
struct ProfileCreatedOutput {
    success: bool,
    name: String,
    rhai_path: String,
    layer_count: usize,
}

/// JSON output structure for success operations.
#[derive(Serialize)]
struct SuccessOutput {
    success: bool,
    message: String,
}

/// Parse template string to ProfileTemplate enum.
fn parse_template(s: &str) -> Result<ProfileTemplate, String> {
    match s.to_lowercase().as_str() {
        "blank" => Ok(ProfileTemplate::Blank),
        "qmk-layers" | "qmk" => Ok(ProfileTemplate::QmkLayers),
        _ => Err(format!(
            "Invalid template '{}'. Use 'blank' or 'qmk-layers'.",
            s
        )),
    }
}

/// Execute the profiles command.
pub async fn execute(args: ProfilesArgs, service: &ProfileService) -> Result<(), i32> {
    match args.command {
        ProfilesCommands::List => handle_list(service, args.json).await,
        ProfilesCommands::Create { name, template } => {
            handle_create(service, &name, template, args.json).await
        }
        ProfilesCommands::Activate { name } => handle_activate(service, &name, args.json).await,
        ProfilesCommands::Delete { name, confirm } => {
            handle_delete(service, &name, confirm, args.json).await
        }
        ProfilesCommands::Duplicate { src, dest } => {
            handle_duplicate(service, &src, &dest, args.json).await
        }
        ProfilesCommands::Export { name, output } => {
            handle_export(service, &name, &output, args.json).await
        }
        ProfilesCommands::Import { input, name } => {
            handle_import(service, &input, &name, args.json).await
        }
    }
}

/// Handle the `list` subcommand.
async fn handle_list(service: &ProfileService, json: bool) -> Result<(), i32> {
    let profiles = match service.list_profiles().await {
        Ok(p) => p,
        Err(e) => {
            output_error(&format!("Failed to list profiles: {}", e), 3001, json);
            return Err(1);
        }
    };
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
            println!("  keyrx profiles create <name>");
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

/// Handle the `create` subcommand.
async fn handle_create(
    service: &ProfileService,
    name: &str,
    template: ProfileTemplate,
    json: bool,
) -> Result<(), i32> {
    logging::log_command_start("profiles create", name);

    match service.create_profile(name, template).await {
        Ok(profile) => {
            logging::log_profile_create(name, profile.layer_count);
            logging::log_command_success("profiles create", 0);

            if json {
                let output = ProfileCreatedOutput {
                    success: true,
                    name: profile.name.clone(),
                    rhai_path: format!("{}/{}.rhai", "~/.config/keyrx/profiles", profile.name),
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
                println!("  $EDITOR ~/.config/keyrx/profiles/{}.rhai", profile.name);
                println!();
                println!("Activate the profile:");
                println!("  keyrx profiles activate {}", name);
            }
            Ok(())
        }
        Err(ProfileError::InvalidName(msg)) => {
            logging::log_command_error("profiles create", &format!("Invalid name: {}", msg));
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(1)
        }
        Err(ProfileError::ProfileLimitExceeded) => {
            logging::log_command_error("profiles create", "Profile limit exceeded (max 100)");
            output_error("Profile limit exceeded (max 100)", 1014, json);
            Err(1)
        }
        Err(ProfileError::AlreadyExists(name)) => {
            logging::log_command_error(
                "profiles create",
                &format!("Profile '{}' already exists", name),
            );
            output_error(&format!("Profile '{}' already exists", name), 1015, json);
            Err(1)
        }
        Err(e) => {
            logging::log_command_error(
                "profiles create",
                &format!("Failed to create profile: {}", e),
            );
            output_error(&format!("Failed to create profile: {}", e), 3001, json);
            Err(1)
        }
    }
}

/// Handle the `activate` subcommand.
async fn handle_activate(service: &ProfileService, name: &str, json: bool) -> Result<(), i32> {
    logging::log_command_start("profiles activate", name);

    match service.activate_profile(name).await {
        Ok(result) => {
            logging::log_profile_activate(name, result.success);
            if result.success {
                logging::log_command_success(
                    "profiles activate",
                    result.compile_time_ms + result.reload_time_ms,
                );
            } else {
                let error_msg = result
                    .error
                    .as_deref()
                    .unwrap_or("Unknown activation error");
                logging::log_command_error("profiles activate", error_msg);
            }

            if json {
                let output = ActivationOutput {
                    success: result.success,
                    compile_time_ms: result.compile_time_ms,
                    reload_time_ms: result.reload_time_ms,
                    error: result.error,
                };
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else if result.success {
                println!("✓ Profile '{}' activated", name);
                println!("  Compile time: {}ms", result.compile_time_ms);
                println!("  Reload time: {}ms", result.reload_time_ms);
                println!(
                    "  Total: {}ms",
                    result.compile_time_ms + result.reload_time_ms
                );
            } else {
                eprintln!("✗ Activation failed");
                if let Some(error) = result.error {
                    eprintln!("  Error: {}", error);
                }
                eprintln!("  Compile time: {}ms", result.compile_time_ms);
                return Err(1);
            }

            if result.success {
                Ok(())
            } else {
                Err(1)
            }
        }
        Err(ProfileError::NotFound(name)) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Profile '{}' not found", name),
            );
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(1)
        }
        Err(ProfileError::Compilation(e)) => {
            logging::log_command_error("profiles activate", &format!("Compilation error: {}", e));
            output_error(&format!("Compilation error: {}", e), 2004, json);
            Err(1)
        }
        Err(e) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Failed to activate profile: {}", e),
            );
            output_error(&format!("Failed to activate profile: {}", e), 2001, json);
            Err(1)
        }
    }
}

/// Handle the `delete` subcommand.
async fn handle_delete(
    service: &ProfileService,
    name: &str,
    confirm: bool,
    json: bool,
) -> Result<(), i32> {
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
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!("✓ Profile '{}' deleted", name);
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            logging::log_command_error("profiles delete", &format!("Profile '{}' not found", name));
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(1)
        }
        Err(e) => {
            logging::log_command_error(
                "profiles delete",
                &format!("Failed to delete profile: {}", e),
            );
            output_error(&format!("Failed to delete profile: {}", e), 3001, json);
            Err(1)
        }
    }
}

/// Handle the `duplicate` subcommand.
async fn handle_duplicate(
    service: &ProfileService,
    src: &str,
    dest: &str,
    json: bool,
) -> Result<(), i32> {
    match service.duplicate_profile(src, dest).await {
        Ok(profile) => {
            if json {
                let output = ProfileCreatedOutput {
                    success: true,
                    name: profile.name.clone(),
                    rhai_path: format!("~/.config/keyrx/profiles/{}.rhai", profile.name),
                    layer_count: profile.layer_count,
                };
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!("✓ Profile '{}' duplicated to '{}'", src, dest);
                println!("  Layers: {}", profile.layer_count);
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(1)
        }
        Err(ProfileError::InvalidName(msg)) => {
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(1)
        }
        Err(ProfileError::ProfileLimitExceeded) => {
            output_error("Profile limit exceeded (max 100)", 1014, json);
            Err(1)
        }
        Err(ProfileError::AlreadyExists(name)) => {
            output_error(&format!("Profile '{}' already exists", name), 1015, json);
            Err(1)
        }
        Err(e) => {
            output_error(&format!("Failed to duplicate profile: {}", e), 3001, json);
            Err(1)
        }
    }
}

/// Handle the `export` subcommand.
async fn handle_export(
    service: &ProfileService,
    name: &str,
    output: &Path,
    json: bool,
) -> Result<(), i32> {
    match service.export_profile(name, output).await {
        Ok(()) => {
            if json {
                let output_msg = SuccessOutput {
                    success: true,
                    message: format!("Profile '{}' exported to {}", name, output.display()),
                };
                println!("{}", serde_json::to_string_pretty(&output_msg).unwrap());
            } else {
                println!("✓ Profile '{}' exported to {}", name, output.display());
            }
            Ok(())
        }
        Err(ProfileError::NotFound(name)) => {
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(1)
        }
        Err(e) => {
            output_error(&format!("Failed to export profile: {}", e), 3001, json);
            Err(1)
        }
    }
}

/// Handle the `import` subcommand.
async fn handle_import(
    service: &ProfileService,
    input: &Path,
    name: &str,
    json: bool,
) -> Result<(), i32> {
    match service.import_profile(input, name).await {
        Ok(profile) => {
            if json {
                let output = ProfileCreatedOutput {
                    success: true,
                    name: profile.name.clone(),
                    rhai_path: format!("~/.config/keyrx/profiles/{}.rhai", profile.name),
                    layer_count: profile.layer_count,
                };
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!("✓ Profile '{}' imported from {}", name, input.display());
                println!("  Layers: {}", profile.layer_count);
            }
            Ok(())
        }
        Err(ProfileError::InvalidName(msg)) => {
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(1)
        }
        Err(ProfileError::ProfileLimitExceeded) => {
            output_error("Profile limit exceeded (max 100)", 1014, json);
            Err(1)
        }
        Err(ProfileError::AlreadyExists(name)) => {
            output_error(&format!("Profile '{}' already exists", name), 1015, json);
            Err(1)
        }
        Err(e) => {
            output_error(&format!("Failed to import profile: {}", e), 3001, json);
            Err(1)
        }
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format system time as a relative time string.
fn format_time(time: &std::time::SystemTime) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template() {
        assert!(matches!(
            parse_template("blank").unwrap(),
            ProfileTemplate::Blank
        ));
        assert!(matches!(
            parse_template("qmk-layers").unwrap(),
            ProfileTemplate::QmkLayers
        ));
        assert!(matches!(
            parse_template("qmk").unwrap(),
            ProfileTemplate::QmkLayers
        ));
        assert!(parse_template("invalid").is_err());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("verylongstring", 8), "veryl...");
        assert_eq!(truncate("abc", 2), "ab");
    }

    #[test]
    fn test_format_time() {
        use std::time::{Duration, SystemTime};

        let now = SystemTime::now();
        assert_eq!(format_time(&now), "just now");

        let five_min_ago = now - Duration::from_secs(300);
        assert_eq!(format_time(&five_min_ago), "5m ago");

        let two_hours_ago = now - Duration::from_secs(7200);
        assert_eq!(format_time(&two_hours_ago), "2h ago");

        let three_days_ago = now - Duration::from_secs(259200);
        assert_eq!(format_time(&three_days_ago), "3d ago");
    }
}
