# Requirements Document: Web UI/UX Improvements

## Introduction

This specification defines improvements to the KeyRX Web UI that enhance usability, visual clarity, and workflow efficiency for keyboard configuration. The improvements focus on optimizing screen real estate, providing clearer visual feedback, and enabling more intuitive configuration workflows. These changes exclude the key palette (separate spec) and WASM fixes (separate spec), focusing solely on layout, visual design, and interaction patterns.

The primary value to users is **faster, more intuitive keyboard configuration** through improved layout organization, better visual feedback, and streamlined workflows that reduce cognitive load during complex multi-layer configuration tasks.

## Alignment with Product Vision

This feature directly supports the product vision's emphasis on:

1. **AI Coding Agent First**: Improved UI structure enables AI agents to more easily generate and validate UI-based configurations. Clear visual separation between global and device-specific configurations makes automated testing more straightforward.

2. **Real-Time Simulation & Preview**: Enhanced visual feedback (tooltips, hover states, mapping indicators) improves the "edit-and-preview" workflow by providing instant, clear visual confirmation of configuration changes.

3. **User Experience Metrics**: Reduces "Configuration Change Time" by streamlining the UI layout, eliminating tab switching, and providing side-by-side views for faster navigation between global and device-specific settings.

4. **Professional Power Users**: Addresses the need for "complex multi-layered keyboard configurations" with improved layer visualization, dual-pane layout for managing global vs. device-specific mappings simultaneously, and better visual distinction of mapped keys.

5. **Observability & Controllability**: Clear visual indicators (sync status, mapping types, tooltips) align with the product principle of making "all internal state inspectable" by surfacing configuration state directly in the UI.

## Requirements

### Requirement 1: Narrow Layer Switcher

**User Story:** As a power user configuring multiple layers, I want the layer switcher to occupy minimal horizontal space, so that I have maximum screen real estate for viewing keyboard layouts.

#### Acceptance Criteria

1. WHEN the layer switcher is rendered THEN it SHALL occupy a fixed narrow width of approximately 80px (sufficient for 7-character layer names like "MD_FF")
2. WHEN layer names are displayed THEN they SHALL be fully visible without horizontal scrolling (layer names are predictable: "Base", "MD_00" through "MD_FF")
3. WHEN the layer list exceeds viewport height THEN it SHALL remain vertically scrollable
4. WHEN the narrow layer switcher is implemented THEN existing layer selection functionality SHALL remain unchanged
5. WHEN the layer switcher width changes THEN it SHALL use Tailwind utility classes (e.g., `w-20` or `w-[80px]`) for consistent styling

### Requirement 2: Dual-Pane Layout with Dedicated Layer Switchers

**User Story:** As a user managing both global and device-specific keyboard configurations, I want to view and edit both simultaneously side-by-side, so that I can understand the relationship between global defaults and device overrides without switching contexts.

#### Acceptance Criteria

1. WHEN the ConfigPage is rendered on desktop screens (≥1024px width) THEN it SHALL display two keyboard visualizer panes side-by-side
2. WHEN the left pane is displayed THEN it SHALL show "Global Keys" configuration with its own narrow layer switcher
3. WHEN the right pane is displayed THEN it SHALL show "Device-Specific Keys" configuration with its own narrow layer switcher and device selector
4. WHEN each pane has its own layer switcher THEN layer selection in one pane SHALL NOT affect the other pane's layer state
5. WHEN each pane is displayed THEN it SHALL be independently scrollable if content exceeds viewport height
6. WHEN the global pane is displayed THEN it SHALL have a header labeled "Global Keys"
7. WHEN the device pane is displayed THEN it SHALL have a header with format "Device: [selector dropdown]"
8. WHEN the device selector is rendered THEN it SHALL only appear in the device-specific pane header (not in global pane)
9. WHEN panes are rendered THEN each SHALL have a visually distinct background tint (e.g., slate-50 vs zinc-50) for clear separation
10. WHEN the dual-pane layout is rendered THEN existing KeyboardVisualizer component functionality SHALL remain unchanged

### Requirement 3: Responsive Layout Adaptation

**User Story:** As a user accessing the configuration UI on different devices, I want the layout to adapt to my screen size, so that I can configure keyboards on tablets and mobile devices without usability issues.

#### Acceptance Criteria

1. WHEN screen width is ≥1024px (desktop) THEN the layout SHALL display dual panes side-by-side horizontally
2. WHEN screen width is between 768px and 1023px (tablet) THEN the layout SHALL display panes stacked vertically with a tab switcher to toggle between "Global" and "Device" views
3. WHEN screen width is <768px (mobile) THEN the layout SHALL display a single pane with a toggle button to switch between "Global" and "Device" modes
4. WHEN responsive breakpoints trigger layout changes THEN keyboard visualizer instances SHALL NOT be duplicated (use conditional rendering)
5. WHEN the layout adapts to different screen sizes THEN all keyboard configuration functionality SHALL remain accessible
6. WHEN responsive classes are applied THEN they SHALL use Tailwind responsive prefixes (lg:, md:, sm:)

### Requirement 4: Streamlined Configuration Editor Header

**User Story:** As a user configuring keyboards, I want a clean, uncluttered header with essential controls clearly visible, so that I can focus on configuration tasks without visual noise.

#### Acceptance Criteria

1. WHEN the header is rendered THEN it SHALL display the profile selector on the left side in a compact design
2. WHEN the header is rendered THEN it SHALL display a keyboard layout selector (ANSI/ISO/JIS) in the center
3. WHEN the header is rendered THEN it SHALL display a sync status indicator on the right side
4. WHEN configuration is synchronized with the daemon THEN the sync status indicator SHALL show a green status dot with "Saved" label
5. WHEN configuration is being synchronized THEN the sync status indicator SHALL show a yellow status dot with "Syncing" label
6. WHEN configuration has unsaved changes THEN the sync status indicator SHALL show a red status dot with "Unsaved" label
7. WHEN the header is rendered THEN tab navigation for "Visual Editor" / "Code Editor" SHALL be removed from the header
8. WHEN the header redesign is implemented THEN profile functionality (save/load operations) SHALL remain unchanged
9. WHEN the header is rendered THEN it SHALL maintain WCAG 2.2 Level AA accessibility standards (keyboard navigation, ARIA labels)

### Requirement 5: Collapsible Code Editor Panel

**User Story:** As a user who wants to view the underlying configuration code, I want a collapsible code panel that appears alongside the visual editor, so that I can inspect code without losing context of the visual editor state.

#### Acceptance Criteria

1. WHEN the code panel toggle button is clicked THEN the code editor SHALL slide up from the bottom of the viewport with a smooth animation
2. WHEN the code panel is visible THEN it SHALL occupy 200-400px of vertical height (user-adjustable via resize handle)
3. WHEN the code panel toggle button is in "Show Code" state THEN clicking it SHALL reveal the code panel
4. WHEN the code panel toggle button is in "Hide Code" state THEN clicking it SHALL collapse the code panel
5. WHEN the code panel is displayed THEN the visual editor SHALL remain visible above it (not replaced)
6. WHEN the code panel is displayed THEN it SHALL include a resize handle allowing users to adjust panel height by dragging
7. WHEN the code panel is shown or hidden THEN it SHALL use Tailwind transition classes for smooth animation
8. WHEN the code panel is displayed THEN bidirectional synchronization between code editor and visual editor SHALL continue to function
9. WHEN tab-based navigation is removed THEN the existing SimpleCodeEditor component functionality SHALL remain unchanged
10. WHEN the code panel is toggled THEN the toggle button SHALL be located in the header for easy access

### Requirement 6: Enhanced Key Configuration Modal

**User Story:** As a user configuring individual keys, I want an intuitive modal that clearly shows the key I'm editing, available mapping types, and the result of my configuration, so that I can configure keys quickly without errors.

#### Acceptance Criteria

1. WHEN the key configuration modal opens THEN it SHALL display a prominent header showing the physical key label being configured
2. WHEN mapping type selection is displayed THEN it SHALL use clear tabs or buttons with icons for: Simple, Tap/Hold, Macro, and Layer mappings
3. WHEN a mapping type is selected THEN the modal SHALL display appropriate input fields for that mapping type
4. WHEN a mapping is configured THEN the modal SHALL show a "Preview" section describing the resulting behavior in plain language
5. WHEN the modal is displayed THEN it SHALL include quick-assign buttons for common mappings (e.g., Escape, Enter, Backspace, Delete)
6. WHEN quick-assign buttons are clicked THEN the mapping SHALL be applied immediately without additional input
7. WHEN the modal is displayed THEN save and cancel functionality SHALL remain unchanged
8. WHEN form inputs are invalid THEN existing validation SHALL continue to prevent invalid mappings
9. WHEN the enhanced modal is implemented THEN it SHALL maintain accessibility standards (keyboard navigation, focus management, ARIA labels)

### Requirement 7: Visual Feedback Improvements for Keyboard Visualizer

**User Story:** As a user hovering over and clicking keyboard keys, I want clear visual feedback showing current mappings and interaction states, so that I understand what each key does without opening configuration modals.

#### Acceptance Criteria

1. WHEN a user hovers over a key in the keyboard visualizer THEN a tooltip SHALL appear showing: physical key code, current mapping, and mapping type
2. WHEN a user clicks a key THEN a brief highlight or ripple animation SHALL provide click feedback
3. WHEN a key has a custom mapping THEN it SHALL display a small icon overlay in the corner indicating the mapping type (simple, tap/hold, macro, layer)
4. WHEN a key has no custom mapping (unmapped) THEN it SHALL display with a visually distinct appearance (e.g., dashed border or muted background)
5. WHEN hover states and animations are added THEN component performance SHALL be maintained (use React.memo and avoid unnecessary re-renders)
6. WHEN tooltips and overlays are added THEN they SHALL maintain WCAG 2.2 Level AA accessibility standards (tooltip text readable, sufficient contrast)
7. WHEN the KeyButton component is enhanced THEN existing key click behavior SHALL remain unchanged

### Requirement 8: Layout Component Test Coverage

**User Story:** As a developer maintaining the UI codebase, I want comprehensive tests for the new layout components, so that future changes do not break the responsive layout, collapsible panels, or dual-pane behavior.

#### Acceptance Criteria

1. WHEN layout tests are created THEN they SHALL include a test verifying dual-pane layout renders correctly at desktop breakpoint (≥1024px)
2. WHEN layout tests are created THEN they SHALL include a test verifying single-pane layout renders at tablet/mobile breakpoints (<1024px)
3. WHEN layout tests are created THEN they SHALL include a test verifying code panel toggles between visible and hidden states
4. WHEN layout tests are created THEN they SHALL include a test verifying LayerSwitcher renders at narrow fixed width
5. WHEN responsive breakpoint tests are written THEN they SHALL use matchMedia mock to simulate different screen sizes
6. WHEN layout tests are written THEN they SHALL follow existing test patterns using renderWithProviders from tests/testUtils.tsx
7. WHEN layout tests are executed THEN they SHALL pass in CI/CD pipeline without errors
8. WHEN test coverage is measured THEN layout components SHALL meet minimum 80% line and branch coverage

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each component file (LayerSwitcher, ConfigPage, KeyConfigModal, KeyboardVisualizer) shall have a single, well-defined purpose
- **Modular Design**: Layout components shall be composable and reusable; dual-pane layout logic shall be extracted to separate components if complexity exceeds 50 lines
- **Dependency Management**: UI components shall minimize interdependencies; shared utilities (tooltips, animations) shall be extracted to separate utility modules
- **Clear Interfaces**: Component props shall be well-typed with TypeScript interfaces; prop types shall clearly define required vs. optional inputs

### Performance

- **Render Performance**: Keyboard visualizer components shall maintain ≥30 FPS during hover interactions and animations
- **Component Memoization**: KeyButton and other frequently re-rendered components shall use React.memo to prevent unnecessary re-renders
- **Lazy Loading**: Code editor panel shall lazy-load editor components to reduce initial bundle size
- **Animation Performance**: CSS transitions and animations shall use GPU-accelerated properties (transform, opacity) to maintain 60 FPS

### Security

- **XSS Prevention**: All user-provided configuration data displayed in UI shall be properly sanitized
- **Configuration Validation**: Key mappings entered via modal shall be validated before submission to prevent invalid configurations
- **No Inline Styles**: All styling shall use Tailwind utility classes or CSS modules to maintain Content Security Policy compatibility

### Reliability

- **Graceful Degradation**: Layout shall remain functional if JavaScript animations fail (fallback to instant show/hide)
- **Error Boundaries**: Layout components shall be wrapped in React Error Boundaries to prevent full-page crashes
- **State Consistency**: Layer selection state shall remain consistent between panes; changes in one pane shall not corrupt state in the other

### Usability

- **WCAG 2.2 Level AA Compliance**: All interactive elements shall meet accessibility standards (keyboard navigation, ARIA labels, sufficient color contrast)
- **Keyboard Navigation**: All UI controls (buttons, modals, tabs, toggles) shall be fully keyboard-accessible
- **Focus Management**: Modal open/close shall properly manage focus (focus trap in modal, return focus on close)
- **Responsive Touch Targets**: All clickable elements shall meet minimum touch target size of 44×44px on mobile devices
- **Intuitive Defaults**: Code panel shall default to collapsed state; dual-pane layout shall default to showing both global and first device configurations
