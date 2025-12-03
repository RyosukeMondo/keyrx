# Requirements Document

## Introduction

Each individual key event creates independent FFI calls and engine invocations with no batching. With rapid key presses, this generates many small transactions. Coalescing events within a time window reduces overhead and improves throughput.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Reduce per-event overhead
- **Efficiency**: Batch operations where possible
- **Throughput**: Handle typing-heavy workloads

## Requirements

### Requirement 1: Event Batching

**User Story:** As a user, I want efficient event processing, so that rapid typing doesn't lag.

#### Acceptance Criteria

1. WHEN events arrive rapidly THEN they SHALL be batched
2. IF batch timeout reached THEN batch SHALL be processed
3. WHEN batch size limit reached THEN immediate processing SHALL occur
4. IF single event arrives THEN it SHALL process without delay

### Requirement 2: Coalescing Rules

**User Story:** As a developer, I want smart coalescing, so that event semantics are preserved.

#### Acceptance Criteria

1. WHEN repeat events occur THEN they MAY be coalesced
2. IF down/up pair within window THEN they SHALL stay paired
3. WHEN modifier state changes THEN batch SHALL flush
4. IF timing matters THEN timestamps SHALL be preserved

### Requirement 3: Configuration

**User Story:** As a user, I want configurable batching, so that I can tune for my use case.

#### Acceptance Criteria

1. WHEN batch size configured THEN it SHALL be respected
2. IF timeout configured THEN it SHALL trigger flush
3. WHEN coalescing disabled THEN events SHALL process individually
4. IF defaults used THEN they SHALL be sensible (5ms, 10 events)

## Non-Functional Requirements

### Performance
- Batching SHALL reduce FFI calls by > 50% under load
- Batch processing latency SHALL be < 1ms
- Memory for batch buffer SHALL be bounded
