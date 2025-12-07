# Requirements Document

## Introduction
Mapping Screen Refresh introduces a clearer end-to-end flow for configuring devices and profiles in the Flutter desktop app. The feature renames the Editor screen to Profiles, adds a new Mapping screen for stepwise key assignment, and brings automatic, OS-appropriate saving of user profiles to the home directory. The goal is to make device naming, profile layout definition, and per-key mapping obvious, responsive, and reliable without explicit save actions.

## Alignment with Product Vision
- Supports the "Visual > Abstract" principle by making layout and key assignment legible and navigable.
- Aligns with "Progressive Complexity" by guiding users through device > profile > mapping in steps while keeping advanced flexibility.
- Upholds "CLI First, GUI Later" by persisting user data in a predictable filesystem location (`~/.keyrx` or `%USERPROFILE%/.keyrx`) shared with the engine/CLI.

## Requirements

### Requirement 1: Profiles screen rename and autosave
**User Story:** As a user, I want the Editor screen renamed to Profiles and to have my profile changes saved automatically, so that I never lose work and understand where to edit layouts.

#### Acceptance Criteria
1. WHEN the app renders the navigation, THEN the existing Editor entry SHALL be labeled “Profiles” (routing preserved).
2. WHEN a profile is created or edited, THEN the app SHALL auto-save without a visible save button.
3. WHEN saving on Windows, THEN data SHALL be written to `%USERPROFILE%/.keyrx`; WHEN on Linux, THEN data SHALL be written to `~/.keyrx`.
4. IF the target directory does not exist, THEN it SHALL be created before writing.
5. WHEN autosave succeeds or fails, THEN the user SHALL receive unobtrusive feedback (success/last saved time or error toast/banner).
6. WHEN autosave is in progress, THEN the UI SHALL indicate the state without blocking interaction.

### Requirement 2: Devices screen clarity
**User Story:** As a user, I want to discover devices and assign a friendly name on the Devices screen, so that I can recognize keyboards when mapping.

#### Acceptance Criteria
1. WHEN devices are listed, THEN each entry SHALL show vendor/product ids and the user-defined friendly name (or placeholder when unset).
2. WHEN the user edits a device name inline or in a dialog, THEN the name SHALL persist immediately and display updated state.
3. IF device discovery returns no devices, THEN the screen SHALL show an empty state with guidance to connect or refresh.
4. WHEN discovery is running, THEN a progress/refresh affordance SHALL be visible and non-blocking.

### Requirement 3: Profiles layout setup UX
**User Story:** As a user, I want to define the keyboard grid layout (rows/columns per row) in the Profiles screen before assigning keys, so that the mapping step matches my hardware.

#### Acceptance Criteria
1. WHEN creating or editing a profile, THEN the layout step SHALL allow setting row count and columns per row with immediate visual preview.
2. WHEN the layout changes, THEN the preview SHALL resize responsively to avoid narrow vertical stacking; minimum sensible height/width SHALL be enforced.
3. IF layout input is invalid (e.g., zero rows), THEN the UI SHALL prevent progression and show inline validation.

### Requirement 4: Mapping screen (new) for key assignment
**User Story:** As a user, I want a dedicated Mapping screen to assign actions/keys to each grid position after layout is defined, so that mapping is clear and scannable.

#### Acceptance Criteria
1. WHEN a profile is selected, THEN the Mapping screen SHALL display the defined grid with consistent cell sizing and readable labels.
2. WHEN assigning or editing a key/action, THEN an editor panel or dialog SHALL present common actions (emit, block, modifier/layer toggle, macro) with search.
3. WHEN many rows/columns exist, THEN the grid SHALL support zoom or density controls (e.g., compact/comfortable) to avoid vertical squish.
4. WHEN a mapping changes, THEN autosave SHALL trigger using the same home-directory location and feedback model as Requirement 1.
5. WHEN searching/filtering mappings, THEN matched cells SHALL be highlighted and non-matching cells de-emphasized.

### Requirement 5: UX consistency and accessibility
**User Story:** As a user, I want the mapping and profile flows to be easy to scan and operate, so that I can complete setup without confusion.

#### Acceptance Criteria
1. UI SHALL adopt consistent spacing, typography scale, and panel widths so the mapping area is not overly narrow.
2. Focus and keyboard navigation SHALL work for primary controls (device rename, layout inputs, mapping grid traversal).
3. Color usage SHALL meet accessible contrast for text and interactive elements.
4. Loading/error states SHALL be visually distinct without blocking the main layout.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Keep screen/page widgets focused (Devices, Profiles, Mapping) with shared sub-widgets for lists, grids, and editors.
- **Modular Design**: Extract autosave service and shared storage path resolver for reuse across profiles and mappings.
- **Dependency Management**: Use existing state management and repositories; avoid duplicating storage logic.
- **Clear Interfaces**: Define clear contracts for profile persistence and device naming.

### Performance
- Autosave SHALL complete within 300ms for typical profiles (under 200 mappings) and not block UI thread.
- Grid rendering SHALL keep 60fps on desktop targets for typical layouts (up to 20x10).

### Security
- Writes SHALL stay within the user home `.keyrx` directory; no elevated permissions.
- Input validation SHALL prevent invalid file names and reject path traversal.

### Reliability
- Autosave retries up to 3 times on transient errors with backoff.
- Errors are surfaced to the user with actionable guidance without losing in-memory edits.

### Usability
- Provide inline guidance/tooltips for layout and mapping steps.
- Maintain visible breadcrumbs or tabs for Devices → Profiles → Mapping.
- Avoid modal blocking except for focused mapping edits; provide escape/close affordances.
