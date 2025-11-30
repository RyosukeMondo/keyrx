# Requirements Document: MVP

## Introduction

This specification defines the Minimum Viable Product (MVP) for KeyRx - an advanced input remapping engine. The MVP establishes the foundational infrastructure enabling the core engine to be installed, run, load basic configurations, perform simple key remapping, and support CLI-first autonomous development and testing. Target platforms are Linux and Windows.

## Alignment with Product Vision

This MVP directly supports Phase 1 ("The Iron Core") from the product roadmap:
- **CLI First, GUI Later**: All features are CLI-exercisable for rapid trial, self-check, and AI agent autonomy
- **Performance > Features**: Establishes latency benchmarking from day one
- **Logic > Configuration**: Implements Rhai scripting foundation for programmable behavior
- **Dependency Injection**: All components testable via mock substitution

## Requirements

### REQ-1: Installation

**User Story:** As a developer, I want to install KeyRx from source, so that I can build and run the engine on my development machine.

#### Acceptance Criteria

1. WHEN a user clones the repository and runs `cargo build --release` THEN the system SHALL produce a `keyrx` binary
2. WHEN building on Linux (kernel 5.0+, x86_64) THEN the build SHALL complete without errors
3. WHEN building on Windows 10+ (x86_64) THEN the build SHALL complete without errors
4. IF required system dependencies are missing THEN the build system SHALL display clear error messages indicating what is needed

### REQ-2: CLI Execution

**User Story:** As a developer, I want to run KeyRx from the command line, so that I can start the engine and verify it works.

#### Acceptance Criteria

1. WHEN a user runs `keyrx --help` THEN the system SHALL display available commands and options
2. WHEN a user runs `keyrx --version` THEN the system SHALL display the current version number
3. WHEN a user runs `keyrx run` without arguments THEN the system SHALL start with default/empty configuration
4. WHEN a user runs `keyrx run --debug` THEN the system SHALL output debug information to stderr
5. WHEN the engine starts successfully THEN the system SHALL exit with code 0 on graceful shutdown (Ctrl+C)
6. IF the engine encounters a fatal error THEN the system SHALL exit with code 1 and display error details

### REQ-3: Configuration Loading

**User Story:** As a user, I want to load a Rhai configuration script, so that I can define my key remapping behavior.

#### Acceptance Criteria

1. WHEN a user runs `keyrx run --script path/to/config.rhai` THEN the system SHALL load and execute the specified script
2. WHEN a user runs `keyrx check path/to/config.rhai` THEN the system SHALL validate the script syntax without executing
3. IF the script file does not exist THEN the system SHALL display an error message and exit with code 1
4. IF the script contains syntax errors THEN `keyrx check` SHALL display line/column error location and exit with code 2
5. WHEN `keyrx check` succeeds THEN the system SHALL display "OK" and exit with code 0
6. WHEN loading a valid script THEN the system SHALL execute `on_init()` hook if defined

### REQ-4: Basic Key Remapping

**User Story:** As a user, I want to remap one key to another, so that I can customize my keyboard behavior.

#### Acceptance Criteria

1. WHEN a Rhai script defines `remap("A", "B")` THEN pressing A SHALL output B
2. WHEN a Rhai script defines `block("CapsLock")` THEN pressing CapsLock SHALL produce no output
3. WHEN a Rhai script defines `pass()` as default THEN undefined keys SHALL pass through unchanged
4. WHEN processing a key event THEN the latency overhead SHALL be measurable via `keyrx bench`
5. IF no remapping is defined for a key THEN the key event SHALL pass through to the OS unchanged
6. WHEN the engine processes key events THEN both key-down and key-up events SHALL be handled correctly

### REQ-5: Event Simulation (Headless Testing)

**User Story:** As a developer, I want to simulate key events without real keyboard input, so that I can test remapping logic autonomously.

#### Acceptance Criteria

1. WHEN a user runs `keyrx simulate --input "A" --script config.rhai` THEN the system SHALL process the simulated key event
2. WHEN simulation completes THEN the system SHALL output the resulting action (remapped key, blocked, passed)
3. WHEN a user runs `keyrx simulate --input "A,B,C"` THEN the system SHALL process multiple keys in sequence
4. WHEN a user specifies `--json` flag THEN output SHALL be in JSON format for programmatic parsing
5. WHEN simulation mode is active THEN NO real keyboard hooks SHALL be installed
6. IF the script modifies engine state THEN subsequent simulated keys SHALL reflect that state

### REQ-6: Self-Diagnostics

**User Story:** As a user, I want to run diagnostics, so that I can verify my system is correctly configured for KeyRx.

#### Acceptance Criteria

1. WHEN a user runs `keyrx doctor` THEN the system SHALL check platform-specific requirements
2. WHEN on Linux THEN `keyrx doctor` SHALL verify uinput/evdev access permissions
3. WHEN on Windows THEN `keyrx doctor` SHALL verify keyboard hook registration capability
4. WHEN all checks pass THEN the system SHALL display "All checks passed" and exit with code 0
5. IF any check fails THEN the system SHALL display the failing check with remediation guidance
6. WHEN a user specifies `--verbose` THEN detailed diagnostic information SHALL be displayed

### REQ-7: Latency Benchmarking

**User Story:** As a developer, I want to benchmark input latency, so that I can ensure performance meets requirements.

#### Acceptance Criteria

1. WHEN a user runs `keyrx bench` THEN the system SHALL run latency benchmarks
2. WHEN benchmarking completes THEN the system SHALL display min/max/mean/p99 latency statistics
3. WHEN a user specifies `--iterations N` THEN the benchmark SHALL run N iterations
4. WHEN a user specifies `--json` THEN output SHALL be in JSON format
5. WHEN latency exceeds 1ms mean THEN the output SHALL include a warning
6. WHEN benchmarking THEN the system SHALL measure script execution time separately from event processing

### REQ-8: Cross-Platform Build

**User Story:** As a developer, I want to build KeyRx for both Linux and Windows from the same codebase, so that I can support multiple platforms.

#### Acceptance Criteria

1. WHEN running `cargo build --release` on Linux THEN a Linux binary SHALL be produced
2. WHEN running `cargo build --release` on Windows THEN a Windows binary SHALL be produced
3. WHEN cross-compiling with `cargo build --target x86_64-pc-windows-gnu` on Linux THEN a Windows binary SHALL be produced
4. WHEN building THEN platform-specific drivers SHALL be conditionally compiled via Cargo features
5. WHEN the binary is built THEN it SHALL have no external runtime dependencies
6. IF a platform is unsupported THEN the build SHALL fail with a clear error message

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each module handles one concern (engine, scripting, drivers, CLI)
- **Modular Design**: OS drivers implement `InputSource` trait for interchangeability
- **Dependency Injection**: All external dependencies injected for testability
- **Clear Interfaces**: Traits define contracts between engine, scripting, and OS layers

### Performance
- Input-to-output latency overhead < 1ms (measured via `keyrx bench`)
- Memory usage < 50MB idle
- Startup time < 500ms to operational state

### Security
- Rhai scripts are sandboxed (no filesystem, network, or system call access)
- Scripts cannot crash the engine (panics are caught and logged)
- No elevated privileges required for core functionality

### Reliability
- Engine survives malformed Rhai scripts without crashing
- Graceful shutdown on SIGINT/SIGTERM (Linux) or Ctrl+C (Windows)
- All key events are processed or explicitly dropped (no silent failures)

### Testability
- 80% code coverage minimum for core modules
- All CLI commands testable via simulated input
- Mock implementations available for InputSource, ScriptRuntime, StateStore traits

### Usability
- All CLI commands support `--help` with usage examples
- Error messages include actionable remediation steps
- JSON output mode (`--json`) for AI agent consumption
