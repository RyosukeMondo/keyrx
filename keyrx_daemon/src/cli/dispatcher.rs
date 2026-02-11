//! CLI command dispatcher for routing commands to handlers.
//!
//! This module handles the routing of CLI commands to their respective handlers,
//! following the Command pattern for extensibility.

use crate::cli::{
    config, devices, layers, layouts, metrics, profiles, simulate, state, status, test,
};
use std::path::PathBuf;

/// Exit codes following Unix conventions.
pub mod exit_codes {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// Configuration error (file not found, parse error).
    pub const CONFIG_ERROR: i32 = 1;
    /// Permission error (cannot access devices, cannot create uinput).
    pub const PERMISSION_ERROR: i32 = 2;
    /// Runtime error (device disconnected with no fallback).
    pub const RUNTIME_ERROR: i32 = 3;
}

/// Available CLI commands.
#[derive(Debug)]
pub enum Command {
    Run {
        config: Option<PathBuf>,
        debug: bool,
        test_mode: bool,
    },
    Devices(devices::DevicesArgs),
    Profiles(profiles::ProfilesArgs),
    Config(config::ConfigArgs),
    Layers(layers::LayersArgs),
    Layouts(layouts::LayoutsArgs),
    Simulate(simulate::SimulateArgs),
    Test(test::TestArgs),
    Status(status::StatusArgs),
    State(state::StateArgs),
    Metrics(metrics::MetricsArgs),
    ListDevices,
    Validate {
        config: PathBuf,
    },
    Record {
        output: PathBuf,
        device: Option<PathBuf>,
    },
}

/// Result type for command execution.
pub type CommandResult = Result<(), (i32, String)>;

/// Dispatch a command to its appropriate handler.
///
/// # Arguments
///
/// * `command` - The command to execute
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn dispatch(command: Command) -> CommandResult {
    match command {
        Command::Run {
            config,
            debug,
            test_mode,
        } => {
            // Delegate to run handler (defined in handlers/run.rs)
            crate::cli::handlers::run::handle_run(config, debug, test_mode)
        }
        Command::Devices(args) => {
            devices::execute(args, None).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Profiles(args) => crate::cli::handlers::profiles::handle_profiles(args),
        Command::Config(args) => {
            config::execute(args, None).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Layers(args) => {
            layers::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Layouts(args) => {
            layouts::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Simulate(args) => {
            simulate::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Test(args) => {
            test::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Status(args) => {
            status::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::State(args) => {
            state::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::Metrics(args) => {
            metrics::execute(args).map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
        }
        Command::ListDevices => crate::cli::handlers::list_devices::handle_list_devices(),
        Command::Validate { config } => crate::cli::handlers::validate::handle_validate(&config),
        Command::Record { output, device } => {
            crate::cli::handlers::record::handle_record(&output, device.as_deref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::CONFIG_ERROR, 1);
        assert_eq!(exit_codes::PERMISSION_ERROR, 2);
        assert_eq!(exit_codes::RUNTIME_ERROR, 3);
    }
}
