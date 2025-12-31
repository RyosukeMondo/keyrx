# Requirements Document

## Introduction

The Profile Management System allows users to save, load, and switch between multiple keyboard remapping configurations. Currently, users can only have one active configuration at a time, requiring manual file editing to switch between different setups (e.g., work vs gaming vs coding profiles).

By providing profile management with quick switching, users can maintain separate configurations for different contexts and switch between them instantly via the web UI or CLI.

## Requirements

### Requirement 1: Profile Storage

**User Story:** As a user with multiple use cases, I want to save my current configuration as a named profile, so that I can switch between work and gaming setups quickly.

**Acceptance Criteria:**
1. WHEN user clicks "Save Profile" THEN a dialog SHALL prompt for profile name and description
2. WHEN user saves a profile THEN it SHALL be stored as a .krx file in ~/.config/keyrx/profiles/
3. WHEN profiles are listed THEN they SHALL show name, description, last modified date
4. WHEN user saves with an existing name THEN a confirmation dialog SHALL ask to overwrite
5. WHEN a profile is saved THEN it SHALL include full configuration (layers, modifiers, mappings)

### Requirement 2: Profile Switching

**User Story:** As a user switching contexts, I want to activate a different profile with one click, so that I don't need to manually reload configurations.

**Acceptance Criteria:**
1. WHEN user selects a profile THEN it SHALL be loaded and applied to the daemon within 500ms
2. WHEN a profile is activated THEN the daemon SHALL reload with zero downtime (seamless switch)
3. WHEN switching fails THEN the previous profile SHALL remain active with error notification
4. WHEN a profile is active THEN it SHALL be indicated in the UI with a checkmark or highlight
5. WHEN user uses CLI to switch THEN `keyrx profile activate <name>` SHALL work

### Requirement 3: Profile Management UI

**User Story:** As a user managing multiple profiles, I want to see all my profiles in a list with metadata, so that I can organize them easily.

**Acceptance Criteria:**
1. WHEN profiles page loads THEN it SHALL display all profiles in a card/list view
2. WHEN user hovers over a profile THEN action buttons SHALL appear (activate, rename, duplicate, delete)
3. WHEN user renames a profile THEN the file SHALL be renamed and references updated
4. WHEN user duplicates a profile THEN a copy SHALL be created with "(Copy)" appended to name
5. WHEN user deletes a profile THEN a confirmation dialog SHALL warn about permanent deletion

### Requirement 4: Default and Auto-Switch

**User Story:** As a power user, I want to set a default profile that loads on daemon startup, so that my preferred config is always active.

**Acceptance Criteria:**
1. WHEN user marks a profile as "Default" THEN it SHALL load automatically on daemon startup
2. WHEN no default is set THEN the last active profile SHALL be remembered
3. WHEN auto-switch is enabled THEN the daemon SHALL switch profiles based on active application (future)
4. WHEN user configures triggers THEN they SHALL be able to map app names to profiles
5. WHEN daemon starts and default profile missing THEN it SHALL fall back to built-in defaults

### Requirement 5: Import/Export

**User Story:** As a user sharing configs with others, I want to export profiles as files, so that I can share them or back them up.

**Acceptance Criteria:**
1. WHEN user exports a profile THEN it SHALL download as a .zip containing .krx + metadata.json
2. WHEN user imports a profile THEN it SHALL be extracted and added to the profiles list
3. WHEN import fails (invalid format) THEN clear error messages SHALL explain the issue
4. WHEN user exports all profiles THEN a single archive SHALL contain all configs
5. WHEN user imports duplicate names THEN a conflict resolution dialog SHALL appear

## Non-Functional Requirements

- **Architecture**: Profile storage in ~/.config/keyrx/profiles/, REST API for CRUD operations
- **Performance**: Profile switch <500ms, list load <100ms
- **Code Quality**: File sizes ≤300 lines (components), TypeScript strict mode
- **Accessibility**: WCAG 2.1 AA (keyboard navigation, screen reader support)

## Dependencies

- Leverage: Existing daemon config loading (keyrx_daemon/src/config.rs)
- New: REST API endpoints for profile management
- UI: React components for profile list and management

## Sources

- [Designing profile, account, and setting pages for better UX](https://medium.com/design-bootcamp/designing-profile-account-and-setting-pages-for-better-ux-345ef4ca1490)
- [App Settings UI Design: Usability Tips & Best Practices](https://www.setproduct.com/blog/settings-ui-design)
- [Top 10 UI/UX Configuration Patterns to Watch in 2025](https://medium.com/@proappdeveoper/top-10-ui-ux-configuration-patterns-to-watch-in-2025-2b2d41054597)
