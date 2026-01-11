# Tasks Document

## Phase 0: Type Safety Infrastructure

- [x] 1. Add typeshare to Rust project
  - File: keyrx_daemon/Cargo.toml
  - Add typeshare dependency and build configuration
  - Annotate existing API structs with #[typeshare] attribute
  - Generate initial TypeScript types to keyrx_ui/src/types/generated.ts
  - Purpose: Establish automatic type generation from Rust to TypeScript
  - _Leverage: None (new infrastructure)_
  - _Requirements: 9.1, 9.4, 9.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer specializing in type systems and build tooling | Task: Add typeshare crate to keyrx_daemon following requirements 9.1, 9.4, 9.7. Add typeshare = "1.0" to Cargo.toml dependencies. Configure output directory in Cargo.toml metadata section. Annotate all public API structs and enums with #[typeshare] attribute (DeviceEntry, ProfileMetadata, LayoutPreset, KeyMapping, LatencyStats, RpcRequest, RpcResponse). Run cargo typeshare to generate keyrx_ui/src/types/generated.ts. Verify generated TypeScript compiles with npm run type-check. Document generation command in keyrx_daemon/README.md | Restrictions: Do not modify existing struct definitions beyond adding #[typeshare] attribute, must generate valid TypeScript that compiles without errors, must not break existing functionality | Success: typeshare dependency added to Cargo.toml, all API structs annotated with #[typeshare], generated.ts file created with valid TypeScript types, types compile successfully, generation command documented in README. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (structs annotated, generated types file, build configuration), then mark task complete [x] in tasks.md_

- [x] 2. Add Zod validation to frontend
  - File: keyrx_ui/package.json, keyrx_ui/src/api/schemas.ts
  - Install zod dependency
  - Create Zod schemas matching all API response types
  - Implement validateApiResponse helper function with error logging
  - Purpose: Enable runtime validation of API responses
  - _Leverage: keyrx_ui/src/types/generated.ts_
  - _Requirements: 9.2, 9.5, 9.6_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend TypeScript Developer specializing in runtime validation and type safety | Task: Add Zod runtime validation following requirements 9.2, 9.5, 9.6. Install zod package with npm install zod. Create keyrx_ui/src/api/schemas.ts file. Define Zod schemas for all API response types: DeviceEntry, DeviceListResponse, ProfileMetadata, ProfileListResponse, ProfileConfigResponse, KeyMapping, LatencyStats, RpcRequest, RpcResponse. Implement validateApiResponse<T>(schema: ZodSchema<T>, data: unknown, endpoint: string): T helper that validates data against schema, throws descriptive error on validation failure, logs validation errors with endpoint context and error details to console.error. Log unexpected fields as warnings (not errors) | Restrictions: Schemas must match generated TypeScript types exactly, validation errors must include endpoint name and detailed message, must handle nullable fields correctly, must not throw on unexpected fields (log as warning) | Success: zod installed in package.json, schemas.ts created with schemas for all API types, validateApiResponse helper implemented, validation errors include endpoint context, unexpected fields logged as warnings, all schemas match generated types. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (schemas created, validation helper, API types covered), then mark task complete [x] in tasks.md_

- [ ] 3. Integrate validation into API hooks
  - File: keyrx_ui/src/hooks/useDevices.ts, keyrx_ui/src/hooks/useProfiles.ts, keyrx_ui/src/hooks/useProfileConfig.ts, keyrx_ui/src/hooks/useUnifiedApi.ts
  - Add validateApiResponse calls to all fetch functions
  - Add validation to WebSocket message handlers
  - Log validation errors for debugging
  - Purpose: Catch API contract violations at runtime
  - _Leverage: keyrx_ui/src/api/schemas.ts, keyrx_ui/src/types/generated.ts_
  - _Requirements: 9.2, 9.5, 9.6_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in data fetching and error handling | Task: Integrate runtime validation into all API hooks following requirements 9.2, 9.5, 9.6. Modify useDevices hook to validate GET /api/devices and PATCH /api/devices/:id responses with DeviceListResponseSchema and DeviceEntrySchema. Modify useProfiles to validate all profile endpoints (GET, POST, DELETE, PATCH). Modify useProfileConfig to validate config GET/PUT responses. Modify useUnifiedApi to validate all WebSocket RPC messages with RpcRequestSchema and RpcResponseSchema. Wrap all fetch response parsing and WebSocket data parsing with validateApiResponse. Log validation failures with console.error including endpoint and error details. Include unexpected fields in error logs | Restrictions: Must not break existing functionality, validation failures should throw errors (fail fast), must preserve existing error handling patterns, must log structured error details | Success: All API hooks validate responses using validateApiResponse, WebSocket RPC messages validated, validation errors thrown with endpoint context, unexpected fields logged, existing error handling preserved, no regressions in functionality. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (hooks modified, validation points added, error handling), then mark task complete [x] in tasks.md_

- [ ] 4. Add validator crate to Rust backend
  - File: keyrx_daemon/Cargo.toml, keyrx_daemon/src/web/handlers/devices.rs, keyrx_daemon/src/web/handlers/profile.rs
  - Add validator dependency to Cargo.toml
  - Add validation attributes to request structs
  - Call .validate() in handler functions
  - Purpose: Validate incoming API requests in Rust
  - _Leverage: keyrx_daemon/src/web/handlers/mod.rs_
  - _Requirements: 9.3_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer specializing in API validation and error handling | Task: Add validator crate for request validation following requirement 9.3. Add validator = { version = "0.16", features = ["derive"] } to keyrx_daemon/Cargo.toml. Add #[derive(Validate)] to all request structs: UpdateDeviceRequest, CreateProfileRequest, SetProfileConfigRequest, SetKeyMappingRequest. Add validation attributes to struct fields: #[validate(length(min = 1, max = 100))] for names, #[validate(range(min = 0, max = 255))] for numeric values, ensure enums are validated. Call .validate() in handler functions before processing requests. Return ApiError::ValidationError with detailed message on validation failure (HTTP 400) | Restrictions: Must not change API response structure, validation errors must return HTTP 400 with clear messages, must preserve existing error handling, must validate all user inputs | Success: validator crate added to dependencies, all request structs derive Validate, validation attributes added to fields, handlers call .validate() before processing, validation errors return HTTP 400 with details, existing error handling preserved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (validation rules, request structs modified, error responses), then mark task complete [x] in tasks.md_

- [ ] 5. Create API contract tests
  - File: keyrx_ui/src/api/contracts.test.ts, keyrx_daemon/tests/api/contracts_test.rs
  - Write frontend contract tests with Zod schema validation
  - Write backend contract tests for request/response structures
  - Test all REST endpoints and WebSocket RPC methods
  - Purpose: Catch API contract violations in automated tests
  - _Leverage: keyrx_ui/src/api/schemas.ts, keyrx_ui/tests/testUtils.tsx, keyrx_daemon/tests/helpers/testUtils.rs_
  - _Requirements: 9.2, 9.3, 9.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in contract testing and API validation | Task: Create comprehensive contract tests following requirements 9.2, 9.3, 9.5. Create keyrx_ui/src/api/contracts.test.ts with tests for all REST endpoints: GET /api/devices validates against DeviceListResponseSchema, PATCH /api/devices/:id validates request and response, GET /api/profiles, POST /api/profiles, GET /api/profiles/:name/config, etc. Test WebSocket RPC message validation (Query, Command, Response types). Create keyrx_daemon/tests/api/contracts_test.rs with Rust-side tests verifying request deserialization, response serialization, and validation error responses (HTTP 400). Mock daemon for frontend tests. Use real handlers for backend tests | Restrictions: Must test all endpoints comprehensively, frontend tests must mock daemon (no real HTTP), backend tests use real handlers, must verify both success and validation error cases, must achieve 100% endpoint coverage | Success: Frontend contract tests created for all endpoints with Zod validation, backend contract tests created with validation error tests, WebSocket RPC messages tested, 100% endpoint coverage achieved, tests catch schema mismatches. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test files created, endpoints covered, validation tested), then mark task complete [x] in tasks.md_

- [ ] 6. Add pre-commit hook for type consistency
  - File: .git/hooks/pre-commit, scripts/check-types.sh
  - Create pre-commit hook to regenerate TypeScript types
  - Fail commit if types changed without being staged
  - Create manual type check script
  - Purpose: Prevent committing code with out-of-sync types
  - _Leverage: None_
  - _Requirements: 9.1, 9.4, 9.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer specializing in Git hooks and build automation | Task: Create pre-commit hook for type consistency following requirements 9.1, 9.4, 9.7. Create .git/hooks/pre-commit script that runs cargo typeshare to regenerate keyrx_ui/src/types/generated.ts, checks if file changed using git diff, fails commit with error message if types changed but not staged, provides instructions to review and stage changes. Create scripts/check-types.sh for manual type checking. Make both scripts executable (chmod +x). Handle merge commits gracefully (skip type check on merge) | Restrictions: Must not auto-commit generated types (require manual review), must provide clear error messages with instructions, must be executable, must handle merge commits, must not break existing pre-commit hooks | Success: .git/hooks/pre-commit created and executable, regenerates types on commit attempt, fails if types out of sync with clear error, provides review instructions, scripts/check-types.sh created for manual checking, merge commits handled correctly, existing hooks preserved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (hook scripts, check script, Git integration), then mark task complete [x] in tasks.md_

- [ ] 7. Add CI/CD type consistency check
  - File: .github/workflows/type-check.yml
  - Create GitHub Actions workflow for type checking
  - Run on all pushes and pull requests
  - Fail CI if types out of sync
  - Purpose: Enforce type consistency in CI/CD pipeline
  - _Leverage: .github/workflows/ci.yml_
  - _Requirements: 9.7, 9.8_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer specializing in CI/CD and GitHub Actions | Task: Create CI/CD type check workflow following requirements 9.7, 9.8. Create .github/workflows/type-check.yml that installs Rust toolchain, installs typeshare-cli (cargo install typeshare-cli), regenerates TypeScript types (cargo typeshare), checks for differences (git diff --exit-code keyrx_ui/src/types/generated.ts), fails with clear error if types differ showing diff, runs TypeScript compilation check (npm run type-check). Trigger on push and pull_request events. Add as job to main .github/workflows/ci.yml for integration. Cache cargo and npm dependencies for speed | Restrictions: Must fail build on type mismatch, must display helpful error with diff, must run on all branches, must integrate with existing CI workflow, must cache dependencies | Success: .github/workflows/type-check.yml created, runs on push and PR events, regenerates types correctly, detects type mismatches and fails build, displays clear error with diff, TypeScript compilation verified, integrated with main CI workflow, dependencies cached. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (workflow file, CI integration, caching setup), then mark task complete [x] in tasks.md_

## Phase 1: Foundation - Rhai Parser and Code Generator

- [ ] 8. Create RhaiParser utility module
  - File: keyrx_ui/src/utils/rhaiParser.ts
  - Implement Rhai script parsing functions
  - Define TypeScript interfaces for AST, DeviceBlock, KeyMapping, ParseError
  - Handle syntax errors with line numbers and suggestions
  - Purpose: Parse Rhai scripts into structured AST for visual editor
  - _Leverage: keyrx_ui/src/components/MonacoEditor.tsx_
  - _Requirements: 6.1, 6.2, 6.4, 7.1, 7.2, 7.3_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend TypeScript Developer specializing in parser implementation | Task: Create Rhai script parser following requirements 6.1, 6.2, 6.4, 7.1-7.3. Implement parseRhaiScript(script: string): ParseResult function. Define TypeScript interfaces: ParseResult, RhaiAST, DeviceBlock, KeyMapping, ParseError, KeyAction. Parse Rhai scripts to extract device() blocks with serial numbers, all key mapping types (simple map, tap_hold, macro, layer_switch), distinguish global vs device-specific scope, extract import statements, preserve comments with line numbers. Handle syntax errors gracefully returning ParseError with line, column, message, and suggestion. Leverage Monaco editor Rhai grammar from keyrx_ui/src/components/MonacoEditor.tsx for syntax understanding | Restrictions: Do not execute Rhai scripts (parsing only), must return structured error objects (never throw), preserve comment locations for code regeneration, parsing must complete within 100ms for 10,000 line scripts | Success: parseRhaiScript function implemented, all TypeScript interfaces defined, parser extracts device blocks and all mapping types, distinguishes global vs device-specific scope, handles syntax errors with actionable feedback, passes unit tests for all mapping types and error cases. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (parser functions, interfaces, mapping type support), then mark task complete [x] in tasks.md_

- [ ] 9. Create RhaiCodeGenerator utility module
  - File: keyrx_ui/src/utils/rhaiCodeGen.ts
  - Implement Rhai code generation functions
  - Generate clean, formatted code with proper indentation
  - Support all mapping types and device blocks
  - Purpose: Generate Rhai code from visual editor state
  - _Leverage: None (new utility)_
  - _Requirements: 6.2, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend TypeScript Developer specializing in code generation | Task: Create Rhai code generator following requirements 6.2, 8.1-8.7. Implement functions: generateRhaiScript(mappings: KeyMapping[], devices: DeviceBlock[]): string, generateDeviceBlock(device: DeviceBlock): string, generateKeyMapping(mapping: KeyMapping): string, formatRhaiScript(script: string): string. Generate valid Rhai syntax for all mapping types: map(Key::A, Key::B) for simple, tap_hold(tap, hold, threshold) for tap-hold, macro sequences, layer switches. Group device-specific mappings in device() blocks by serial number. Preserve comments and formatting where possible. Follow formatting rules: 4-space indentation, 100 character max line length, 1 blank line between device blocks | Restrictions: Must preserve existing comments when regenerating, group multiple mappings for same device in single device() block, maintain consistent formatting, code generation must complete within 50ms for 1,000 mappings | Success: All generation functions implemented, generates valid Rhai syntax for all mapping types, groups device blocks correctly, preserves comments and formatting, passes unit tests for simple/tap_hold/macro/layer mappings, formatting matches project standards. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (generator functions, code templates, formatting rules), then mark task complete [x] in tasks.md_

- [ ] 10. Create RhaiFormatter utility module
  - File: keyrx_ui/src/utils/rhaiFormatter.ts
  - Implement Rhai code formatting and indentation
  - Support configurable formatting options
  - Preserve comments while reformatting
  - Purpose: Ensure consistent Rhai code style
  - _Leverage: keyrx_ui/src/utils/rhaiCodeGen.ts_
  - _Requirements: 8.6, 8.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend TypeScript Developer specializing in code formatting | Task: Create Rhai formatter following requirements 8.6, 8.7. Implement functions: formatRhaiScript(script: string, options?: FormatOptions): string, indentBlock(lines: string[], level: number): string[], preserveComments(original: string, formatted: string): string. Define FormatOptions interface with indentSize (default 4), maxLineLength (default 100), blankLinesBetweenDevices (default 1). Preserve all inline and block comments in original positions. Leverage code generation utilities from keyrx_ui/src/utils/rhaiCodeGen.ts | Restrictions: Must preserve all comments in original positions, maintain semantic equivalence after formatting, formatting must complete within 50ms for 1,000 line scripts, do not reorder statements | Success: Formatting functions implemented, formats scripts with consistent indentation, preserves all comments, respects line length limits, passes unit tests for various input styles, integrates with code generator. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (formatter functions, comment preservation logic), then mark task complete [x] in tasks.md_

- [ ] 11. Write unit tests for RhaiParser
  - File: keyrx_ui/src/utils/rhaiParser.test.ts
  - Test parsing all mapping types
  - Test device block extraction
  - Test error handling
  - Purpose: Ensure parser reliability
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in TypeScript unit testing and Vitest | Task: Create comprehensive RhaiParser tests following requirements 7.1-7.6. Test all mapping types: simple (map(Key::A, Key::B)), tap_hold (tap_hold(Key::Escape, Key::LCtrl, 200)), macro, layer_switch. Test device block extraction with serial numbers. Test global vs device-specific scope identification. Test import statement parsing. Test comment preservation with line numbers. Test syntax error handling returning line numbers and suggestions. Test edge cases: empty scripts, comments-only scripts, scripts with 10,000+ lines. Use test utilities from keyrx_ui/tests/testUtils.tsx. Use descriptive test names | Restrictions: Must test both success and failure scenarios, maintain test isolation, use descriptive test names (test_[scenario]_[expected]), achieve 90% code coverage | Success: All parser functions tested comprehensively, edge cases covered (empty, large scripts, malformed syntax), error handling verified with line numbers, tests run independently, 90% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, test cases, coverage report), then mark task complete [x] in tasks.md_

- [ ] 12. Write unit tests for RhaiCodeGenerator
  - File: keyrx_ui/src/utils/rhaiCodeGen.test.ts
  - Test code generation for all mapping types
  - Test device block grouping
  - Test formatting and comment preservation
  - Purpose: Ensure code generation correctness
  - _Leverage: keyrx_ui/tests/testUtils.tsx, keyrx_ui/src/utils/rhaiParser.ts_
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in code generation validation | Task: Create comprehensive RhaiCodeGenerator tests following requirements 8.1-8.7. Test simple mapping generation (map(Key::A, Key::B)), tap-hold generation (tap_hold with threshold), macro sequences, layer switches. Test device block grouping (multiple mappings in single device() block). Test comment preservation. Test formatting (4-space indent, 100 char lines, 1 blank line between devices). Verify generated code is valid by parsing it with RhaiParser (round-trip test). Test edge cases: empty mappings, no devices, 1000+ mappings. Use test utilities | Restrictions: Must verify generated code parses correctly, test preservation of comments and formatting, test edge cases, achieve 90% coverage | Success: All generation functions tested, generated code is valid Rhai syntax, formatting rules verified, comment preservation tested, edge cases covered, round-trip tests pass (generate -> parse -> generate), 90% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, round-trip tests, coverage), then mark task complete [x] in tasks.md_

## Phase 2: Bidirectional Sync Engine

- [ ] 13. Create RhaiSyncEngine component
  - File: keyrx_ui/src/components/RhaiSyncEngine.tsx
  - Implement bidirectional sync coordinator
  - Manage sync state machine
  - Debounce code editor changes
  - Purpose: Coordinate real-time sync between editors
  - _Leverage: keyrx_ui/src/utils/rhaiParser.ts, keyrx_ui/src/utils/rhaiCodeGen.ts, keyrx_ui/src/hooks/useAutoSave.ts_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in state management and real-time sync | Task: Create RhaiSyncEngine component following requirements 6.1-6.7. Implement bidirectional sync: parse Rhai script on code editor changes and update visual editor within 500ms, generate Rhai code on visual editor changes immediately. Implement state machine: idle, parsing, generating, syncing, error. Debounce code editor changes (500ms) using useAutoSave hook. Preserve unsaved changes to localStorage. Handle parse errors gracefully: show last valid state in visual editor, display warning banner with error details. Leverage RhaiParser, RhaiCodeGen utilities | Restrictions: Must maintain sync state consistency, prevent infinite sync loops, handle parse errors without crashing, preserve user edits during sync, debounce must prevent excessive processing | Success: Bidirectional sync works correctly, visual editor updates reflect in code editor immediately, code editor changes update visual editor within 500ms, debouncing prevents excessive calls, parse errors display with last valid state preserved, unsaved changes persist to localStorage, state machine transitions correctly, no infinite sync loops. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (sync component, state machine, debouncing logic), then mark task complete [x] in tasks.md_

- [ ] 14. Write unit tests for RhaiSyncEngine
  - File: keyrx_ui/src/components/RhaiSyncEngine.test.tsx
  - Test sync from visual to code editor
  - Test sync from code to visual editor
  - Test debouncing and error handling
  - Purpose: Ensure sync engine reliability
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in React component testing | Task: Create comprehensive RhaiSyncEngine tests following requirements 6.1-6.3, 6.6-6.7. Test sync from visual to code (immediate update), sync from code to visual (500ms delay), debouncing behavior (multiple rapid changes result in single parse), parse error handling (show last valid state, display error), state machine transitions (idle→parsing→syncing→idle, error recovery), localStorage persistence (unsaved changes survive reload). Use renderWithProviders from testUtils. Mock RhaiParser and RhaiCodeGen | Restrictions: Must test both success and error paths, verify debouncing prevents excessive calls, test state machine exhaustively, mock localStorage correctly, achieve 85% coverage | Success: All sync scenarios tested (visual→code, code→visual), debouncing verified with timing tests, error handling tested (parse errors, localStorage failures), state machine transitions validated, localStorage persistence verified, 85% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, state machine tests, debounce tests), then mark task complete [x] in tasks.md_

## Phase 3: Device Selector Modifications

- [ ] 15. Modify DeviceSelector component
  - File: keyrx_ui/src/components/DeviceSelector.tsx
  - Remove scope toggle UI
  - Add multi-device checkboxes
  - Add global checkbox
  - Purpose: Enable device-aware configuration
  - _Leverage: keyrx_ui/src/hooks/useDevices.ts_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in UI components and accessibility | Task: Modify DeviceSelector component following requirements 1.1-1.4, 5.1-5.5. Remove scope selector toggle completely. Add multi-device selection with checkboxes (allow selecting multiple devices simultaneously). Add "Global" checkbox at top of device list. Display device connection status badges (connected: green, disconnected: gray). Allow selection of disconnected devices (for portable configs). Update props: multiSelect: boolean, showGlobalOption: boolean, selectedDevices: string[], globalSelected: boolean, onSelectionChange: (devices: string[], global: boolean) => void. Leverage useDevices hook | Restrictions: Must maintain WCAG 2.2 Level AA accessibility (ARIA labels, keyboard navigation), preserve existing device editing (name, layout), ensure responsive design (mobile and desktop), warn if no devices selected | Success: Scope toggle removed completely, multi-device checkboxes functional, "Global" option works correctly, connection status badges displayed, device editing preserved, accessibility compliant (keyboard nav, ARIA labels), responsive design maintained. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (component changes, new props, UI modifications), then mark task complete [x] in tasks.md_

- [ ] 16. Write unit tests for modified DeviceSelector
  - File: keyrx_ui/src/components/DeviceSelector.test.tsx
  - Test multi-device selection
  - Test global checkbox
  - Test accessibility
  - Purpose: Ensure component works correctly
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in React component testing and accessibility | Task: Update DeviceSelector tests following requirements 5.1-5.5. Test multi-device selection (check multiple devices, verify onSelectionChange called with array). Test "Global" checkbox (check global, verify devices parameter). Test connection status badges (connected shows green, disconnected shows gray). Test device editing (name changes persist, layout changes persist). Test accessibility (keyboard navigation with Tab, ARIA labels present, screen reader announcements). Use renderWithProviders, mock useDevices hook | Restrictions: Must verify scope toggle completely removed, test edge cases (all disconnected, no selection warning), validate WCAG 2.2 compliance, achieve 80% coverage | Success: Multi-device selection tested thoroughly, global checkbox works independently, connection badges verified, device editing preserved, accessibility tests pass (keyboard nav, ARIA, contrast), no scope toggle remains, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, accessibility tests, coverage), then mark task complete [x] in tasks.md_

## Phase 4: Devices Page Modifications

- [ ] 17. Add global layout selector to DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.tsx
  - Add "Global Settings" card at top
  - Implement global layout selector dropdown
  - Save to daemon configuration
  - Purpose: Set default keyboard layout
  - _Leverage: keyrx_ui/src/hooks/useDevices.ts_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in settings management | Task: Modify DevicesPage following requirements 2.1-2.5. Add "Global Settings" card at top of page. Implement layout selector dropdown with options: ANSI 104, ISO 105, JIS 109, HHKB, NUMPAD. Save global layout to daemon via new API endpoint PUT /api/settings/global-layout. Apply global layout as default for newly detected devices. Display current global layout prominently. Show save feedback (saving spinner, success checkmark, error). Leverage useDevices hook | Restrictions: Must not affect existing device-specific layout overrides, save must persist across daemon restarts, provide clear visual feedback on save, handle API errors gracefully | Success: Global Settings card displays at top, layout selector functional with all options, save persists to daemon, new devices inherit global layout, existing device overrides preserved, save feedback clear (spinner/success/error), error handling works. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (Global Settings card, API integration, save feedback), then mark task complete [x] in tasks.md_

- [ ] 18. Remove scope-related UI from DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.tsx
  - Remove all scope selector UI elements
  - Remove scope from device cards
  - Update device edit modal
  - Purpose: Align with Rhai-driven scope
  - _Leverage: keyrx_ui/src/components/DeviceSelector.tsx_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in refactoring | Task: Modify DevicesPage following requirements 1.1-1.4. Remove all scope selector UI (global/device-specific toggle). Remove scope field from device cards. Update device edit modal to show only name and layout fields. Update API calls (PATCH /api/devices/:id) to exclude scope parameter. Verify modified DeviceSelector is used correctly. Leverage updated DeviceSelector component | Restrictions: Must preserve all other device functionality (name editing, layout selection, forget device), ensure no scope-related data sent to API, maintain responsive design, preserve accessibility | Success: All scope UI elements removed completely, device edit modal shows only name and layout, API calls exclude scope parameter, device name and layout editing works, forget device preserved, responsive design maintained, no regressions. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (UI removed, API changes, preserved functionality), then mark task complete [x] in tasks.md_

- [ ] 19. Write integration tests for DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.test.tsx
  - Test global layout selector
  - Test device list and editing
  - Test responsive design
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in integration testing | Task: Create integration tests for DevicesPage following requirements 1.1-1.4, 2.1-2.5. Test global layout selector (change layout, save, verify persistence, verify new devices inherit). Test device list rendering (all devices shown, connection badges, name/serial/layout). Test device editing (inline name edit, layout selector, save, verify API excludes scope). Test forget device (confirmation dialog, delete, verify removed). Test responsive design (mobile vs desktop layout). Use renderWithProviders, mock API calls | Restrictions: Must verify scope completely absent (no UI, no API calls), test both online and offline device scenarios, validate error handling, achieve 80% coverage | Success: Global layout selector fully tested (save, inherit), device list tests complete (rendering, badges, editing), forget device tested with confirmation, responsive design verified, scope completely removed (no UI, no API), error scenarios covered, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, integration scenarios, coverage), then mark task complete [x] in tasks.md_

## Phase 5: Profiles Page Modifications

- [ ] 20. Add Rhai file path display to ProfilesPage
  - File: keyrx_ui/src/pages/ProfilesPage.tsx, keyrx_ui/src/components/ProfileCard.tsx
  - Display Rhai file path in profile cards
  - Add tooltip with full path
  - Make path clickable to navigate
  - Purpose: Show Rhai script location
  - _Leverage: keyrx_ui/src/hooks/useProfiles.ts_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in UI components | Task: Modify ProfilesPage and ProfileCard following requirements 3.1-3.4. Display Rhai file path on profile cards (show relative path like "~/.config/keyrx/profiles/gaming.rhai"). Add tooltip on hover showing full absolute path. Make path clickable to navigate to ConfigPage for that profile. Display error badge if Rhai file doesn't exist. Update ProfileCard component to receive rhaiPath prop and fileExists boolean. Leverage useProfiles hook | Restrictions: Must truncate long paths gracefully (ellipsis in middle), ensure tooltip is accessible (keyboard users can trigger), error badge must be visually distinct, maintain existing profile card functionality | Success: Rhai path displayed on each card, tooltip shows full path on hover and keyboard focus, path is clickable and navigates to config page, error badge shown for missing files, long paths truncated cleanly, accessibility maintained (ARIA labels, keyboard nav). After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (path display, tooltip, navigation), then mark task complete [x] in tasks.md_

- [ ] 21. Implement auto-generate default profile
  - File: keyrx_ui/src/pages/ProfilesPage.tsx
  - Check for empty profile list on load
  - Auto-generate "default" profile
  - Display notification
  - Purpose: Seamless first-time experience
  - _Leverage: keyrx_ui/src/hooks/useCreateProfile.ts, keyrx_ui/src/hooks/useActivateProfile.ts_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in user onboarding | Task: Modify ProfilesPage following requirements 4.1-4.5. Implement auto-generate default profile logic: check if profiles list is empty on page load (useEffect), call create profile API with name "default" and template "blank", display success notification ("Default profile created. Click 'Edit' to customize."), automatically activate the new profile, handle failures gracefully (show error with retry button, check daemon connection). Leverage useCreateProfile and useActivateProfile hooks | Restrictions: Must only auto-generate on first load with zero profiles, must not auto-generate if profiles exist, must handle daemon offline scenario, must provide clear error messages, must not block page rendering | Success: Default profile auto-generated when no profiles exist, notification displayed on success, profile activated automatically, error handling works (offline daemon, creation failure), no auto-generation when profiles exist, page loads without blocking, user can retry on error. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (auto-generation logic, error handling, notifications), then mark task complete [x] in tasks.md_

- [ ] 22. Write integration tests for ProfilesPage
  - File: keyrx_ui/src/pages/ProfilesPage.test.tsx
  - Test Rhai path display
  - Test auto-generate default profile
  - Test profile CRUD operations
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in integration testing | Task: Create integration tests for ProfilesPage following requirements 3.1-3.4, 4.1-4.5. Test Rhai path display (path shown, tooltip on hover, click navigates to config). Test error badge (shown when file missing, not shown when exists). Test auto-generate default (triggered when list empty, creates "default" profile with blank template, activates automatically, shows notification). Test auto-generate error handling (daemon offline, creation failure, retry button works). Test existing profile CRUD (create, edit, delete, activate, duplicate). Use renderWithProviders, mock API calls | Restrictions: Must test first-time user scenario (empty list → auto-generate), verify navigation to config page, validate error badge visibility, test notification display, achieve 80% coverage | Success: Rhai path display fully tested (show, tooltip, click), error badge tested (shown/hidden correctly), auto-generate tested (creates default, activates, notifies), error scenarios covered (offline, failure, retry), existing CRUD preserved and tested, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, user scenarios, coverage), then mark task complete [x] in tasks.md_

## Phase 6: Config Page Integration

- [ ] 23. Integrate RhaiSyncEngine into ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Add RhaiSyncEngine component
  - Connect visual and code editors
  - Display sync status
  - Purpose: Enable bidirectional sync
  - _Leverage: keyrx_ui/src/components/RhaiSyncEngine.tsx, keyrx_ui/src/components/MonacoEditor.tsx, keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in component integration | Task: Modify ConfigPage following requirements 6.1-6.3, 6.6-6.7. Integrate RhaiSyncEngine component. Connect visual editor state (KeyMapping[]) to sync engine via onVisualChange callback. Connect Monaco code editor content to sync engine via onCodeChange callback. Display sync status (parsing/generating/syncing/error) with loading indicators. Display parse errors with line numbers and suggestions. Handle tab switching (preserve sync state when switching between visual and code tabs). Leverage RhaiSyncEngine, MonacoEditor, KeyboardVisualizer | Restrictions: Must maintain sync state across tab switches, must not trigger sync on initial load, must handle parse errors gracefully (show last valid state), must preserve unsaved changes, must debounce code editor changes | Success: RhaiSyncEngine integrated and functional, visual editor changes update code immediately, code editor changes update visual within 500ms, sync status displayed clearly, parse errors shown with details, tab switching preserves state, unsaved changes preserved, no infinite sync loops. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (sync integration, state management, error display), then mark task complete [x] in tasks.md_

- [ ] 24. Add multi-device support to ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Update device selector for multi-device
  - Display multiple keyboards side-by-side
  - Filter mappings by selected devices
  - Purpose: Enable device-aware editing
  - _Leverage: keyrx_ui/src/components/DeviceSelector.tsx, keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in multi-view UI | Task: Modify ConfigPage following requirements 5.1-5.6. Update device selector to support multi-device selection (use modified DeviceSelector with multiSelect=true). Display multiple KeyboardVisualizer components side-by-side when multiple devices selected (one per device, labeled with device name). Display global keyboard when "Global" checked. Filter visual editor mappings by selected devices (show only relevant mappings). Generate device-specific device() blocks in Rhai when mappings exist for specific devices. Warn if no devices selected. Leverage DeviceSelector and KeyboardVisualizer | Restrictions: Must maintain performance with multiple visualizers (optimize re-renders), must clearly label each device's keyboard, must handle global + device-specific simultaneously, must warn if no selection, must generate correct Rhai structure | Success: Multi-device selection works (checkboxes for each device + global), multiple keyboards displayed side-by-side when needed, global keyboard shown when global selected, mappings filtered correctly per device, device-specific Rhai blocks generated on save, warning shown for no selection, performance acceptable with 3+ devices. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (multi-device display, filtering, Rhai generation), then mark task complete [x] in tasks.md_

- [ ] 25. Implement Rhai-driven device detection
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Parse Rhai to detect device blocks
  - Auto-populate device selector
  - Mark disconnected devices
  - Purpose: Sync device selector with Rhai
  - _Leverage: keyrx_ui/src/utils/rhaiParser.ts, keyrx_ui/src/components/DeviceSelector.tsx_
  - _Requirements: 5.6, 7.4, 7.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in data parsing | Task: Modify ConfigPage following requirements 5.6, 7.4-7.5. Implement Rhai-driven device detection: parse loaded Rhai script with RhaiParser to extract device() blocks, auto-populate device selector checkboxes based on devices found in script, mark devices as "connected" (currently attached) or "disconnected" (in script but not attached) with badges, display scope info derived from Rhai (global mappings vs device-specific mappings), allow editing disconnected devices (for portable configs). Leverage rhaiParser to extract DeviceBlock[], match against connected devices from API | Restrictions: Must handle devices in script that are not connected, must distinguish global vs device-specific scope from Rhai content (not UI setting), must update device selector when Rhai changes, must preserve disconnected device configs | Success: Device selector auto-populated from Rhai script on load, disconnected devices shown with badge, connected devices shown normally, scope info derived from Rhai (global vs device blocks), device selector updates when Rhai changes, disconnected device editing works, no manual scope selection UI. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (Rhai parsing, device detection, scope derivation), then mark task complete [x] in tasks.md_

- [ ] 26. Write integration tests for ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.test.tsx
  - Test bidirectional sync
  - Test multi-device display
  - Test Rhai-driven detection
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in complex integration testing | Task: Create integration tests for ConfigPage following requirements 5.1-5.6, 6.1-6.3, 6.6-6.7. Test bidirectional sync (visual change → code updated, code change → visual updated, debouncing works, no infinite loops). Test multi-device selection (select multiple devices → multiple keyboards shown, global + device shown simultaneously, mappings filtered correctly). Test Rhai-driven device detection (load script with device blocks → device selector auto-populated, disconnected devices shown with badge). Test parse error handling (syntax error → error displayed, last valid state shown, can fix in code editor). Test save functionality (generates correct Rhai with device blocks, persists to daemon, handles save errors). Use renderWithProviders, mock API and parser | Restrictions: Must test real user workflows (load profile, edit visual, switch to code, verify sync, save), test error scenarios (parse errors, save failures), verify timing (debounce, sync delays), achieve 80% coverage | Success: Bidirectional sync fully tested (both directions, debounce, errors), multi-device tested (multiple keyboards, filtering, global + device), Rhai-driven detection tested (auto-populate, disconnected badges), parse error handling verified, save tested (Rhai generation, API call, errors), user workflows validated end-to-end, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, integration workflows, coverage), then mark task complete [x] in tasks.md_

## Phase 7: Backend API Updates

- [ ] 27. Update device API to remove scope
  - File: keyrx_daemon/src/web/handlers/devices.rs
  - Remove scope field from PATCH endpoint
  - Update DeviceEntry struct
  - Migrate device registry
  - Purpose: Align backend with Rhai-driven scope
  - _Leverage: keyrx_daemon/src/web/handlers/devices.rs_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer specializing in REST API | Task: Modify device API following requirements 1.1-1.4. Remove scope field from PATCH /api/devices/:id endpoint request body. Remove scope field from DeviceEntry struct in keyrx_daemon/src/web/handlers/devices.rs. Update device registry JSON serialization to exclude scope. Ensure existing device registry files migrate gracefully (ignore scope field if present, don't error). Preserve all other device fields (id, name, serial, layout) | Restrictions: Must maintain backward compatibility with existing registry files (ignore unknown fields), must not break existing device CRUD operations, must update API documentation, must preserve device identification and layout functionality | Success: PATCH endpoint no longer accepts scope field, DeviceEntry struct excludes scope, device registry serializes without scope, existing registry files load correctly (ignoring old scope field), device CRUD operations work normally (name, layout, forget), API tests updated and passing. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (API changes, struct modifications, migration handling), then mark task complete [x] in tasks.md_

- [ ] 28. Add global layout settings API
  - File: keyrx_daemon/src/web/handlers/settings.rs
  - Create PUT /api/settings/global-layout endpoint
  - Create GET endpoint for current layout
  - Store in daemon configuration
  - Purpose: Support global layout from UI
  - _Leverage: keyrx_daemon/src/web/handlers/mod.rs_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer specializing in configuration management | Task: Create settings API handler following requirements 2.1-2.5. Implement PUT /api/settings/global-layout endpoint (accepts layout: LayoutPreset in request body, saves to daemon config). Implement GET /api/settings/global-layout endpoint (returns current global layout). Store global layout in daemon configuration file (~/.config/keyrx/daemon.toml). Apply global layout to newly detected devices (when no device-specific override exists). Create new file keyrx_daemon/src/web/handlers/settings.rs with handler functions. Register routes in mod.rs | Restrictions: Must persist across daemon restarts, must not override existing device-specific layouts, must validate layout enum (ANSI_104, ISO_105, JIS_109, HHKB, NUMPAD), must return clear error if invalid layout | Success: PUT endpoint saves global layout to config, GET endpoint returns current global layout, global layout persists across restarts, new devices inherit global layout by default, device-specific overrides still work, validation rejects invalid layouts, API tests passing. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (API endpoints, config persistence, device application), then mark task complete [x] in tasks.md_

- [ ] 29. Write backend API tests
  - File: keyrx_daemon/tests/api/devices_test.rs, keyrx_daemon/tests/api/settings_test.rs
  - Test device API without scope
  - Test global layout API
  - Test device layout inheritance
  - Purpose: Ensure backend changes work
  - _Leverage: keyrx_daemon/tests/helpers/testUtils.rs_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in Rust API testing | Task: Create backend API tests following requirements 1.1-1.4, 2.1-2.5. Test device API changes (PATCH without scope succeeds, scope field ignored if sent, DeviceEntry has no scope, existing registry files load correctly). Test global layout API (PUT with valid layout saves, GET returns saved layout, PUT with invalid layout returns 400). Test device layout inheritance (new device inherits global layout, device-specific override takes precedence, global layout change doesn't affect existing overrides). Use test utilities from keyrx_daemon/tests/helpers/testUtils.rs | Restrictions: Must test both success and error scenarios, must verify persistence (restart daemon between tests), must validate migration from old registry format, achieve 80% coverage for new code | Success: Device API tests pass without scope field, global layout API fully tested (GET/PUT, validation, persistence), device inheritance tested (global default, device override, precedence), migration from old registry verified, error scenarios covered, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test files, test scenarios, coverage), then mark task complete [x] in tasks.md_

## Phase 8: Documentation and Final Integration

- [ ] 30. Update frontend documentation
  - File: keyrx_ui/README.md
  - Document RhaiParser, RhaiCodeGen, RhaiFormatter
  - Document RhaiSyncEngine
  - Document component modifications
  - Purpose: Ensure future developers understand architecture
  - _Leverage: None_
  - _Requirements: All_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer specializing in frontend documentation | Task: Update keyrx_ui/README.md covering all requirements. Document new utilities: RhaiParser (parsing Rhai to AST, usage examples), RhaiCodeGen (generating Rhai from mappings, code templates), RhaiFormatter (formatting rules). Document RhaiSyncEngine component (bidirectional sync, state machine, debouncing). Document modified components: DeviceSelector (multi-device, no scope), ConfigPage (device-aware editing, Rhai-driven scope), DevicesPage (global layout, removed scope), ProfilesPage (Rhai path, auto-generate). Include architecture diagrams, code examples, usage patterns | Restrictions: Must be clear and concise, must include code examples, must explain architectural decisions, must document testing approach | Success: README updated with comprehensive documentation, utilities documented with usage examples, components documented with props and behavior, architecture explained clearly, testing approach documented, code examples included, useful for developers and AI agents. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (documentation sections, examples, diagrams), then mark task complete [x] in tasks.md_

- [ ] 31. Update backend documentation
  - File: keyrx_daemon/README.md
  - Document API changes
  - Document device registry changes
  - Document migration path
  - Purpose: Ensure backend changes documented
  - _Leverage: None_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer specializing in backend API documentation | Task: Update keyrx_daemon/README.md covering requirements 1.1-1.4, 2.1-2.5. Document API changes: PATCH /api/devices/:id no longer accepts scope, DeviceEntry struct excludes scope, migration handling for old registry files. Document new global layout API: PUT /api/settings/global-layout, GET /api/settings/global-layout, request/response formats. Document device registry format changes (removed scope, backward compatibility). Document device layout inheritance (global default, device-specific override). Include API examples and migration notes | Restrictions: Must be clear and accurate, must include request/response examples, must document migration path, must explain architectural rationale (Rhai-driven scope) | Success: README updated with API changes, removed scope documented, global layout API documented with examples, device registry format changes explained, migration path documented, architectural rationale explained, useful for developers and AI agents. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (documentation sections, API examples, migration guide), then mark task complete [x] in tasks.md_

- [ ] 32. Create architecture diagram
  - File: keyrx_ui/docs/architecture.md
  - Create Mermaid diagrams
  - Document bidirectional sync flow
  - Document data flow
  - Purpose: Visualize architecture
  - _Leverage: None_
  - _Requirements: All_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Architect specializing in system design documentation | Task: Create keyrx_ui/docs/architecture.md covering all requirements. Create Mermaid diagrams for: bidirectional sync flow (Visual Editor ↔ RhaiSyncEngine ↔ Code Editor, parsing/code generation paths, state machine), Rhai parser/codegen architecture (RhaiParser input/output, AST structure, RhaiCodeGen input/output, formatting), device-aware configuration flow (device selection → keyboard display → mapping filtering → Rhai generation), data flow (ConfigPage state → DeviceSelector → KeyboardVisualizer → RhaiSyncEngine → API). Include component interaction diagrams and sequence diagrams | Restrictions: Must use Mermaid syntax for diagrams, must be clear and comprehensive, must show all major components and data flows, must explain Rhai-driven scope architecture | Success: Architecture document created with multiple diagrams, bidirectional sync flow visualized clearly, Rhai parser/codegen architecture explained, device-aware flow documented, data flow diagrams complete, diagrams render correctly in Markdown viewers, useful for onboarding and AI agents. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (diagrams created, architecture documented), then mark task complete [x] in tasks.md_

- [ ] 33. Run final integration testing
  - File: Multiple files (verification)
  - Run full test suite
  - Fix failing tests
  - Remove deprecated code
  - Purpose: Ensure everything works together
  - _Leverage: All previous tasks_
  - _Requirements: All_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior QA Engineer specializing in integration testing | Task: Perform final integration testing covering all requirements. Run full test suite: frontend (npm test), backend (cargo test), type validation (npm run type-check, cargo typeshare). Fix any failing tests. Remove all deprecated scope-related code (search for "scope" in codebase, remove UI elements, update tests). Verify WCAG 2.2 Level AA accessibility (npm run test:a11y, fix violations). Run end-to-end user workflows (first-time user with auto-generate, multi-device config, Rhai editing). Check for regressions (device management, profile management, save/load). Use all previous tasks as leverage | Restrictions: Must achieve 100% test pass rate, must remove all scope-related code, must verify accessibility compliance, must test all user workflows, must check for performance regressions | Success: All tests passing (frontend and backend), deprecated scope code removed completely, accessibility tests passing (zero violations), end-to-end workflows validated (first-time user, multi-device, Rhai editing), no regressions found, code clean and ready for merge. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test results, removed code, accessibility report, workflow validation), then mark task complete [x] in tasks.md_

- [ ] 34. Verify type safety infrastructure
  - File: Multiple files (verification)
  - Verify typeshare generates correct types
  - Verify Zod validates all endpoints
  - Verify contract tests pass
  - Purpose: Ensure type safety is working
  - _Leverage: Phase 0 tasks_
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior QA Engineer specializing in type safety validation | Task: Verify type safety infrastructure following requirements 9.1-9.8. Verify typeshare generates correct TypeScript types (run cargo typeshare, check generated.ts, verify types compile). Verify Zod validates all API endpoints (run contract tests, check validation errors caught). Verify contract tests pass 100% (frontend and backend tests). Verify pre-commit hook works (make dummy Rust struct change, try committing, verify hook catches it). Verify CI/CD type check works (check GitHub Actions, verify failures on type mismatch). Test breaking API change detection (modify Rust struct, verify build fails with diff). Leverage Phase 0 tasks | Restrictions: Must verify all type safety mechanisms work, must test both success and failure scenarios, must ensure breaking changes caught, must validate error messages are clear | Success: typeshare generates correct types, Zod validation catches invalid responses, contract tests pass 100%, pre-commit hook prevents out-of-sync commits, CI/CD fails on type mismatch with clear diff, breaking API changes detected and build fails with helpful error, all type safety mechanisms validated. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (validation results, type safety verification, CI/CD checks), then mark task complete [x] in tasks.md_
