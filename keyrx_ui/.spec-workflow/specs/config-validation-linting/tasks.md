# Tasks Document

## Phase 1: Dependencies & TypeScript Foundation

- [x] 1. Add Monaco Editor dependencies to keyrx_ui/package.json
  - File: keyrx_ui/package.json
  - Add dependencies: @monaco-editor/react@^4.6.0, monaco-editor@^0.45.0, lodash.debounce@^4.0.8
  - Add devDependencies: @types/lodash.debounce@^4.0.9
  - Purpose: Install Monaco editor and debounce utility for validation
  - _Leverage: Existing keyrx_ui/package.json_
  - _Requirements: Dependency management_
  - _Prompt: Role: Frontend Build Engineer | Task: Add Monaco editor and utility dependencies to keyrx_ui/package.json with exact versions, ensuring compatibility with React 18+ and TypeScript 5+ | Restrictions: Use exact versions (not ^), verify no peer dependency conflicts with existing packages, add both runtime and type definitions | Success: ✅ npm install completes without warnings, ✅ Monaco editor types available in TypeScript, ✅ No version conflicts_

- [x] 2. Configure Vite for Monaco Editor assets in keyrx_ui/vite.config.ts
  - File: keyrx_ui/vite.config.ts
  - Configure Monaco editor worker integration
  - Add optimizeDeps for Monaco modules
  - Set up MonacoWebpackPlugin equivalent for Vite
  - Purpose: Ensure Monaco editor loads correctly in browser with Web Workers
  - _Leverage: Existing keyrx_ui/vite.config.ts plugins_
  - _Requirements: Build pipeline changes_
  - _Prompt: Role: Frontend Build Engineer with Vite expertise | Task: Configure Vite in keyrx_ui/vite.config.ts to support Monaco editor following Monaco documentation, adding vite-plugin-monaco-editor and configuring optimizeDeps.include for monaco-editor modules | Restrictions: Must not break existing React HMR, ensure Monaco workers load correctly, configure for both dev and production builds | Success: ✅ Monaco editor loads in browser, ✅ Syntax highlighting works, ✅ Workers load without errors, ✅ Production build optimized_

- [x] 3. Create validation types in keyrx_ui/src/types/validation.ts
  - File: keyrx_ui/src/types/validation.ts
  - Define ValidationResult, ValidationError, ValidationWarning, ValidationHint types
  - Define QuickFix, TextEdit, ConfigStats types
  - Export all types for use in validator and components
  - Purpose: Establish type-safe contracts for validation data structures
  - _Leverage: TypeScript 5.0+ strict mode_
  - _Requirements: Clear interfaces from design doc_
  - _Prompt: Role: TypeScript Types Architect | Task: Create comprehensive validation type definitions in keyrx_ui/src/types/validation.ts following design doc interfaces exactly, ensuring type safety for all validation scenarios | Restrictions: File ≤200 lines, TypeScript strict mode required, no `any` types, all fields must have clear JSDoc comments explaining purpose, use readonly where applicable | Success: ✅ All types defined matching design doc, ✅ JSDoc comments complete, ✅ TypeScript compiles with strict mode, ✅ Types exported correctly_

## Phase 2: Core Validation Logic

- [x] 4. Create ConfigValidator service in keyrx_ui/src/utils/validator.ts
  - File: keyrx_ui/src/utils/validator.ts
  - Implement ConfigValidator class with validate() method
  - Integrate with WasmCore.loadConfig() for syntax validation
  - Implement parseWasmError() to convert WASM errors to ValidationError format
  - Add runLintingRules() for optional code quality checks
  - Purpose: Core validation engine that wraps WASM module
  - _Leverage: WasmCore API from wasm-simulation-integration spec_
  - _Requirements: 1.1-1.5 (Real-time validation), 6.1-6.5 (Linting)_
  - _Prompt: Role: TypeScript Service Developer with WASM integration expertise | Task: Implement ConfigValidator class in keyrx_ui/src/utils/validator.ts following requirements 1.1-1.5 and 6.1-6.5, wrapping WasmCore.loadConfig() with error parsing and optional linting rules | **validate() method**: Call await WasmCore.loadConfig(rhaiSource), catch errors and parse with regex `/line (\d+), column (\d+): (.+)/`, return ValidationResult with errors array | **parseWasmError()**: Extract line/column from WASM error message, create ValidationError with code "WASM_ERROR", handle unparseable errors gracefully | **runLintingRules()**: Check for unused layers (defined but never activated), inconsistent naming (mix of camelCase and snake_case), large configs (>500 lines warning) | Restrictions: File ≤300 lines, TypeScript strict mode, no `any` types, all async functions must have explicit Promise<> return types, handle WASM crashes gracefully (try/catch with fallback error), export singleton instance `export const validator = new ConfigValidator()` | Success: ✅ validate() returns ValidationResult with parsed errors, ✅ Line numbers extracted correctly from WASM errors, ✅ Linting rules detect unused layers and naming issues, ✅ WASM crashes handled without UI crash, ✅ All methods have unit tests_

- [x] 5. Create useConfigValidator React hook in keyrx_ui/src/hooks/useConfigValidator.ts
  - File: keyrx_ui/src/hooks/useConfigValidator.ts
  - Implement useConfigValidator custom hook with debounced validation
  - Manage validation state (validationResult, isValidating)
  - Provide validate, clearValidation methods
  - Handle WASM initialization check
  - Purpose: React hook for editor components to trigger validation
  - _Leverage: React 18 hooks, lodash.debounce_
  - _Requirements: 1.1 (500ms debounce), 5.4 (non-blocking validation)_
  - _Prompt: Role: React Hooks Expert | Task: Create useConfigValidator custom hook in keyrx_ui/src/hooks/useConfigValidator.ts following requirements 1.1 and 5.4, implementing debounced validation with state management | **Hook interface**: Return `{ validationResult, isValidating, validate, clearValidation, wasmAvailable }` | **Debouncing**: Use useMemo + lodash.debounce with 500ms delay, cancel pending validation on unmount | **WASM check**: useEffect to check WasmCore.init() on mount, set wasmAvailable state, show fallback error if WASM unavailable | **State management**: useState for validationResult and isValidating, update after validation completes | Restrictions: File ≤150 lines, TypeScript strict mode, must cleanup debounce timer in useEffect return, memoize callbacks with useCallback, provide clear loading states | Success: ✅ Validation debounced by 500ms, ✅ isValidating flag updates correctly, ✅ WASM unavailable handled gracefully, ✅ Hook cleans up on unmount, ✅ Tests verify debouncing works_

## Phase 3: Monaco Editor Integration

- [x] 6. Configure Rhai language syntax for Monaco in keyrx_ui/src/utils/monacoConfig.ts
  - File: keyrx_ui/src/utils/monacoConfig.ts
  - Define Rhai language ID and file extensions
  - Create monarch syntax highlighting rules for Rhai
  - Configure language features (brackets, comments, indentation)
  - Register language with Monaco editor
  - Purpose: Enable Rhai syntax highlighting in Monaco editor
  - _Leverage: Monaco editor language API_
  - _Requirements: Editor integration_
  - _Prompt: Role: Monaco Editor Configuration Expert | Task: Configure Rhai language syntax for Monaco in keyrx_ui/src/utils/monacoConfig.ts, defining monarch tokenizer for Rhai keywords, operators, strings, comments | **Language registration**: `monaco.languages.register({ id: 'rhai', extensions: ['.rhai'] })` | **Monarch tokenizer**: Define keywords (layer, map, to, if, else, fn), operators (=, ==, !=, &&, ||), strings (double quotes), comments (// and /* */), numbers | **Language config**: Set brackets [['{', '}'], ['[', ']']], autoClosingPairs, comments ({ lineComment: '//', blockComment: ['/*', '*/'] }) | Restrictions: File ≤250 lines, must include all Rhai keywords from language spec, syntax highlighting must work in both light and dark themes | Success: ✅ Rhai files syntax-highlighted correctly, ✅ Keywords colored distinctly, ✅ Comments grayed out, ✅ Bracket matching works_

- [x] 7. Implement Monaco error markers in keyrx_ui/src/utils/monacoMarkers.ts
  - File: keyrx_ui/src/utils/monacoMarkers.ts
  - Create updateEditorMarkers() function to display errors/warnings
  - Convert ValidationResult to Monaco IMarkerData format
  - Set severity levels (Error, Warning, Info)
  - Clear markers when validation succeeds
  - Purpose: Display error squiggles and warnings in Monaco editor
  - _Leverage: Monaco editor markers API_
  - _Requirements: 1.2 (red squiggly lines), 2.1 (orange warning underlines)_
  - _Prompt: Role: Monaco Editor API Specialist | Task: Implement updateEditorMarkers() function in keyrx_ui/src/utils/monacoMarkers.ts to display validation errors/warnings as Monaco markers following requirements 1.2 and 2.1 | **Function signature**: `updateEditorMarkers(editor: monaco.editor.IStandaloneCodeEditor, validationResult: ValidationResult): void` | **Marker conversion**: Map ValidationError to { severity: monaco.MarkerSeverity.Error, startLineNumber: err.line, startColumn: err.column, endLineNumber: err.endLine || err.line, endColumn: err.endColumn || err.column + 10, message: err.message, code: err.code } | **Warning markers**: Map ValidationWarning to MarkerSeverity.Warning | **Clear markers**: Call `monaco.editor.setModelMarkers(model, 'config-validator', markers)` to replace all markers | Restrictions: File ≤200 lines, TypeScript strict mode, handle null model gracefully, endColumn defaults to column + 10 if not provided, markers must clear when validationResult.errors is empty | Success: ✅ Error markers appear as red squiggles, ✅ Warning markers appear as orange squiggles, ✅ Markers clear when errors fixed, ✅ Hover shows error message_

- [x] 8. Implement Quick Fix code actions in keyrx_ui/src/utils/monacoQuickFix.ts
  - File: keyrx_ui/src/utils/monacoQuickFix.ts
  - Register Monaco code action provider for "Quick Fix"
  - Convert QuickFix to Monaco ICodeAction format
  - Apply TextEdit changes to editor model
  - Purpose: Enable "Quick Fix" button in error tooltips
  - _Leverage: Monaco code actions API_
  - _Requirements: 3.4-3.5 (Quick Fix suggestions)_
  - _Prompt: Role: Monaco Code Actions Expert | Task: Implement Quick Fix code actions in keyrx_ui/src/utils/monacoQuickFix.ts following requirements 3.4-3.5, converting QuickFix suggestions to Monaco code actions | **Code action provider**: `monaco.languages.registerCodeActionProvider('rhai', { provideCodeActions: (model, range, context) => ... })` | **Context matching**: Check if marker at cursor has quickFix suggestion, return { title: quickFix.title, kind: 'quickfix', edit: { edits: [...] } } | **Edit application**: Convert TextEdit to Monaco edit format with range and newText | Restrictions: File ≤200 lines, TypeScript strict mode, only provide Quick Fix for markers with suggestion field, edits must be atomic (all or nothing) | Success: ✅ Quick Fix appears in lightbulb menu, ✅ Clicking Quick Fix applies edit, ✅ Editor content updated correctly, ✅ Works with multiple errors_

## Phase 4: React Components

- [x] 9. Create ConfigEditor component with Monaco in keyrx_ui/src/components/ConfigEditor.tsx
  - File: keyrx_ui/src/components/ConfigEditor.tsx
  - Integrate @monaco-editor/react component
  - Connect to useConfigValidator hook
  - Display validation status in status bar
  - Handle save with validation check
  - **UI Layout**: (See design.md for full layout)
  - Purpose: Main configuration editor with real-time validation
  - _Leverage: @monaco-editor/react, useConfigValidator hook_
  - _Requirements: 7.1-7.5 (Editor integration)_
  - _Prompt: Role: React Developer with Monaco editor integration expertise | Task: Create ConfigEditor component in keyrx_ui/src/components/ConfigEditor.tsx following requirements 7.1-7.5, integrating Monaco editor with useConfigValidator hook for real-time validation | **Component props**: `{ initialValue?: string, onSave: (content: string) => Promise<void>, onValidationChange?: (result: ValidationResult) => void }` | **State management**: useState for content, useConfigValidator for validation, useRef for Monaco editor instance | **onChange handler**: `handleEditorChange(value) => { setContent(value); validate(value); }` - debounced validation via hook | **Save handler**: `handleSave() => { if (validationResult.errors.length > 0) { alert('Fix errors before saving'); return; } await onSave(content); }` | **Monaco integration**: `<Editor height="600px" language="rhai" value={content} onChange={handleEditorChange} onMount={handleEditorMount} options={{ minimap: { enabled: false }, fontSize: 14, lineNumbers: 'on' }} />` | **Keyboard shortcuts**: F8 to jump to next error (use Monaco command API) | Restrictions: File ≤400 lines, TypeScript strict mode, must handle null editor ref, save button disabled if errors > 0, display "Validating..." indicator when isValidating true, follow existing component styling patterns | Success: ✅ Editor renders with Rhai syntax highlighting, ✅ Validation triggers after typing stops (500ms), ✅ Error squiggles appear, ✅ Save blocked if errors exist, ✅ F8 jumps to errors_

- [x] 10. Create ValidationStatusPanel component in keyrx_ui/src/components/ValidationStatusPanel.tsx
  - File: keyrx_ui/src/components/ValidationStatusPanel.tsx
  - Display error/warning/hint counts with badges
  - List all errors with "Jump" buttons
  - Show success indicator (green checkmark) if 0 errors
  - Collapsible panel for space efficiency
  - **UI Layout**: (See design.md for full layout)
  - Purpose: Summary panel showing all validation issues
  - _Leverage: ValidationResult type_
  - _Requirements: 4.1-4.5 (Validation status summary)_
  - _Prompt: Role: React UI Developer | Task: Create ValidationStatusPanel component in keyrx_ui/src/components/ValidationStatusPanel.tsx following requirements 4.1-4.5, displaying validation status with error/warning counts and jump-to-error functionality | **Component props**: `{ validationResult: ValidationResult | null, isValidating: boolean, onErrorClick: (error: ValidationError) => void, onWarningClick: (warning: ValidationWarning) => void }` | **Badge display**: Show "❌ {count} Errors" (red), "⚠️ {count} Warnings" (orange), "💡 {count} Hints" (blue) | **Error list**: Map validationResult.errors to `<li key={index}>Line {err.line}: {err.message} <button onClick={() => onErrorClick(err)}>Jump</button></li>` | **Success state**: If validationResult?.errors.length === 0 && warnings.length === 0, show green banner "✓ Configuration valid" | **Collapsible**: useState for isExpanded, toggle on header click | Restrictions: File ≤200 lines, TypeScript strict mode, show "Validating..." spinner when isValidating true, limit error list to 10 items (show "...and N more" if > 10), accessible (ARIA labels, keyboard navigation) | Success: ✅ Counts update in real-time, ✅ Jump buttons scroll to error line, ✅ Success indicator shows when valid, ✅ Panel collapses/expands, ✅ 0 axe violations_

## Phase 5: Linting Rules

- [x] 11. Implement unused layer detection in keyrx_ui/src/utils/lintingRules.ts
  - File: keyrx_ui/src/utils/lintingRules.ts
  - Create lintUnusedLayers() function
  - Parse config AST to find defined vs activated layers
  - Return ValidationWarning for unused layers
  - Purpose: Detect layers that are defined but never activated
  - _Leverage: WASM config AST (if exposed)_
  - _Requirements: 6.1 (unused layer warning)_
  - _Prompt: Role: Code Quality Engineer | Task: Implement lintUnusedLayers() function in keyrx_ui/src/utils/lintingRules.ts following requirement 6.1, detecting layers defined but never activated | **Function signature**: `lintUnusedLayers(configSource: string): ValidationWarning[]` | **Implementation**: Regex parse `layer "(\w+)"` to find definitions, regex parse layer activation keywords (e.g., `activate_layer "(\w+)"`), compare defined vs activated sets, return warnings for unused | **Warning format**: `{ line: layerDefLine, column: 1, message: "Layer '{name}' is defined but never activated", code: "UNUSED_LAYER" }` | Restrictions: File ≤300 lines (shared with other linting rules), TypeScript strict mode, handle regex errors gracefully, only detect layers (not modifiers/locks) | Success: ✅ Detects unused layers correctly, ✅ Warning includes layer name, ✅ No false positives for used layers_

- [x] 12. Implement naming convention linting in keyrx_ui/src/utils/lintingRules.ts (continue)
  - File: keyrx_ui/src/utils/lintingRules.ts (continue from task 11)
  - Create lintNamingConsistency() function
  - Detect mix of camelCase and snake_case in layer/modifier names
  - Return ValidationHint suggesting consistent style
  - Purpose: Encourage consistent naming conventions
  - _Requirements: 6.2 (naming consistency hint)_
  - _Prompt: Role: Code Quality Engineer | Task: Implement lintNamingConsistency() function in keyrx_ui/src/utils/lintingRules.ts following requirement 6.2, detecting inconsistent naming styles | **Function signature**: `lintNamingConsistency(configSource: string): ValidationHint[]` | **Implementation**: Extract all layer/modifier names via regex, classify as camelCase (has uppercase in middle) or snake_case (has underscores), if both styles present return hint | **Hint format**: `{ line: 1, column: 1, message: "Consider using consistent naming (e.g., all snake_case). Found: camelCase (3) and snake_case (5)", code: "NAMING_INCONSISTENCY" }` | Restrictions: Part of lintingRules.ts (≤300 lines total), TypeScript strict mode, only hint (not error/warning), provide counts for each style | Success: ✅ Detects mixed naming styles, ✅ Hint message includes counts, ✅ No hint if all names consistent_

## Phase 6: Integration & Testing

- [x] 13. Integrate validation into existing config editor (if exists)
  - File: keyrx_ui/src/pages/ConfigurationPage.tsx (or wherever config editing happens)
  - Replace textarea/simple editor with ConfigEditor component
  - Pass onSave handler to ConfigEditor
  - Connect "Test Configuration" button to simulator
  - Purpose: Replace existing editor with validation-enabled Monaco editor
  - _Leverage: ConfigEditor component, existing config loading logic_
  - _Requirements: 7.1-7.5 (Editor integration)_
  - _Prompt: Role: React Integration Developer | Task: Integrate ConfigEditor component into existing configuration editing page following requirements 7.1-7.5, replacing simple textarea with Monaco editor | **Component replacement**: Remove old textarea, add `<ConfigEditor initialValue={currentConfig} onSave={handleSaveConfig} onValidationChange={setValidationResult} />` | **Save handler**: `handleSaveConfig = async (content: string) => { await apiClient.post('/api/config', { content }); showNotification('Configuration saved'); }` | **Test button**: Pass validationResult to simulator (no re-validation needed) | Restrictions: File ≤300 lines (modify existing page), preserve existing save functionality, maintain URL routing, handle loading states | Success: ✅ Monaco editor renders in config page, ✅ Save works correctly, ✅ Validation status visible, ✅ Test button uses validated config_

- [x] 14. Write unit tests for validator service in keyrx_ui/src/utils/validator.test.ts
  - File: keyrx_ui/src/utils/validator.test.ts
  - Test validate() with valid Rhai configs (0 errors)
  - Test validate() with invalid syntax (parse errors with line numbers)
  - Test parseWasmError() extracts line/column correctly
  - Test linting rules (unused layers, naming)
  - Purpose: Ensure validator parses WASM errors correctly
  - _Leverage: Vitest, mocked WasmCore_
  - _Requirements: Testing strategy from design doc_
  - _Prompt: Role: TypeScript Test Engineer | Task: Write comprehensive unit tests for ConfigValidator in keyrx_ui/src/utils/validator.test.ts, mocking WasmCore to test error parsing and linting | **Test valid config**: Mock WasmCore.loadConfig to return ConfigHandle, verify validate() returns { errors: [], warnings: [], hints: [] } | **Test syntax error**: Mock WasmCore.loadConfig to throw error with message "Parse error at line 4, column 9: Missing semicolon", verify validate() returns error with line: 4, column: 9 | **Test linting**: Call validate() with config having unused layer, verify warning returned | **Test WASM crash**: Mock WasmCore to throw unexpected error, verify graceful fallback error | Restrictions: File ≤400 lines, use Vitest, mock WasmCore completely (import { vi } from 'vitest'), test all error paths, coverage ≥90% | Success: ✅ All tests pass, ✅ Line/column extraction verified, ✅ Linting rules tested, ✅ WASM crashes handled_

- [x] 15. Write unit tests for useConfigValidator hook in keyrx_ui/src/hooks/useConfigValidator.test.ts
  - File: keyrx_ui/src/hooks/useConfigValidator.test.ts
  - Test debouncing (validate only after 500ms idle)
  - Test validation state updates (isValidating flag)
  - Test WASM unavailable fallback
  - Test cleanup on unmount
  - Purpose: Ensure hook debounces correctly and manages state
  - _Leverage: @testing-library/react-hooks, vi.useFakeTimers_
  - _Requirements: Testing strategy_
  - _Prompt: Role: React Hooks Test Engineer | Task: Write unit tests for useConfigValidator hook in keyrx_ui/src/hooks/useConfigValidator.test.ts, testing debouncing, state management, and cleanup | **Test debouncing**: Use vi.useFakeTimers(), call validate() multiple times within 500ms, advance timers by 500ms, verify only one validator.validate() call made | **Test isValidating**: Call validate(), verify isValidating becomes true, await completion, verify becomes false | **Test WASM unavailable**: Mock WasmCore.init() to reject, verify wasmAvailable false, validate() returns fallback error | **Test cleanup**: Render hook, call validate(), unmount before completion, verify debounce timer cancelled | Restrictions: File ≤300 lines, use @testing-library/react, vi.useFakeTimers for debounce testing, coverage ≥85% | Success: ✅ Debouncing verified (only 1 call per 500ms), ✅ State updates correct, ✅ Cleanup prevents memory leaks_

- [x] 16. Write React component tests for ConfigEditor in keyrx_ui/src/components/ConfigEditor.test.tsx
  - File: keyrx_ui/src/components/ConfigEditor.test.tsx
  - Test Monaco editor rendering
  - Test validation triggers on typing (debounced)
  - Test save button disabled when errors exist
  - Test keyboard shortcuts (F8 jump to error)
  - Purpose: Ensure editor component integrates validation correctly
  - _Leverage: @testing-library/react, mocked useConfigValidator_
  - _Requirements: Testing strategy_
  - _Prompt: Role: React Component Test Engineer | Task: Write comprehensive component tests for ConfigEditor in keyrx_ui/src/components/ConfigEditor.test.tsx, testing rendering, validation integration, and user interactions | **Test rendering**: Render ConfigEditor, verify Monaco editor appears (check for `.monaco-editor` class) | **Test validation**: Type invalid config, wait 500ms (vi.advanceTimersByTime), verify error squiggles appear (mock Monaco markers API) | **Test save disabled**: Set validationResult with errors via mocked hook, verify save button has disabled attribute | **Test F8 shortcut**: Simulate F8 keypress, verify editor cursor moves to first error line | Restrictions: File ≤400 lines, use @testing-library/react, mock useConfigValidator hook, mock Monaco editor API (updateEditorMarkers, etc.), coverage ≥80% | Success: ✅ All rendering tests pass, ✅ Validation integration works, ✅ Save disabled correctly, ✅ Keyboard shortcuts functional_

- [x] 17. Write accessibility tests for ValidationStatusPanel in keyrx_ui/src/components/ValidationStatusPanel.test.tsx
  - File: keyrx_ui/src/components/ValidationStatusPanel.test.tsx
  - Test ARIA labels and roles
  - Test keyboard navigation (Tab, Enter)
  - Test screen reader announcements (aria-live)
  - Test color contrast for error badges
  - Purpose: Ensure validation panel is WCAG 2.1 AA compliant
  - _Leverage: @axe-core/react, @testing-library/react_
  - _Requirements: Accessibility requirements, WCAG 3.3 Input Assistance_
  - _Prompt: Role: Accessibility QA Engineer | Task: Write comprehensive accessibility tests for ValidationStatusPanel in keyrx_ui/src/components/ValidationStatusPanel.test.tsx following WCAG 2.1 AA standards | **Axe audit**: Render component, run `const results = await axe(container); expect(results).toHaveNoViolations();` | **ARIA labels**: Verify error list has aria-label="Validation Errors", buttons have aria-label="Jump to error" | **Keyboard navigation**: Simulate Tab key, verify focus moves through error list, simulate Enter on "Jump" button, verify onErrorClick called | **Screen reader**: Verify error count has aria-live="polite" for real-time updates | **Color contrast**: Verify error badge (red) and warning badge (orange) meet 3:1 contrast ratio (use axe color-contrast rule) | Restrictions: File ≤300 lines, use jest-axe (import { axe, toHaveNoViolations } from 'jest-axe'), ZERO axe violations required, all interactive elements must be keyboard-accessible | Success: ✅ 0 axe violations, ✅ All ARIA attributes correct, ✅ Keyboard navigation works, ✅ Screen reader announces updates_

- [x] 18. Write E2E test for full validation workflow in keyrx_ui/tests/e2e/config-validation.spec.ts
  - File: keyrx_ui/tests/e2e/config-validation.spec.ts
  - Test user journey: open editor → type invalid config → see error → fix error → save
  - Test Quick Fix action
  - Test "Apply Configuration" button disabled when errors exist
  - Purpose: Validate complete validation feature works end-to-end
  - _Leverage: Playwright or Cypress_
  - _Requirements: All requirements (end-to-end scenario)_
  - _Prompt: Role: QA Automation Engineer with E2E testing expertise | Task: Write end-to-end test for full validation workflow in keyrx_ui/tests/e2e/config-validation.spec.ts, testing complete user journey from typing to saving | **Test invalid config**: Navigate to /config-editor, type `layer "test" { map KEY_INVALID to KEY_A }`, wait 500ms, verify red squiggle appears on "KEY_INVALID" | **Test hover**: Hover over error, verify tooltip shows "Invalid key code 'KEY_INVALID'" | **Test Quick Fix**: Click Quick Fix button in tooltip, verify "KEY_INVALID" replaced with suggestion | **Test save disabled**: Verify "Apply Configuration" button disabled (check `disabled` attribute), fix error, verify button enabled | **Test save success**: Click save, verify API call made (intercept POST /api/config), verify success notification | Restrictions: File ≤400 lines, use Playwright, test must run in headless mode, verify WASM loads correctly, test runs reliably in CI | Success: ✅ Full user journey works, ✅ Validation triggers correctly, ✅ Quick Fix applies edits, ✅ Save blocked on errors, ✅ Test passes in CI_

## Phase 7: Documentation & Finalization

- [x] 19. Create validation documentation in docs/config-validation.md
  - File: docs/config-validation.md (new file)
  - Document how to use real-time validation in editor
  - Explain error messages and Quick Fix actions
  - Provide examples of common errors
  - Document linting rules and how to toggle them
  - Purpose: Help users understand and use validation features
  - _Leverage: Requirements and design docs_
  - _Requirements: User-facing documentation_
  - _Prompt: Role: Technical Writer | Task: Create comprehensive documentation for config validation feature in docs/config-validation.md, explaining usage, error messages, and troubleshooting | **Introduction**: Explain real-time validation benefits (catch errors early, IDE-like experience) | **Usage guide**: Step-by-step: open editor → type config → see errors → hover for details → click Quick Fix | **Error reference**: Table of common errors (syntax errors, undefined layers, invalid key codes) with explanations and solutions | **Linting rules**: Explain each rule (unused layers, naming consistency, large configs), show how to toggle in settings | **Troubleshooting**: Cover "Validation unavailable" error (WASM failed to load), performance issues (large configs), browser compatibility | Restrictions: File ≤1000 lines (markdown), include screenshots or ASCII diagrams, provide copy-paste examples, maintain consistent formatting with existing docs | Success: ✅ Documentation clear and comprehensive, ✅ Examples provided for common scenarios, ✅ Troubleshooting section addresses common issues_

- [x] 20. Log implementation artifacts
  - Use: mcp spec-workflow log-implementation tool
  - Purpose: Document all implementation artifacts for future AI agents
  - _Leverage: Completed implementation from all previous tasks_
  - _Requirements: All requirements (create searchable knowledge base)_
  - _Prompt: Role: Documentation Engineer | Task: Create comprehensive implementation log documenting all config validation artifacts using log-implementation tool following spec workflow | **artifacts.components**: Document React components (ConfigEditor, ValidationStatusPanel) with name, type, purpose, location, props, exports | **artifacts.functions**: Document utility functions (validate, parseWasmError, runLintingRules, updateEditorMarkers, registerQuickFixProvider) with name, purpose, location (file:line), signature, isExported | **artifacts.classes**: Document ConfigValidator class with name, purpose, location, methods (validate, parseWasmError, runLintingRules), isExported | **artifacts.integrations**: Document editor-validator data flow (User types → debounce → useConfigValidator → validator.validate() → WasmCore.loadConfig() → errors parsed → Monaco markers updated) | Include filesModified, filesCreated, statistics (linesAdded, linesRemoved) | Restrictions: Must document ALL artifacts comprehensively, include exact file paths with line numbers, provide clear purpose statements, record accurate statistics | Success: ✅ Implementation log complete, ✅ All components/functions documented, ✅ Data flows explained clearly, ✅ Statistics accurate_
