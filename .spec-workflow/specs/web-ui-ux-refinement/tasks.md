# Tasks Document

## Phase 0: Test Infrastructure (BLOCKING - Must Complete First)

### Phase 0.1: Backend API Integration Tests

- [x] 0.1.1. Create test infrastructure helpers
  - Files: `keyrx_daemon/tests/integration/helpers.rs`
  - Create `TestApp` struct with isolated config directory using `tempfile`
  - Implement helper methods for HTTP requests (GET, POST, PATCH, DELETE)
  - _Leverage: `axum-test` crate, `tempfile` crate_
  - _Requirements: 0.A (Backend API Contract Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer specializing in integration testing and async Rust | Task: Create comprehensive test infrastructure helpers in keyrx_daemon/tests/integration/helpers.rs, implementing TestApp struct with isolated config directory and HTTP request helpers | Restrictions: Must use tempfile for test isolation, do not share state between tests, ensure tests can run in parallel | _Leverage: axum-test crate for HTTP testing, tempfile crate for isolated directories | _Requirements: Requirement 0.A (Backend API Contract Testing) | Success: TestApp provides clean test fixtures, HTTP helpers work correctly, tests are isolated and can run in parallel | Instructions: 1. Edit tasks.md to mark this task [-] as in-progress. 2. Implement the code. 3. Use log-implementation tool to record: taskId="0.1.1", summary (1-2 sentences), filesModified, filesCreated, statistics (linesAdded, linesRemoved), and REQUIRED artifacts field with apiEndpoints, components, functions, classes, and integrations as described in the log-implementation tool documentation. 4. Edit tasks.md to mark this task [x] as completed._

- [x] 0.1.2. Implement device persistence integration tests
  - Files: `keyrx_daemon/tests/api_devices_test.rs`
  - Test: Device layout save persists to `~/.config/keyrx/devices/{serial}.json`
  - Test: Device scope save persists to filesystem
  - Test: Device config loads correctly on daemon restart
  - _Leverage: `helpers::TestApp`, `serde_json`_
  - _Requirements: 0.C (Device Persistence Integration Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Integration Test Developer with expertise in filesystem I/O testing | Task: Implement comprehensive device persistence integration tests in keyrx_daemon/tests/integration/api_devices_test.rs, verifying layout and scope save to filesystem and load correctly | Restrictions: Must verify actual filesystem writes, do not mock filesystem operations, ensure cleanup after tests | _Leverage: helpers::TestApp for test fixtures, serde_json for config validation | _Requirements: Requirement 0.C (Device Persistence Integration Testing) | Success: Tests verify data persists to correct file paths, config loads correctly, filesystem cleanup works | Instructions: 1. Mark task [-] in tasks.md. 2. Implement tests. 3. Log with log-implementation tool (taskId="0.1.2", artifacts with functions tested). 4. Mark task [x] in tasks.md._

- [x] 0.1.3. Implement profile validation integration tests
  - Files: `keyrx_daemon/tests/api_profiles_test.rs`
  - Test: Valid profile template compiles successfully
  - Test: Invalid profile template returns compilation errors
  - Test: Profile validation endpoint returns structured errors
  - _Leverage: `helpers::TestApp`, `keyrx_compiler`_
  - _Requirements: 0.B (Profile Template Validation Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Integration Test Developer with keyrx compiler knowledge | Task: Implement profile validation integration tests in keyrx_daemon/tests/integration/api_profiles_test.rs, testing template compilation and error reporting | Restrictions: Must test both valid and invalid templates, verify error format matches API contract, ensure compiler integration works | _Leverage: helpers::TestApp, keyrx_compiler for compilation | _Requirements: Requirement 0.B (Profile Template Validation Testing) | Success: Tests verify template compilation works, errors are structured correctly, API contract is validated | Instructions: 1. Mark [-]. 2. Implement. 3. Log (taskId="0.1.3"). 4. Mark [x]._

### Phase 0.2: Profile Template Creation and Validation

- [x] 0.2.1. Create valid profile templates
  - Files: `keyrx_daemon/templates/blank.rhai`, `keyrx_daemon/templates/simple_remap.rhai`, `keyrx_daemon/templates/capslock_escape.rhai`, `keyrx_daemon/templates/vim_navigation.rhai`, `keyrx_daemon/templates/gaming.rhai`
  - Implement blank template with minimal valid syntax
  - Implement simple_remap with A→B examples
  - Implement capslock_escape with CapsLock→Escape mapping
  - Implement vim_navigation with HJKL layer
  - Implement gaming template optimized for gaming
  - _Leverage: `examples/01-simple-remap.rhai`, `examples/02-capslock-escape.rhai`, `examples/03-vim-navigation.rhai`_
  - _Requirements: 0.B (Profile Template Validation Testing), Requirement 4 (Validate Profiles Before Activation)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rhai DSL Developer with keyrx configuration expertise | Task: Create 5 valid profile templates using correct device_start/device_end syntax, ensuring all templates compile successfully | Restrictions: MUST use device_start/device_end syntax (NOT layer() function), templates must compile with keyrx_compiler, follow examples from examples/ directory | _Leverage: examples/01-simple-remap.rhai for syntax patterns, examples/02-capslock-escape.rhai for CapsLock remapping | _Requirements: Requirement 0.B (Profile Template Validation Testing), Requirement 4 (Validate Profiles Before Activation) | Success: All 5 templates use correct syntax, all compile successfully, templates provide useful starting points for users | Instructions: 1. Mark [-]. 2. Create templates. 3. Log (taskId="0.2.1", filesCreated=[...templates]). 4. Mark [x]._

- [x] 0.2.2. Implement template compilation validation tests
  - Files: `keyrx_daemon/tests/integration/template_validation_test.rs`
  - Test: All templates in keyrx_daemon/templates/ compile successfully
  - Test: Invalid template with layer() function is rejected
  - Test: Compilation errors have line numbers and helpful messages
  - _Leverage: `keyrx_compiler`, template files from 0.2.1_
  - _Requirements: 0.B (Profile Template Validation Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer with compiler validation expertise | Task: Implement template compilation tests in keyrx_daemon/tests/integration/template_validation_test.rs, verifying all templates compile and errors are helpful | Restrictions: Must test ALL templates from templates/ directory, verify error messages are user-friendly, ensure line numbers are accurate | _Leverage: keyrx_compiler for compilation, template files from task 0.2.1 | _Requirements: Requirement 0.B (Profile Template Validation Testing) | Success: All template files compile successfully in tests, invalid syntax is caught with clear errors, CI/CD catches broken templates before merge | Instructions: 1. Mark [-]. 2. Implement tests. 3. Log (taskId="0.2.2"). 4. Mark [x]._

### Phase 0.3: WASM FFI Boundary Tests

- [x] 0.3.1. Implement WASM validation FFI tests
  - Files: `keyrx_core/tests/wasm_ffi/validation_test.rs`
  - Test: validate_config() accepts valid Rhai syntax
  - Test: validate_config() rejects invalid syntax with structured errors
  - Test: Error structure matches TypeScript ValidationError interface
  - _Leverage: `wasm-bindgen-test`, `keyrx_core` validation logic_
  - _Requirements: 0.D (WASM FFI Boundary Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust WASM Developer with wasm-bindgen expertise | Task: Implement WASM FFI validation tests in keyrx_core/tests/wasm_ffi/validation_test.rs, ensuring validate_config function works correctly across FFI boundary | Restrictions: Must use wasm-bindgen-test for browser testing, verify error format matches TypeScript expectations, ensure no undefined values cross boundary | _Leverage: wasm-bindgen-test for WASM testing, keyrx_core validation logic | _Requirements: Requirement 0.D (WASM FFI Boundary Testing) | Success: WASM tests run in browser, validation works across FFI, error structure is correct | Instructions: 1. Mark [-]. 2. Implement WASM tests. 3. Log (taskId="0.3.1"). 4. Mark [x]._

- [x] 0.3.2. Implement TypeScript type verification for WASM
  - Files: `keyrx_ui/src/wasm/__tests__/ffi-types.test.ts`
  - Test: validate_config TypeScript signature matches WASM implementation
  - Test: ValidationResult interface matches WASM return type
  - Test: Runtime validation matches compile-time types
  - _Leverage: `vitest`, WASM bindings, TypeScript strict mode_
  - _Requirements: 0.D (WASM FFI Boundary Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer with WASM FFI expertise | Task: Implement TypeScript type verification tests in keyrx_ui/src/wasm/__tests__/ffi-types.test.ts, ensuring WASM function signatures match TypeScript definitions | Restrictions: Must use TypeScript strict mode, verify both compile-time and runtime types, ensure no `any` types used | _Leverage: vitest for testing, WASM bindings from @/wasm/pkg, TypeScript compiler | _Requirements: Requirement 0.D (WASM FFI Boundary Testing) | Success: TypeScript compilation fails if WASM signature changes, runtime tests verify type correctness, no type errors at FFI boundary | Instructions: 1. Mark [-]. 2. Implement type tests. 3. Log (taskId="0.3.2"). 4. Mark [x]._

### Phase 0.4: WebSocket Contract Tests

- [x] 0.4.1. Define Zod schemas for WebSocket messages
  - Files: `keyrx_ui/tests/contract/schemas.ts`
  - Define DeviceConnectedEventSchema
  - Define ProfileActivatedEventSchema
  - Define DaemonStateEventSchema
  - Export TypeScript types inferred from schemas
  - _Leverage: `zod` library_
  - _Requirements: 0.E (WebSocket Contract Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer specializing in schema validation and runtime type checking | Task: Define comprehensive Zod schemas for WebSocket messages in keyrx_ui/tests/contract/schemas.ts, covering all event types | Restrictions: Must define schemas for ALL WebSocket message types, export inferred TypeScript types, ensure schemas are strict and validate all required fields | _Leverage: zod library for schema definition | _Requirements: Requirement 0.E (WebSocket Contract Testing) | Success: Schemas cover all WebSocket events, TypeScript types are correctly inferred, schemas are strict and comprehensive | Instructions: 1. Mark [-]. 2. Define schemas. 3. Log (taskId="0.4.1", artifacts with classes/functions). 4. Mark [x]._

- [x] 0.4.2. Implement WebSocket contract tests
  - Files: `keyrx_ui/tests/contract/websocket.test.ts`
  - Test: device_connected message validates against schema
  - Test: profile_activated message validates against schema
  - Test: daemon_state message validates against schema
  - Test: Invalid message format caught by schema validation
  - _Leverage: `vitest`, `mock-socket`, schemas from 0.4.1_
  - _Requirements: 0.E (WebSocket Contract Testing)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer with WebSocket and contract testing expertise | Task: Implement WebSocket contract tests in keyrx_ui/tests/contract/websocket.test.ts, verifying all messages match Zod schemas | Restrictions: Must test ALL message types, verify both valid and invalid formats, use mock-socket for testing | _Leverage: vitest for testing, mock-socket for WebSocket mocking, schemas from task 0.4.1 | _Requirements: Requirement 0.E (WebSocket Contract Testing) | Success: Contract tests catch format changes, all message types validated, invalid formats rejected with clear errors | Instructions: 1. Mark [-]. 2. Implement tests. 3. Log (taskId="0.4.2"). 4. Mark [x]._

### Phase 0.5: E2E User Flow Tests

- [x] 0.5.1. Implement profile creation E2E test
  - Files: `keyrx_ui/e2e/profile-flow.spec.ts`
  - Test: Create profile → Verify template syntax → Activate → Verify [Active] badge
  - Test: Create profile with invalid syntax → Verify warning badge → [Activate] disabled
  - Test: Edit profile → Change code → Save → Verify validation
  - _Leverage: Playwright (already configured), existing E2E test utilities_
  - _Requirements: 0.F (End-to-End User Flow Testing), Requirement 4 (Validate Profiles Before Activation)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer with Playwright expertise | Task: Implement profile creation E2E test in keyrx_ui/e2e/profile-flow.spec.ts, testing complete user flow from creation to activation | Restrictions: Must test full user workflow, verify UI feedback at each step, ensure tests are reliable and not flaky | _Leverage: Playwright for E2E testing, existing test configuration | _Requirements: Requirement 0.F (End-to-End User Flow Testing), Requirement 4 (Validate Profiles Before Activation) | Success: E2E test covers complete profile creation flow, validation errors are caught, [Active] badge persists correctly | Instructions: 1. Mark [-]. 2. Implement E2E test. 3. Log (taskId="0.5.1"). 4. Mark [x]._

- [ ] 0.5.2. Implement device configuration E2E test
  - Files: `keyrx_ui/e2e/device-flow.spec.ts`
  - Test: Select device layout → Navigate away → Return → Verify persistence
  - Test: Select device scope → Verify save feedback → Verify persistence
  - Test: Multiple devices → Configure each → Verify independent configs
  - _Leverage: Playwright, existing E2E utilities_
  - _Requirements: 0.F (End-to-End User Flow Testing), Requirement 3 (Persist DevicesPage Layout and Scope)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer with E2E testing expertise | Task: Implement device configuration E2E test in keyrx_ui/e2e/device-flow.spec.ts, verifying layout and scope persistence | Restrictions: Must test actual persistence across navigation, verify visual feedback, test multiple devices independently | _Leverage: Playwright for testing, existing E2E setup | _Requirements: Requirement 0.F (End-to-End User Flow Testing), Requirement 3 (Persist DevicesPage Layout and Scope) | Success: Device configuration persists correctly, visual feedback works, multiple devices have independent configs | Instructions: 1. Mark [-]. 2. Implement E2E test. 3. Log (taskId="0.5.2"). 4. Mark [x]._

## Phase 1: Feature Implementation (AFTER Phase 0 Complete)

### Requirement 1: Remove WASM from ConfigPage

- [ ] 1.1. Remove useWasm() hook from ConfigPage
  - Files: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Remove `const { validateConfig, isWasmReady } = useWasm();` (line 45)
  - Remove WASM warning display (lines 379-385)
  - Remove validation using WASM (lines 132-135)
  - _Leverage: Existing ConfigPage structure_
  - _Requirements: Requirement 1 (Remove WASM from Configuration Editor)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with component refactoring expertise | Task: Remove all WASM-related code from ConfigPage.tsx (line 45, 132-135, 379-385), eliminating WASM validation in configuration editor | Restrictions: Do not break existing visual editor functionality, preserve drag-and-drop features, ensure no console errors | _Leverage: Existing ConfigPage structure, MonacoEditor component | _Requirements: Requirement 1 (Remove WASM from Configuration Editor) | Success: ConfigPage loads without WASM errors, visual editor still works, console shows zero WASM-related warnings | Instructions: 1. Mark [-]. 2. Remove WASM code. 3. Log (taskId="1.1", filesModified). 4. Mark [x]._

- [ ] 1.2. Create useValidateConfig hook with backend API
  - Files: `keyrx_ui/src/hooks/useValidateConfig.ts`
  - Implement hook using React Query mutation
  - Call POST `/api/profiles/validate` endpoint
  - Return validation results with structured errors
  - _Leverage: `useUnifiedApi()` hook, React Query patterns_
  - _Requirements: Requirement 1 (Remove WASM from Configuration Editor)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hook Developer with React Query expertise | Task: Create useValidateConfig hook in keyrx_ui/src/hooks/useValidateConfig.ts, implementing backend validation via REST API | Restrictions: Must use React Query for caching, handle loading and error states, follow existing hook patterns | _Leverage: useUnifiedApi() for API base URL, React Query for state management | _Requirements: Requirement 1 (Remove WASM from Configuration Editor) | Success: Hook calls backend API correctly, returns structured validation errors, integrates with existing error handling | Instructions: 1. Mark [-]. 2. Implement hook. 3. Log (taskId="1.2", filesCreated, artifacts with functions). 4. Mark [x]._

- [ ] 1.3. Integrate backend validation in ConfigPage
  - Files: `keyrx_ui/src/pages/ConfigPage.tsx`
  - Replace WASM validation with useValidateConfig() hook
  - Update validation callback to use backend API
  - Display validation errors from backend
  - _Leverage: `useValidateConfig()` from 1.2, existing error display logic_
  - _Requirements: Requirement 1 (Remove WASM from Configuration Editor)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with API integration expertise | Task: Integrate backend validation in ConfigPage.tsx using useValidateConfig hook, replacing WASM validation | Restrictions: Must maintain existing user experience, show validation errors inline, preserve auto-save functionality | _Leverage: useValidateConfig() hook from task 1.2, existing error display components | _Requirements: Requirement 1 (Remove WASM from Configuration Editor) | Success: Backend validation works seamlessly, validation errors display correctly, no WASM code remains | Instructions: 1. Mark [-]. 2. Integrate validation. 3. Log (taskId="1.3", filesModified, artifacts with integrations). 4. Mark [x]._

### Requirement 3: Persist DevicesPage Layout and Scope

- [ ] 3.1. Implement backend device configuration endpoint
  - Files: `keyrx_daemon/src/web/api/devices.rs`
  - Implement `PATCH /api/devices/:serial` endpoint
  - Accept `{ layout: Option<String>, scope: Option<Scope> }` request body
  - Persist to `~/.config/keyrx/devices/{serial}.json`
  - _Leverage: Existing REST API patterns, `serde_json`, `dirs` crate_
  - _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Backend Developer with Axum web framework expertise | Task: Implement PATCH /api/devices/:serial endpoint in keyrx_daemon/src/web/api/devices.rs, persisting device configuration to filesystem | Restrictions: Must persist to correct directory (~/.config/keyrx/devices/), handle filesystem errors gracefully, validate serial parameter | _Leverage: Existing REST API patterns, serde_json for config serialization, dirs crate for config directory | _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection) | Success: Endpoint accepts layout and scope, data persists to filesystem, config loads correctly on daemon restart | Instructions: 1. Mark [-]. 2. Implement endpoint. 3. Log (taskId="3.1", filesModified/Created, artifacts with apiEndpoints). 4. Mark [x]._

- [ ] 3.2. Create DeviceConfig model
  - Files: `keyrx_daemon/src/config/device.rs`
  - Define DeviceConfig struct with serial, layout, scope fields
  - Implement Default trait
  - Implement Serialize/Deserialize with serde
  - _Leverage: `serde`, existing config models_
  - _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Data Model Developer with serde expertise | Task: Create DeviceConfig model in keyrx_daemon/src/config/device.rs with proper serialization | Restrictions: Must implement proper Default, follow existing config patterns, use kebab-case for JSON serialization | _Leverage: serde for serialization, existing config models as patterns | _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection) | Success: DeviceConfig serializes/deserializes correctly, Default implementation works, follows project patterns | Instructions: 1. Mark [-]. 2. Implement model. 3. Log (taskId="3.2", filesCreated, artifacts with classes). 4. Mark [x]._

- [ ] 3.3. Create useUpdateDevice hook
  - Files: `keyrx_ui/src/hooks/useUpdateDevice.ts`
  - Implement mutation hook calling PATCH /api/devices/:serial
  - Invalidate devices query on success
  - Handle loading and error states
  - _Leverage: React Query, `useUnifiedApi()`, existing mutation patterns_
  - _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hook Developer with mutation expertise | Task: Create useUpdateDevice hook in keyrx_ui/src/hooks/useUpdateDevice.ts for device configuration updates | Restrictions: Must invalidate queries on success, handle errors gracefully, follow React Query patterns | _Leverage: React Query for mutations, useUnifiedApi() for base URL, existing mutation hooks as patterns | _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection) | Success: Hook calls API correctly, query invalidation works, optimistic updates optional but beneficial | Instructions: 1. Mark [-]. 2. Implement hook. 3. Log (taskId="3.3", filesCreated, artifacts with functions). 4. Mark [x]._

- [ ] 3.4. Integrate device persistence in DevicesPage
  - Files: `keyrx_ui/src/pages/DevicesPage.tsx`
  - Update layout change handler to use useUpdateDevice()
  - Update scope change handler to use useUpdateDevice()
  - Show save feedback (✓ Saved / ✗ Error)
  - Invalidate cache to show updated values
  - _Leverage: `useUpdateDevice()` from 3.3, existing `useAutoSave()` patterns_
  - _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with state management expertise | Task: Integrate device persistence in DevicesPage.tsx using useUpdateDevice hook, showing save feedback | Restrictions: Must show visual feedback for save states, handle errors gracefully, maintain auto-save behavior | _Leverage: useUpdateDevice() hook from task 3.3, existing useAutoSave() patterns for feedback | _Requirements: Requirement 3 (Persist DevicesPage Layout and Scope Selection) | Success: Layout and scope changes persist, user sees save feedback, values display correctly after navigation | Instructions: 1. Mark [-]. 2. Integrate persistence. 3. Log (taskId="3.4", filesModified, artifacts with integrations). 4. Mark [x]._

### Requirement 4: Validate Profiles Before Activation

- [ ] 4.1. Implement profile validation endpoint
  - Files: `keyrx_daemon/src/web/api/profiles.rs`
  - Implement `POST /api/profiles/:name/validate` endpoint
  - Compile Rhai source using keyrx_compiler
  - Return `{ valid: bool, errors: ValidationError[] }`
  - _Leverage: `keyrx_compiler`, existing profile endpoints_
  - _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust API Developer with compiler integration expertise | Task: Implement profile validation endpoint POST /api/profiles/:name/validate in keyrx_daemon/src/web/api/profiles.rs | Restrictions: Must use keyrx_compiler for validation, return structured errors with line numbers, handle compilation failures gracefully | _Leverage: keyrx_compiler for Rhai compilation, existing profile API patterns | _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template) | Success: Endpoint validates profiles correctly, errors include line numbers, invalid syntax is caught before activation | Instructions: 1. Mark [-]. 2. Implement endpoint. 3. Log (taskId="4.1", filesModified, artifacts with apiEndpoints). 4. Mark [x]._

- [ ] 4.2. Update profile creation to use valid templates
  - Files: `keyrx_daemon/src/web/api/profiles.rs` (modify create_profile function)
  - Load template from keyrx_daemon/templates/*.rhai
  - Verify template compiles before creating profile
  - Support template selection: blank, simple, capslock_escape, vim_navigation, gaming
  - _Leverage: Templates from 0.2.1, `include_str!` macro_
  - _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Backend Developer with template system expertise | Task: Update create_profile function to use valid templates from keyrx_daemon/templates/, verifying compilation before creating profile | Restrictions: Must verify template compiles before use, support all 5 template types, use include_str! for template loading | _Leverage: Template files from task 0.2.1, include_str! macro for compile-time inclusion | _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template) | Success: Profile creation uses valid templates, templates compile successfully, users never get invalid syntax | Instructions: 1. Mark [-]. 2. Update creation logic. 3. Log (taskId="4.2", filesModified, artifacts with functions). 4. Mark [x]._

- [ ] 4.3. Create useProfileValidation hook
  - Files: `keyrx_ui/src/hooks/useProfileValidation.ts`
  - Implement query hook calling GET /api/profiles/:name/validation
  - Cache validation results for 1 minute
  - Return `{ valid: boolean, errors: ValidationError[] }`
  - _Leverage: React Query, `useUnifiedApi()`_
  - _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hook Developer with caching expertise | Task: Create useProfileValidation hook in keyrx_ui/src/hooks/useProfileValidation.ts for profile validation status | Restrictions: Must cache results to avoid excessive validation requests, handle loading states, follow React Query patterns | _Leverage: React Query for caching, useUnifiedApi() for API base URL | _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template) | Success: Hook fetches validation status, caches results efficiently, integrates with ProfileCard component | Instructions: 1. Mark [-]. 2. Implement hook. 3. Log (taskId="4.3", filesCreated, artifacts with functions). 4. Mark [x]._

- [ ] 4.4. Add validation badge to ProfileCard
  - Files: `keyrx_ui/src/components/ProfileCard.tsx`
  - Use useProfileValidation() hook to fetch validation status
  - Display "✓ Valid" badge for valid profiles
  - Display "⚠️ Invalid Configuration" badge for invalid profiles
  - Show tooltip with first error message on hover
  - Disable [Activate] button if profile is invalid
  - _Leverage: `useProfileValidation()` from 4.3, existing Badge component, Tooltip component_
  - _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer with accessibility expertise | Task: Add validation badge to ProfileCard.tsx using useProfileValidation hook, with tooltip and disabled state | Restrictions: Must show clear visual feedback, tooltip must be accessible, [Activate] button must be disabled for invalid profiles | _Leverage: useProfileValidation() hook from task 4.3, existing Badge and Tooltip components | _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template) | Success: Validation status displays clearly, tooltip shows error details, invalid profiles cannot be activated | Instructions: 1. Mark [-]. 2. Add validation UI. 3. Log (taskId="4.4", filesModified, artifacts with components). 4. Mark [x]._

- [ ] 4.5. Add template selector to profile creation modal
  - Files: `keyrx_ui/src/pages/ProfilesPage.tsx`
  - Add template dropdown to create profile modal
  - Options: Blank, Simple Remap, CapsLock→Escape, Vim Navigation, Gaming
  - Pass selected template to backend on creation
  - _Leverage: Existing Modal component, Dropdown component_
  - _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template)_
  - _Prompt: Implement the task for spec web-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer with form handling expertise | Task: Add template selector dropdown to profile creation modal in ProfilesPage.tsx | Restrictions: Must offer all 5 template options, default to 'blank', pass template selection to backend API | _Leverage: Existing Modal and Dropdown components | _Requirements: Requirement 4 (Validate Profiles Before Activation and Fix Template) | Success: Template selector displays all options, selection passes to backend, users can choose starting template | Instructions: 1. Mark [-]. 2. Add template selector. 3. Log (taskId="4.5", filesModified, artifacts with components). 4. Mark [x]._

## Phase 2: Optional Enhancements (After Phase 0 and 1 Complete)

### Requirement 5: Redesign ConfigPage as QMK-Style Editor (Optional - Large Task)

**Note**: This is a substantial redesign and can be deferred to a separate spec. Marking as optional for this iteration.

- [ ] 5.1. Design QMK-style key mapping interface
  - Research QMK Configurator UI patterns
  - Design drag-and-drop key palette (VK_, MD_, LK_)
  - Design key mapping dialog (Simple, Tap-Hold, Macro, Layer Switch)
  - Create mockups for review
  - _Leverage: Existing KeyboardVisualizer, KeyAssignmentPanel components_
  - _Requirements: Requirement 5 (Redesign ConfigPage as QMK-Style Profile Editor)_
  - _Prompt: [OPTIONAL - DEFER] Role: UI/UX Designer with QMK Configurator knowledge | Task: Design QMK-style key mapping interface for ConfigPage redesign | Restrictions: Must maintain accessibility, follow 2025 UI/UX trends, ensure keyboard navigation | _Leverage: Existing KeyboardVisualizer and KeyAssignmentPanel | _Requirements: Requirement 5 | Success: Design provides clear visual feedback, drag-and-drop is intuitive, supports all mapping types | Instructions: 1. Mark [-]. 2. Create design. 3. Log (taskId="5.1"). 4. Mark [x]._

### Requirement 6: Move ConfigPage to Profile Sub-Route (Optional)

**Note**: This is optional and can be deferred. Current query parameter approach works fine.

- [ ] 6.1. Implement nested routing for /profiles/:name/config
  - Files: `keyrx_ui/src/App.tsx` (routing configuration)
  - Add nested route: `/profiles/:name/config`
  - Update ProfileCard "Edit" button to navigate to nested route
  - Update breadcrumb navigation
  - _Leverage: React Router, existing route configuration_
  - _Requirements: Requirement 6 (Move ConfigPage to Profile Sub-Route)_
  - _Prompt: [OPTIONAL - DEFER] Role: React Router expert with navigation design expertise | Task: Implement nested routing for profile configuration | Restrictions: Must maintain backward compatibility with query parameter approach, update all navigation links | _Leverage: React Router, existing routing configuration | _Requirements: Requirement 6 | Success: Nested routes work correctly, breadcrumb navigation is clear, no broken links | Instructions: 1. Mark [-]. 2. Implement routing. 3. Log (taskId="6.1"). 4. Mark [x]._

## Summary

**Total Tasks**: 25 (18 required + 2 optional)

**Phase 0 (Test Infrastructure)**: Tasks 0.1.1 - 0.5.2 (10 tasks) - **BLOCKING**
**Phase 1 (Feature Implementation)**: Tasks 1.1 - 4.5 (15 tasks) - **After Phase 0**
**Phase 2 (Optional Enhancements)**: Tasks 5.1, 6.1 (2 tasks) - **Defer to future spec**

**Implementation Order**:
1. ✅ Complete all Phase 0 tasks first (test infrastructure)
2. ✅ Verify all tests pass in CI/CD
3. ✅ Then proceed with Phase 1 tasks (features)
4. ⏸️ Defer Phase 2 (optional) to future iteration

**Estimated Duration**:
- Phase 0: ~8-11 days (test infrastructure)
- Phase 1: ~5-7 days (feature implementation with tests catching bugs)
- **Total**: ~15-20 days
