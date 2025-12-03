# Requirements Document

## Introduction

The engine module (6,001 LOC across 21 files) has state distributed across multiple structs: `KeyStateTracker`, `LayerStack`, `ModifierState`, `PendingDecisionQueue`, and more. This distributed state makes reasoning about consistency difficult and causes subtle timing bugs. This spec unifies engine state under a single facade with explicit mutation boundaries.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Unified state enables atomic updates and reduces lock contention
- **Visual > Abstract**: State visible through single inspection point
- **Safety First**: Consistent state prevents undefined behavior

Per tech.md: "No Global State: All instances are self-contained structs" and "Event Sourcing: Input treated as immutable event stream"

## Requirements

### Requirement 1: Unified State Container

**User Story:** As a developer, I want all engine state in a single container, so that I can reason about state consistency.

#### Acceptance Criteria

1. WHEN inspecting engine state THEN a single `EngineState` struct SHALL contain all state
2. IF state is accessed THEN it SHALL be through typed accessors on EngineState
3. WHEN state is modified THEN it SHALL go through mutation methods
4. IF state snapshots are needed THEN EngineState SHALL be Clone-able

### Requirement 2: Explicit Mutation Boundaries

**User Story:** As a developer, I want clear mutation boundaries, so that I know exactly when and how state changes.

#### Acceptance Criteria

1. WHEN an event is processed THEN mutations SHALL happen in a single `apply()` call
2. IF multiple state changes are needed THEN they SHALL be batched atomically
3. WHEN mutations occur THEN a `StateChange` record SHALL be emitted
4. IF a mutation fails THEN state SHALL remain unchanged (rollback)

### Requirement 3: State Synchronization

**User Story:** As a developer, I want state components synchronized, so that modifiers, layers, and pending decisions are consistent.

#### Acceptance Criteria

1. WHEN a key is released THEN all dependent state (modifiers, pending) SHALL update
2. IF a layer is popped THEN affected modifiers SHALL be deactivated
3. WHEN state is queried THEN it SHALL reflect all prior mutations
4. IF timing decisions resolve THEN dependent state SHALL update immediately

### Requirement 4: State Inspection API

**User Story:** As a Flutter developer, I want to inspect engine state, so that I can display it in the debugger UI.

#### Acceptance Criteria

1. WHEN state is requested THEN a serializable snapshot SHALL be provided
2. IF specific state is needed THEN focused queries SHALL be available
3. WHEN state history is needed THEN recent changes SHALL be accessible
4. IF real-time updates are needed THEN state change events SHALL be emitted

### Requirement 5: State Persistence

**User Story:** As a user, I want engine state to persist across sessions, so that I don't lose layer/modifier state on restart.

#### Acceptance Criteria

1. WHEN engine stops THEN state SHALL be serializable to disk
2. IF engine starts THEN previous state SHALL be restorable
3. WHEN state format changes THEN migration SHALL be supported
4. IF state is corrupted THEN engine SHALL start with clean state

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: EngineState manages state, engine manages logic
- **Modular Design**: State components are internal modules
- **Dependency Management**: State doesn't depend on engine logic
- **Clear Interfaces**: Public API for state access, internal for mutation

### Performance
- State access SHALL be O(1) for common operations
- State cloning SHALL be efficient (< 1ms for typical state)
- Lock contention SHALL be minimized (consider RwLock)

### Security
- State SHALL not contain sensitive information (no file paths)
- State exports SHALL be sanitized for sharing

### Reliability
- State invariants SHALL be validated after each mutation
- Invalid state transitions SHALL panic in debug, log in release
- State corruption SHALL be detectable

### Usability
- State inspection SHALL be available in one method call
- State debugging SHALL be supported via Debug impl
- State documentation SHALL explain all components
