# Requirements Document

## Introduction

KeyRx has 15+ state-related types scattered across the codebase with overlapping responsibilities. `EngineState` is defined in multiple places, alongside `RecordingState`, `DiscoverySessionState`, `MockState`, `ReplayState`, and others. This creates confusion, bugs, and makes it impossible to reason about state transitions. This spec audits all state types and creates a unified state graph.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Reliability**: Clear state management prevents bugs
- **Maintainability**: Single source of truth for state
- **Debuggability**: State transitions are traceable

Per tech.md: "No Global State" and "Event Sourcing"

## Requirements

### Requirement 1: State Inventory

**User Story:** As a developer, I want to know all state types, so that I can understand the system.

#### Acceptance Criteria

1. WHEN auditing state THEN all state types SHALL be cataloged
2. IF state types overlap THEN they SHALL be documented
3. WHEN inventory is complete THEN ownership SHALL be clear
4. IF state is duplicated THEN consolidation plan SHALL exist

### Requirement 2: State Graph

**User Story:** As a developer, I want a state transition graph, so that I can see valid transitions.

#### Acceptance Criteria

1. WHEN states exist THEN valid transitions SHALL be documented
2. IF a transition occurs THEN it SHALL be in the graph
3. WHEN invalid transition attempted THEN it SHALL be rejected
4. IF transitions are documented THEN they SHALL be enforced

### Requirement 3: State Validation

**User Story:** As a developer, I want state invariants checked, so that bugs are caught early.

#### Acceptance Criteria

1. WHEN state changes THEN invariants SHALL be validated
2. IF invariant violated THEN error SHALL be raised
3. WHEN in debug mode THEN extra validation SHALL occur
4. IF validation fails THEN state change SHALL be rejected

### Requirement 4: State Transition Logging

**User Story:** As a developer, I want transition logs, so that I can debug state issues.

#### Acceptance Criteria

1. WHEN state transitions THEN it SHALL be logged
2. IF replay is needed THEN log SHALL be sufficient
3. WHEN debugging THEN transition history SHALL be available
4. IF logging disabled THEN overhead SHALL be zero

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Source of Truth**: One canonical state representation
- **State Ownership**: Clear ownership boundaries
- **Transition Enforcement**: Type-system enforced transitions

### Maintainability
- State types SHALL be in dedicated module
- Transitions SHALL be explicit enum variants
- Overlapping types SHALL be consolidated

### Debuggability
- State snapshots SHALL be serializable
- Transitions SHALL be replayable
- History SHALL be queryable
