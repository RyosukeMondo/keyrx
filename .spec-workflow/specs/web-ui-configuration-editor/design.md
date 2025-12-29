# Design Document: Web UI Configuration Editor

## Overview

This design spec provides **detailed ASCII art layouts** for a world-class web-based keyboard configuration UI. The UI is a thin visual layer over the CLI API from `web-ui-ux-comprehensive` spec (v1.0).

**Design Philosophy**:
- **Visual First**: Every screen has ASCII art layout before implementation
- **Component-Based**: Reusable components (Button, Input, Card, etc.)
- **Accessibility Built-In**: ARIA labels, keyboard navigation, focus management from day one
- **Dark Theme**: Modern dark UI with high contrast for readability

---

## Dependencies

### Production Dependencies

- **react** 18.2+
  - Latest React with concurrent features, automatic batching
  - Rationale: Industry standard, excellent TypeScript support, concurrent rendering for better UX

- **react-dom** 18.2+
  - React DOM renderer
  - Rationale: Required for React web applications

- **react-router-dom** 6.20+
  - Type-safe routing with data APIs
  - Rationale: Declarative routing, data loading, nested routes, TypeScript support

- **zustand** 4.4+
  - Lightweight state management (3KB vs Redux 20KB)
  - Rationale: Simple API, no boilerplate, TypeScript-first, 93% smaller than Redux

- **tailwindcss** 3.4+
  - Utility-first CSS framework
  - Rationale: Design token integration, small bundle size (only used classes), fast iteration

- **@tanstack/react-query** 5.0+
  - Server state management with caching
  - Rationale: Automatic refetching, cache invalidation, optimistic updates, error handling

### Dev Dependencies

- **vite** 5.0+
  - Fast build tool and dev server
  - Rationale: 10-100x faster than Webpack, native ESM, instant HMR

- **vitest** 1.0+
  - Fast unit testing framework
  - Rationale: Vite-native, 10x faster than Jest, native ESM support, compatible with @testing-library

- **@testing-library/react** 14.0+
  - Component testing utilities
  - Rationale: Accessibility-focused testing, encourages best practices, widely adopted

- **playwright** 1.40+
  - End-to-end and visual regression testing
  - Rationale: Cross-browser support, reliable selectors, screenshot comparison, network mocking

- **@axe-core/react** 4.8+
  - Automated accessibility testing
  - Rationale: WCAG 2.1 AA compliance verification, real-time violation detection

- **eslint** 8.0+ + **prettier** 3.0+
  - Code quality and formatting
  - Rationale: Enforce coding standards, prevent bugs, consistent formatting

- **typescript** 5.0+
  - Static type checking
  - Rationale: Catch errors at compile time, better IDE support, self-documenting code

### Optional Dependencies (for production features)

- **@floating-ui/react** 0.26+
  - Tooltip and dropdown positioning
  - Rationale: Smart positioning, viewport-aware, accessible

- **framer-motion** 10.0+ (optional)
  - Animation library
  - Rationale: Declarative animations, gesture support, reduced motion support

- **react-window** 1.8+
  - Virtual scrolling for event log
  - Rationale: Render 1000s of items without performance degradation

- **recharts** 2.10+ or **chart.js** 4.0+
  - Charting library for latency graph
  - Rationale: SVG-based charts, responsive, accessible

---

## Code Quality Metrics

**Enforced by ESLint, Prettier, and CI:**

- **File size limit**: ≤500 lines (excluding comments and blank lines)
  - If exceeded: Extract sub-components or helper modules
  - Example: KeyboardVisualizer ≤500 lines → extract KeyButton to separate component

- **Function size limit**: ≤50 lines
  - If exceeded: Extract helper functions, apply SLAP (Single Level of Abstraction Principle)
  - Example: Complex form validation → extract to validateProfileName() helper

- **Test coverage**: ≥80% minimum (≥90% for critical components)
  - Critical components: Button, Input, Modal, Dropdown, KeyboardVisualizer, Zustand stores
  - Coverage measured by Vitest with c8

- **Bundle size budget**:
  - Initial JS bundle: ≤250KB gzipped
  - Initial CSS: ≤50KB gzipped
  - Enforced by vite-plugin-compression with build failure on exceed

- **Accessibility**:
  - 0 axe-core violations (automated scan)
  - Lighthouse accessibility score ≥95
  - Manual testing with NVDA/JAWS screen readers

- **Performance**:
  - Lighthouse performance score ≥90
  - All Core Web Vitals in "Good" range (LCP <2.5s, FID <100ms, CLS <0.1)
  - 60fps animations (no dropped frames)

- **Code quality gates** (enforced in CI):
  - ESLint: 0 errors, 0 warnings
  - Prettier: All files formatted
  - TypeScript: Strict mode, 0 errors
  - No console.log in production code (use proper logging)

---

## Test Strategy

### Unit Tests (Vitest + @testing-library/react)

**Scope**: All components in `src/components/`

**Coverage**: ≥80% lines, ≥90% for Button, Input, Modal, Dropdown, KeyboardVisualizer

**What to test**:
- Rendering with different props (variants, sizes, states)
- Event handlers (onClick, onChange, onSubmit, onKeyDown)
- Accessibility (ARIA attributes, keyboard events, focus management)
- Edge cases (empty state, error state, loading state, disabled state)
- Conditional rendering (tooltip shows on hover, error message appears)

**Example test structure**:
```typescript
// src/components/Button.test.tsx
describe('Button', () => {
  it('renders with primary variant', () => { /* ... */ });
  it('calls onClick when clicked', () => { /* ... */ });
  it('shows loading spinner when loading prop is true', () => { /* ... */ });
  it('has aria-label attribute', () => { /* ... */ });
  it('is keyboard accessible (Enter/Space)', () => { /* ... */ });
});
```

### Integration Tests (Vitest + MSW)

**Scope**: All pages in `src/pages/`

**API Mocking**: Mock Service Worker (MSW) for deterministic API responses

**What to test**:
- User flows end-to-end within a page (rename device → API call → state update → UI update)
- Error handling (API failure → error message displayed → retry works)
- State synchronization (Zustand store ↔ React components)
- Form submissions with validation

**Example test scenario**:
```typescript
// src/pages/DevicesPage.test.tsx
it('renames device successfully', async () => {
  // Setup: Mock API to return success
  server.use(
    rest.put('/api/devices/:id/name', (req, res, ctx) => {
      return res(ctx.json({ success: true }));
    })
  );

  // Action: User clicks rename, enters name, presses Enter
  render(<DevicesPage />);
  const renameButton = screen.getByLabelText('Rename device Main Keyboard');
  await userEvent.click(renameButton);
  const input = screen.getByRole('textbox');
  await userEvent.clear(input);
  await userEvent.type(input, 'New Name{Enter}');

  // Assert: UI updates with new name
  await waitFor(() => {
    expect(screen.getByText('New Name')).toBeInTheDocument();
  });
});
```

### E2E Tests (Playwright)

**Scope**: Critical user flows across multiple pages

**Scenarios**:
1. **Create profile → configure key → activate → verify in simulator** (happy path)
2. **Rename device → change scope → verify persistence** (device management)
3. **Full keyboard navigation test** (accessibility: Tab, Enter, Escape, Arrow keys)
4. **Profile activation with compilation error** (error handling)

**Environments**: Chrome, Firefox, Safari (desktop + mobile viewports)

**Example E2E test**:
```typescript
// tests/e2e/profile-creation.spec.ts
test('create and activate profile', async ({ page }) => {
  await page.goto('/profiles');
  await page.click('text=Create Profile');
  await page.fill('input[name="profileName"]', 'Test Profile');
  await page.click('text=Create');
  await expect(page.locator('text=Test Profile')).toBeVisible();
  await page.click('text=Activate');
  await expect(page.locator('text=ACTIVE')).toBeVisible();
});
```

### Visual Regression Tests (Playwright)

**Scope**: All pages at 3 breakpoints (mobile 375px, tablet 768px, desktop 1280px)

**Baseline**: Screenshots stored in `tests/visual/baselines/`

**Comparison**: Pixel-perfect diff, threshold 0.1% (allow for anti-aliasing)

**Example visual test**:
```typescript
// tests/visual/pages.spec.ts
test('HomePage matches baseline', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveScreenshot('home-desktop.png', {
    fullPage: true,
    threshold: 0.001,
  });
});
```

### Accessibility Tests

**Automated**: @axe-core/react (0 violations required)

**Manual**: NVDA/JAWS screen readers, keyboard-only navigation

**Tools**: Lighthouse accessibility audit (≥95 score)

**Continuous**: axe-core runs in development mode, violations logged to console

### Performance Tests (Lighthouse CI)

**Metrics**:
- LCP (Largest Contentful Paint): <2.5s
- FCP (First Contentful Paint): <1.5s
- TTI (Time to Interactive): <3.0s
- CLS (Cumulative Layout Shift): <0.1
- FID (First Input Delay): <100ms

**Bundle Size**: vite-plugin-compression, rollup-plugin-visualizer

**Frequency**: Every CI run, fails build if budgets exceeded

**Profiling**: Chrome DevTools Performance tab, React DevTools Profiler

---

## Error Code Enumeration

**UI Error Codes** (5000-7999):

### 5000-5999: UI Validation Errors

- **5001**: Invalid input (empty profile name, device name)
- **5002**: Input length exceeded (device name >64 chars, profile name >32 chars)
- **5003**: Invalid characters (profile name contains special chars, only a-z 0-9 - _ allowed)
- **5004**: Invalid threshold (tap-hold <10ms or >2000ms)
- **5005**: Macro sequence too large (>100 steps)
- **5006**: Circular dependency detected (key mapping creates loop)
- **5007**: Profile limit reached (100 profiles maximum)
- **5008**: Heading hierarchy skipped (h1 → h3, accessibility warning)
- **5009**: aria-label too long (>100 characters, suggest aria-describedby)

### 6000-6999: API Communication Errors

- **6001**: Network error (fetch failed, no response)
- **6002**: API timeout (request took >5 seconds)
- **6003**: API error 4xx (client error from daemon: 400 Bad Request, 404 Not Found, etc.)
- **6004**: API error 5xx (server error from daemon: 500 Internal Server Error, etc.)
- **6005**: WebSocket disconnected (real-time metrics unavailable)
- **6006**: WebSocket reconnect failed (after 5 retries with exponential backoff)
- **6007**: CORS error (same-origin policy violation)
- **6008**: API response malformed (JSON parse error)

### 7000-7999: Performance/Resource Errors

- **7001**: WASM failed to load (keyrx_core WASM module, simulator disabled)
- **7002**: Bundle size exceeded (webpack/vite compilation failed, >250KB JS)
- **7003**: Performance budget exceeded (Lighthouse score <90, build warning)
- **7004**: Accessibility violation detected (axe-core found WCAG 2.1 AA violation)
- **7005**: Memory leak detected (React DevTools Profiler shows growing heap)
- **7006**: Layout shift detected (CLS >0.1, visual instability)
- **7007**: Slow render detected (component took >16ms to render, 60fps dropped)
- **7008**: Font loading failed (Inter or JetBrains Mono failed to load, fallback to system font)

**Error Handling Pattern**:
```typescript
// Centralized error handler
export function handleError(error: Error, code: number) {
  console.error(`[${code}] ${error.message}`, error);

  // User-friendly message
  const userMessage = getUserFriendlyMessage(code);

  // Display in ErrorDialog
  errorStore.setError({ code, message: userMessage, details: error.message });

  // Optional: Send to error tracking (Sentry, etc.)
  if (import.meta.env.PROD) {
    trackError(code, error);
  }
}
```

---

## Screen Layouts (ASCII Art)

### Layout 1: Dashboard / Home Screen (Desktop 1280px+)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Dashboard                                                                       │
│                │                                                                                  │
│  🏠 Home       │  ┌─────────────────────────────────────────────────────────────────┐            │
│  📱 Devices    │  │ Active Profile                                                  │            │
│  📋 Profiles   │  │ ┌─────────────────────────────────────────────────────────────┐│            │
│  ⌨️  Config     │  │ │  🎮 Gaming                                          [Edit]  ││            │
│  📊 Metrics    │  │ │                                                               ││            │
│  🧪 Simulator  │  │ │  • 5 Layers  • Modified: 2 hours ago  • 127 key mappings   ││            │
│                │  │ └─────────────────────────────────────────────────────────────┘│            │
│                │  └─────────────────────────────────────────────────────────────────┘            │
│                │                                                                                  │
│                │  ┌─────────────────────────────────────────────────────────────────┐            │
│                │  │ Connected Devices (2)                             [Manage Devices] │           │
│                │  │                                                                   │           │
│                │  │  ┌──────────────────────────────────────────┐                   │           │
│                │  │  │ 🖮 Main Keyboard                 ✓ Active │                   │           │
│                │  │  │ USB\VID_1234&PID_5678\ABC                │                   │           │
│                │  │  │ Scope: Global  •  Layout: ANSI 104        │                   │           │
│                │  │  └──────────────────────────────────────────┘                   │           │
│                │  │                                                                   │           │
│                │  │  ┌──────────────────────────────────────────┐                   │           │
│                │  │  │ 🎮 Left Numpad                    Active  │                   │           │
│                │  │  │ USB\VID_5678&PID_1234\XYZ                │                   │           │
│                │  │  │ Scope: Device-Specific  •  Layout: Numpad │                   │           │
│                │  │  └──────────────────────────────────────────┘                   │           │
│                │  └─────────────────────────────────────────────────────────────────┘            │
│                │                                                                                  │
│                │  ┌─────────────────────────────────────────────────────────────────┐            │
│                │  │ Quick Stats                                                      │            │
│                │  │                                                                  │            │
│                │  │  Latency: 2.3ms avg  •  Events: 1,247 today  •  Uptime: 5h 23m │            │
│                │  └─────────────────────────────────────────────────────────────────┘            │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

**Layout Breakdown**:
- Left sidebar (200px): Persistent navigation with icons + labels
- Main content area: Cards for active profile, devices, and quick stats
- Top bar: App title, settings, help icons
- Spacing: 24px between cards, 16px internal padding

---

### Layout 2: Devices Page (Desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Devices                                                    [Refresh] [+ Add]   │
│                │                                                                                  │
│  🏠 Home       │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  📱 Devices ◀  │  │ Device List (2 connected)                                                 │  │
│  📋 Profiles   │  │                                                                            │  │
│  ⌨️  Config     │  │  ┌────────────────────────────────────────────────────────────────────┐│  │
│  📊 Metrics    │  │  │ 🖮 Main Keyboard                                        ✓ Connected  ││  │
│  🧪 Simulator  │  │  │ USB\VID_1234&PID_5678\ABC123                                         ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Name: [Main Keyboard                              ] [Rename]          ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Scope:  ⦿ Global    ○ Device-Specific                                 ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Layout: [ANSI 104 ▼]                                                  ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Serial: ABC123  •  Vendor: 0x1234  •  Product: 0x5678                 ││  │
│                │  │  │ Last seen: 3 minutes ago                                              ││  │
│                │  │  │                                                                        ││  │
│                │  │  │                                              [Forget Device]           ││  │
│                │  │  └────────────────────────────────────────────────────────────────────┘│  │
│                │  │                                                                            │  │
│                │  │  ┌────────────────────────────────────────────────────────────────────┐│  │
│                │  │  │ 🎮 Left Numpad                                          ✓ Connected  ││  │
│                │  │  │ USB\VID_5678&PID_1234\XYZ789                                         ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Name: [Left Numpad                                ] [Rename]          ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Scope:  ○ Global    ⦿ Device-Specific                                 ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Layout: [Numpad ▼]                                                    ││  │
│                │  │  │                                                                        ││  │
│                │  │  │ Serial: XYZ789  •  Vendor: 0x5678  •  Product: 0x1234                 ││  │
│                │  │  │ Last seen: 1 minute ago                                               ││  │
│                │  │  │                                                                        ││  │
│                │  │  │                                              [Forget Device]           ││  │
│                │  │  └────────────────────────────────────────────────────────────────────┘│  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

**Interactive Elements**:
- Rename input: Click to edit inline, Enter to save, Escape to cancel
- Scope radio buttons: Immediate save on change
- Layout dropdown: Populated from LayoutManager
- Forget Device: Shows confirmation dialog

---

### Layout 3: Profiles Page (Desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Profiles                                               [+ Create Profile]       │
│                │                                                                                  │
│  🏠 Home       │  ┌──────────────┬──────────────┬──────────────┬──────────────┐                 │
│  📱 Devices    │  │              │              │              │              │                 │
│  📋 Profiles ◀ │  │  🎮 Gaming   │  💼 Work     │  🎬 Stream   │  ⚙️  Default  │                 │
│  ⌨️  Config     │  │     ✓        │              │              │              │                 │
│  📊 Metrics    │  │   ACTIVE     │              │              │              │                 │
│  🧪 Simulator  │  │              │              │              │              │                 │
│                │  │  5 layers    │  3 layers    │  6 layers    │  1 layer     │                 │
│                │  │  127 keys    │  45 keys     │  89 keys     │  0 keys      │                 │
│                │  │              │              │              │              │                 │
│                │  │  Modified:   │  Modified:   │  Modified:   │  Modified:   │                 │
│                │  │  2 hrs ago   │  1 day ago   │  3 days ago  │  Never       │                 │
│                │  │              │              │              │              │                 │
│                │  │  [Activate]  │  [Activate]  │  [Activate]  │  [Activate]  │                 │
│                │  │  [Edit]      │  [Edit]      │  [Edit]      │  [Edit]      │                 │
│                │  │  [Duplicate] │  [Duplicate] │  [Duplicate] │  [Duplicate] │                 │
│                │  │  [Export]    │  [Export]    │  [Export]    │  [Export]    │                 │
│                │  │  [Delete]    │  [Delete]    │  [Delete]    │  [Delete]    │                 │
│                │  │              │              │              │              │                 │
│                │  └──────────────┴──────────────┴──────────────┴──────────────┘                 │
│                │                                                                                  │
│                │  Profile Actions:                                                                │
│                │  • Activate: Compile and hot-reload (shows progress)                             │
│                │  • Edit: Navigate to Config page with this profile                              │
│                │  • Duplicate: Create copy with name prompt                                       │
│                │  • Export: Download .rhai file                                                   │
│                │  • Delete: Confirmation dialog (cannot delete active profile)                    │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

**Profile Card States**:
- Active: Green checkmark badge, "ACTIVE" label, Activate button disabled
- Inactive: No badge, Activate button enabled
- Hover: Subtle scale transform (1.02), shadow elevation increase

---

### Layout 4: Configuration Editor (Desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Configuration Editor  —  Profile: Gaming           [🧪 Preview Mode: OFF]      │
│                │                                                                                  │
│  🏠 Home       │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  📱 Devices    │  │ Keyboard Layout                                       Layout: [ANSI 104▼]│  │
│  📋 Profiles   │  │                                                                            │  │
│  ⌨️  Config   ◀│  │  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬───┐│  │
│  📊 Metrics    │  │  │Esc │ F1 │ F2 │ F3 │ F4 │ F5 │ F6 │ F7 │ F8 │ F9 │F10 │F11 │F12 │Del││  │
│  🧪 Simulator  │  │  └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴───┘│  │
│                │  │  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───────┐          │  │
│                │  │  │ ` │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │ 0 │ - │ = │ Bksp  │          │  │
│                │  │  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───────┘          │  │
│                │  │  ┌─────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬─────┐          │  │
│                │  │  │ Tab │ Q │ W │ E │ R │ T │ Y │ U │ I │ O │ P │ [ │ ] │  \  │          │  │
│                │  │  └─────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴─────┘          │  │
│                │  │  ┌───────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───────┐          │  │
│                │  │  │ *Caps*│ A │ S │ D │ F │ G │ H │ J │ K │ L │ ; │ ' │ Enter │          │  │
│                │  │  └───────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───────┘          │  │
│                │  │  ┌─────────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬─────────┐          │  │
│                │  │  │  Shift  │ Z │ X │ C │ V │ B │ N │ M │ , │ . │ / │  Shift  │          │  │
│                │  │  └─────────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴─────────┘          │  │
│                │  │  ┌─────┬─────┬─────┬───────────────────────┬─────┬─────┬─────┬─────┐    │  │
│                │  │  │Ctrl │ GUI │ Alt │       Space           │ Alt │ GUI │ Menu│Ctrl │    │  │
│                │  │  └─────┴─────┴─────┴───────────────────────┴─────┴─────┴─────┴─────┘    │  │
│                │  │                                                                            │  │
│                │  │  *Caps* = Tap: Escape, Hold (200ms): Ctrl                                 │  │
│                │  │                                                                            │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
│                │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│                │  │ Active Layer: MD_00 (Base)                                  [Layer List▼]│  │
│                │  │                                                                            │  │
│                │  │ Layers:  [Base] [Nav] [Num] [Fn] [Gaming]                                │  │
│                │  │                                                                            │  │
│                │  │ Modified keys in this layer: 37                                           │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘

Hover State:
  Key → Tooltip appears: "CapsLock → Tap: Escape, Hold: Ctrl"
  Key color changes to highlight (lighter shade)

Click State:
  Opens configuration dialog (see Modal 1 below)
```

**Key Visual States**:
- Default: Dark gray background, light text
- Modified: Blue tint to indicate custom mapping
- Hover: Lighter background, tooltip
- Active (in simulator): Green highlight
- Disabled: Grayed out with reduced opacity

---

### Layout 5: Metrics / Debugging Page (Desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Performance Metrics                                              [Refresh]      │
│                │                                                                                  │
│  🏠 Home       │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  📱 Devices    │  │ Latency Statistics                                                        │  │
│  📋 Profiles   │  │                                                                            │  │
│  ⌨️  Config     │  │  Min: 0.8ms  •  Avg: 2.3ms  •  Max: 5.1ms  •  P95: 3.2ms  •  P99: 4.5ms │  │
│  📊 Metrics   ◀│  │                                                                            │  │
│  🧪 Simulator  │  │  ┌──────────────────────────────────────────────────────────────────┐    │  │
│                │  │  │  Latency over time (last 60 seconds)                             │    │  │
│                │  │  │  ms                                                               │    │  │
│                │  │  │  5│      *                                                        │    │  │
│                │  │  │  4│    * * *   *                                                  │    │  │
│                │  │  │  3│  * * * * * * * *                                              │    │  │
│                │  │  │  2│* * * * * * * * * * * * * * * * * * * * * * *                  │    │  │
│                │  │  │  1│* * * * * * * * * * * * * * * * * * * * * * * * * * * * * *    │    │  │
│                │  │  │  0└────────────────────────────────────────────────────────────┐  │    │  │
│                │  │  │    0s                    30s                            60s      │  │    │  │
│                │  │  └──────────────────────────────────────────────────────────────────┘    │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
│                │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│                │  │ Event Log (last 50 events)                                   [Clear Log]  │  │
│                │  │                                                                            │  │
│                │  │  ┌──────────┬──────────┬────────────┬────────────┬──────────────────┐    │  │
│                │  │  │ Time     │ Device   │ Event      │ Key        │ Output           │    │  │
│                │  │  ├──────────┼──────────┼────────────┼────────────┼──────────────────┤    │  │
│                │  │  │14:32:45  │ Main KB  │ Press      │ CapsLock   │ (pending 200ms)  │    │  │
│                │  │  │14:32:45  │ Main KB  │ Timeout    │ CapsLock   │ → Ctrl (hold)    │    │  │
│                │  │  │14:32:46  │ Main KB  │ Press      │ A          │ → Ctrl+A         │    │  │
│                │  │  │14:32:46  │ Main KB  │ Release    │ A          │ → Release A      │    │  │
│                │  │  │14:32:46  │ Main KB  │ Release    │ CapsLock   │ → Release Ctrl   │    │  │
│                │  │  │14:32:47  │ Numpad   │ Press      │ Num7       │ → Home           │    │  │
│                │  │  │14:32:47  │ Numpad   │ Release    │ Num7       │ → Release Home   │    │  │
│                │  │  └──────────┴──────────┴────────────┴────────────┴──────────────────┘    │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
│                │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│                │  │ State Inspector                                                           │  │
│                │  │                                                                            │  │
│                │  │  Active Modifiers: Ctrl ✓  Shift   Alt   GUI                             │  │
│                │  │  Active Locks:     CapsLock   NumLock ✓  ScrollLock                      │  │
│                │  │  Active Layers:    MD_00 (Base) ✓                                        │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

**Real-time Updates**:
- Latency chart: Updates every second via WebSocket
- Event log: New events appear at top, auto-scroll disabled if user scrolled up
- State inspector: Updates on every modifier/lock/layer change

---

### Layout 6: Simulator Page (Desktop)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 🎹 KeyRx Configuration                                                           [⚙️ Settings] [?] │
├────────────────┬─────────────────────────────────────────────────────────────────────────────────┤
│                │                                                                                  │
│  Navigation    │  Keyboard Simulator  —  Profile: Gaming                    [Reset] [Save Log]   │
│                │                                                                                  │
│  🏠 Home       │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  📱 Devices    │  │ Interactive Keyboard (Click or Type to Test)                              │  │
│  📋 Profiles   │  │                                                                            │  │
│  ⌨️  Config     │  │  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───────┐        │  │
│  📊 Metrics    │  │  │ ` │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │ 0 │ - │ = │ Bksp  │        │  │
│  🧪 Simulator ◀│  │  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───────┘        │  │
│                │  │  ┌─────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬─────┐        │  │
│                │  │  │ Tab │ Q │ W │ E │ R │ T │ Y │ U │ I │ O │ P │ [ │ ] │  \  │        │  │
│                │  │  └─────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴─────┘        │  │
│                │  │  ┌───────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───────┐        │  │
│                │  │  │ *Caps*│ A │ S │ D │ F │ G │ H │ J │ K │ L │ ; │ ' │ Enter │        │  │
│                │  │  └───────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───────┘        │  │
│                │  │  ┌─────────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬─────────┐        │  │
│                │  │  │  Shift  │ Z │ X │ C │ V │ B │ N │ M │ , │ . │ / │  Shift  │        │  │
│                │  │  └─────────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴─────────┘        │  │
│                │  │  ┌─────┬─────┬─────┬───────────────────────┬─────┬─────┬─────┬─────┐  │  │
│                │  │  │Ctrl │ GUI │ Alt │       Space           │ Alt │ GUI │ Menu│Ctrl │  │  │
│                │  │  └─────┴─────┴─────┴───────────────────────┴─────┴─────┴─────┴─────┘  │  │
│                │  │                                                                            │  │
│                │  │  Keys pressed: CapsLock (held 150ms)  ← Timer shown during hold          │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
│                │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│                │  │ State Display                                                             │  │
│                │  │                                                                            │  │
│                │  │  Active Layer: MD_00 (Base)                                               │  │
│                │  │  Modifiers:    Ctrl ⏱️ (pending)  Shift   Alt   GUI                        │  │
│                │  │  Locks:        CapsLock   NumLock ✓  ScrollLock                          │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
│                │  ┌──────────────────────────────────────────────────────────────────────────┐  │
│                │  │ Output Preview (what would be sent to OS)                                 │  │
│                │  │                                                                            │  │
│                │  │  14:45:12.345  Press CapsLock   (tap-hold: waiting 200ms...)             │  │
│                │  │  14:45:12.555  Timeout → Output: Ctrl (hold activated)                   │  │
│                │  │                                                                            │  │
│                │  │  [Ctrl is now active - next key press will be Ctrl+Key]                   │  │
│                │  └──────────────────────────────────────────────────────────────────────────┘  │
│                │                                                                                  │
└────────────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

**Simulator Features**:
- Click keys with mouse: Simulates press + release
- Hold keys: Shows timer countdown for tap-hold
- Type naturally: Physical keyboard input drives simulation
- Visual feedback: Pressed keys highlighted in green
- Real-time output: Shows exactly what daemon would send to OS

---

### Layout 7: Mobile Layout (< 768px)

```
┌─────────────────────────────────┐
│ 🎹 KeyRx         [☰] [⚙️] [?]   │
├─────────────────────────────────┤
│                                 │
│  Dashboard                      │
│                                 │
│  ┌─────────────────────────────┐│
│  │ Active Profile              ││
│  │                             ││
│  │ 🎮 Gaming           [Edit] ││
│  │                             ││
│  │ 5 Layers • 127 keys         ││
│  │ Modified: 2 hrs ago         ││
│  └─────────────────────────────┘│
│                                 │
│  ┌─────────────────────────────┐│
│  │ Connected Devices (2)       ││
│  │                             ││
│  │ 🖮 Main Keyboard     ✓     ││
│  │ USB\...\ABC          Global ││
│  │                             ││
│  │ 🎮 Left Numpad       ✓     ││
│  │ USB\...\XYZ     Dev-Specific││
│  └─────────────────────────────┘│
│                                 │
│  ┌─────────────────────────────┐│
│  │ Quick Stats                 ││
│  │                             ││
│  │ Latency: 2.3ms              ││
│  │ Events: 1,247               ││
│  │ Uptime: 5h 23m              ││
│  └─────────────────────────────┘│
│                                 │
├─────────────────────────────────┤
│ [🏠] [📱] [📋] [⌨️] [📊]        │
│ Home Devs Prof Conf Metr        │
└─────────────────────────────────┘
```

**Mobile Adaptations**:
- Bottom tab navigation replaces sidebar
- Cards stack vertically
- Hamburger menu (☰) for additional options
- Touch-optimized (≥44px tap targets)
- Horizontal scroll for keyboard layout

---

## Modal Dialogs

### Modal 1: Key Configuration Dialog

```
┌─────────────────────────────────────────────────────────────┐
│ Configure Key: CapsLock                                  [×]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Action Type:  ⦿ Tap-Hold    ○ Simple Remap                │
│                ○ Macro        ○ Layer Switch                │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Tap Action (quick press)                            │  │
│  │                                                       │  │
│  │  Output Key: [Escape              ▼]                │  │
│  │                                                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Hold Action (threshold exceeded)                     │  │
│  │                                                       │  │
│  │  Output Key: [Ctrl                ▼]                │  │
│  │                                                       │  │
│  │  Threshold:  [200] ms         [────●────────]       │  │
│  │              (Range: 10-2000ms)                      │  │
│  │                                                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Preview                                              │  │
│  │                                                       │  │
│  │  Quick tap: CapsLock → Escape                       │  │
│  │  Hold 200ms: CapsLock → Ctrl (modifier active)      │  │
│  │                                                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│                               [Cancel]  [Save & Compile]   │
└─────────────────────────────────────────────────────────────┘
```

**Form States**:
- Validation: Threshold must be 10-2000ms
- Preview: Auto-updates as user changes values
- Save: Calls `keyrx config set-key`, shows compilation progress

---

### Modal 2: Profile Activation Progress

```
┌─────────────────────────────────────────────────────────────┐
│ Activating Profile: Gaming                                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Compiling configuration...                                 │
│                                                             │
│  ████████████████████████████████████─────────────  75%    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Compile Log:                                        │  │
│  │                                                       │  │
│  │  ✓ Parsing Rhai source (25ms)                       │  │
│  │  ✓ Validating layer structure (12ms)                │  │
│  │  ✓ Building DFA state machine (89ms)                │  │
│  │  ⏳ Generating MPHF lookup table...                  │  │
│  │                                                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  Estimated time remaining: 3 seconds                        │
│                                                             │
│                                              [Cancel]       │
└─────────────────────────────────────────────────────────────┘

(On success:)
┌─────────────────────────────────────────────────────────────┐
│ Profile Activated Successfully                           [×]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ✓ Profile "Gaming" is now active                          │
│                                                             │
│  Compilation: 327ms                                         │
│  Hot-reload: 45ms                                           │
│                                                             │
│                                                      [OK]   │
└─────────────────────────────────────────────────────────────┘

(On error:)
┌─────────────────────────────────────────────────────────────┐
│ Compilation Failed                                       [×]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ✗ Error in gaming.rhai                                    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Line 45: Undefined modifier 'SuperCtrl'             │  │
│  │                                                       │  │
│  │   43 | layer.set_key("A", key("B"));                │  │
│  │   44 | layer.set_key("S", key("C"));                │  │
│  │ → 45 | layer.set_key("D", with_mod("SuperCtrl"));   │  │
│  │   46 | layer.set_key("F", key("E"));                │  │
│  │                                                       │  │
│  │ Available modifiers: Ctrl, Shift, Alt, GUI          │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  Previous profile "Work" remains active.                    │
│                                                             │
│                                 [Edit Config]  [Close]     │
└─────────────────────────────────────────────────────────────┘
```

---

### Modal 3: Create Profile

```
┌─────────────────────────────────────────────────────────────┐
│ Create New Profile                                       [×]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Profile Name:                                              │
│  [My New Profile                                         ] │
│  (32 characters max, a-z 0-9 - _ only)                      │
│                                                             │
│  Template:                                                  │
│  ⦿ Blank                                                    │
│     Start with empty configuration                          │
│                                                             │
│  ○ QMK-style Layers                                         │
│     Base layer + 3 extra layers (Nav, Num, Fn)             │
│                                                             │
│  ○ Duplicate Existing                                       │
│     Copy from: [Gaming              ▼]                     │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Preview: Blank Template                             │  │
│  │                                                       │  │
│  │  • 1 layer (MD_00 Base)                              │  │
│  │  • No key mappings                                   │  │
│  │  • Ready for customization                           │  │
│  │                                                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│                                        [Cancel]  [Create]  │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Hierarchy

```
App
├── AppShell
│   ├── TopBar
│   │   ├── Logo
│   │   ├── SettingsButton
│   │   └── HelpButton
│   ├── Sidebar (desktop) / BottomNav (mobile)
│   │   ├── NavItem (Home)
│   │   ├── NavItem (Devices)
│   │   ├── NavItem (Profiles)
│   │   ├── NavItem (Config)
│   │   ├── NavItem (Metrics)
│   │   └── NavItem (Simulator)
│   └── MainContent
│       ├── HomePage
│       │   ├── ActiveProfileCard
│       │   ├── DeviceListCard
│       │   │   └── DeviceCard[] (repeating)
│       │   └── QuickStatsCard
│       ├── DevicesPage
│       │   └── DeviceDetailPanel[] (repeating)
│       │       ├── DeviceNameInput
│       │       ├── ScopeRadioGroup
│       │       ├── LayoutDropdown
│       │       └── ForgetButton
│       ├── ProfilesPage
│       │   ├── CreateProfileButton
│       │   └── ProfileCard[] (repeating)
│       │       ├── ActivateButton
│       │       ├── EditButton
│       │       ├── DuplicateButton
│       │       ├── ExportButton
│       │       └── DeleteButton
│       ├── ConfigPage
│       │   ├── LayoutSelector
│       │   ├── KeyboardVisualizer
│       │   │   └── KeyButton[] (104+ keys, repeating)
│       │   ├── LayerSelector
│       │   └── KeyConfigDialog (modal)
│       │       ├── ActionTypeRadio
│       │       ├── TapHoldForm
│       │       │   ├── TapKeyPicker
│       │       │   ├── HoldKeyPicker
│       │       │   └── ThresholdSlider
│       │       ├── SimpleRemapForm
│       │       ├── MacroForm
│       │       └── PreviewPanel
│       ├── MetricsPage
│       │   ├── LatencyCard
│       │   │   ├── LatencyStats
│       │   │   └── LatencyChart
│       │   ├── EventLogCard
│       │   │   └── EventLogTable
│       │   └── StateInspectorCard
│       └── SimulatorPage
│           ├── InteractiveKeyboard
│           │   └── SimulatorKeyButton[] (repeating)
│           ├── StateDisplay
│           └── OutputPreview
└── GlobalModals
    ├── ProfileActivationProgress
    ├── CreateProfileModal
    ├── ConfirmDialog
    └── ErrorDialog
```

**Component Reusability**:
- `Button` - Used everywhere (primary, secondary, danger variants)
- `Card` - Wraps all content sections
- `Input` - Text inputs with validation
- `Dropdown` - Selects (layouts, keys, profiles)
- `Modal` - Dialog container (reused for all modals)
- `Tooltip` - Hover explanations

---

## State Management Architecture

**Technology**: Zustand (lightweight React state management)

```typescript
// stores/deviceStore.ts
interface DeviceStore {
  devices: DeviceEntry[];
  loading: boolean;
  error: string | null;

  fetchDevices: () => Promise<void>;
  renameDevice: (id: string, name: string) => Promise<void>;
  setScope: (id: string, scope: DeviceScope) => Promise<void>;
  forgetDevice: (id: string) => Promise<void>;
}

// stores/profileStore.ts
interface ProfileStore {
  profiles: ProfileMetadata[];
  activeProfile: string | null;
  activating: boolean;
  activationProgress: number;

  fetchProfiles: () => Promise<void>;
  createProfile: (name: string, template: Template) => Promise<void>;
  activateProfile: (name: string) => Promise<ActivationResult>;
  deleteProfile: (name: string) => Promise<void>;
}

// stores/configStore.ts
interface ConfigStore {
  currentProfile: string | null;
  activeLayer: string;
  keyMappings: Map<string, KeyMapping>;

  fetchConfig: (profile: string) => Promise<void>;
  setKeyMapping: (key: string, mapping: KeyMapping) => Promise<void>;
  deleteKeyMapping: (key: string) => Promise<void>;
  switchLayer: (layerId: string) => void;
}

// stores/metricsStore.ts
interface MetricsStore {
  latencyStats: LatencyStats;
  eventLog: EventRecord[];
  currentState: DaemonState;

  fetchMetrics: () => Promise<void>;
  subscribeToEvents: () => void;  // WebSocket
  unsubscribeFromEvents: () => void;
}
```

**State Flow**:
```
User Action (click button)
    ↓
React Component (e.g., DevicesPage)
    ↓
Zustand Store Action (deviceStore.renameDevice)
    ↓
API Call (fetch to /api/devices/:id/name)
    ↓
CLI Command Execution (keyrx devices rename)
    ↓
Response JSON
    ↓
Zustand Store Update (devices array updated)
    ↓
React Re-render (UI reflects change)
```

---

## API Integration Patterns

All API endpoints are thin wrappers calling CLI commands:

```typescript
// api/devices.ts
export async function renameDevice(id: string, name: string): Promise<void> {
  const response = await fetch(`/api/devices/${id}/name`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }
}

// api/profiles.ts
export async function activateProfile(name: string): Promise<ActivationResult> {
  const response = await fetch(`/api/profiles/${name}/activate`, {
    method: 'POST',
  });

  const data = await response.json();
  if (!response.ok) throw new Error(data.error.message);

  return {
    compileTimeMs: data.compile_time_ms,
    reloadTimeMs: data.reload_time_ms,
    success: data.success,
  };
}
```

**Backend (Rust)**:
```rust
// keyrx_daemon/src/web/api.rs (NEW - web UI backend)
async fn rename_device(
    Path(id): Path<String>,
    Json(payload): Json<RenameRequest>,
) -> Result<Json<Value>, ApiError> {
    // Call same logic as CLI
    let mut registry = DeviceRegistry::load()?;
    registry.rename(&id, &payload.name)?;
    registry.save()?;

    Ok(Json(json!({ "success": true })))
}
```

---

## Accessibility Implementation

### ARIA Labels

```html
<!-- Keyboard key button -->
<button
  class="keyboard-key"
  aria-label="Key CapsLock. Current mapping: Tap Escape, Hold Control. Click to configure."
  aria-pressed="false"
  tabindex="0"
>
  Caps
</button>

<!-- Profile card activate button -->
<button
  aria-label="Activate profile Gaming. Compile and hot-reload configuration."
  aria-busy="false"
>
  Activate
</button>

<!-- Device scope toggle -->
<fieldset aria-labelledby="scope-legend">
  <legend id="scope-legend">Configuration Scope</legend>
  <input type="radio" id="scope-global" name="scope" aria-checked="true" />
  <label for="scope-global">Global (all devices)</label>
  <input type="radio" id="scope-device" name="scope" aria-checked="false" />
  <label for="scope-device">Device-Specific</label>
</fieldset>
```

### Focus Management

```typescript
// When modal opens
function openModal(modalId: string) {
  const modal = document.getElementById(modalId);
  const firstFocusable = modal.querySelector('button, input, select');

  // Store previous focus
  previousFocus = document.activeElement;

  // Focus first element
  firstFocusable?.focus();

  // Trap focus within modal
  modal.addEventListener('keydown', trapFocus);
}

// When modal closes
function closeModal() {
  // Return focus to trigger element
  previousFocus?.focus();
}
```

### Screen Reader Announcements

```html
<!-- Live region for dynamic updates -->
<div aria-live="polite" aria-atomic="true" class="sr-only">
  {statusMessage}
</div>

<!-- Example messages -->
<div aria-live="assertive">Profile activated successfully</div>
<div aria-live="assertive">Error: Compilation failed on line 45</div>
```

---

## Visual Design System (Detailed)

### Color Palette (Dark Theme)

```css
:root {
  /* Primary (Blue) */
  --color-primary-50: #EFF6FF;
  --color-primary-100: #DBEAFE;
  --color-primary-200: #BFDBFE;
  --color-primary-300: #93C5FD;
  --color-primary-400: #60A5FA;
  --color-primary-500: #3B82F6;  /* Main */
  --color-primary-600: #2563EB;  /* Hover */
  --color-primary-700: #1D4ED8;

  /* Background (Slate) */
  --color-bg-primary: #0F172A;   /* slate-900 */
  --color-bg-secondary: #1E293B; /* slate-800 */
  --color-bg-tertiary: #334155;  /* slate-700 */

  /* Text */
  --color-text-primary: #F1F5F9;   /* slate-100 */
  --color-text-secondary: #94A3B8; /* slate-400 */
  --color-text-disabled: #64748B;  /* slate-500 */

  /* Borders */
  --color-border: #334155;  /* slate-700 */
  --color-border-hover: #475569;  /* slate-600 */

  /* Status */
  --color-success: #10B981;  /* green-500 */
  --color-error: #EF4444;    /* red-500 */
  --color-warning: #F59E0B;  /* amber-500 */
  --color-info: #3B82F6;     /* blue-500 */
}
```

### Typography Scale

```css
:root {
  --font-family-base: 'Inter', system-ui, -apple-system, sans-serif;
  --font-family-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;

  /* Sizes */
  --font-size-xs: 12px;
  --font-size-sm: 13px;
  --font-size-base: 14px;
  --font-size-lg: 16px;
  --font-size-xl: 18px;
  --font-size-2xl: 24px;
  --font-size-3xl: 32px;

  /* Line heights */
  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;

  /* Font weights */
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;
}

/* Usage */
h1 { font-size: var(--font-size-3xl); line-height: var(--line-height-tight); }
h2 { font-size: var(--font-size-2xl); line-height: var(--line-height-tight); }
body { font-size: var(--font-size-base); line-height: var(--line-height-normal); }
code { font-family: var(--font-family-mono); font-size: var(--font-size-sm); }
```

### Spacing Grid

```css
:root {
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;
  --spacing-2xl: 48px;
  --spacing-3xl: 64px;
}

/* Component spacing examples */
.card { padding: var(--spacing-lg); margin-bottom: var(--spacing-md); }
.button { padding: var(--spacing-sm) var(--spacing-md); }
.modal { padding: var(--spacing-xl); }
```

### Shadows

```css
:root {
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.card { box-shadow: var(--shadow-md); }
.modal { box-shadow: var(--shadow-xl); }
```

### Border Radius

```css
:root {
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;
}

.button { border-radius: var(--radius-md); }
.card { border-radius: var(--radius-lg); }
.avatar { border-radius: var(--radius-full); }
```

---

## Responsive Breakpoints

```css
/* Mobile first approach */
.sidebar {
  display: none;  /* Hidden on mobile */
}

@media (min-width: 768px) {  /* Tablet */
  .sidebar {
    display: flex;
    width: 200px;
    position: fixed;
  }

  .bottom-nav {
    display: none;  /* Hidden on tablet+ */
  }
}

@media (min-width: 1280px) {  /* Desktop */
  .sidebar {
    width: 240px;
  }

  .keyboard-visualizer {
    zoom: 1.2;  /* Larger on desktop */
  }
}
```

---

## Performance Optimizations

### Code Splitting

```typescript
// Lazy load heavy components
const KeyboardVisualizer = lazy(() => import('./components/KeyboardVisualizer'));
const SimulatorPage = lazy(() => import('./pages/SimulatorPage'));

// Routes with Suspense
<Routes>
  <Route path="/" element={<HomePage />} />
  <Route path="/config" element={
    <Suspense fallback={<LoadingSpinner />}>
      <KeyboardVisualizer />
    </Suspense>
  } />
</Routes>
```

### Virtual Scrolling

```typescript
// For event log (1000s of events)
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={400}
  itemCount={eventLog.length}
  itemSize={32}
  width="100%"
>
  {({ index, style }) => (
    <div style={style}>
      {eventLog[index].timestamp} - {eventLog[index].event}
    </div>
  )}
</FixedSizeList>
```

### Memoization

```typescript
// Expensive computations cached
const keyboardLayout = useMemo(() => {
  return parseKLEJson(layoutData);
}, [layoutData]);

// Prevent unnecessary re-renders
const KeyButton = memo(({ keyCode, mapping, onClick }) => {
  return <button onClick={() => onClick(keyCode)}>{keyCode}</button>;
});
```

---

## Testing Strategy

### Visual Regression Testing
- **Tool**: Playwright with screenshot comparison
- **Coverage**: All major screens at 3 breakpoints (mobile, tablet, desktop)

### Accessibility Testing
- **Tool**: axe-core (automated WCAG checks)
- **Manual**: Keyboard navigation, screen reader (NVDA/JAWS)

### E2E Testing
- **Tool**: Playwright
- **Scenarios**: Full user flows (create profile → configure key → activate → verify)

---

## Next Steps

After implementing this visual design:
1. User testing with 5-10 keyboard enthusiasts
2. Iterate on feedback (especially keyboard layout UX)
3. Add theme switcher (dark/light)
4. Internationalization (i18n) framework
5. Advanced features (export as QMK firmware, share configs online)

---

**This design serves as the blueprint for world-class UI/UX implementation.**
