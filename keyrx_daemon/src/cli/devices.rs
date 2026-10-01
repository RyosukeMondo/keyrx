//! Device management CLI commands.
//!
//! This module implements the `keyrx devices` command and all its subcommands
//! for managing device metadata, including renaming and layout assignment.

use crate::cli::common::output_error;
use crate::cli::logging;
use crate::config::device_registry::{DeviceEntry, DeviceRegistry, DeviceValidationError};
use crate::error::{CliError, DaemonResult};
use crate::ipc::CapturedDevice;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Device management subcommands.
#[derive(Args, Debug)]
pub struct DevicesArgs {
    #[command(subcommand)]
    command: DevicesCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum DevicesCommands {
    /// List the keyboards the running daemon manages (and registered devices).
    List,

    /// Rename a device.
    Rename {
        /// Device ID to rename.
        device_id: String,

        /// New name for the device (max 64 chars).
        new_name: String,
    },

    /// Forget (remove) a device from the registry.
    Forget {
        /// Device ID to forget.
        device_id: String,
    },

    /// Set keyboard layout for a device.
    SetLayout {
        /// Device ID.
        device_id: String,

        /// Layout name (e.g., "ansi_104", "iso_105").
        layout: String,
    },
}

/// JSON output structure for device list.
#[derive(Serialize)]
struct DeviceListOutput {
    /// Registry entries (names / layouts the user assigned).
    devices: Vec<DeviceEntry>,
    /// Whether a daemon answered on the IPC endpoint.
    daemon_running: bool,
    /// Keyboards that daemon has captured right now (empty if not running).
    captured: Vec<CapturedDevice>,
}

/// Asks the running daemon which keyboards it has captured. `None` when no
/// daemon is reachable.
fn captured_by_daemon() -> Option<Vec<CapturedDevice>> {
    use crate::ipc::client::IpcClient;
    use crate::ipc::{DaemonIpc, IpcEndpoint, IpcRequest, IpcResponse};

    let mut ipc = IpcClient::new(IpcEndpoint::default_for_platform());
    match ipc.send_request(&IpcRequest::GetDevices) {
        Ok(IpcResponse::Devices { devices }) => Some(devices),
        _ => None,
    }
}

/// JSON output structure for success operations.
#[derive(Serialize)]
struct SuccessOutput {
    success: bool,
    message: String,
}

/// Execute the devices command.
pub fn execute(args: DevicesArgs, registry_path: Option<PathBuf>) -> DaemonResult<()> {
    // Save json flag for error handling
    let json = args.json;

    // Execute command and handle errors
    let result = execute_inner(args, registry_path);

    // Format errors based on JSON flag before returning (if not already formatted)
    // Note: Individual handlers may have already called output_error, but we ensure
    // consistent output here for any errors that bubble up
    if let Err(e) = &result {
        // Check if error message is just "Command failed" (already handled by handler)
        // In that case, output_error was already called, so don't duplicate
        if !e.to_string().contains("Command failed") {
            output_error(&e.to_string(), 1, json);
        }
    }

    result
}

/// Inner execute function that returns errors for formatting.
fn execute_inner(args: DevicesArgs, registry_path: Option<PathBuf>) -> DaemonResult<()> {
    // Determine registry path (KEYRX_CONFIG_DIR > dirs::config_dir)
    let registry_path = registry_path.unwrap_or_else(|| {
        let base = crate::cli::config_dir::get_config_dir().unwrap_or_else(|_| PathBuf::from("."));
        base.join("devices.json")
    });

    // Load or create registry (with automatic recovery from corruption)
    let mut registry = match DeviceRegistry::load(&registry_path) {
        Ok(reg) => reg,
        Err(e) => {
            output_error(
                &format!("Failed to load device registry: {}", e),
                1001,
                args.json,
            );
            return Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into());
        }
    };

    match args.command {
        DevicesCommands::List => handle_list(&registry, args.json),
        DevicesCommands::Rename {
            device_id,
            new_name,
        } => handle_rename(&mut registry, &device_id, &new_name, args.json),
        DevicesCommands::Forget { device_id } => {
            handle_forget(&mut registry, &device_id, args.json)
        }
        DevicesCommands::SetLayout { device_id, layout } => {
            handle_set_layout(&mut registry, &device_id, &layout, args.json)
        }
    }
}

/// Handle the `list` subcommand.
fn handle_list(registry: &DeviceRegistry, json: bool) -> DaemonResult<()> {
    let devices = registry.list();
    let captured = captured_by_daemon();

    if json {
        let output = DeviceListOutput {
            devices: devices.into_iter().cloned().collect(),
            daemon_running: captured.is_some(),
            captured: captured.unwrap_or_default(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(CliError::from)?
        );
        return Ok(());
    }

    match &captured {
        Some(captured) => print_captured(captured),
        None => {
            println!("The keyrx daemon is not running, so no keyboards are captured.");
            println!("Start it with 'keyrx_daemon run'; 'keyrx_daemon list-devices' shows");
            println!("the keyboards connected right now.");
            println!();
        }
    }
    print_registered(&devices);
    Ok(())
}

/// Prints the keyboards the running daemon has grabbed.
fn print_captured(captured: &[CapturedDevice]) {
    println!("Captured by the running daemon:");
    if captured.is_empty() {
        println!("  (none - no keyboard matches the active profile's device_start patterns)");
        println!();
        return;
    }
    println!("{:<40} {:<30} PATH", "ID", "NAME");
    println!("{}", "-".repeat(95));
    for device in captured {
        println!(
            "{:<40} {:<30} {}",
            truncate(&device.id, 40),
            truncate(&device.name, 30),
            device.path
        );
    }
    println!();
}

/// Prints the registry (user-assigned names and layouts).
fn print_registered(devices: &[&DeviceEntry]) {
    if devices.is_empty() {
        println!("No devices registered (rename a device or set its layout to add it).");
        return;
    }

    println!("Registered Devices:");
    println!();
    println!("{:<40} {:<25} LAYOUT", "ID", "NAME");
    println!("{}", "-".repeat(85));

    for device in devices {
        let layout_str = device.layout.as_deref().unwrap_or("-");

        println!(
            "{:<40} {:<25} {}",
            truncate(&device.id, 40),
            truncate(&device.name, 25),
            layout_str
        );
    }

    println!();
    println!("Total: {} device(s)", devices.len());
}

/// Handle the `rename` subcommand.
fn handle_rename(
    registry: &mut DeviceRegistry,
    device_id: &str,
    new_name: &str,
    json: bool,
) -> DaemonResult<()> {
    logging::log_command_start("devices rename", &format!("{} -> {}", device_id, new_name));

    match registry.rename(device_id, new_name) {
        Ok(()) => {
            if let Err(e) = registry.save() {
                logging::log_command_error(
                    "devices rename",
                    &format!("Device renamed but failed to save: {}", e),
                );
                output_error(
                    &format!("Device renamed but failed to save: {}", e),
                    3001,
                    json,
                );
                return Err(CliError::CommandFailed {
                    command: "devices".to_string(),
                    reason: "Command failed".to_string(),
                }
                .into());
            }

            logging::log_device_operation("rename", device_id);
            logging::log_command_success("devices rename", 0);

            if json {
                let output = SuccessOutput {
                    success: true,
                    message: format!("Device '{}' renamed to '{}'", device_id, new_name),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Device '{}' renamed to '{}'", device_id, new_name);
            }
            Ok(())
        }
        Err(DeviceValidationError::DeviceNotFound(id)) => {
            logging::log_command_error(
                "devices rename",
                &format!("Device '{}' not found in registry", id),
            );
            output_error(
                &format!("Device '{}' not found in registry", id),
                1001,
                json,
            );
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(DeviceValidationError::InvalidName(msg)) => {
            logging::log_command_error("devices rename", &format!("Invalid name: {}", msg));
            output_error(&format!("Invalid name: {}", msg), 1006, json);
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(e) => {
            logging::log_command_error(
                "devices rename",
                &format!("Failed to rename device: {}", e),
            );
            output_error(&format!("Failed to rename device: {}", e), 3001, json);
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
    }
}

/// Handle the `set-scope` subcommand.
/// Handle the `forget` subcommand.
fn handle_forget(registry: &mut DeviceRegistry, device_id: &str, json: bool) -> DaemonResult<()> {
    match registry.forget(device_id) {
        Ok(device) => {
            if let Err(e) = registry.save() {
                output_error(
                    &format!("Device forgotten but failed to save: {}", e),
                    3001,
                    json,
                );
                return Err(CliError::CommandFailed {
                    command: "devices".to_string(),
                    reason: "Command failed".to_string(),
                }
                .into());
            }

            if json {
                let output = SuccessOutput {
                    success: true,
                    message: format!("Device '{}' forgotten", device.name),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Device '{}' forgotten", device.name);
            }
            Ok(())
        }
        Err(DeviceValidationError::DeviceNotFound(id)) => {
            output_error(
                &format!("Device '{}' not found in registry", id),
                1001,
                json,
            );
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(e) => {
            output_error(&format!("Failed to forget device: {}", e), 3001, json);
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
    }
}

/// Handle the `set-layout` subcommand.
fn handle_set_layout(
    registry: &mut DeviceRegistry,
    device_id: &str,
    layout: &str,
    json: bool,
) -> DaemonResult<()> {
    match registry.set_layout(device_id, layout) {
        Ok(()) => {
            if let Err(e) = registry.save() {
                output_error(
                    &format!("Layout updated but failed to save: {}", e),
                    3001,
                    json,
                );
                return Err(CliError::CommandFailed {
                    command: "devices".to_string(),
                    reason: "Command failed".to_string(),
                }
                .into());
            }

            if json {
                let output = SuccessOutput {
                    success: true,
                    message: format!("Device '{}' layout set to '{}'", device_id, layout),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Device '{}' layout set to '{}'", device_id, layout);
            }
            Ok(())
        }
        Err(DeviceValidationError::DeviceNotFound(id)) => {
            output_error(
                &format!("Device '{}' not found in registry", id),
                1001,
                json,
            );
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(e) => {
            output_error(&format!("Failed to set layout: {}", e), 3001, json);
            Err(CliError::CommandFailed {
                command: "devices".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("verylongstring", 8), "veryl...");
        assert_eq!(truncate("abc", 2), "ab");
    }
}
