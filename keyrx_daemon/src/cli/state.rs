//! State CLI command.
//!
//! This module implements the `keyrx state inspect` command for querying the
//! daemon's current runtime state via IPC. Displays the 255-bit modifier/lock
//! state as a JSON array or human-readable format.

use crate::ipc::unix_socket::UnixSocketIpc;
use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

/// State subcommand arguments.
#[derive(Args)]
pub struct StateArgs {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: StateCommand,
}

/// State subcommands.
#[derive(clap::Subcommand)]
pub enum StateCommand {
    /// Inspect the current modifier/lock state.
    Inspect(InspectArgs),
}

/// Arguments for the inspect subcommand.
#[derive(Args)]
pub struct InspectArgs {
    /// Output as JSON.
    #[arg(long)]
    pub json: bool,

    /// Custom socket path (defaults to /tmp/keyrx-daemon.sock).
    #[arg(long)]
    pub socket: Option<PathBuf>,
}

/// JSON output structure for state.
#[derive(Serialize)]
struct StateOutput {
    /// 255-bit state array (true = active, false = inactive)
    state: Vec<bool>,
    /// Number of active bits
    active_count: usize,
}

/// Execute the state command.
pub fn execute(args: StateArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        StateCommand::Inspect(inspect_args) => execute_inspect(inspect_args),
    }
}

/// Execute the inspect subcommand.
fn execute_inspect(args: InspectArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine socket path
    let socket_path = args
        .socket
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));

    // Create IPC client
    let mut ipc = UnixSocketIpc::new(socket_path);

    // Send GetState request
    let response = ipc.send_request(&IpcRequest::GetState)?;

    // Parse response
    match response {
        IpcResponse::State { state } => {
            if args.json {
                print_json_output(&state)?;
            } else {
                print_human_output(&state);
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
fn print_json_output(state: &[bool]) -> Result<(), Box<dyn std::error::Error>> {
    let active_count = state.iter().filter(|&&b| b).count();
    let output = StateOutput {
        state: state.to_vec(),
        active_count,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print human-readable output.
fn print_human_output(state: &[bool]) {
    println!("Runtime State (255-bit modifier/lock state):");
    println!();

    let active_count = state.iter().filter(|&&b| b).count();
    println!("  Active bits: {}/255", active_count);
    println!();

    if active_count == 0 {
        println!("  (No modifiers or locks currently active)");
    } else {
        println!("  Active bit indices:");
        for (idx, &bit) in state.iter().enumerate() {
            if bit {
                println!("    - Bit {}", idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_output_format() {
        let mut state = vec![false; 255];
        state[0] = true;
        state[10] = true;
        state[100] = true;

        let output = StateOutput {
            state: state.clone(),
            active_count: 3,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"active_count\":3"));
        assert!(json.contains("\"state\""));
    }

    #[test]
    fn test_state_output_empty() {
        let state = vec![false; 255];
        let output = StateOutput {
            state: state.clone(),
            active_count: 0,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"active_count\":0"));
    }

    #[test]
    fn test_state_output_all_active() {
        let state = vec![true; 255];
        let output = StateOutput {
            state: state.clone(),
            active_count: 255,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"active_count\":255"));
    }
}
