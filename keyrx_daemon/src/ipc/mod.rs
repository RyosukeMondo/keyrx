//! Inter-Process Communication (IPC) infrastructure for CLI-daemon communication.
//!
//! The daemon serves newline-delimited JSON requests for status, state and
//! metrics on an [`IpcEndpoint`]: a per-user Unix socket file (`$XDG_RUNTIME_DIR/keyrx-daemon.sock`)
//! or a Windows named pipe (`\\.\pipe\keyrx-daemon`). [`client::IpcClient`]
//! is the CLI side, [`server::spawn`] the daemon side.

use serde::{Deserialize, Serialize};

use crate::web::events::{KeyEventData, LatencyStats};
use std::time::Duration;
use thiserror::Error;

pub mod client;
pub mod commands;
mod endpoint;
pub mod server;

pub use endpoint::IpcEndpoint;

/// Default timeout for IPC requests (5 seconds)
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// IPC request types sent from CLI to daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcRequest {
    /// Get daemon status (running, uptime, active profile, device count)
    GetStatus,
    /// Get current modifier/lock state (255-bit state array)
    GetState,
    /// Get latency metrics (min, avg, max, p95, p99)
    GetLatencyMetrics,
    /// Get tail of recent events (last N events)
    GetEventsTail { count: usize },
    /// Clear the recent-events ring buffer
    ClearEvents,
    /// Activate a profile by name (test mode only)
    ActivateProfile { name: String },
}

/// IPC response types sent from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcResponse {
    /// Daemon status information
    Status {
        running: bool,
        uptime_secs: u64,
        active_profile: Option<String>,
        device_count: usize,
    },
    /// Current state (255-bit modifier/lock state)
    State { state: Vec<bool> },
    /// Latency metrics in microseconds
    Latency { stats: LatencyStats },
    /// Recent key events, oldest first (same records as REST and the WS feed)
    Events { events: Vec<KeyEventData> },
    /// Result of clearing the recent-events ring buffer (number removed)
    EventsCleared { count: usize },
    /// The profile was compiled and the daemon asked to switch to it
    ProfileActivated { name: String },
    /// Error response
    Error { code: u16, message: String },
}

/// IPC error types
#[derive(Debug, Error)]
pub enum IpcError {
    /// No IPC endpoint (socket file or named pipe): the daemon is not running
    #[error("Daemon not running: no IPC endpoint at {0} (error code 3005)")]
    SocketNotFound(String),

    /// Socket file exists but nothing listens on it: the daemon exited without
    /// removing it (crash, kill). Same meaning as `SocketNotFound`.
    #[error("Daemon not running: stale socket at {0} (error code 3005)")]
    StaleSocket(String),

    /// Request timeout (daemon not responding)
    #[error("Request timeout after {0:?} (error code 3007)")]
    Timeout(Duration),

    /// Deserialization error (invalid response format)
    #[error("Failed to deserialize response: {0} (error code 1009)")]
    DeserializeError(String),

    /// Serialization error (invalid request format)
    #[error("Failed to serialize request: {0}")]
    SerializeError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl IpcError {
    /// Get the error code for this error
    pub fn code(&self) -> u16 {
        match self {
            IpcError::SocketNotFound(_) | IpcError::StaleSocket(_) => 3005,
            IpcError::Timeout(_) => 3007,
            IpcError::DeserializeError(_) => 1009,
            IpcError::SerializeError(_) => 1008,
            IpcError::IoError(_) => 3001,
        }
    }
}

/// Trait for daemon IPC communication
pub trait DaemonIpc {
    /// Send a request to the daemon and receive a response
    fn send_request(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError>;

    /// Receive a response from the daemon (used by server-side)
    fn receive_response(&mut self) -> Result<IpcResponse, IpcError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_request_serialization() {
        let req = IpcRequest::GetStatus;
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: IpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_ipc_request_get_events_tail() {
        let req = IpcRequest::GetEventsTail { count: 100 };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("get_events_tail"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_ipc_response_status_serialization() {
        let resp = IpcResponse::Status {
            running: true,
            uptime_secs: 3600,
            active_profile: Some("default".to_string()),
            device_count: 2,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: IpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_ipc_response_error_serialization() {
        let resp = IpcResponse::Error {
            code: 3005,
            message: "Socket not found".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("3005"));
        assert!(json.contains("Socket not found"));
    }

    #[test]
    fn test_ipc_error_codes() {
        assert_eq!(
            IpcError::SocketNotFound("/tmp/test.sock".to_string()).code(),
            3005
        );
        assert_eq!(IpcError::Timeout(Duration::from_secs(5)).code(), 3007);
        assert_eq!(IpcError::DeserializeError("test".to_string()).code(), 1009);
    }
}
