# Requirements Document

## Introduction

This specification addresses critical UX inconsistencies and missing functionality in the KeyRX web UI that prevent users from effectively managing keyboard configurations. Based on user feedback and 2025 UI/UX design trends, this refinement focuses on:

1. **Data Consistency**: Eliminating inconsistent device counts and disappearing state
2. **Configuration Persistence**: Auto-saving user preferences (layout selections, profile activations)
3. **Profile-Config Integration**: Connecting profiles to actual .rhai/.krx configuration files
4. **Visual Configuration Editor**: Replacing WASM-based config editor with QMK-style drag-and-drop key assignment
5. **Contextual Information**: Displaying active profile across all relevant pages

**Value to Users:**
- **Reduced Confusion**: Consistent data display across all pages
- **Faster Workflow**: Auto-save eliminates manual save operations
- **Better Mental Model**: Clear profile → rhai → krx → daemon relationship
- **Easier Configuration**: Visual drag-drop editor matches QMK/VIA user expectations
- **Complete Visibility**: Always know which profile is active and running

## Alignment with Product Vision

This feature supports multiple KeyRX product principles:

**AI-First Verification** (Product Principle #1):
- Auto-save creates immediate feedback loops for AI agents to verify changes
- Structured profile-config relationship enables deterministic testing

**Single Source of Truth** (Product Principle #1):
- Profile activation triggers daemon reload with specific .krx file
- UI always reflects backend state (no drift between pages)

**Observability & Controllability** (Product Principle #3):
- Metrics page shows current active profile for correlation with performance
- Configuration editor shows device-specific vs global mappings clearly

**User Experience Metrics** (Success Metrics):
- Configuration Change Time: <5 seconds from edit to live deployment (improved by auto-save)
- Error Detection: Configuration errors caught in visual editor before compilation

## Requirements

### Requirement 1: Consistent Device Count Display

**User Story:** As a user, I want to see the same device count across Dashboard and Devices pages, so that I can trust the information displayed.

#### Acceptance Criteria

1. WHEN I view the Dashboard THEN the "Connected Devices" count SHALL match the number of devices shown in Device List
2. WHEN a device is connected or disconnected THEN both Dashboard and Devices page SHALL update within 1 second
3. WHEN I navigate between Dashboard and Devices pages THEN the device count SHALL remain consistent
4. WHEN the backend returns device data THEN all UI components SHALL use a single source of truth (React Query cache)

### Requirement 2: Auto-Save Device Layout Selection

**User Story:** As a user, I want my device layout selection (JIS 109, ANSI 104, etc.) to be saved automatically, so that I don't lose my configuration when navigating away.

#### Acceptance Criteria

1. WHEN I select a layout from the dropdown THEN the system SHALL save the selection to the backend within 500ms
2. WHEN the save operation completes THEN the UI SHALL show a brief success indicator (e.g., checkmark icon)
3. IF the save operation fails THEN the system SHALL show an error message AND revert to the previous layout
4. WHEN I navigate away and return to Devices page THEN the layout selection SHALL persist
5. WHEN I refresh the page THEN the layout selection SHALL be restored from backend

### Requirement 3: Persistent Profile Activation

**User Story:** As a user, I want the active profile to remain active after I activate it, so that I know which configuration is running.

#### Acceptance Criteria

1. WHEN I click "Activate" on a profile THEN the [Active] badge SHALL appear and remain visible
2. WHEN profile activation completes THEN the backend SHALL persist the active state to disk
3. WHEN I navigate between pages THEN the active profile indicator SHALL remain consistent
4. WHEN I refresh the browser THEN the correct active profile SHALL be indicated
5. WHEN profile activation succeeds THEN the daemon SHALL reload with the corresponding .krx file
6. IF the daemon fails to load the .krx file THEN the UI SHALL show an error AND the profile SHALL NOT be marked active

### Requirement 4: Profile-to-Configuration File Mapping

**User Story:** As a user, I want profiles to represent actual .rhai configuration files, so that activating a profile loads my custom keyboard mappings in the daemon.

#### Acceptance Criteria

1. WHEN I create a profile THEN the system SHALL create a corresponding .rhai file in `~/.config/keyrx/profiles/[profile-name].rhai`
2. WHEN I activate a profile THEN the system SHALL compile the .rhai file to .krx AND reload the daemon
3. WHEN compilation fails THEN the system SHALL show the error message AND NOT activate the profile
4. WHEN I edit a profile's configuration THEN changes SHALL be saved to the corresponding .rhai file
5. WHEN I delete a profile THEN the system SHALL delete the .rhai and .krx files
6. WHEN I duplicate a profile THEN the system SHALL copy the source .rhai file to a new file
7. IF a .rhai file is missing for an existing profile THEN the system SHALL show a warning and offer to recreate it

### Requirement 5: Visual Configuration Editor with Drag-and-Drop

**User Story:** As a user, I want a visual drag-and-drop configuration editor similar to QMK Configurator, so that I can easily assign keys without writing Rhai code.

#### Acceptance Criteria

1. WHEN I open the Config page THEN the system SHALL display a keyboard layout visualization
2. WHEN I select a device from the device list THEN the keyboard SHALL show device-specific key mappings
3. WHEN I toggle "Global" mode THEN the keyboard SHALL show global key mappings (applicable to all devices)
4. WHEN I click on a key THEN a popup SHALL appear with key assignment options (VK_*, MD_*, LK_*, layers, macros)
5. WHEN I drag a key from the available keys palette THEN I SHALL be able to drop it onto any keyboard key
6. WHEN I drop a key THEN the assignment SHALL be saved automatically to the profile's .rhai file
7. WHEN I assign a key THEN the visual keyboard SHALL update immediately to reflect the new mapping
8. WHEN I remove a key assignment THEN the system SHALL restore the default mapping
9. WHEN I switch layers (e.g., base → vim) THEN the visual keyboard SHALL show that layer's mappings
10. WHEN I save changes THEN the system SHALL validate via WASM before persisting to .rhai file
11. IF validation fails THEN the system SHALL show errors and prevent saving

### Requirement 6: Move WASM to Keyboard Simulator Page

**User Story:** As a user, I want WASM-based simulation on the Keyboard Simulator page, so that I can test my configuration before applying it to the daemon.

#### Acceptance Criteria

1. WHEN I open the Keyboard Simulator page THEN the system SHALL load the WASM module (if not already loaded)
2. WHEN I select a profile from a dropdown THEN the simulator SHALL load that profile's compiled configuration
3. WHEN I press keys on the virtual keyboard THEN the WASM engine SHALL process them using the same logic as the daemon
4. WHEN the simulator processes a key event THEN it SHALL display the output action (key mapped to, modifiers applied, layer switched)
5. WHEN I enable "Real-time Mode" THEN the simulator SHALL show DFA state transitions (Pending → Held → Tapped)
6. WHEN I clear the simulator THEN all state (pressed keys, modifiers, locks) SHALL reset
7. WHEN simulator encounters an error THEN it SHALL display the error message without crashing

### Requirement 7: Display Active Profile in Metrics Page

**User Story:** As a user, I want to see which profile is currently active on the Metrics page, so that I can correlate performance metrics with specific configurations.

#### Acceptance Criteria

1. WHEN I view the Metrics page THEN the system SHALL display the active profile name in the header
2. WHEN the active profile changes THEN the Metrics page SHALL update the displayed profile name within 1 second
3. WHEN no profile is active THEN the system SHALL display "No Active Profile"
4. WHEN I click on the profile name THEN the system SHALL navigate to the Profiles page (optional enhancement)

### Requirement 8: Unified Layout Decision

**User Story:** As a product designer, I want a consistent navigation and page layout strategy aligned with 2025 UI/UX trends, so that users have a coherent experience.

#### Acceptance Criteria

1. WHEN deciding between nested routes (e.g., Config as sub-page of Profiles) vs flat routes THEN the system SHALL use **flat routes** with contextual breadcrumbs
2. WHEN a user selects a profile THEN the profile context SHALL be passed via URL query parameters (e.g., `/config?profile=my-profile`)
3. WHEN navigation includes context THEN breadcrumbs SHALL show the navigation path (e.g., "Profiles → my-profile → Configuration")
4. WHEN pages share context (e.g., active profile) THEN they SHALL use React Query shared cache, not prop drilling

**Rationale** (Based on 2025 UI/UX Research):
- **Flat navigation** (Dashboard, Devices, Profiles, Config, Metrics, Simulator at same level) is trending in 2025 for:
  - **AI-driven personalization**: Context-aware navigation adapts to user role without nested hierarchy
  - **Zero-interface philosophy**: Reduce menu depth, provide direct access to all features
  - **Role-based access**: Easier to show/hide top-level pages based on permissions
- **Contextual breadcrumbs** provide navigational clarity without hierarchy constraints
- **Query parameters** enable deep linking and browser history integration

**Sources:**
- [Dashboard UI Design Principles & Best Practices Guide 2025](https://www.designstudiouiux.com/blog/dashboard-ui-design-guide/)
- [10 UI/UX Design Trends That Will Dominate 2025 & Beyond](https://www.bootstrapdash.com/blog/ui-ux-design-trends)
- [20 Principles Modern Dashboard UI/UX Design for 2025 Success](https://medium.com/@allclonescript/20-best-dashboard-ui-ux-design-principles-you-need-in-2025-30b661f2f795)

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each component has one clear purpose (e.g., DeviceListCard fetches and displays devices only)
- **Modular Design**: Extract shared utilities for:
  - Device list fetching (used by Dashboard and Devices pages)
  - Profile activation logic (used by Profiles and Metrics pages)
  - WASM validation (used by Config and Simulator pages)
- **Dependency Management**: Use React Query for all server state, avoid direct fetch() calls in components
- **Clear Interfaces**: Define TypeScript interfaces for all RPC methods and API responses

### Performance

1. **Auto-Save Debouncing**: Debounce user input by 500ms before triggering save API calls (prevent excessive requests)
2. **Optimistic UI Updates**: Show changes immediately, rollback only if backend fails (perceived performance <100ms)
3. **Real-Time Updates**: WebSocket events SHALL update UI within 1 second of backend state change
4. **WASM Loading**: Load WASM module asynchronously, show loading indicator during initialization
5. **Lazy Loading**: Pages SHALL load on-demand using React.lazy() (already implemented)

### Security

1. **Input Validation**: All user input (device names, profile names, key mappings) SHALL be validated on both frontend and backend
2. **File Path Sanitization**: .rhai file names SHALL be sanitized to prevent directory traversal attacks
3. **WASM Sandboxing**: WASM validation SHALL run in isolated context, cannot access filesystem or network

### Reliability

1. **Error Recovery**: When backend operations fail, UI SHALL:
   - Display user-friendly error message
   - Revert to previous state (for optimistic updates)
   - Offer retry option for transient failures
2. **Data Consistency**: All pages consuming same data (e.g., active profile) SHALL use single React Query cache entry
3. **Graceful Degradation**: If WASM fails to load, Config page SHALL fall back to code editor only (no visual editor)

### Usability

1. **Auto-Save Feedback**: Show non-intrusive save indicators (e.g., "Saving...", "Saved" with checkmark, fades after 2s)
2. **Loading States**: Show skeleton loaders or spinners during data fetching (already partially implemented)
3. **Error Messages**: Use toast notifications for errors (non-blocking, auto-dismiss after 5s)
4. **Keyboard Shortcuts**: Simulator SHALL support keyboard input for testing (already implemented)
5. **Responsive Design**: All pages SHALL work on mobile, tablet, and desktop (already implemented)
6. **Accessibility**: Follow WCAG 2.2 guidelines (2025 trend):
   - All interactive elements keyboard-navigable
   - ARIA labels for icon-only buttons
   - Color contrast ratio ≥4.5:1 for text

### Compatibility

1. **Browser Support**: Chrome 90+, Firefox 88+, Safari 15+ (WASM requirement)
2. **Backend API Version**: Ensure frontend is compatible with daemon API v1.0

### Data Integrity

1. **Profile-Config Consistency**: Profile metadata (name, active state) SHALL always match corresponding .rhai file existence
2. **Atomic Operations**: Profile activation (compile + daemon reload) SHALL be atomic; rollback both if either fails
3. **Conflict Resolution**: If .rhai file modified outside UI, SHALL detect and offer to reload or overwrite

## References

### 2025 UI/UX Trends Applied

1. **AI-Driven Personalization**: Contextual navigation (query params carry profile context across pages)
2. **Real-Time Data**: WebSocket subscriptions keep all pages in sync
3. **Auto-Save Configuration**: Eliminate manual save buttons, follow 2025 trend of "zero friction" UX
4. **Minimalism with Function**: Flat navigation reduces cognitive load, breadcrumbs provide context
5. **Data Transparency**: Metrics page shows active profile, Config page shows global vs device-specific scope

**Sources:**
- [Dashboard UI Design Principles & Best Practices Guide 2025](https://www.designstudiouiux.com/blog/dashboard-ui-design-guide/)
- [10 UI/UX Design Trends That Will Dominate 2025 & Beyond](https://www.bootstrapdash.com/blog/ui-ux-design-trends)
- [20 Principles Modern Dashboard UI/UX Design for 2025 Success](https://medium.com/@allclonescript/20-best-dashboard-ui-ux-design-principles-you-need-in-2025-30b661f2f795)
- [Top Dashboard Design Trends for 2025](https://fuselabcreative.com/top-dashboard-design-trends-2025/)
- [QMK Configurator](https://config.qmk.fm/) - Visual key assignment reference

### KeyRX Steering Documents

- **Product Vision**: `.spec-workflow/steering/product.md`
- **Technology Stack**: `.spec-workflow/steering/tech.md`
- **Project Structure**: `.spec-workflow/steering/structure.md`
