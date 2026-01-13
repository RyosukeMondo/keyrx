# improve-web-ui - Tasks

## Overview
General UI/UX improvements for the KeyRX Web UI excluding key palette (separate spec) and WASM (separate spec).

---

## Task 1: Narrow Layer Switcher Width

- [x] 1.1 Refactor LayerSwitcher to fixed narrow width
  - File: `keyrx_ui/src/components/LayerSwitcher.tsx`
  - Change from full-width to fixed narrow width (~80px / 7 chars)
  - Layers are predictable: "Base", "MD_00", "MD_FF" (max 7 chars)
  - Use `w-20` or similar Tailwind class for consistent narrow width
  - Keep vertical scrollable layout
  - Purpose: Reduce horizontal space, allow room for keyboard layouts
  - _Leverage: keyrx_ui/src/components/LayerSwitcher.tsx existing implementation_
  - _Requirements: Narrow layer switcher with predictable width_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer | Task: Refactor LayerSwitcher to use fixed narrow width (~80px) since layer names are predictable (Base, MD_00-MD_FF). Replace responsive width with Tailwind w-20 or w-[80px]. Maintain scroll behavior and search. | Restrictions: Keep existing functionality, don't break layer selection | Success: LayerSwitcher renders at consistent narrow width, fits alongside keyboard layouts | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 2: Layer-Dedicated Keyboard Layout Side-by-Side

- [x] 2.1 Create dual-pane layout for Global and Device-Specific keys
  - File: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Layout: `LAYERS(narrow) | Global Keyboard | Device-Specific Keyboard`
  - Each keyboard view has its own dedicated layer switcher
  - Global keys: Apply to all devices, layer switcher on left
  - Device keys: Device-specific overrides, layer switcher on left of that section
  - Purpose: Clear separation between global and device-specific configurations
  - _Leverage: keyrx_ui/src/pages/ConfigPage.tsx, KeyboardVisualizer.tsx, LayerSwitcher.tsx_
  - _Requirements: Side-by-side layout with dedicated layer switchers_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Layout Architect | Task: Redesign ConfigPage to show dual-pane layout: narrow LayerSwitcher + Global Keyboard on left, narrow LayerSwitcher + Device Keyboard on right. Use flex layout with gap. Each pane independently scrollable. | Restrictions: Keep existing KeyboardVisualizer component, maintain layer state per pane | Success: Two keyboard layouts visible side-by-side with dedicated layer switchers | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 2.2 Add pane header labels and device selector
  - File: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Add "Global Keys" header above left pane
  - Add "Device: [dropdown]" header above right pane
  - Device selector only in device-specific pane
  - Visual distinction: different background tint per pane
  - Purpose: Clear UX for which pane affects what
  - _Leverage: keyrx_ui/src/pages/ConfigPage.tsx existing DeviceSelector logic_
  - _Requirements: Clear visual separation with labeled headers_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer | Task: Add pane headers "Global Keys" and "Device: [selector]" above respective keyboard views. Move device selector to device pane header. Add subtle background tint (slate-50 vs zinc-50) to distinguish panes. | Restrictions: Keep existing device selection logic, maintain accessibility | Success: Each pane clearly labeled, device selector in correct location | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 3: Responsive Layout for Smaller Screens

- [x] 3.1 Add responsive breakpoints for dual-pane layout
  - File: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Desktop (lg+): Side-by-side dual pane
  - Tablet (md): Stacked vertical with tabs
  - Mobile (sm): Single pane with toggle
  - Purpose: Usable on all screen sizes
  - _Leverage: Tailwind responsive classes, existing layout_
  - _Requirements: Responsive design for all screen sizes_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in responsive design | Task: Add Tailwind responsive classes: lg:flex-row md:flex-col. For md and below, add tab switcher between Global/Device views. Use hidden/block classes for responsive visibility. | Restrictions: Don't duplicate keyboard visualizer instances, use conditional rendering | Success: Layout adapts gracefully from desktop to mobile | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 4: Improve Config Editor Header

- [x] 4.1 Streamline profile selector and status area
  - File: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Move profile selector to left, compact design
  - Add sync status indicator (saved/unsaved/syncing)
  - Add keyboard layout selector (ANSI/ISO/JIS)
  - Remove redundant visual editor/code editor tabs from header
  - Purpose: Cleaner header with essential controls
  - _Leverage: keyrx_ui/src/pages/ConfigPage.tsx header section_
  - _Requirements: Streamlined header with status indicators_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI/UX Developer | Task: Redesign header: Profile dropdown (left), Layout selector (center), Sync status badge (right). Remove tab navigation from header. Add green/yellow/red status dot for sync state. | Restrictions: Keep profile functionality, maintain save/load operations | Success: Clean header with clear status indication | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 5: Add Code Editor as Collapsible Panel

- [x] 5.1 Convert code editor to collapsible bottom panel
  - File: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Remove tab-based switching between visual/code editor
  - Add collapsible panel at bottom for code view
  - Toggle button: "Show Code" / "Hide Code"
  - Panel height: 200-400px, resizable
  - Purpose: See code alongside visual editor
  - _Leverage: keyrx_ui/src/pages/ConfigPage.tsx, SimpleCodeEditor component_
  - _Requirements: Code editor as collapsible overlay, not separate tab_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer | Task: Replace tab navigation with collapsible code panel. Add toggle button in header. Panel slides up from bottom with animation. Use Tailwind transition classes. Add resize handle for panel height. | Restrictions: Keep bidirectional sync working, maintain editor functionality | Success: Code panel can be toggled without losing visual editor context | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 6: Improve Key Config Modal

- [x] 6.1 Enhance key configuration modal/popup UX
  - File: `keyrx_ui/src/components/KeyConfigModal.tsx`
  - Show current key info prominently at top
  - Clearer mapping type selector (Simple, Tap/Hold, Macro, Layer)
  - Preview of resulting behavior before save
  - Quick-assign shortcuts for common mappings
  - Purpose: Faster, more intuitive key configuration
  - _Leverage: keyrx_ui/src/components/KeyConfigModal.tsx, KeyConfigDialog.tsx_
  - _Requirements: Improved modal UX with clear mapping options_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Modal Specialist | Task: Enhance KeyConfigModal: Add key info header showing physical key label. Add mapping type tabs with icons. Add "Preview" section showing resulting behavior. Add quick-assign buttons for common maps (Escape, Enter, Backspace). | Restrictions: Keep existing save/cancel functionality, maintain validation | Success: Modal is more intuitive with clear feedback | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 7: Add Visual Feedback Improvements

- [ ] 7.1 Add hover states and tooltips to keyboard visualizer
  - File: `keyrx_ui/src/components/KeyboardVisualizer.tsx`, `KeyButton.tsx`
  - Hover: Show tooltip with key info and current mapping
  - Click feedback: Brief highlight animation
  - Mapped keys: Show mapping type icon overlay
  - Unmapped keys: Subtle different appearance
  - Purpose: Better visual feedback during configuration
  - _Leverage: keyrx_ui/src/components/KeyButton.tsx, Tooltip component_
  - _Requirements: Rich visual feedback on keyboard keys_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Animation Developer | Task: Add hover tooltips to KeyButton showing "Physical: X, Mapped to: Y, Type: Z". Add click ripple effect. Add small icons in corner for mapping type. Style unmapped keys with dashed border. | Restrictions: Keep performance (memo components), maintain accessibility | Success: Keys provide rich visual feedback on interaction | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 8: Unit Tests for Layout Changes

- [ ] 8.1 Add tests for new layout components
  - File: `keyrx_ui/src/__tests__/ConfigPage.layout.test.tsx` (create)
  - Test dual-pane layout renders correctly
  - Test responsive breakpoint behavior
  - Test collapsible code panel toggle
  - Test layer switcher width
  - Purpose: Prevent regression of layout changes
  - _Leverage: tests/testUtils.tsx, React Testing Library_
  - _Requirements: Test coverage for new layout features_
  - _Prompt: Implement the task for spec improve-web-ui, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Create layout tests: 1) Dual pane renders at lg breakpoint, 2) Single pane at md, 3) Code panel toggles visibility, 4) LayerSwitcher has narrow width. Use matchMedia mock for responsive tests. | Restrictions: Follow existing test patterns, use renderWithProviders | Success: All layout behaviors tested, CI passes | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1.1 | Narrow LayerSwitcher width | LayerSwitcher.tsx |
| 2.1-2.2 | Dual-pane layout with dedicated layer switchers | ConfigPage.tsx |
| 3.1 | Responsive breakpoints | ConfigPage.tsx |
| 4.1 | Streamlined header | ConfigPage.tsx |
| 5.1 | Collapsible code panel | ConfigPage.tsx |
| 6.1 | Enhanced key config modal | KeyConfigModal.tsx |
| 7.1 | Visual feedback improvements | KeyboardVisualizer.tsx, KeyButton.tsx |
| 8.1 | Unit tests | ConfigPage.layout.test.tsx |

Total: 8 subtasks across layout, UX, and testing
