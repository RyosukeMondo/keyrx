# Tasks Document: Web UI Bugfix and Enhancement

## Phase 1: Bugfixes (Low Risk)

### Task 1: Fix DeviceListCard to fetch devices from API

- [x] 1. Fix DeviceListCard to fetch devices from API
  - File: `keyrx_ui/src/components/DeviceListCard.tsx`
  - Purpose: Fix hardcoded device count, fetch actual devices from API
  - _Leverage: `keyrx_ui/src/hooks/useDevices.ts` (already exists), `LoadingSkeleton.tsx`, `ErrorState.tsx`_
  - _Requirements: Requirement 1 - Dashboard Device Count Accuracy_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in hooks and data fetching | Task: Modify DeviceListCard component in keyrx_ui/src/components/DeviceListCard.tsx to fetch devices using the existing useDevices hook instead of receiving devices as props. Display loading skeleton while fetching, error state on failure with retry button, and actual device count when loaded. Follow Requirement 1 acceptance criteria. | Restrictions: Do not modify useDevices hook, maintain existing component styling, ensure backward compatibility with HomePage usage | Success: ✅ Component fetches devices from API, ✅ Loading state shows LoadingSkeleton, ✅ Error state shows ErrorState with retry, ✅ Device count updates correctly, ✅ HomePage renders DeviceListCard without errors | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts (document component changes, API integration), then mark [x] when done_

---

### Task 2: Verify DevicesPage layout auto-save functionality

- [x] 2. Verify DevicesPage layout auto-save functionality
  - File: `keyrx_ui/src/pages/DevicesPage.tsx` (verification only, may not need changes)
  - Purpose: Test and verify existing auto-save implementation works correctly
  - _Leverage: `keyrx_ui/src/hooks/useAutoSave.ts` (already implemented), `RpcClient.setDeviceLayout` method_
  - _Requirements: Requirement 2 - Device Layout Selection Persistence_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with React testing expertise | Task: Verify DevicesPage layout auto-save functionality at lines 76-88 in keyrx_ui/src/pages/DevicesPage.tsx. Test that selecting layout (JIS 109, ANSI 104, etc.) triggers useAutoSave hook, calls rpcClient.setDeviceLayout with correct parameters, displays save indicators, and persists across navigation. If bugs found, fix them. Follow Requirement 2 acceptance criteria. | Restrictions: Do not modify useAutoSave hook unless broken, maintain debounce timing (500ms), ensure error handling with rollback | Success: ✅ Layout selection triggers auto-save within 500ms, ✅ API call succeeds with correct params, ✅ "✓ Saved" indicator appears for 2 seconds, ✅ Selection persists after navigation, ✅ Integration test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting test results and any fixes, then mark [x] when done_

---

### Task 3: Fix ProfilesPage activation state persistence

- [x] 3. Fix ProfilesPage activation state persistence
  - File: `keyrx_ui/src/pages/ProfilesPage.tsx` (lines 113-132)
  - Purpose: Fix disappearing [Active] badge after profile activation
  - _Leverage: `useQueryClient` from `@tanstack/react-query`, existing `activateProfileMutation` hook_
  - _Requirements: Requirement 3 - Profile Activation State Persistence_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in React Query and state management | Task: Fix profile activation persistence bug in keyrx_ui/src/pages/ProfilesPage.tsx (lines 113-132). After successful activation (no compilation errors), invalidate profiles cache using queryClient.invalidateQueries(['profiles']) to refetch updated isActive state. Ensure error cases don't invalidate cache. Follow Requirement 3 acceptance criteria. | Restrictions: Do not modify API endpoints, maintain existing error handling logic, ensure compilation errors prevent cache invalidation | Success: ✅ [Active] badge persists indefinitely after activation, ✅ Cache invalidates only on success, ✅ Compilation errors show error modal and keep previous profile active, ✅ Page refresh maintains active state, ✅ E2E test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with artifacts documenting cache invalidation logic, then mark [x] when done_

---

### Task 4: Create ProfileHeader component for ConfigPage

- [x] 4. Create ProfileHeader component for ConfigPage
  - File: `keyrx_ui/src/components/config/ProfileHeader.tsx` (NEW)
  - Purpose: Display profile context in ConfigPage header (name, active badge, profile selector)
  - _Leverage: `Dropdown.tsx`, `Badge` component or styled span, `useProfiles` hook_
  - _Requirements: Requirement 5 - Profile-Centric Configuration Workflow_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in component design | Task: Create ProfileHeader component in new file keyrx_ui/src/components/config/ProfileHeader.tsx. Display "Editing: {profileName}", show green [Active] badge if profile is active in daemon, display last modified timestamp, and include profile selector dropdown. Follow TypeScript interface from design.md. Follow Requirement 5 acceptance criteria. | Restrictions: Use existing Dropdown component, ensure responsive design, add aria-labels for accessibility | Success: ✅ Component renders profile name correctly, ✅ Active badge shows when isActive=true, ✅ Profile selector dropdown works, ✅ Props are type-safe, ✅ Unit tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with component artifacts, then mark [x] when done_

---

### Task 5: Add active profile display to MetricsPage

- [x] 5. Add active profile display to MetricsPage
  - File: `keyrx_ui/src/pages/MetricsPage.tsx`
  - Purpose: Display currently active profile name and .rhai file in header
  - _Leverage: `useProfiles` hook, `Link` from react-router-dom_
  - _Requirements: Requirement 6 - Metrics Page Profile Display_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with routing experience | Task: Modify MetricsPage in keyrx_ui/src/pages/MetricsPage.tsx to display active profile information in header. Fetch profiles using useProfiles hook, find active profile (isActive=true), display "Active Profile: {name}" with clickable link to /config?profile={name}, show .rhai filename. Follow Requirement 6 acceptance criteria. | Restrictions: Do not modify existing metrics components, maintain existing layout, ensure no profile case shows "No active profile" | Success: ✅ Active profile name displays correctly, ✅ Link navigates to ConfigPage with correct query param, ✅ .rhai filename shown, ✅ "No active profile" shown when daemon not running, ✅ Unit test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting integration, then mark [x] when done_

---

## Phase 2: Simulator Enhancement (Medium Risk)

### Task 6: Enhance SimulatorPage with profile selector and WASM

- [x] 6. Enhance SimulatorPage with profile selector and WASM integration
  - File: `keyrx_ui/src/pages/SimulatorPage.tsx`
  - Purpose: Move WASM simulation to dedicated page with profile selection
  - _Leverage: `useWasm` hook, `KeyboardVisualizer` component, `StateIndicatorPanel`, `useProfiles` hook, `Dropdown`_
  - _Requirements: Requirement 7 - WASM Simulator Dedicated Page_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in WASM integration | Task: Enhance SimulatorPage in keyrx_ui/src/pages/SimulatorPage.tsx to include profile selector dropdown, load profile config via useGetProfileConfig, initialize WASM simulator using useWasm hook, render KeyboardVisualizer in simulator mode with onKeyClick handler, display event log and state inspector. Follow design.md implementation and Requirement 7 acceptance criteria. | Restrictions: Reuse existing KeyboardVisualizer and StateIndicatorPanel components, handle WASM loading errors gracefully, ensure event log displays input→output events | Success: ✅ Profile selector loads all profiles, ✅ WASM initializes with selected profile config, ✅ Clicking keys processes events through WASM, ✅ Event log shows input/output, ✅ State inspector updates on modifier/lock changes, ✅ Integration test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with WASM integration artifacts, then mark [x] when done_

---

## Phase 3: Drag-and-Drop Configuration Editor (High Complexity)

### Task 7: Install @dnd-kit dependencies

- [x] 7. Install @dnd-kit dependencies for drag-and-drop
  - Files: `keyrx_ui/package.json` (modify)
  - Purpose: Add keyboard-accessible drag-and-drop library
  - _Leverage: None (new dependency)_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Build Engineer | Task: Install @dnd-kit/core@^6.0.8 and @dnd-kit/utilities@^3.2.1 in keyrx_ui directory. Run npm install and verify package.json is updated. Check bundle size impact (should be ~17KB gzipped). | Restrictions: Use exact versions specified, verify no peer dependency conflicts, do not install @dnd-kit/sortable (not needed) | Success: ✅ Dependencies installed successfully, ✅ package.json updated, ✅ npm run build succeeds, ✅ Bundle size increase <20KB gzipped | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting package versions, then mark [x] when done_

---

### Task 8: Create TypeScript types for key mappings

- [x] 8. Create TypeScript types for key mappings and drag-and-drop
  - File: `keyrx_ui/src/types/config.ts` (NEW)
  - Purpose: Define TypeScript interfaces for KeyMapping, AssignableKey, Layer types
  - _Leverage: None (new file, but follow existing type patterns)_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer specializing in type systems | Task: Create new file keyrx_ui/src/types/config.ts with KeyMapping, AssignableKey, and Layer interfaces as defined in design.md Data Models section. Ensure strict typing for union types (simple | tap-hold | macro | layer-switch), proper optional fields, and comprehensive JSDoc comments. | Restrictions: Follow existing type naming conventions, use strict TypeScript mode, export all types | Success: ✅ All types compile without errors, ✅ Types match design.md specifications exactly, ✅ JSDoc comments complete and helpful, ✅ No type errors in IDE | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with type definitions as artifacts, then mark [x] when done_

---

### Task 9: Create DragKeyPalette component

- [ ] 9. Create DragKeyPalette component with draggable keys
  - File: `keyrx_ui/src/components/config/DragKeyPalette.tsx` (NEW)
  - Purpose: Display palette of draggable virtual keys, modifiers, locks, layers
  - _Leverage: `@dnd-kit/core` (useDraggable hook), `Button` component for styling_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in drag-and-drop UIs | Task: Create DragKeyPalette component in new file keyrx_ui/src/components/config/DragKeyPalette.tsx. Render draggable keys using useDraggable hook from @dnd-kit/core, organize keys by category (VK_, MD_, LK_, Layers, Macros), support filterCategory prop, emit onDragStart/onDragEnd callbacks. Follow design.md interface. Create static list of assignable keys (VK_A-VK_Z, common modifiers, locks). | Restrictions: Use existing Button/Card styling, ensure touch-friendly targets (≥44px), add aria-labels for each key, support keyboard drag (Space to grab) | Success: ✅ Keys are draggable with mouse, ✅ Keys organized by category, ✅ onDragStart callback fires with AssignableKey data, ✅ Accessible via keyboard (Tab + Space), ✅ Unit tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with component artifacts, then mark [x] when done_

---

### Task 10: Create KeyMappingDialog component

- [ ] 10. Create KeyMappingDialog modal for key configuration
  - File: `keyrx_ui/src/components/config/KeyMappingDialog.tsx` (NEW)
  - Purpose: Modal dialog for configuring individual key mappings (simple, tap-hold, macro, layer-switch)
  - _Leverage: `Modal` component, `Dropdown`, `Input` components, `Button`_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer specializing in forms and modals | Task: Create KeyMappingDialog component in new file keyrx_ui/src/components/config/KeyMappingDialog.tsx. Use existing Modal component as wrapper, display keyCode in title, show radio buttons for mapping type (simple/tap-hold/macro/layer-switch), render dynamic form fields based on selection, validate inputs before save, emit onSave callback with KeyMapping object. Follow design.md interface. | Restrictions: Use existing form components (Dropdown, Input), validate timeout range (100-500ms), ensure Escape key closes dialog, return focus to trigger element | Success: ✅ Dialog opens/closes correctly, ✅ Form fields change based on mapping type, ✅ Validation prevents invalid saves, ✅ onSave callback emits correct KeyMapping structure, ✅ Accessible with keyboard, ✅ Unit tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with component artifacts and form validation logic, then mark [x] when done_

---

### Task 11: Modify KeyboardVisualizer to support drop zones

- [ ] 11. Modify KeyboardVisualizer component to add drag-and-drop drop zones
  - File: `keyrx_ui/src/components/KeyboardVisualizer.tsx` (MODIFY EXISTING)
  - Purpose: Add drop zone functionality to existing keyboard visualizer
  - _Leverage: `@dnd-kit/core` (useDroppable hook), existing `KeyButton` component_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with drag-and-drop expertise | Task: Modify existing KeyboardVisualizer in keyrx_ui/src/components/KeyboardVisualizer.tsx to add optional onKeyDrop callback prop and keyMappings prop. Wrap KeyButton components with useDroppable hook from @dnd-kit/core to make each key a drop zone. Highlight keys on drag-over (ring-2 ring-blue-500), display mapping labels when keyMappings provided, support activeLayer prop to show layer-specific mappings. Follow design.md modifications section. | Restrictions: Maintain backward compatibility (all new props optional), do not break existing usage, preserve current styling for non-drag mode, ensure drop zones work with keyboard (Space to drop) | Success: ✅ Existing usage still works without new props, ✅ Drop zones highlight on drag-over, ✅ onKeyDrop callback fires with correct keyCode, ✅ Mapping labels display correctly, ✅ Layer-specific mappings shown when activeLayer specified, ✅ Unit tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting backward compatibility, then mark [x] when done_

---

### Task 12: Create useDragAndDrop hook for state management

- [ ] 12. Create useDragAndDrop custom hook for drag-and-drop state
  - File: `keyrx_ui/src/hooks/useDragAndDrop.ts` (NEW)
  - Purpose: Encapsulate drag-and-drop state logic (active drag key, drop handlers, save logic)
  - _Leverage: `useSetProfileConfig` hook, `KeyMapping` and `AssignableKey` types_
  - _Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hooks specialist | Task: Create useDragAndDrop custom hook in new file keyrx_ui/src/hooks/useDragAndDrop.ts. Manage activeDragKey state, provide handleDragStart/handleDragEnd/handleKeyDrop callbacks, implement auto-save logic with useSetProfileConfig, handle optimistic updates with error rollback. Accept profileName and selectedLayer as parameters. Return drag handlers and current state. | Restrictions: Use React best practices (avoid stale closures), implement proper error handling with rollback, debounce API calls if needed, ensure type safety | Success: ✅ Hook manages drag state correctly, ✅ handleKeyDrop saves to API and updates local state, ✅ Optimistic updates work with rollback on error, ✅ Type-safe interface, ✅ Unit tests pass | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with hook implementation as artifact, then mark [x] when done_

---

### Task 13: Integrate drag-and-drop in ConfigPage

- [ ] 13. Integrate drag-and-drop UI in ConfigPage
  - File: `keyrx_ui/src/pages/ConfigPage.tsx` (MAJOR MODIFICATION)
  - Purpose: Add drag-and-drop layer to ConfigPage with DndContext wrapper
  - _Leverage: `DndContext` from @dnd-kit/core, all new components (DragKeyPalette, KeyMappingDialog, ProfileHeader), modified KeyboardVisualizer, useDragAndDrop hook_
  - _Requirements: Requirement 4, 5, 8 - QMK Drag-and-Drop + Profile Context + Device Integration_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior React Developer with full-stack experience | Task: Integrate drag-and-drop UI into ConfigPage in keyrx_ui/src/pages/ConfigPage.tsx. Wrap entire page in DndContext, add ProfileHeader at top, use DeviceScopeToggle with real devices from useDevices (not mock), add LayerSelector, render DragKeyPalette + KeyboardVisualizer in grid layout, use useDragAndDrop hook for state management, keep Monaco editor as fallback tab. Follow design.md ConfigPage architecture. | Restrictions: Maintain existing Monaco editor functionality, ensure visual/code tabs still work, lazy load @dnd-kit to reduce initial bundle, handle loading states for devices/profiles | Success: ✅ Drag-and-drop works end-to-end (palette → keyboard → save → API), ✅ ProfileHeader displays correct profile info, ✅ DeviceScopeToggle shows real devices, ✅ Layer selector changes active layer, ✅ Monaco editor tab still functional, ✅ Integration test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with full integration artifacts, then mark [x] when done_

---

### Task 14: Add keyboard accessibility to drag-and-drop

- [ ] 14. Add keyboard accessibility for drag-and-drop (WCAG 2.2 Level AA)
  - Files: `keyrx_ui/src/components/config/DragKeyPalette.tsx`, `KeyboardVisualizer.tsx` (modify)
  - Purpose: Ensure drag-and-drop works with keyboard (Space to grab, arrows to move, Space to drop)
  - _Leverage: `@dnd-kit/core` keyboard sensor, existing accessibility utilities_
  - _Requirements: Requirement 4 - Accessibility (WCAG 2.2 Level AA)_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Accessibility Specialist with React expertise | Task: Add keyboard accessibility to drag-and-drop in DragKeyPalette and KeyboardVisualizer. Implement Salesforce pattern: Tab to focus, Space to grab, Arrow keys to navigate, Space to drop, Escape to cancel. Add aria-labels ("Grabbed VK_A, use arrows to select target"), ensure focus indicators visible (2px outline), add screen reader announcements. Follow Requirement 4 accessibility criteria. | Restrictions: Must work without mouse, maintain WCAG 2.2 Level AA compliance, ensure focus management correct (trap focus during drag, return focus after drop), test with NVDA/JAWS | Success: ✅ Can drag and drop using only keyboard, ✅ Screen reader announces drag state, ✅ Focus indicators visible on all elements, ✅ Escape cancels drag operation, ✅ Accessibility tests (axe-core) pass with 0 violations | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting accessibility features, then mark [x] when done_

---

### Task 15: Integrate real device list in ConfigPage

- [ ] 15. Integrate real device list in ConfigPage DeviceScopeToggle
  - File: `keyrx_ui/src/pages/ConfigPage.tsx` (minor modification from Task 13)
  - Purpose: Ensure DeviceScopeToggle receives real devices from API, not mock data
  - _Leverage: `useDevices` hook, existing `DeviceScopeToggle` component_
  - _Requirements: Requirement 8 - Device List Integration in ConfigPage_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer | Task: Verify and fix ConfigPage device integration in keyrx_ui/src/pages/ConfigPage.tsx. Ensure useDevices hook fetches real devices, transform API response to DeviceOption format (map to {serial, name}), pass to DeviceScopeToggle component. Handle global vs device-specific scope changes, save mappings to correct scope. Follow Requirement 8 acceptance criteria. | Restrictions: Do not modify DeviceScopeToggle component, handle loading states for devices, ensure offline devices still appear in selector (greyed out) | Success: ✅ DeviceScopeToggle shows real connected devices from API, ✅ Selecting device-specific mode loads device configs, ✅ Mappings save to correct scope (global or device-serial), ✅ Device override indicator shows when device-specific conflicts with global, ✅ Integration test passes | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting device integration flow, then mark [x] when done_

---

## Phase 4: Testing and Quality Assurance

### Task 16: Write unit tests for new components

- [ ] 16. Write unit tests for new drag-and-drop components
  - Files: `keyrx_ui/src/components/config/*.test.tsx` (NEW), `keyrx_ui/src/hooks/useDragAndDrop.test.ts` (NEW)
  - Purpose: Ensure ≥80% test coverage for all new components and hooks
  - _Leverage: `@testing-library/react`, `vitest`, existing test utilities in `tests/testUtils.tsx`_
  - _Requirements: All Phase 3 tasks (comprehensive unit testing)_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in React Testing Library | Task: Write comprehensive unit tests for DragKeyPalette, KeyMappingDialog, ProfileHeader, and useDragAndDrop hook. Test rendering, user interactions, prop changes, error states, callbacks. Mock @dnd-kit hooks, useDevices, useProfiles, API calls. Follow testing best practices (test behavior not implementation, use userEvent for interactions, waitFor for async). Achieve ≥80% coverage. | Restrictions: Do not test third-party libraries, mock all external dependencies, ensure tests run independently, use renderWithProviders from testUtils for context | Success: ✅ All components have unit tests, ✅ Test coverage ≥80% for new code, ✅ All tests pass reliably, ✅ Mocks are properly configured, ✅ Coverage report shows green for all new files | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting test coverage, then mark [x] when done_

---

### Task 17: Write integration tests for drag-and-drop flow

- [ ] 17. Write integration tests for drag-and-drop user flow
  - File: `keyrx_ui/tests/integration/ConfigPage.integration.test.tsx` (NEW)
  - Purpose: Test complete drag-and-drop workflow from palette to keyboard to API save
  - _Leverage: `@testing-library/react`, `msw` for API mocking, existing integration test patterns_
  - _Requirements: Requirement 4 - End-to-end drag-and-drop functionality_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer | Task: Write integration tests for drag-and-drop flow in new file keyrx_ui/tests/integration/ConfigPage.integration.test.tsx. Test: (1) Drag VK_A from palette and drop onto CapsLock, (2) Verify mapping appears on keyboard, (3) Verify API call to PUT /api/config with correct payload, (4) Test error handling with API failure and rollback, (5) Test layer switching changes displayed mappings. Use MSW to mock API responses. Follow design.md integration testing strategy. | Restrictions: Test real component interactions (no mocking DragKeyPalette or KeyboardVisualizer), use fireEvent for drag events, verify API calls with msw handlers, ensure tests clean up properly | Success: ✅ Drag-and-drop flow test passes end-to-end, ✅ API call verification works, ✅ Error rollback test passes, ✅ Layer switching test passes, ✅ Tests run reliably and fast | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with integration test artifacts, then mark [x] when done_

---

### Task 18: Write end-to-end tests with Playwright

- [ ] 18. Write E2E tests for complete configuration workflow
  - File: `keyrx_ui/e2e/configuration-workflow.spec.ts` (NEW)
  - Purpose: Test complete user journey from creating profile to drag-and-drop config to activation
  - _Leverage: `@playwright/test`, existing E2E test setup_
  - _Requirements: All requirements - Complete user workflow validation_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Automation Engineer | Task: Write Playwright E2E test in new file keyrx_ui/e2e/configuration-workflow.spec.ts. Test complete workflow: (1) Navigate to profiles page, (2) Create new profile, (3) Open config page, (4) Drag VK_A onto CapsLock key, (5) Verify mapping appears, (6) Activate profile, (7) Navigate to metrics page, (8) Verify active profile displayed. Start real daemon in test setup, use actual API. Follow design.md E2E testing strategy. | Restrictions: Start daemon before tests (use beforeAll hook), clean up test data after, use data-testid attributes for reliable selectors, ensure test runs in CI environment | Success: ✅ E2E test passes locally, ✅ Test works in headless mode, ✅ Test cleans up created profiles, ✅ Screenshots on failure captured, ✅ Test runs in <60 seconds | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting E2E test coverage, then mark [x] when done_

---

### Task 19: Write accessibility tests with axe-core

- [ ] 19. Write accessibility tests for all pages with axe-core
  - File: `keyrx_ui/tests/accessibility/pages.a11y.test.tsx` (NEW or extend existing)
  - Purpose: Ensure WCAG 2.2 Level AA compliance with 0 violations
  - _Leverage: `@axe-core/react` or `jest-axe`, existing accessibility test setup_
  - _Requirements: Requirement 4 - Accessibility (WCAG 2.2 Level AA)_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Accessibility Testing Specialist | Task: Write automated accessibility tests for all modified pages (HomePage, DevicesPage, ProfilesPage, ConfigPage, MetricsPage, SimulatorPage). Use axe-core to scan for WCAG 2.2 Level AA violations. Test color contrast (≥4.5:1), keyboard navigation, ARIA labels, focus indicators, screen reader announcements. Verify 0 violations on all pages. Follow design.md accessibility compliance section. | Restrictions: Run axe on fully rendered pages with real data, test both light and dark themes if applicable, ensure focus management tested (tab order, focus trap in modals), verify touch targets ≥44px | Success: ✅ All pages pass axe-core scan with 0 violations, ✅ Keyboard navigation works on all pages, ✅ Focus indicators visible and correct, ✅ ARIA labels present and accurate, ✅ Accessibility report shows 100% compliance | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion documenting accessibility compliance, then mark [x] when done_

---

## Phase 5: Documentation and Finalization

### Task 20: Update component documentation and README

- [ ] 20. Document all new components and update README
  - Files: `keyrx_ui/README.md` (update), JSDoc comments in all new components
  - Purpose: Provide clear documentation for future developers
  - _Leverage: Existing documentation patterns_
  - _Requirements: All requirements - Comprehensive documentation_
  - _Prompt: Implement the task for spec web-ui-bugfix-and-enhancement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with React documentation expertise | Task: Add comprehensive JSDoc comments to all new components (DragKeyPalette, KeyMappingDialog, ProfileHeader, useDragAndDrop hook). Update keyrx_ui/README.md with "Drag-and-Drop Configuration" section explaining how to use the visual editor, keyboard shortcuts, and troubleshooting tips. Include code examples for each component. Document all props, callbacks, and types. | Restrictions: Follow existing documentation style, use markdown for README, ensure examples are copy-paste ready, include accessibility notes | Success: ✅ All new components have JSDoc comments, ✅ README updated with drag-and-drop section, ✅ Code examples provided and tested, ✅ Documentation is clear and helpful | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with documentation artifacts, then mark [x] when done_

---

## Summary

**Total Tasks**: 20

**Phases**:
- Phase 1 (Bugfixes): Tasks 1-5 (Low risk, 2-4 hours)
- Phase 2 (Simulator): Task 6 (Medium risk, 3-5 hours)
- Phase 3 (Drag-and-Drop): Tasks 7-15 (High complexity, 8-12 hours)
- Phase 4 (Testing): Tasks 16-19 (QA, 4-6 hours)
- Phase 5 (Documentation): Task 20 (1-2 hours)

**Estimated Total Time**: 18-29 hours

**Risk Assessment**:
- Low Risk: Tasks 1-5 (bugfixes in existing components)
- Medium Risk: Task 6 (WASM integration)
- High Risk: Tasks 7-15 (new drag-and-drop feature, complex interactions)

**Dependencies**:
- Phase 1 and 2 can run in parallel
- Phase 3 must complete Task 7-8 before 9-15
- Phase 4 depends on Phase 3 completion
- Phase 5 can start anytime after Phase 3

**Success Criteria**:
- All 8 requirements implemented and tested
- ≥80% test coverage on new code
- 0 accessibility violations (WCAG 2.2 Level AA)
- Bundle size increase <20KB gzipped
- All E2E tests passing
