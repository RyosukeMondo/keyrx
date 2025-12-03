# Tasks Document

## Phase 1: Core Profile System

- [ ] 1. Create Profile struct
  - File: `core/src/profile/mod.rs`
  - Profile definition
  - Serialization
  - Purpose: Data model
  - _Requirements: 1.1_

- [ ] 2. Implement ProfileManager
  - File: `core/src/profile/manager.rs`
  - CRUD operations
  - Profile storage
  - Purpose: Profile management
  - _Requirements: 1.1, 1.3, 1.4_

- [ ] 3. Add profile inheritance
  - File: `core/src/profile/inheritance.rs`
  - Base profile resolution
  - Override merging
  - Purpose: Config reuse
  - _Requirements: 3.1_

## Phase 2: Switching Mechanisms

- [ ] 4. Implement profile switching
  - File: `core/src/profile/switcher.rs`
  - Fast activation
  - State preservation
  - Purpose: Profile activation
  - _Requirements: 1.2, 2.1_

- [ ] 5. Add auto-switch rules
  - File: `core/src/profile/auto_switch.rs`
  - Application detection
  - Rule matching
  - Purpose: Context awareness
  - _Requirements: 2.4_

- [ ] 6. Implement hotkey switching
  - File: `core/src/profile/hotkeys.rs`
  - Profile cycle hotkey
  - Direct profile hotkeys
  - Purpose: Quick access
  - _Requirements: 2.2_

## Phase 3: CLI Integration

- [ ] 7. Add profile CLI commands
  - File: `core/src/cli/commands/profile.rs`
  - list, create, delete, switch
  - export, import
  - Purpose: CLI management
  - _Requirements: 2.1, 3.2, 3.3_

## Phase 4: Flutter UI

- [ ] 8. Create profile selector widget
  - File: `ui/lib/widgets/profile_selector.dart`
  - Dropdown with icons
  - Quick switch
  - Purpose: UI switching
  - _Requirements: 2.3, 3.4_

- [ ] 9. Add profile management page
  - File: `ui/lib/pages/profiles_page.dart`
  - Create/edit profiles
  - Auto-switch rules
  - Purpose: Full management
  - _Requirements: 1.1, 2.4_
