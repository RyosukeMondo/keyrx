# Tasks: Refactor ConfigPage Component

## Phase 1: Extract Custom Hooks

- [x] 1.1 Create useProfileSelection hook
  - File: src/hooks/useProfileSelection.ts
  - Extract profile selection logic with fallback priority: manual > prop > route > query > active > 'Default'
  - Return selectedProfileName and setSelectedProfileName
  - Purpose: Centralize profile selection logic for reusability
  - _Leverage: src/hooks/useProfiles.ts, src/hooks/useActiveProfileQuery.ts_
  - _Requirements: 2.1, TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer specializing in custom hooks and state management | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create useProfileSelection custom hook that encapsulates profile selection logic from ConfigPage.tsx lines 56-78. The hook must handle priority fallback: manual selection > prop > route param > query param > active profile > 'Default'. Return { selectedProfileName, setSelectedProfileName }. Extract logic from ConfigPage.tsx without modifying the page yet. | Restrictions: Do not modify ConfigPage.tsx yet, ensure hook is pure and side-effect free except for useActiveProfileQuery, no UI logic, TypeScript strict mode, all parameters optional | Success: Hook created with proper TypeScript types, unit test covers all fallback scenarios, manual selection takes precedence, returns correct profile name in all cases | After completion: 1) Mark task as in-progress [-] in tasks.md before starting, 2) Use log-implementation tool to record implementation with detailed artifacts (apiEndpoints, components, functions, classes, integrations), 3) Mark task as complete [x] in tasks.md_

- [x] 1.2 Create useCodePanel hook
  - File: src/hooks/useCodePanel.ts
  - Extract code panel state: isOpen, height, toggleOpen, setHeight with localStorage persistence
  - Purpose: Manage collapsible code panel UI state
  - _Leverage: existing localStorage patterns in codebase_
  - _Requirements: 2.2, TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer specializing in custom hooks and browser storage | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create useCodePanel custom hook that manages code panel state (isOpen, height) with localStorage persistence. Extract from ConfigPage.tsx lines 83-84. Implement toggleOpen callback and setHeight with persistence to 'codePanel.height' key. Default height: 300px. | Restrictions: Do not modify ConfigPage.tsx yet, use localStorage for persistence only (not sessionStorage), memoize callbacks with useCallback, provide cleanup for storage listeners | Success: Hook manages open/closed state, height persists across sessions, callbacks are stable references, unit tests cover open/close toggle and height persistence | After completion: 1) Mark task as in-progress [-] in tasks.md before starting, 2) Use log-implementation tool with detailed artifacts, 3) Mark task as complete [x]_

- [x] 1.3 Create useKeyboardLayout hook
  - File: src/hooks/useKeyboardLayout.ts
  - Extract keyboard layout state and memoized layout keys parsing
  - Purpose: Manage keyboard layout selection and parsed keys
  - _Leverage: src/utils/kle-parser.ts, src/data/layouts/*_
  - _Requirements: 2.3, TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer with expertise in useMemo optimization | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create useKeyboardLayout custom hook that manages layout type selection and memoized layout keys. Extract from ConfigPage.tsx lines 224-231. Accept initialLayout parameter (default 'ANSI_104'). Return { layout, setLayout, layoutKeys } where layoutKeys is memoized and recalculates only when layout changes. Use parseKLEToSVG from kle-parser. | Restrictions: Do not modify ConfigPage.tsx yet, layoutKeys must be properly memoized with useMemo, include layoutData mapping, no unnecessary re-renders | Success: Hook provides layout state and memoized keys, keys only recompute on layout change, unit tests verify memoization, all layout types supported | After completion: 1) Mark task as in-progress [-] in tasks.md, 2) Use log-implementation tool with artifacts, 3) Mark [x]_

- [x] 1.4 Create useConfigSync hook
  - File: src/hooks/useConfigSync.ts
  - Encapsulate RhaiSyncEngine initialization and sync status management
  - Purpose: Centralize config synchronization logic
  - _Leverage: src/components/RhaiSyncEngine.tsx, src/hooks/useRhaiSyncEngine.ts_
  - _Requirements: 2.4, TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer specializing in data synchronization patterns | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create useConfigSync custom hook that encapsulates RhaiSyncEngine initialization and sync status. Extract from ConfigPage.tsx lines 86-96 and 232-234. Accept profileName parameter. Initialize syncEngine with storageKey `profile-${profileName}`, debounceMs: 500. Manage syncStatus ('saved' | 'unsaved' | 'saving') and lastSaveTime state. Return { syncEngine, syncStatus, lastSaveTime, setSyncStatus, setLastSaveTime }. | Restrictions: Do not modify ConfigPage.tsx yet, ensure syncEngine properly reinitializes when profileName changes, handle cleanup on unmount, debounce must be 500ms | Success: Hook initializes syncEngine with correct config, sync status managed properly, cleanup on unmount, unit tests with mocked syncEngine | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [x] 1.5 Write tests for all custom hooks
  - Files: src/hooks/useProfileSelection.test.ts, useCodePanel.test.ts, useKeyboardLayout.test.ts, useConfigSync.test.ts
  - Comprehensive unit tests for each hook with >80% coverage
  - Purpose: Ensure hook reliability and enable safe refactoring
  - _Leverage: tests/testUtils.tsx, @testing-library/react-hooks_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in React hooks testing with @testing-library/react-hooks | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write comprehensive unit tests for useProfileSelection, useCodePanel, useKeyboardLayout, and useConfigSync hooks. Test all state transitions, fallback logic, memoization behavior, side effects, and edge cases. Use renderHook from @testing-library/react. Mock dependencies: useActiveProfileQuery, useRhaiSyncEngine. Achieve >80% coverage. | Restrictions: Test hooks in isolation, mock all external dependencies, no integration tests here, use act() for state updates, verify memoization with toBe reference equality | Success: All hooks have test files, >80% coverage achieved, all state transitions tested, memoization verified, tests pass reliably | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [x] 1.6 Update ConfigPage to use custom hooks
  - File: src/pages/ConfigPage.tsx
  - Replace inline state management with custom hooks
  - Purpose: Simplify ConfigPage by delegating to hooks
  - _Leverage: src/hooks/useProfileSelection.ts, useCodePanel.ts, useKeyboardLayout.ts, useConfigSync.ts_
  - _Requirements: 2.5, TR-3_
  - _Prompt: Role: React Refactoring Specialist with expertise in component simplification | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Refactor ConfigPage.tsx to use the four custom hooks created in tasks 1.1-1.4. Replace lines 56-78 with useProfileSelection, lines 83-84 with useCodePanel, lines 224-231 with useKeyboardLayout, lines 86-96 and 232-234 with useConfigSync. Remove replaced code. Ensure all functionality remains identical. | Restrictions: No behavior changes, maintain all existing props interfaces, existing tests must pass without modification, do not extract components yet (Phase 2), preserve all callbacks and event handlers | Success: ConfigPage uses all four hooks, behavior unchanged, all existing tests pass, line count reduced by ~50 lines, ESLint passes | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

## Phase 2: Extract Container Components

- [x] 2.1 Create KeyboardVisualizerContainer component
  - File: src/components/config/KeyboardVisualizerContainer.tsx
  - Extract keyboard visualization with layout selection and key click handling
  - Purpose: Isolate keyboard display logic
  - _Leverage: src/components/KeyboardVisualizer.tsx, src/hooks/useKeyboardLayout.ts, src/types/index.ts_
  - _Requirements: 3.1, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in container/presentational pattern | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create KeyboardVisualizerContainer component that wraps KeyboardVisualizer with layout management. Use useKeyboardLayout hook. Accept props: { profileName, activeLayer, mappings, onKeyClick, selectedKeyCode }. Render layout selector (dropdown) and KeyboardVisualizer component. Pass layoutKeys, mappings, onKeyClick, selectedKey to KeyboardVisualizer. Extract from ConfigPage.tsx keyboard visualization section. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, use existing KeyboardVisualizer component unchanged, TypeScript strict mode, no direct DOM manipulation | Success: Component renders keyboard with layout selector, handles key clicks, layout changes work, props interface clear, unit tests with mocked KeyboardVisualizer | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 2.2 Create CodePanelContainer component
  - File: src/components/config/CodePanelContainer.tsx
  - Extract collapsible code editor panel with useCodePanel hook
  - Purpose: Isolate code editor UI and state
  - _Leverage: src/components/MonacoEditor.tsx, src/hooks/useCodePanel.ts_
  - _Requirements: 3.2, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in collapsible panels and editors | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create CodePanelContainer component for collapsible code editor. Use useCodePanel hook. Accept props: { profileName, rhaiCode, onChange, syncEngine }. Implement toggle button, resizable panel (drag handle), MonacoEditor integration. Panel height controlled by useCodePanel. Show/hide based on isOpen state. Extract from ConfigPage.tsx code panel section. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, reuse MonacoEditor component, implement resize handle with mouse events, persist height via useCodePanel | Success: Panel toggles open/closed, resizable with drag handle, MonacoEditor displays code, onChange callback works, height persists, unit tests with mocked MonacoEditor | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 2.3 Create ConfigurationPanel component
  - File: src/components/config/ConfigurationPanel.tsx
  - Extract right sidebar with device selector, layer switcher, mappings summary, key config panel, key palette
  - Purpose: Group all configuration controls in one component
  - _Leverage: src/components/DeviceSelector.tsx, LayerSwitcher.tsx, CurrentMappingsSummary.tsx, KeyConfigPanel.tsx, KeyPalette.tsx_
  - _Requirements: 3.3, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in component composition | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create ConfigurationPanel component that composes existing UI components into right sidebar. Accept props: { profileName, selectedPhysicalKey, selectedPaletteKey, onPaletteKeySelect, onSaveMapping, onClearMapping, activeLayer, onLayerChange, devices, selectedDevices, onDeviceToggle }. Render vertical layout: DeviceSelector → LayerSwitcher → KeyPalette → KeyConfigPanel → CurrentMappingsSummary. Extract from ConfigPage.tsx right panel section. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, reuse all existing components unchanged, maintain visual layout, props drilling acceptable for clarity | Success: Panel renders all sub-components in correct order, props passed correctly, callbacks work, visual layout matches original, unit tests with mocked children | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 2.4 Write tests for container components
  - Files: src/components/config/KeyboardVisualizerContainer.test.tsx, CodePanelContainer.test.tsx, ConfigurationPanel.test.tsx
  - Comprehensive unit tests with >80% coverage
  - Purpose: Ensure container reliability before integrating
  - _Leverage: tests/testUtils.tsx, @testing-library/react_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in React component testing | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write unit tests for KeyboardVisualizerContainer, CodePanelContainer, and ConfigurationPanel. Mock all child components and hooks. Test prop passing, callback invocation, conditional rendering, state management. Use @testing-library/react. Achieve >80% coverage. | Restrictions: Mock all child components with jest.mock, test component behavior not implementation, verify callbacks called with correct arguments, no snapshot tests | Success: All containers have test files, >80% coverage, all props and callbacks tested, conditional rendering verified, tests pass reliably | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 2.5 Update ConfigPage to use container components
  - File: src/pages/ConfigPage.tsx
  - Replace inline JSX with container components
  - Purpose: Simplify ConfigPage structure
  - _Leverage: src/components/config/KeyboardVisualizerContainer.tsx, CodePanelContainer.tsx, ConfigurationPanel.tsx_
  - _Requirements: 3.4, TR-3_
  - _Prompt: Role: React Refactoring Specialist focusing on component composition | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Refactor ConfigPage.tsx to use KeyboardVisualizerContainer, CodePanelContainer, and ConfigurationPanel components. Replace keyboard visualization JSX with KeyboardVisualizerContainer. Replace code panel JSX with CodePanelContainer. Replace right sidebar JSX with ConfigurationPanel. Pass required props and callbacks. Remove replaced JSX. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, preserve all state management, do not extract ProfileSelector yet | Success: ConfigPage uses all three containers, behavior unchanged, existing tests pass, line count reduced by ~200 lines, ESLint passes | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

## Phase 3: Extract ProfileSelector Component

- [ ] 3.1 Create ProfileSelector component
  - File: src/components/config/ProfileSelector.tsx
  - Extract profile selection dropdown and create profile button
  - Purpose: Centralize profile UI in dedicated component
  - _Leverage: src/hooks/useProfiles.ts, src/hooks/useCreateProfile.ts_
  - _Requirements: 4.1, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in forms and selectors | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create ProfileSelector component with dropdown for profile selection and button for profile creation. Accept props: { value, onChange, profiles, isLoading, onCreateProfile }. Render select element with profile options, loading state, create button. Handle profile creation flow (modal or inline). Extract from ConfigPage.tsx profile selector section. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, accessible (ARIA labels), loading state indicator, create button only enabled when valid name entered | Success: Component renders dropdown with profiles, selection changes call onChange, create button triggers onCreateProfile, loading state displays, unit tests with user events | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 3.2 Write tests for ProfileSelector
  - File: src/components/config/ProfileSelector.test.tsx
  - Test profile selection, creation, loading states
  - Purpose: Ensure ProfileSelector reliability
  - _Leverage: tests/testUtils.tsx, @testing-library/user-event_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in user interaction testing | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write unit tests for ProfileSelector component. Test profile selection changes, create profile button click, loading state display, disabled states, accessibility. Use userEvent for interactions. Achieve >80% coverage. | Restrictions: Test user-facing behavior, verify callbacks called with correct values, test edge cases (empty profiles, loading), check ARIA attributes | Success: Test file created, >80% coverage, selection and creation tested, loading states verified, accessibility checked, tests pass | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 3.3 Update ConfigPage to use ProfileSelector
  - File: src/pages/ConfigPage.tsx
  - Replace profile selection JSX with ProfileSelector component
  - Purpose: Further simplify ConfigPage
  - _Leverage: src/components/config/ProfileSelector.tsx_
  - _Requirements: 4.2, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Refactor ConfigPage.tsx to use ProfileSelector component. Replace profile selection JSX (select/dropdown and create button) with <ProfileSelector>. Pass value={selectedProfileName}, onChange={setSelectedProfileName}, profiles, isLoading, onCreateProfile props. Remove replaced JSX. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, preserve profile creation flow | Success: ConfigPage uses ProfileSelector, behavior unchanged, tests pass, line count reduced by ~30 lines, ESLint passes | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

## Phase 4: Extract ConfigurationLayout Component

- [ ] 4.1 Create ConfigurationLayout component
  - File: src/components/config/ConfigurationLayout.tsx
  - Extract main layout structure with responsive grid
  - Purpose: Centralize layout logic for config page
  - _Leverage: existing CSS grid patterns in codebase_
  - _Requirements: 5.1, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in responsive layouts | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create ConfigurationLayout component that provides responsive grid layout for config page. Accept props: { profileName, children }. Implement CSS grid with areas for keyboard (left), config panel (right), code panel (bottom). Support resizable panels and responsive breakpoints. Extract layout structure from ConfigPage.tsx. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, use CSS Grid or Flexbox, responsive (mobile/tablet/desktop), resizable panel boundaries | Success: Layout renders children in correct grid areas, responsive at all breakpoints, resizable panels work, unit tests verify rendering | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 4.2 Write tests for ConfigurationLayout
  - File: src/components/config/ConfigurationLayout.test.tsx
  - Test layout rendering and responsive behavior
  - Purpose: Ensure layout reliability
  - _Leverage: tests/testUtils.tsx_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in layout testing | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write unit tests for ConfigurationLayout component. Test children rendering in correct positions, responsive breakpoints (mock window.matchMedia), grid structure, resizable behavior. Achieve >80% coverage. | Restrictions: Mock window.matchMedia for responsive tests, verify DOM structure, test resize handlers, no visual regression tests | Success: Test file created, >80% coverage, children placement verified, responsive behavior tested, tests pass | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 4.3 Update ConfigPage to use ConfigurationLayout
  - File: src/pages/ConfigPage.tsx
  - Wrap components in ConfigurationLayout
  - Purpose: Final ConfigPage simplification
  - _Leverage: src/components/config/ConfigurationLayout.tsx_
  - _Requirements: 5.2, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Refactor ConfigPage.tsx to use ConfigurationLayout component. Wrap KeyboardVisualizerContainer, ConfigurationPanel, and CodePanelContainer in <ConfigurationLayout>. Remove manual layout/styling code from ConfigPage. ConfigPage should now be just: ProfileSelector, ConfigurationLayout with children. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, verify responsive behavior unchanged | Success: ConfigPage uses ConfigurationLayout, behavior unchanged, tests pass, line count reduced to <200 lines, main component function <50 lines | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

## Phase 5: Testing & Cleanup

- [ ] 5.1 Update ConfigPage tests
  - File: src/pages/ConfigPage.test.tsx
  - Update tests to match new component structure
  - Purpose: Ensure ConfigPage integration tests pass
  - _Leverage: tests/testUtils.tsx, new component mocks_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer specializing in integration testing | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Update ConfigPage.test.tsx to test refactored component. Mock ProfileSelector, ConfigurationLayout, KeyboardVisualizerContainer, ConfigurationPanel, CodePanelContainer. Test composition and prop passing. Verify all integration scenarios work. Maintain or improve coverage. | Restrictions: Test integration not implementation details, mock child components, verify data flow through props/callbacks, maintain existing test scenarios | Success: ConfigPage tests updated and passing, coverage maintained/improved, integration scenarios verified, no failing tests | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.2 Run full test suite and fix failures
  - Run npm test and fix any failures
  - Purpose: Ensure no regressions introduced
  - _Leverage: all test files, debugging tools_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer with debugging expertise | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Run full test suite (npm test). Identify and fix any failing tests caused by refactoring. Debug issues, update mocks if needed, fix broken assertions. Ensure all tests pass. Run coverage report and verify >80% coverage maintained. | Restrictions: Do not skip or disable tests, fix root causes not symptoms, maintain test quality, no snapshot updates without verification | Success: All tests pass (npm test), coverage >80%, no skipped tests, no console errors, test run time similar to before | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.3 Fix ESLint errors and apply Prettier
  - Run npm run lint:fix and npm run format
  - Purpose: Ensure code quality compliance
  - _Leverage: .eslintrc.js, .prettierrc_
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Run ESLint (npm run lint) on all refactored files. Fix all errors and warnings. Run Prettier (npm run format) to apply consistent formatting. Verify no TypeScript errors (tsc --noEmit). Ensure all files follow project conventions. | Restrictions: Fix all ESLint errors, do not disable rules, apply Prettier to all changed files, verify TypeScript compiles | Success: ESLint 0 errors/warnings, Prettier applied to all files, TypeScript compiles, code follows project style guide | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.4 Verify code metrics compliance
  - Check all files ≤500 lines, all functions ≤50 lines
  - Purpose: Confirm code quality standards met
  - _Leverage: scripts/verify_file_sizes.sh or manual verification_
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Auditor | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Verify all refactored files meet code quality metrics. Check ConfigPage.tsx ≤200 lines (target: <200 code lines). Check all new files ≤500 lines. Check all functions ≤50 lines. Use scripts/verify_file_sizes.sh if available or manual count. Document metrics in completion report. | Restrictions: Count code lines only (exclude comments/blanks), identify any violations, create summary report | Success: ConfigPage.tsx <200 code lines, all new files ≤500 lines, all functions ≤50 lines, metrics documented, report created | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.5 Update documentation
  - Update README, add JSDoc comments to new components/hooks
  - Purpose: Ensure maintainability
  - _Leverage: existing documentation patterns_
  - _Requirements: NFR-2_
  - _Prompt: Role: Technical Writer with React expertise | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Add JSDoc comments to all new components (ProfileSelector, ConfigurationLayout, KeyboardVisualizerContainer, ConfigurationPanel, CodePanelContainer) and hooks (useProfileSelection, useCodePanel, useKeyboardLayout, useConfigSync). Document props, return values, usage examples. Update keyrx_ui/README.md with new component structure. Add component hierarchy diagram if helpful. | Restrictions: Use JSDoc format with @param, @returns, @example tags, document all public APIs, keep docs concise and accurate | Success: All new components/hooks have JSDoc comments, README updated with new structure, docs are clear and helpful, examples provided | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.6 Run integration and E2E tests
  - Run npm run test:integration and npm run test:e2e
  - Purpose: Verify end-to-end functionality unchanged
  - _Leverage: existing integration and E2E test suites_
  - _Requirements: TR-3, NFR-3_
  - _Prompt: Role: QA Engineer specializing in integration and E2E testing | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Run integration tests (npm run test:integration) and E2E tests (npm run test:e2e) if they exist. Verify all tests pass without modification. Debug any failures. Ensure user-facing behavior is identical. Record test results and any issues found. | Restrictions: Do not modify E2E tests unless absolutely necessary, verify actual user flows work, test in browser if E2E uses Playwright/Cypress | Success: All integration tests pass, all E2E tests pass, user flows verified, no behavior changes detected, test results documented | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.7 Performance verification
  - Benchmark render performance before/after refactoring
  - Purpose: Ensure no performance degradation
  - _Leverage: React DevTools Profiler, Chrome DevTools Performance_
  - _Requirements: NFR-1_
  - _Prompt: Role: Performance Engineer specializing in React optimization | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Benchmark ConfigPage render performance. Use React DevTools Profiler to measure render times, re-render counts, committed changes. Compare before/after metrics. Test key interactions: profile change, key click, layer change, code edit. Verify no performance degradation (±5% acceptable). Document results. | Restrictions: Test in production build (npm run build), use consistent hardware/browser, measure multiple runs, identify any performance issues | Success: Render performance within ±5% of baseline, no unnecessary re-renders introduced, memoization effective, performance report created | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 5.8 Final verification and cleanup
  - Remove any dead code, verify all requirements met
  - Purpose: Complete refactoring and verify success
  - _Leverage: all project files_
  - _Requirements: All_
  - _Prompt: Role: Senior Developer with code review expertise | Task: Implement the task for spec refactor-config-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Perform final verification of refactoring. Review all modified files for dead code, unused imports, TODOs. Verify all requirements met: ConfigPage <200 lines, all functions ≤50 lines, all tests pass, coverage >80%, ESLint 0 errors, docs updated. Create final checklist verification. Remove any temporary code or comments. | Restrictions: Remove all dead code, no TODOs left, all imports used, clean git diff, ready for PR | Success: All requirements verified and met, no dead code, clean codebase, final checklist completed, ready for code review | After completion: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_
