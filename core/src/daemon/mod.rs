//! KeyRx daemon for background service mode.
//!
//! This module implements a daemon that runs as a system service, handling
//! input capture and remapping independently of the Flutter UI. The daemon
//! exposes an IPC interface (Unix socket) with JSON-RPC 2.0 protocol for
//! control and monitoring.
//!
//! # Architecture
//!
//! ```text
//! Flutter App (User)  ←─ IPC ─→  keyrx daemon (Root/Elevated)
//!   │                                │
//!   ├─ Start/Stop                    ├─ Engine Manager
//!   ├─ Reload config                 ├─ Multi-device Input
//!   ├─ Get status                    ├─ Event Broadcasting
//!   └─ Subscribe events              └─ IPC Server
//! ```

pub mod ipc;
pub mod manager;
pub mod protocol;

pub use ipc::IpcServer;
pub use manager::EngineManager;
pub use protocol::{JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

/// Daemon configuration.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// IPC socket path.
    pub socket_path: PathBuf,
    /// PID file path.
    pub pid_file: Option<PathBuf>,
    /// Default config path.
    pub config_path: Option<PathBuf>,
    /// Default script path.
    pub script_path: Option<PathBuf>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
            pid_file: Some(default_pid_file()),
            config_path: None,
            script_path: None,
        }
    }
}

/// Get default socket path based on platform.
///
/// - Linux/macOS: `$XDG_RUNTIME_DIR/keyrx/daemon.sock` or `/tmp/keyrx/daemon.sock`
/// - Windows: Not implemented (would use named pipe)
fn default_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("keyrx").join("daemon.sock")
        } else {
            PathBuf::from("/tmp/keyrx/daemon.sock")
        }
    }
    #[cfg(not(unix))]
    {
        panic!("Windows named pipes not yet implemented");
    }
}

/// Get default PID file path.
fn default_pid_file() -> PathBuf {
    #[cfg(unix)]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("keyrx").join("daemon.pid")
        } else {
            PathBuf::from("/tmp/keyrx/daemon.pid")
        }
    }
    #[cfg(not(unix))]
    {
        PathBuf::from("C:\\ProgramData\\keyrx\\daemon.pid")
    }
}

/// Main daemon entry point.
pub async fn run_daemon(config: DaemonConfig) -> Result<()> {
    info!(
        service = "keyrx",
        event = "daemon_start",
        socket = %config.socket_path.display(),
        "Starting KeyRx daemon"
    );

    // Create socket directory if it doesn't exist
    if let Some(parent) = config.socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write PID file
    if let Some(ref pid_file) = config.pid_file {
        if let Some(parent) = pid_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(pid_file, std::process::id().to_string())?;
        info!(
            service = "keyrx",
            event = "pid_file_written",
            path = %pid_file.display(),
            pid = std::process::id(),
            "PID file written"
        );
    }

    // Create engine manager
    let manager = EngineManager::new();

    // Start IPC server
    let mut server = IpcServer::new(config.socket_path.clone(), manager);

    // Set up signal handlers for graceful shutdown
    let shutdown_result = tokio::select! {
        result = server.run() => {
            error!("IPC server exited: {:?}", result);
            result
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT, shutting down gracefully");
            Ok(())
        }
    };

    // Cleanup
    if let Some(ref pid_file) = config.pid_file {
        let _ = std::fs::remove_file(pid_file);
    }
    let _ = std::fs::remove_file(&config.socket_path);

    info!(
        service = "keyrx",
        event = "daemon_stop",
        "KeyRx daemon stopped"
    );

    shutdown_result
}
