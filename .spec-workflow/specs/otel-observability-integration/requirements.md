# Requirements Document

## Introduction

The codebase has an `opentelemetry` optional feature in Cargo.toml but it's unused. There are no traces exported for latency analysis, no metrics exported for dashboards, only local tracing logs. This limits production monitoring capabilities.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Observability**: Production monitoring
- **Reliability**: Issue detection and diagnosis
- **Operations**: Integration with monitoring tools

## Requirements

### Requirement 1: Span Instrumentation

**User Story:** As an operator, I want distributed traces, so that I can analyze latency.

#### Acceptance Criteria

1. WHEN event processed THEN span SHALL be created
2. IF span has children THEN hierarchy SHALL be preserved
3. WHEN span ends THEN duration SHALL be recorded
4. IF attributes exist THEN they SHALL be included

### Requirement 2: Metrics Export

**User Story:** As an operator, I want metrics exported, so that I can build dashboards.

#### Acceptance Criteria

1. WHEN metrics collected THEN OTEL export SHALL be available
2. IF histogram used THEN buckets SHALL be configured
3. WHEN counter incremented THEN it SHALL be exported
4. IF gauge updated THEN current value SHALL export

### Requirement 3: Configuration

**User Story:** As an operator, I want configurable endpoints, so that I can point to my backend.

#### Acceptance Criteria

1. WHEN OTEL enabled THEN endpoint SHALL be configurable
2. IF env var set THEN it SHALL be used
3. WHEN export fails THEN fallback SHALL occur
4. IF disabled THEN overhead SHALL be zero

## Non-Functional Requirements

### Performance
- OTEL overhead SHALL be < 5% when enabled
- Export SHALL be async/batched
- Disabled OTEL SHALL have zero overhead
