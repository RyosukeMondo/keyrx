# Requirements Document

## Introduction

This feature adds comprehensive keyboard layout support to the keyrx_ui keyboard visualizer component. Currently, only ANSI 104 is implemented while ISO 105, JIS 109, HHKB, and NUMPAD are placeholders. This enhancement will provide accurate visual representations for the most popular keyboard layouts worldwide, covering approximately 80% of users.

## Alignment with Product Vision

This feature aligns with keyrx's goals of:
- **Cross-platform flexibility**: Supporting diverse keyboard hardware without firmware constraints
- **Professional power user support**: Enabling accurate visualization for various keyboard form factors
- **AI-first verification**: Providing deterministic layout data that can be validated programmatically

## Requirements

### Requirement 1: Full-Size Layouts

**User Story:** As a user with a full-size keyboard, I want to see my exact keyboard layout visualized, so that I can accurately configure key mappings.

#### Acceptance Criteria

1. WHEN user selects ANSI 104 layout THEN system SHALL display all 104 keys with correct positioning
2. WHEN user selects ISO 105 layout THEN system SHALL display the ISO Enter key shape (vertical) and additional key left of Z
3. WHEN user selects JIS 109 layout THEN system SHALL display Japanese-specific keys (Henkan, Muhenkan, Katakana/Hiragana, Yen)

### Requirement 2: Tenkeyless (TKL) Layouts

**User Story:** As a user with a TKL keyboard, I want to see my keyboard without the numpad, so that the visualizer matches my actual hardware.

#### Acceptance Criteria

1. WHEN user selects ANSI 87 (TKL) layout THEN system SHALL display 87 keys without numpad section
2. WHEN user selects ISO 88 (TKL) layout THEN system SHALL display 88 keys with ISO Enter without numpad
3. IF user has both TKL and full-size keyboards THEN system SHALL allow different layouts per device

### Requirement 3: Compact Layouts

**User Story:** As a user with a compact keyboard (60%/65%/75%), I want accurate visualization of my reduced key layout, so that I can configure layers effectively.

#### Acceptance Criteria

1. WHEN user selects 60% layout THEN system SHALL display ~61 keys (no F-row, no nav cluster, no arrows, no numpad)
2. WHEN user selects 65% layout THEN system SHALL display ~68 keys (no F-row, no numpad, but includes arrows and minimal nav)
3. WHEN user selects 75% layout THEN system SHALL display ~84 keys (F-row present, compressed nav cluster, no numpad)

### Requirement 4: Specialized Layouts

**User Story:** As a user with specialized hardware, I want to see niche layouts supported, so that keyrx works with my specific keyboard.

#### Acceptance Criteria

1. WHEN user selects HHKB layout THEN system SHALL display 60-key HHKB-specific layout with split backspace and Fn in place of Ctrl
2. WHEN user selects 96% layout THEN system SHALL display compact full-size layout (~96 keys)
3. WHEN user selects Numpad layout THEN system SHALL display standalone 17-key numpad

### Requirement 5: Layout Selection UI

**User Story:** As a user, I want to easily browse and select from available layouts, so that I can find my keyboard type quickly.

#### Acceptance Criteria

1. WHEN user opens layout selector THEN system SHALL display layouts grouped by category (Full-size, TKL, Compact, Specialized)
2. WHEN user hovers over a layout option THEN system SHALL show preview thumbnail and key count
3. IF layout is selected THEN system SHALL persist the selection for the current device/profile

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each layout JSON file contains only layout data for one specific layout
- **Modular Design**: Layout files are separate from parsing/rendering logic
- **Dependency Management**: No external dependencies for layout data (pure JSON)
- **Clear Interfaces**: Layout data follows consistent JSON schema across all layouts

### Performance
- Layout JSON files should be under 20KB each
- Layout parsing should complete in under 5ms
- Layout selection should not cause visible re-render lag

### Security
- Layout JSON files are static and validated at build time
- No user-provided layout data execution

### Reliability
- All layouts must be validated against schema before build
- Missing layout gracefully falls back to ANSI 104

### Usability
- Layout names should use common terminology (ANSI, ISO, JIS, TKL, etc.)
- Visual preview should accurately represent physical keyboard appearance
