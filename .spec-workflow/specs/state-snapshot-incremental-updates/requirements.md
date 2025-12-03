# Requirements Document

## Introduction

Every FFI call to publish engine state sends the full state snapshot. The Flutter UI renders 60fps, but the entire EngineState is serialized to JSON each frame even when only one modifier changed. This creates unnecessary serialization overhead and Flutter rebuilds.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Reduce FFI bandwidth
- **Efficiency**: Only send what changed
- **Responsiveness**: Faster UI updates

## Requirements

### Requirement 1: Delta Protocol

**User Story:** As a Flutter developer, I want incremental updates, so that the UI is efficient.

#### Acceptance Criteria

1. WHEN state changes THEN only delta SHALL be sent
2. IF full sync needed THEN complete state SHALL be available
3. WHEN delta applied THEN state SHALL match source
4. IF delta corrupted THEN full sync SHALL recover

### Requirement 2: Version Tracking

**User Story:** As a developer, I want state versioning, so that deltas are consistent.

#### Acceptance Criteria

1. WHEN state changes THEN version SHALL increment
2. IF delta received THEN version SHALL be validated
3. WHEN versions mismatch THEN full sync SHALL trigger
4. IF version overflows THEN wrap-around SHALL be handled

### Requirement 3: Efficient Encoding

**User Story:** As a developer, I want compact deltas, so that bandwidth is minimal.

#### Acceptance Criteria

1. WHEN delta encoded THEN size SHALL be < full state
2. IF no changes THEN empty delta SHALL be sent
3. WHEN many changes THEN full state MAY be more efficient
4. IF encoding fails THEN fallback to full state SHALL occur

## Non-Functional Requirements

### Performance
- Delta serialization SHALL be < 50μs
- Delta size SHALL be < 20% of full state on average
- Version check SHALL be O(1)
