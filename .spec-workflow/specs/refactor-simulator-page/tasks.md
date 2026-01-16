# Tasks: Refactor SimulatorPage Component

- [x] 1. Create useSimulation hook
  - File: src/hooks/useSimulation.ts
  - Extract simulation state management: events array, isRunning, WebSocket subscription
  - Return { events, isRunning, addEvent, clearEvents, start, stop, statistics }
  - Purpose: Centralize simulation state for reusability and testability
  - _Leverage: src/hooks/useUnifiedApi.ts for WebSocket, existing event types_
  - _Requirements: 1.4, TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer specializing in WebSocket integration | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create useSimulation hook that manages simulation state (events, isRunning) and WebSocket subscription. Extract from SimulatorPage.tsx lines 39-140. Accept options: { maxEvents (default 1000), autoStart (default false) }. Manage events array with max limit (FIFO queue). Implement start/stop to control WebSocket subscription. Compute statistics (event counts by type). | Restrictions: Must be ≤500 lines, all functions ≤50 lines, properly cleanup WebSocket on unmount, memoize callbacks, handle edge cases (rapid start/stop) | Success: Hook manages events and WebSocket, max events enforced, statistics computed, unit tests with mocked WebSocket >80% coverage | After completion: 1) Mark [-] in tasks.md, 2) log-implementation with artifacts, 3) Mark [x]_

- [ ] 2. Create EventList component with virtualization
  - File: src/components/simulator/EventList.tsx
  - Display virtualized event list with auto-scroll to latest
  - Purpose: Performant event display for 1000+ events
  - _Leverage: react-window or react-virtualized, existing event formatting_
  - _Requirements: 2.1, TR-1, TR-2, TR-4_
  - _Prompt: Role: React Component Developer specializing in performance optimization | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create EventList component with virtualized rendering using react-window. Accept props: { events, maxEvents, onClear, virtualizeThreshold (default 100) }. Render event list with auto-scroll to latest event. Use FixedSizeList from react-window when events > virtualizeThreshold. Format event display with timestamp, key code, event type. Extract from SimulatorPage.tsx event list section. | Restrictions: Must be ≤500 lines, all functions ≤50 lines, virtualize only when needed, auto-scroll smooth, accessible (ARIA labels) | Success: Component renders events efficiently, virtualization works at threshold, auto-scroll functions, unit tests with various event counts | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 3. Create SimulationControls component
  - File: src/components/simulator/SimulationControls.tsx
  - Extract start/stop/clear buttons and statistics display
  - Purpose: Group simulation controls in dedicated component
  - _Leverage: existing button components_
  - _Requirements: 1.1, TR-1, TR-2_
  - _Prompt: Role: React Component Developer focusing on UI controls | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create SimulationControls component with start/stop/clear buttons and statistics display. Accept props: { isRunning, eventCount, onStart, onStop, onClear, statistics }. Render button group (Start/Stop toggle, Clear button) and stats panel (event count, events/sec, etc). Extract from SimulatorPage.tsx controls section. Disable clear when no events. | Restrictions: Must be ≤200 lines, all functions ≤50 lines, accessible buttons, visual feedback for running state | Success: Controls render correctly, callbacks invoked on click, disabled states work, statistics displayed, unit tests with user events | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 4. Create EventInjectionForm component
  - File: src/components/simulator/EventInjectionForm.tsx
  - Extract event injection form with key selector and event type selector
  - Purpose: Isolate injection UI and validation
  - _Leverage: src/components/KeyPalette.tsx for key selection_
  - _Requirements: 3.1, TR-1, TR-2_
  - _Prompt: Role: React Component Developer specializing in forms | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Create EventInjectionForm component with key code input, event type selector (press/release), and inject button. Accept props: { onInjectEvent, disabled }. Validate key code (non-empty). Call onInjectEvent with validated data on submit. Extract from SimulatorPage.tsx injection form section. | Restrictions: Must be ≤200 lines, all functions ≤50 lines, validate inputs before submission, accessible form labels, disabled prop disables all inputs | Success: Form renders inputs, validation prevents invalid submission, onInjectEvent called with correct data, unit tests cover validation | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 5. Write tests for useSimulation hook
  - File: src/hooks/useSimulation.test.ts
  - Comprehensive tests for hook state management and WebSocket
  - Purpose: Ensure hook reliability
  - _Leverage: @testing-library/react-hooks, WebSocket mocks_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in hooks testing | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write unit tests for useSimulation hook. Test event addition, max events enforcement (FIFO), start/stop state transitions, WebSocket subscription lifecycle, statistics computation. Mock WebSocket. Use renderHook from @testing-library/react. Achieve >80% coverage. | Restrictions: Mock WebSocket completely, test in isolation, verify cleanup, test edge cases (max events, rapid state changes) | Success: All hook behaviors tested, >80% coverage, WebSocket lifecycle verified, statistics accurate, tests pass | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 6. Write tests for all components
  - Files: EventList.test.tsx, SimulationControls.test.tsx, EventInjectionForm.test.tsx
  - Unit tests for each component with >80% coverage
  - Purpose: Ensure component reliability
  - _Leverage: @testing-library/react, @testing-library/user-event_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in React component testing | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Write unit tests for EventList, SimulationControls, and EventInjectionForm components. Test rendering, prop handling, callbacks, conditional logic, user interactions. Mock dependencies (react-window, etc). Use userEvent for interactions. Achieve >80% coverage per component. | Restrictions: Test behavior not implementation, mock heavy dependencies, verify callbacks called correctly, test accessibility | Success: All components have tests, >80% coverage each, interactions tested, accessibility verified, tests pass | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 7. Update SimulatorPage to use extracted components
  - File: src/pages/SimulatorPage.tsx
  - Replace inline JSX with new components
  - Purpose: Simplify SimulatorPage structure
  - _Leverage: useSimulation hook, EventList, SimulationControls, EventInjectionForm components_
  - _Requirements: 4.1, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Refactor SimulatorPage.tsx to use useSimulation hook and extracted components. Replace event management with useSimulation. Replace controls JSX with SimulationControls. Replace event list JSX with EventList. Replace injection form with EventInjectionForm. Remove replaced code. Pass props and callbacks. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, verify WebSocket connection unchanged | Success: SimulatorPage uses all new components, behavior unchanged, tests pass, line count reduced to <300 lines, ESLint passes | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 8. Update SimulatorPage tests
  - File: src/pages/SimulatorPage.test.tsx
  - Update tests to match new component structure
  - Purpose: Ensure integration tests pass
  - _Leverage: component mocks_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer specializing in integration testing | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Update SimulatorPage.test.tsx to test refactored page. Mock useSimulation, EventList, SimulationControls, EventInjectionForm. Test composition and prop passing. Verify integration scenarios. Maintain/improve coverage. | Restrictions: Test integration not details, mock children, verify data flow, maintain test scenarios | Success: Tests updated and passing, coverage maintained/improved, integration verified, no failing tests | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 9. Fix ESLint errors and apply Prettier
  - Run linting and formatting on all modified files
  - Purpose: Ensure code quality compliance
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Run ESLint on all refactored files, fix errors/warnings. Run Prettier to format. Verify TypeScript compiles (tsc --noEmit). | Restrictions: Fix all ESLint errors, apply Prettier to all files, verify TS compiles | Success: ESLint 0 errors, Prettier applied, TypeScript compiles, code follows style | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [ ] 10. Performance verification and final checks
  - Benchmark performance, verify metrics, update docs
  - Purpose: Confirm success criteria met
  - _Requirements: TR-4, All_
  - _Prompt: Role: Senior Developer with performance expertise | Task: Implement the task for spec refactor-simulator-page. First run spec-workflow-guide to get the workflow guide, then implement the task: Verify SimulatorPage refactoring success. Test event list performance with 1000+ events (should be smooth with virtualization). Verify SimulatorPage <300 lines, all functions ≤50 lines. Run full test suite. Update docs. Create verification checklist. | Restrictions: Test with production build, verify all metrics, document results | Success: Performance verified (smooth with 1000+ events), all metrics met, tests pass, docs updated, ready for review | After completion: 1) Mark [-], 2) log-implementation, 3) Mark [x]_
