# Requirements Document: Web UI Bugfix and Enhancement

## Introduction

The existing keyrx web UI (`keyrx_ui/`) has several critical bugs and missing features that prevent it from providing a professional, QMK-style configuration experience. Users report:
- **Inconsistent device counts** between Dashboard and Devices pages
- **Lost configuration state** when navigating between pages (layout selections, profile activation)
- **No visual drag-and-drop editor** for key remapping (current Monaco-based editor doesn't match QMK Configurator UX)
- **WASM misplaced** in config page instead of dedicated simulator
- **Missing profile context** in metrics and configuration workflows

This specification addresses these issues by **fixing bugs in the existing `keyrx_ui/` codebase** and adding a **QMK-style drag-and-drop configuration editor** aligned with 2026 UI/UX best practices.

**Scope**: Fix existing UI bugs + add drag-and-drop key mapping. Does NOT create a new UI from scratch.

## Alignment with Product Vision

From `product.md`:
- **AI-First Verification**: Visual config editor must generate deterministically testable Rhai code
- **Real-Time Simulation & Preview**: WASM simulator provides instant feedback on configuration changes
- **Sub-Millisecond Latency**: UI must show active profile to confirm daemon is running correct configuration
- **Multi-Device Support**: Device-specific vs global configuration must be clearly visualized

From `tech.md`:
- **React + WASM frontend**: Reuse existing keyrx_ui architecture
- **Embedded Web Server (axum)**: REST API + WebSocket already implemented
- **TypeScript type safety**: Maintain strict typing throughout

## 2026 UI/UX Design Trends Research

Based on industry research ([UI trends 2026](https://www.uxstudioteam.com/ux-blog/ui-trends-2019), [12 UI/UX Design Trends](https://www.index.dev/blog/ui-ux-design-trends), [Drag-and-Drop UX Guidelines](https://smart-interface-design-patterns.com/articles/drag-and-drop-ux/)):

1. **Keyboard-Accessible Drag-and-Drop**: Salesforce pattern (Tab to focus, Space to grab, arrows to move, Space to drop) for WCAG 2.2 compliance
2. **Visual No-Code Builders**: Drag-and-drop interfaces (Webflow, Unicorn Studio) enable faster iteration without code
3. **Full Keyboard Navigation**: Microsoft Teams demonstrates polished, professional UX with complete keyboard control
4. **Accessibility First**: WCAG 2.2 standards, clear color contrast, semantic HTML, screen reader support

**Application to keyrx**: QMK-style visual editor with drag-and-drop key assignment, full keyboard navigation, and auto-generation of Rhai code.

## Requirements

### Requirement 1: Dashboard Device Count Accuracy

**User Story:** As a user, I want the Dashboard to show the actual number of connected devices, so that I can quickly verify my keyboards are detected.

**Current Bug**: Dashboard shows "Connected Devices (0)" but `/devices` page shows "Device List (3 connected)".

**Root Cause**: Dashboard's `DeviceListCard` component doesn't fetch from `/api/devices`.

#### Acceptance Criteria

1. WHEN Dashboard loads THEN it SHALL fetch device count from GET `/api/devices` API
2. WHEN 3 devices are connected THEN Dashboard SHALL display "Connected Devices (3)"
3. WHEN device count changes THEN Dashboard SHALL update within 2 seconds (via polling or WebSocket)
4. WHEN API request fails THEN Dashboard SHALL display "Connected Devices (error)" with retry button
5. WHEN navigating Dashboard → Devices → Dashboard THEN device count SHALL remain consistent

#### Input Validation

6. WHEN API returns null/undefined THEN Dashboard SHALL treat as 0 devices
7. WHEN API returns non-array THEN Dashboard SHALL log error and display 0 devices

#### Error Scenarios

8. WHEN daemon is offline THEN Dashboard SHALL show "Daemon offline" message with "Start Daemon" button
9. WHEN fetch timeout (>5 seconds) THEN Dashboard SHALL show cached count with "Refreshing..." indicator

---

### Requirement 2: Device Layout Selection Persistence

**User Story:** As a user, I want my keyboard layout selection (JIS 109, ANSI 104, etc.) to persist when I navigate away, so that I don't have to reconfigure it every time.

**Current Bug**: Devices page allows selecting layout via dropdown, but selection is lost on navigation.

**Existing Implementation**: `DevicesPage.tsx` has `useAutoSave` hook (lines 76-88) that should persist layout changes.

#### Acceptance Criteria

1. WHEN user selects "JIS 109" layout THEN system SHALL call `rpcClient.setDeviceLayout(serial, "JIS_109")` within 500ms (debounced)
2. WHEN API call succeeds THEN system SHALL display "✓ Saved" indicator for 2 seconds
3. WHEN user navigates away and returns THEN layout selection SHALL show saved value from API
4. WHEN API call fails THEN system SHALL display "✗ Error" with error message tooltip
5. WHEN user changes layout multiple times rapidly THEN system SHALL debounce and only save final value

#### Input Validation

6. WHEN device has no serial number THEN system SHALL disable layout dropdown with tooltip "Serial required"
7. WHEN invalid layout value (not in LAYOUT_OPTIONS) THEN system SHALL reject with error

#### Error Scenarios

8. WHEN network error occurs THEN system SHALL retry up to 3 times with exponential backoff (500ms, 1s, 2s)
9. WHEN daemon returns 500 error THEN system SHALL show error message and keep dropdown enabled for retry
10. WHEN user is offline THEN system SHALL queue changes and sync when connection restored

---

### Requirement 3: Profile Activation State Persistence

**User Story:** As a user, I want the active profile badge to persist after activation, so that I can verify which profile the daemon is running.

**Current Bug**: Clicking "Activate" shows "[Active]" badge for ~1 second, then disappears. Profile activation state is not persisting.

**Root Cause**: `ProfilesPage.tsx` lines 113-132 show activation logic with error handling, but `isActive` state comes from API and may not be updating correctly.

#### Acceptance Criteria

1. WHEN user clicks "Activate" on profile THEN system SHALL call `activateProfileMutation.mutateAsync(profileId)`
2. WHEN activation succeeds THEN system SHALL:
   - Deactivate previous active profile (set `isActive: false`)
   - Activate selected profile (set `isActive: true`)
   - Persist state to daemon configuration
   - Display "[Active]" badge with green checkmark indefinitely
3. WHEN activation fails due to compilation error THEN system SHALL:
   - Display error modal with line numbers and error messages
   - Keep previous profile active
   - Show red "Compilation Failed" indicator on failed profile card
4. WHEN page refreshes THEN active profile SHALL load from GET `/api/profiles` with `isActive: true`
5. WHEN daemon is running active profile THEN metrics page SHALL display profile name in header

#### Input Validation

6. WHEN profile name is empty THEN system SHALL reject activation with error "Invalid profile name"
7. WHEN profile doesn't exist THEN system SHALL return 404 error with "Profile not found"

#### Error Scenarios

8. WHEN daemon crashes during activation THEN system SHALL:
   - Auto-restart daemon with previous working profile
   - Display error notification "Daemon restarted with previous profile"
   - Log crash details for debugging
9. WHEN compilation times out (>30 seconds) THEN system SHALL:
   - Kill compilation process
   - Return error "Compilation timeout"
   - Keep previous profile active
10. WHEN .rhai file has syntax error THEN system SHALL:
    - Return compilation errors with line numbers
    - Highlight errors in Monaco editor (if config page is open)
    - Keep previous profile active

---

### Requirement 4: QMK-Style Drag-and-Drop Configuration Editor

**User Story:** As a user, I want to assign key mappings by dragging virtual keys (VK_, MD_, LK_) from a palette and dropping them onto a visual keyboard layout, so that I can configure my keyboard without writing code.

**Current Implementation**: `ConfigPage.tsx` uses Monaco editor with WASM validation. This is code-first, not visual-first.

**Target UX**: QMK Configurator style with visual keyboard layout, drag-and-drop key assignment, and auto-generated Rhai code.

#### Acceptance Criteria

1. WHEN ConfigPage loads THEN system SHALL display:
   - Visual keyboard layout (selected from ANSI 104, ISO 105, JIS 109, HHKB, Numpad)
   - Key assignment palette with categories: VK_ (virtual keys), MD_ (modifiers), LK_ (locks), Layers, Macros
   - Current profile name in header
   - Active layer selector (base, nav, num, fn, gaming)
2. WHEN user drags VK_A from palette THEN system SHALL:
   - Show drag preview (visual key with label "A")
   - Highlight valid drop targets (all keyboard keys in current layer)
   - Support keyboard navigation: Tab to focus key, Space to grab, arrows to move, Space to drop ([Salesforce pattern](https://smart-interface-design-patterns.com/articles/drag-and-drop-ux/))
3. WHEN user drops VK_A onto physical key "CapsLock" THEN system SHALL:
   - Update key mapping: CapsLock → A (in current layer)
   - Auto-generate Rhai code: `map("CapsLock", VK_A)`
   - Save mapping via PUT `/api/config/:profile/key`
   - Display success indicator "✓ Saved" for 2 seconds
   - Update visual keyboard to show "A" label on CapsLock key
4. WHEN user clicks keyboard key THEN system SHALL:
   - Open KeyMappingDialog modal with current assignment
   - Allow selection of: Simple Key, Tap-Hold, Macro, Layer Switch
   - Show form fields based on selection type
   - Validate inputs before allowing Save
5. WHEN user selects "Tap-Hold" THEN dialog SHALL show:
   - Tap action dropdown (VK_ keys)
   - Hold action dropdown (VK_ keys, Layers, Modifiers)
   - Timeout slider (100-500ms, default 200ms)
   - Preview text: "Tap: A, Hold: Layer(Nav), Timeout: 200ms"
6. WHEN user saves mapping THEN system SHALL:
   - Generate Rhai code: `tap_hold("CapsLock", VK_A, Layer("nav"), 200)`
   - Call PUT `/api/config/:profile/key` with mapping
   - Auto-compile profile (daemon hot-reload)
   - Update visual keyboard display
7. WHEN user switches layers THEN system SHALL:
   - Load mappings for selected layer from profile config
   - Update visual keyboard to show layer-specific mappings
   - Highlight layer-switch keys (e.g., "Hold for Nav" on Space key)

#### Layer Visualization

8. WHEN base layer is active THEN all keys SHALL display their base mappings
9. WHEN nav layer is selected THEN:
   - Keys with nav-layer mappings SHALL show nav actions (e.g., "←" "→" "↑" "↓")
   - Keys without nav mappings SHALL show greyed-out base mappings
   - Layer-switch trigger keys SHALL show "→ Nav" indicator
10. WHEN user drags MD_SHIFT (modifier) onto key THEN system SHALL:
    - Generate code: `modifier("LeftShift", MD_SHIFT)`
    - Show modifier badge on visual key

#### Input Validation

11. WHEN user tries to map reserved system keys (Power, Sleep) THEN system SHALL reject with warning
12. WHEN user creates circular layer dependency (Layer A → Layer B → Layer A) THEN system SHALL reject with error
13. WHEN user exceeds 16 layers limit THEN system SHALL disable "Add Layer" button

#### Error Scenarios

14. WHEN Rhai code generation fails THEN system SHALL:
    - Display error modal with details
    - Revert to previous working mapping
    - Log error for debugging
15. WHEN daemon compilation fails THEN system SHALL:
    - Display compilation errors in error panel
    - Keep previous working configuration active
    - Allow user to fix errors via Monaco code editor (fallback)

#### Accessibility (WCAG 2.2 Level AA)

16. WHEN user navigates via keyboard THEN:
    - Tab/Shift+Tab: Navigate between palette items and keyboard keys
    - Space: Grab/Drop dragged item
    - Arrow keys: Move dragged item to adjacent key
    - Escape: Cancel drag operation
    - Enter: Open KeyMappingDialog for focused key
17. WHEN screen reader is active THEN all keys SHALL have aria-labels: "CapsLock mapped to A"
18. WHEN drag operation is active THEN system SHALL announce: "Grabbed VK_A, use arrows to select drop target"

---

### Requirement 5: Profile-Centric Configuration Workflow

**User Story:** As a user, I want configuration to be clearly tied to a specific profile, so that I understand which .rhai file I'm editing and can switch between profile configs easily.

**Current Implementation**: ConfigPage accepts `profileName` as prop/query param but doesn't show clear profile context.

#### Acceptance Criteria

1. WHEN ConfigPage loads THEN header SHALL display:
   - Active profile name (e.g., "Editing: Gaming Profile")
   - Active profile badge (if this profile is active in daemon)
   - Last modified timestamp
2. WHEN user opens ConfigPage from ProfilesPage THEN system SHALL:
   - Pass profile name via query param: `/config?profile=gaming`
   - Load configuration for that specific profile
   - Show breadcrumb: "Profiles > Gaming > Edit Configuration"
3. WHEN user switches profiles THEN system SHALL:
   - Display profile selector dropdown in ConfigPage header
   - Load new profile's configuration
   - Update URL: `/config?profile=work`
   - Preserve unsaved changes warning if editing
4. WHEN profile is active in daemon THEN ConfigPage SHALL:
   - Show green "Active" badge next to profile name
   - Display warning: "This profile is currently running. Changes will be applied after hot-reload."
5. WHEN user saves configuration THEN system SHALL:
   - Auto-compile .rhai to .krx
   - If profile is active: hot-reload daemon (<100ms)
   - If profile is inactive: just save to disk

#### Input Validation

6. WHEN profile name in URL doesn't exist THEN system SHALL redirect to ProfilesPage with error
7. WHEN user has unsaved changes and tries to switch profiles THEN system SHALL show confirmation dialog

#### Error Scenarios

8. WHEN hot-reload fails THEN system SHALL:
   - Keep daemon running with previous configuration
   - Display error notification with rollback option
   - Log error details for debugging

---

### Requirement 6: Metrics Page Profile Display

**User Story:** As a user, I want the Metrics page to show which profile the daemon is currently running, so that I can verify my configuration is active.

**Current Implementation**: `MetricsPage.tsx` shows latency stats and event log but no profile information.

#### Acceptance Criteria

1. WHEN MetricsPage loads THEN header SHALL display:
   - "Active Profile: [profile name]" (e.g., "Active Profile: Gaming")
   - .rhai file name (e.g., "gaming.rhai")
   - Profile activation timestamp (e.g., "Activated: 2026-01-03 15:56")
2. WHEN daemon is running with default profile THEN Metrics SHALL show "Active Profile: Default"
3. WHEN profile changes (user activates different profile) THEN Metrics SHALL update within 2 seconds via WebSocket event
4. WHEN user clicks profile name THEN system SHALL navigate to ConfigPage for that profile

#### Input Validation

5. WHEN daemon is not running any profile THEN Metrics SHALL show "No active profile" with "Activate Profile" button

#### Error Scenarios

6. WHEN daemon is offline THEN Metrics SHALL show "Daemon offline - Profile information unavailable"

---

### Requirement 7: WASM Simulator Dedicated Page

**User Story:** As a user, I want to test my keyboard configuration in a realistic simulator that uses the same WASM engine as the daemon, so that I can verify behavior before activating a profile.

**Current Implementation**: ConfigPage has WASM integration but it's mixed with Monaco editor. User wants WASM on dedicated Simulator page.

#### Acceptance Criteria

1. WHEN SimulatorPage loads THEN system SHALL display:
   - Profile selector dropdown (select which profile to simulate)
   - Visual keyboard layout matching selected device layout
   - Event log panel (input events → output events)
   - State inspector (active modifiers, locks, layer)
2. WHEN user selects "Gaming" profile THEN system SHALL:
   - Load gaming.rhai configuration
   - Compile to WASM-compatible format
   - Initialize WASM simulator with profile config
   - Display "Simulating: Gaming" in header
3. WHEN user clicks virtual key "A" on keyboard THEN system SHALL:
   - Send KeyPress event to WASM simulator
   - Process event through DFA (Deterministic Finite Automaton)
   - Display output event in event log: "Input: A (Press) → Output: A"
   - Update state inspector if key is modifier/lock
4. WHEN user holds "CapsLock" (mapped to tap-hold) THEN system SHALL:
   - Start timeout timer (visual countdown)
   - After threshold: Activate layer/modifier
   - Display state transition: "Pending → Held (200ms)"
   - Update event log: "CapsLock (Hold) → Layer(Nav) activated"
5. WHEN user types key sequence THEN simulator SHALL:
   - Process each event in order
   - Show real-time state transitions
   - Display final output sequence
   - Support clipboard copy of event log for debugging

#### WASM Integration

6. WHEN WASM module loads THEN system SHALL:
   - Initialize keyrx_core WASM (compiled from Rust)
   - Load profile configuration via `loadConfig(rhaiSource)`
   - Expose `processKeyEvent(keyCode, eventType)` API
   - Return `{outputEvents: [], newState: {}}` on each call
7. WHEN WASM initialization fails THEN system SHALL:
   - Display error message with fallback instructions
   - Disable simulator controls
   - Offer "Reload WASM" button

#### Input Validation

8. WHEN profile has invalid Rhai syntax THEN system SHALL:
   - Display compilation errors in error panel
   - Disable simulator until errors are fixed
   - Highlight errors if Monaco editor tab is available

#### Error Scenarios

9. WHEN simulator state becomes inconsistent (bug in WASM) THEN system SHALL:
   - Display "State error detected" warning
   - Offer "Reset Simulator" button
   - Log state dump for debugging

---

### Requirement 8: Device List Integration in ConfigPage

**User Story:** As a user, I want the ConfigPage to show my connected devices and allow toggling between global and device-specific configurations, so that I can create device-specific key mappings.

**Current Implementation**: ConfigPage has `DeviceScopeToggle` component with mock devices.

#### Acceptance Criteria

1. WHEN ConfigPage loads THEN device selector SHALL display:
   - "Global" option (applies to all devices)
   - List of connected devices with names (from GET `/api/devices`)
   - Currently selected scope (global or device-specific)
2. WHEN user selects "Global" THEN system SHALL:
   - Load global configuration mappings
   - Display "Editing: Global Configuration" in header
   - Save mappings to global scope in profile
3. WHEN user selects specific device (e.g., "ARCHISS PK85PD") THEN system SHALL:
   - Load device-specific mappings for that serial number
   - Display "Editing: ARCHISS PK85PD (Device-Specific)" in header
   - Save mappings to device-specific scope in profile
4. WHEN device-specific mapping conflicts with global THEN system SHALL:
   - Apply device-specific mapping (device overrides global)
   - Show visual indicator: "⚠ Overrides global mapping"
   - Allow user to "Reset to Global" via context menu

#### Input Validation

5. WHEN device is disconnected THEN system SHALL:
   - Grey out device in selector
   - Show "Offline" status
   - Still allow editing (config persists for when device reconnects)

#### Error Scenarios

6. WHEN API fails to fetch devices THEN system SHALL:
   - Show cached device list with "Using cached data" indicator
   - Retry fetch in background
   - Display error if cache is empty

---

## Non-Functional Requirements

### Code Architecture and Modularity

#### Single Responsibility Principle
- **DeviceListCard**: Only displays device count and basic info (no configuration logic)
- **DevicesPage**: Manages device list, rename, scope toggle, layout selection
- **ConfigPage**: Manages key mapping configuration (delegates to KeyMappingDialog for per-key config)
- **SimulatorPage**: Manages WASM simulation (no configuration editing)
- **KeyMappingDialog**: Handles single key configuration modal (no keyboard layout rendering)

#### Modular Design
- **Shared Components**: Extract common patterns:
  - `useAutoSave` hook (already exists, reuse across DevicesPage and ConfigPage)
  - `DeviceScopeToggle` component (reuse in ConfigPage)
  - `KeyboardVisualizer` component (reuse in ConfigPage and SimulatorPage)
  - `ProfileSelector` dropdown (reuse in ConfigPage and SimulatorPage headers)
- **API Layer**: Centralize API calls in `src/api/` directory:
  - `devicesApi.ts`: GET/PUT device methods
  - `profilesApi.ts`: GET/POST/DELETE/PATCH profile methods
  - `configApi.ts`: GET/PUT config and key mapping methods
  - `metricsApi.ts`: GET metrics and WebSocket subscription

#### Dependency Management
- **Minimize Interdependencies**: ConfigPage doesn't import DevicesPage, they share via common hooks
- **Clear Interfaces**: TypeScript interfaces in `src/types/`:
  - `Device`, `Profile`, `KeyMapping`, `Layer`, `DaemonState`, `LatencyMetrics`

#### Clear Interfaces
- **React Query**: Use `@tanstack/react-query` for server state management
  - Automatic caching and refetching
  - Optimistic updates with rollback on error
  - Shared cache across components (eliminates inconsistencies)
- **Zustand Stores**: Use `zustand` only for UI state (not server state)
  - `useUIStore`: Sidebar collapsed, theme, etc.
  - Avoid duplicating server state in Zustand (use React Query instead)

### Performance

1. **Bundle Size**: ConfigPage with drag-and-drop SHALL NOT exceed current bundle size by >50KB gzipped
   - Use dynamic imports for `@dnd-kit` library: `import('@dnd-kit/core')` only when ConfigPage loads
2. **Latency**: Auto-save debounce SHALL be 500ms (balance between UX and API load)
3. **Rendering**: KeyboardVisualizer with 104 keys SHALL render in <100ms on desktop, <200ms on mobile
4. **WebSocket**: Metrics page SHALL handle >100 events/second without dropped frames (use virtualization)

### Security

1. **Input Sanitization**: All user inputs (device names, profile names, key mappings) SHALL be sanitized before sending to API
2. **XSS Prevention**: All rendered user content SHALL be escaped (React default behavior)
3. **API Authentication**: Future requirement (not in scope for this spec, daemon runs locally)

### Reliability

1. **Error Handling**: All API calls SHALL have timeout (5 seconds) and retry logic (3 attempts with exponential backoff)
2. **Offline Support**: ConfigPage SHALL queue changes when offline, sync when connection restored
3. **State Recovery**: On crash/refresh, page SHALL restore to last known good state via React Query cache persistence

### Usability

1. **Accessibility**: WCAG 2.2 Level AA compliance for all interactive elements (keyboard navigation, screen reader support)
2. **Responsive Design**: All pages SHALL work on mobile (≥375px width), tablet (≥768px), desktop (≥1280px)
3. **Loading States**: All async operations SHALL show loading indicators (skeleton screens, spinners)
4. **Error Messages**: All errors SHALL be user-friendly with actionable next steps (not raw API errors)

### Testing

1. **Unit Tests**: All new components SHALL have ≥80% test coverage (vitest + @testing-library/react)
2. **Integration Tests**: Drag-and-drop flow SHALL have E2E test (Playwright) covering grab → drag → drop → save → verify
3. **Accessibility Tests**: All pages SHALL pass axe-core automated accessibility audit (0 violations)
4. **Visual Regression**: KeyboardVisualizer SHALL have screenshot comparison test (prevent layout breaks)

---

## Sources

Research for 2026 UI/UX design trends:
- [UI trends 2026: top 10 trends your users will love](https://www.uxstudioteam.com/ux-blog/ui-trends-2019)
- [12 UI/UX Design Trends That Will Dominate 2026 (Data-Backed)](https://www.index.dev/blog/ui-ux-design-trends)
- [Drag-and-Drop UX: Guidelines and Best Practices](https://smart-interface-design-patterns.com/articles/drag-and-drop-ux/)
- [UI/UX Design Trends to Watch Out for in 2026 | Big Human](https://www.bighuman.com/blog/top-ui-ux-design-trends)
