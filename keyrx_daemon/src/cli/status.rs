//! Status CLI command.
//!
//! This module implements the `keyrx status` command for querying daemon status
//! via IPC. Displays running state, uptime, active profile, and device count.

use crate::ipc::client::IpcClient;
use crate::ipc::{DaemonIpc, IpcEndpoint, IpcRequest, IpcResponse};
use clap::Args;
use serde::Serialize;

/// Status subcommands.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Output as JSON.
    #[arg(long)]
    pub json: bool,

    /// Custom IPC endpoint: socket path, or pipe name on Windows (defaults to
    /// $XDG_RUNTIME_DIR/keyrx-daemon.sock, or the keyrx-daemon named pipe on Windows).
    #[arg(long)]
    pub socket: Option<String>,
}

/// JSON output structure for status.
#[derive(Serialize)]
struct StatusOutput {
    running: bool,
    uptime_secs: u64,
    active_profile: Option<String>,
    device_count: usize,
    input_overflows: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_error: Option<String>,
}

/// Execute the status command.
pub fn execute(args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut ipc = IpcClient::new(IpcEndpoint::from_cli(args.socket.as_deref()));

    // Send GetStatus request
    let response = ipc.send_request(&IpcRequest::GetStatus)?;

    // Parse response
    match response {
        IpcResponse::Status {
            running,
            uptime_secs,
            active_profile,
            device_count,
            input_overflows,
            config_error,
        } => {
            if args.json {
                print_json_output(
                    running,
                    uptime_secs,
                    active_profile,
                    device_count,
                    input_overflows,
                    config_error,
                )?;
            } else {
                print_human_output(
                    running,
                    uptime_secs,
                    active_profile,
                    device_count,
                    input_overflows,
                    config_error,
                );
            }
            Ok(())
        }
        IpcResponse::Error { code, message } => {
            Err(format!("Daemon error {}: {}", code, message).into())
        }
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// Print JSON output.
fn print_json_output(
    running: bool,
    uptime_secs: u64,
    active_profile: Option<String>,
    device_count: usize,
    input_overflows: u64,
    config_error: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = StatusOutput {
        running,
        uptime_secs,
        active_profile,
        device_count,
        input_overflows,
        config_error,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print human-readable output.
fn print_human_output(
    running: bool,
    uptime_secs: u64,
    active_profile: Option<String>,
    device_count: usize,
    input_overflows: u64,
    config_error: Option<String>,
) {
    println!("Daemon Status:");
    println!("  Running:        {}", if running { "Yes" } else { "No" });
    println!("  Uptime:         {} seconds", uptime_secs);

    // Format uptime in human-readable form
    let hours = uptime_secs / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    let seconds = uptime_secs % 60;
    println!("                  ({}h {}m {}s)", hours, minutes, seconds);

    println!(
        "  Active Profile: {}",
        active_profile.unwrap_or_else(|| "None".to_string())
    );
    println!("  Device Count:   {}", device_count);
    if let Some(error) = config_error {
        println!("  Config error:   {error}");
        println!("                  (no keyboard is grabbed until a valid profile is active)");
    }
    if input_overflows > 0 {
        println!(
            "  Input overflows: {} (keyboard input outran the daemon; resynced)",
            input_overflows
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_output_format() {
        let output = StatusOutput {
            running: true,
            uptime_secs: 3661,
            active_profile: Some("default".to_string()),
            device_count: 2,
            input_overflows: 0,
            config_error: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"running\":true"));
        assert!(json.contains("\"uptime_secs\":3661"));
        assert!(json.contains("\"active_profile\":\"default\""));
        assert!(json.contains("\"device_count\":2"));
    }

    #[test]
    fn test_status_output_no_profile() {
        let output = StatusOutput {
            running: false,
            uptime_secs: 0,
            active_profile: None,
            device_count: 0,
            input_overflows: 0,
            config_error: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"running\":false"));
        assert!(json.contains("\"active_profile\":null"));
    }
}
