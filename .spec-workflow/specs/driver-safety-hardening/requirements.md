# Requirements Document

## Introduction

The Windows driver has 8 unsafe blocks in `hook.rs` (562 LOC) with raw calls to `SetWindowsHookEx`, `GetAsyncKeyState`, and thread-local storage for event routing. The Linux driver uses evdev/uinput with complex error handling. This spec isolates unsafe code into minimal safety wrappers with thorough documentation and testing.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Safety First**: Minimal unsafe code with maximum safety guarantees
- **Performance > Features**: Safety wrappers must not impact latency
- **Reliability**: Drivers must handle all edge cases gracefully

Per tech.md: "Minimize unsafe blocks and encapsulate in safe wrappers"

## Requirements

### Requirement 1: Unsafe Isolation

**User Story:** As a developer, I want unsafe code isolated in dedicated safety modules, so that I can audit and reason about unsafe behavior.

#### Acceptance Criteria

1. WHEN unsafe code exists THEN it SHALL be in a `safety/` submodule
2. IF a function has unsafe internals THEN it SHALL expose a safe public API
3. WHEN adding unsafe code THEN it SHALL have a SAFETY comment explaining invariants
4. IF unsafe code is spread across functions THEN it SHALL be consolidated

### Requirement 2: Windows Hook Safety

**User Story:** As a Windows user, I want keyboard hooks to be robust, so that my keyboard never becomes unusable.

#### Acceptance Criteria

1. WHEN the hook is set THEN the handle SHALL be stored for cleanup
2. IF the hook callback panics THEN the hook SHALL remain functional
3. WHEN the application exits THEN the hook SHALL be removed
4. IF SetWindowsHookEx fails THEN a clear error SHALL be returned

### Requirement 3: Thread-Local Safety

**User Story:** As a developer, I want thread-local storage to be safe, so that events route correctly without races.

#### Acceptance Criteria

1. WHEN thread-local storage is used THEN it SHALL be encapsulated in a type
2. IF multiple threads access state THEN synchronization SHALL be used
3. WHEN thread-local is accessed THEN it SHALL never panic
4. IF initialization fails THEN a fallback SHALL be available

### Requirement 4: Linux evdev Safety

**User Story:** As a Linux user, I want device handling to be robust, so that device disconnection doesn't crash the engine.

#### Acceptance Criteria

1. WHEN a device disconnects THEN the driver SHALL return an error, not panic
2. IF device permissions are denied THEN a helpful error SHALL be shown
3. WHEN grabbing a device THEN exclusive access SHALL be verified
4. IF uinput creation fails THEN cleanup SHALL be performed

### Requirement 5: Error Recovery

**User Story:** As a user, I want drivers to recover from errors, so that temporary issues don't require restarts.

#### Acceptance Criteria

1. WHEN a temporary error occurs THEN the driver SHALL retry with backoff
2. IF recovery fails THEN the user SHALL be notified with suggested actions
3. WHEN errors occur THEN detailed logs SHALL be available at debug level
4. IF the keyboard is stuck THEN the emergency exit (Ctrl+Alt+Shift+Esc) SHALL work

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Safety modules wrap one unsafe API
- **Modular Design**: Each platform has its own safety module
- **Dependency Management**: Safe wrappers don't depend on each other
- **Clear Interfaces**: Safe APIs with unsafe internals

### Performance
- Safety wrappers SHALL add < 10 microseconds overhead
- Hook callbacks SHALL complete in < 100 microseconds
- No additional allocations in hot path

### Security
- Hooks SHALL only process keyboard events, not inject malware
- Drivers SHALL not elevate privileges
- Error messages SHALL not leak sensitive system info

### Reliability
- Drivers SHALL handle all documented error codes
- Panics in drivers SHALL be caught and logged
- Device disconnection SHALL be handled gracefully

### Usability
- Driver errors SHALL suggest solutions
- Platform-specific issues SHALL be documented
- Debugging drivers SHALL be possible with environment flags
