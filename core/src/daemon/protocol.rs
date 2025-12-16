//! JSON-RPC 2.0 protocol types for daemon IPC.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    pub id: RequestId,
}

/// JSON-RPC 2.0 response (success).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Value,
    pub id: RequestId,
}

/// JSON-RPC 2.0 error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub error: ErrorObject,
    pub id: RequestId,
}

/// Error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (no ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

/// Request ID (can be string, number, or null).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

/// Standard JSON-RPC error codes.
#[allow(dead_code)]
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // Custom error codes
    pub const ENGINE_NOT_RUNNING: i32 = -32000;
    pub const DEVICE_NOT_FOUND: i32 = -32001;
    pub const CONFIG_LOAD_FAILED: i32 = -32002;
    pub const SCRIPT_COMPILE_FAILED: i32 = -32003;
    pub const PERMISSION_DENIED: i32 = -32004;
}

impl JsonRpcResponse {
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result,
            id,
        }
    }
}

impl JsonRpcError {
    pub fn new(id: RequestId, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            error: ErrorObject {
                code,
                message,
                data,
            },
            id,
        }
    }

    pub fn parse_error(id: RequestId) -> Self {
        Self::new(
            id,
            error_codes::PARSE_ERROR,
            "Parse error".to_string(),
            None,
        )
    }

    pub fn invalid_request(id: RequestId) -> Self {
        Self::new(
            id,
            error_codes::INVALID_REQUEST,
            "Invalid request".to_string(),
            None,
        )
    }

    pub fn method_not_found(id: RequestId, method: &str) -> Self {
        Self::new(
            id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", method),
            None,
        )
    }

    pub fn invalid_params(id: RequestId, message: String) -> Self {
        Self::new(id, error_codes::INVALID_PARAMS, message, None)
    }

    pub fn internal_error(id: RequestId, message: String) -> Self {
        Self::new(id, error_codes::INTERNAL_ERROR, message, None)
    }

    pub fn engine_not_running(id: RequestId) -> Self {
        Self::new(
            id,
            error_codes::ENGINE_NOT_RUNNING,
            "Engine is not running".to_string(),
            None,
        )
    }
}

impl JsonRpcNotification {
    pub fn new(method: String, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

/// Incoming message (can be request or notification).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IncomingMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
}

/// Outgoing message (can be response, error, or notification).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutgoingMessage {
    Response(JsonRpcResponse),
    Error(JsonRpcError),
    Notification(JsonRpcNotification),
}

// Method names
pub const METHOD_ENGINE_START: &str = "engine.start";
pub const METHOD_ENGINE_STOP: &str = "engine.stop";
pub const METHOD_ENGINE_RELOAD: &str = "engine.reload";
pub const METHOD_ENGINE_STATUS: &str = "engine.status";
pub const METHOD_EVENTS_SUBSCRIBE: &str = "events.subscribe";
pub const METHOD_EVENTS_UNSUBSCRIBE: &str = "events.unsubscribe";

// Notification method names
pub const NOTIFY_EVENT_INPUT: &str = "event.input";
pub const NOTIFY_EVENT_OUTPUT: &str = "event.output";
pub const NOTIFY_EVENT_STATE_CHANGE: &str = "event.state_change";

/// Parameters for engine.start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStartParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    #[serde(default = "default_true")]
    pub use_all_devices: bool,
}

fn default_true() -> bool {
    true
}

/// Parameters for engine.reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReloadParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
}

/// Parameters for events.subscribe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsSubscribeParams {
    #[serde(default = "default_event_types")]
    pub event_types: Vec<String>,
}

fn default_event_types() -> Vec<String> {
    vec![
        "input".to_string(),
        "output".to_string(),
        "state_change".to_string(),
    ]
}

/// Device info in status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
}

/// Engine status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub status: String, // "running", "stopped", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_path: Option<String>,
    #[serde(default)]
    pub devices: Vec<DeviceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_processed: Option<u64>,
}

/// Input event notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEventParams {
    pub timestamp_us: u64,
    pub device_path: String,
    pub device_name: String,
    pub event_type: String, // "key_press", "key_release"
    pub key_code: u16,
    pub key_name: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

/// Output event notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEventParams {
    pub timestamp_us: u64,
    pub action_type: String, // "key_press", "key_release", "tap"
    pub key_code: u16,
    pub key_name: String,
}

/// State change notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeParams {
    pub r#type: String, // "config_reloaded", "script_reloaded", "engine_started", "engine_stopped"
    pub timestamp_us: u64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
