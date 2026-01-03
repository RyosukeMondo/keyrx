# Requirements Document

## Introduction

This specification addresses critical UI/UX gaps and inconsistencies in the keyrx web interface that impact user experience and workflow efficiency. The current implementation has several architectural mismatches where WASM is used inappropriately (ConfigPage), state persistence failures (ProfilesPage, DevicesPage), and data consistency issues (Dashboard device count). These issues prevent users from achieving the seamless, QMK-style configuration experience that keyrx promises.

The refinement focuses on aligning the UI architecture with user mental models: configuration editing should be React-based drag-and-drop (like QMK Configurator), simulation should use WASM for accuracy, and dashboard data should reflect real-time system state.

## Alignment with Product Vision

This specification directly supports keyrx's product vision by:

1. **Eliminating Friction**: Users expect instant, persistent configuration changes without manual saves - fulfilling the "software-level flexibility" promise
2. **AI-First Verification**: Properly segregating WASM to the Simulator page enables automated testing workflows mentioned in product.md
3. **Real-Time Simulation & Preview**: Fixing the ConfigPage architecture delivers the promised "edit-and-preview workflow"
4. **Multi-Device Support**: Consistent device detection across Dashboard and DevicesPage enables the "N:M device-to-configuration mapping" feature

## Requirements

### Requirement 0: Test Infrastructure for FFI Boundaries (FOUNDATIONAL)

**User Story:** As an AI coding agent implementing features, I want comprehensive test infrastructure for all FFI boundaries, so that I can verify implementations work correctly without human UAT

**Priority:** 🔴 **BLOCKING** - Must be implemented BEFORE all other requirements

#### Acceptance Criteria

**A. Backend API Contract Testing**

1. WHEN backend implements REST endpoint THEN OpenAPI/Swagger spec SHALL be generated automatically
2. WHEN UI calls API endpoint THEN endpoint SHALL exist and match contract (verified by contract tests)
3. WHEN API returns error THEN error format SHALL match `{error: string, code: number}` contract
4. IF API endpoint missing THEN CI/CD SHALL fail with "Contract violation: endpoint not found"

**B. Profile Template Validation Testing**

1. WHEN profile is created with template THEN template SHALL compile successfully (verified by integration test)
2. WHEN template uses invalid function THEN compiler SHALL reject with specific error message
3. WHEN new template is added THEN automated test SHALL verify it compiles to valid .krx
4. IF template breaks THEN CI/CD SHALL fail BEFORE merge

**C. Device Persistence Integration Testing**

1. WHEN layout is saved THEN data SHALL persist to `~/.config/keyrx/devices/{serial}.json` (verified by filesystem check)
2. WHEN daemon restarts THEN saved layout SHALL be loaded from filesystem
3. WHEN device is removed THEN config file SHALL remain on disk
4. IF persistence fails THEN integration test SHALL fail with filesystem error

**D. WASM FFI Boundary Testing**

1. WHEN WASM function is called from JavaScript THEN types SHALL match TypeScript definitions
2. WHEN WASM returns error THEN JavaScript SHALL receive proper Error object (not undefined)
3. WHEN profile config is invalid THEN WASM validation SHALL return structured errors
4. IF WASM function signature changes THEN TypeScript compilation SHALL fail

**E. WebSocket Contract Testing**

1. WHEN daemon sends WebSocket message THEN message SHALL match TypeScript interface
2. WHEN profile is activated THEN UI SHALL receive `profile_activated` event within 1 second
3. WHEN device is connected THEN UI SHALL receive `device_connected` event with device info
4. IF WebSocket message format changes THEN contract test SHALL fail

**F. End-to-End User Flow Testing**

1. WHEN user creates profile → edits config → activates THEN E2E test SHALL verify complete flow
2. WHEN user changes device layout → navigates away → returns THEN E2E test SHALL verify persistence
3. WHEN daemon is offline → user clicks Activate THEN E2E test SHALL verify error handling
4. IF critical user flow breaks THEN E2E test SHALL fail in CI/CD

#### Technical Context

**Why This is Foundational:**
- **Current state**: Features implemented without tests → bugs found in production
- **FFI complexity**: 3 boundaries to test (WASM ↔ JS, REST API ↔ UI, WebSocket ↔ UI)
- **AI-First principle**: Product.md promises "100% configuration verification without human UAT"
- **Test pyramid**:
  ```
  E2E Tests (10%) ← Full user flows
  Integration Tests (30%) ← API contracts, persistence, WebSocket
  Unit Tests (60%) ← Component logic, utilities
  ```

**Test Infrastructure Stack:**

1. **Backend Integration Tests** (Rust):
   - Framework: `tokio::test` + `axum-test`
   - Contract testing: OpenAPI spec validation
   - Filesystem testing: `tempfile` for isolated config directories
   - Example test:
     ```rust
     #[tokio::test]
     async fn test_device_layout_persistence() {
         let app = create_test_app().await;
         let response = app.patch("/api/devices/ABC123")
             .json(&json!({"layout": "JIS_109"}))
             .await;
         assert_eq!(response.status(), 200);

         // Verify filesystem persistence
         let config = fs::read_to_string("/tmp/test/devices/ABC123.json")?;
         assert!(config.contains("JIS_109"));
     }
     ```

2. **WASM FFI Tests** (JavaScript + Rust):
   - Framework: `wasm-bindgen-test`
   - Type checking: TypeScript strict mode
   - Example test:
     ```typescript
     import { validate_config } from '@/wasm/pkg/keyrx_core';

     test('WASM validates invalid template', () => {
         const invalidConfig = 'layer("base", #{});'; // Invalid syntax
         expect(() => validate_config(invalidConfig))
             .toThrow('Function not found: layer');
     });
     ```

3. **WebSocket Contract Tests** (TypeScript):
   - Framework: `vitest` + `ws` mock
   - Schema validation: `zod` for runtime type checking
   - Example test:
     ```typescript
     test('WebSocket sends valid device_connected event', async () => {
         const ws = new MockWebSocket();
         const message = await ws.waitForMessage();

         const schema = z.object({
             type: z.literal('device_connected'),
             device: z.object({
                 serial: z.string(),
                 name: z.string(),
             }),
         });

         expect(() => schema.parse(message)).not.toThrow();
     });
     ```

4. **E2E Tests** (Playwright):
   - Already partially implemented in web-ui-bugfix-and-enhancement
   - Need to add: Profile creation flow, Device configuration flow
   - Example test:
     ```typescript
     test('Create profile → Edit → Activate flow', async ({ page }) => {
         await page.goto('/profiles');
         await page.click('button:has-text("Create Profile")');
         await page.fill('input[name="name"]', 'Test');
         await page.click('button:has-text("Create")');

         await page.click('button:has-text("Edit")');
         await page.click('button:has-text("Code Editor")');
         // Verify template is valid
         await expect(page.locator('text=device_start')).toBeVisible();

         await page.click('button:has-text("Activate")');
         await expect(page.locator('text=[Active]')).toBeVisible();
     });
     ```

**Implementation Order:**

1. **Phase 0.1**: Backend API integration tests (catch persistence bugs)
2. **Phase 0.2**: Profile template validation tests (catch invalid templates)
3. **Phase 0.3**: WASM FFI boundary tests (catch type mismatches)
4. **Phase 0.4**: WebSocket contract tests (catch state sync issues)
5. **Phase 0.5**: E2E user flow tests (catch UI bugs)

**Success Criteria:**

- ✅ All API endpoints have integration tests with ≥90% coverage
- ✅ All profile templates compile successfully (verified by CI/CD)
- ✅ All WASM functions have TypeScript type definitions (verified by tsc)
- ✅ All WebSocket messages validate against Zod schemas
- ✅ Critical user flows have E2E tests (create profile, edit config, activate)
- ✅ CI/CD fails if any test fails - no broken code reaches main branch

### Requirement 1: Remove WASM from Configuration Editor

**User Story:** As a keyboard customization user, I want to configure key mappings using a React-based drag-and-drop interface without WASM errors, so that I can focus on remapping keys instead of debugging console errors

#### Acceptance Criteria

1. WHEN ConfigPage loads THEN the page SHALL NOT call `useWasm()` hook
2. WHEN user navigates to /config THEN browser console SHALL show zero WASM-related errors
3. WHEN user edits configuration in visual mode THEN validation SHALL use backend API, not WASM
4. WHEN user switches between visual and code tabs THEN WASM module SHALL NOT be loaded
5. IF user makes configuration changes THEN changes SHALL be validated via REST API `/api/profiles/{name}/validate` endpoint

#### Technical Context

- Current issue: ConfigPage.tsx:45 calls `useWasm()` causing error "Cannot read properties of undefined (reading 'wasm_init')"
- Root cause: WASM module doesn't exist at `@/wasm/pkg/keyrx_core.js` and isn't needed for configuration editing
- Impact: User sees console warnings and validation fails

### Requirement 2: Dashboard Device Count Consistency (✅ FIXED)

**User Story:** As a user monitoring my system, I want the Dashboard to show the same device count as the Devices page, so that I have accurate real-time information about connected keyboards

#### Acceptance Criteria

1. WHEN user views HomePage THEN DeviceListCard SHALL display actual connected device count ✅ WORKING
2. WHEN 3 devices are connected AND user navigates to / THEN Dashboard SHALL show "Connected Devices (3)" ✅ WORKING
3. WHEN user navigates to /devices THEN device count SHALL match Dashboard count ✅ WORKING
4. WHEN device is connected/disconnected THEN both Dashboard and DevicesPage SHALL update within 2 seconds
5. IF device fetch fails THEN DeviceListCard SHALL show error state with retry button

#### Technical Context

- **Status**: ✅ **FIXED** - Dashboard now correctly shows "Connected Devices (3)"
- **Current behavior**: DeviceListCard properly displays all connected devices with serial numbers
- **No changes needed** - This issue resolved itself
- **Verification**: User confirmed devices display correctly:
  - ARCHISS PK85PD (JP)
  - keyrx (/dev/input/event26)
  - USB Keyboard (/dev/input/event7)

### Requirement 3: Persist DevicesPage Layout and Scope Selection

**User Story:** As a user configuring my keyboard layout and scope, I want my selections (ANSI 104, JIS 109, Global, Device-Specific) to persist when I navigate away, so that I don't have to reconfigure every time

#### Acceptance Criteria

1. WHEN user selects JIS 109 layout on /devices THEN selection SHALL persist to backend within 500ms
2. WHEN user selects "Device-Specific" scope THEN selection SHALL persist to backend within 500ms
3. WHEN user navigates away and returns to /devices THEN both layout AND scope SHALL display saved values (not "Not Set")
4. WHEN layout save completes THEN user SHALL see "✓ Saved" feedback within 500ms
5. WHEN scope save completes THEN user SHALL see "✓ Saved" feedback within 500ms
6. WHEN save fails THEN user SHALL see "✗ Error" with error message
7. IF backend API returns error THEN user SHALL see toast notification with retry option
8. WHEN page loads THEN devices SHALL display actual saved values (e.g., "Layout: JIS 109", "Scope: Device-Specific")

#### Technical Context

- **Current state**: Auto-save IS implemented via `useAutoSave` hook (DevicesPage.tsx:76-88)
- **User report**: All devices show "Scope: Not Set" and "Layout: Not Set" despite selections
- **Root cause**: Backend API endpoint missing or not persisting data
  - Expected endpoint: `PATCH /api/devices/{serial}` with `{layout: "JIS_109", scope: "device-specific"}`
  - Expected persistence: `~/.config/keyrx/devices/{serial}.json`
- **Fix needed**:
  1. Implement backend REST endpoint if missing
  2. Ensure data persists to filesystem
  3. Add visible success/error feedback in UI
  4. Invalidate query cache on success to show updated values

### Requirement 4: Validate Profiles Before Activation and Fix Template

**User Story:** As a user activating a keyboard profile, I want to know if my profile configuration is valid before clicking Activate, so that I don't encounter cryptic compiler errors

#### Acceptance Criteria

1. WHEN profile is created with template THEN template SHALL use correct Rhai syntax (`device_start()`/`device_end()`, NOT `layer()`)
2. WHEN profile has invalid Rhai syntax THEN profile card SHALL show "⚠️ Invalid Configuration" badge
3. WHEN profile has valid syntax THEN profile card SHALL show "✓ Valid" badge or no warning
4. WHEN user hovers over "⚠️ Invalid" badge THEN tooltip SHALL show first error message
5. IF profile is invalid THEN [Activate] button SHALL be disabled with tooltip "Fix configuration errors first"
6. WHEN user clicks "Edit" on invalid profile THEN ConfigPage SHALL show validation errors inline
7. WHEN user clicks "Activate" on valid profile AND activation fails THEN error message SHALL show:
   - File path and line number
   - Specific syntax error
   - Suggestion to check DSL manual
8. WHEN user creates new profile THEN template options SHALL be offered:
   - Blank (minimal valid config)
   - Simple Remap (A→B examples)
   - CapsLock→Escape
   - Vim Navigation Layer
   - Gaming Profile
9. WHEN activation succeeds THEN [Active] badge SHALL remain visible permanently
10. WHEN user refreshes page THEN previously activated profile SHALL still show [Active] badge

#### Technical Context

- **Current issue**: Profile "ooo" created with invalid template using `layer()` function which doesn't exist
- **Actual error**: `Function not found: layer (&str | ImmutableString | String, map)` in `/home/rmondo/.config/keyrx/profiles/ooo.rhai:3:1`
- **Wrong template**:
  ```rhai
  layer("base", #{
      // Add your key mappings here
  });
  ```
- **Correct syntax** (from examples/01-simple-remap.rhai):
  ```rhai
  device_start("*");
    map("VK_A", "VK_B");  // Example mapping
  device_end();
  ```
- **Fix needed**:
  1. Update profile creation template to use `device_start()`/`device_end()`
  2. Add validation endpoint: `POST /api/profiles/{name}/validate` returns compilation errors
  3. Run validation on profile load and show status badge
  4. Disable [Activate] button if validation fails
  5. Provide template selector when creating new profiles
  6. Fix [Active] badge persistence (likely WebSocket state overwriting local state)

### Requirement 5: Redesign ConfigPage as QMK-Style Profile Editor

**User Story:** As a keyboard remapping user familiar with QMK Configurator, I want a visual drag-and-drop editor to assign VK_ keys, MD_ modifiers, and LK_ locks to physical keys, so that I can configure my keyboard without writing Rhai code

#### Acceptance Criteria

1. WHEN user navigates to /config?profile=Gaming THEN page SHALL show profile-specific configuration
2. WHEN user views ConfigPage THEN page SHALL display:
   - Device list with global/device-specific toggle
   - Layer selector (Base, Layer 1, Layer 2, etc.) extracted from profile config
   - Keyboard layout visualizer with current key mappings
   - Key palette with draggable VK_, MD_, LK_ keys
3. WHEN user drags VK_65 (key 'A') from palette onto CAPS key THEN:
   - Key mapping SHALL update in real-time
   - Backend SHALL receive PATCH `/api/profiles/{name}/mapping` request
   - Visual indicator SHALL show mapping assigned
4. WHEN user clicks physical key on visualizer THEN modal SHALL open showing:
   - Simple mapping: Select target key
   - Tap-Hold mapping: Tap action, Hold action, threshold (ms)
   - Macro mapping: Key sequence editor
   - Layer switch mapping: Target layer selector
5. WHEN user saves mapping THEN ConfigPage SHALL update Rhai source code automatically
6. IF user switches to Code Editor tab THEN visual changes SHALL be reflected in Rhai syntax

#### Technical Context

- Current state: ConfigPage has drag-and-drop but uses WASM for validation
- Gap: User expects QMK-style editor, not WASM-based config validator
- Architecture change needed: ConfigPage = Profile-centric editor, SimulatorPage = WASM simulation
- Comparison with QMK Configurator:
  - QMK: Drag keys → Update firmware JSON → Compile → Flash
  - keyrx: Drag keys → Update Rhai config → Compile .krx → Daemon reload

### Requirement 6: Move ConfigPage to Profile Sub-Route (Optional UX Enhancement)

**User Story:** As a user organizing my workflow, I want the configuration editor to be logically grouped under the profile I'm editing, so that the navigation hierarchy matches my mental model

#### Acceptance Criteria

1. WHEN user clicks "Edit" on Gaming profile THEN navigation SHALL go to `/profiles/Gaming/config`
2. WHEN user is on `/profiles/Gaming/config` THEN breadcrumb SHALL show: Home → Profiles → Gaming → Configuration
3. WHEN user saves changes THEN user SHALL remain on `/profiles/Gaming/config` (no redirect)
4. WHEN user clicks profile name in breadcrumb THEN navigation SHALL return to `/profiles`

#### Technical Context

- Current route: `/config?profile=Gaming` (query parameter)
- Proposed route: `/profiles/Gaming/config` (nested route)
- Rationale: Follows 2025 UX trend of hierarchical navigation (see Web Search results)
- Optional: Can be deferred if user prefers current structure

### Requirement 7: Display Active Profile on MetricsPage (Already Implemented)

**User Story:** As a user monitoring performance, I want the Metrics page to show which profile and .rhai file is currently active, so that I can correlate metrics with the active configuration

#### Acceptance Criteria

1. WHEN user navigates to /metrics THEN page SHALL display active profile name ✅ (Already implemented)
2. WHEN active profile is "Gaming" THEN page SHALL show "Gaming.rhai" ✅ (Already implemented)
3. WHEN no profile is active THEN page SHALL show "No active profile" ✅ (Already implemented)
4. WHEN profile is changed THEN MetricsPage SHALL update within 2 seconds ✅ (Already implemented)

#### Technical Context

- **Status**: ✅ Already correctly implemented in MetricsPage.tsx:263-314
- Uses `useActiveProfile()` hook to fetch active profile
- Displays profile name, .rhai filename, and last modified timestamp
- Shows "Activate a profile" link when no profile is active
- **No changes needed**

### Requirement 8: Keep WASM Simulation in SimulatorPage (Already Implemented)

**User Story:** As a user testing my configuration, I want the Keyboard Simulator to use WASM for accurate, deterministic simulation matching the daemon behavior, so that I can verify tap-hold timing and layer switching before activating the profile

#### Acceptance Criteria

1. WHEN user selects profile on /simulator THEN WASM SHALL load profile's .rhai config ✅ (Already implemented)
2. WHEN user clicks key in simulator THEN WASM SHALL process event using same DFA logic as daemon ✅ (Already implemented)
3. WHEN simulation runs THEN event log SHALL show state transitions (press, hold, tap, output) ✅ (Already implemented)
4. WHEN WASM not available THEN page SHALL show "⚠ WASM not available (run build:wasm)" warning ✅ (Already implemented)
5. IF WASM simulation fails THEN page SHALL fall back to mock simulation with warning ✅ (Already implemented)

#### Technical Context

- **Status**: ✅ Already correctly implemented in SimulatorPage.tsx:64, 156-210, 271-324
- Loads WASM module correctly via `useWasm()` hook
- Uses profile config for realistic simulation
- Shows clear warnings when WASM unavailable
- **No changes needed** - this is the correct place for WASM usage

## Non-Functional Requirements

### Code Architecture and Modularity

1. **Separation of Concerns**:
   - ConfigPage SHALL focus on profile editing (no WASM)
   - SimulatorPage SHALL focus on testing (with WASM)
   - Shared UI components (KeyboardVisualizer, KeyAssignmentPanel) SHALL remain reusable
   - Validation logic SHALL be abstracted into `useConfigValidation()` hook

2. **File Size Limits**:
   - All new/modified files SHALL not exceed 500 lines of code (excluding comments/blanks)
   - If ConfigPage exceeds limit, extract sub-components (e.g., ProfileConfigEditor, DeviceScopeSelector)

3. **Dependency Injection**:
   - ConfigPage SHALL accept optional `apiClient` prop for testing
   - All WebSocket subscriptions SHALL be abstracted via `useUnifiedApi()` hook
   - Device fetching SHALL use `useDevices()` hook consistently

4. **State Management**:
   - Profile activation state SHALL use React Query's optimistic updates
   - Device layout changes SHALL use optimistic updates with rollback on error
   - Configuration changes SHALL debounce at 500ms before API calls

### Performance

1. **ConfigPage Load Time**:
   - SHALL load within 1 second without WASM overhead
   - SHALL defer loading Monaco editor until Code Editor tab is opened (lazy loading)

2. **Dashboard Real-Time Updates**:
   - Device status changes SHALL reflect within 2 seconds via WebSocket
   - Profile activation changes SHALL reflect within 1 second

3. **Layout Persistence**:
   - Layout selection SHALL save within 500ms (auto-save debounce)
   - User SHALL see visual feedback (spinner + success checkmark) within 100ms of save completion

### Security

1. **API Validation**:
   - All configuration updates SHALL validate on backend before persisting
   - Invalid Rhai syntax SHALL return HTTP 400 with line numbers
   - Profile activation SHALL return compilation errors with stack traces

2. **Input Sanitization**:
   - Device names SHALL be sanitized to prevent XSS
   - Profile names SHALL be validated against filesystem constraints (no `/`, `\`, null bytes)

### Reliability

1. **Error Recovery**:
   - IF DeviceListCard fetch fails THEN show error state with retry button
   - IF profile activation fails THEN show error message without changing UI state
   - IF layout save fails THEN revert dropdown to previous value + show error toast

2. **Offline Behavior**:
   - IF daemon is offline THEN disable Activate buttons with tooltip "Daemon offline"
   - IF WebSocket disconnects THEN show warning banner "Live updates paused"

### Usability

1. **Accessibility (WCAG 2.2 Level AA)**:
   - All drag-and-drop operations SHALL have keyboard alternatives (Tab to key, Space to activate, Arrow keys to select target, Enter to confirm)
   - All state changes SHALL announce via `aria-live` regions
   - All interactive elements SHALL have minimum 44×44px touch target (already implemented in web-ui-bugfix-and-enhancement spec)

2. **Visual Feedback**:
   - All async operations (save, activate, fetch) SHALL show loading spinners
   - All success states SHALL show checkmark icon + green text for 2 seconds
   - All error states SHALL show red text with actionable error message

3. **Mobile Responsiveness**:
   - ConfigPage drag-and-drop SHALL work on touch screens (already implemented via @dnd-kit)
   - Device cards SHALL stack vertically on screens <768px (already implemented)
   - Modal dialogs SHALL fill 90% of screen width on mobile

4. **User Guidance**:
   - ConfigPage SHALL show onboarding tooltip on first visit: "Drag keys from palette or click keys to configure"
   - Empty states SHALL provide clear next actions: "No devices connected. Connect a keyboard to get started."

## Out of Scope

The following are explicitly NOT part of this specification:

1. **WASM Build Infrastructure**: This spec assumes WASM module already exists at build time (covered by separate build system spec)
2. **Backend API Changes**: This spec works with existing REST APIs; no new endpoints required (only fixes to existing persistence logic)
3. **Rhai Language Features**: No changes to Rhai DSL syntax or compiler behavior
4. **New Configuration Features**: No new mapping types (tap-hold-double-tap, chording, etc.) - only UI for existing features
5. **Daemon Hot-Reload**: Daemon reload behavior on profile activation is out of scope (assumes existing reload mechanism works)

## Success Metrics

1. **Zero WASM Errors on ConfigPage**: Browser console shows 0 WASM-related errors when loading /config
2. **Device Count Consistency**: ✅ Dashboard and DevicesPage show identical device count 100% of the time (ALREADY ACHIEVED)
3. **Layout/Scope Persistence Rate**: 100% of layout and scope selections persist after navigation (currently 0%)
4. **Profile Template Validity**: 100% of newly created profiles use valid Rhai syntax (currently 0% - all use invalid `layer()` syntax)
5. **Profile Validation Before Activation**: 100% of invalid profiles show warning badge and disabled [Activate] button (currently 0% - no validation)
6. **Profile Activation Success Rate**: Valid profiles show [Active] badge permanently after activation in >95% of cases
7. **User Task Completion Time**: Time to "create valid profile and remap Caps Lock to Escape" reduces from ~10 minutes (fixing template + writing Rhai) to <2 minutes (correct template + drag-and-drop)

## 2025 UI/UX Trends Applied

Based on web search results, this specification incorporates the following 2025 design trends:

1. **Accessibility-First Design** (Sources [1], [2], [3], [4], [5])
   - Keyboard navigation for all drag-and-drop operations
   - ARIA live regions for state announcements
   - Screen reader compatibility tested with axe-core (already implemented in web-ui-bugfix-and-enhancement)

2. **Minimalist and Inclusive Design** (Sources [6], [7])
   - Removed unnecessary WASM complexity from ConfigPage
   - Clear visual hierarchy: Devices → Layers → Keys → Mappings
   - Consistent color palette for state indicators (green=active, yellow=pending, red=error)

3. **Cross-Platform Synchronization** (Sources [8])
   - WebSocket real-time updates ensure Dashboard and DevicesPage show consistent data
   - Profile activation state syncs across all pages within 1 second

4. **Motion Design and Micro-Interactions** (Sources [9])
   - Drag-and-drop with visual overlay (already implemented via @dnd-kit)
   - Success checkmark animation on save completion
   - Loading spinners for async operations

5. **Dark Mode** (Sources [10])
   - Already implemented with Tailwind dark mode classes
   - High contrast ensured via WCAG color contrast requirements (already tested in a11y tests)

## References

**2025 UI/UX Trends Sources**:
- [1] [8 Latest UI UX Design Trends to Know in 2026 | AND Academy](https://www.andacademy.com/resources/blog/ui-ux-design/latest-ui-ux-design-trends/)
- [2] [Top UI/UX Design Trends to Watch in 2025 | Medium](https://medium.com/@designstudiouiux/top-ui-ux-design-trends-to-watch-in-2025-75d46042cb07)
- [3] [Top UI/UX Design Trends for 2026 - eLEOPARD](https://eleopardsolutions.com/ui-ux-trends/)
- [4] [UI/UX Design Trends 2025: Redefining Digital Experiences - Reactree](https://reactree.com/ui-ux-design-trends-2025-redefining-digital-experiences/)
- [5] [Top 10 UI/UX Design Trends Reshaping the Future in 2025 | UX Planet](https://uxplanet.org/10-game-changing-ui-ux-design-trends-to-watch-in-2025-b28863831a6a)
- [6] [The Future of UI and UX Design: Top Trends for 2025 - BlueWhaleApps](https://bluewhaleapps.com/blog/ui-ux-design-trends-2025)
- [7] [Top UI/UX Trends in 2025: What to Expect and Implement](https://www.wildnetedge.com/blogs/top-ui-ux-trends-2025-what-you-need-to-know-about-user-interface-trends)
- [8] [Top UI/UX design trends to watch in 2025 | Medium](https://medium.com/@ryan.almeida86/top-ui-ux-design-trends-to-watch-in-2025-2bb4e68d6a9c)
- [9] [UI trends 2026: top 10 trends your users will love](https://www.uxstudioteam.com/ux-blog/ui-trends-2019)
- [10] [Top 20 UI/UX Design Trends to Watch in 2025](https://arounda.agency/blog/top-20-ui-ux-design-trends-to-watch-in-2025)
