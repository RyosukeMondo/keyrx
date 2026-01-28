//! Simulator endpoints.

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::simulation_engine::{
    EventSequence, EventType, ScenarioResult, SimulatedEvent, SimulationError,
};
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/simulator/events", post(simulate_events))
        .route("/simulator/reset", post(reset_simulator))
        .route("/simulator/load-profile", post(load_profile))
        .route("/simulator/scenarios/all", post(run_all_scenarios))
}

/// Convert SimulationError to ApiError
impl From<SimulationError> for ApiError {
    fn from(e: SimulationError) -> Self {
        match e {
            SimulationError::LoadError(msg) => ApiError::NotFound(msg),
            SimulationError::ScenarioNotFound(msg) => {
                ApiError::NotFound(format!("Scenario not found: {}", msg))
            }
            SimulationError::TooManyEvents(count) => {
                ApiError::BadRequest(format!("Too many events: {}", count))
            }
            SimulationError::FileTooLarge(size) => {
                ApiError::BadRequest(format!("File too large: {} bytes", size))
            }
            SimulationError::InvalidTimestamp(ts) => {
                ApiError::BadRequest(format!("Invalid timestamp: {}", ts))
            }
            SimulationError::InvalidEventFile(msg) => ApiError::BadRequest(msg),
            SimulationError::MemoryLimitExceeded => {
                ApiError::InternalError("Memory limit exceeded".to_string())
            }
            SimulationError::IoError(e) => ApiError::InternalError(e.to_string()),
            SimulationError::JsonError(e) => ApiError::BadRequest(e.to_string()),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadProfileRequest {
    /// Profile name (without .krx extension)
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulateEventsRequest {
    /// Optional scenario name (e.g., "tap-hold-under-threshold")
    scenario: Option<String>,
    /// Optional DSL string (e.g., "press:A,wait:50,release:A")
    dsl: Option<String>,
    /// Optional custom event sequence
    events: Option<Vec<SimulatedEvent>>,
    /// Seed for deterministic behavior
    seed: Option<u64>,
}

/// Output event in API response format
#[derive(Serialize)]
struct OutputEventResponse {
    /// Key identifier
    key: String,
    /// Event type: "press" or "release"
    event_type: String,
    /// Timestamp in microseconds
    timestamp_us: u64,
}

#[derive(Serialize)]
struct SimulateEventsResponse {
    success: bool,
    /// List of output events
    outputs: Vec<OutputEventResponse>,
}

#[derive(Serialize)]
struct AllScenariosResponse {
    success: bool,
    /// Results for all scenarios
    scenarios: Vec<ScenarioResult>,
    /// Total number of scenarios
    total: usize,
    /// Number of passed scenarios
    passed: usize,
    /// Number of failed scenarios
    failed: usize,
}

/// POST /api/simulator/load-profile - Load a profile for simulation
async fn load_profile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoadProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    use super::validation::validate_profile_name;
    validate_profile_name(&payload.name)?;

    state.simulation_service.load_profile(&payload.name)?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Profile '{}' loaded successfully", payload.name)
    })))
}

/// POST /api/simulator/events - Simulate events
async fn simulate_events(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SimulateEventsRequest>,
) -> Result<Json<SimulateEventsResponse>, ApiError> {
    // Validate that only one input method is provided
    let input_count = [
        payload.scenario.is_some(),
        payload.dsl.is_some(),
        payload.events.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if input_count == 0 {
        return Err(ApiError::BadRequest(
            "Must provide either 'scenario', 'dsl', or 'events'".to_string(),
        ));
    }

    if input_count > 1 {
        return Err(ApiError::BadRequest(
            "Must provide exactly one of: 'scenario', 'dsl', or 'events'".to_string(),
        ));
    }

    // Determine which simulation method to use
    let outputs = if let Some(scenario_name) = payload.scenario {
        // Use built-in scenario
        let result = state.simulation_service.run_scenario(&scenario_name)?;
        result.output
    } else if let Some(dsl) = payload.dsl {
        // Use DSL
        // Validate DSL length (max 10KB to prevent abuse)
        if dsl.len() > 10_000 {
            return Err(ApiError::BadRequest(format!(
                "DSL too long (max 10000 characters, got {})",
                dsl.len()
            )));
        }

        let seed = payload.seed.unwrap_or(0);
        state.simulation_service.replay_dsl(&dsl, seed).await?
    } else if let Some(events) = payload.events {
        // Use custom event sequence
        // Validate event count (max 10000 events to prevent abuse)
        if events.len() > 10_000 {
            return Err(ApiError::BadRequest(format!(
                "Too many events (max 10000, got {})",
                events.len()
            )));
        }

        let sequence = EventSequence {
            events,
            seed: payload.seed.unwrap_or(0),
        };
        state.simulation_service.replay(&sequence).await?
    } else {
        // This should never happen due to input_count check above
        return Err(ApiError::InternalError(
            "Unreachable: no input method selected".to_string(),
        ));
    };

    // Convert outputs to API response format
    let response_outputs: Vec<OutputEventResponse> = outputs
        .iter()
        .map(|e| OutputEventResponse {
            key: e.key.clone(),
            event_type: match e.event_type {
                EventType::Press => "press".to_string(),
                EventType::Release => "release".to_string(),
            },
            timestamp_us: e.timestamp_us,
        })
        .collect();

    Ok(Json(SimulateEventsResponse {
        success: true,
        outputs: response_outputs,
    }))
}

/// POST /api/simulator/scenarios/all - Run all built-in scenarios
async fn run_all_scenarios(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AllScenariosResponse>, ApiError> {
    let scenarios = state.simulation_service.run_all_scenarios()?;

    let total = scenarios.len();
    let passed = scenarios.iter().filter(|s| s.passed).count();
    let failed = total - passed;

    Ok(Json(AllScenariosResponse {
        success: true,
        scenarios,
        total,
        passed,
        failed,
    }))
}

/// POST /api/simulator/reset - Reset simulator
async fn reset_simulator(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    state.simulation_service.reset();

    Ok(Json(json!({
        "success": true,
        "message": "Simulator state reset successfully"
    })))
}
