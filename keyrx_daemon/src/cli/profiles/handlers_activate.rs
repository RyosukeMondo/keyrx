//! The `activate` subcommand handler.

use super::output::ActivationOutput;
use crate::cli::common::output_error;
use crate::cli::logging;
use crate::config::profile_manager::ProfileError;
use crate::error::{CliError, DaemonResult};
use crate::services::ProfileService;

/// Asks a running daemon to switch to `name` — the same activation request
/// REST, MCP and WS-RPC make in-process. Without a daemon, the persisted
/// `.active` profile takes effect when it starts.
fn notify_running_daemon(name: &str) -> String {
    use crate::ipc::unix_socket::UnixSocketIpc;
    use crate::ipc::{DaemonIpc, IpcError, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};

    let mut ipc = UnixSocketIpc::new(std::path::PathBuf::from(DEFAULT_SOCKET_PATH));
    let request = IpcRequest::ActivateProfile {
        name: name.to_string(),
    };
    match ipc.send_request(&request) {
        Ok(IpcResponse::ProfileActivated { .. }) => "running daemon switched".to_string(),
        Ok(IpcResponse::Error { message, .. }) => format!("running daemon refused: {message}"),
        Ok(other) => format!("unexpected daemon response: {other:?}"),
        Err(IpcError::SocketNotFound(_) | IpcError::StaleSocket(_)) => {
            "not running; applies on next start".to_string()
        }
        Err(e) => format!("could not reach daemon: {e}"),
    }
}

/// Handle the `activate` subcommand.
pub(super) async fn handle_activate(
    service: &ProfileService,
    name: &str,
    json: bool,
) -> DaemonResult<()> {
    logging::log_command_start("profiles activate", name);

    match service.activate_profile(name).await {
        Ok(result) => {
            logging::log_profile_activate(name, result.success);
            let daemon = result.success.then(|| notify_running_daemon(name));
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
                    daemon,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).map_err(CliError::from)?
                );
            } else if result.success {
                println!("✓ Profile '{}' activated", name);
                println!("  Compile time: {}ms", result.compile_time_ms);
                println!("  Reload time: {}ms", result.reload_time_ms);
                println!(
                    "  Total: {}ms",
                    result.compile_time_ms + result.reload_time_ms
                );
                if let Some(daemon) = daemon {
                    println!("  Daemon: {daemon}");
                }
            } else {
                eprintln!("✗ Activation failed");
                if let Some(ref error) = result.error {
                    eprintln!("  Error: {}", error);
                }
                eprintln!("  Compile time: {}ms", result.compile_time_ms);
                return Err(CliError::CommandFailed {
                    command: "activate".to_string(),
                    reason: result
                        .error
                        .unwrap_or_else(|| "Unknown activation error".to_string()),
                }
                .into());
            }

            if result.success {
                Ok(())
            } else {
                Err(CliError::CommandFailed {
                    command: "profiles".to_string(),
                    reason: "Command failed".to_string(),
                }
                .into())
            }
        }
        Err(ProfileError::NotFound(name)) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Profile '{}' not found", name),
            );
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(CliError::CommandFailed {
                command: "profiles".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(ProfileError::Compilation(e)) => {
            logging::log_command_error("profiles activate", &format!("Compilation error: {}", e));
            output_error(&format!("Compilation error: {}", e), 2004, json);
            Err(CliError::CommandFailed {
                command: "profiles".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
        Err(e) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Failed to activate profile: {}", e),
            );
            output_error(&format!("Failed to activate profile: {}", e), 2001, json);
            Err(CliError::CommandFailed {
                command: "profiles".to_string(),
                reason: "Command failed".to_string(),
            }
            .into())
        }
    }
}
