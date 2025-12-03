# Requirements Document

## Introduction

KeyRx has 201 raw println/eprintln calls mixed with structured tracing. Logging is inconsistent, making debugging difficult. There's no centralized metrics collection, and FFI has no observability bridge to Flutter. This spec standardizes logging and adds a metrics bridge.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Debuggability**: Consistent, searchable logs
- **Observability**: Metrics visible in UI
- **Reliability**: Issues are detectable

Per tech.md: "Structured logging: JSON format with timestamp, level, service, event, context"

## Requirements

### Requirement 1: Structured Logging

**User Story:** As a developer, I want structured logs, so that I can search and filter effectively.

#### Acceptance Criteria

1. WHEN logging occurs THEN structured format SHALL be used
2. IF println exists THEN it SHALL be replaced with tracing
3. WHEN log entry created THEN it SHALL have timestamp, level, context
4. IF JSON output enabled THEN logs SHALL be valid JSON

### Requirement 2: Log Levels

**User Story:** As a developer, I want consistent log levels, so that I can filter by severity.

#### Acceptance Criteria

1. WHEN log level is set THEN it SHALL filter appropriately
2. IF error occurs THEN error level SHALL be used
3. WHEN warning needed THEN warn level SHALL be used
4. IF debug info needed THEN debug/trace levels SHALL be used

### Requirement 3: Metrics Collection

**User Story:** As a user, I want metrics visible, so that I can see engine health.

#### Acceptance Criteria

1. WHEN engine runs THEN metrics SHALL be collected
2. IF latency measured THEN histogram SHALL be used
3. WHEN metrics requested THEN snapshot SHALL be available
4. IF metrics change THEN subscribers SHALL be notified

### Requirement 4: FFI Observability Bridge

**User Story:** As a Flutter developer, I want metrics in UI, so that users see engine status.

#### Acceptance Criteria

1. WHEN metrics collected THEN FFI export SHALL be available
2. IF logs emitted THEN callback to Flutter SHALL be optional
3. WHEN errors occur THEN Flutter SHALL be notified
4. IF subscription requested THEN real-time updates SHALL work

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Logging Pattern**: All code uses tracing
- **Centralized Metrics**: One metrics collector
- **Observable FFI**: Metrics bridge to Flutter

### Performance
- Logging overhead SHALL be < 1 microsecond when disabled
- Metrics collection SHALL be < 10 microseconds
- FFI bridge SHALL not block engine

### Maintainability
- Log format SHALL be documented
- Metrics SHALL be documented
- Adding new metrics SHALL be simple
