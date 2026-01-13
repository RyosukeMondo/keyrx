# Requirements Document

## Introduction

The sophisticated-key-palette feature transforms the existing basic key palette into a comprehensive, professional-grade key selection interface inspired by industry-leading keyboard configurators (QMK Configurator and VIA). This feature enables users to discover, search, and assign keys from a catalog of 200+ keycodes across 7 logical categories, with advanced features including fuzzy search, physical key capture, custom keycode input, and drag-and-drop visual feedback.

The value to users is threefold:
1. **Discoverability**: Comprehensive categorization makes it easy to find specialized keys (media, layer functions, macros) without memorizing keycodes
2. **Efficiency**: Recent/favorite keys, fuzzy search, and physical key capture dramatically reduce time spent assigning common keys
3. **Power User Support**: "Any" category with custom keycode input enables advanced users to leverage the full QMK-compatible syntax without UI limitations

## Alignment with Product Vision

This feature directly supports keyrx's product vision in multiple dimensions:

**AI-First Verification (Product Principle #1)**
- Comprehensive key definitions database creates a single source of truth for all available keycodes, enabling AI agents to validate configurations against known keys
- Structured key metadata (id, label, category, aliases, description) provides machine-readable documentation for AI-driven configuration generation
- Custom keycode validation ensures only syntactically correct QMK expressions are accepted, preventing runtime errors

**Complete Determinism (Product Principle #2)**
- Physical key capture maps DOM keyboard events deterministically to key IDs, enabling reproducible configuration workflows
- Recent/favorite key persistence to localStorage provides consistent state across sessions

**Extreme Configuration Flexibility (Key Feature #2)**
- Exposes all 255 custom modifiers and 255 lock keys through the UI in an organized manner
- Layer function keys (MO, TO, TG, OSL, LT) provide full QMK-style layer control
- "Any" category allows power users to input arbitrary valid QMK syntax for advanced use cases

**Real-Time Simulation & Preview (Key Feature #6)**
- Enhanced drag-and-drop visual feedback (drop zones, drag preview, success animations) provides immediate visual confirmation during remapping
- Grid/list view toggle adapts to different user workflows (compact vs. detailed)

**User Experience Metrics (Success Metrics)**
- Supports <5 second configuration change time through quick access (recent/favorites), fast search, and physical key capture
- Rich tooltips and descriptions reduce learning curve for complex features (layer functions, mod-tap combinations)

## Requirements

### Requirement 1: VIA-Style Key Categorization

**User Story:** As a keyboard customization user, I want keys organized into logical categories with subcategories, so that I can quickly find specialized keys without scrolling through a flat list of 200+ keycodes.

#### Acceptance Criteria

1. WHEN the user views the key palette THEN the system SHALL display 7 category tabs: Basic, Modifiers, Media, Macro, Layers, Special, Any
2. WHEN the user selects the "Basic" category THEN the system SHALL display subcategories for Letters, Numbers, Punctuation, and Navigation keys
3. WHEN the user selects the "Modifiers" category THEN the system SHALL display standard modifiers (Ctrl, Shift, Alt, Meta) and custom modifiers (MD_00 through MD_254)
4. WHEN the user selects the "Media" category THEN the system SHALL display volume controls, playback controls, and brightness controls
5. WHEN the user selects the "Macro" category THEN the system SHALL display user-defined macro slots (M0 through M15)
6. WHEN the user selects the "Layers" category THEN the system SHALL display layer function keys (MO, TO, TG, OSL, LT) for layers 0-15
7. WHEN the user selects the "Special" category THEN the system SHALL display mouse keys, system keys, and unicode entry keys
8. WHEN the user selects the "Any" category THEN the system SHALL display a custom keycode input field
9. IF a key exists in the current palette THEN the system SHALL ensure it is categorized and accessible in the new design

### Requirement 2: Comprehensive Key Definitions Database

**User Story:** As a keyboard power user, I want access to all QMK-compatible keycodes with metadata (descriptions, aliases), so that I can leverage the full feature set of the keyrx remapping system.

#### Acceptance Criteria

1. WHEN the system initializes THEN the system SHALL load a key definitions database containing at least 250 keys
2. IF a key is defined THEN the system SHALL include properties: id, label, category, subcategory, description, and aliases
3. WHEN a user searches for "KC_A" or "VK_A" THEN the system SHALL return the same key (support for both QMK and Windows virtual key prefixes)
4. WHEN the system displays a key THEN the system SHALL show the label prominently and the id as secondary text
5. IF a key supports modifier combinations THEN the system SHALL include examples in the description (e.g., "LCTL(KC_C) for Ctrl+C")
6. WHEN keys F1 through F24 are requested THEN the system SHALL provide all 24 function keys
7. WHEN numpad keys are requested THEN the system SHALL provide all numpad keys (0-9, operators, Enter, Period)
8. WHEN media keys are requested THEN the system SHALL provide Play/Pause, Next, Previous, Volume Up/Down/Mute, Brightness Up/Down
9. WHEN system keys are requested THEN the system SHALL provide Power, Sleep, Wake, Calculator, Browser, Mail

### Requirement 3: Fuzzy Search with Highlighting

**User Story:** As a user unfamiliar with keycode naming conventions, I want to search for keys using partial or fuzzy matching, so that I can quickly find keys without knowing exact names.

#### Acceptance Criteria

1. WHEN the user types text into the search input THEN the system SHALL filter displayed keys in real-time
2. WHEN the user types "ctrl" THEN the system SHALL match "Left Ctrl", "Right Ctrl", "LCTL", "KC_LCTL", and any key with "ctrl" in the description
3. WHEN a search matches a key THEN the system SHALL highlight the matching text in the label, id, or description
4. WHEN a search query returns results THEN the system SHALL display the result count (e.g., "8 keys found")
5. WHEN a search query returns no results THEN the system SHALL display a "No results found" message with suggestions (e.g., "Try: modifiers, media, layers")
6. WHEN the user presses arrow keys in search mode THEN the system SHALL navigate through filtered results using keyboard
7. WHEN the user presses Enter on a highlighted key THEN the system SHALL select that key
8. IF the search input is cleared THEN the system SHALL restore the full key list for the active category

### Requirement 4: Recent and Favorite Keys

**User Story:** As a user who frequently assigns the same keys, I want quick access to recently used and favorite keys, so that I can complete common assignments without searching or navigating categories.

#### Acceptance Criteria

1. WHEN the user assigns a key THEN the system SHALL add it to the "Recent" list (maximum 10 keys, FIFO)
2. WHEN the user opens the key palette THEN the system SHALL display the "Recent" section at the top, above category tabs
3. WHEN the user clicks the star icon on a key THEN the system SHALL add it to the "Favorites" list
4. WHEN the user clicks the star icon on a favorited key THEN the system SHALL remove it from the "Favorites" list
5. WHEN the system restarts THEN the system SHALL restore recent and favorite keys from localStorage
6. IF the "Recent" list is empty THEN the system SHALL display "No recent keys yet"
7. IF the "Favorites" list is empty THEN the system SHALL display "Click ★ to add favorites"
8. IF localStorage write fails THEN the system SHALL continue operation without throwing errors (graceful degradation)

### Requirement 5: Custom Keycode Input ("Any" Category)

**User Story:** As an advanced user, I want to input custom keycodes using QMK syntax, so that I can use advanced features like mod-tap (MT), layer-tap (LT), and custom combinations not represented in the UI.

#### Acceptance Criteria

1. WHEN the user selects the "Any" category THEN the system SHALL display a text input field with placeholder "Enter QMK keycode (e.g., KC_A, LCTL(KC_C), MO(1))"
2. WHEN the user types a valid keycode (e.g., "KC_A") THEN the system SHALL display a green checkmark icon
3. WHEN the user types an invalid keycode (e.g., "INVALID") THEN the system SHALL display a red X icon and error message "Invalid QMK syntax"
4. WHEN the user types a valid QMK function (e.g., "LCTL(KC_C)") THEN the system SHALL validate the syntax and show green checkmark
5. WHEN the user types a valid layer function (e.g., "MO(1)", "LT(2,KC_SPC)") THEN the system SHALL validate and show green checkmark
6. WHEN the user clicks "Apply" with valid input THEN the system SHALL create a temporary key entry and trigger onKeySelect
7. IF the user clicks "Apply" with invalid input THEN the system SHALL display error and prevent selection
8. WHEN the user hovers over the help icon THEN the system SHALL display a tooltip explaining QMK syntax with examples

### Requirement 6: Visual Key Display with Icons and Tooltips

**User Story:** As a visual learner, I want keys to display with icons, color coding, and rich tooltips, so that I can quickly distinguish key types and understand their function.

#### Acceptance Criteria

1. WHEN a key is displayed THEN the system SHALL show an icon (if applicable), label, and secondary label (alias or id)
2. WHEN a key is in the "Basic" category THEN the system SHALL show a keyboard icon and neutral color border
3. WHEN a key is in the "Modifiers" category THEN the system SHALL show a modifier icon and cyan color border
4. WHEN a key is in the "Media" category THEN the system SHALL show a media icon (play, volume, brightness) and green color border
5. WHEN a key is in the "Layers" category THEN the system SHALL show a layers icon and yellow color border
6. WHEN the user hovers over a key THEN the system SHALL display a tooltip with the full description
7. IF a key has no description THEN the system SHALL display the id as the tooltip
8. WHEN a key has multiple purposes (e.g., Mod-Tap) THEN the system SHALL include usage examples in the tooltip

### Requirement 7: Grid and List View Toggle

**User Story:** As a user with different workflow preferences, I want to toggle between compact grid view and detailed list view, so that I can choose the layout that suits my current task.

#### Acceptance Criteria

1. WHEN the user views the key palette THEN the system SHALL display a view toggle button in the palette header
2. WHEN the user clicks the grid icon THEN the system SHALL display keys in a 4-column grid layout with compact spacing
3. WHEN the user clicks the list icon THEN the system SHALL display keys in a single-column list with full descriptions visible
4. WHEN the user toggles the view THEN the system SHALL persist the preference to localStorage
5. WHEN the system restarts THEN the system SHALL restore the last selected view mode from localStorage
6. IF localStorage is unavailable THEN the system SHALL default to grid view
7. WHEN the palette is in grid view THEN the system SHALL show only labels and icons (descriptions in tooltips)
8. WHEN the palette is in list view THEN the system SHALL show labels, icons, and full descriptions inline

### Requirement 8: Enhanced Drag-and-Drop Feedback

**User Story:** As a user assigning keys via drag-and-drop, I want clear visual feedback during dragging, so that I know where I can drop the key and whether the drop succeeded.

#### Acceptance Criteria

1. WHEN the user starts dragging a key THEN the system SHALL display a semi-transparent drag preview following the cursor
2. WHEN a dragged key is over a valid drop target THEN the system SHALL highlight the target with a blue ring (ring-2 ring-blue-400)
3. WHEN a dragged key is over an invalid drop target THEN the system SHALL highlight the target with a red ring (ring-2 ring-red-400)
4. WHEN the user drops a key on a valid target THEN the system SHALL play a brief scale animation on the target key
5. WHEN the user drops a key on an invalid target THEN the system SHALL return the key to its original position with animation
6. IF drag-and-drop is not supported by the browser THEN the system SHALL fall back to click-to-select mode
7. WHEN the drag preview displays THEN the system SHALL include the key label and icon
8. IF the user presses Escape during drag THEN the system SHALL cancel the drag operation

### Requirement 9: Physical Key Capture

**User Story:** As a user who finds it faster to press keys than search for them, I want to capture a physical key press to select that key in the palette, so that I can quickly assign keys without navigating the UI.

#### Acceptance Criteria

1. WHEN the user clicks "Capture Key" button THEN the system SHALL enter key capture mode and display a modal "Press any key..."
2. WHEN the user presses a physical key in capture mode THEN the system SHALL map the DOM keyboard event to a key id
3. WHEN a key is captured THEN the system SHALL display a confirmation "Captured: [Key Label] - Use this key?" with Confirm and Cancel buttons
4. WHEN the user clicks Confirm THEN the system SHALL select the captured key and close the modal
5. WHEN the user clicks Cancel or presses Escape THEN the system SHALL exit capture mode without selecting a key
6. IF the user presses a modifier key (Ctrl, Shift, Alt) THEN the system SHALL capture the modifier itself, not trigger browser shortcuts
7. IF the user presses Tab or Escape during normal use THEN the system SHALL NOT capture these keys (only capture when in capture mode)
8. WHEN capture mode is active THEN the system SHALL prevent all default browser behavior for captured keys

### Requirement 10: Layer Function Keys

**User Story:** As an advanced user leveraging keyrx's layer system, I want access to all layer functions (MO, TO, TG, OSL, LT), so that I can configure complex layer behaviors like QMK firmware.

#### Acceptance Criteria

1. WHEN the user selects the "Layers" category THEN the system SHALL display sections for MO, TO, TG, OSL, and LT functions
2. WHEN the user views MO (Momentary) keys THEN the system SHALL display MO(0) through MO(15) with descriptions "Hold to activate Layer N"
3. WHEN the user views TO (Toggle-To) keys THEN the system SHALL display TO(0) through TO(15) with descriptions "Tap to switch to Layer N"
4. WHEN the user views TG (Toggle) keys THEN the system SHALL display TG(0) through TG(15) with descriptions "Tap to toggle Layer N on/off"
5. WHEN the user views OSL (One-Shot Layer) keys THEN the system SHALL display OSL(0) through OSL(15) with descriptions "Next key only on Layer N"
6. WHEN the user views LT (Layer-Tap) section THEN the system SHALL display a special entry with layer selector (0-15) and key selector
7. IF the user configures LT(2, KC_SPC) THEN the system SHALL describe it as "Hold: Layer 2, Tap: Space"
8. WHEN layer function keys are generated THEN the system SHALL create them programmatically (not hardcoded) for layers 0-15
9. IF the user hovers over a layer function key THEN the system SHALL display a tooltip explaining the behavior with examples

### Requirement 11: Comprehensive Unit Tests

**User Story:** As a developer, I want comprehensive tests for all new key palette features, so that the palette remains reliable through future refactoring and feature additions.

#### Acceptance Criteria

1. WHEN the test suite runs THEN the system SHALL test category navigation and verify correct keys render per category
2. WHEN search tests run THEN the system SHALL test fuzzy matching, highlighting, keyboard navigation, and empty state
3. WHEN drag-and-drop tests run THEN the system SHALL test drag preview display, drop zone highlighting, and success animation
4. WHEN favorites tests run THEN the system SHALL test adding, removing, and localStorage persistence
5. WHEN custom input tests run THEN the system SHALL test valid QMK syntax acceptance and invalid syntax rejection
6. WHEN physical capture tests run THEN the system SHALL test key capture, modifier handling, and escape cancellation
7. IF localStorage is unavailable in tests THEN the system SHALL mock localStorage and test graceful degradation
8. WHEN all tests pass THEN the system SHALL achieve at least 80% code coverage for new components

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**:
  - `KeyPalette.tsx` handles palette layout, category navigation, and search UI
  - `keyDefinitions.ts` is the single source of truth for all key metadata
  - `KeyPaletteItem.tsx` handles individual key display and interactions
  - Search logic isolated in utility functions (`searchKeys`, `highlightMatches`)

- **Modular Design**:
  - Key definitions exported as structured data (`getKeysByCategory`, `searchKeys`, `getKeyById`)
  - Drag-and-drop hooks reuse existing @dnd-kit setup
  - localStorage access abstracted through hooks (`useLocalStorage`, `useFavorites`)

- **Dependency Management**:
  - Minimize dependencies: use existing Lucide icons, Tailwind, @dnd-kit
  - No external fuzzy search library (keep bundle small with custom scoring)
  - Leverage existing `keyCodeMapping.ts` for key code translation

- **Clear Interfaces**:
  - `PaletteKey` interface extended with subcategory, aliases, icon
  - `KeyDefinition` interface for database entries
  - `onKeySelect` callback remains unchanged for backward compatibility

### Performance

- **Search Responsiveness**: Fuzzy search SHALL complete within 50ms for 250+ keys
- **Render Performance**: Grid view SHALL render 200+ keys without janky scrolling (60fps target)
- **localStorage I/O**: Recent/favorite persistence SHALL NOT block UI (use async writes)
- **Drag Preview**: Drag preview SHALL render within 16ms of drag start (1 frame at 60fps)

### Security

- **Custom Input Validation**: Reject keycodes containing script injection attempts (no eval, no innerHTML)
- **localStorage Sanitization**: Validate JSON structure before parsing localStorage data
- **XSS Prevention**: All user input (custom keycodes) displayed via text nodes, not HTML

### Reliability

- **localStorage Failures**: Graceful degradation if localStorage is unavailable (in-memory fallback)
- **Keyboard Event Edge Cases**: Handle browser differences in event.code and event.key
- **Category Integrity**: All existing keys MUST be categorized (no orphaned keys)
- **Backward Compatibility**: Existing `onKeySelect` callback signature unchanged

### Usability

- **Keyboard Accessibility**: All features (search, category tabs, key selection) accessible via keyboard
- **Touch Targets**: Minimum 44px × 44px touch targets for mobile/tablet
- **Color Contrast**: WCAG 2.2 Level AA compliance (4.5:1 for normal text, 3:0:1 for large text)
- **Focus Indicators**: Clear focus rings on all interactive elements
- **Empty States**: Helpful messages for empty recent/favorites, no search results
- **Error Messages**: Specific, actionable feedback for invalid custom keycodes
