# Tasks: Remaining Code Quality Fixes

## Phase 1: Extract Shared KeyConfig Components

- [x] 1.1 Create MappingTypeSelector component
  - File: src/components/keyConfig/MappingTypeSelector.tsx
  - Reusable mapping type selector (simple, modifier, lock, tap_hold, layer_active)
  - Purpose: DRY between KeyConfigModal and KeyConfigPanel
  - _Leverage: existing mapping type UI patterns from both Modal and Panel_
  - _Requirements: 1.1, TR-1, TR-4_
  - _Prompt: Role: React Component Developer specializing in reusable UI components | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide to get the workflow guide, then implement the task: Create MappingTypeSelector component that displays mapping type options as radio buttons or tabs. Extract from KeyConfigModal.tsx (lines with MAPPING_TYPE_CONFIG) and KeyConfigPanel.tsx. Accept props: { selectedType: MappingType, onChange: (type) => void, supportedTypes: MappingType[], layout: 'horizontal' | 'vertical' (default horizontal) }. Render icon + label + description for each type. Highlight selected type. Call onChange on click. | Restrictions: Must be ≤300 lines, all functions ≤50 lines, accessible (ARIA radio group), support both Modal (5 types) and Panel (2 types) via supportedTypes prop | Success: Component renders type options, selection works, onChange called, supports both layouts, accessible, unit tests with user events >80% coverage | After completion: 1) Mark task as in-progress [-] in tasks.md before starting, 2) Use log-implementation tool to record implementation with detailed artifacts (components created, props interface, usage examples), 3) Mark task as complete [x] in tasks.md_

- [x] 1.2 Create KeySelectionTabs component
  - File: src/components/keyConfig/KeySelectionTabs.tsx
  - Tabbed interface for key selection (keyboard, modifier, lock, layer tabs)
  - Purpose: DRY key selection UI between Modal and Panel
  - _Leverage: existing tab patterns, SVGKeyboard component_
  - _Requirements: 1.2, TR-1, TR-4_
  - _Prompt: Role: React Component Developer specializing in tabbed interfaces | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create KeySelectionTabs component with tabs for different key categories. Extract from KeyConfigModal and KeyConfigPanel. Accept props: { activeTab: string, onTabChange: (tab) => void, availableTabs: string[], onKeySelect: (keyCode) => void, layoutKeys?: SVGKey[] }. Render tab buttons (Keyboard, Modifier, Lock, Layer). Show SVGKeyboard or key grid based on active tab. Call onKeySelect when key clicked. | Restrictions: Must be ≤400 lines, all functions ≤50 lines, accessible tabs (ARIA tablist), keyboard navigation, support subset of tabs via availableTabs | Success: Tabs render and switch correctly, keyboard display works, key selection calls onKeySelect, accessible, unit tests >80% coverage | After: 1) Mark [-], 2) log-implementation with artifacts, 3) Mark [x]_

- [x] 1.3 Create MappingConfigForm component
  - File: src/components/keyConfig/MappingConfigForm.tsx
  - Dynamic form fields based on mapping type (target key, modifiers, timings, etc)
  - Purpose: DRY configuration forms between Modal and Panel
  - _Leverage: existing form patterns from Modal and Panel_
  - _Requirements: 1.3, TR-1, TR-4_
  - _Prompt: Role: React Component Developer specializing in dynamic forms | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create MappingConfigForm component that renders form fields based on mappingType. Extract form rendering from KeyConfigModal and KeyConfigPanel. Accept props: { mappingType: MappingType, currentConfig?: Partial<KeyMapping>, onChange: (config) => void, onValidate: (config) => ValidationResult }. Render different fields per type: simple (target key), modifier (modifier key), tap_hold (tap key, hold key, threshold), lock (target key), layer_active (layer number). Call onChange on field change. Validate on blur. | Restrictions: Must be ≤400 lines, all functions ≤50 lines, validate inputs (required fields, numeric ranges), accessible form labels, error messages | Success: Form renders correct fields per type, validation works, onChange called with updates, error display, unit tests >80% coverage | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 1.4 Write tests for shared keyConfig components
  - Files: MappingTypeSelector.test.tsx, KeySelectionTabs.test.tsx, MappingConfigForm.test.tsx
  - Comprehensive unit tests with >80% coverage
  - Purpose: Ensure shared components work before integration
  - _Leverage: @testing-library/react, @testing-library/user-event_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer specializing in React component testing | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Write unit tests for all three shared keyConfig components. For MappingTypeSelector: test type selection, onChange callback, layout variants, disabled types. For KeySelectionTabs: test tab switching, key selection, available tabs filtering, accessibility. For MappingConfigForm: test all mapping types, field validation, onChange/onValidate callbacks, error display. Use userEvent for interactions. Achieve >80% coverage per component. | Restrictions: Test behavior not implementation, mock SVGKeyboard, verify callbacks with correct args, test accessibility (ARIA attributes) | Success: All components tested, >80% coverage each, interactions verified, accessibility checked, tests pass | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

## Phase 2: Refactor KeyConfigModal

- [x] 2.1 Refactor KeyConfigModal to use shared components
  - File: src/components/KeyConfigModal.tsx
  - Replace inline UI with MappingTypeSelector, KeySelectionTabs, MappingConfigForm
  - Purpose: Reduce KeyConfigModal from 641 to <500 lines
  - _Leverage: shared components from Phase 1_
  - _Requirements: 1.1, TR-1, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Refactor KeyConfigModal.tsx to use shared components. Import MappingTypeSelector, KeySelectionTabs, MappingConfigForm. Replace mapping type selection UI with <MappingTypeSelector>. Replace key selection tabs with <KeySelectionTabs>. Replace configuration forms with <MappingConfigForm>. Keep Modal wrapper, orchestration logic, save/cancel handlers. Remove replaced code. Target <500 lines. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, preserve modal UX, all props interfaces unchanged | Success: KeyConfigModal uses shared components, behavior unchanged, <500 lines, all functions ≤50 lines, tests pass, ESLint passes | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 2.2 Update KeyConfigModal tests
  - File: src/components/KeyConfigModal.test.tsx
  - Update tests to match new component structure
  - Purpose: Ensure KeyConfigModal integration tests pass
  - _Leverage: shared component mocks_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer specializing in integration testing | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Update KeyConfigModal.test.tsx to test refactored modal. Mock MappingTypeSelector, KeySelectionTabs, MappingConfigForm. Test modal open/close, component composition, prop passing to children, save/cancel callbacks. Verify all integration scenarios work. Maintain or improve coverage. | Restrictions: Test integration not details, mock shared components, verify data flow through props, maintain existing test scenarios | Success: KeyConfigModal tests updated and passing, coverage maintained/improved, integration verified, no failing tests | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

## Phase 3: Refactor KeyConfigPanel

- [x] 3.1 Refactor KeyConfigPanel to use shared components
  - File: src/components/KeyConfigPanel.tsx
  - Replace inline UI with shared components (same as Modal)
  - Purpose: Reduce KeyConfigPanel from 634 to <500 lines
  - _Leverage: shared components from Phase 1_
  - _Requirements: 1.2, TR-1, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Refactor KeyConfigPanel.tsx to use shared components. Import MappingTypeSelector, KeySelectionTabs, MappingConfigForm. Replace mapping type selection UI with <MappingTypeSelector supportedTypes={['simple', 'tap_hold']}>. Replace key selection tabs with <KeySelectionTabs>. Replace configuration forms with <MappingConfigForm>. Keep inline panel wrapper, save/clear handlers. Remove replaced code. Target <500 lines. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, preserve inline panel UX, simplified types (only simple + tap_hold) | Success: KeyConfigPanel uses shared components, behavior unchanged, <500 lines, all functions ≤50 lines, tests pass, ESLint passes | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 3.2 Update KeyConfigPanel tests
  - File: src/components/KeyConfigPanel.test.tsx
  - Update tests to match new structure
  - Purpose: Ensure KeyConfigPanel integration tests pass
  - _Leverage: shared component mocks_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Update KeyConfigPanel.test.tsx to test refactored panel. Mock shared components. Test panel rendering, component composition, save/clear callbacks. Verify integration scenarios. Maintain/improve coverage. | Restrictions: Test integration, mock components, verify data flow, maintain scenarios | Success: Tests passing, coverage maintained, integration verified | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

## Phase 4: Extract MetricsPage Components

- [x] 4.1 Create MetricsStatsCards component
  - File: src/components/metrics/MetricsStatsCards.tsx
  - Display 4 metric stat cards (latency, throughput, CPU, memory)
  - Purpose: Extract metrics display from MetricsPage
  - _Leverage: existing Card component, lucide-react icons_
  - _Requirements: 2.1, TR-1_
  - _Prompt: Role: React Component Developer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create MetricsStatsCards component that renders 4 stat cards. Extract from MetricsPage.tsx metrics cards section. Accept props: { latencyStats: LatencyStats, eventCount: number, connected: boolean }. Render grid of 4 cards: Avg Latency (Activity icon), Throughput (Zap icon), CPU Usage (Cpu icon), Memory Usage (FileCode icon). Display values with units. Show connection status. | Restrictions: Must be ≤300 lines, all functions ≤50 lines, responsive grid (2x2 on desktop, stacked on mobile), loading states | Success: Cards render with correct data, responsive layout, icons display, connection indicator works, unit tests >80% coverage | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4.2 Create LatencyChart component
  - File: src/components/metrics/LatencyChart.tsx
  - Recharts line chart for latency over time
  - Purpose: Extract chart rendering from MetricsPage
  - _Leverage: recharts, existing chart patterns_
  - _Requirements: 2.2, TR-1_
  - _Prompt: Role: React Component Developer specializing in data visualization | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create LatencyChart component wrapping recharts LineChart. Extract from MetricsPage.tsx chart section. Accept props: { data: LatencyDataPoint[], maxDataPoints?: number (default 60) }. Render ResponsiveContainer with LineChart, XAxis (time), YAxis (latency ms), Line, CartesianGrid, Tooltip. Format timestamps on X axis. Show units on Y axis. | Restrictions: Must be ≤250 lines, all functions ≤50 lines, responsive (adapts to container), handle empty data, memoize expensive operations | Success: Chart renders with data, responsive, axes formatted, tooltip works, handles empty state, unit tests with mock data >80% coverage | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4.3 Create EventLogList component
  - File: src/components/metrics/EventLogList.tsx
  - Virtualized event log list with react-window
  - Purpose: Extract event log from MetricsPage
  - _Leverage: react-window FixedSizeList, existing virtualization patterns_
  - _Requirements: 2.3, TR-1_
  - _Prompt: Role: React Component Developer specializing in virtualization | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create EventLogList component with virtualized list. Extract from MetricsPage.tsx event log section. Accept props: { events: EventLogEntry[], maxEvents?: number }. Use FixedSizeList from react-window for performance. Render event rows with timestamp, type, keyCode, latency, device. Format timestamps. Color-code event types. Auto-scroll to latest. | Restrictions: Must be ≤300 lines, all functions ≤50 lines, virtualize for 1000+ events, accessible (ARIA labels), auto-scroll smooth | Success: List renders events, virtualization works efficiently, formatting correct, auto-scroll functions, handles large lists, unit tests >80% coverage | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4.4 Create StateSnapshot component
  - File: src/components/metrics/StateSnapshot.tsx
  - Display current daemon state (layer, modifiers, locks)
  - Purpose: Extract state display from MetricsPage
  - _Leverage: existing Card component_
  - _Requirements: 2.4, TR-1_
  - _Prompt: Role: React Component Developer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Create StateSnapshot component displaying current state. Extract from MetricsPage.tsx state section. Accept props: { state: StateSnapshot }. Render Card with sections: Active Layer, Active Modifiers (badges), Locks (badges), Tap/Hold Timers (count), Queued Events (count). Use badge/pill UI for modifiers/locks. Show counts for timers/queue. | Restrictions: Must be ≤200 lines, all functions ≤50 lines, handle empty state (no modifiers/locks), clear visual hierarchy | Success: State displays correctly, badges render for modifiers/locks, counts shown, empty state handled, unit tests >80% coverage | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4.5 Write tests for metrics components
  - Files: MetricsStatsCards.test.tsx, LatencyChart.test.tsx, EventLogList.test.tsx, StateSnapshot.test.tsx
  - Comprehensive unit tests with >80% coverage
  - Purpose: Ensure metrics components work before integration
  - _Leverage: @testing-library/react, mocked data_
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Write unit tests for all four metrics components. For MetricsStatsCards: test card rendering, data display, connection status. For LatencyChart: test chart rendering with recharts mock, data handling, empty state. For EventLogList: test event rendering, virtualization, auto-scroll. For StateSnapshot: test state display, badges, empty state. Achieve >80% coverage per component. | Restrictions: Mock recharts and react-window, test behavior, verify data rendering, test edge cases (empty data, null states) | Success: All components tested, >80% coverage each, rendering verified, edge cases covered, tests pass | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

## Phase 5: Refactor MetricsPage

- [x] 5.1 Refactor MetricsPage to use extracted components
  - File: src/pages/MetricsPage.tsx
  - Replace inline JSX with MetricsStatsCards, LatencyChart, EventLogList, StateSnapshot
  - Purpose: Reduce MetricsPage from 532 to <500 lines
  - _Leverage: metrics components from Phase 4_
  - _Requirements: 3.1, TR-1, TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Refactor MetricsPage.tsx to use extracted components. Import MetricsStatsCards, LatencyChart, EventLogList, StateSnapshot. Replace metrics cards section with <MetricsStatsCards latencyStats={latencyStats} eventCount={storeEventLog.length} connected={connected}>. Replace chart with <LatencyChart data={latencyHistory}>. Replace event log with <EventLogList events={storeEventLog}>. Replace state display with <StateSnapshot state={storeState}>. Keep WebSocket subscription logic, latency history state management. Remove replaced JSX. Target <500 lines. | Restrictions: No behavior changes, maintain all functionality, existing tests must pass, preserve WebSocket logic, latency history updates unchanged | Success: MetricsPage uses all components, behavior unchanged, <500 lines, all functions ≤50 lines, tests pass, ESLint passes | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 5.2 Update MetricsPage tests
  - File: src/pages/MetricsPage.test.tsx
  - Update tests to match new component structure
  - Purpose: Ensure MetricsPage integration tests pass
  - _Leverage: metrics component mocks_
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Update MetricsPage.test.tsx to test refactored page. Mock MetricsStatsCards, LatencyChart, EventLogList, StateSnapshot. Test page rendering, component composition, prop passing to children, WebSocket subscription, data updates. Verify integration scenarios. Maintain/improve coverage. | Restrictions: Test integration not details, mock components, verify data flow, test WebSocket lifecycle, maintain scenarios | Success: Tests passing, coverage maintained/improved, integration verified, WebSocket tested | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

## Phase 6: Cleanup & Verification

- [x] 6.1 Fix ESLint errors in all modified files
  - Run ESLint on all refactored files and fix errors
  - Purpose: Ensure code quality compliance
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Run ESLint (npm run lint) on all files modified in this spec: KeyConfigModal, KeyConfigPanel, MetricsPage, and all new components in keyConfig/ and metrics/ directories. Fix all errors and warnings. Common fixes: remove unused imports, fix any types, remove console statements. Run lint:fix for auto-fixable issues. | Restrictions: Fix all errors, do not disable rules, maintain type safety, verify no new errors introduced | Success: ESLint 0 errors/warnings on all modified files, code follows project style guide, TypeScript compiles | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6.2 Apply Prettier to all modified files
  - Format all refactored code with Prettier
  - Purpose: Ensure consistent formatting
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Run Prettier (npm run format) on all modified files. Verify formatting applied correctly. Review git diff to ensure only formatting changes for existing files, and proper formatting for new files. | Restrictions: Only formatting changes, no code logic changes, apply to all modified files | Success: Prettier applied to all files, formatting consistent, format:check passes | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6.3 Run full test suite and verify coverage
  - Run all tests and ensure >80% coverage maintained
  - Purpose: Confirm no regressions
  - _Requirements: TR-2, TR-3_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Run full test suite (npm test). Verify all tests pass. Run coverage report (npm run test:coverage:unit). Verify >80% coverage maintained for all modified files. Fix any failing tests. Debug any coverage gaps. | Restrictions: All tests must pass, coverage must be >80%, no tests skipped, fix root causes not symptoms | Success: All tests pass (0 failures), coverage >80% maintained, no skipped tests, coverage report shows compliance | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6.4 Verify code metrics compliance
  - Check all files ≤500 lines, all functions ≤50 lines
  - Purpose: Confirm quality standards met
  - _Requirements: TR-1_
  - _Prompt: Role: Code Quality Auditor | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Verify all refactored files meet code quality metrics. Check KeyConfigModal.tsx ≤500 lines (target from 641). Check KeyConfigPanel.tsx ≤500 lines (target from 634). Check MetricsPage.tsx ≤500 lines (target from 532). Check all new component files ≤500 lines. Check all functions ≤50 lines. Count code lines (exclude comments/blanks). Create metrics report. | Restrictions: Count code only, identify any violations, document all metrics | Success: All 3 main files ≤500 lines, all new files ≤500 lines, all functions ≤50 lines, metrics documented in report | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6.5 Update documentation
  - Add JSDoc comments to new components, update README
  - Purpose: Ensure maintainability
  - _Requirements: TR-1_
  - _Prompt: Role: Technical Writer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Add JSDoc comments to all new components: MappingTypeSelector, KeySelectionTabs, MappingConfigForm (keyConfig/), MetricsStatsCards, LatencyChart, EventLogList, StateSnapshot (metrics/). Document props, purpose, usage examples. Update keyrx_ui/README.md with new component sections: "Shared KeyConfig Components" and "Metrics Components". Include component hierarchy and usage patterns. | Restrictions: Use JSDoc format with @param, @returns, @example tags, document all public APIs, keep docs concise | Success: All new components have JSDoc, README updated with new sections, docs clear and helpful, usage examples provided | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6.6 Final verification and cleanup
  - Final check: all requirements met, no dead code, ready for review
  - Purpose: Complete refactoring verification
  - _Requirements: All_
  - _Prompt: Role: Senior Developer | Task: Implement the task for spec remaining-quality-fixes. First run spec-workflow-guide, then implement: Perform final verification of refactoring. Review all modified files for dead code, unused imports, TODOs. Verify all requirements met: KeyConfigModal ≤500 lines, KeyConfigPanel ≤500 lines, MetricsPage ≤500 lines, all new components created and tested, shared components reused (DRY), all functions ≤50 lines, all tests pass, coverage >80%, ESLint 0 errors, docs updated. Create final verification checklist. Remove any temporary code or comments. | Restrictions: Remove all dead code, no TODOs left, all imports used, clean git diff, ready for PR | Success: All requirements verified and met, no dead code, clean codebase, final checklist completed, ready for code review | After: 1) Mark [-], 2) log-implementation with final artifacts summary, 3) Mark [x]_
