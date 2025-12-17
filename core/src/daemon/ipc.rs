//! IPC server implementation using Unix domain sockets.

use crate::daemon::manager::{EngineEvent, EngineManager};
use crate::daemon::protocol::*;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// IPC server that handles client connections and JSON-RPC requests.
pub struct IpcServer {
    socket_path: PathBuf,
    manager: Arc<EngineManager>,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf, manager: EngineManager) -> Self {
        Self {
            socket_path,
            manager: Arc::new(manager),
        }
    }

    /// Run the IPC server (blocking).
    pub async fn run(&mut self) -> Result<()> {
        // Remove existing socket file if it exists
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path)?;

        info!(
            service = "keyrx",
            event = "ipc_server_start",
            socket = %self.socket_path.display(),
            "IPC server listening"
        );

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let manager = self.manager.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, manager).await {
                            error!("Client handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

/// Handle a single client connection.
async fn handle_client(stream: UnixStream, manager: Arc<EngineManager>) -> Result<()> {
    info!(
        service = "keyrx",
        event = "client_connected",
        "Client connected"
    );

    let (reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));
    let mut reader = BufReader::new(reader);

    // Spawn event broadcaster task
    let writer_clone = writer.clone();
    let mut event_rx = manager.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if let Some(notification) = event_to_notification(event) {
                let _ =
                    send_message(&writer_clone, OutgoingMessage::Notification(notification)).await;
            }
        }
    });

    // Handle incoming requests
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                debug!(
                    service = "keyrx",
                    event = "request_received",
                    request = %line,
                    "Received request"
                );

                match serde_json::from_str::<IncomingMessage>(line) {
                    Ok(IncomingMessage::Request(request)) => {
                        let response = handle_request(request, &manager).await;
                        if let Err(e) = send_message(&writer, response).await {
                            error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Ok(IncomingMessage::Notification(_notification)) => {
                        // Clients shouldn't send notifications, but we'll ignore them
                        debug!("Ignoring notification from client");
                    }
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        let error_response = JsonRpcError::parse_error(RequestId::Number(0));
                        let _ = send_message(&writer, OutgoingMessage::Error(error_response)).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to read from client: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Handle a JSON-RPC request and return a response.
async fn handle_request(request: JsonRpcRequest, manager: &EngineManager) -> OutgoingMessage {
    let id = request.id.clone();

    match request.method.as_str() {
        METHOD_ENGINE_START => match parse_params::<EngineStartParams>(request.params) {
            Ok(params) => {
                let config_path = params.config_path.map(PathBuf::from);
                let script_path = params.script_path.map(PathBuf::from);

                match manager
                    .start(config_path, script_path, params.use_all_devices)
                    .await
                {
                    Ok(devices) => {
                        let result = json!({
                            "status": "started",
                            "devices": devices,
                        });
                        OutgoingMessage::Response(JsonRpcResponse::success(id, result))
                    }
                    Err(e) => OutgoingMessage::Error(JsonRpcError::internal_error(
                        id,
                        format!("Failed to start engine: {}", e),
                    )),
                }
            }
            Err(e) => OutgoingMessage::Error(JsonRpcError::invalid_params(id, e)),
        },

        METHOD_ENGINE_STOP => match manager.stop().await {
            Ok(_) => {
                let result = json!({
                    "status": "stopped",
                });
                OutgoingMessage::Response(JsonRpcResponse::success(id, result))
            }
            Err(e) => OutgoingMessage::Error(JsonRpcError::internal_error(
                id,
                format!("Failed to stop engine: {}", e),
            )),
        },

        METHOD_CONFIG_RELOAD => match parse_params::<ConfigReloadParams>(request.params) {
            Ok(params) => {
                let config_path = params.config_path.map(PathBuf::from);
                let script_path = params.script_path.map(PathBuf::from);

                match manager
                    .reload(config_path.clone(), script_path.clone())
                    .await
                {
                    Ok(_) => {
                        let result = json!({
                            "status": "reloaded",
                            "loaded_config": config_path.map(|p| p.display().to_string()),
                            "loaded_script": script_path.map(|p| p.display().to_string()),
                        });
                        OutgoingMessage::Response(JsonRpcResponse::success(id, result))
                    }
                    Err(e) => OutgoingMessage::Error(JsonRpcError::internal_error(
                        id,
                        format!("Failed to reload: {}", e),
                    )),
                }
            }
            Err(e) => OutgoingMessage::Error(JsonRpcError::invalid_params(id, e)),
        },

        METHOD_ENGINE_STATUS => {
            let status = manager.status().await;
            let result = serde_json::to_value(status).unwrap_or(json!(null));
            OutgoingMessage::Response(JsonRpcResponse::success(id, result))
        }

        METHOD_EVENTS_SUBSCRIBE => {
            // Subscription is automatic via the event broadcaster task
            let result = json!({
                "subscribed": true,
            });
            OutgoingMessage::Response(JsonRpcResponse::success(id, result))
        }

        METHOD_EVENTS_UNSUBSCRIBE => {
            // TODO: Implement unsubscribe by stopping the broadcaster task
            let result = json!({
                "subscribed": false,
            });
            OutgoingMessage::Response(JsonRpcResponse::success(id, result))
        }

        _ => OutgoingMessage::Error(JsonRpcError::method_not_found(id, &request.method)),
    }
}

/// Send a message to the client.
async fn send_message(
    writer: &Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    message: OutgoingMessage,
) -> Result<()> {
    let json = serde_json::to_string(&message)?;
    let mut writer = writer.lock().await;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// Parse request parameters.
fn parse_params<T: serde::de::DeserializeOwned>(params: Option<Value>) -> Result<T, String> {
    match params {
        Some(value) => serde_json::from_value(value).map_err(|e| format!("Invalid params: {}", e)),
        None => {
            // Try to deserialize from empty object (allows default values)
            serde_json::from_value(json!({})).map_err(|e| format!("Missing required params: {}", e))
        }
    }
}

/// Convert engine event to JSON-RPC notification.
fn event_to_notification(event: EngineEvent) -> Option<JsonRpcNotification> {
    match event {
        EngineEvent::Started => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_STATE_CHANGE.to_string(),
            json!({
                "type": "engine_started",
                "timestamp_us": current_timestamp_us(),
                "message": "Engine started",
            }),
        )),
        EngineEvent::Stopped => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_STATE_CHANGE.to_string(),
            json!({
                "type": "engine_stopped",
                "timestamp_us": current_timestamp_us(),
                "message": "Engine stopped",
            }),
        )),
        EngineEvent::ConfigReloaded(path) => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_STATE_CHANGE.to_string(),
            json!({
                "type": "config_reloaded",
                "timestamp_us": current_timestamp_us(),
                "message": format!("Configuration reloaded: {}", path.display()),
                "details": {
                    "config_path": path.display().to_string(),
                },
            }),
        )),
        EngineEvent::ScriptReloaded(path) => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_STATE_CHANGE.to_string(),
            json!({
                "type": "script_reloaded",
                "timestamp_us": current_timestamp_us(),
                "message": format!("Script reloaded: {}", path.display()),
                "details": {
                    "script_path": path.display().to_string(),
                },
            }),
        )),
        EngineEvent::InputEvent {
            timestamp_us,
            device_path,
            device_name,
            key_code,
            key_name,
            is_press,
        } => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_INPUT.to_string(),
            json!({
                "timestamp_us": timestamp_us,
                "device_path": device_path,
                "device_name": device_name,
                "event_type": if is_press { "key_press" } else { "key_release" },
                "key_code": key_code,
                "key_name": key_name,
                "modifiers": [],
            }),
        )),
        EngineEvent::OutputEvent {
            timestamp_us,
            key_code,
            key_name,
            is_press,
        } => Some(JsonRpcNotification::new(
            NOTIFY_EVENT_OUTPUT.to_string(),
            json!({
                "timestamp_us": timestamp_us,
                "action_type": if is_press { "key_press" } else { "key_release" },
                "key_code": key_code,
                "key_name": key_name,
            }),
        )),
    }
}

/// Get current timestamp in microseconds.
fn current_timestamp_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}
