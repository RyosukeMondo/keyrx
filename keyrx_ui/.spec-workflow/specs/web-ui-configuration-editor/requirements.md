# Requirements Document: Web UI Configuration Editor

## Introduction

KeyRx2 v1.0 delivers a complete CLI for keyboard configuration management. This specification adds a **world-class web-based visual configuration editor** as an optional interface for users who prefer visual tools over command-line interaction.

**Key Principle**: The web UI is a **thin visual layer** over the CLI. It provides no additional functionality - every UI action maps to existing CLI commands. The focus is on exceptional UI/UX design, accessibility, and visual polish.

**Foundation**: This spec builds on the stable CLI API from `web-ui-ux-comprehensive` spec (v1.0). All business logic remains in the CLI; the web UI only handles presentation and user interaction.

## Alignment with Product Vision

From `product.md`:
- **Browser-based UI**: Optional WASM-based configuration editor
- **Deterministic Testing**: UI state changes testable via API endpoint verification
- **CLI First, GUI Later**: Web UI comes after CLI v1.0 is complete and stable

From `tech.md`:
- **Zero Server Dependency**: UI served from embedded static files
- **WASM Integration**: keyrx_core compiled to WASM for client-side simulation preview

## Requirements

### Requirement 1: Visual Design System

**User Story:** As a user, I want a cohesive, professional visual design that feels polished and modern.

#### Acceptance Criteria

1. WHEN app loads THEN all components SHALL follow consistent color palette (primary, secondary, background, text, error, success, warning)
2. WHEN viewing any text THEN typography SHALL use Inter font family with clear hierarchy (headings, body, code, labels)
3. WHEN viewing spacing THEN all components SHALL use 4px base grid (8px, 16px, 24px, 32px, 48px)
4. WHEN hovering interactive elements THEN visual feedback SHALL be immediate (<16ms) with consistent hover states
5. WHEN clicking buttons THEN ripple effect SHALL provide tactile feedback
6. WHEN viewing borders THEN consistent border-radius SHALL be 4px (small), 8px (medium), 12px (large)
7. WHEN viewing shadows THEN 3 levels SHALL be available (subtle, medium, prominent) for depth hierarchy

#### Design Tokens

**Colors**:
- Primary: #3B82F6 (blue-500)
- Primary-hover: #2563EB (blue-600)
- Secondary: #8B5CF6 (violet-500)
- Background: #0F172A (slate-900)
- Surface: #1E293B (slate-800)
- Border: #334155 (slate-700)
- Text-primary: #F1F5F9 (slate-100)
- Text-secondary: #94A3B8 (slate-400)
- Success: #10B981 (green-500)
- Error: #EF4444 (red-500)
- Warning: #F59E0B (amber-500)

**Typography**:
- Font family: 'Inter', system-ui, sans-serif
- Headings: 24px/32px/16px (h1/h2/h3)
- Body: 14px
- Small: 12px
- Code: 'JetBrains Mono', monospace, 13px

**Spacing**:
- xs: 4px, sm: 8px, md: 16px, lg: 24px, xl: 32px, 2xl: 48px

#### Input Validation

8. WHEN color tokens are modified THEN WCAG AA contrast ratio SHALL be maintained (4.5:1 for text)
9. WHEN font sizes are changed THEN minimum 12px SHALL be enforced for readability

#### Error Scenarios

10. WHEN custom theme is invalid THEN system SHALL fall back to default dark theme with warning
11. WHEN font fails to load THEN system SHALL use system font stack as fallback

---

### Requirement 2: Responsive Design

**User Story:** As a user, I want the UI to work perfectly on desktop, tablet, and mobile devices.

#### Acceptance Criteria

1. WHEN viewport is ≥1280px (desktop) THEN sidebar navigation SHALL be expanded by default
2. WHEN viewport is 768-1279px (tablet) THEN sidebar SHALL be collapsible with hamburger menu
3. WHEN viewport is <768px (mobile) THEN bottom tab navigation SHALL replace sidebar
4. WHEN device has touch input THEN touch targets SHALL be ≥44px for accessibility
5. WHEN viewport width changes THEN layout SHALL reflow smoothly without content clipping
6. WHEN keyboard visualizer exceeds viewport THEN horizontal scroll SHALL be available
7. WHEN orientation changes THEN layout SHALL adapt within 100ms

#### Breakpoints

- Mobile: 0-767px
- Tablet: 768-1279px
- Desktop: 1280px+

#### Input Validation

8. WHEN viewport <320px THEN warning SHALL display "Minimum 320px width required"
9. WHEN viewport height <400px THEN warning SHALL suggest landscape orientation

#### Error Scenarios

10. WHEN browser doesn't support CSS Grid THEN fallback flexbox layout SHALL be used
11. WHEN viewport resize causes layout shift THEN CLS (Cumulative Layout Shift) SHALL be <0.1

---

### Requirement 3: Accessibility (WCAG 2.1 Level AA)

**User Story:** As a user with disabilities, I want the UI to be fully accessible via keyboard, screen reader, and assistive technologies.

#### Acceptance Criteria

1. WHEN user presses Tab THEN focus indicator SHALL be visible with 2px outline at all times
2. WHEN user presses Escape in dialog THEN dialog SHALL close and focus SHALL return to trigger element
3. WHEN screen reader is active THEN all interactive elements SHALL have aria-label or aria-labelledby
4. WHEN images are present THEN alt text SHALL describe content (or alt="" for decorative)
5. WHEN using keyboard THEN all functionality SHALL be accessible (no mouse-only features)
6. WHEN errors occur THEN error messages SHALL be announced to screen readers via aria-live="assertive"
7. WHEN form submitted THEN success/error SHALL be announced and focus SHALL move to result

#### Keyboard Navigation

- Tab/Shift+Tab: Navigate forward/backward
- Enter/Space: Activate buttons
- Escape: Close modals/dropdowns
- Arrow keys: Navigate within lists/dropdowns
- Home/End: Jump to first/last item

#### Input Validation

8. WHEN aria-label exceeds 100 characters THEN warning SHALL suggest using aria-describedby
9. WHEN heading hierarchy skips levels THEN warning SHALL be logged (e.g., h1 → h3)

#### Error Scenarios

10. WHEN focus is lost (trapped) THEN focus SHALL reset to document body with warning
11. WHEN color-only conveyed information THEN additional text/icon SHALL be provided

---

### Requirement 4: Performance and Animation

**User Story:** As a user, I want the UI to feel fast and responsive with smooth animations.

#### Acceptance Criteria

1. WHEN app loads THEN First Contentful Paint (FCP) SHALL be <1.5 seconds
2. WHEN app becomes interactive THEN Time to Interactive (TTI) SHALL be <3.0 seconds
3. WHEN navigating between pages THEN transition SHALL complete in <200ms
4. WHEN scrolling THEN frame rate SHALL maintain 60fps (no dropped frames)
5. WHEN animations run THEN CSS transform/opacity SHALL be used (no layout/paint properties)
6. WHEN user prefers reduced motion THEN animations SHALL be disabled or simplified
7. WHEN clicking interactive elements THEN visual feedback SHALL appear within 16ms (1 frame)

#### Performance Budget

- Initial JS bundle: ≤250KB gzipped
- Initial CSS: ≤50KB gzipped
- Largest Contentful Paint (LCP): <2.5s
- Cumulative Layout Shift (CLS): <0.1
- First Input Delay (FID): <100ms

#### Input Validation

9. WHEN bundle size exceeds budget THEN build SHALL fail with size breakdown
10. WHEN animation duration exceeds 500ms THEN warning SHALL suggest performance optimization

#### Error Scenarios

11. WHEN FCP >3s THEN loading skeleton SHALL display with progress indicator
12. WHEN script loading fails THEN graceful error page SHALL display with retry option

---

### Requirement 5: User Flows - Device Management

**User Story:** As a user, I want to easily view, rename, and configure detected keyboard devices.

#### Acceptance Criteria

1. WHEN dashboard loads THEN all connected devices SHALL be visible in device list
2. WHEN clicking device card THEN device details SHALL expand inline
3. WHEN clicking "Rename" button THEN inline text input SHALL appear with current name pre-filled
4. WHEN pressing Enter in rename input THEN API call SHALL save name and update display
5. WHEN renaming fails THEN error SHALL display inline under input field
6. WHEN toggling scope (global/device-specific) THEN toggle switch SHALL provide immediate visual feedback
7. WHEN clicking "Forget Device" THEN confirmation dialog SHALL appear with device name

#### User Flow Diagram

```
[Dashboard] → [Device List] → [Click Device Card] → [Expanded Details]
                                                         ↓
                                                    [Rename Button]
                                                         ↓
                                                    [Inline Input]
                                                         ↓
                                              [Press Enter / Click Save]
                                                         ↓
                                            [API Call: keyrx devices rename]
                                                         ↓
                                          [Success: Update Display] or [Error: Show Message]
```

#### Input Validation

8. WHEN device name input is empty THEN save button SHALL be disabled
9. WHEN device name exceeds 64 characters THEN input SHALL truncate and show error

#### Error Scenarios

10. WHEN rename API fails THEN input SHALL revert to previous value with error message
11. WHEN device disconnects during rename THEN warning SHALL display "Device no longer connected"

---

### Requirement 6: User Flows - Profile Management

**User Story:** As a user, I want to create, switch, and manage keyboard profiles visually.

#### Acceptance Criteria

1. WHEN viewing profiles page THEN all profiles SHALL be displayed as cards with name, layer count, modified date
2. WHEN clicking "Create Profile" button THEN modal SHALL open with name input and template selector
3. WHEN selecting template (Blank, QMK-style) THEN preview SHALL show example layout
4. WHEN clicking "Activate" on profile card THEN loading indicator SHALL show during compilation
5. WHEN activation succeeds THEN profile card SHALL show "Active" badge and green checkmark
6. WHEN activation fails THEN error modal SHALL display compilation errors with line numbers
7. WHEN clicking "Delete" THEN confirmation dialog SHALL show with profile name and warning

#### User Flow Diagram

```
[Profiles Page] → [Create Profile Button] → [Modal Opens]
                                                 ↓
                                       [Enter Name + Select Template]
                                                 ↓
                                            [Create Button]
                                                 ↓
                                    [API Call: keyrx profiles create]
                                                 ↓
                              [Success: New Card Appears] or [Error: Show Message]

[Profile Card] → [Activate Button] → [Loading Indicator]
                                           ↓
                              [API Call: keyrx profiles activate]
                                           ↓
                          [Compile Progress Bar: 0-100%]
                                           ↓
                      [Success: Badge Updates] or [Error: Modal with Compiler Errors]
```

#### Input Validation

8. WHEN profile name is empty THEN create button SHALL be disabled
9. WHEN profile name contains invalid characters THEN error SHALL display with allowed characters

#### Error Scenarios

10. WHEN compilation timeout (>30s) THEN error SHALL display "Compilation timed out. Check configuration syntax."
11. WHEN profile limit (100) reached THEN create button SHALL be disabled with tooltip explaining limit

---

### Requirement 7: User Flows - Keyboard Configuration

**User Story:** As a user, I want to visually configure key mappings by clicking keys on a keyboard layout.

#### Acceptance Criteria

1. WHEN viewing config page THEN keyboard layout SHALL render based on selected layout preset (ANSI, ISO, JIS, HHKB)
2. WHEN hovering over key THEN key SHALL highlight with tooltip showing current mapping
3. WHEN clicking key THEN configuration dialog SHALL open centered on screen
4. WHEN selecting "Simple Remap" THEN key picker SHALL allow selecting any key
5. WHEN selecting "Tap-Hold" THEN two key pickers SHALL appear (tap action, hold action) with threshold input
6. WHEN selecting "Macro" THEN sequence editor SHALL allow adding key events with timing
7. WHEN clicking "Save" in dialog THEN API call SHALL update configuration and auto-compile

#### User Flow Diagram

```
[Config Page] → [Keyboard Layout Rendered] → [Hover Key] → [Tooltip: Current Mapping]
                                                  ↓
                                            [Click Key]
                                                  ↓
                                    [Config Dialog Opens]
                                                  ↓
                          [Action Type Selector: Simple/Tap-Hold/Macro/Layer]
                                                  ↓
                                    [Action-Specific Form Appears]
                                                  ↓
                                         [Fill Form Fields]
                                                  ↓
                                           [Save Button]
                                                  ↓
                              [API Call: keyrx config set-key]
                                                  ↓
                                [Auto-compile: Inline Progress]
                                                  ↓
                          [Success: Key Updates Visually] or [Error: Show Compiler Message]
```

#### Input Validation

8. WHEN tap-hold threshold <10ms THEN error SHALL display "Minimum 10ms threshold"
9. WHEN tap-hold threshold >2000ms THEN warning SHALL display "Threshold unusually high (>2s)"
10. WHEN macro sequence exceeds 100 steps THEN warning SHALL display "Large macro may impact performance"

#### Error Scenarios

11. WHEN configuration creates circular dependency THEN error SHALL display with explanation
12. WHEN key mapping conflicts with system shortcuts THEN warning SHALL notify user

---

### Requirement 8: Real-Time Preview and Simulation

**User Story:** As a user, I want to test my configuration changes with visual feedback before applying them to my actual keyboard.

#### Acceptance Criteria

1. WHEN viewing config page THEN "Preview Mode" toggle SHALL be available
2. WHEN enabling preview mode THEN keyboard layout SHALL become interactive simulator
3. WHEN clicking/typing keys in simulator THEN output SHALL display in real-time log panel
4. WHEN tap-hold is configured THEN simulator SHALL show timer countdown on key hold
5. WHEN macro is triggered THEN simulator SHALL show sequence playback animation
6. WHEN layer is activated THEN keyboard SHALL visually update to show active layer keys
7. WHEN reset button is clicked THEN simulator state SHALL return to initial state

#### Simulator Display

```
┌─────────────────────────────────────────────────┐
│ Keyboard Simulator [Preview Mode: ON]    [×]   │
├─────────────────────────────────────────────────┤
│                                                 │
│  Active Layer: MD_00 (Base)                    │
│  Modifiers: Ctrl ✓  Shift   Alt   Gui          │
│                                                 │
│  ┌──────────────────────────────────┐          │
│  │  [Keyboard Layout Visualization] │          │
│  │  (Keys change color when pressed)│          │
│  └──────────────────────────────────┘          │
│                                                 │
│  Event Log (last 10):                          │
│  ┌──────────────────────────────────┐          │
│  │ 14:32:01  Press   CapsLock       │          │
│  │ 14:32:01  →Wait   200ms (hold)   │          │
│  │ 14:32:01  Output  Ctrl (hold)    │          │
│  │ 14:32:02  Press   A → Output A   │          │
│  │ 14:32:02  Release CapsLock       │          │
│  │ 14:32:02  Output  Release Ctrl   │          │
│  └──────────────────────────────────┘          │
│                                                 │
│  [Reset Simulator]  [Copy Event Log]           │
└─────────────────────────────────────────────────┘
```

#### Input Validation

8. WHEN simulator runs for >60 seconds THEN auto-pause SHALL engage with notification
9. WHEN event log exceeds 1000 events THEN oldest SHALL be removed (FIFO)

#### Error Scenarios

10. WHEN WASM fails to load THEN simulator SHALL be disabled with error message
11. WHEN configuration is invalid THEN simulator SHALL display "Fix errors before preview"

---

## Non-Functional Requirements

### Browser Compatibility

- **Required**: Chrome/Edge 90+, Firefox 88+, Safari 14+
- **Mobile**: iOS Safari 14+, Chrome Android 90+
- **Technologies**: ES2020, CSS Grid, WebAssembly, Async/Await

### Security

- **No Authentication Required**: UI is local-only (localhost:9867)
- **CORS**: Same-origin only (no external API calls)
- **CSP**: Content Security Policy prevents inline scripts
- **No Data Collection**: Zero telemetry or analytics

### Internationalization (Future)

- English (en-US) - v1.1 release
- Localization framework ready for future languages
- RTL support for Arabic/Hebrew (future)

---

## Success Criteria

This spec is successful when:
- ✅ Users with zero CLI knowledge can configure keyboards via UI
- ✅ WCAG 2.1 AA compliance verified via automated tools (axe, Lighthouse)
- ✅ 60fps smooth animations on low-end devices (verified via Chrome DevTools)
- ✅ Keyboard-only navigation works for 100% of functionality
- ✅ Visual design receives positive feedback (professional, modern, polished)
- ✅ UI loads and becomes interactive in <3 seconds on 3G connection
