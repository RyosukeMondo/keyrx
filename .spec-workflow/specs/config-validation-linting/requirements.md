# Requirements Document

## Introduction

The Configuration Validation & Linting feature provides real-time feedback on Rhai configuration correctness directly in the web UI editor. Currently, users must save a configuration, compile it to .krx format, and reload the daemon before discovering syntax errors, semantic issues, or invalid key codes. This creates a frustrating trial-and-error workflow that slows down configuration development.

By leveraging the WASM simulation module (from wasm-simulation-integration spec), we can validate configurations instantly as users type, highlighting errors inline with precise line numbers and actionable fix suggestions. This transforms the configuration editing experience from batch-and-pray to continuous feedback, catching errors at the earliest possible moment.

This is critical for:
- **Developer Velocity**: Catch syntax errors immediately without leaving the editor
- **Error Prevention**: Identify semantic issues (undefined layers, invalid key codes) before apply
- **Learning**: Users see errors explained in real-time, building mental models faster
- **Confidence**: Green checkmarks and "0 errors" indicators provide psychological safety before applying configs

## Alignment with Product Vision

This feature aligns with the KeyRx vision of providing a user-friendly keyboard remapping solution by:

- **Immediate Feedback**: Shift-left on error detection from daemon reload to editor typing
- **Best-in-Class UX**: Match modern IDE-like validation (VSCode, Monaco editor patterns)
- **Code Reuse**: 100% reuse of WASM module validation logic ensures consistency with daemon
- **Accessibility**: WCAG 2.1 AA compliance for error messages and visual indicators
- **Progressive Disclosure**: Show errors inline without overwhelming users with all issues at once

## Requirements

### Requirement 1: Real-Time Syntax Validation

**User Story:** As a user editing a Rhai configuration in the web UI, I want syntax errors highlighted immediately after I stop typing, so that I can fix typos and missing brackets before saving.

#### Acceptance Criteria

1. WHEN a user types in the configuration editor THEN syntax validation SHALL run after 500ms of inactivity (debounced)
2. WHEN syntax errors exist THEN the editor SHALL underline invalid text with red squiggly lines
3. WHEN a user hovers over an underlined error THEN a tooltip SHALL display the error message with line/column numbers
4. WHEN syntax validation succeeds THEN all red underlines SHALL be removed
5. WHEN validation is running THEN a subtle "Validating..." indicator SHALL appear in the editor status bar

### Requirement 2: Semantic Validation (Logical Errors)

**User Story:** As a user defining layer-specific mappings, I want to be warned if I reference undefined layers or modifiers, so that I don't apply a configuration that will silently fail.

#### Acceptance Criteria

1. WHEN a user references a layer name that doesn't exist THEN the editor SHALL highlight it with an orange warning underline
2. WHEN a user uses an invalid key code (e.g., "KEY_INVALID") THEN the editor SHALL flag it as an error
3. WHEN a user defines circular dependencies (layer A triggers layer B triggers layer A) THEN a warning SHALL be displayed
4. WHEN semantic validation detects >10 issues THEN only the first 10 SHALL be shown (with "...and 5 more" indicator)
5. WHEN a user clicks on a warning THEN the editor SHALL jump to the relevant line and highlight the issue

### Requirement 3: Actionable Error Messages

**User Story:** As a beginner user encountering an error, I want clear explanations and suggested fixes, so that I can resolve issues without reading documentation.

#### Acceptance Criteria

1. WHEN an error message is displayed THEN it SHALL include:
   - Clear description of the problem
   - Line and column number
   - Suggested fix (if applicable)
   - Link to relevant documentation (optional)
2. WHEN a syntax error is "missing semicolon" THEN the message SHALL say: "Missing semicolon at end of statement. Add ';' after line 42."
3. WHEN an undefined layer is referenced THEN the message SHALL list available layers: "Layer 'gamming' not found. Did you mean 'gaming'? Available: [base, gaming, coding]"
4. WHEN a fix suggestion is available THEN a "Quick Fix" button SHALL appear in the tooltip
5. WHEN a user clicks "Quick Fix" THEN the editor SHALL apply the suggested change automatically

### Requirement 4: Validation Status Summary

**User Story:** As a user completing a configuration edit, I want a clear summary of validation status (errors, warnings, success), so that I know if it's safe to apply changes.

#### Acceptance Criteria

1. WHEN the editor is open THEN a validation status panel SHALL appear at the bottom showing:
   - Error count (red badge)
   - Warning count (orange badge)
   - Success indicator (green checkmark if 0 errors)
2. WHEN there are errors THEN the "Apply Configuration" button SHALL be disabled with tooltip: "Fix 3 errors before applying"
3. WHEN there are only warnings (no errors) THEN the "Apply Configuration" button SHALL be enabled with tooltip: "3 warnings (click to review)"
4. WHEN validation succeeds with 0 errors/warnings THEN a green banner SHALL appear: "✓ Configuration valid (42 lines, 12 mappings)"
5. WHEN a user clicks on the error/warning count THEN the editor SHALL jump to the first issue

### Requirement 5: Performance Optimization

**User Story:** As a user editing a large configuration (1000+ lines), I want validation to remain fast and non-blocking, so that typing feels responsive.

#### Acceptance Criteria

1. WHEN a user types in a <500 line config THEN validation SHALL complete in <100ms
2. WHEN a user types in a 1000+ line config THEN validation SHALL complete in <300ms
3. WHEN validation exceeds 300ms THEN it SHALL be cancelled and re-queued after next idle period
4. WHEN a user is actively typing THEN validation SHALL NOT block the UI (runs in background)
5. WHEN WASM validation fails or crashes THEN a fallback error SHALL be shown: "Validation unavailable. Please try reloading."

### Requirement 6: Linting Rules (Code Quality)

**User Story:** As a power user, I want optional linting rules that suggest best practices (e.g., unused layers, inconsistent naming), so that I can write cleaner configurations.

#### Acceptance Criteria

1. WHEN a layer is defined but never used THEN a warning SHALL appear: "Layer 'debug' is defined but never activated"
2. WHEN key mappings use inconsistent naming (camelCase vs snake_case) THEN a hint SHALL suggest: "Consider using consistent naming (e.g., all snake_case)"
3. WHEN a configuration exceeds 500 lines THEN a hint SHALL suggest: "Consider splitting into multiple files for maintainability"
4. WHEN linting rules are enabled THEN users SHALL be able to toggle them on/off in settings
5. WHEN a user hovers over a linting hint THEN the tooltip SHALL explain the rule and why it matters

### Requirement 7: Integration with Configuration Editor

**User Story:** As a user editing my configuration in the web UI, I want validation integrated seamlessly into the editor, so that it feels like a native IDE experience.

#### Acceptance Criteria

1. WHEN the configuration editor loads THEN validation SHALL initialize automatically
2. WHEN a user opens an existing configuration THEN it SHALL be validated immediately
3. WHEN a user saves a configuration THEN validation SHALL run one final time before save
4. WHEN a user switches between configurations THEN validation state SHALL reset for the new config
5. WHEN the "Test Configuration" button is clicked THEN the simulator SHALL use the validated config (no re-validation needed)

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**:
  - Validation logic isolated in keyrx_ui/src/utils/validator.ts
  - Editor integration isolated in keyrx_ui/src/hooks/useConfigValidator.ts
  - UI components isolated in keyrx_ui/src/components/ValidationPanel.tsx
- **Modular Design**:
  - Validator uses WASM module via WasmCore API (no direct WASM calls)
  - Validation rules extensible (add new rules without modifying core validator)
  - Editor agnostic (works with any Monaco-like editor)
- **Dependency Management**:
  - Reuse WasmCore from wasm-simulation-integration spec
  - Monaco editor for syntax highlighting and error squiggles
  - Debounce validation with lodash.debounce or custom hook
- **Clear Interfaces**:
  - ValidationResult type: `{ errors: Error[], warnings: Warning[], hints: Hint[] }`
  - Error type: `{ line: number, column: number, message: string, suggestion?: string }`
  - Validator API: `validate(rhaiSource: string): Promise<ValidationResult>`

### Performance

- **Validation Latency**: <100ms for <500 line configs, <300ms for 1000+ line configs
- **Debounce Delay**: 500ms idle time before triggering validation
- **UI Responsiveness**: Validation runs in background (Web Worker if needed), never blocks typing
- **Memory Usage**: Validation state <10MB for typical configs

### Security

- **Sandboxing**: Validation runs in WASM sandbox (no arbitrary code execution)
- **Input Validation**: Large configs (>10MB) rejected before validation
- **Error Sanitization**: Error messages do not expose internal WASM memory addresses

### Reliability

- **Error Handling**: WASM crashes caught and reported gracefully
- **Graceful Degradation**: If WASM fails to load, validation is disabled with clear user message
- **Deterministic**: Same config + same validator version = identical validation results

### Usability

- **Discovery**: Validation status panel visible by default in config editor
- **Feedback**: Error underlines appear within 500ms of stopping typing
- **Error Messages**: Follow WCAG 3.3 Input Assistance (identify errors, describe them, suggest solutions)
- **Keyboard Shortcuts**: F8 to jump to next error, Shift+F8 to jump to previous error
- **Accessibility**: Error messages announced to screen readers, color-blind safe indicators

## Dependencies

### Existing Infrastructure

This feature builds on existing KeyRx components:

1. **WASM Simulation Module** (wasm-simulation-integration spec):
   - load_config function already validates Rhai syntax and semantics
   - Reuse ConfigHandle validation without re-implementing parser
   - Errors already include line numbers from parser

2. **keyrx_ui Configuration Editor**:
   - Existing React editor component (if implemented)
   - Need to integrate Monaco editor or similar for syntax highlighting

3. **WasmCore API** (keyrx_ui/src/wasm/core.ts):
   - Already wraps WASM functions with TypeScript Promise APIs
   - Extend with validation-specific methods

### New Dependencies

- **Monaco Editor** (or CodeMirror 6): Syntax highlighting, error squiggles, tooltips
- **@monaco-editor/react**: React wrapper for Monaco editor
- **lodash.debounce** (or custom hook): Debounced validation triggering
- **react-error-boundary**: Graceful error handling for WASM crashes

### Build Pipeline Changes

- No build changes required (WASM module already built by wasm-simulation-integration spec)
- Add Monaco editor assets to Vite bundler config
- Configure Monaco worker for syntax highlighting in Web Worker

## Sources

This specification incorporates world-class UI/UX patterns from:
- [Building UX for Error Validation Strategy](https://medium.com/@olamishina/building-ux-for-error-validation-strategy-36142991017a)
- [A Complete Guide To Live Validation UX — Smashing Magazine](https://www.smashingmagazine.com/2022/09/inline-validation-web-forms-ux/)
- [Inline Validation UX — Smart Interface Design Patterns](https://smart-interface-design-patterns.com/articles/inline-validation-ux/)
- [Error handling - UX design patterns](https://medium.com/design-bootcamp/error-handling-ux-design-patterns-c2a5bbae5f8d)
