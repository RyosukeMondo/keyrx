# Requirements Document

## Introduction

The engine has no timeout mechanism, memory usage bounds, or output queue limits beyond Rhai's built-in limits. A malicious or buggy script could exhaust resources. This spec adds runtime resource constraints enforcement.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Safety First**: Prevent resource exhaustion
- **Reliability**: Bounded resource usage
- **Security**: Protect against malicious scripts

## Requirements

### Requirement 1: Execution Timeout

**User Story:** As a user, I want script timeouts, so that runaway scripts don't freeze my keyboard.

#### Acceptance Criteria

1. WHEN script runs too long THEN it SHALL be terminated
2. IF timeout occurs THEN key SHALL pass through
3. WHEN timeout configured THEN it SHALL be respected
4. IF default timeout THEN it SHALL be 100ms

### Requirement 2: Memory Limits

**User Story:** As a user, I want memory limits, so that scripts can't exhaust RAM.

#### Acceptance Criteria

1. WHEN memory limit reached THEN script SHALL terminate
2. IF allocation fails THEN graceful error SHALL occur
3. WHEN limit configured THEN it SHALL be respected
4. IF default limit THEN it SHALL be 10MB

### Requirement 3: Queue Limits

**User Story:** As a developer, I want output bounds, so that event queues don't overflow.

#### Acceptance Criteria

1. WHEN output queue full THEN oldest events MAY be dropped
2. IF queue limit reached THEN warning SHALL be logged
3. WHEN limit configured THEN it SHALL be respected
4. IF default limit THEN it SHALL be 1000 events

## Non-Functional Requirements

### Performance
- Limit checks SHALL be < 1μs
- Enforcement SHALL not impact normal operation
- Recovery from limit SHALL be < 1ms
