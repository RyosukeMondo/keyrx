# UAT UI Fixes - Tasks

## Overview
Fix UI issues identified during User Acceptance Testing for Dashboard, Devices, Profiles, Config, and Layers pages.

---

## Task 1: Dashboard - Add Virtual/Physical Device Indicator

- [x] 1.1 Add device type detection to API/types
  - File: `src/types/index.ts`, `src/api/devices.ts`
  - Add `isVirtual: boolean` field to DeviceEntry interface
  - Detection logic: device name starts with "keyrx" = virtual (uinput-created)
  - Purpose: Distinguish daemon-created virtual keyboards from physical hardware
  - _Leverage: src/types/index.ts, src/api/devices.ts_
  - _Requirements: Dashboard shows clear indicator for virtual vs physical device_
  - _Prompt: Implement the task for spec uat-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend TypeScript Developer | Task: Add isVirtual boolean field to DeviceEntry interface and implement detection in API layer. Virtual devices are those with name "keyrx" (the daemon's uinput device). Update fetchDevices to populate this field. | Restrictions: Do not modify backend API, derive from existing device name field, maintain backward compatibility | Success: DeviceEntry has isVirtual field, fetchDevices correctly identifies virtual device | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 1.2 Add virtual device badge to Dashboard
  - File: `src/pages/DashboardPage.tsx`, `src/components/StateIndicatorPanel.tsx`
  - Add visual indicator (icon/badge) showing "Virtual" or "Physical" for each device
  - Virtual: show computer/software icon with "Virtual" label
  - Physical: show keyboard hardware icon with "Hardware" label
  - Purpose: Users can distinguish daemon output device from real keyboards
  - _Leverage: src/pages/DashboardPage.tsx, existing icon patterns_
  - _Requirements: Clear visual distinction between virtual and physical devices_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React/UI Developer | Task: Add device type badges to device cards on Dashboard. Use distinct icons/colors - Virtual (purple/software icon), Physical (gray/keyboard icon). Show badge next to device name. | Restrictions: Follow existing badge styling patterns, ensure accessibility with aria-labels | Success: Each device clearly shows if it's virtual or physical hardware | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 2: Devices Page - Replace Delete with Enable/Disable Toggle

- [x] 2.1 Add enabled/disabled state to device management
  - File: `src/types/index.ts`, `src/api/devices.ts`, `src/hooks/useDevices.ts`
  - Add `enabled: boolean` field to DeviceEntry (default: true)
  - Add `setDeviceEnabled(id, enabled)` API function
  - Persist enabled state to backend or localStorage
  - Purpose: Allow users to hide devices without "forgetting" them
  - _Leverage: src/types/index.ts, src/api/devices.ts, src/hooks/useDevices.ts_
  - _Requirements: Devices can be disabled without deletion, persist across refresh_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: Full-stack Developer | Task: Add enabled boolean field to DeviceEntry. Create setDeviceEnabled API call (PUT /api/devices/{id}/enabled). Add useSetDeviceEnabled hook with optimistic updates. If backend doesn't support, use localStorage fallback. | Restrictions: Maintain backward compatibility, handle offline gracefully | Success: Device enabled state persists across page refresh | After completion: Use log-implementation tool, mark [x] complete_

- [x] 2.2 Replace delete button with enable/disable toggle
  - File: `src/pages/DevicesPage.tsx`
  - Replace ✕ delete button with toggle switch
  - Disabled devices appear grayed out with reduced opacity
  - Remove confirmation modal (no longer needed for toggle)
  - Add visual indicator when device is disabled
  - Purpose: Intuitive on/off control without permanent deletion
  - _Leverage: src/pages/DevicesPage.tsx DeviceRow component_
  - _Requirements: Toggle switch UI, grayed out disabled devices_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React UI Developer | Task: Replace delete button (✕) with toggle switch in DeviceRow. When disabled: row has opacity:0.5, gray overlay, "(Disabled)" badge. Toggle triggers setDeviceEnabled. Remove forget confirmation modal. | Restrictions: Maintain existing device info display, ensure keyboard accessible toggle | Success: Toggle works, disabled devices visually distinct, no delete functionality | After completion: Use log-implementation tool, mark [x] complete_

- [x] 2.3 Filter disabled devices from Config page
  - File: `src/pages/ConfigPage.tsx`, `src/components/DeviceSelector.tsx`
  - Don't show disabled devices in DeviceSelector dropdown
  - Don't include disabled devices in merged device list
  - Purpose: Disabled devices shouldn't appear in configuration
  - _Leverage: src/pages/ConfigPage.tsx, src/components/DeviceSelector.tsx_
  - _Requirements: Disabled devices not shown in config page_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React Developer | Task: Filter out devices where enabled===false from ConfigPage mergedDevices list and DeviceSelector dropdown. Add filter before mapping. | Restrictions: Keep Rhai-defined disconnected devices, only filter user-disabled devices | Success: Disabled devices don't appear in config page device selector | After completion: Use log-implementation tool, mark [x] complete_

- [x] 2.4 Add unit tests for device enable/disable
  - File: `src/__tests__/DevicesPage.test.tsx` (create or extend)
  - Test toggle functionality
  - Test disabled device visual state
  - Test filtering in ConfigPage
  - Purpose: Prevent regression of enable/disable feature
  - _Leverage: tests/testUtils.tsx, existing test patterns_
  - _Requirements: Test coverage for new functionality_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: QA Engineer | Task: Write unit tests for device enable/disable: 1) Toggle changes enabled state, 2) Disabled devices have opacity styling, 3) Disabled devices filtered from ConfigPage. Use renderWithProviders and mock API. | Restrictions: Follow existing test patterns, use React Testing Library | Success: All tests pass, covers happy path and edge cases | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 3: Profiles Page - Improve UX and Active Indicator

- [x] 3.1 Add inline editing for profile name and description
  - File: `src/components/ProfileCard.tsx`
  - Show name and description prominently at top of card
  - Click on name/description to edit inline (contenteditable or input toggle)
  - Auto-save on blur with debounce
  - Remove separate Edit button
  - Purpose: Streamline profile editing experience
  - _Leverage: src/components/ProfileCard.tsx, existing auto-save patterns_
  - _Requirements: Inline edit for name/description, remove Edit button_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React UI Developer | Task: In ProfileCard, make name and description inline-editable. Click shows input field, blur saves with 500ms debounce. Remove Edit button. Show pencil icon on hover to indicate editability. Add API call for profile rename if not exists. | Restrictions: Handle empty values gracefully, prevent XSS with proper escaping | Success: Name/description editable inline, saves automatically, Edit button removed | After completion: Use log-implementation tool, mark [x] complete_

- [x] 3.2 Add clear active profile indicator
  - File: `src/components/ProfileCard.tsx`, `src/pages/ProfilesPage.tsx`
  - Active profile has prominent visual distinction (thick colored border, background tint)
  - Show "ACTIVE" badge prominently at top
  - Only one profile can be active at a time (mutex)
  - Inactive profiles have "Activate" button, active profile has no button or "Active" text
  - Purpose: Clear at-a-glance identification of current active profile
  - _Leverage: src/components/ProfileCard.tsx existing active styling_
  - _Requirements: Unmistakable active indicator, exclusive activation_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: UI/UX Developer | Task: Enhance active profile indication: 4px left border in primary color, subtle background gradient, large "ACTIVE" badge at top. Hide Activate button on active profile. Ensure activation is mutex (API should handle). | Restrictions: Maintain accessibility contrast, work in both light/dark themes | Success: Active profile instantly recognizable, Activate button only on inactive profiles | After completion: Use log-implementation tool, mark [x] complete_

- [x] 3.3 Add activation success notification
  - File: `src/pages/ProfilesPage.tsx`, add notification component if needed
  - Show toast/snackbar notification on successful activation
  - Message: "Profile '{name}' activated successfully!"
  - Auto-dismiss after 5 seconds
  - Purpose: Confirm to user that activation was successful
  - _Leverage: existing notification patterns or add new toast component_
  - _Requirements: Visual confirmation of profile activation_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React Developer | Task: Add toast notification on successful profile activation. Use existing notification system or create simple toast component. Message: "Profile '{name}' is now active!" with check icon. Auto-dismiss 5s. Position: top-right. | Restrictions: Don't add heavy toast library, keep lightweight | Success: Toast appears on activation, auto-dismisses, shows profile name | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 4: Config Page - Fix Navigation Menu Sync

- [x] 4.1 Sync sidebar menu when navigating via file path link
  - File: `src/App.tsx`, `src/pages/ConfigPage.tsx`
  - When clicking profile file path on ProfilesPage to navigate to ConfigPage
  - Update sidebar navigation to highlight "Config" menu item
  - Pass profile selection via URL param or state
  - Purpose: Maintain UI consistency during navigation
  - _Leverage: src/App.tsx router, navigation state management_
  - _Requirements: Sidebar highlights correct menu after navigation_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React Router Developer | Task: When navigating from ProfilesPage to ConfigPage via file path link, ensure sidebar "Config" item is highlighted. Use React Router's useLocation or NavLink active state. Pass selectedProfile via URL query param (?profile=name). | Restrictions: Don't break existing navigation, maintain browser history | Success: Clicking file path navigates to Config with correct menu highlight | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 5: Config Page - Fix RPC Error

- [ ] 5.1 Debug and fix "Invalid client RPC message" error
  - File: `src/hooks/useProfileConfig.ts`, `src/api/profiles.ts`
  - Error: Invalid input - "content" field expected object but got wrong type
  - Investigate WebSocket/API message format
  - Fix request payload structure to match server expectations
  - Purpose: Eliminate error when saving configuration
  - _Leverage: Browser DevTools, src/hooks/useProfileConfig.ts_
  - _Requirements: No RPC errors when saving config_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: Full-stack Debug Specialist | Task: Debug RPC error. Check useSetProfileConfig mutation payload structure. Server expects { content: object } but receiving wrong format. Trace from API call through WebSocket. Fix payload serialization. | Restrictions: Don't change server API contract, fix client-side only | Success: Saving configuration works without RPC error | After completion: Use log-implementation tool, mark [x] complete_

- [ ] 5.2 Add test to prevent RPC format regression
  - File: `src/__tests__/useProfileConfig.test.ts` (create)
  - Test that setProfileConfig sends correct payload format
  - Mock WebSocket/fetch and verify request body structure
  - Purpose: Prevent future RPC format regressions
  - _Leverage: tests/testUtils.tsx, MSW or fetch mocking_
  - _Requirements: Test coverage for RPC payload format_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: QA Engineer | Task: Create test for useSetProfileConfig hook. Mock fetch/WebSocket. Assert request body has correct structure { content: {...} }. Test both success and error scenarios. | Restrictions: Use existing test infrastructure, mock network calls | Success: Test verifies payload format, catches format regressions | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 6: Layers Section - Show All 255 Modifiers

- [ ] 6.1 Update LayerSwitcher to show all MD_00 to MD_FF layers
  - File: `src/components/LayerSwitcher.tsx`
  - Change from hardcoded 6 layers to full 256 (0x00-0xFF)
  - Use vertical scrollable layout
  - Show layer names: "Base", "MD_00", "MD_01", ... "MD_FF"
  - Hex display format for clarity
  - Purpose: Support all 255 modifier layers as designed
  - _Leverage: src/components/LayerSwitcher.tsx_
  - _Requirements: All 256 layers visible in scrollable vertical layout_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: React UI Developer | Task: Refactor LayerSwitcher to show all 256 layers (Base + MD_00 through MD_FF). Use vertical layout with overflow-y:scroll, max-height ~400px. Generate layers programmatically with hex formatting. Add search/filter input for quick navigation. | Restrictions: Maintain existing layer selection logic, ensure smooth scroll performance | Success: All 256 layers displayed vertically, scrollable, searchable | After completion: Use log-implementation tool, mark [x] complete_

---

## Task 7: Config Page - Fix Key Dropdown Empty State

- [ ] 7.1 Populate key dropdown with all available keys
  - File: `src/components/KeyAssignmentPanel.tsx` or related modal component
  - Currently dropdown shows no keys - investigate data source
  - Add all possible key codes from evdev/HID key list
  - Categories: Letters (A-Z), Numbers (0-9), Function (F1-F24), Modifiers, Navigation, Numpad, Special
  - Purpose: Allow users to select any key for remapping
  - _Leverage: src/utils/keyCodeMapping.ts, existing key definitions_
  - _Requirements: All possible keys available in dropdown_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: Frontend Developer | Task: Fix empty key dropdown in config modal. Ensure keyCodeMapping.ts exports complete key list. Categories: Letters, Numbers, Function, Modifiers (Shift/Ctrl/Alt/Meta), Navigation (Arrows/Home/End/PgUp/PgDn), Numpad, Special (Tab/Enter/Backspace/Escape/Space/Caps). Verify dropdown component receives and renders all options. | Restrictions: Use existing key code constants, ensure codes match daemon expectations | Success: Dropdown shows all possible keys organized by category | After completion: Use log-implementation tool, mark [x] complete_

- [ ] 7.2 Add tests for key dropdown population
  - File: `src/__tests__/KeyAssignmentPanel.test.tsx` (create or extend)
  - Test that dropdown contains expected key categories
  - Test that selecting a key updates state correctly
  - Purpose: Prevent regression of key dropdown functionality
  - _Leverage: tests/testUtils.tsx_
  - _Requirements: Test coverage for key dropdown_
  - _Prompt: Implement the task for spec uat-ui-fixes: Role: QA Engineer | Task: Write tests for key dropdown: 1) Contains all key categories (Letters, Numbers, Function, etc.), 2) Selection updates form state, 3) Search/filter works if implemented. Verify at least 100 keys available. | Restrictions: Follow existing test patterns | Success: Tests verify complete key list and selection behavior | After completion: Use log-implementation tool, mark [x] complete_

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1.1-1.2 | Virtual/Physical device indicator | types, api, Dashboard |
| 2.1-2.4 | Enable/Disable toggle (replace delete) | Devices, Config, types, tests |
| 3.1-3.3 | Profile inline edit + active indicator + notification | ProfileCard, ProfilesPage |
| 4.1 | Fix navigation menu sync | App.tsx, ConfigPage |
| 5.1-5.2 | Fix RPC error + tests | useProfileConfig, tests |
| 6.1 | Show all 255 MD layers | LayerSwitcher |
| 7.1-7.2 | Fix empty key dropdown + tests | KeyAssignmentPanel, keyCodeMapping |

Total: 14 subtasks across 7 feature areas
