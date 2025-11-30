# Requirements Document

## Introduction

Device Discovery adds a first-run and on-demand workflow that maps physical keyboards to KeyRx’s blank-canvas model. It guides users through identifying new keyboards, captures physical positions (row/column) per key, and persists per-device profiles so remapping logic can be consistent across OS layouts. The feature must be CLI-first with hooks for future GUI integration, fail-safe, and fast enough to run during normal setup.

## Alignment with Product Vision

- Reinforces the “True Blank Canvas” pillar by treating every keyboard as a grid of buttons, not predefined key names.
- Supports “Safety First” by ensuring discovery cannot lock users out and keeps an emergency exit intact.
- Advances “CLI First” by making discovery fully operable headless before GUI surfaces.
- Enables “Testable Configs” by producing deterministic device profiles usable in simulation and replay flows.

## Requirements

### Requirement 1: Detect and prompt for unknown devices

**User Story:** As a power user connecting a keyboard for the first time, I want KeyRx to recognize it and offer to discover its layout so I can remap by physical position without manual config digging.

#### Acceptance Criteria

1. WHEN a keyboard with (vendor_id, product_id) not in the registry is connected THEN the CLI SHALL prompt to start discovery (with skip/defer option) within 2 seconds.
2. IF the user skips discovery THEN the system SHALL load the fallback default profile and continue running without blocking input.
3. WHEN a known device reconnects THEN the system SHALL load its saved profile automatically without re-prompting.

### Requirement 2: Guided physical layout capture

**User Story:** As a user running discovery, I want a guided sequence to press keys in order so KeyRx can map each physical position accurately.

#### Acceptance Criteria

1. WHEN discovery starts THEN the system SHALL display progress (current row/column or key count) and expected next key prompt.
2. IF a key press is ambiguous or duplicated THEN the system SHALL surface a clear error and require re-press without corrupting prior progress.
3. WHEN the user completes all required positions THEN the system SHALL summarize the discovered grid (rows, cols, unmapped keys) and ask for confirmation before saving.

### Requirement 3: Per-device profile persistence and retrieval

**User Story:** As a user with multiple keyboards, I want each device to keep its own discovered profile so switching devices is seamless.

#### Acceptance Criteria

1. WHEN discovery is confirmed THEN the system SHALL write a JSON profile under `~/.config/keyrx/devices/{vendor_id}_{product_id}.json` including scan_code→position mapping, dimensions, alias metadata, and timestamp.
2. WHEN the same device reconnects THEN the system SHALL load the stored profile and validate schema version before use; on version mismatch it SHALL fallback to default and prompt for re-discovery.
3. IF profile loading fails (corruption, missing file) THEN the system SHALL log the failure, fall back to default, and prompt for optional re-discovery without crashing.

### Requirement 4: Re-discovery and edits

**User Story:** As a user whose keyboard changed (firmware/layout), I want to rerun discovery or edit aliases without losing the original profile history.

#### Acceptance Criteria

1. WHEN the user invokes `keyrx discover --device {vendor_id}:{product_id}` THEN the system SHALL run discovery using existing dimensions as defaults and create a new versioned profile file on save.
2. WHEN aliases are edited (e.g., naming keys) THEN the system SHALL update the profile while preserving scan_code and position integrity.
3. IF re-discovery is canceled mid-process THEN the system SHALL keep the previous profile active and discard partial data.

### Requirement 5: Safety and interoperability

**User Story:** As a safety-conscious user, I want discovery to be non-intrusive and respect the emergency exit combo.

#### Acceptance Criteria

1. DURING discovery the emergency exit combo (Ctrl+Alt+Shift+Esc) SHALL always bypass the discovery flow and restore normal input handling.
2. WHEN discovery is running THEN the system SHALL not intercept non-discovery keyboards/mice; only the target device events are captured exclusively.
3. Discovery SHALL operate headless via CLI and expose hooks/events for GUI visualization without duplicating logic.

## Non-Functional Requirements

### Code Architecture and Modularity
- Single responsibility per file (driver detection, discovery flow controller, profile persistence, CLI command).
- Discovery logic reusable by both CLI and future GUI via shared Rust module.
- Clear trait-based boundaries for input sources and persistence to allow mocking in tests.

### Performance
- New device prompt within 2 seconds of connection.
- Full discovery session for 104-key layouts completes in < 90 seconds with responsive prompts.
- Profile load overhead under 5 ms on reconnect.

### Security
- No elevated privileges beyond existing driver requirements; never executes user-supplied scripts during discovery.
- Profiles stored under user config directory with sanitized filenames and validated JSON schema.

### Reliability
- Discovery flow recoverable after interruption; partial progress does not corrupt existing profiles.
- Corrupt profiles fall back to default without panics and emit actionable diagnostics.
- Deterministic replay hooks for discovery sessions to support testing.

### Usability
- CLI prompts are concise with progress indicators and clear retry guidance.
- Summary confirmation before saving changes.
- Default fallback ensures users retain keyboard control even if discovery fails.
