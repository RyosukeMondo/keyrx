# Tasks Document: Web UI Configuration Editor

## Phase 0: Environment Setup

- [ ] 0. Initialize React + TypeScript + Vite project
  - Files: `keyrx_ui_v2/` (new directory), `package.json`, `tsconfig.json`, `vite.config.ts`, `tailwind.config.js`, `.eslintrc.js`, `.prettierrc`
  - Purpose: Prepare complete development environment with build tooling, testing infrastructure, and code quality tools. This foundation supports all future development with fast iteration, type safety, and automated quality gates.
  - Requirements: Design.md Dependencies section, Req 4 (Performance Budget)
  - Prompt: Role: Frontend Infrastructure Developer | Task: Set up modern React development environment with:

    **Project Initialization**:
    ```bash
    npm create vite@latest keyrx_ui_v2 -- --template react-ts
    cd keyrx_ui_v2
    ```

    **Production Dependencies** (install with exact versions):
    ```bash
    npm install react@18.2.0 react-dom@18.2.0
    npm install react-router-dom@6.20.0
    npm install zustand@4.4.5
    npm install tailwindcss@3.4.0 postcss@8.4.32 autoprefixer@10.4.16
    npm install @tanstack/react-query@5.17.0
    ```

    **Dev Dependencies**:
    ```bash
    npm install -D vitest@1.0.4 @testing-library/react@14.1.2 @testing-library/jest-dom@6.1.5
    npm install -D playwright@1.40.1 @playwright/test@1.40.1
    npm install -D @axe-core/react@4.8.3
    npm install -D eslint@8.56.0 prettier@3.1.1
    npm install -D @typescript-eslint/eslint-plugin@6.16.0 @typescript-eslint/parser@6.16.0
    npm install -D vite-plugin-compression@0.5.1
    ```

    **Tailwind Configuration** (tailwind.config.js):
    ```javascript
    export default {
      content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
      theme: {
        extend: {
          colors: {
            primary: {
              500: '#3B82F6',
              600: '#2563EB',
            },
            // ... all design tokens from design.md
          },
          fontFamily: {
            sans: ['Inter', 'system-ui', 'sans-serif'],
            mono: ['JetBrains Mono', 'Consolas', 'monospace'],
          },
          spacing: {
            xs: '4px',
            sm: '8px',
            md: '16px',
            lg: '24px',
            xl: '32px',
            '2xl': '48px',
          },
        },
      },
    };
    ```

    **TypeScript Configuration** (tsconfig.json):
    ```json
    {
      "compilerOptions": {
        "target": "ES2020",
        "lib": ["ES2020", "DOM", "DOM.Iterable"],
        "module": "ESNext",
        "skipLibCheck": true,
        "moduleResolution": "bundler",
        "allowImportingTsExtensions": true,
        "resolveJsonModule": true,
        "isolatedModules": true,
        "noEmit": true,
        "jsx": "react-jsx",
        "strict": true,
        "noUnusedLocals": true,
        "noUnusedParameters": true,
        "noFallthroughCasesInSwitch": true,
        "baseUrl": ".",
        "paths": {
          "@/*": ["./src/*"]
        }
      },
      "include": ["src"],
      "exclude": ["node_modules"]
    }
    ```

    **ESLint Configuration** (.eslintrc.js):
    ```javascript
    module.exports = {
      extends: [
        'eslint:recommended',
        'plugin:@typescript-eslint/recommended',
        'plugin:react-hooks/recommended',
      ],
      rules: {
        'no-console': ['error', { allow: ['warn', 'error'] }],
        '@typescript-eslint/no-explicit-any': 'error',
      },
    };
    ```

    **Prettier Configuration** (.prettierrc):
    ```json
    {
      "semi": true,
      "singleQuote": true,
      "tabWidth": 2,
      "trailingComma": "es5"
    }
    ```

    **Vite Configuration** (vite.config.ts):
    ```typescript
    import { defineConfig } from 'vite';
    import react from '@vitejs/plugin-react';
    import compression from 'vite-plugin-compression';

    export default defineConfig({
      plugins: [react(), compression()],
      resolve: {
        alias: {
          '@': '/src',
        },
      },
      build: {
        target: 'es2020',
        rollupOptions: {
          output: {
            manualChunks: {
              react: ['react', 'react-dom', 'react-router-dom'],
              zustand: ['zustand'],
            },
          },
        },
      },
    });
    ```

  | Restrictions: TypeScript strict mode REQUIRED, no `any` types allowed, all dependencies must have exact versions (no ^), ESLint 0 errors 0 warnings, Prettier must format all files
  | Success: ✅ `npm run dev` starts dev server on http://localhost:5173, ✅ Hot reload works (changes reflect <100ms), ✅ TypeScript compiles without errors, ✅ ESLint shows 0 errors, ✅ Prettier formats all files, ✅ Tailwind generates utility classes, ✅ All dependencies installed with correct versions, ✅ `npm run build` succeeds without warnings

---

## Phase 1: Design System & Core Components

- [ ] 1. Implement design tokens and CSS variables
  - File: `src/styles/tokens.css`
  - Define all CSS variables from design.md:
    - Colors (primary, background, text, status)
    - Typography (font sizes, weights, line heights)
    - Spacing (xs to 3xl scale)
    - Shadows, border-radius, breakpoints
  - Create Tailwind config extending tokens
  - Purpose: Centralize design system for consistency
  - _Leverage: Tailwind CSS custom theme_
  - _Requirements: Req 1 (Visual Design System)_
  - _Success: ✅ All tokens defined, ✅ Tailwind generates utility classes, ✅ No magic numbers in components

- [ ] 2. Create Button component
  - File: `src/components/Button.tsx`
  - Variants: primary, secondary, danger, ghost
  - Sizes: sm, md, lg
  - States: default, hover, active, disabled, loading
  - Props: `variant`, `size`, `disabled`, `loading`, `onClick`, `aria-label`
  - Ripple effect on click
  - Purpose: Reusable button component
  - _Leverage: Tailwind variants_
  - _Requirements: Req 1 (consistent interactive elements)_
  - _Success: ✅ All variants render correctly, ✅ Ripple animates on click, ✅ Focus outline visible, ✅ Disabled state prevents clicks

- [ ] 3. Create Input component
  - File: `src/components/Input.tsx`
  - Types: text, number
  - States: default, focus, error, disabled
  - Props: `value`, `onChange`, `error`, `disabled`, `aria-label`, `maxLength`
  - Error message display below input
  - Character counter for maxLength
  - Purpose: Reusable text input with validation
  - _Leverage: React controlled components_
  - _Requirements: Req 3 (Accessibility)_
  - _Success: ✅ Error state shows red border and message, ✅ Character counter updates, ✅ Focus outline visible, ✅ aria-invalid set on error

- [ ] 4. Create Card component
  - File: `src/components/Card.tsx`
  - Variants: default, elevated (higher shadow)
  - Optional: header, footer slots
  - Padding: configurable (sm, md, lg)
  - Purpose: Container for content sections
  - _Leverage: Tailwind shadow utilities_
  - _Requirements: Design.md card pattern_
  - _Success: ✅ Card renders with correct padding and shadow, ✅ Header and footer slots work, ✅ Elevated variant has larger shadow

- [ ] 5. Create Modal component
  - File: `src/components/Modal.tsx`
  - Features: backdrop, close button, Escape to close, focus trap
  - Animations: fade in/out, scale transform
  - Accessibility: aria-modal, aria-labelledby, return focus on close
  - Props: `open`, `onClose`, `title`, `children`
  - Purpose: Reusable dialog container
  - _Leverage: React Portal for overlay_
  - _Requirements: Req 3 (Accessibility - Escape closes, focus returns)_
  - _Success: ✅ Modal opens/closes with animation, ✅ Escape closes modal, ✅ Focus returns to trigger, ✅ Focus trapped within modal

- [ ] 6. Create Dropdown component
  - File: `src/components/Dropdown.tsx`
  - Features: searchable, keyboard navigation (arrow keys)
  - States: open, closed, focused
  - Props: `options`, `value`, `onChange`, `searchable`, `aria-label`
  - Purpose: Reusable select/dropdown
  - _Leverage: Headless UI or Radix UI (a11y primitives)_
  - _Requirements: Req 3 (keyboard navigation)_
  - _Success: ✅ Arrow keys navigate options, ✅ Enter selects, ✅ Escape closes, ✅ Search filters options

- [ ] 7. Create Tooltip component
  - File: `src/components/Tooltip.tsx`
  - Positioning: auto (top, bottom, left, right based on viewport)
  - Delay: 500ms before show
  - Props: `content`, `children`, `position`
  - Purpose: Contextual help on hover
  - _Leverage: Floating UI for positioning_
  - _Requirements: Req 7 (keyboard hover tooltips)_
  - _Success: ✅ Tooltip shows on hover after delay, ✅ Positions correctly near viewport edges, ✅ Hides on mouseout

---

## Phase 2: Layout Components

- [ ] 8. Create AppShell layout
  - File: `src/components/AppShell.tsx`
  - Components: TopBar, Sidebar (desktop), BottomNav (mobile), MainContent
  - Responsive: sidebar → bottom nav at <768px
  - Sidebar: collapsible on tablet
  - Purpose: Main application layout
  - _Leverage: CSS Grid for layout_
  - _Requirements: Req 2 (Responsive Design)_
  - _Success: ✅ Sidebar visible on desktop, ✅ Bottom nav on mobile, ✅ Layout adapts smoothly on resize

- [ ] 9. Create TopBar component
  - File: `src/components/TopBar.tsx`
  - Elements: logo, title, settings button, help button
  - Responsive: title hides on <768px
  - Purpose: Application header
  - _Leverage: Flexbox for alignment_
  - _Requirements: Design.md Layout 1_
  - _Success: ✅ Logo and buttons align correctly, ✅ Buttons have hover states, ✅ Title hides on mobile

- [ ] 10. Create Sidebar navigation
  - File: `src/components/Sidebar.tsx`
  - Navigation items: Home, Devices, Profiles, Config, Metrics, Simulator
  - Active state: highlighted with indicator
  - Icons: use Lucide React (or Heroicons)
  - Collapsible: hamburger button on tablet
  - Purpose: Main navigation
  - _Leverage: React Router NavLink for active state_
  - _Requirements: Req 3 (keyboard navigation)_
  - _Success: ✅ Active page highlighted, ✅ Keyboard navigable (Tab, Enter), ✅ Collapses on tablet

- [ ] 11. Create BottomNav (mobile)
  - File: `src/components/BottomNav.tsx`
  - Items: Home, Devices, Profiles, Config, Metrics (5 icons)
  - Fixed position at bottom
  - Active state: filled icon + label color
  - Purpose: Mobile navigation
  - _Leverage: Fixed positioning_
  - _Requirements: Req 2 (mobile navigation), Req 3 (touch targets ≥44px)_
  - _Success: ✅ Fixed at bottom on mobile, ✅ Touch targets ≥44px, ✅ Active state visible

---

## Phase 3: Feature Pages

- [ ] 12. Create HomePage / Dashboard
  - File: `src/pages/HomePage.tsx`
  - Components: ActiveProfileCard, DeviceListCard, QuickStatsCard
  - Layout: from design.md Layout 1
  - Purpose: Main dashboard view
  - _Leverage: Card components from Phase 1_
  - _Requirements: Req 5 (device management), Req 6 (profile management)_
  - _Success: ✅ Layout matches design.md, ✅ Cards render with correct data, ✅ Responsive (stacks on mobile)

- [ ] 13. Create DevicesPage
  - File: `src/pages/DevicesPage.tsx`
  - Features: device list, inline rename, scope toggle, layout selector
  - Components: DeviceDetailPanel (repeating)
  - Inline edit: click "Rename" → input appears, Enter saves
  - Purpose: Device management interface
  - _Leverage: Input component, Dropdown component_
  - _Requirements: Req 5 (User Flows - Device Management)_
  - _Success: ✅ Rename works inline, ✅ Scope toggle saves immediately, ✅ Forget device shows confirmation

- [ ] 14. Create ProfilesPage
  - File: `src/pages/ProfilesPage.tsx`
  - Features: profile cards grid, create button, activate/edit/delete actions
  - Components: ProfileCard (repeating)
  - Active profile: green checkmark badge, "ACTIVE" label
  - Purpose: Profile management interface
  - _Leverage: Card component, Modal for create_
  - _Requirements: Req 6 (User Flows - Profile Management)_
  - _Success: ✅ Grid layout matches design.md, ✅ Active profile highlighted, ✅ Create modal works

- [ ] 15. Create ConfigPage / Editor
  - File: `src/pages/ConfigPage.tsx`
  - Features: keyboard visualizer, layer selector, key config dialog
  - Layout: from design.md Layout 4
  - Purpose: Visual keyboard configuration
  - _Leverage: KeyboardVisualizer component (Task 16)_
  - _Requirements: Req 7 (User Flows - Keyboard Configuration)_
  - _Success: ✅ Layout matches design.md, ✅ Layer selector works, ✅ Clicking key opens dialog

- [ ] 16. Create KeyboardVisualizer component
  - File: `src/components/KeyboardVisualizer.tsx`
  - Features: render keyboard from KLE JSON, interactive keys, hover tooltips
  - States: default, hover (lighter), modified (blue tint)
  - Layout: ANSI 104, ISO 105, JIS 109, HHKB, Numpad (selectable)
  - Click handler: opens KeyConfigDialog
  - Purpose: Visual keyboard representation
  - _Leverage: SVG or CSS Grid for key layout_
  - _Requirements: Req 7.2 (hover tooltips), Req 7.3 (click opens dialog)_
  - _Success: ✅ All layouts render correctly, ✅ Hover shows tooltip, ✅ Click opens config dialog, ✅ Modified keys have blue tint

- [ ] 17. Create KeyConfigDialog modal
  - File: `src/components/KeyConfigDialog.tsx`
  - Features: action type selector, dynamic form (tap-hold/simple/macro/layer)
  - Forms: TapHoldForm, SimpleRemapForm, MacroForm, LayerSwitchForm
  - Preview panel: shows mapping description
  - Purpose: Configure individual key mappings
  - _Leverage: Modal component, Dropdown for key pickers_
  - _Requirements: Req 7.4-7.6 (action type forms)_
  - _Success: ✅ Action type selector works, ✅ Forms render correctly, ✅ Preview updates, ✅ Save calls API

- [ ] 18. Create MetricsPage
  - File: `src/pages/MetricsPage.tsx`
  - Components: LatencyCard, EventLogTable, StateInspectorCard
  - Chart: latency over time (last 60s)
  - Event log: virtual scrolling for performance
  - Purpose: Performance monitoring and debugging
  - _Leverage: Chart library (Recharts or Chart.js), react-window for virtual scrolling_
  - _Requirements: Design.md Layout 5_
  - _Success: ✅ Chart renders with real data, ✅ Event log scrolls smoothly, ✅ State inspector shows current state

- [ ] 19. Create SimulatorPage
  - File: `src/pages/SimulatorPage.tsx`
  - Features: interactive keyboard, click/type to simulate, output preview
  - State display: active layer, modifiers, locks
  - Timer display: shows countdown during tap-hold
  - Purpose: Test configurations before applying to real keyboard
  - _Leverage: WASM keyrx_core for simulation_
  - _Requirements: Req 8 (Real-Time Preview and Simulation)_
  - _Success: ✅ Click keys simulates press/release, ✅ Hold shows timer, ✅ Output preview updates, ✅ Reset clears state

---

## Phase 4: API Integration & State Management

- [ ] 20. Set up Zustand stores
  - Files: `src/stores/deviceStore.ts`, `profileStore.ts`, `configStore.ts`, `metricsStore.ts`
  - Implement state management for all data
  - Actions: fetch, create, update, delete
  - Purpose: Centralized state management
  - _Leverage: Zustand (lightweight, TypeScript-friendly)_
  - _Requirements: Design.md State Management Architecture_
  - _Success: ✅ All stores defined, ✅ Actions call API, ✅ State updates trigger re-renders

- [ ] 21. Implement API client
  - Files: `src/api/devices.ts`, `profiles.ts`, `config.ts`, `metrics.ts`
  - Endpoints: match CLI API from web-ui-ux-comprehensive spec
  - Error handling: throw descriptive errors with API error codes
  - Purpose: API communication layer
  - _Leverage: fetch API, React Query for caching_
  - _Requirements: Design.md API Integration Patterns_
  - _Success: ✅ All endpoints implemented, ✅ Errors thrown with messages, ✅ TypeScript types for requests/responses

- [ ] 22. Implement WebSocket for real-time metrics
  - File: `src/api/websocket.ts`
  - Connect to daemon WebSocket (ws://localhost:9867/ws)
  - Subscribe to: event stream, state changes, latency updates
  - Reconnect on disconnect
  - Purpose: Real-time updates for metrics page
  - _Leverage: native WebSocket API_
  - _Requirements: Req 8 (real-time preview)_
  - _Success: ✅ WebSocket connects on mount, ✅ Events update state, ✅ Reconnects on disconnect, ✅ Unsubscribes on unmount

- [ ] 23. Integrate React Query for caching
  - Files: Update all stores to use React Query hooks
  - Features: automatic refetching, cache invalidation, optimistic updates
  - Purpose: Improve performance and user experience
  - _Leverage: @tanstack/react-query_
  - _Requirements: Req 4 (Performance)_
  - _Success: ✅ Queries cache data, ✅ Mutations invalidate cache, ✅ Stale data refetches

---

## Phase 5: Responsive Design & Polish

- [ ] 24. Implement responsive layouts
  - Files: All page components
  - Breakpoints: mobile (<768px), tablet (768-1279px), desktop (≥1280px)
  - Adaptations:
    - Sidebar → BottomNav on mobile
    - Cards stack vertically on mobile
    - Keyboard layout horizontal scroll on mobile
  - Purpose: Ensure usability on all devices
  - _Leverage: Tailwind responsive utilities (sm:, md:, lg:)_
  - _Requirements: Req 2 (Responsive Design)_
  - _Success: ✅ All pages work on mobile, ✅ Touch targets ≥44px, ✅ No horizontal scroll (except keyboard)

- [ ] 25. Implement animations and transitions
  - Files: Update Button, Modal, Card components
  - Animations:
    - Modal: fade in/out + scale
    - Button: ripple effect on click
    - Page transitions: fade between routes
  - Reduced motion: disable animations if `prefers-reduced-motion`
  - Purpose: Polished, smooth user experience
  - _Leverage: Tailwind transitions, Framer Motion (optional)_
  - _Requirements: Req 4 (animations use transform/opacity), Req 4.6 (reduced motion)_
  - _Success: ✅ Animations smooth (60fps), ✅ Reduced motion disables animations, ✅ No layout shifts

- [ ] 26. Add loading states and skeletons
  - Files: Create LoadingSkeleton component, update all pages
  - Skeleton screens: matching layout structure, pulsing animation
  - Loading spinners: for buttons during async operations
  - Purpose: Feedback during data loading
  - _Leverage: Tailwind animations_
  - _Requirements: Req 4 (visual feedback <16ms)_
  - _Success: ✅ Skeletons match page layout, ✅ Spinners show during API calls, ✅ No "flash of no content"

- [ ] 27. Implement error states and boundaries
  - Files: Create ErrorBoundary component, ErrorState component
  - Error boundary: catch React errors, show fallback UI
  - Error states: for failed API calls (retry button)
  - Purpose: Graceful error handling
  - _Leverage: React ErrorBoundary_
  - _Requirements: Req 4.12 (graceful error page)_
  - _Success: ✅ Errors caught and displayed, ✅ Retry button works, ✅ Error boundary prevents white screen

---

## Phase 6: Accessibility Implementation

- [ ] 28. Add ARIA labels and roles
  - Files: All interactive components
  - Requirements:
    - All buttons have aria-label
    - Form inputs have aria-labelledby or labels
    - Error messages have aria-live="assertive"
    - Modals have aria-modal, role="dialog"
  - Purpose: Screen reader compatibility
  - _Leverage: ARIA spec_
  - _Requirements: Req 3.3 (ARIA labels)_
  - _Success: ✅ Screen reader announces all elements, ✅ Errors announced, ✅ Modal role correct

- [ ] 29. Implement keyboard navigation
  - Files: All interactive components, pages
  - Requirements:
    - Tab order logical (top to bottom, left to right)
    - Focus visible (2px outline)
    - Escape closes modals
    - Arrow keys navigate dropdowns/lists
  - Purpose: Keyboard-only usability
  - _Leverage: tabindex, onKeyDown handlers_
  - _Requirements: Req 3.1-3.2 (Tab, Escape navigation)_
  - _Success: ✅ All features accessible via keyboard, ✅ Focus visible, ✅ Tab order logical

- [ ] 30. Implement focus management
  - Files: Modal, Dropdown components
  - Requirements:
    - Focus trapped in modal (Tab loops within)
    - Focus returns to trigger on close
    - First element focused on modal open
  - Purpose: Proper focus flow for screen readers
  - _Leverage: Focus trap library or custom implementation_
  - _Requirements: Req 3.2 (focus returns on modal close)_
  - _Success: ✅ Focus trapped in modal, ✅ Focus returns on close, ✅ First element focused on open

- [ ] 31. Test with accessibility tools
  - Tools: axe-core, Lighthouse, WAVE
  - Requirements:
    - 0 violations in axe-core automated scan
    - Lighthouse accessibility score ≥95
  - Manual testing: NVDA/JAWS screen readers, keyboard-only
  - Purpose: Verify WCAG 2.1 AA compliance
  - _Leverage: @axe-core/react, Lighthouse CI_
  - _Requirements: Req 3 (WCAG 2.1 Level AA)_
  - _Success: ✅ 0 axe violations, ✅ Lighthouse ≥95, ✅ Manual testing passes

---

## Phase 7: Testing & Quality

- [ ] 32. Unit tests for components
  - Files: `src/components/*.test.tsx`
  - Test: Button, Input, Card, Modal, Dropdown
  - Coverage: ≥80% for all components
  - Test cases:
    - Rendering with different props
    - Event handlers (onClick, onChange)
    - Accessibility (aria attributes, keyboard events)
  - Purpose: Ensure component reliability
  - _Leverage: Vitest, @testing-library/react_
  - _Requirements: Non-functional (quality)_
  - _Success: ✅ All tests pass, ✅ Coverage ≥80%, ✅ No console errors

- [ ] 33. Integration tests for pages
  - Files: `src/pages/*.test.tsx`
  - Test: User flows (device rename, profile activation, key config)
  - Mock API calls with MSW (Mock Service Worker)
  - Purpose: Ensure page interactions work
  - _Leverage: Vitest, MSW_
  - _Requirements: Req 5-7 (user flows)_
  - _Success: ✅ All flows work, ✅ API mocks return expected data, ✅ No race conditions

- [ ] 34. E2E tests with Playwright
  - Files: `tests/e2e/*.spec.ts`
  - Scenarios:
    - Create profile → configure key → activate → verify in simulator
    - Rename device → change scope → verify persistence
    - Full keyboard navigation test
  - Purpose: End-to-end verification
  - _Leverage: Playwright_
  - _Requirements: All user flows_
  - _Success: ✅ All scenarios pass, ✅ Screenshots match expected, ✅ Tests run in CI

- [ ] 35. Visual regression tests
  - Files: `tests/visual/*.spec.ts`
  - Screenshots: all pages at 3 breakpoints (mobile, tablet, desktop)
  - Compare: against baseline screenshots
  - Purpose: Catch unintended visual changes
  - _Leverage: Playwright visual comparisons_
  - _Requirements: Req 1 (consistent visual design)_
  - _Success: ✅ All screenshots match baseline, ✅ No unexpected diffs

- [ ] 36. Performance testing
  - Files: `tests/performance/*.spec.ts`
  - Metrics: LCP, FCP, TTI, CLS, FID
  - Budgets:
    - LCP < 2.5s
    - FCP < 1.5s
    - TTI < 3.0s
    - CLS < 0.1
    - FID < 100ms
  - Purpose: Verify performance targets
  - _Leverage: Lighthouse CI_
  - _Requirements: Req 4 (Performance Budget)_
  - _Success: ✅ All metrics within budget, ✅ Lighthouse score ≥90

- [ ] 37. Bundle size optimization
  - Tools: vite-plugin-compression, rollup-plugin-visualizer
  - Actions:
    - Code splitting (lazy load routes)
    - Tree shaking (unused code removed)
    - Compression (gzip/brotli)
  - Target: JS bundle ≤250KB gzipped, CSS ≤50KB gzipped
  - Purpose: Fast load times
  - _Leverage: Vite build optimizations_
  - _Requirements: Req 4 (Performance Budget)_
  - _Success: ✅ Bundle sizes within limits, ✅ Lazy loading works, ✅ No unused dependencies

---

## Phase 8: Production Build & Deployment

- [ ] 38. Configure production build
  - Files: `vite.config.ts`, `.env.production`
  - Optimizations: minification, tree shaking, code splitting
  - Environment variables: API_URL, WS_URL
  - Purpose: Production-ready build
  - _Leverage: Vite build command_
  - _Requirements: Non-functional_
  - _Success: ✅ Build completes without errors, ✅ Output is minified, ✅ Source maps generated

- [ ] 39. Embed UI in daemon
  - Files: `keyrx_daemon/src/web/static_files.rs`, `keyrx_daemon/Cargo.toml`
  - Embed built UI files in binary using `include_dir!` macro
  - Serve from `/` route (Axum)
  - Purpose: Self-contained web UI (no separate server)
  - _Leverage: include_dir crate, Axum static file serving_
  - _Requirements: Design philosophy (embedded UI)_
  - _Success: ✅ UI files embedded in binary, ✅ Served from http://localhost:9867, ✅ No external dependencies

- [ ] 40. Add CI/CD pipeline
  - Files: `.github/workflows/ui-tests.yml`
  - Steps:
    - Install dependencies
    - Run linter (ESLint)
    - Run unit tests (Vitest)
    - Run E2E tests (Playwright)
    - Run accessibility tests (axe)
    - Build production bundle
    - Check bundle size
  - Purpose: Automated quality gates
  - _Leverage: GitHub Actions_
  - _Requirements: Non-functional_
  - _Success: ✅ All checks pass in CI, ✅ Bundle size within limits, ✅ No accessibility violations

---

## Summary Statistics

**Total Tasks**: 40
**Estimated Effort**: 80-100 hours (3-4 weeks full-time)

**By Phase**:
- Phase 0 (Environment Setup): 1 task, ~4 hours
- Phase 1 (Design System & Core Components): 6 tasks, ~20 hours
- Phase 2 (Layout Components): 4 tasks, ~12 hours
- Phase 3 (Feature Pages): 8 tasks, ~24 hours
- Phase 4 (API Integration): 4 tasks, ~8 hours
- Phase 5 (Responsive & Polish): 4 tasks, ~8 hours
- Phase 6 (Accessibility): 4 tasks, ~8 hours
- Phase 7 (Testing & Quality): 6 tasks, ~12 hours
- Phase 8 (Production & Deployment): 3 tasks, ~4 hours

**Milestones**:
- ✅ Phase 0 complete → Development environment ready
- ✅ Phase 1 complete → Design system and reusable components built
- ✅ Phase 2 complete → Application shell and navigation working
- ✅ Phase 3 complete → All feature pages implemented
- ✅ Phase 4 complete → API integration and real-time updates working
- ✅ Phase 5 complete → Responsive design and polish finished
- ✅ Phase 6 complete → WCAG 2.1 AA accessibility verified
- ✅ Phase 7 complete → Comprehensive testing coverage achieved
- ✅ Phase 8 complete → **Production-ready web UI embedded in daemon**

**Critical Path**: Phases 0-3 (foundation), then Phases 4-8 (integration and quality)

**Dependencies**: Requires `web-ui-ux-comprehensive` CLI spec (v1.0) to be implemented first for API endpoints.

**Testing Philosophy**: Every component has unit tests. Every user flow has E2E tests. Accessibility verified with automated and manual testing.
