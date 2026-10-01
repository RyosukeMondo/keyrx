//! The `activate` subcommand handler.

use super::output::ActivationOutput;
use crate::cli::common::output_error;
use crate::cli::logging;
use crate::config::profile_manager::ProfileError;
use crate::error::{CliError, DaemonResult};
use crate::ipc::{IpcError, IpcResponse};
use crate::services::ProfileService;

/// What the running daemon (if any) did with an activation request.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum DaemonOutcome {
    /// The daemon swapped to the profile; it is live.
    Switched,
    /// No daemon is listening; the profile applies on the next start.
    NotRunning,
    /// A daemon answered but refused, or could not be reached.
    Failed(String),
}

impl DaemonOutcome {
    fn describe(&self) -> String {
        match self {
            Self::Switched => "running daemon switched".to_string(),
            Self::NotRunning => "not running; applies on next start".to_string(),
            Self::Failed(why) => why.clone(),
        }
    }

    fn failure(&self) -> Option<&str> {
        match self {
            Self::Failed(why) => Some(why),
            _ => None,
        }
    }
}

/// Maps the daemon's IPC answer to a [`DaemonOutcome`].
pub(super) fn classify_daemon_reply(reply: Result<IpcResponse, IpcError>) -> DaemonOutcome {
    match reply {
        Ok(IpcResponse::ProfileActivated { .. }) => DaemonOutcome::Switched,
        Ok(IpcResponse::Error { message, .. }) => {
            DaemonOutcome::Failed(format!("running daemon refused: {message}"))
        }
        Ok(other) => DaemonOutcome::Failed(format!("unexpected daemon response: {other:?}")),
        Err(IpcError::SocketNotFound(_) | IpcError::StaleSocket(_)) => DaemonOutcome::NotRunning,
        Err(e) => DaemonOutcome::Failed(format!("could not reach daemon: {e}")),
    }
}

/// Asks a running daemon to switch to `name` - the same activation request
/// REST, MCP and WS-RPC make in-process. Without a daemon, the persisted
/// `.active` profile takes effect when it starts.
fn notify_running_daemon(name: &str) -> DaemonOutcome {
    use crate::ipc::client::IpcClient;
    use crate::ipc::{DaemonIpc, IpcEndpoint, IpcRequest};

    let mut ipc = IpcClient::new(IpcEndpoint::default_for_platform());
    classify_daemon_reply(ipc.send_request(&IpcRequest::ActivateProfile {
        name: name.to_string(),
    }))
}

/// Prints the outcome of an activation (text or JSON).
fn report(
    name: &str,
    result: &crate::config::ActivationResult,
    daemon: Option<&DaemonOutcome>,
    error: Option<String>,
    ok: bool,
    json: bool,
) -> DaemonResult<()> {
    if json {
        let output = ActivationOutput {
            success: ok,
            compile_time_ms: result.compile_time_ms,
            reload_time_ms: result.reload_time_ms,
            error,
            daemon: daemon.map(DaemonOutcome::describe),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(CliError::from)?
        );
    } else if ok {
        println!("✓ Profile '{}' activated", name);
        println!("  Compile time: {}ms", result.compile_time_ms);
        println!("  Reload time: {}ms", result.reload_time_ms);
        println!(
            "  Total: {}ms",
            result.compile_time_ms + result.reload_time_ms
        );
        if let Some(daemon) = daemon {
            println!("  Daemon: {}", daemon.describe());
        }
    } else {
        eprintln!("✗ Activation failed");
        if let Some(error) = &error {
            eprintln!("  Error: {error}");
        }
        if result.success {
            eprintln!("  The profile is saved as active and applies on the next start.");
        }
    }
    Ok(())
}

/// Handle the `activate` subcommand.
///
/// Succeeds only when the profile compiled AND, if a daemon is running, the
/// daemon accepted it: the old "activated" + "daemon refused" + exit 0 combo
/// made a failed switch look like a success. Every failure is printed here,
/// exactly once; the returned [`CliError::Reported`] only carries the exit code.
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
            let daemon_failure = daemon.as_ref().and_then(DaemonOutcome::failure);
            let ok = result.success && daemon_failure.is_none();
            let error = result
                .error
                .clone()
                .or_else(|| daemon_failure.map(String::from));
            if ok {
                logging::log_command_success(
                    "profiles activate",
                    result.compile_time_ms + result.reload_time_ms,
                );
            } else {
                logging::log_command_error(
                    "profiles activate",
                    error.as_deref().unwrap_or("Unknown activation error"),
                );
            }
            report(name, &result, daemon.as_ref(), error, ok, json)?;
            if ok {
                Ok(())
            } else {
                Err(CliError::Reported.into())
            }
        }
        Err(ProfileError::NotFound(name)) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Profile '{}' not found", name),
            );
            output_error(&format!("Profile '{}' not found", name), 1001, json);
            Err(CliError::Reported.into())
        }
        Err(ProfileError::Compilation(e)) => {
            logging::log_command_error("profiles activate", &format!("Compilation error: {}", e));
            output_error(&format!("Compilation error: {}", e), 2004, json);
            Err(CliError::Reported.into())
        }
        Err(e) => {
            logging::log_command_error(
                "profiles activate",
                &format!("Failed to activate profile: {}", e),
            );
            output_error(&format!("Failed to activate profile: {}", e), 2001, json);
            Err(CliError::Reported.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusing_daemon_is_a_failure_not_a_success() {
        let outcome = classify_daemon_reply(Ok(IpcResponse::Error {
            code: 5002,
            message: "Profile not found".into(),
        }));
        assert_eq!(
            outcome.failure(),
            Some("running daemon refused: Profile not found")
        );
    }

    #[test]
    fn a_switching_daemon_and_an_absent_daemon_are_not_failures() {
        let switched =
            classify_daemon_reply(Ok(IpcResponse::ProfileActivated { name: "a".into() }));
        assert_eq!(switched, DaemonOutcome::Switched);
        assert!(switched.failure().is_none());
        let absent = classify_daemon_reply(Err(IpcError::SocketNotFound("x".into())));
        assert_eq!(absent, DaemonOutcome::NotRunning);
        assert!(absent.failure().is_none());
    }

    #[test]
    fn an_unreachable_daemon_is_a_failure() {
        let outcome =
            classify_daemon_reply(Err(IpcError::Timeout(std::time::Duration::from_secs(1))));
        assert!(outcome.failure().is_some());
    }
}
