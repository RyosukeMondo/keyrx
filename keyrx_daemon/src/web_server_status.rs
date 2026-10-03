//! Whether the daemon's web server (REST, WebSocket, UI) is serving.
//!
//! A web-server failure (e.g. the port is taken) is deliberately non-fatal:
//! keyboard remapping keeps working. It must not be silent either, so the
//! outcome is published through `DaemonSharedState` and reported by
//! `DaemonQueryService::get_status` on every transport.

use serde::{Deserialize, Serialize};

/// State of the web server, as reported in daemon status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum WebServerStatus {
    /// Not bound yet (or this process runs no web server).
    #[default]
    Starting,
    /// Bound and serving.
    Up,
    /// Could not bind or stopped with an error.
    Failed {
        /// Human-readable cause, including the address.
        error: String,
    },
}

impl WebServerStatus {
    /// One-line description for human-readable output.
    pub fn describe(&self) -> String {
        match self {
            Self::Starting => "starting".to_string(),
            Self::Up => "up".to_string(),
            Self::Failed { error } => format!("FAILED - {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format_is_tagged_and_round_trips() {
        let failed = WebServerStatus::Failed {
            error: "address in use".into(),
        };
        let json = serde_json::to_string(&failed).unwrap();
        assert_eq!(json, r#"{"state":"failed","error":"address in use"}"#);
        assert_eq!(
            serde_json::from_str::<WebServerStatus>(&json).unwrap(),
            failed
        );
        assert_eq!(
            serde_json::to_string(&WebServerStatus::Up).unwrap(),
            r#"{"state":"up"}"#
        );
    }

    #[test]
    fn failure_is_visible_in_the_description() {
        let s = WebServerStatus::Failed {
            error: "boom".into(),
        };
        assert!(s.describe().contains("FAILED") && s.describe().contains("boom"));
    }
}
