# Tasks: Refactor KeyPalette Component

- [x] 1. Extract useRecentKeys hook
  - File: src/hooks/useRecentKeys.ts
  - Manage recent keys with localStorage (max 10 keys, FIFO)
  - _Leverage: localStorage patterns_
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create useRecentKeys hook managing recent keys with localStorage. Extract from KeyPalette.tsx lines 259-289. Return { recentKeys: string[], addRecentKey(id), clearRecentKeys }. Store in 'keyrx_recent_keys', max 10 keys FIFO. | Restrictions: ≤200 lines, memoize callbacks, handle storage errors | Success: Hook manages recent keys, FIFO enforced, localStorage persists, unit tests >80% | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 2. Extract useFavoriteKeys hook
  - File: src/hooks/useFavoriteKeys.ts
  - Manage favorite keys with localStorage
  - _Leverage: localStorage patterns_
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create useFavoriteKeys hook managing favorites with localStorage. Extract from KeyPalette.tsx lines 291-323. Return { favoriteKeys: string[], toggleFavorite(id), isFavorite(id) }. Store in 'keyrx_favorite_keys'. | Restrictions: ≤200 lines, memoize callbacks, handle errors | Success: Hook manages favorites, toggle works, localStorage persists, tests >80% | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 3. Extract usePaletteSearch hook
  - File: src/hooks/usePaletteSearch.ts
  - Fuzzy search logic with ranking
  - _Leverage: existing fuzzyMatch function_
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Hooks Developer with search expertise | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create usePaletteSearch hook with fuzzy search. Extract fuzzyMatch from KeyPalette.tsx lines 61-118. Accept keys array. Return { query, setQuery, results (sorted by score) }. Memoize search results. | Restrictions: ≤300 lines, all functions ≤50 lines, memoize expensive operations | Success: Hook provides fuzzy search, results ranked, memoization works, tests >80% | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4. Create PaletteSearch component
  - File: src/components/palette/PaletteSearch.tsx
  - Search input with results dropdown
  - _Leverage: existing search UI patterns_
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Component Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create PaletteSearch component. Props: { value, onChange, results, onSelect }. Render search input with dropdown showing fuzzy match results (highlighted matches). Extract from KeyPalette.tsx search section. | Restrictions: ≤300 lines, all functions ≤50 lines, accessible (ARIA) | Success: Search input works, results display, selection works, tests with user events | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 5. Create KeyCategorySection component
  - File: src/components/palette/KeyCategorySection.tsx
  - Reusable category renderer
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Component Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create KeyCategorySection component. Props: { title, keys, onKeySelect, favorites, onToggleFavorite, collapsible }. Render category title and key grid. Support favorite toggle button on each key. Extract category rendering from KeyPalette.tsx. | Restrictions: ≤300 lines, all functions ≤50 lines, keyboard navigation | Success: Category renders keys, selection works, favorite toggle works, tests | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6. Create PaletteViewModeTabs component
  - File: src/components/palette/PaletteViewModeTabs.tsx
  - View mode tabs (Basic/Recent/Favorites/All)
  - _Requirements: TR-1, TR-2_
  - _Prompt: Role: React Component Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Create PaletteViewModeTabs component. Props: { activeView, onChange }. Render tab buttons for Basic/Recent/Favorites/All views. Extract from KeyPalette.tsx tabs section. | Restrictions: ≤150 lines, accessible tabs (ARIA), visual active state | Success: Tabs render, onClick changes view, active state displays, tests | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 7. Write tests for all hooks
  - Files: useRecentKeys.test.ts, useFavoriteKeys.test.ts, usePaletteSearch.test.ts
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Write unit tests for all three hooks. Mock localStorage. Test FIFO, favorites toggle, fuzzy search ranking. Achieve >80% coverage. | Restrictions: Mock localStorage, test in isolation, cover edge cases | Success: All hooks tested, >80% coverage, tests pass | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 8. Write tests for all components
  - Files: PaletteSearch.test.tsx, KeyCategorySection.test.tsx, PaletteViewModeTabs.test.tsx
  - _Requirements: TR-2_
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Write unit tests for all three components. Test rendering, callbacks, user interactions, accessibility. Use userEvent. Achieve >80% coverage. | Restrictions: Test behavior, mock dependencies, verify callbacks | Success: All components tested, >80% coverage, tests pass | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 9. Refactor KeyPalette to use extracted code
  - File: src/components/KeyPalette.tsx
  - Orchestrate new components/hooks
  - _Requirements: TR-3_
  - _Prompt: Role: React Refactoring Specialist | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Refactor KeyPalette.tsx to use all hooks and components. Replace inline logic with hooks. Replace JSX sections with components. Remove replaced code. Target <500 lines, main function <50 lines. | Restrictions: No behavior changes, tests must pass, maintain UI/UX | Success: KeyPalette uses all new code, <500 lines, tests pass, ESLint passes | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 10. Final verification
  - Run tests, linting, verify metrics
  - _Requirements: All_
  - _Prompt: Role: Senior Developer | Task: Implement the task for spec refactor-key-palette. First run spec-workflow-guide, then implement: Verify KeyPalette refactoring complete. Test file size (≤500 lines), function sizes (≤50 lines), all tests pass, ESLint 0 errors, Prettier applied. Update docs. | Restrictions: Verify all metrics, fix any issues | Success: All criteria met, tests pass, ready for review | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_
