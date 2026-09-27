//! Metrics and simulator RPC method handlers.
//!
//! This module implements all metrics and simulation-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the daemon read model (latency, event log) or the simulation engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;

use crate::config::simulation_engine::{BuiltinScenario, SimulatedEvent};
use crate::macro_recorder::MacroRecorder;
use crate::services::DaemonQueryService;
use crate::web::events::KeyEventData;
use crate::web::rpc_types::{RpcError, INTERNAL_ERROR, INVALID_PARAMS};

/// Parameters for get_latency query
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetLatencyParams {
    // No parameters needed
}

/// Parameters for get_events query
#[derive(Debug, Deserialize)]
struct GetEventsParams {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    100
}

/// Parameters for clear_events command
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClearEventsParams {
    // No parameters needed
}

/// Parameters for simulate command
#[derive(Debug, Deserialize)]
struct SimulateParams {
    /// Built-in scenario name or custom events
    scenario: Option<String>,
    /// Custom event sequence (if not using scenario)
    events: Option<Vec<SimulatedEvent>>,
    /// Optional seed for deterministic simulation
    #[allow(dead_code)]
    seed: Option<u64>,
}

/// Parameters for reset_simulator command
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResetSimulatorParams {
    // No parameters needed
}

/// A page of the live key-event log returned by get_events (oldest first).
#[typeshare]
#[derive(Debug, Serialize)]
pub struct EventPage {
    pub events: Vec<KeyEventData>,
    /// Events in the log.
    #[typeshare(serialized_as = "number")]
    pub total: usize,
    #[typeshare(serialized_as = "number")]
    pub limit: usize,
    #[typeshare(serialized_as = "number")]
    pub offset: usize,
}

/// Simulation result
#[derive(Debug, Serialize)]
struct SimulationRpcResult {
    success: bool,
    event_count: usize,
    output_count: usize,
    duration_us: u64,
    outputs: Vec<String>,
}

/// Get latency statistics (same `LatencyStats` as REST and the WS feed)
pub async fn get_latency(query: &DaemonQueryService, _params: Value) -> Result<Value, RpcError> {
    to_rpc_value(&query.get_latency_stats())
}

/// Get the live key-event log with pagination
pub async fn get_events(query: &DaemonQueryService, params: Value) -> Result<Value, RpcError> {
    let params: GetEventsParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    // Enforce limits: default 100, max 1000
    const MAX_LIMIT: usize = 1000;
    let limit = params.limit.min(MAX_LIMIT);
    log::debug!("RPC: get_events limit={} offset={}", limit, params.offset);

    let all = query.get_recent_events(usize::MAX);
    let total = all.len();
    let events = all.into_iter().skip(params.offset).take(limit).collect();
    to_rpc_value(&EventPage {
        events,
        total,
        limit,
        offset: params.offset,
    })
}

/// Clear the live key-event log
pub async fn clear_events(query: &DaemonQueryService, _params: Value) -> Result<Value, RpcError> {
    let cleared = query.clear_events();
    log::info!("RPC: clear_events ({cleared} removed)");
    Ok(serde_json::json!({ "success": true, "cleared": cleared }))
}

fn to_rpc_value<T: Serialize>(value: &T) -> Result<Value, RpcError> {
    serde_json::to_value(value).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Run simulation
pub async fn simulate(_macro_recorder: &MacroRecorder, params: Value) -> Result<Value, RpcError> {
    let params: SimulateParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    log::debug!("RPC: simulate scenario={:?}", params.scenario);

    // Determine event sequence
    let events = if let Some(scenario_name) = params.scenario {
        // Use built-in scenario
        let scenario = match scenario_name.as_str() {
            "tap-hold-under-threshold" => BuiltinScenario::TapHoldUnderThreshold,
            "tap-hold-over-threshold" => BuiltinScenario::TapHoldOverThreshold,
            "permissive-hold" => BuiltinScenario::PermissiveHold,
            "cross-device-modifiers" => BuiltinScenario::CrossDeviceModifiers,
            "macro-sequence" => BuiltinScenario::MacroSequence,
            _ => {
                return Err(RpcError::new(
                    INVALID_PARAMS,
                    format!("Unknown scenario: {}", scenario_name),
                ))
            }
        };
        scenario.generate_events().events
    } else if let Some(events) = params.events {
        // Use custom event sequence
        events
    } else {
        return Err(RpcError::new(
            INVALID_PARAMS,
            "Must provide either 'scenario' or 'events'".to_string(),
        ));
    };

    // Note: Actual simulation would require running the events through
    // the keyrx_core processor with the active profile's config.
    // For now, return a placeholder result showing the input events.
    let duration_us = events.last().map(|e| e.timestamp_us).unwrap_or(0);

    let result = SimulationRpcResult {
        success: true,
        event_count: events.len(),
        output_count: events.len(), // Placeholder: would be actual output count
        duration_us,
        outputs: events
            .iter()
            .map(|e| format!("{:?} {} at {}μs", e.event_type, e.key, e.timestamp_us))
            .collect(),
    };

    serde_json::to_value(&result).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Reset simulator state
pub async fn reset_simulator(
    _macro_recorder: &MacroRecorder,
    _params: Value,
) -> Result<Value, RpcError> {
    log::debug!("RPC: reset_simulator");

    // Simulator state is ephemeral (no persistent state)
    Ok(serde_json::json!({
        "success": true,
        "message": "Simulator state is ephemeral"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_get_latency_params() {
        let params = json!({});
        let result: Result<GetLatencyParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_get_events_params_defaults() {
        let params = json!({});
        let result: Result<GetEventsParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let p = result.expect("Failed to deserialize GetEventsParams with defaults");
        assert_eq!(p.limit, 100);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_deserialize_get_events_params_custom() {
        let params = json!({
            "limit": 50,
            "offset": 10
        });
        let result: Result<GetEventsParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let p = result.expect("Failed to deserialize GetEventsParams");
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 10);
    }

    #[test]
    fn test_deserialize_simulate_params_scenario() {
        let params = json!({
            "scenario": "tap-hold-under-threshold"
        });
        let result: Result<SimulateParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let params = result.expect("Failed to deserialize SimulateParams");
        assert_eq!(
            params.scenario.expect("Scenario should be present"),
            "tap-hold-under-threshold"
        );
    }

    /// Regression: get_latency returned hard-coded zeros and get_events /
    /// clear_events used the macro recorder instead of the live event log.
    #[tokio::test]
    async fn test_rpc_metrics_read_the_live_read_model() {
        use crate::daemon::{DaemonSharedState, DaemonTelemetry};
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let telemetry = Arc::new(DaemonTelemetry::new());
        telemetry.latency_recorder().record(250);
        telemetry.push_event(KeyEventData::test_press("A"));
        telemetry.push_event(KeyEventData::test_press("B"));
        let shared = Arc::new(DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            None,
            std::path::PathBuf::new(),
            0,
        ));
        let query = DaemonQueryService::new(shared, Arc::clone(&telemetry));

        let latency = get_latency(&query, json!({})).await.unwrap();
        assert_eq!(latency["max"], 250);
        assert_eq!(latency["samples"], 1);

        let page = get_events(&query, json!({"limit": 1, "offset": 1}))
            .await
            .unwrap();
        assert_eq!(page["total"], 2);
        assert_eq!(page["events"][0]["input"], "B");

        let cleared = clear_events(&query, json!({})).await.unwrap();
        assert_eq!(cleared["cleared"], 2);
        assert!(telemetry.recent_events(10).is_empty());
    }

    #[test]
    fn test_simulation_result_serialization() {
        let result = SimulationRpcResult {
            success: true,
            event_count: 5,
            output_count: 5,
            duration_us: 1000,
            outputs: vec!["event1".to_string()],
        };
        let json = serde_json::to_value(&result).expect("Failed to serialize SimulationRpcResult");
        assert_eq!(json["success"], true);
        assert_eq!(json["event_count"], 5);
    }
}
