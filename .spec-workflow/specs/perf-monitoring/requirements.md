# Requirements Document

## Introduction

KeyRx has no performance monitoring infrastructure - no p99/p95 latency tracking, no hot path profiling, no memory tracking. For a keyboard remapping engine where latency directly impacts user experience, this is critical. This spec adds comprehensive performance monitoring with minimal overhead.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Can't optimize what you don't measure
- **Low Latency**: Sub-millisecond targets require measurement
- **Reliability**: Memory leaks must be detectable

Per tech.md: "Hook callbacks SHALL complete in < 100 microseconds"

## Requirements

### Requirement 1: Latency Tracking

**User Story:** As a developer, I want to track p50/p95/p99 latencies, so that I can identify performance regressions.

#### Acceptance Criteria

1. WHEN a key event is processed THEN processing time SHALL be recorded
2. IF latency exceeds threshold THEN a warning SHALL be logged
3. WHEN percentiles are calculated THEN p50/p95/p99 SHALL be available
4. IF latency is requested THEN rolling window stats SHALL be returned

### Requirement 2: Memory Monitoring

**User Story:** As a developer, I want to track memory usage, so that I can detect memory leaks.

#### Acceptance Criteria

1. WHEN the engine runs THEN memory usage SHALL be tracked periodically
2. IF memory grows beyond threshold THEN a warning SHALL be logged
3. WHEN stats are requested THEN current/peak/average SHALL be available
4. IF a leak is detected THEN allocation site hints SHALL be provided

### Requirement 3: Hot Path Profiling

**User Story:** As a developer, I want to profile hot paths, so that I can optimize critical code.

#### Acceptance Criteria

1. WHEN profiling is enabled THEN function timings SHALL be recorded
2. IF a hot spot exists THEN it SHALL be identified in reports
3. WHEN profiling data is exported THEN flamegraph format SHALL be available
4. IF overhead exceeds 5% THEN profiling SHALL be disabled

### Requirement 4: Metrics Export

**User Story:** As a user, I want to view performance metrics, so that I can verify the engine is running well.

#### Acceptance Criteria

1. WHEN metrics are requested THEN JSON format SHALL be available
2. IF FFI exports metrics THEN they SHALL be serializable
3. WHEN Flutter UI requests metrics THEN real-time updates SHALL work
4. IF historical data is needed THEN configurable retention SHALL exist

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility**: Each metric type has one module
- **Modular Design**: Metrics can be enabled/disabled independently
- **Dependency Injection**: Metrics collector is injectable
- **Clear Interfaces**: Simple record/query API

### Performance
- Metric recording overhead SHALL be < 1 microsecond
- Memory for metrics SHALL be bounded
- Percentile calculation SHALL be O(1) using histograms
- No allocations in hot path recording

### Reliability
- Metrics SHALL survive engine errors
- Overflow SHALL be handled gracefully
- Missing samples SHALL not corrupt stats
