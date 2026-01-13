# sophisticated-key-palette - Tasks

## Overview
Implement a sophisticated key palette inspired by QMK Configurator and VIA, with comprehensive key categories, search functionality, and intuitive selection UX.

## Research Summary
Based on QMK Configurator and VIA best practices:
- **VIA Categories**: Basic, Lighting, Media, Macro, Layers, Special, Other
- **QMK Pattern**: Drag-and-drop, click-select, physical key input
- **Progressive Disclosure**: Common keys first, advanced via search/Any key
- **Hover Tooltips**: Brief explanations without overwhelming
- **Real-time Visual Feedback**: Keyboard updates during remapping

Sources:
- [QMK Configurator Step by Step](https://docs.qmk.fm/configurator_step_by_step)
- [VIA Usage Guide](https://docs.keeb.io/via)
- [VIA App](https://usevia.app/)

---

## Task 1: Redesign Key Palette Categories

- [x] 1.1 Implement VIA-style category structure
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - Categories: Basic, Modifiers, Media, Macro, Layers, Special, Any
  - Basic: Letters A-Z, Numbers 0-9, Punctuation, Navigation
  - Modifiers: Ctrl, Shift, Alt, Meta, Mod-Tap combinations
  - Media: Volume, Playback, Brightness
  - Macro: User-defined macros (M0-M15)
  - Layers: MO(), TO(), TG(), OSL() layer functions
  - Special: Mouse keys, System keys, Unicode
  - Any: Custom keycode input field
  - Purpose: Organize 200+ keys into logical, discoverable groups
  - _Leverage: keyrx_ui/src/components/KeyPalette.tsx, KeyAssignmentPanel.tsx_
  - _Requirements: VIA-style categorization with comprehensive key coverage_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Component Architect | Task: Redesign KeyPalette with 7 VIA-style categories. Create category data structure with icons. Implement tab-based category navigation. Each category has subcategories (Basic -> Letters, Numbers, Punctuation, Navigation). | Restrictions: Maintain existing key IDs, ensure all current keys are categorized | Success: All 200+ keys organized into 7 categories with subcategories | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 1.2 Create comprehensive key definitions file
  - File: `keyrx_ui/src/data/keyDefinitions.ts` (create)
  - Define all keys with: id, label, category, subcategory, description, aliases
  - Include QMK-compatible key names (KC_A, KC_ENTER, etc.)
  - Support modifier combinations (LCTL(KC_C), LSFT(KC_A))
  - Add all F1-F24, numpad, media, system keys
  - Purpose: Single source of truth for key definitions
  - _Leverage: keyrx_ui/src/utils/keyCodeMapping.ts existing mappings_
  - _Requirements: Complete key definition database with metadata_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Data Architect | Task: Create keyDefinitions.ts with typed key data. Structure: { id, label, category, subcategory, description, aliases, icon? }. Include all QMK keycodes. Export getKeysByCategory(), searchKeys(), getKeyById() functions. | Restrictions: Match existing key IDs, support both VK_ and KC_ prefixes | Success: 250+ keys defined with full metadata, utility functions work | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 2: Implement Advanced Search Functionality

- [x] 2.1 Add fuzzy search with highlighting
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - Search input at top of palette, always visible
  - Fuzzy matching: "ctrl" matches "Left Ctrl", "LCTL", "KC_LCTL"
  - Search across: label, id, description, aliases
  - Highlight matching text in results
  - Filter reduces displayed keys in real-time
  - Show "No results" with suggestions
  - Purpose: Fast key discovery regardless of naming convention
  - _Leverage: keyrx_ui/src/data/keyDefinitions.ts, existing search logic_
  - _Requirements: Fuzzy search across all key metadata_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Search UX Developer | Task: Implement fuzzy search using simple scoring algorithm. Search across id, label, description, aliases. Highlight matches with <mark> tags. Show result count. Add keyboard navigation (arrow keys, enter to select). | Restrictions: No external fuzzy library, keep bundle small | Success: Typing "ctrl" shows all control-related keys with highlights | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 2.2 Add recent/favorite keys section
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - "Recent" section: Last 10 used keys (persisted to localStorage)
  - "Favorites" section: User-pinned keys (star button)
  - Show at top before category tabs
  - Quick access without searching
  - Purpose: Speed up common key assignments
  - _Leverage: localStorage, keyrx_ui/src/hooks/useLocalStorage.ts_
  - _Requirements: Recent and favorite keys with persistence_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React State Manager | Task: Add "Recent" (auto-tracked, max 10) and "Favorites" (user-pinned) sections above categories. Use localStorage for persistence. Add star icon to toggle favorite. Show empty state messages. | Restrictions: Don't break existing palette layout, handle localStorage errors | Success: Recent keys appear after use, favorites persist across sessions | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 3: Implement "Any Key" Custom Input

- [x] 3.1 Add custom keycode input field
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - "Any" category with text input for custom keycodes
  - Support QMK syntax: KC_A, LCTL(KC_C), MO(1), LT(2,KC_SPC)
  - Validation with real-time feedback
  - Help tooltip explaining syntax
  - Purpose: Power users can input any valid keycode
  - _Leverage: keyrx_ui/src/utils/keyCodeMapping.ts for validation_
  - _Requirements: Custom keycode input with validation_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Form UX Developer | Task: Add "Any" tab with text input. Validate input against known patterns (KC_*, MO(), LT(), etc.). Show green checkmark for valid, red X for invalid with error message. Add "Apply" button to use custom code. | Restrictions: Validate syntax, don't accept arbitrary strings | Success: Users can input "LCTL(KC_C)" and apply it as mapping | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 4: Enhance Visual Key Display

- [x] 4.1 Add key icons and visual hierarchy
  - File: `keyrx_ui/src/components/KeyPalette.tsx`, new `KeyPaletteItem.tsx`
  - Key items show: icon (optional), label, secondary label (alias)
  - Category-specific icons: keyboard for basic, layers icon for layers
  - Color coding by category (consistent with VIA)
  - Hover state with full description tooltip
  - Purpose: Visual distinction and rich information
  - _Leverage: lucide-react icons, existing Tooltip component_
  - _Requirements: Rich visual display with icons and tooltips_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI Component Developer | Task: Create KeyPaletteItem component with icon slot, label, sublabel, and tooltip. Add Lucide icons per category. Color-code borders/backgrounds by category. Show description on hover. | Restrictions: Keep items compact (44px min touch target), use existing icon library | Success: Keys display with icons, colors, and informative tooltips | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 4.2 Add grid/list view toggle
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - Grid view: Compact 4-column layout (current)
  - List view: Single column with full descriptions
  - Toggle button in palette header
  - Persist preference to localStorage
  - Purpose: User choice between compact and detailed views
  - _Leverage: Tailwind grid classes, localStorage_
  - _Requirements: Toggle between grid and list views_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer | Task: Add view toggle (Grid/List icons) in palette header. Grid: 4-col compact. List: 1-col with description visible. Use Tailwind responsive grid. Persist to localStorage. | Restrictions: Keep both views accessible, maintain drag-and-drop in both | Success: Users can switch between compact grid and detailed list | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 5: Implement Drag-and-Drop Enhancement

- [-] 5.1 Add drag preview and drop zone feedback
  - File: `keyrx_ui/src/components/KeyPalette.tsx`, `KeyboardVisualizer.tsx`
  - Drag preview: Show key being dragged with semi-transparency
  - Drop zones: Highlight valid drop targets on keyboard
  - Invalid drop: Show red indicator
  - Drop success: Brief animation/flash
  - Purpose: Clear visual feedback during drag operations
  - _Leverage: @dnd-kit existing setup, Tailwind animations_
  - _Requirements: Enhanced drag-and-drop visual feedback_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DnD Specialist | Task: Enhance drag-and-drop: Add DragOverlay with key preview. Highlight droppable keys with ring-2 ring-blue-400. Invalid drops show ring-red-400. Success shows brief scale animation. | Restrictions: Use existing @dnd-kit setup, don't add new dependencies | Success: Dragging key shows preview, drop zones highlight appropriately | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 6: Add Physical Key Capture

- [ ] 6.1 Implement "Press physical key" input mode
  - File: `keyrx_ui/src/components/KeyPalette.tsx`
  - Button: "Capture Key" enters listening mode
  - Listen for keydown event, map to key ID
  - Show captured key name, allow confirmation
  - Escape to cancel capture mode
  - Purpose: Assign key by pressing it (QMK Configurator pattern)
  - _Leverage: DOM keyboard events, keyrx_ui/src/utils/keyCodeMapping.ts_
  - _Requirements: Physical key capture for quick assignment_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Event Handler Developer | Task: Add "Capture Key" button. On click: show modal "Press any key...", listen for keydown, map event.code to key ID, show confirmation "Captured: A - Use this key?". Handle Escape to cancel. | Restrictions: Handle modifier keys correctly, don't capture Tab/Escape during normal use | Success: User can press physical key to select it in palette | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 7: Layer Function Keys

- [ ] 7.1 Add comprehensive layer function keys
  - File: `keyrx_ui/src/data/keyDefinitions.ts`, `KeyPalette.tsx`
  - MO(n): Momentary layer (hold to activate)
  - TO(n): Toggle to layer (tap to switch)
  - TG(n): Toggle layer (on/off)
  - OSL(n): One-shot layer (next key only)
  - LT(n, kc): Layer-tap (hold=layer, tap=keycode)
  - Purpose: Full layer control like QMK/VIA
  - _Leverage: keyrx_ui/src/data/keyDefinitions.ts_
  - _Requirements: All layer function types available_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Keyboard Firmware Expert | Task: Add layer function keys to Layers category. For each type (MO, TO, TG, OSL), create entries for layers 0-15. Add LT as special entry with layer+key selector. Include descriptions explaining each function. | Restrictions: Match QMK naming conventions, generate programmatically | Success: All layer functions available, descriptions explain behavior | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 8: Unit Tests for Key Palette

- [ ] 8.1 Add comprehensive tests for key palette
  - File: `keyrx_ui/src/__tests__/KeyPalette.test.tsx` (create)
  - Test category navigation
  - Test search functionality with various queries
  - Test drag-and-drop interaction
  - Test favorites persistence
  - Test custom keycode validation
  - Purpose: Ensure palette reliability
  - _Leverage: tests/testUtils.tsx, React Testing Library, @dnd-kit testing_
  - _Requirements: Full test coverage for new features_
  - _Prompt: Implement the task for spec sophisticated-key-palette, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Create KeyPalette tests: 1) Categories render with correct keys, 2) Search filters correctly, 3) Favorites persist, 4) Custom input validates, 5) Drag preview shows. Mock localStorage and keyboard events. | Restrictions: Follow existing test patterns, comprehensive coverage | Success: All new features have tests, CI passes | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1.1-1.2 | VIA-style categories + key definitions | KeyPalette.tsx, keyDefinitions.ts |
| 2.1-2.2 | Fuzzy search + recent/favorites | KeyPalette.tsx |
| 3.1 | Custom keycode input ("Any") | KeyPalette.tsx |
| 4.1-4.2 | Visual enhancements + view toggle | KeyPalette.tsx, KeyPaletteItem.tsx |
| 5.1 | Enhanced drag-and-drop feedback | KeyPalette.tsx, KeyboardVisualizer.tsx |
| 6.1 | Physical key capture | KeyPalette.tsx |
| 7.1 | Layer function keys (MO, TO, TG, OSL, LT) | keyDefinitions.ts, KeyPalette.tsx |
| 8.1 | Unit tests | KeyPalette.test.tsx |

Total: 10 subtasks for comprehensive key palette redesign
