//! WebSocket RPC handler for bidirectional client-server communication.
//!
//! This module implements the WebSocket RPC protocol for handling queries, commands,
//! and subscriptions from web clients. It uses the message types defined in rpc_types.rs
//! and provides request/response correlation via UUID tracking.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::web::rpc_types::{ClientMessage, RpcError, ServerMessage};
use crate::web::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(websocket_handler))
        .with_state(state)
}

/// WebSocket upgrade handler
async fn websocket_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle WebSocket RPC connection
async fn handle_websocket(mut socket: WebSocket, state: Arc<AppState>) {
    // Generate unique client ID
    let client_id = state.subscription_manager.new_client_id().await;
    log::info!("WebSocket RPC client {} connected", client_id);

    // Send Connected handshake immediately
    let connected = ServerMessage::Connected {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64,
    };

    let connected_json = match serde_json::to_string(&connected) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to serialize Connected message: {}", e);
            return;
        }
    };

    if socket.send(Message::Text(connected_json)).await.is_err() {
        log::warn!("Failed to send Connected handshake");
        return;
    }

    // Queue for outgoing messages
    let outgoing_queue = Arc::new(Mutex::new(VecDeque::<ServerMessage>::new()));

    // Main message processing loop
    loop {
        // Check if there are outgoing messages to send
        let next_message = {
            let mut queue = outgoing_queue.lock().await;
            queue.pop_front()
        };

        if let Some(response) = next_message {
            let response_json = match serde_json::to_string(&response) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("Failed to serialize response: {}", e);
                    continue;
                }
            };

            if socket.send(Message::Text(response_json)).await.is_err() {
                log::info!("WebSocket RPC client disconnected (send failed)");
                break;
            }
        }

        // Process incoming messages
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                log::debug!("Received WebSocket RPC message: {}", text);

                // Parse message
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("Failed to parse message: {}", e);
                        // Send parse error response (can't correlate to request ID)
                        let error_response = ServerMessage::Response {
                            id: String::new(),
                            result: None,
                            error: Some(RpcError::parse_error(format!("Invalid JSON: {}", e))),
                        };
                        let mut queue = outgoing_queue.lock().await;
                        queue.push_back(error_response);
                        continue;
                    }
                };

                // Process message and queue response
                process_client_message(
                    client_msg,
                    client_id,
                    Arc::clone(&outgoing_queue),
                    Arc::clone(&state),
                )
                .await;
            }
            Some(Ok(Message::Close(_))) => {
                log::info!("WebSocket RPC client closed connection");
                break;
            }
            Some(Ok(_)) => {
                // Ignore binary/ping/pong messages
            }
            Some(Err(e)) => {
                log::warn!("WebSocket error: {}", e);
                break;
            }
            None => {
                log::info!("WebSocket RPC client {} disconnected", client_id);
                break;
            }
        }
    }

    // Cleanup: unsubscribe from all channels
    state.subscription_manager.unsubscribe_all(client_id).await;
    log::info!("WebSocket RPC connection {} closed", client_id);
}

/// Process a client message and queue the appropriate response
async fn process_client_message(
    msg: ClientMessage,
    client_id: usize,
    outgoing_queue: Arc<Mutex<VecDeque<ServerMessage>>>,
    state: Arc<AppState>,
) {
    let response = match msg {
        ClientMessage::Query { id, method, params } => {
            handle_query(id, method, params, &state).await
        }
        ClientMessage::Command { id, method, params } => {
            handle_command(id, method, params, &state).await
        }
        ClientMessage::Subscribe { id, channel } => {
            handle_subscribe(id, channel, client_id, &state).await
        }
        ClientMessage::Unsubscribe { id, channel } => {
            handle_unsubscribe(id, channel, client_id, &state).await
        }
    };

    // Queue the response
    let mut queue = outgoing_queue.lock().await;
    queue.push_back(response);
}

/// Handle query request (read-only operations)
async fn handle_query(
    id: String,
    method: String,
    params: Value,
    state: &AppState,
) -> ServerMessage {
    use crate::web::handlers::{config, device, metrics, profile};

    log::debug!("Handling query: {} with params: {}", method, params);

    let result = match method.as_str() {
        "get_profiles" => profile::get_profiles(&state.profile_service, params).await,
        "get_devices" => device::get_devices(&state.device_service, params).await,
        "get_config" => config::get_config(&state.config_service, params).await,
        "get_layers" => config::get_layers(&state.config_service, params).await,
        "get_latency" => metrics::get_latency(&state.macro_recorder, params).await,
        "get_events" => metrics::get_events(&state.macro_recorder, params).await,
        _ => Err(RpcError::method_not_found(&method)),
    };

    match result {
        Ok(data) => ServerMessage::Response {
            id,
            result: Some(data),
            error: None,
        },
        Err(error) => ServerMessage::Response {
            id,
            result: None,
            error: Some(error),
        },
    }
}

/// Handle command request (state-modifying operations)
async fn handle_command(
    id: String,
    method: String,
    params: Value,
    state: &AppState,
) -> ServerMessage {
    use crate::web::handlers::{config, device, metrics, profile};

    log::debug!("Handling command: {} with params: {}", method, params);

    let result = match method.as_str() {
        "create_profile" => profile::create_profile(&state.profile_service, params).await,
        "activate_profile" => profile::activate_profile(&state.profile_service, params).await,
        "delete_profile" => profile::delete_profile(&state.profile_service, params).await,
        "duplicate_profile" => profile::duplicate_profile(&state.profile_service, params).await,
        "rename_profile" => profile::rename_profile(&state.profile_service, params).await,
        "rename_device" => device::rename_device(&state.device_service, params).await,
        "set_scope_device" => device::set_scope_device(&state.device_service, params).await,
        "forget_device" => device::forget_device(&state.device_service, params).await,
        "update_config" => config::update_config(&state.config_service, params).await,
        "set_key_mapping" => config::set_key_mapping(&state.config_service, params).await,
        "delete_key_mapping" => config::delete_key_mapping(&state.config_service, params).await,
        "clear_events" => metrics::clear_events(&state.macro_recorder, params).await,
        "simulate" => metrics::simulate(&state.macro_recorder, params).await,
        "reset_simulator" => metrics::reset_simulator(&state.macro_recorder, params).await,
        _ => Err(RpcError::method_not_found(&method)),
    };

    match result {
        Ok(data) => ServerMessage::Response {
            id,
            result: Some(data),
            error: None,
        },
        Err(error) => ServerMessage::Response {
            id,
            result: None,
            error: Some(error),
        },
    }
}

/// Handle subscription request
async fn handle_subscribe(
    id: String,
    channel: String,
    client_id: usize,
    state: &AppState,
) -> ServerMessage {
    log::debug!("Client {} subscribing to channel: {}", client_id, channel);

    state
        .subscription_manager
        .subscribe(client_id, &channel)
        .await;

    ServerMessage::Response {
        id,
        result: Some(serde_json::json!({
            "subscribed": true,
            "channel": channel
        })),
        error: None,
    }
}

/// Handle unsubscribe request
async fn handle_unsubscribe(
    id: String,
    channel: String,
    client_id: usize,
    state: &AppState,
) -> ServerMessage {
    log::debug!(
        "Client {} unsubscribing from channel: {}",
        client_id,
        channel
    );

    state
        .subscription_manager
        .unsubscribe(client_id, &channel)
        .await;

    ServerMessage::Response {
        id,
        result: Some(serde_json::json!({
            "unsubscribed": true,
            "channel": channel
        })),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProfileManager;
    use crate::macro_recorder::MacroRecorder;
    use crate::services::{ConfigService, ProfileService};
    use std::path::PathBuf;

    #[test]
    fn test_create_router() {
        let config_dir = PathBuf::from("/tmp/keyrx-test");
        let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(crate::services::DeviceService::new(config_dir));
        let config_service = Arc::new(ConfigService::new(profile_manager));
        let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());
        let state = Arc::new(AppState::new(
            Arc::new(MacroRecorder::new()),
            profile_service,
            device_service,
            config_service,
            subscription_manager,
        ));
        let router = create_router(state);
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
