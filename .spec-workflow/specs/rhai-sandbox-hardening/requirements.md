# Requirements Document

## Introduction

KeyRx exposes 87 Rhai functions with minimal input validation. There's no centralized allowlist for safe_mode, no resource limits (CPU, memory, recursion), and script context can be corrupted by invalid state. The registry lookup is O(n) per keypress. This spec hardens the Rhai sandbox for production use.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Safety First**: Scripts cannot harm system or user
- **Performance**: Script execution is bounded
- **Reliability**: Invalid scripts fail gracefully

Per tech.md: "Security - no privilege escalation"

## Requirements

### Requirement 1: Resource Limits

**User Story:** As a user, I want scripts bounded, so that a bug doesn't freeze my keyboard.

#### Acceptance Criteria

1. WHEN script runs THEN instruction count SHALL be limited
2. IF recursion exceeds depth THEN script SHALL terminate
3. WHEN memory exceeds limit THEN script SHALL terminate
4. IF timeout reached THEN script SHALL terminate

### Requirement 2: Capability System

**User Story:** As a developer, I want function tiers, so that safe_mode is enforced.

#### Acceptance Criteria

1. WHEN function is registered THEN capability tier SHALL be assigned
2. IF safe_mode enabled THEN only safe functions SHALL be callable
3. WHEN capability checked THEN it SHALL be O(1)
4. IF tier violated THEN clear error SHALL be returned

### Requirement 3: Input Validation

**User Story:** As a user, I want input validated, so that invalid data doesn't crash the engine.

#### Acceptance Criteria

1. WHEN function receives input THEN it SHALL validate parameters
2. IF validation fails THEN error SHALL describe problem
3. WHEN types mismatch THEN conversion SHALL be attempted
4. IF conversion fails THEN clear error SHALL be returned

### Requirement 4: Registry Optimization

**User Story:** As a user, I want fast script execution, so that there's no perceptible lag.

#### Acceptance Criteria

1. WHEN registry is queried THEN lookup SHALL be O(1)
2. IF KeyCode maps to function THEN HashMap SHALL be used
3. WHEN function called THEN dispatch SHALL be direct
4. IF caching helps THEN it SHALL be implemented

## Non-Functional Requirements

### Code Architecture and Modularity
- **Capability Tiers**: Clear function categorization
- **Centralized Limits**: Single resource budget definition
- **Validation Layer**: Input validation separate from logic

### Security
- Scripts SHALL NOT access filesystem directly
- Scripts SHALL NOT execute shell commands
- Scripts SHALL NOT elevate privileges
- Resource exhaustion SHALL be prevented

### Performance
- Registry lookup SHALL be O(1)
- Validation overhead SHALL be < 10 microseconds
- Capability check SHALL be < 1 microsecond
