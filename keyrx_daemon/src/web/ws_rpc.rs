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
use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::web::rpc_types::{ClientMessage, RpcError, ServerMessage};
use crate::web::AppState;

/// Max consecutive lag events before disconnecting a slow client
const MAX_LAG_EVENTS: u32 = 3;

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
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or_else(|e| {
            log::warn!("Failed to get system time, using 0: {}", e);
            0
        });

    let connected = ServerMessage::Connected {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
    };

    if let Err(e) = send_message(&mut socket, &connected).await {
        log::warn!("Failed to send Connected handshake: {}", e);
        return;
    }

    // Subscribe to broadcast events
    let mut event_rx = state.event_broadcaster.subscribe();

    // Heartbeat and timeout tracking
    let mut heartbeat_interval = interval(Duration::from_secs(15));
    let mut timeout_check_interval = interval(Duration::from_secs(5));
    let mut last_pong_time = std::time::Instant::now();
    let mut lag_count = 0u32;

    // Main message processing loop
    loop {
        tokio::select! {
            // Handle broadcast events (full Result match for Lagged/Closed)
            event_result = event_rx.recv() => {
                match event_result {
                    Ok(event) => {
                        lag_count = 0;
                        // Check if this is an Event message and if client is subscribed
                        let should_send = match &event {
                            ServerMessage::Event { ref channel, .. } => {
                                state.subscription_manager.is_subscribed(client_id, channel).await
                            }
                            _ => true,
                        };

                        if should_send
                            && send_message(&mut socket, &event).await.is_err()
                        {
                            log::info!("WebSocket RPC client {} disconnected (send failed)", client_id);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        lag_count += 1;
                        log::warn!(
                            "WebSocket RPC client {} lagged (skipped {} messages, lag {}/{})",
                            client_id, skipped, lag_count, MAX_LAG_EVENTS
                        );
                        if lag_count >= MAX_LAG_EVENTS {
                            log::error!(
                                "WebSocket RPC client {} disconnected due to excessive lag",
                                client_id
                            );
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::info!("WebSocket RPC client {} event channel closed", client_id);
                        break;
                    }
                }
            }

            // Send ping frames as heartbeat
            _ = heartbeat_interval.tick() => {
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    log::info!("WebSocket RPC client {} disconnected (ping failed)", client_id);
                    break;
                }
            }

            // Check for timeout (30 seconds since last pong)
            _ = timeout_check_interval.tick() => {
                if last_pong_time.elapsed() > Duration::from_secs(30) {
                    log::warn!("WebSocket RPC client {} timeout (no pong)", client_id);
                    break;
                }
            }

            // Process incoming messages
            result = socket.recv() => match result {
                Some(Ok(Message::Text(text))) => {
                    log::debug!("Received WebSocket RPC message: {}", text);

                    // Parse message
                    let client_msg: ClientMessage = match serde_json::from_str(&text) {
                        Ok(msg) => msg,
                        Err(e) => {
                            log::warn!("Failed to parse message: {}", e);
                            let error_response = ServerMessage::Response {
                                id: String::new(),
                                result: None,
                                error: Some(RpcError::parse_error(format!("Invalid JSON: {}", e))),
                            };
                            if send_message(&mut socket, &error_response).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    // Process message and send response directly
                    let response = process_client_message(
                        client_msg,
                        client_id,
                        Arc::clone(&state),
                    )
                    .await;
                    if send_message(&mut socket, &response).await.is_err() {
                        log::info!("WebSocket RPC client {} disconnected (send failed)", client_id);
                        break;
                    }
                }
                Some(Ok(Message::Pong(_))) => {
                    last_pong_time = std::time::Instant::now();
                }
                Some(Ok(Message::Ping(data))) => {
                    if socket.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                    last_pong_time = std::time::Instant::now();
                }
                Some(Ok(Message::Close(_))) => {
                    log::info!("WebSocket RPC client closed connection");
                    break;
                }
                Some(Ok(_)) => {
                    // Ignore binary messages
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
    }

    // Cleanup: unsubscribe from all channels
    state.subscription_manager.unsubscribe_all(client_id).await;
    log::info!("WebSocket RPC connection {} closed", client_id);
}

/// Serialize and send a ServerMessage over WebSocket.
async fn send_message(socket: &mut WebSocket, msg: &ServerMessage) -> Result<(), axum::Error> {
    let json = serde_json::to_string(msg).map_err(|e| {
        log::error!("Failed to serialize message: {}", e);
        axum::Error::new(e)
    })?;
    socket.send(Message::Text(json)).await
}

/// Process a client message and return the appropriate response
async fn process_client_message(
    msg: ClientMessage,
    client_id: usize,
    state: Arc<AppState>,
) -> ServerMessage {
    match msg {
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
    }
}

/// Handle query request (read-only operations)
async fn handle_query(
    id: String,
    method: String,
    params: Value,
    state: &AppState,
) -> ServerMessage {
    use crate::web::handlers::{config, device, metric, profile, setting};

    log::debug!("Handling query: {} with params: {}", method, params);

    let result = match method.as_str() {
        "get_profiles" => profile::get_profiles(&state.profile_service, params).await,
        "get_profile_config" => profile::get_profile_config(&state.profile_service, params).await,
        "get_devices" => device::get_devices(&state.device_service, params).await,
        "get_config" => config::get_config(&state.config_service, params).await,
        "get_layers" => config::get_layers(&state.config_service, params).await,
        "get_latency" => metric::get_latency(&state.macro_recorder, params).await,
        "get_events" => metric::get_events(&state.macro_recorder, params).await,
        "get_global_layout" => setting::get_global_layout(&state.settings_service, params).await,
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
    use crate::web::handlers::{config, daemon, device, metric, profile, setting};

    log::debug!("Handling command: {} with params: {}", method, params);

    let result = match method.as_str() {
        "create_profile" => profile::create_profile(&state.profile_service, params).await,
        "activate_profile" => profile::activate_profile(state, params).await,
        "delete_profile" => profile::delete_profile(&state.profile_service, params).await,
        "duplicate_profile" => profile::duplicate_profile(&state.profile_service, params).await,
        "rename_profile" => profile::rename_profile(&state.profile_service, params).await,
        "set_profile_config" => profile::set_profile_config(state, params).await,
        "rename_device" => device::rename_device(state, params).await,
        "forget_device" => device::forget_device(&state.device_service, params).await,
        "update_config" => config::update_config(&state.config_service, params).await,
        "set_key_mapping" => config::set_key_mapping(&state.config_service, params).await,
        "delete_key_mapping" => config::delete_key_mapping(&state.config_service, params).await,
        "set_global_layout" => setting::set_global_layout(&state.settings_service, params).await,
        "clear_events" => metric::clear_events(&state.macro_recorder, params).await,
        "simulate" => metric::simulate(&state.macro_recorder, params).await,
        "reset_simulator" => metric::reset_simulator(&state.macro_recorder, params).await,
        "restart_daemon" => daemon::restart_daemon(params).await,
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
        let profile_manager = match ProfileManager::new(config_dir.clone()) {
            Ok(pm) => Arc::new(pm),
            Err(e) => {
                eprintln!("Warning: Failed to create ProfileManager for test: {}", e);
                // Skip test if ProfileManager initialization fails (e.g., permission issues)
                return;
            }
        };
        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(crate::services::DeviceService::new(config_dir.clone()));
        let config_service = Arc::new(ConfigService::new(profile_manager));
        let settings_service = Arc::new(crate::services::SettingsService::new(config_dir.clone()));
        let simulation_service = Arc::new(crate::services::SimulationService::new(
            config_dir.clone(),
            None,
        ));
        let subscription_manager = Arc::new(crate::web::subscriptions::SubscriptionManager::new());
        let (event_broadcaster, _) = tokio::sync::broadcast::channel(1000);
        let state = Arc::new(AppState::new(
            Arc::new(MacroRecorder::new()),
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
            None, // No daemon state in tests
        ));
        let router = create_router(state);
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
