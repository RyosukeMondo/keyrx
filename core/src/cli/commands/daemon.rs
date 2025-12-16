//! Daemon command for running KeyRx as a background service.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode};
use crate::daemon::{run_daemon, DaemonConfig};
use std::path::PathBuf;

/// Run KeyRx as a background daemon service.
pub struct DaemonCommand {
    /// IPC socket path.
    pub socket_path: Option<PathBuf>,
    /// PID file path.
    pub pid_file: Option<PathBuf>,
    /// Default config path.
    pub config_path: Option<PathBuf>,
    /// Default script path.
    pub script_path: Option<PathBuf>,
}

impl DaemonCommand {
    pub fn new(
        socket_path: Option<PathBuf>,
        pid_file: Option<PathBuf>,
        config_path: Option<PathBuf>,
        script_path: Option<PathBuf>,
    ) -> Self {
        Self {
            socket_path,
            pid_file,
            config_path,
            script_path,
        }
    }
}

impl Command for DaemonCommand {
    fn name(&self) -> &str {
        "daemon"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        let mut config = DaemonConfig::default();

        if let Some(ref socket_path) = self.socket_path {
            config.socket_path = socket_path.clone();
        }

        if let Some(ref pid_file) = self.pid_file {
            config.pid_file = Some(pid_file.clone());
        }

        config.config_path = self.config_path.clone();
        config.script_path = self.script_path.clone();

        // Run the daemon (blocking)
        let result = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(run_daemon(config))
        });

        match result {
            Ok(()) => CommandResult::success(()),
            Err(err) => {
                CommandResult::failure(ExitCode::GeneralError, format!("Daemon failed: {:#}", err))
            }
        }
    }
}
