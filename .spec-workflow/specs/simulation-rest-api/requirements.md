# Requirements Document: Simulation REST API

## Introduction

Enable deterministic keyboard event simulation via REST API, completing the simulation infrastructure chain: daemon CLI → REST API → Playwright E2E tests. This addresses the WASM browser error and wires the existing `SimulationEngine` to REST endpoints, enabling AI-first automated testing of keyboard configurations.

## Alignment with Product Vision

This feature directly supports the **AI Coding Agent First** principle from product.md:
- **Deterministic Simulation Testing (DST)** with virtual clock enables 100% configuration verification
- **CLI-first design**: Every simulation operation accessible via machine-readable REST API
- **Browser-based WASM simulation** for edit-and-preview workflow
- **Structured JSON responses** for AI agent observability

## Requirements

### Requirement 1: WASM Environment Compatibility

**User Story:** As a developer, I want the WASM module to load in the browser without errors, so that I can use browser-based simulation.

#### Acceptance Criteria

1. WHEN the browser loads the WASM module THEN the system SHALL provide all required environment imports (including `env.now`)
2. IF the WASM module requires timing functions THEN the system SHALL provide high-resolution timestamps via `performance.now()`
3. WHEN WASM initialization completes THEN the system SHALL call `wasm_init()` successfully without console errors

### Requirement 2: Simulation Profile Loading

**User Story:** As a tester, I want to load a profile into the simulator, so that I can run simulation scenarios against specific configurations.

#### Acceptance Criteria

1. WHEN POST /api/simulator/load-profile is called with `{"profile": "name"}` THEN the system SHALL load the corresponding .krx file
2. IF the profile does not exist THEN the system SHALL return 400 Bad Request with error message
3. WHEN a profile is loaded THEN the system SHALL retain it for subsequent simulation requests

### Requirement 3: Event Simulation via REST API

**User Story:** As an AI agent, I want to simulate keyboard events via REST API, so that I can verify configuration behavior deterministically.

#### Acceptance Criteria

1. WHEN POST /api/simulator/events is called with `{"scenario": "tap-hold-under-threshold"}` THEN the system SHALL run the built-in scenario and return output events
2. WHEN POST /api/simulator/events is called with `{"dsl": "press:A,wait:50,release:A"}` THEN the system SHALL parse and execute the event DSL
3. WHEN POST /api/simulator/events is called with `{"events": [...]}` THEN the system SHALL process the custom event sequence
4. IF no profile is loaded THEN the system SHALL return 500 Internal Error with "No profile loaded" message
5. WHEN `seed` parameter is provided THEN the system SHALL produce deterministic, reproducible results

### Requirement 4: Built-in Scenario Execution

**User Story:** As a tester, I want to run built-in test scenarios, so that I can verify standard tap-hold and modifier behaviors.

#### Acceptance Criteria

1. WHEN POST /api/simulator/scenarios/all is called THEN the system SHALL run all 5 built-in scenarios
2. THEN the response SHALL include pass/fail status for each scenario
3. WHEN a scenario fails THEN the response SHALL include error details

### Requirement 5: Simulator State Reset

**User Story:** As a tester, I want to reset the simulator state, so that I can run isolated test sequences.

#### Acceptance Criteria

1. WHEN POST /api/simulator/reset is called THEN the system SHALL clear the loaded profile
2. WHEN reset completes THEN subsequent simulation requests SHALL fail until a profile is loaded

### Requirement 6: E2E Test Integration

**User Story:** As a CI pipeline, I want Playwright tests that verify simulation endpoints, so that I can catch regressions automatically.

#### Acceptance Criteria

1. WHEN the E2E test suite runs THEN it SHALL include simulator API tests
2. THEN tests SHALL verify built-in scenarios produce expected output events
3. THEN tests SHALL verify DSL parsing produces deterministic results
4. THEN tests SHALL verify error handling for missing profiles

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: SimulationService handles simulation logic, separate from HTTP routing
- **Modular Design**: Service layer injectable via AppState for testability
- **Dependency Management**: SimulationService depends only on SimulationEngine (no circular deps)
- **Clear Interfaces**: REST endpoints documented with request/response schemas

### Performance
- Simulation of 1000 events completes in <100ms
- No additional latency on daemon startup (lazy initialization)

### Security
- No PII in simulation logs
- Event DSL parsing validates input before execution

### Reliability
- Simulation errors return structured JSON errors, not panics
- Mutex-protected state prevents concurrent access issues

### Usability
- All endpoints return JSON with `success` boolean field
- Error responses include actionable `message` field
