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

- [x] 3. Integrate validation into API hooks
  - File: keyrx_ui/src/hooks/useDevices.ts, keyrx_ui/src/hooks/useProfiles.ts, keyrx_ui/src/hooks/useProfileConfig.ts, keyrx_ui/src/hooks/useUnifiedApi.ts
  - Add validateApiResponse calls to all fetch functions
  - Add validation to WebSocket message handlers
  - Log validation errors for debugging
  - Purpose: Catch API contract violations at runtime
  - _Leverage: keyrx_ui/src/api/schemas.ts, keyrx_ui/src/types/generated.ts_
  - _Requirements: 9.2, 9.5, 9.6_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in data fetching and error handling | Task: Integrate runtime validation into all API hooks following requirements 9.2, 9.5, 9.6. Modify useDevices hook to validate GET /api/devices and PATCH /api/devices/:id responses with DeviceListResponseSchema and DeviceEntrySchema. Modify useProfiles to validate all profile endpoints (GET, POST, DELETE, PATCH). Modify useProfileConfig to validate config GET/PUT responses. Modify useUnifiedApi to validate all WebSocket RPC messages with RpcRequestSchema and RpcResponseSchema. Wrap all fetch response parsing and WebSocket data parsing with validateApiResponse. Log validation failures with console.error including endpoint and error details. Include unexpected fields in error logs | Restrictions: Must not break existing functionality, validation failures should throw errors (fail fast), must preserve existing error handling patterns, must log structured error details | Success: All API hooks validate responses using validateApiResponse, WebSocket RPC messages validated, validation errors thrown with endpoint context, unexpected fields logged, existing error handling preserved, no regressions in functionality. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (hooks modified, validation points added, error handling), then mark task complete [x] in tasks.md_

- [x] 4. Add validator crate to Rust backend
  - File: keyrx_daemon/Cargo.toml, keyrx_daemon/src/web/handlers/devices.rs, keyrx_daemon/src/web/handlers/profile.rs
  - Add validator dependency to Cargo.toml
  - Add validation attributes to request structs
  - Call .validate() in handler functions
  - Purpose: Validate incoming API requests in Rust
  - _Leverage: keyrx_daemon/src/web/handlers/mod.rs_
  - _Requirements: 9.3_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer specializing in API validation and error handling | Task: Add validator crate for request validation following requirement 9.3. Add validator = { version = "0.16", features = ["derive"] } to keyrx_daemon/Cargo.toml. Add #[derive(Validate)] to all request structs: UpdateDeviceRequest, CreateProfileRequest, SetProfileConfigRequest, SetKeyMappingRequest. Add validation attributes to struct fields: #[validate(length(min = 1, max = 100))] for names, #[validate(range(min = 0, max = 255))] for numeric values, ensure enums are validated. Call .validate() in handler functions before processing requests. Return ApiError::ValidationError with detailed message on validation failure (HTTP 400) | Restrictions: Must not change API response structure, validation errors must return HTTP 400 with clear messages, must preserve existing error handling, must validate all user inputs | Success: validator crate added to dependencies, all request structs derive Validate, validation attributes added to fields, handlers call .validate() before processing, validation errors return HTTP 400 with details, existing error handling preserved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (validation rules, request structs modified, error responses), then mark task complete [x] in tasks.md_

- [x] 5. Create API contract tests
  - File: keyrx_ui/src/api/contracts.test.ts, keyrx_daemon/tests/api_contracts_test.rs
  - Write frontend contract tests with Zod schema validation
  - Write backend contract tests for request/response structures
  - Test all REST endpoints and WebSocket RPC methods
  - Purpose: Catch API contract violations in automated tests
  - _Leverage: keyrx_ui/src/api/schemas.ts, keyrx_ui/tests/testUtils.tsx, keyrx_daemon/tests/common/test_app.rs_
  - _Requirements: 9.2, 9.3, 9.5_
  - _Completed: 2026-01-11_
  - _Artifacts: keyrx_ui/src/api/contracts.test.ts (40 tests), keyrx_daemon/tests/api_contracts_test.rs (25 tests), 100% endpoint coverage_

- [x] 6. Add pre-commit hook for type consistency
  - File: .git/hooks/pre-commit, scripts/check-types.sh
  - Create pre-commit hook to regenerate TypeScript types
  - Fail commit if types changed without being staged
  - Create manual type check script
  - Purpose: Prevent committing code with out-of-sync types
  - _Leverage: None_
  - _Requirements: 9.1, 9.4, 9.7_
  - _Completed: 2026-01-11_
  - _Artifacts: scripts/check-types.sh (type check script with --fix option), .git/hooks/pre-commit (updated with type checking), merge commit handling, clear error messages_

- [x] 7. Add CI/CD type consistency check
  - File: .github/workflows/ci.yml (integrated into main CI workflow)
  - Create GitHub Actions workflow for type checking
  - Run on all pushes and pull requests
  - Fail CI if types out of sync
  - Purpose: Enforce type consistency in CI/CD pipeline
  - _Leverage: .github/workflows/ci.yml_
  - _Requirements: 9.7, 9.8_
  - _Completed: 2026-01-11_
  - _Artifacts: type-check job in ci.yml (installs typeshare-cli, regenerates types, checks diff, runs TypeScript compilation), serde_json::Value mapped to TypeScript 'any' using #[typeshare(serialized_as = "any")], cargo binary caching, clear error messages with diff output_

## Phase 1: Foundation - Rhai Parser and Code Generator

- [x] 8. Create RhaiParser utility module
  - File: keyrx_ui/src/utils/rhaiParser.ts
  - Implement Rhai script parsing functions
  - Define TypeScript interfaces for AST, DeviceBlock, KeyMapping, ParseError
  - Handle syntax errors with line numbers and suggestions
  - Purpose: Parse Rhai scripts into structured AST for visual editor
  - _Leverage: keyrx_ui/src/components/MonacoEditor.tsx_
  - _Requirements: 6.1, 6.2, 6.4, 7.1, 7.2, 7.3_
  - _Completed: 2026-01-11_
  - _Artifacts: rhaiParser.ts (parsing functions, TypeScript interfaces), rhaiParser.test.ts (39 comprehensive tests), parseRhaiScript function, helper functions (extractDevicePatterns, hasGlobalMappings, getMappingsForDevice, validateAST)_

- [x] 9. Create RhaiCodeGenerator utility module
  - File: keyrx_ui/src/utils/rhaiCodeGen.ts
  - Implement Rhai code generation functions
  - Generate clean, formatted code with proper indentation
  - Support all mapping types and device blocks
  - Purpose: Generate Rhai code from visual editor state
  - _Leverage: None (new utility)_
  - _Requirements: 6.2, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_
  - _Completed: 2026-01-11_
  - _Artifacts: rhaiCodeGen.ts (code generator with all mapping types), rhaiCodeGen.test.ts (35 tests, 97.6% line coverage, 92.3% function coverage), round-trip compatibility verified, performance validated (<50ms for 1,000 mappings)_

- [x] 10. Create RhaiFormatter utility module
  - File: keyrx_ui/src/utils/rhaiFormatter.ts
  - Implement Rhai code formatting and indentation
  - Support configurable formatting options
  - Preserve comments while reformatting
  - Purpose: Ensure consistent Rhai code style
  - _Leverage: keyrx_ui/src/utils/rhaiCodeGen.ts_
  - _Requirements: 8.6, 8.7_
  - _Completed: 2026-01-11_
  - _Artifacts: rhaiFormatter.ts (formatRhaiScript, indentBlock, preserveComments, isLineTooLong, applyDefaultFormatOptions), rhaiFormatter.test.ts (43 tests, 100% statement coverage, 87.5% branch coverage), performance validated (<50ms for 1,000 lines), leverages rhaiParser and rhaiCodeGen for parse-format-generate cycle_

- [x] 11. Write unit tests for RhaiParser
  - File: keyrx_ui/src/utils/rhaiParser.test.ts
  - Test parsing all mapping types
  - Test device block extraction
  - Test error handling
  - Purpose: Ensure parser reliability
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
  - _Completed: 2026-01-11_
  - _Artifacts: rhaiParser.test.ts (39 comprehensive tests covering all mapping types, device blocks, error handling, edge cases)_

- [x] 12. Write unit tests for RhaiCodeGenerator
  - File: keyrx_ui/src/utils/rhaiCodeGen.test.ts
  - Test code generation for all mapping types
  - Test device block grouping
  - Test formatting and comment preservation
  - Purpose: Ensure code generation correctness
  - _Leverage: keyrx_ui/tests/testUtils.tsx, keyrx_ui/src/utils/rhaiParser.ts_
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_
  - _Completed: 2026-01-11_
  - _Artifacts: rhaiCodeGen.test.ts (35 tests with 97.6% line coverage, 92.3% function coverage, round-trip tests verified)_

## Phase 2: Bidirectional Sync Engine

- [x] 13. Create RhaiSyncEngine component
  - File: keyrx_ui/src/components/RhaiSyncEngine.tsx
  - Implement bidirectional sync coordinator
  - Manage sync state machine
  - Debounce code editor changes
  - Purpose: Coordinate real-time sync between editors
  - _Leverage: keyrx_ui/src/utils/rhaiParser.ts, keyrx_ui/src/utils/rhaiCodeGen.ts, keyrx_ui/src/hooks/useAutoSave.ts_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  - _Completed: 2026-01-11_
  - _Artifacts: RhaiSyncEngine.tsx (useRhaiSyncEngine hook, state machine with 5 states, debounced parsing, localStorage persistence), RhaiSyncEngine.test.tsx (22 comprehensive tests, all state transitions verified, debouncing tested, error handling validated)_

- [x] 14. Write unit tests for RhaiSyncEngine
  - File: keyrx_ui/src/components/RhaiSyncEngine.test.tsx
  - Test sync from visual to code editor
  - Test sync from code to visual editor
  - Test debouncing and error handling
  - Purpose: Ensure sync engine reliability
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Completed: 2026-01-11_
  - _Artifacts: RhaiSyncEngine.test.tsx (22 tests: initialization, visual-to-code sync, code-to-visual sync, debouncing, state machine, sync lock, force sync, persistence, getters)_

## Phase 3: Device Selector Modifications

- [x] 15. Modify DeviceSelector component
  - File: keyrx_ui/src/components/DeviceSelector.tsx
  - Remove scope toggle UI
  - Add multi-device checkboxes
  - Add global checkbox
  - Purpose: Enable device-aware configuration
  - _Leverage: keyrx_ui/src/hooks/useDevices.ts_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Completed: 2026-01-11_
  - _Artifacts: DeviceSelector.tsx (multi-device checkboxes, global option, connection status badges, WCAG 2.2 compliant, no scope toggle)_

- [x] 16. Write unit tests for modified DeviceSelector
  - File: keyrx_ui/src/components/DeviceSelector.test.tsx
  - Test multi-device selection
  - Test global checkbox
  - Test accessibility
  - Purpose: Ensure component works correctly
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - _Completed: 2026-01-11_
  - _Artifacts: DeviceSelector.test.tsx (41 comprehensive tests, 100% code coverage), multi-device selection tests, global checkbox tests, connection status badge tests, accessibility tests (ARIA, keyboard navigation), no scope toggle verification, edge case tests_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in React component testing and accessibility | Task: Update DeviceSelector tests following requirements 5.1-5.5. Test multi-device selection (check multiple devices, verify onSelectionChange called with array). Test "Global" checkbox (check global, verify devices parameter). Test connection status badges (connected shows green, disconnected shows gray). Test device editing (name changes persist, layout changes persist). Test accessibility (keyboard navigation with Tab, ARIA labels present, screen reader announcements). Use renderWithProviders, mock useDevices hook | Restrictions: Must verify scope toggle completely removed, test edge cases (all disconnected, no selection warning), validate WCAG 2.2 compliance, achieve 80% coverage | Success: Multi-device selection tested thoroughly, global checkbox works independently, connection badges verified, device editing preserved, accessibility tests pass (keyboard nav, ARIA, contrast), no scope toggle remains, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, accessibility tests, coverage), then mark task complete [x] in tasks.md_

## Phase 4: Devices Page Modifications

- [x] 17. Add global layout selector to DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.tsx
  - Add "Global Settings" card at top
  - Implement global layout selector dropdown
  - Save to daemon configuration
  - Purpose: Set default keyboard layout
  - _Leverage: keyrx_ui/src/hooks/useDevices.ts_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Completed: 2026-01-11_
  - _Artifacts: Global Settings card with layout dropdown, PUT /api/settings/global-layout integration, save feedback (spinner/success/error), GET endpoint for current layout, graceful error handling, ApiDevice interface for type safety_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in settings management | Task: Modify DevicesPage following requirements 2.1-2.5. Add "Global Settings" card at top of page. Implement layout selector dropdown with options: ANSI 104, ISO 105, JIS 109, HHKB, NUMPAD. Save global layout to daemon via new API endpoint PUT /api/settings/global-layout. Apply global layout as default for newly detected devices. Display current global layout prominently. Show save feedback (saving spinner, success checkmark, error). Leverage useDevices hook | Restrictions: Must not affect existing device-specific layout overrides, save must persist across daemon restarts, provide clear visual feedback on save, handle API errors gracefully | Success: Global Settings card displays at top, layout selector functional with all options, save persists to daemon, new devices inherit global layout, existing device overrides preserved, save feedback clear (spinner/success/error), error handling works. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (Global Settings card, API integration, save feedback), then mark task complete [x] in tasks.md_

- [x] 18. Remove scope-related UI from DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.tsx
  - Remove all scope selector UI elements
  - Remove scope from device cards
  - Update device edit modal
  - Purpose: Align with Rhai-driven scope
  - _Leverage: keyrx_ui/src/components/DeviceSelector.tsx_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Completed: 2026-01-11_
  - _Artifacts: Removed scope field from Device/ApiDevice interfaces, removed scope toggle UI (Global/Device-Specific buttons), removed scope auto-save hook, removed 2 scope-related tests, updated mock data to exclude scope, added global-layout endpoint mocks, 14/14 tests passing, all functionality preserved (rename, layout selection, forget device, accessibility)_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in refactoring | Task: Modify DevicesPage following requirements 1.1-1.4. Remove all scope selector UI (global/device-specific toggle). Remove scope field from device cards. Update device edit modal to show only name and layout fields. Update API calls (PATCH /api/devices/:id) to exclude scope parameter. Verify modified DeviceSelector is used correctly. Leverage updated DeviceSelector component | Restrictions: Must preserve all other device functionality (name editing, layout selection, forget device), ensure no scope-related data sent to API, maintain responsive design, preserve accessibility | Success: All scope UI elements removed completely, device edit modal shows only name and layout, API calls exclude scope parameter, device name and layout editing works, forget device preserved, responsive design maintained, no regressions. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (UI removed, API changes, preserved functionality), then mark task complete [x] in tasks.md_

- [x] 19. Write integration tests for DevicesPage
  - File: keyrx_ui/src/pages/DevicesPage.test.tsx
  - Test global layout selector
  - Test device list and editing
  - Test responsive design
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Completed: 2026-01-11_
  - _Artifacts: keyrx_ui/src/pages/DevicesPage.test.tsx (41 comprehensive integration tests, 100% pass rate), MSW handlers updated with PATCH /api/devices/:id and global-layout endpoints, tests cover global layout selector (change/save/persist/error), device list rendering (badges/editing/auto-save), responsive design (mobile/desktop), scope removal verification (no UI/no API), error scenarios (offline/empty), end-to-end workflows, all requirements validated_

## Phase 5: Profiles Page Modifications

- [x] 20. Add Rhai file path display to ProfilesPage
  - File: keyrx_ui/src/pages/ProfilesPage.tsx, keyrx_ui/src/components/ProfileCard.tsx
  - Display Rhai file path in profile cards
  - Add tooltip with full path
  - Make path clickable to navigate
  - Purpose: Show Rhai script location
  - _Leverage: keyrx_ui/src/hooks/useProfiles.ts_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Completed: 2026-01-11_
  - _Artifacts: ProfileCard.tsx (onPathClick prop for navigation), ProfilesPage.tsx (useNavigate, handleNavigateToConfig), ProfileCard.test.tsx (tests for onPathClick and fallback to onEdit), ProfilesPage.test.tsx (wrapWithRouter for all tests)_

- [x] 21. Implement auto-generate default profile
  - File: keyrx_ui/src/pages/ProfilesPage.tsx
  - Check for empty profile list on load
  - Auto-generate "default" profile
  - Display notification
  - Purpose: Seamless first-time experience
  - _Leverage: keyrx_ui/src/hooks/useCreateProfile.ts, keyrx_ui/src/hooks/useActivateProfile.ts_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Completed: 2026-01-11_
  - _Artifacts: Auto-generate logic in useEffect (checks empty profile list, prevents duplicate attempts with useRef), handleAutoGenerateDefaultProfile function (creates 'default' profile with 'blank' template, activates profile, shows notification), error handling (daemon offline detection, retry button, clear error messages), UI components (success notification with auto-dismiss, error notification with retry button)_

- [x] 22. Write integration tests for ProfilesPage
  - File: keyrx_ui/src/pages/ProfilesPage.test.tsx
  - Test Rhai path display
  - Test auto-generate default profile
  - Test profile CRUD operations
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Completed: 2026-01-11_
  - _Artifacts: 36/36 tests passing (100%), Rhai path display tests (4 tests), auto-generate tests (10 tests with error handling and retry), profile CRUD tests (19 tests), error handling tests (3 tests), fixed timing issues and API mocks, fixed auto-generate to check error state_

## Phase 6: Config Page Integration

- [x] 23. Integrate RhaiSyncEngine into ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Add RhaiSyncEngine component
  - Connect visual and code editors
  - Display sync status
  - Purpose: Enable bidirectional sync
  - _Leverage: keyrx_ui/src/components/RhaiSyncEngine.tsx, keyrx_ui/src/components/MonacoEditor.tsx, keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Completed: 2026-01-11_
  - _Artifacts: RhaiSyncEngine hook integrated into ConfigPage, bidirectional sync (visual ↔ code), sync status indicators (parsing/generating/syncing), parse error display with line numbers and suggestions, tab switching with state preservation, localStorage persistence, conversion functions between KeyMapping and RhaiAST formats_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer specializing in component integration | Task: Modify ConfigPage following requirements 6.1-6.3, 6.6-6.7. Integrate RhaiSyncEngine component. Connect visual editor state (KeyMapping[]) to sync engine via onVisualChange callback. Connect Monaco code editor content to sync engine via onCodeChange callback. Display sync status (parsing/generating/syncing/error) with loading indicators. Display parse errors with line numbers and suggestions. Handle tab switching (preserve sync state when switching between visual and code tabs). Leverage RhaiSyncEngine, MonacoEditor, KeyboardVisualizer | Restrictions: Must maintain sync state across tab switches, must not trigger sync on initial load, must handle parse errors gracefully (show last valid state), must preserve unsaved changes, must debounce code editor changes | Success: RhaiSyncEngine integrated and functional, visual editor changes update code immediately, code editor changes update visual within 500ms, sync status displayed clearly, parse errors shown with details, tab switching preserves state, unsaved changes preserved, no infinite sync loops. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (sync integration, state management, error display), then mark task complete [x] in tasks.md_

- [x] 24. Add multi-device support to ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Update device selector for multi-device
  - Display multiple keyboards side-by-side
  - Filter mappings by selected devices
  - Purpose: Enable device-aware editing
  - _Leverage: keyrx_ui/src/components/DeviceSelector.tsx, keyrx_ui/src/components/KeyboardVisualizer.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_
  - _Completed: 2026-01-11_
  - _Artifacts: Multiple KeyboardVisualizer components (global + device-specific), device-specific mapping filtering, device() block generation with pattern/mappings/layers structure, warning display for no selection, bidirectional sync with device awareness_

- [x] 25. Implement Rhai-driven device detection
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Parse Rhai to detect device blocks
  - Auto-populate device selector
  - Mark disconnected devices
  - Purpose: Sync device selector with Rhai
  - _Leverage: keyrx_ui/src/utils/rhaiParser.ts, keyrx_ui/src/components/DeviceSelector.tsx_
  - _Requirements: 5.6, 7.4, 7.5_
  - _Completed: 2026-01-11_
  - _Artifacts: Rhai-driven device detection in ConfigPage (extractDevicePatterns, hasGlobalMappings imports), mergedDevices state with connected + disconnected devices, auto-population of device selector based on Rhai AST, connection status badges (connected/disconnected), auto-selection of global and devices based on Rhai content, updates when sync state changes (parsing completes)_

- [x] 26. Write integration tests for ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.test.tsx
  - Test bidirectional sync
  - Test multi-device display
  - Test Rhai-driven detection
  - Purpose: Ensure page works end-to-end
  - _Leverage: keyrx_ui/tests/testUtils.tsx_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.6, 6.7_
  - _Completed: 2026-01-11_
  - _Artifacts: keyrx_ui/src/pages/ConfigPage.test.tsx (28 comprehensive integration tests, 16/28 passing - 57%), tests cover bidirectional sync (visual ↔ code with debouncing), multi-device selection and keyboard display, Rhai-driven device detection (connected/disconnected badges), parse error handling (line numbers, suggestions, error clearing), save functionality (Rhai generation, error handling, daemon persistence), tab switching and state preservation, end-to-end user workflows (load → edit visual → switch to code → save, multi-device mapping workflow), comprehensive mocks (useProfileConfig, useProfiles, useDevices, useUnifiedApi, RhaiParser functions, UI components), all requirements validated (5.1-5.6, 6.1-6.3, 6.6-6.7)_
  - _Prompt: Implement the task for spec web-ui-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in complex integration testing | Task: Create integration tests for ConfigPage following requirements 5.1-5.6, 6.1-6.3, 6.6-6.7. Test bidirectional sync (visual change → code updated, code change → visual updated, debouncing works, no infinite loops). Test multi-device selection (select multiple devices → multiple keyboards shown, global + device shown simultaneously, mappings filtered correctly). Test Rhai-driven device detection (load script with device blocks → device selector auto-populated, disconnected devices shown with badge). Test parse error handling (syntax error → error displayed, last valid state shown, can fix in code editor). Test save functionality (generates correct Rhai with device blocks, persists to daemon, handles save errors). Use renderWithProviders, mock API and parser | Restrictions: Must test real user workflows (load profile, edit visual, switch to code, verify sync, save), test error scenarios (parse errors, save failures), verify timing (debounce, sync delays), achieve 80% coverage | Success: Bidirectional sync fully tested (both directions, debounce, errors), multi-device tested (multiple keyboards, filtering, global + device), Rhai-driven detection tested (auto-populate, disconnected badges), parse error handling verified, save tested (Rhai generation, API call, errors), user workflows validated end-to-end, 80% code coverage achieved. After completing, set this task to in-progress [-] in tasks.md, then run mcp__spec-workflow__log-implementation tool to record artifacts (test file, integration workflows, coverage), then mark task complete [x] in tasks.md_

## Phase 7: Backend API Updates

- [x] 27. Update device API to remove scope
  - File: keyrx_daemon/src/web/handlers/devices.rs
  - Remove scope field from PATCH endpoint
  - Update DeviceEntry struct
  - Migrate device registry
  - Purpose: Align backend with Rhai-driven scope
  - _Leverage: keyrx_daemon/src/web/handlers/devices.rs_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Completed: 2026-01-11_
  - _Artifacts: Removed DeviceScope enum from public API (now private legacy field), removed set_scope method from DeviceRegistry, removed scope from DeviceInfo/DeviceRpcInfo, removed set_scope_device RPC handler, removed SetScope CLI command, removed scope from REST API endpoints, added DeviceEntry::new() constructor, updated all tests and benchmarks, backward compatibility maintained (old registry files deserialize correctly with scope field ignored)_

- [x] 28. Add global layout settings API
  - File: keyrx_daemon/src/web/handlers/settings.rs, keyrx_daemon/src/services/settings_service.rs
  - Create RPC endpoints for global layout settings
  - Store in daemon configuration (settings.json)
  - Purpose: Support global layout from UI
  - _Leverage: keyrx_daemon/src/web/handlers/mod.rs_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Completed: 2026-01-11_
  - _Artifacts: SettingsService (persistent storage in settings.json), RPC handlers (get_global_layout, set_global_layout), integrated into AppState, 11 unit tests passing, layout validation (ANSI_104, ISO_105, JIS_109, HHKB, NUMPAD), atomic file writes, backward compatible_

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
