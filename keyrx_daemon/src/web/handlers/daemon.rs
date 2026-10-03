//! Daemon control RPC method handlers.
//!
//! This module implements daemon control operations like restart and status.

use serde::Serialize;
use serde_json::Value;
use typeshare::typeshare;

use crate::web::rpc_types::RpcError;

/// Result of restart_daemon command
#[typeshare]
#[derive(Debug, Serialize)]
pub struct RestartResult {
    pub success: bool,
    pub message: String,
}

/// Restart the daemon process.
///
/// Under systemd (`INVOCATION_ID` set) the process exits with a non-zero code so
/// `Restart=on-failure` brings the unit back; spawning a copy there would leave
/// the unit "inactive (success)" and the keyboard unremapped. Otherwise the
/// process image is replaced with `exec()` (Unix) or a copy is spawned (Windows).
/// The WebSocket connection will be lost and the client should reconnect.
pub async fn restart_daemon(_params: Value) -> Result<Value, RpcError> {
    log::info!("Daemon restart requested via RPC");

    // Spawn a thread to perform the restart after a brief delay
    // This allows the RPC response to be sent before the process restarts
    std::thread::spawn(|| {
        // Brief delay to allow response to be sent
        std::thread::sleep(std::time::Duration::from_millis(100));
        perform_restart();
    });

    let result = RestartResult {
        success: true,
        message: "Daemon restart initiated. Reconnect in a moment.".to_string(),
    };

    serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
}

/// Exit code used to ask a supervisor for a restart (EX_TEMPFAIL).
const RESTART_EXIT_CODE: i32 = 75;

/// How a restart request is carried out.
#[derive(Debug, PartialEq, Eq)]
enum RestartStrategy {
    /// Exit non-zero and let the supervisor (systemd) restart us.
    ExitForSupervisor(i32),
    /// Replace/respawn the process ourselves.
    Reexec,
}

/// Pure decision: are we supervised by systemd?
fn restart_strategy(invocation_id: Option<&str>) -> RestartStrategy {
    match invocation_id {
        Some(id) if !id.is_empty() => RestartStrategy::ExitForSupervisor(RESTART_EXIT_CODE),
        _ => RestartStrategy::Reexec,
    }
}

/// Perform the actual process restart
fn perform_restart() {
    log::info!("Performing daemon restart...");

    let invocation = std::env::var("INVOCATION_ID").ok();
    if let RestartStrategy::ExitForSupervisor(code) = restart_strategy(invocation.as_deref()) {
        log::info!("Supervised by systemd; exiting with {code} so the unit restarts");
        std::process::exit(code);
    }

    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to get current executable path: {}", e);
            return;
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();

    log::info!("Restarting with: {:?} {:?}", exe, args);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // exec() only returns on failure.
        let err = std::process::Command::new(&exe).args(&args).exec();
        log::error!("Failed to exec new daemon process: {}", err);
    }

    #[cfg(not(unix))]
    match std::process::Command::new(&exe).args(&args).spawn() {
        Ok(_) => {
            log::info!("New daemon process spawned, exiting current process");
            std::process::exit(0);
        }
        Err(e) => {
            log::error!("Failed to spawn new daemon process: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_invocation_exits_nonzero_for_restart() {
        assert_eq!(
            restart_strategy(Some("abc123")),
            RestartStrategy::ExitForSupervisor(75)
        );
    }

    #[test]
    fn unsupervised_or_empty_invocation_reexecs() {
        assert_eq!(restart_strategy(None), RestartStrategy::Reexec);
        assert_eq!(restart_strategy(Some("")), RestartStrategy::Reexec);
    }
}
