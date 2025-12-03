# Requirements Document

## Introduction

KeyRx has 749 `unwrap/expect` calls and 13 `panic!/todo!` macros in critical input remapping code. For "Tier 0" software where crashes make the keyboard unusable, this is unacceptable. A single panic in the hook callback leaves the user unable to type. This spec systematically removes panics from the critical path and adds recovery mechanisms.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Safety First**: Keyboard must never become unusable
- **Reliability**: Graceful degradation over crashes
- **User Trust**: Users trust KeyRx with their primary input device

Per tech.md: "Panics in drivers SHALL be caught and logged"

## Requirements

### Requirement 1: Critical Path Audit

**User Story:** As a user, I want the keyboard to never become stuck, so that I can always type.

#### Acceptance Criteria

1. WHEN code is in the critical path THEN it SHALL NOT contain unwrap/expect
2. IF an error occurs in critical path THEN it SHALL fallback gracefully
3. WHEN panic occurs THEN the hook SHALL remain functional
4. IF recovery fails THEN emergency exit SHALL still work

### Requirement 2: Error Propagation

**User Story:** As a developer, I want proper error types, so that failures are explicit and handled.

#### Acceptance Criteria

1. WHEN a function can fail THEN it SHALL return Result<T, E>
2. IF an error is unrecoverable THEN it SHALL be a CriticalError
3. WHEN errors propagate THEN context SHALL be preserved
4. IF error occurs THEN appropriate fallback SHALL execute

### Requirement 3: Panic Recovery

**User Story:** As a user, I want panics to be recovered, so that a bug doesn't break my keyboard.

#### Acceptance Criteria

1. WHEN code may panic THEN catch_unwind SHALL wrap it
2. IF panic is caught THEN state SHALL be recovered
3. WHEN panic occurs THEN it SHALL be logged with backtrace
4. IF repeated panics occur THEN circuit breaker SHALL activate

### Requirement 4: Fallback Behavior

**User Story:** As a user, I want graceful degradation, so that partial failures don't break everything.

#### Acceptance Criteria

1. WHEN driver fails THEN passthrough mode SHALL activate
2. IF config loading fails THEN defaults SHALL be used
3. WHEN script errors THEN key SHALL pass through
4. IF state is corrupted THEN reset SHALL occur

## Non-Functional Requirements

### Code Architecture and Modularity
- **Result Types**: All fallible operations return Result
- **Error Hierarchy**: CriticalError for unrecoverable, RecoverableError for others
- **Panic Guards**: Wrapper types for panic-prone code

### Reliability
- Critical path SHALL have 0 unwrap/expect calls
- Panic recovery SHALL complete in < 1ms
- Fallback behavior SHALL be tested
- Emergency exit SHALL work in all scenarios

### Observability
- Panics SHALL be logged with full backtrace
- Recovery events SHALL be reported to UI
- Circuit breaker state SHALL be visible
