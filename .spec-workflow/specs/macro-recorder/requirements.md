# Requirements Document

## Introduction

The Macro Recorder allows users to record keyboard sequences and save them as macros that can be triggered by hotkeys. Currently, creating complex key sequences requires manually writing Rhai code with precise timing, which is tedious and error-prone.

By providing a macro recorder that captures real keyboard input with precise timestamps and converts it to Rhai macro definitions, we make it easy for users to create complex sequences (text snippets, shortcuts, game combos) by demonstration rather than coding.

## Requirements

### Requirement 1: Macro Recording

**User Story:** As a user creating a complex key sequence, I want to record my actual key presses with timing, so that I can replay them exactly as performed.

**Acceptance Criteria:**
1. WHEN user clicks "Record Macro" THEN the daemon SHALL enter recording mode and capture all key events
2. WHEN recording THEN the UI SHALL show a red "Recording..." indicator with elapsed time
3. WHEN user presses the stop hotkey (e.g., Ctrl+Shift+R) THEN recording SHALL stop and captured events SHALL be displayed
4. WHEN events are captured THEN they SHALL include timestamps (relative to recording start)
5. WHEN recording completes THEN the UI SHALL show the captured sequence with a preview

### Requirement 2: Macro Editing

**User Story:** As a user who recorded a macro with mistakes, I want to edit the captured sequence before saving, so that I can fix errors without re-recording.

**Acceptance Criteria:**
1. WHEN a macro is recorded THEN users SHALL be able to add/remove/edit individual events
2. WHEN events are edited THEN timestamps SHALL be adjustable via numeric input or visual timeline
3. WHEN user adds a delay THEN they SHALL specify duration in milliseconds
4. WHEN user removes an event THEN subsequent timestamps SHALL optionally auto-adjust
5. WHEN edits are made THEN a live preview SHALL show the generated Rhai code

### Requirement 3: Macro Playback Testing

**User Story:** As a user testing my macro, I want to play it back in the simulator before saving, so that I can verify it works correctly.

**Acceptance Criteria:**
1. WHEN user clicks "Test Macro" THEN the simulator SHALL replay the sequence
2. WHEN playback occurs THEN the UI SHALL show progress through the sequence
3. WHEN playback completes THEN the UI SHALL display the results (expected vs actual output)
4. WHEN playback fails THEN error messages SHALL indicate which event caused the issue
5. WHEN user is satisfied THEN they SHALL be able to save the macro to their configuration

### Requirement 4: Macro Storage and Triggering

**User Story:** As a user with saved macros, I want to assign them to hotkeys, so that I can trigger them during normal use.

**Acceptance Criteria:**
1. WHEN user saves a macro THEN they SHALL assign a trigger hotkey (e.g., Ctrl+Shift+M)
2. WHEN a macro is saved THEN it SHALL be added to the active configuration
3. WHEN the trigger hotkey is pressed THEN the macro SHALL replay the recorded sequence
4. WHEN multiple macros exist THEN they SHALL be listed in the UI with their trigger keys
5. WHEN user deletes a macro THEN it SHALL be removed from the configuration

### Requirement 5: Common Macro Templates

**User Story:** As a new user, I want pre-built macro templates (text snippets, email signatures), so that I can quickly create common macros.

**Acceptance Criteria:**
1. WHEN user clicks "New Macro from Template" THEN they SHALL see a list of templates
2. WHEN user selects "Text Snippet" THEN they SHALL enter text and it SHALL be converted to key sequences
3. WHEN user selects "Email Signature" THEN a multi-line text input SHALL generate the full sequence
4. WHEN templates are used THEN they SHALL generate valid Rhai macro code
5. WHEN user customizes a template THEN they SHALL be able to edit the generated macro

## Non-Functional Requirements

- **Architecture**: Daemon records events via broadcast channel, UI displays and edits, generates Rhai
- **Performance**: Recording <5ms latency, playback maintains original timing (±10ms accuracy)
- **Code Quality**: File sizes ≤300 lines (components), TypeScript strict mode
- **Accessibility**: WCAG 2.1 AA (keyboard shortcuts, screen reader announcements)

## Dependencies

- Leverage: Daemon event capture (already exists), WASM simulator (for testing)
- New: Timeline UI component for visual editing

## Sources

- [Keyboard Shortcuts design pattern](https://ui-patterns.com/patterns/keyboard-shortcuts)
- [Best Macros Software for 2025](https://textexpander.com/blog/best-macros-software)
- [Macro Recorder patterns](https://www.keysmith.app/)
