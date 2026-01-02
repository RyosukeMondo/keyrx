# Tasks Document

## Backend Tasks (Rust)

- [x] 1. Create ProfileManager service in keyrx_daemon/src/profile_manager.rs
  - File: keyrx_daemon/src/profile_manager.rs
  - Implement profile lifecycle management (create, activate, delete, get/set config)
  - Add compilation via keyrx_compiler crate
  - Add daemon reload signaling
  - Purpose: Central service for profile-config-daemon integration
  - _Leverage: keyrx_compiler/src/lib.rs, keyrx_daemon/src/daemon/mod.rs_
  - _Requirements: R4 (Profile-to-Configuration File Mapping)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Rust Developer with expertise in async I/O and system integration | Task: Create ProfileManager service implementing profile lifecycle (create → compile → activate → delete) with integration to keyrx_compiler for .rhai → .krx compilation and daemon reload signaling, following requirement R4. Use existing compiler from keyrx_compiler crate and daemon control patterns from keyrx_daemon/src/daemon/mod.rs | Restrictions: Must handle all file I/O errors gracefully, ensure atomic operations (rollback on failure), do not block async runtime, follow Rust error handling best practices (Result types), maintain SSOT principle | Success: ProfileManager can create .rhai files from templates, compile to .krx successfully, activate profiles with daemon reload, delete profiles atomically, all operations tested with unit tests covering success and failure scenarios | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (classes, methods, integrations), and mark as complete [x] when done_

- [ ] 2. Extend profiles API in keyrx_daemon/src/web/api/profiles.rs
  - File: keyrx_daemon/src/web/api/profiles.rs
  - Add endpoints: GET /api/profiles/:name/config, PUT /api/profiles/:name/config
  - Add endpoint: POST /api/profiles/:name/activate (calls ProfileManager)
  - Add endpoint: GET /api/profiles/active (returns current active profile)
  - Purpose: Expose profile-config operations to frontend via RPC
  - _Leverage: keyrx_daemon/src/web/api/mod.rs, keyrx_daemon/src/profile_manager.rs_
  - _Requirements: R4 (Profile-to-Configuration File Mapping), R3 (Persistent Profile Activation)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: API Developer with expertise in Rust axum framework and RESTful design | Task: Extend profiles.rs API with config CRUD endpoints (GET/PUT /api/profiles/:name/config), activation endpoint (POST /api/profiles/:name/activate), and active profile query (GET /api/profiles/active) following requirements R3 and R4. Integrate with ProfileManager service from task 1, use existing API patterns from keyrx_daemon/src/web/api/mod.rs | Restrictions: Must validate all inputs (profile names, config content), return proper HTTP status codes (200, 400, 404, 500), use structured JSON errors, do not expose internal paths in responses, ensure CORS compatibility | Success: All endpoints return correct responses, profile activation triggers compilation and daemon reload, config endpoints read/write .rhai files correctly, errors are handled gracefully with descriptive messages, API integration tests pass | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (apiEndpoints with method/path/purpose/formats/location), and mark as complete [x] when done_

- [ ] 3. Add device layout persistence in keyrx_daemon/src/web/api/devices.rs
  - File: keyrx_daemon/src/web/api/devices.rs
  - Add endpoint: PUT /api/devices/:serial/layout (save layout preference)
  - Add endpoint: GET /api/devices/:serial/layout (retrieve saved layout)
  - Store preferences in ~/.config/keyrx/device_layouts.json
  - Purpose: Persist device layout selections across sessions
  - _Leverage: keyrx_daemon/src/web/api/mod.rs, serde_json for file I/O_
  - _Requirements: R2 (Auto-Save Device Layout Selection)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Developer with expertise in Rust file I/O and API design | Task: Add device layout persistence endpoints (PUT/GET /api/devices/:serial/layout) that store preferences in ~/.config/keyrx/device_layouts.json using serde_json, following requirement R2. Use existing API patterns from keyrx_daemon/src/web/api/mod.rs | Restrictions: Must validate serial numbers (alphanumeric, max 64 chars), validate layout enum values, handle file corruption gracefully (recreate if invalid), use atomic file writes (write to temp, rename), do not block on file I/O | Success: Layout preferences persist across daemon restarts, concurrent writes are safe (no data loss), invalid JSON is handled gracefully, API returns 404 for unknown devices, unit tests cover save/load/corruption scenarios | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (apiEndpoints, functions for file I/O), and mark as complete [x] when done_

- [ ] 4. Add daemon state active_profile field in keyrx_daemon/src/daemon/mod.rs
  - File: keyrx_daemon/src/daemon/mod.rs
  - Extend DaemonState struct with active_profile: Option<String>
  - Update WebSocket daemon-state event to include active profile
  - Read active profile from ~/.config/keyrx/active_profile.txt on startup
  - Purpose: Expose active profile to all WebSocket subscribers
  - _Leverage: keyrx_daemon/src/web/ws.rs, existing DaemonState struct_
  - _Requirements: R7 (Display Active Profile in Metrics Page), R3 (Persistent Profile Activation)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Programmer with expertise in Rust state management and WebSocket protocols | Task: Extend DaemonState struct with active_profile field, update WebSocket daemon-state events to include active profile name, load active profile from ~/.config/keyrx/active_profile.txt on daemon startup following requirements R3 and R7. Integrate with existing WebSocket handler in keyrx_daemon/src/web/ws.rs | Restrictions: Must not break existing WebSocket clients (backward compatible JSON), handle missing active_profile.txt gracefully (None value), ensure thread-safe state access, do not add latency to state updates | Success: All WebSocket clients receive active profile in daemon-state events, profile persists across daemon restarts, state updates are thread-safe, WebSocket message format is backward compatible | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (classes, integrations for WebSocket), and mark as complete [x] when done_

## Frontend Shared Utilities

- [ ] 5. Create useAutoSave hook in keyrx_ui/src/hooks/useAutoSave.ts
  - File: keyrx_ui/src/hooks/useAutoSave.ts
  - Implement generic debounced auto-save hook with loading/error states
  - Add retry logic for failed saves (3 attempts with exponential backoff)
  - Track lastSavedAt timestamp for UI feedback
  - Purpose: Reusable auto-save pattern across all pages
  - _Leverage: lodash.debounce or custom debounce utility_
  - _Requirements: R2 (Auto-Save Device Layout Selection), NFR Performance (Auto-Save Debouncing)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer with expertise in custom hooks and state management | Task: Create generic useAutoSave hook with debouncing (500ms default), retry logic (3 attempts, exponential backoff), loading/error states, and lastSavedAt tracking following requirement R2 and NFR Performance. Use debounce utility (lodash or custom) | Restrictions: Must clean up timers on unmount, do not retry on validation errors (4xx), cancel pending saves when component unmounts, use TypeScript generics for type safety, follow React hooks best practices | Success: Hook debounces multiple rapid calls into single save, retries failed saves automatically, exposes clear loading/error states, cancels pending operations on unmount, TypeScript types are fully inferred, unit tests cover debouncing, retry, and cleanup | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (functions with signatures and purpose), and mark as complete [x] when done_

- [ ] 6. Extend RpcClient with new methods in keyrx_ui/src/api/rpc.ts
  - File: keyrx_ui/src/api/rpc.ts
  - Add method: setDeviceLayout(serial: string, layout: string): Promise<void>
  - Add method: getProfileConfig(name: string): Promise<ProfileConfig>
  - Add method: setProfileConfig(name: string, source: string): Promise<void>
  - Add method: getActiveProfile(): Promise<string | null>
  - Purpose: Type-safe RPC methods for new backend endpoints
  - _Leverage: existing RpcClient class structure, useUnifiedApi hook_
  - _Requirements: R2, R4_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer with expertise in RPC protocols and type safety | Task: Extend RpcClient class with new methods for device layout persistence and profile-config operations following requirements R2 and R4. Use existing RpcClient patterns from src/api/rpc.ts and useUnifiedApi hook integration | Restrictions: Must follow existing RPC method naming conventions, maintain type safety (no 'any' types), use proper error handling (throw typed errors), do not add dependencies, ensure JSON-RPC 2.0 compliance | Success: All new methods are type-safe, RPC requests are properly formatted, responses are correctly parsed, errors are thrown with descriptive messages, methods integrate seamlessly with existing RpcClient usage | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (functions with full signatures and purpose, apiEndpoints called), and mark as complete [x] when done_

- [ ] 7. Create useProfileConfig hook in keyrx_ui/src/hooks/useProfileConfig.ts
  - File: keyrx_ui/src/hooks/useProfileConfig.ts
  - Implement React Query hook for profile config CRUD
  - Add useGetProfileConfig(name: string) query hook
  - Add useSetProfileConfig() mutation hook with optimistic updates
  - Purpose: Centralized profile config state management
  - _Leverage: src/lib/queryClient.ts, src/api/rpc.ts_
  - _Requirements: R4 (Profile-to-Configuration File Mapping)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in React Query and state management | Task: Create useProfileConfig hook with query (useGetProfileConfig) and mutation (useSetProfileConfig) using React Query, implementing optimistic updates and cache invalidation following requirement R4. Leverage existing query client configuration from src/lib/queryClient.ts and RPC methods from src/api/rpc.ts | Restrictions: Must follow existing React Query patterns, use proper query keys (centralized in queryClient.ts), implement optimistic updates with rollback on error, do not duplicate query logic, ensure stale-while-revalidate behavior | Success: Config fetches are cached and revalidated appropriately, mutations update cache optimistically, rollback works on errors, query keys are properly namespaced, hook TypeScript types are fully inferred | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (functions, integrations with React Query and RPC), and mark as complete [x] when done_

## Frontend Page Updates

- [ ] 8. Add auto-save to DevicesPage in keyrx_ui/src/pages/DevicesPage.tsx
  - File: keyrx_ui/src/pages/DevicesPage.tsx
  - Replace layout dropdown with auto-save version using useAutoSave hook
  - Add visual feedback (saving spinner, checkmark on success, error icon on failure)
  - Load saved layout on page mount from new RPC endpoint
  - Purpose: Persist device layout selections automatically
  - _Leverage: src/hooks/useAutoSave.ts, src/api/rpc.ts_
  - _Requirements: R2 (Auto-Save Device Layout Selection)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend React Developer with expertise in form handling and UX | Task: Extend DevicesPage to auto-save layout selections using useAutoSave hook, add visual feedback indicators (spinner/checkmark/error), and load saved layouts on mount following requirement R2. Leverage useAutoSave from src/hooks/useAutoSave.ts and RPC methods from src/api/rpc.ts | Restrictions: Must not break existing device list functionality, preserve responsive design, ensure accessibility (ARIA labels for icons), do not block UI during save, show non-intrusive feedback (fade out after 2s) | Success: Layout changes save automatically after 500ms, visual feedback is clear and non-intrusive, saved layouts persist across page refreshes, error handling shows user-friendly messages, component remains accessible | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components modified, integrations with hooks and RPC), and mark as complete [x] when done_

- [ ] 9. Update ProfilesPage activation flow in keyrx_ui/src/pages/ProfilesPage.tsx
  - File: keyrx_ui/src/pages/ProfilesPage.tsx
  - Update useActivateProfile mutation to call new backend endpoint
  - Update optimistic update to persist active state correctly
  - Add error handling for compilation failures (show line/column errors)
  - Update cache invalidation to include daemonState and activeProfile queries
  - Purpose: Ensure profile activation triggers full backend flow (compile + reload)
  - _Leverage: src/hooks/useProfiles.ts, src/api/rpc.ts_
  - _Requirements: R3 (Persistent Profile Activation), R4 (Profile-to-Configuration File Mapping)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in mutation handling and error management | Task: Update ProfilesPage activation flow to call new backend activation endpoint, handle compilation errors gracefully, update cache invalidation to include daemonState/activeProfile queries following requirements R3 and R4. Extend useActivateProfile from src/hooks/useProfiles.ts and use RPC methods from src/api/rpc.ts | Restrictions: Must maintain optimistic update pattern, rollback on errors, show compilation errors with line numbers in toast, do not break existing profile CRUD, ensure [Active] badge persists correctly | Success: Profile activation calls backend endpoint correctly, compilation errors are displayed with line/column info, active badge persists across refreshes, React Query cache updates propagate to all pages, error rollback works correctly | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, functions, integrations), and mark as complete [x] when done_

- [ ] 10. Add active profile header to MetricsPage in keyrx_ui/src/pages/MetricsPage.tsx
  - File: keyrx_ui/src/pages/MetricsPage.tsx
  - Add header section showing active profile name
  - Subscribe to daemon-state WebSocket events for real-time updates
  - Show "No Active Profile" when none is active
  - Purpose: Display active profile context in metrics view
  - _Leverage: src/hooks/useMetrics.ts, WebSocket subscription pattern_
  - _Requirements: R7 (Display Active Profile in Metrics Page)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in WebSocket subscriptions and real-time UIs | Task: Add active profile header to MetricsPage that subscribes to daemon-state WebSocket events and displays current active profile name following requirement R7. Use existing WebSocket patterns from src/hooks/useMetrics.ts | Restrictions: Must not duplicate WebSocket subscriptions, update within 1 second of profile change, show loading state during initial fetch, do not break existing metrics display, maintain responsive layout | Success: Active profile name displays in header, updates in real-time via WebSocket, shows "No Active Profile" when appropriate, loading state is handled gracefully, component is responsive | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, integrations with WebSocket), and mark as complete [x] when done_

- [ ] 11. Add profile selector to SimulatorPage in keyrx_ui/src/pages/SimulatorPage.tsx
  - File: keyrx_ui/src/pages/SimulatorPage.tsx
  - Add dropdown to select profile for simulation
  - Load selected profile's compiled .krx into WASM simulator
  - Update WASM initialization to accept profile parameter
  - Purpose: Enable testing any profile in simulator
  - _Leverage: src/hooks/useWasm.ts, src/hooks/useProfiles.ts_
  - _Requirements: R6 (Move WASM to Keyboard Simulator Page)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in WASM integration and state management | Task: Add profile selector dropdown to SimulatorPage and integrate with WASM module to load selected profile's configuration following requirement R6. Use useWasm hook from src/hooks/useWasm.ts and useProfiles from src/hooks/useProfiles.ts | Restrictions: Must handle missing profiles gracefully, show loading state during WASM initialization, do not reload WASM module unnecessarily (only on profile change), maintain existing simulator functionality | Success: Profile dropdown lists all available profiles, selecting a profile loads its .krx into simulator, simulation runs with correct profile logic, loading states are handled, no unnecessary WASM reloads | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, integrations with WASM and profiles), and mark as complete [x] when done_

## Visual Configuration Editor

- [ ] 12. Create KeyAssignmentPanel component in keyrx_ui/src/components/KeyAssignmentPanel.tsx
  - File: keyrx_ui/src/components/KeyAssignmentPanel.tsx
  - Create categorized key palette (VK_*, MD_*, LK_*, Layers, Macros)
  - Implement drag source handlers using @dnd-kit/core
  - Add search/filter functionality
  - Purpose: Provide draggable key sources for visual editor
  - _Leverage: @dnd-kit/core library_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer with expertise in drag-and-drop libraries and component design | Task: Create KeyAssignmentPanel component with categorized key palette (VK_*, MD_*, LK_*, Layers, Macros), drag source handlers using @dnd-kit/core, and search filter following requirement R5 | Restrictions: Must follow existing component patterns, ensure keyboard navigation (tab through keys), use semantic HTML, maintain accessibility (ARIA labels), keep component under 300 lines | Success: Component renders categorized key list, keys are draggable, search filter works correctly, keyboard navigation is functional, component is accessible and responsive | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, props, exports), and mark as complete [x] when done_

- [ ] 13. Create DeviceScopeToggle component in keyrx_ui/src/components/DeviceScopeToggle.tsx
  - File: keyrx_ui/src/components/DeviceScopeToggle.tsx
  - Create segmented control for Global vs Device-Specific toggle
  - Add device selector dropdown (when device-specific mode)
  - Purpose: Switch between global and per-device key mapping scopes
  - _Leverage: existing UI component patterns_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer with expertise in form controls and component composition | Task: Create DeviceScopeToggle component with segmented control (Global/Device-Specific) and device selector dropdown following requirement R5. Use existing UI patterns for consistent styling | Restrictions: Must follow existing component patterns, use controlled component pattern (value/onChange props), ensure accessibility (radio group semantics), maintain responsive design, keep component under 200 lines | Success: Component renders segmented control correctly, device dropdown shows when device-specific selected, value/onChange props work correctly, component is accessible with ARIA labels, responsive design works on mobile | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, props, exports), and mark as complete [x] when done_

- [ ] 14. Create LayerSelector component in keyrx_ui/src/components/LayerSelector.tsx
  - File: keyrx_ui/src/components/LayerSelector.tsx
  - Create dropdown to select layer (base, vim, gaming, etc.)
  - Load layers from profile config
  - Purpose: Switch between keyboard layers in visual editor
  - _Leverage: src/hooks/useProfileConfig.ts_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in dropdown components and state management | Task: Create LayerSelector dropdown component that loads layers from profile config using useProfileConfig hook following requirement R5 | Restrictions: Must use controlled component pattern, handle loading state while fetching layers, show "No layers" message when profile has no layers, ensure accessibility (select element or ARIA combobox), keep component under 150 lines | Success: Component fetches layers from profile config, renders dropdown correctly, handles loading/empty states, onChange callback fires on selection, component is accessible | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, integrations with useProfileConfig), and mark as complete [x] when done_

- [ ] 15. Extend KeyboardVisualizer with drag-drop in keyrx_ui/src/components/KeyboardVisualizer.tsx
  - File: keyrx_ui/src/components/KeyboardVisualizer.tsx
  - Add drop zone handlers using @dnd-kit/core
  - Update visual feedback for drag-over states
  - Add click handler to open key assignment popup
  - Purpose: Make keyboard interactive for key assignment
  - _Leverage: existing KeyboardVisualizer component, @dnd-kit/core_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in drag-and-drop interactions and SVG manipulation | Task: Extend existing KeyboardVisualizer component with drop zone handlers, drag-over visual feedback, and click handlers for key assignment popup following requirement R5. Use @dnd-kit/core and existing component structure | Restrictions: Must not break existing visualization functionality, maintain performance (no lag during drag), use CSS for drag-over states (not inline styles), ensure keyboard accessibility (click via Enter/Space), maintain component modularity | Success: Keys accept drops from KeyAssignmentPanel, visual feedback shows on drag-over, click opens assignment popup, drag performance is smooth, keyboard navigation works, component remains under 500 lines | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components modified, functions added, integrations), and mark as complete [x] when done_

- [ ] 16. Create KeyAssignmentPopup component in keyrx_ui/src/components/KeyAssignmentPopup.tsx
  - File: keyrx_ui/src/components/KeyAssignmentPopup.tsx
  - Create modal popup for key assignment selection
  - Add tabs for Key, Modifier, Lock, Layer, Macro
  - Add tap-hold configuration UI
  - Purpose: Detailed key assignment interface (alternative to drag-drop)
  - _Leverage: existing modal patterns_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer with expertise in modal dialogs and complex forms | Task: Create KeyAssignmentPopup modal with tabbed interface (Key/Modifier/Lock/Layer/Macro) and tap-hold configuration UI following requirement R5. Use existing modal patterns for consistency | Restrictions: Must follow existing modal patterns, ensure focus trap (accessibility), close on Escape key, prevent body scroll when open, use controlled component pattern, keep component under 400 lines | Success: Modal opens/closes correctly, tabs switch smoothly, tap-hold UI is intuitive, focus management works (returns to trigger on close), keyboard navigation is functional, component is accessible | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components, props, exports), and mark as complete [x] when done_

- [ ] 17. Implement ConfigPage visual editor in keyrx_ui/src/pages/ConfigPage.tsx
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Replace existing implementation with visual editor layout
  - Integrate KeyboardVisualizer, KeyAssignmentPanel, DeviceScopeToggle, LayerSelector
  - Add auto-save for key mappings using useAutoSave hook
  - Add WASM validation before saving
  - Load profile from query parameter (?profile=name)
  - Purpose: Complete visual drag-drop configuration editor
  - _Leverage: All visual editor components from tasks 12-16, useAutoSave, useWasm, useProfileConfig_
  - _Requirements: R5 (Visual Configuration Editor with Drag-and-Drop)_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior React Developer with expertise in complex page composition and state orchestration | Task: Implement complete ConfigPage visual editor integrating KeyboardVisualizer, KeyAssignmentPanel, DeviceScopeToggle, LayerSelector with auto-save and WASM validation following requirement R5. Read profile from query param, orchestrate all child components, manage drag-drop state. Leverage all components from tasks 12-16, useAutoSave, useWasm, useProfileConfig hooks | Restrictions: Must maintain responsive layout (mobile-first), handle all error states (WASM validation failures, save errors), show loading states during profile fetch, use breadcrumb navigation (Profiles → [name] → Config), ensure accessibility throughout, keep page component under 600 lines | Success: Page loads profile from query param, all components render and interact correctly, drag-drop works end-to-end, auto-save persists changes after 500ms, WASM validation prevents invalid saves, error handling is comprehensive, page is fully responsive and accessible | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (components created, integrations with hooks/WASM/RPC, data flow description), and mark as complete [x] when done_

## Integration and Testing

- [ ] 18. Update React Query cache keys in keyrx_ui/src/lib/queryClient.ts
  - File: keyrx_ui/src/lib/queryClient.ts
  - Add new query keys: deviceLayout(serial), profileConfig(name), activeProfile
  - Update invalidation patterns for profile activation
  - Purpose: Centralize query key management for new features
  - _Leverage: existing queryKeys object_
  - _Requirements: R1, R2, R3, R4_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Developer with expertise in React Query and cache management | Task: Extend queryKeys object with new keys (deviceLayout, profileConfig, activeProfile) and update invalidation patterns for profile activation following requirements R1-R4. Extend existing queryKeys structure in src/lib/queryClient.ts | Restrictions: Must follow existing naming conventions (camelCase factory functions), ensure type safety (TypeScript), document invalidation dependencies (comments), do not duplicate keys, maintain backward compatibility | Success: New query keys are properly namespaced, invalidation patterns are documented, no key conflicts, TypeScript types are inferred correctly, cache coordination works across pages | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (functions for query keys, integrations), and mark as complete [x] when done_

- [ ] 19. Write unit tests for useAutoSave hook in keyrx_ui/tests/hooks/useAutoSave.test.tsx
  - File: keyrx_ui/tests/hooks/useAutoSave.test.tsx
  - Test debouncing behavior (multiple calls → single save)
  - Test retry logic (failed save retries with backoff)
  - Test cleanup on unmount (pending saves canceled)
  - Test success/error states
  - Purpose: Ensure useAutoSave hook reliability
  - _Leverage: @testing-library/react-hooks, vitest_
  - _Requirements: R2, NFR Reliability_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in React hook testing and Vitest | Task: Write comprehensive unit tests for useAutoSave hook covering debouncing, retry logic, cleanup, and state management following requirement R2 and NFR Reliability. Use @testing-library/react-hooks and vitest | Restrictions: Must test behavior not implementation, use fake timers (vi.useFakeTimers), mock saveFn, test all edge cases (unmount during save, rapid changes, failures), achieve >90% coverage | Success: All tests pass, debouncing is verified (single save after rapid calls), retry logic is tested (3 attempts with backoff), cleanup is verified (no memory leaks), coverage exceeds 90%, tests run reliably | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (test files, coverage metrics), and mark as complete [x] when done_

- [ ] 20. Write integration tests for ProfileManager in keyrx_daemon/tests/profile_manager_test.rs
  - File: keyrx_daemon/tests/profile_manager_test.rs
  - Test profile creation (file created with template)
  - Test activation flow (compile → persist → reload)
  - Test activation rollback on compilation error
  - Test deletion (files removed, active marker cleared)
  - Purpose: Ensure ProfileManager service reliability
  - _Leverage: tempfile crate for test directory, mock compiler_
  - _Requirements: R3, R4_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer with expertise in integration testing and async testing | Task: Write integration tests for ProfileManager covering create, activate (success and failure), delete operations following requirements R3 and R4. Use tempfile crate for isolated test directories, mock compiler and daemon control | Restrictions: Must use async test runtime (tokio::test), ensure test isolation (separate temp dirs), test both success and failure paths, clean up resources, use assert macros for clear failures | Success: All tests pass, profile creation verified, activation flow tested (compile + persist + reload), rollback tested (compilation error), deletion tested, no test pollution (isolated temp dirs), >85% code coverage | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (test files, integration scenarios covered), and mark as complete [x] when done_

- [ ] 21. Write E2E tests for visual config editor in keyrx_ui/tests/e2e/config-editor.spec.ts
  - File: keyrx_ui/tests/e2e/config-editor.spec.ts
  - Test drag-drop flow (drag key from palette to keyboard)
  - Test device scope toggle (global ↔ device-specific)
  - Test layer switching (base → vim → gaming)
  - Test auto-save (changes persist on refresh)
  - Test error handling (invalid config rejected)
  - Purpose: Validate complete visual editor user flow
  - _Leverage: Playwright or Cypress, mock backend_
  - _Requirements: R5_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer with expertise in E2E testing and Playwright/Cypress | Task: Write end-to-end tests for visual config editor covering drag-drop, scope toggle, layer switching, auto-save, and error handling following requirement R5. Use Playwright or Cypress with mock backend | Restrictions: Must test real user interactions (mouse/keyboard), use data-testid for selectors (not CSS classes), ensure tests are deterministic (no flaky tests), clean state between tests, run in headless mode for CI | Success: All E2E tests pass reliably, drag-drop flow is validated end-to-end, auto-save persistence is verified (refresh test), error handling prevents invalid saves, tests run in CI without flakiness, critical user journeys are covered | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (test files, user journeys covered), and mark as complete [x] when done_

- [ ] 22. Update documentation in docs/ui-ux-refinement.md
  - File: docs/ui-ux-refinement.md
  - Document new visual editor usage
  - Document profile-config integration workflow
  - Add screenshots of new UI components
  - Document auto-save behavior
  - Purpose: User-facing documentation for new features
  - _Leverage: existing docs structure_
  - _Requirements: All_
  - _Prompt: Implement the task for spec keyrx-ui-ux-refinement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with expertise in user documentation and UI/UX | Task: Create comprehensive documentation for UI/UX refinement covering visual editor, profile-config workflow, auto-save behavior, with screenshots following all requirements. Use existing docs structure for consistency | Restrictions: Must use clear language (avoid jargon), include screenshots for visual features, provide step-by-step instructions, maintain existing doc format (markdown), ensure accessibility (alt text for images) | Success: Documentation covers all new features, step-by-step guides are clear and accurate, screenshots illustrate key workflows, language is user-friendly, doc structure matches existing docs | Instructions: After completing the task, run spec-workflow-guide to get the workflow guide, then update tasks.md to mark this task as in-progress [-], log the implementation with log-implementation tool including detailed artifacts (documentation files created), and mark as complete [x] when done_
