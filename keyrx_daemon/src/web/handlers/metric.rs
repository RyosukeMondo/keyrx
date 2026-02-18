//! Metrics and simulator RPC method handlers.
//!
//! This module implements all metrics and simulation-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the MacroRecorder or simulation engine for execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;

use crate::config::simulation_engine::{BuiltinScenario, SimulatedEvent};
use crate::macro_recorder::MacroRecorder;
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

/// Latency statistics returned by get_latency
#[typeshare]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyRpcStats {
    #[typeshare(serialized_as = "number")]
    pub min_us: u64,
    #[typeshare(serialized_as = "number")]
    pub avg_us: u64,
    #[typeshare(serialized_as = "number")]
    pub max_us: u64,
    #[typeshare(serialized_as = "number")]
    pub p50_us: u64,
    #[typeshare(serialized_as = "number")]
    pub p95_us: u64,
    #[typeshare(serialized_as = "number")]
    pub p99_us: u64,
    #[typeshare(serialized_as = "number")]
    pub count: usize,
}

/// Event in event log
#[typeshare]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRpcEntry {
    #[typeshare(serialized_as = "number")]
    pub timestamp: u64,
    pub key_code: u16,
    pub event_type: String,
    pub device_id: String,
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

/// Get latency statistics
pub async fn get_latency(
    macro_recorder: &MacroRecorder,
    _params: Value,
) -> Result<Value, RpcError> {
    // For now, return placeholder stats
    // In the future, this would query the daemon's actual latency metrics
    let stats = LatencyRpcStats {
        min_us: 0,
        avg_us: 0,
        max_us: 0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        count: macro_recorder.event_count(),
    };

    serde_json::to_value(&stats).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Get event log with pagination
pub async fn get_events(macro_recorder: &MacroRecorder, params: Value) -> Result<Value, RpcError> {
    let params: GetEventsParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    // Enforce limits: default 100, max 1000
    const MAX_LIMIT: usize = 1000;
    let limit = params.limit.min(MAX_LIMIT);

    log::debug!("RPC: get_events limit={} offset={}", limit, params.offset);

    // Get recorded events
    let all_events = macro_recorder
        .get_recorded_events()
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    // Apply pagination
    let total = all_events.len();
    let events_page: Vec<EventRpcEntry> = all_events
        .into_iter()
        .skip(params.offset)
        .take(limit)
        .map(|e| EventRpcEntry {
            timestamp: e.relative_timestamp_us,
            key_code: e.event.keycode() as u16,
            event_type: format!("{:?}", e.event.event_type()),
            device_id: e.event.device_id().unwrap_or("default").to_string(),
        })
        .collect();

    serde_json::to_value(serde_json::json!({
        "events": events_page,
        "total": total,
        "limit": limit,
        "offset": params.offset
    }))
    .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Clear event log
pub async fn clear_events(
    macro_recorder: &MacroRecorder,
    _params: Value,
) -> Result<Value, RpcError> {
    log::info!("RPC: clear_events");

    macro_recorder
        .clear_events()
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    Ok(serde_json::json!({ "success": true }))
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

    #[test]
    fn test_latency_stats_serialization() {
        let stats = LatencyRpcStats {
            min_us: 10,
            avg_us: 50,
            max_us: 100,
            p50_us: 45,
            p95_us: 90,
            p99_us: 98,
            count: 1000,
        };
        let json = serde_json::to_value(&stats).expect("Failed to serialize LatencyRpcStats");
        assert_eq!(json["minUs"], 10);
        assert_eq!(json["count"], 1000);
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
