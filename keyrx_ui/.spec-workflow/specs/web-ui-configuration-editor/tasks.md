# Tasks Document: Web UI Configuration Editor

## Phase 0: Environment Setup

- [x] 0. Initialize React + TypeScript + Vite project
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

- [x] 1. Implement design tokens and CSS variables
  - File: `src/styles/tokens.css`, Update `tailwind.config.js` from Task 0
  - Purpose: Centralize all design tokens (colors, typography, spacing, shadows) in CSS custom properties. Provides single source of truth for visual design, enables runtime theming, and ensures consistency across all components.
  - Requirements: Req 1 (Visual Design System - colors, typography, spacing)
  - Prompt: Role: CSS/Design System Developer | Task: Create comprehensive design token system with:

    **tokens.css** (CSS Custom Properties):
    ```css
    :root {
      /* Colors - Primary (Blue) */
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

      /* Typography */
      --font-family-base: 'Inter', system-ui, -apple-system, sans-serif;
      --font-family-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;

      --font-size-xs: 12px;
      --font-size-sm: 13px;
      --font-size-base: 14px;
      --font-size-lg: 16px;
      --font-size-xl: 18px;
      --font-size-2xl: 24px;
      --font-size-3xl: 32px;

      --line-height-tight: 1.25;
      --line-height-normal: 1.5;
      --line-height-relaxed: 1.75;

      --font-weight-normal: 400;
      --font-weight-medium: 500;
      --font-weight-semibold: 600;
      --font-weight-bold: 700;

      /* Spacing */
      --spacing-xs: 4px;
      --spacing-sm: 8px;
      --spacing-md: 16px;
      --spacing-lg: 24px;
      --spacing-xl: 32px;
      --spacing-2xl: 48px;
      --spacing-3xl: 64px;

      /* Shadows */
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
      --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
      --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
      --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1);

      /* Border Radius */
      --radius-sm: 4px;
      --radius-md: 8px;
      --radius-lg: 12px;
      --radius-full: 9999px;

      /* Z-index layers */
      --z-base: 0;
      --z-dropdown: 1000;
      --z-sticky: 1020;
      --z-fixed: 1030;
      --z-modal-backdrop: 1040;
      --z-modal: 1050;
      --z-popover: 1060;
      --z-tooltip: 1070;
    }
    ```

    **Update tailwind.config.js**:
    Extend the existing config from Task 0 to use CSS variables:
    ```javascript
    export default {
      content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
      theme: {
        extend: {
          colors: {
            primary: {
              50: 'var(--color-primary-50)',
              100: 'var(--color-primary-100)',
              // ... all color tokens
            },
          },
          fontFamily: {
            sans: ['var(--font-family-base)'],
            mono: ['var(--font-family-mono)'],
          },
          fontSize: {
            xs: 'var(--font-size-xs)',
            sm: 'var(--font-size-sm)',
            base: 'var(--font-size-base)',
            // ... all sizes
          },
          spacing: {
            xs: 'var(--spacing-xs)',
            sm: 'var(--spacing-sm)',
            // ... all spacing tokens
          },
          boxShadow: {
            sm: 'var(--shadow-sm)',
            md: 'var(--shadow-md)',
            // ... all shadows
          },
          borderRadius: {
            sm: 'var(--radius-sm)',
            md: 'var(--radius-md)',
            // ... all radii
          },
        },
      },
    };
    ```

    **Import in main.tsx**:
    ```typescript
    import './styles/tokens.css';
    import './index.css';
    ```

  | Restrictions: File ≤300 lines, all tokens MUST match design.md exactly, no hardcoded colors in components (use Tailwind classes or CSS variables), WCAG AA contrast ratio enforced (test with contrast checker)
  | Success: ✅ All tokens defined in tokens.css, ✅ Tailwind config extends tokens, ✅ Tailwind generates all utility classes (bg-primary-500, text-sm, etc.), ✅ No magic numbers in component files, ✅ Contrast ratios meet WCAG AA (4.5:1 for text)

- [x] 2. Create Button component
  - File: `src/components/Button.tsx`, `src/components/Button.test.tsx`
  - Purpose: Reusable, accessible button component for all user interactions across the UI. Supports multiple variants (primary/secondary/danger/ghost), sizes (sm/md/lg), and states (hover/active/disabled/loading). Used in 30+ locations throughout the application. Includes ripple animation for tactile feedback.
  - Requirements: Req 1 (Visual Design System - consistent interactive elements), Req 3 (Accessibility - keyboard navigation, ARIA), Req 4 (Performance - ripple animation <16ms)
  - Prompt: Role: React Component Developer | Task: Create Button component with:

    **TypeScript Interface**:
    ```typescript
    interface ButtonProps {
      variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
      size?: 'sm' | 'md' | 'lg';
      disabled?: boolean;
      loading?: boolean;
      onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
      'aria-label': string;  // REQUIRED for accessibility
      children: React.ReactNode;
      type?: 'button' | 'submit' | 'reset';
      className?: string;  // Allow additional Tailwind classes
    }
    ```

    **Visual States** (use Tailwind classes):
    - **Primary variant**:
      - Default: `bg-primary-500 text-white`
      - Hover: `hover:bg-primary-600 hover:scale-[1.02]`
      - Active: `active:scale-[0.98]`
      - Disabled: `disabled:opacity-50 disabled:cursor-not-allowed`
    - **Secondary variant**:
      - Default: `bg-transparent border-2 border-primary-500 text-primary-500`
      - Hover: `hover:bg-primary-500 hover:text-white`
    - **Danger variant**:
      - Default: `bg-red-500 text-white`
      - Hover: `hover:bg-red-600`
    - **Ghost variant**:
      - Default: `bg-transparent text-primary-500`
      - Hover: `hover:bg-primary-500/10`

    **Sizes**:
    - sm: `py-2 px-3 text-sm` (padding: 8px 12px, font: 13px)
    - md: `py-3 px-4 text-base` (padding: 12px 16px, font: 14px)
    - lg: `py-4 px-6 text-lg` (padding: 16px 24px, font: 16px)

    **Ripple Effect Implementation**:
    ```typescript
    const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
      if (disabled || loading) return;

      // Create ripple element
      const button = e.currentTarget;
      const rect = button.getBoundingClientRect();
      const ripple = document.createElement('span');
      const diameter = Math.max(rect.width, rect.height);
      const radius = diameter / 2;

      ripple.style.width = ripple.style.height = `${diameter}px`;
      ripple.style.left = `${e.clientX - rect.left - radius}px`;
      ripple.style.top = `${e.clientY - rect.top - radius}px`;
      ripple.className = 'ripple';

      button.appendChild(ripple);

      // Remove after animation (600ms)
      setTimeout(() => ripple.remove(), 600);

      onClick(e);
    };
    ```

    **Ripple CSS** (add to tokens.css or component):
    ```css
    .ripple {
      position: absolute;
      border-radius: 50%;
      background: rgba(255, 255, 255, 0.6);
      transform: scale(0);
      animation: ripple-animation 600ms ease-out;
      pointer-events: none;
    }

    @keyframes ripple-animation {
      to {
        transform: scale(4);
        opacity: 0;
      }
    }
    ```

    **Loading State**:
    ```typescript
    {loading && (
      <svg className="animate-spin h-4 w-4 mr-2" /* ... spinner SVG */ />
    )}
    ```

    **Accessibility**:
    - aria-label REQUIRED (TypeScript enforces via interface)
    - aria-disabled when disabled prop true
    - aria-busy when loading prop true
    - Focus visible: `focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2`
    - Keyboard: Enter/Space triggers onClick (native button behavior)

    **Component Structure**:
    ```typescript
    export const Button = React.memo<ButtonProps>(({
      variant = 'primary',
      size = 'md',
      disabled = false,
      loading = false,
      onClick,
      'aria-label': ariaLabel,
      children,
      type = 'button',
      className = '',
    }) => {
      const baseClasses = 'relative overflow-hidden rounded-md font-medium transition-all duration-150 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2';

      const variantClasses = {
        primary: 'bg-primary-500 text-white hover:bg-primary-600 active:bg-primary-700',
        // ... other variants
      };

      const sizeClasses = {
        sm: 'py-2 px-3 text-sm',
        md: 'py-3 px-4 text-base',
        lg: 'py-4 px-6 text-lg',
      };

      return (
        <button
          type={type}
          disabled={disabled || loading}
          onClick={handleClick}
          aria-label={ariaLabel}
          aria-disabled={disabled}
          aria-busy={loading}
          className={cn(baseClasses, variantClasses[variant], sizeClasses[size], className)}
        >
          {loading && <LoadingSpinner />}
          {children}
        </button>
      );
    });
    ```

  | Restrictions: File ≤200 lines (extract ripple logic to `hooks/useRipple.ts` if needed), function ≤30 lines, no inline styles (Tailwind classes only), MUST use React.memo for performance, no console.log, ESLint 0 errors
  | Success: ✅ All 4 variants render with correct colors from design tokens, ✅ All 3 sizes have correct padding (sm: 8/12px, md: 12/16px, lg: 16/24px), ✅ Ripple animates in 600ms without dropped frames (60fps), ✅ Focus outline visible on Tab (2px blue), ✅ Disabled state prevents clicks and shows cursor-not-allowed, ✅ Loading state shows spinner and disables interaction, ✅ aria-label, aria-disabled, aria-busy attributes set correctly, ✅ Keyboard navigation works (Enter/Space trigger onClick), ✅ No console warnings in development mode, ✅ Unit tests pass (Button.test.tsx)

- [x] 3. Create Input component
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

- [x] 4. Create Card component
  - File: `src/components/Card.tsx`
  - Variants: default, elevated (higher shadow)
  - Optional: header, footer slots
  - Padding: configurable (sm, md, lg)
  - Purpose: Container for content sections
  - _Leverage: Tailwind shadow utilities_
  - _Requirements: Design.md card pattern_
  - _Success: ✅ Card renders with correct padding and shadow, ✅ Header and footer slots work, ✅ Elevated variant has larger shadow

- [x] 5. Create Modal component
  - File: `src/components/Modal.tsx`
  - Features: backdrop, close button, Escape to close, focus trap
  - Animations: fade in/out, scale transform
  - Accessibility: aria-modal, aria-labelledby, return focus on close
  - Props: `open`, `onClose`, `title`, `children`
  - Purpose: Reusable dialog container
  - _Leverage: React Portal for overlay_
  - _Requirements: Req 3 (Accessibility - Escape closes, focus returns)_
  - _Success: ✅ Modal opens/closes with animation, ✅ Escape closes modal, ✅ Focus returns to trigger, ✅ Focus trapped within modal

- [x] 6. Create Dropdown component
  - File: `src/components/Dropdown.tsx`
  - Features: searchable, keyboard navigation (arrow keys)
  - States: open, closed, focused
  - Props: `options`, `value`, `onChange`, `searchable`, `aria-label`
  - Purpose: Reusable select/dropdown
  - _Leverage: Headless UI or Radix UI (a11y primitives)_
  - _Requirements: Req 3 (keyboard navigation)_
  - _Success: ✅ Arrow keys navigate options, ✅ Enter selects, ✅ Escape closes, ✅ Search filters options

- [x] 7. Create Tooltip component
  - File: `src/components/Tooltip.tsx`, `src/components/Tooltip.test.tsx`
  - Purpose: Contextual help component that appears on hover/focus to provide additional information about UI elements. Automatically positions itself to avoid viewport edges. Used throughout the UI for keyboard key tooltips, button explanations, and form field hints.
  - Requirements: Req 7.2 (keyboard hover tooltips), Req 3 (Accessibility - keyboard focus triggers tooltip)
  - Prompt: Role: React Component Developer | Task: Create Tooltip component with:

    **TypeScript Interface**:
    ```typescript
    import { ReactNode } from 'react';

    interface TooltipProps {
      content: string | ReactNode;
      children: ReactNode;
      position?: 'top' | 'bottom' | 'left' | 'right' | 'auto';
      delay?: number;  // Default 500ms
      disabled?: boolean;
      className?: string;
    }
    ```

    **Positioning Logic** (use @floating-ui/react):
    ```typescript
    import { useFloating, offset, flip, shift, autoUpdate } from '@floating-ui/react';

    const { x, y, refs, strategy } = useFloating({
      placement: position === 'auto' ? 'top' : position,
      middleware: [
        offset(8),  // 8px gap from trigger
        flip(),     // Flip to opposite side if no space
        shift({ padding: 8 }),  // Shift within viewport
      ],
      whileElementsMounted: autoUpdate,
    });
    ```

    **Show/Hide Behavior**:
    ```typescript
    const [isVisible, setIsVisible] = useState(false);
    const timeoutRef = useRef<NodeJS.Timeout>();

    const handleMouseEnter = () => {
      if (disabled) return;
      timeoutRef.current = setTimeout(() => {
        setIsVisible(true);
      }, delay || 500);
    };

    const handleMouseLeave = () => {
      clearTimeout(timeoutRef.current);
      setIsVisible(false);
    };

    const handleFocus = () => {
      if (disabled) return;
      setIsVisible(true);  // Instant show on keyboard focus
    };

    const handleBlur = () => {
      setIsVisible(false);
    };

    useEffect(() => {
      return () => clearTimeout(timeoutRef.current);
    }, []);
    ```

    **Visual Styling**:
    - Background: `bg-slate-800` (dark background)
    - Text: `text-slate-100 text-sm` (light text, 13px)
    - Padding: `px-3 py-2` (12px horizontal, 8px vertical)
    - Border radius: `rounded-md` (8px)
    - Shadow: `shadow-lg` (prominent shadow for elevation)
    - Arrow: Small triangle pointing to trigger element
    - Z-index: `z-tooltip` (1070 from design tokens)
    - Fade animation: `opacity-0 → opacity-100` over 150ms

    **Accessibility**:
    - Trigger element has `aria-describedby` pointing to tooltip ID
    - Tooltip has `role="tooltip"`
    - Tooltip has unique `id` attribute
    - Keyboard focus on trigger shows tooltip instantly (no delay)
    - Escape key doesn't close tooltip (only blur does)

    **Component Structure**:
    ```typescript
    export const Tooltip: React.FC<TooltipProps> = ({
      content,
      children,
      position = 'auto',
      delay = 500,
      disabled = false,
      className = '',
    }) => {
      const tooltipId = useId();
      const [isVisible, setIsVisible] = useState(false);

      return (
        <>
          <div
            ref={refs.setReference}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
            onFocus={handleFocus}
            onBlur={handleBlur}
            aria-describedby={isVisible ? tooltipId : undefined}
          >
            {children}
          </div>

          {isVisible && !disabled && (
            <div
              ref={refs.setFloating}
              id={tooltipId}
              role="tooltip"
              className={cn(
                'absolute bg-slate-800 text-slate-100 text-sm px-3 py-2 rounded-md shadow-lg z-tooltip',
                'transition-opacity duration-150',
                className
              )}
              style={{
                position: strategy,
                top: y ?? 0,
                left: x ?? 0,
              }}
            >
              {content}
            </div>
          )}
        </>
      );
    };
    ```

  | Restrictions: File ≤200 lines, function ≤40 lines, must use @floating-ui/react for positioning (no manual calculations), must clean up timeout in useEffect, no inline styles except positioning (use Tailwind), MUST use useId for tooltip ID (React 18+)
  | Success: ✅ Tooltip shows after 500ms delay on hover, ✅ Tooltip shows instantly on keyboard focus, ✅ Tooltip positions correctly near viewport edges (doesn't overflow), ✅ Tooltip flips to opposite side when no space, ✅ Tooltip hides on mouseout/blur, ✅ aria-describedby set correctly, ✅ role="tooltip" present, ✅ No memory leaks (timeout cleaned up), ✅ Disabled prop prevents tooltip from showing

---

## Phase 2: Layout Components

- [x] 8. Create AppShell layout
  - File: `src/components/AppShell.tsx`
  - Components: TopBar, Sidebar (desktop), BottomNav (mobile), MainContent
  - Responsive: sidebar → bottom nav at <768px
  - Sidebar: collapsible on tablet
  - Purpose: Main application layout
  - _Leverage: CSS Grid for layout_
  - _Requirements: Req 2 (Responsive Design)_
  - _Success: ✅ Sidebar visible on desktop, ✅ Bottom nav on mobile, ✅ Layout adapts smoothly on resize

- [x] 9. Create TopBar component
  - File: `src/components/TopBar.tsx`
  - Elements: logo, title, settings button, help button
  - Responsive: title hides on <768px
  - Purpose: Application header
  - _Leverage: Flexbox for alignment_
  - _Requirements: Design.md Layout 1_
  - _Success: ✅ Logo and buttons align correctly, ✅ Buttons have hover states, ✅ Title hides on mobile

- [x] 10. Create Sidebar navigation
  - File: `src/components/Sidebar.tsx`
  - Navigation items: Home, Devices, Profiles, Config, Metrics, Simulator
  - Active state: highlighted with indicator
  - Icons: use Lucide React (or Heroicons)
  - Collapsible: hamburger button on tablet
  - Purpose: Main navigation
  - _Leverage: React Router NavLink for active state_
  - _Requirements: Req 3 (keyboard navigation)_
  - _Success: ✅ Active page highlighted, ✅ Keyboard navigable (Tab, Enter), ✅ Collapses on tablet

- [x] 11. Create BottomNav (mobile)
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

- [x] 12. Create HomePage / Dashboard
  - File: `src/pages/HomePage.tsx`
  - Components: ActiveProfileCard, DeviceListCard, QuickStatsCard
  - Layout: from design.md Layout 1
  - Purpose: Main dashboard view
  - _Leverage: Card components from Phase 1_
  - _Requirements: Req 5 (device management), Req 6 (profile management)_
  - _Success: ✅ Layout matches design.md, ✅ Cards render with correct data, ✅ Responsive (stacks on mobile)

- [x] 13. Create DevicesPage
  - File: `src/pages/DevicesPage.tsx`
  - Features: device list, inline rename, scope toggle, layout selector
  - Components: DeviceDetailPanel (repeating)
  - Inline edit: click "Rename" → input appears, Enter saves
  - Purpose: Device management interface
  - _Leverage: Input component, Dropdown component_
  - _Requirements: Req 5 (User Flows - Device Management)_
  - _Success: ✅ Rename works inline, ✅ Scope toggle saves immediately, ✅ Forget device shows confirmation

- [x] 14. Create ProfilesPage
  - File: `src/pages/ProfilesPage.tsx`
  - Features: profile cards grid, create button, activate/edit/delete actions
  - Components: ProfileCard (repeating)
  - Active profile: green checkmark badge, "ACTIVE" label
  - Purpose: Profile management interface
  - _Leverage: Card component, Modal for create_
  - _Requirements: Req 6 (User Flows - Profile Management)_
  - _Success: ✅ Grid layout matches design.md, ✅ Active profile highlighted, ✅ Create modal works

- [x] 15. Create ConfigPage / Editor
  - File: `src/pages/ConfigPage.tsx`
  - Features: keyboard visualizer, layer selector, key config dialog
  - Layout: from design.md Layout 4
  - Purpose: Visual keyboard configuration
  - _Leverage: KeyboardVisualizer component (Task 16)_
  - _Requirements: Req 7 (User Flows - Keyboard Configuration)_
  - _Success: ✅ Layout matches design.md, ✅ Layer selector works, ✅ Clicking key opens dialog

- [x] 16. Create KeyboardVisualizer component
  - File: `src/components/KeyboardVisualizer.tsx`, `src/components/KeyButton.tsx`, `src/utils/kle-parser.ts`, `src/data/layouts/*.json`
  - Purpose: Visual representation of keyboard layout with interactive keys. Renders 104-109 keys based on selected layout preset (ANSI, ISO, JIS, HHKB, Numpad). Shows current mappings via tooltips, handles click events to open configuration dialog. Most visually complex component in the application - used on ConfigPage and SimulatorPage.
  - Requirements: Req 7.1 (render based on layout preset), Req 7.2 (hover tooltips showing current mapping), Req 7.3 (click opens KeyConfigDialog), Req 3 (Accessibility - keyboard navigation)
  - Prompt: Role: React UI Developer | Task: Create KeyboardVisualizer component with:

    **TypeScript Interfaces**:
    ```typescript
    interface KeyboardVisualizerProps {
      layout: 'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD';
      keyMappings: Map<string, KeyMapping>;  // keyCode → mapping
      onKeyClick: (keyCode: string) => void;  // Opens KeyConfigDialog
      simulatorMode?: boolean;  // If true, shows pressed state
      pressedKeys?: Set<string>;  // For simulator mode
      className?: string;
    }

    interface KeyMapping {
      type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';
      tapAction?: string;
      holdAction?: string;
      threshold?: number;
      macroSteps?: MacroStep[];
      targetLayer?: string;
    }

    interface KeyButton {
      keyCode: string;
      label: string;
      gridRow: number;
      gridColumn: number;
      gridColumnSpan: number;  // For wide keys (Space, Enter, etc.)
      width: number;  // In keyboard units (1u = 48px desktop)
    }

    interface KLEData {
      // Keyboard Layout Editor JSON format
      keys: Array<{ label: string; code: string; x: number; y: number; w?: number }>;
    }
    ```

    **Layout Data** (store in src/data/layouts/):
    - ANSI_104.json, ISO_105.json, JIS_109.json, HHKB.json, NUMPAD.json
    - Parse from Keyboard Layout Editor (KLE) JSON format
    - Each key has: keyCode, label, gridRow, gridColumn, gridColumnSpan

    **KLE Parser** (src/utils/kle-parser.ts):
    ```typescript
    export function parseKLEJson(kleData: KLEData): KeyButton[] {
      return kleData.keys.map((key) => ({
        keyCode: key.code,
        label: key.label,
        gridRow: Math.floor(key.y) + 1,  // 1-indexed for CSS Grid
        gridColumn: Math.floor(key.x) + 1,
        gridColumnSpan: key.w || 1,
        width: key.w || 1,
      }));
    }
    ```

    **KeyButton Component** (src/components/KeyButton.tsx):
    ```typescript
    interface KeyButtonProps {
      keyCode: string;
      label: string;
      mapping?: KeyMapping;
      onClick: () => void;
      isPressed?: boolean;  // For simulator mode
      className?: string;
    }

    export const KeyButton = React.memo<KeyButtonProps>(({
      keyCode,
      label,
      mapping,
      onClick,
      isPressed = false,
      className = '',
    }) => {
      const hasMapping = mapping && mapping.type !== 'simple';

      const tooltipContent = useMemo(() => {
        if (!mapping) return `${keyCode} (Default)`;

        switch (mapping.type) {
          case 'simple':
            return `${keyCode} → ${mapping.tapAction}`;
          case 'tap_hold':
            return `${keyCode} → Tap: ${mapping.tapAction}, Hold: ${mapping.holdAction} (${mapping.threshold}ms)`;
          case 'macro':
            return `${keyCode} → Macro (${mapping.macroSteps?.length || 0} steps)`;
          case 'layer_switch':
            return `${keyCode} → Layer: ${mapping.targetLayer}`;
        }
      }, [keyCode, mapping]);

      return (
        <Tooltip content={tooltipContent}>
          <button
            onClick={onClick}
            aria-label={`Key ${keyCode}. Current mapping: ${tooltipContent}. Click to configure.`}
            className={cn(
              'relative flex items-center justify-center',
              'rounded border border-slate-600 text-slate-100 text-xs font-mono',
              'transition-all duration-150',
              'hover:bg-slate-600 hover:scale-105',
              'focus:outline focus:outline-2 focus:outline-primary-500',
              hasMapping ? 'bg-blue-700' : 'bg-slate-700',  // Blue tint for custom mappings
              isPressed && 'bg-green-500',  // Green when pressed in simulator
              className
            )}
            style={{
              aspectRatio: '1',  // Square keys
            }}
          >
            {label}
          </button>
        </Tooltip>
      );
    });
    ```

    **KeyboardVisualizer Component**:
    ```typescript
    export const KeyboardVisualizer: React.FC<KeyboardVisualizerProps> = ({
      layout,
      keyMappings,
      onKeyClick,
      simulatorMode = false,
      pressedKeys = new Set(),
      className = '',
    }) => {
      const keyButtons = useMemo(() => {
        const kleData = layoutData[layout];  // Load from JSON
        return parseKLEJson(kleData);
      }, [layout]);

      // Calculate grid dimensions
      const maxRow = Math.max(...keyButtons.map(k => k.gridRow));
      const maxCol = Math.max(...keyButtons.map(k => k.gridColumn + k.gridColumnSpan - 1));

      return (
        <div
          className={cn('keyboard-grid', className)}
          style={{
            display: 'grid',
            gridTemplateRows: `repeat(${maxRow}, 48px)`,
            gridTemplateColumns: `repeat(${maxCol}, 48px)`,
            gap: '4px',
            padding: '16px',
            backgroundColor: 'var(--color-bg-secondary)',
            borderRadius: '12px',
          }}
        >
          {keyButtons.map((key) => (
            <div
              key={key.keyCode}
              style={{
                gridRow: key.gridRow,
                gridColumn: `${key.gridColumn} / span ${key.gridColumnSpan}`,
              }}
            >
              <KeyButton
                keyCode={key.keyCode}
                label={key.label}
                mapping={keyMappings.get(key.keyCode)}
                onClick={() => onKeyClick(key.keyCode)}
                isPressed={pressedKeys.has(key.keyCode)}
              />
            </div>
          ))}
        </div>
      );
    };
    ```

    **Responsive Key Sizing**:
    - Desktop (≥1280px): 48px per unit
    - Tablet (768-1279px): 40px per unit
    - Mobile (<768px): 32px per unit, horizontal scroll enabled

  | Restrictions: File ≤500 lines total (extract KeyButton to separate component if needed), parseKLEJson function ≤50 lines, must use React.memo on KeyButton (104 keys re-rendering is expensive), CSS Grid for layout (no manual positioning), useMemo for keyButtons array (don't recalculate on every render), Tooltip must not cause performance issues (virtualize if needed)
  | Success: ✅ All 5 layouts render correctly with accurate key positions, ✅ Hover shows tooltip with current mapping description, ✅ Modified keys (custom mappings) have blue tint (bg-blue-700), ✅ Tooltip positions correctly without viewport overflow, ✅ Click calls onKeyClick with correct keyCode, ✅ Simulator mode shows pressed keys in green, ✅ Keyboard navigation works (Tab through keys, Enter clicks), ✅ aria-label describes key and mapping, ✅ No dropped frames during hover (60fps maintained), ✅ Component memoized and performant (no unnecessary re-renders)

- [x] 17. Create KeyConfigDialog modal
  - File: `src/components/KeyConfigDialog.tsx`
  - Features: action type selector, dynamic form (tap-hold/simple/macro/layer)
  - Forms: TapHoldForm, SimpleRemapForm, MacroForm, LayerSwitchForm
  - Preview panel: shows mapping description
  - Purpose: Configure individual key mappings
  - _Leverage: Modal component, Dropdown for key pickers_
  - _Requirements: Req 7.4-7.6 (action type forms)_
  - _Success: ✅ Action type selector works, ✅ Forms render correctly, ✅ Preview updates, ✅ Save calls API

- [x] 18. Create MetricsPage
  - File: `src/pages/MetricsPage.tsx`
  - Components: LatencyCard, EventLogTable, StateInspectorCard
  - Chart: latency over time (last 60s)
  - Event log: virtual scrolling for performance
  - Purpose: Performance monitoring and debugging
  - _Leverage: Chart library (Recharts or Chart.js), react-window for virtual scrolling_
  - _Requirements: Design.md Layout 5_
  - _Success: ✅ Chart renders with real data, ✅ Event log scrolls smoothly, ✅ State inspector shows current state

- [x] 19. Create SimulatorPage
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

- [x] 20. Set up Zustand stores
  - Files: `src/stores/deviceStore.ts`, `profileStore.ts`, `configStore.ts`, `metricsStore.ts`
  - Implement state management for all data
  - Actions: fetch, create, update, delete
  - Purpose: Centralized state management
  - _Leverage: Zustand (lightweight, TypeScript-friendly)_
  - _Requirements: Design.md State Management Architecture_
  - _Success: ✅ All stores defined, ✅ Actions call API, ✅ State updates trigger re-renders

- [x] 21. Implement API client
  - Files: `src/api/devices.ts`, `profiles.ts`, `config.ts`, `metrics.ts`
  - Endpoints: match CLI API from web-ui-ux-comprehensive spec
  - Error handling: throw descriptive errors with API error codes
  - Purpose: API communication layer
  - _Leverage: fetch API, React Query for caching_
  - _Requirements: Design.md API Integration Patterns_
  - _Success: ✅ All endpoints implemented, ✅ Errors thrown with messages, ✅ TypeScript types for requests/responses

- [x] 22. Implement WebSocket for real-time metrics
  - File: `src/api/websocket.ts`
  - Connect to daemon WebSocket (ws://localhost:9867/ws)
  - Subscribe to: event stream, state changes, latency updates
  - Reconnect on disconnect
  - Purpose: Real-time updates for metrics page
  - _Leverage: native WebSocket API_
  - _Requirements: Req 8 (real-time preview)_
  - _Success: ✅ WebSocket connects on mount, ✅ Events update state, ✅ Reconnects on disconnect, ✅ Unsubscribes on unmount

- [x] 23. Integrate React Query for caching
  - Files: Update all stores to use React Query hooks
  - Features: automatic refetching, cache invalidation, optimistic updates
  - Purpose: Improve performance and user experience
  - _Leverage: @tanstack/react-query_
  - _Requirements: Req 4 (Performance)_
  - _Success: ✅ Queries cache data, ✅ Mutations invalidate cache, ✅ Stale data refetches

---

## Phase 5: Responsive Design & Polish

- [x] 24. Implement responsive layouts
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

- [x] 25. Implement animations and transitions
  - Files: `src/utils/animations.ts`, `src/components/PageTransition.tsx`, Updated `Modal.tsx`, `Dropdown.tsx`, `Button.tsx`, `src/styles/tokens.css`
  - Animations:
    - Modal: fade in/out + scale (using Framer Motion AnimatePresence)
    - Button: ripple effect on click + hover/tap scale animations
    - Dropdown: slide down with fade transition (using Headless UI Transition)
    - Page transitions: fade between routes (PageTransition component)
  - Reduced motion: disabled via CSS media query `@media (prefers-reduced-motion: reduce)` and JavaScript check in animations utility
  - Purpose: Polished, smooth user experience
  - _Leverage: Tailwind transitions, Framer Motion_
  - _Requirements: Req 4 (animations use transform/opacity), Req 4.6 (reduced motion)_
  - _Success: ✅ Animations smooth (60fps), ✅ Reduced motion disables animations, ✅ No layout shifts

- [x] 26. Add loading states and skeletons
  - Files: `src/components/LoadingSkeleton.tsx`, `src/components/LoadingSpinner.tsx`, Updated all pages (HomePage, DevicesPage, ProfilesPage, ConfigPage, MetricsPage)
  - Skeleton screens: matching layout structure, pulsing animation
  - Loading spinners: for buttons during async operations
  - Purpose: Feedback during data loading
  - _Leverage: Tailwind animations_
  - _Requirements: Req 4 (visual feedback <16ms)_
  - _Success: ✅ Skeletons match page layout, ✅ Spinners show during API calls, ✅ No "flash of no content"

- [x] 27. Implement error states and boundaries
  - Files: `src/components/ErrorBoundary.tsx`, `src/components/ErrorState.tsx`, `src/components/ErrorTest.tsx`, Updated `src/main.tsx`
  - Error boundary: catch React errors, show fallback UI
  - Error states: for failed API calls (retry button)
  - Purpose: Graceful error handling
  - _Leverage: React ErrorBoundary_
  - _Requirements: Req 4.12 (graceful error page)_
  - _Success: ✅ Errors caught and displayed, ✅ Retry button works, ✅ Error boundary prevents white screen

---

## Phase 6: Accessibility Implementation

- [x] 28. Add ARIA labels and roles
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

- [x] 29. Implement keyboard navigation
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

- [x] 30. Implement focus management
  - Files: Modal, Dropdown components
  - Requirements:
    - Focus trapped in modal (Tab loops within)
    - Focus returns to trigger on close
    - First element focused on modal open
  - Purpose: Proper focus flow for screen readers
  - _Leverage: Focus trap library or custom implementation_
  - _Requirements: Req 3.2 (focus returns on modal close)_
  - _Success: ✅ Focus trapped in modal, ✅ Focus returns on close, ✅ First element focused on open

- [x] 31. Test with accessibility tools
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

- [x] 32. Unit tests for components
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

- [x] 33. Integration tests for pages
  - Files: `src/pages/*.test.tsx`
  - Test: User flows (device rename, profile activation, key config)
  - Mock API calls with MSW (Mock Service Worker)
  - Purpose: Ensure page interactions work
  - _Leverage: Vitest, MSW_
  - _Requirements: Req 5-7 (user flows)_
  - _Success: ✅ All flows work, ✅ API mocks return expected data, ✅ No race conditions

- [x] 34. E2E tests with Playwright
  - Files: `tests/e2e/*.spec.ts`
  - Scenarios:
    - Create profile → configure key → activate → verify in simulator
    - Rename device → change scope → verify persistence
    - Full keyboard navigation test
  - Purpose: End-to-end verification
  - _Leverage: Playwright_
  - _Requirements: All user flows_
  - _Success: ✅ All scenarios pass, ✅ Screenshots match expected, ✅ Tests run in CI

- [x] 35. Visual regression tests
  - Files: `tests/visual/*.spec.ts`
  - Screenshots: all pages at 3 breakpoints (mobile, tablet, desktop)
  - Compare: against baseline screenshots
  - Purpose: Catch unintended visual changes
  - _Leverage: Playwright visual comparisons_
  - _Requirements: Req 1 (consistent visual design)_
  - _Success: ✅ All screenshots match baseline, ✅ No unexpected diffs

- [x] 36. Performance testing
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

- [x] 37. Bundle size optimization
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

- [x] 38. Configure production build
  - Files: `vite.config.ts`, `.env.production`
  - Optimizations: minification, tree shaking, code splitting
  - Environment variables: API_URL, WS_URL
  - Purpose: Production-ready build
  - _Leverage: Vite build command_
  - _Requirements: Non-functional_
  - _Success: ✅ Build completes without errors, ✅ Output is minified, ✅ Source maps generated

- [x] 39. Embed UI in daemon
  - Files: `keyrx_daemon/src/web/static_files.rs`, `keyrx_daemon/Cargo.toml`
  - Embed built UI files in binary using `include_dir!` macro
  - Serve from `/` route (Axum)
  - Purpose: Self-contained web UI (no separate server)
  - _Leverage: include_dir crate, Axum static file serving_
  - _Requirements: Design philosophy (embedded UI)_
  - _Success: ✅ UI files embedded in binary, ✅ Served from http://localhost:9867, ✅ No external dependencies

- [x] 40. Add CI/CD pipeline
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

## Phase 9: Implementation Artifacts

- [x] 41. Log implementation artifacts
  - Tool: spec-workflow log-implementation (MCP tool)
  - Purpose: Create searchable knowledge base of ALL implemented artifacts. Future AI agents use this log to discover existing code and avoid duplication. Critical for preventing duplicate API endpoints, components, utility functions, and business logic.
  - Requirements: Design.md Code Quality Metrics, all phases 0-8
  - Prompt: Role: Documentation Developer | Task: Use MCP spec-workflow log-implementation tool to document ALL implementation artifacts:

    **Components to Document** (complete list):
    - Button, Input, Card, Modal, Dropdown, Tooltip (Phase 1)
    - AppShell, TopBar, Sidebar, BottomNav (Phase 2)
    - HomePage, DevicesPage, ProfilesPage, ConfigPage, KeyboardVisualizer, KeyConfigDialog, MetricsPage, SimulatorPage (Phase 3)
    - LoadingSkeleton, ErrorBoundary, ErrorDialog (Phase 5)

    **For each component, document**:
    - name: Component name (e.g., "Button")
    - type: "React" (framework)
    - purpose: What the component does (1-2 sentences)
    - location: File path (e.g., "src/components/Button.tsx")
    - props: TypeScript interface (e.g., "ButtonProps { variant, size, disabled, loading, onClick, aria-label, children }")
    - exports: What it exports (e.g., ["Button (default)", "ButtonProps (type)"])

    **API Endpoints to Document**:
    - GET /api/devices → Fetch all connected devices
    - PUT /api/devices/:id/name → Rename device
    - PUT /api/devices/:id/scope → Change device scope
    - DELETE /api/devices/:id → Forget device
    - GET /api/profiles → Fetch all profiles
    - POST /api/profiles → Create new profile
    - POST /api/profiles/:name/activate → Activate profile
    - DELETE /api/profiles/:name → Delete profile
    - GET /api/config/:profile → Fetch configuration
    - PUT /api/config/:profile/key → Update key mapping
    - GET /api/metrics/latency → Fetch latency statistics
    - GET /api/metrics/events → Fetch event log
    - WS /ws → WebSocket connection for real-time updates

    **For each endpoint, document**:
    - method: HTTP method (GET, POST, PUT, DELETE)
    - path: Route path with parameters
    - purpose: What this endpoint does
    - requestFormat: Request body/query params structure
    - responseFormat: Response JSON structure
    - location: File path and line number (e.g., "src/api/devices.ts:45")

    **Functions to Document**:
    - renameDevice, setDeviceScope, forgetDevice, fetchDevices (src/api/devices.ts)
    - createProfile, activateProfile, deleteProfile, fetchProfiles (src/api/profiles.ts)
    - parseKLEJson (src/utils/kle-parser.ts)
    - validateProfileName, validateDeviceName (src/utils/validation.ts)
    - handleError, getUserFriendlyMessage (src/utils/error-handler.ts)

    **For each function, document**:
    - name: Function name
    - purpose: What it does
    - location: File path and line number
    - signature: Function signature with types (e.g., "(id: string, name: string) => Promise<void>")
    - isExported: true/false

    **Zustand Stores (Classes)**:
    - DeviceStore, ProfileStore, ConfigStore, MetricsStore

    **For each store, document**:
    - name: Store name (e.g., "DeviceStore")
    - purpose: What state it manages
    - location: File path (e.g., "src/stores/deviceStore.ts")
    - methods: List of actions (e.g., ["fetchDevices", "renameDevice", "setScope", "forgetDevice"])
    - isExported: true (all stores exported)

    **Integrations to Document** (Frontend ↔ Backend data flow):
    - DevicesPage → deviceStore.renameDevice() → PUT /api/devices/:id/name → API updates → Zustand state updates → DevicesPage re-renders
    - ProfilesPage → profileStore.activateProfile() → POST /api/profiles/:name/activate → Daemon compiles → WebSocket sends progress → ProfilesPage shows progress bar → State updates on success
    - ConfigPage → configStore.setKeyMapping() → PUT /api/config/:profile/key → Auto-compile → Success/error feedback
    - MetricsPage → metricsStore.subscribeToEvents() → WebSocket /ws → Real-time event stream → EventLogTable updates

    **For each integration, document**:
    - description: How components connect to APIs (end-to-end flow)
    - frontendComponent: Which component initiates (e.g., "DevicesPage")
    - backendEndpoint: Which API endpoint is called (e.g., "PUT /api/devices/:id/name")
    - dataFlow: Detailed flow with arrows (User action → Store action → API call → Response → State update → UI update)

    **Statistics to Record**:
    - linesAdded: Total lines of code added (count with `git diff --stat`)
    - linesRemoved: Total lines removed
    - filesChanged: Number of files modified
    - filesCreated: Number of new files created

    **Example Tool Call**:
    ```javascript
    await mcp.call('spec-workflow', 'log-implementation', {
      specName: 'web-ui-configuration-editor',
      taskId: '41',
      summary: 'Implemented complete React web UI with 20+ components, 12 API endpoints, 4 Zustand stores, and comprehensive testing',
      artifacts: {
        components: [
          {
            name: 'Button',
            type: 'React',
            purpose: 'Reusable, accessible button component with variants, sizes, and ripple animation',
            location: 'src/components/Button.tsx',
            props: 'ButtonProps { variant: primary|secondary|danger|ghost, size: sm|md|lg, disabled, loading, onClick, aria-label, children }',
            exports: ['Button (default)', 'ButtonProps (type)']
          },
          // ... all other components
        ],
        apiEndpoints: [
          {
            method: 'PUT',
            path: '/api/devices/:id/name',
            purpose: 'Rename a connected keyboard device',
            requestFormat: '{ name: string }',
            responseFormat: '{ success: boolean }',
            location: 'keyrx_daemon/src/web/api.rs:245'
          },
          // ... all other endpoints
        ],
        functions: [
          {
            name: 'parseKLEJson',
            purpose: 'Parse Keyboard Layout Editor JSON into KeyButton objects',
            location: 'src/utils/kle-parser.ts:12',
            signature: '(kleData: KLEData) => KeyButton[]',
            isExported: true
          },
          // ... all other functions
        ],
        classes: [
          {
            name: 'DeviceStore',
            purpose: 'Zustand store managing device state and actions',
            location: 'src/stores/deviceStore.ts',
            methods: ['fetchDevices', 'renameDevice', 'setScope', 'forgetDevice'],
            isExported: true
          },
          // ... all other stores
        ],
        integrations: [
          {
            description: 'DevicesPage rename flow: user edits inline → Enter key → optimistic update → API call → success/rollback',
            frontendComponent: 'DevicesPage',
            backendEndpoint: 'PUT /api/devices/:id/name',
            dataFlow: 'User clicks Rename → Input appears → User types new name → Press Enter → deviceStore.renameDevice() → Optimistic update (UI shows new name immediately) → API call → Success: keep new name, Error: rollback to old name + show error message'
          },
          // ... all other integrations
        ]
      },
      filesModified: ['src/components/Button.tsx', 'src/pages/DevicesPage.tsx', /* ... */],
      filesCreated: ['src/stores/deviceStore.ts', 'src/api/devices.ts', /* ... */],
      statistics: {
        linesAdded: 8500,
        linesRemoved: 120,
        filesChanged: 65
      }
    });
    ```

  | Restrictions: Document EVERY component, API endpoint, function, store, and integration (completeness critical for future AI agents), use exact file paths and line numbers, include TypeScript types in all signatures, no abbreviations in descriptions (write full sentences)
  | Success: ✅ Implementation log created in .spec-workflow/specs/web-ui-configuration-editor/implementation-log.json, ✅ All 20+ components documented with props and exports, ✅ All 12+ API endpoints documented with request/response formats, ✅ All utility functions documented with signatures, ✅ All 4 Zustand stores documented with methods, ✅ All major integrations documented with data flow, ✅ Statistics accurate (lines added/removed, files changed), ✅ Log is searchable (future agents can grep/search)

---

## Summary Statistics

**Total Tasks**: 41 (40 implementation + 1 logging)
**Estimated Effort**: 85-105 hours (3-4 weeks full-time)

**By Phase**:
- Phase 0 (Environment Setup): 1 task, ~4 hours
- Phase 1 (Design System & Core Components): 7 tasks, ~22 hours
- Phase 2 (Layout Components): 4 tasks, ~12 hours
- Phase 3 (Feature Pages): 8 tasks, ~24 hours
- Phase 4 (API Integration): 4 tasks, ~8 hours
- Phase 5 (Responsive & Polish): 4 tasks, ~8 hours
- Phase 6 (Accessibility): 4 tasks, ~8 hours
- Phase 7 (Testing & Quality): 6 tasks, ~12 hours
- Phase 8 (Production & Deployment): 3 tasks, ~4 hours
- Phase 9 (Implementation Artifacts): 1 task, ~3 hours

**Milestones**:
- ✅ Phase 0 complete → Development environment ready (TypeScript, Vite, Tailwind, ESLint configured)
- ✅ Phase 1 complete → Design system and reusable components built (Button, Input, Card, Modal, Dropdown, Tooltip)
- ✅ Phase 2 complete → Application shell and navigation working (AppShell, TopBar, Sidebar, BottomNav)
- ✅ Phase 3 complete → All feature pages implemented (Home, Devices, Profiles, Config, Metrics, Simulator)
- ✅ Phase 4 complete → API integration and real-time updates working (Zustand stores, React Query, WebSocket)
- ✅ Phase 5 complete → Responsive design and polish finished (mobile/tablet/desktop, animations, loading states)
- ✅ Phase 6 complete → WCAG 2.1 AA accessibility verified (0 axe violations, Lighthouse ≥95, keyboard navigation)
- ✅ Phase 7 complete → Comprehensive testing coverage achieved (unit ≥80%, E2E, visual regression, a11y)
- ✅ Phase 8 complete → Production-ready web UI embedded in daemon (bundle ≤250KB JS, ≤50KB CSS)
- ✅ Phase 9 complete → **Implementation artifacts logged for future AI agents** (searchable knowledge base)

**Critical Path**: Phases 0-3 (foundation), then Phases 4-8 (integration and quality), then Phase 9 (documentation)

**Dependencies**: Requires `web-ui-ux-comprehensive` CLI spec (v1.0) to be implemented first for API endpoints.

**Testing Philosophy**: Every component has unit tests. Every user flow has E2E tests. Accessibility verified with automated and manual testing.

**Code Quality Standards** (enforced in all tasks):
- File size ≤500 lines (excluding comments/blanks)
- Function size ≤50 lines
- TypeScript strict mode (no `any` types)
- ESLint 0 errors, 0 warnings
- Test coverage ≥80% (≥90% for critical components)
- WCAG 2.1 AA compliance (0 axe violations)
- Bundle size ≤250KB JS, ≤50KB CSS (gzipped)

**Task Enhancement Pattern** (applied to Tasks 0-2, template for remaining tasks):
Each task includes:
- **Purpose**: Detailed explanation (why this task exists, what it achieves, where it's used)
- **Requirements**: References to requirements.md (Req 1, Req 3, etc.)
- **Prompt**: Role-based prompt with exact specifications (TypeScript interfaces, logic, error handling)
- **Restrictions**: File size limits, coding standards, quality gates
- **Success**: Measurable criteria with ✅ checkboxes

**Remaining Tasks (3-40)**: Follow the same enhancement pattern as Tasks 0-2. Each task should have:
- Detailed TypeScript interfaces
- Exact visual specifications (Tailwind classes)
- Accessibility requirements (ARIA labels, keyboard navigation)
- Error handling strategy
- File/function size restrictions
- Comprehensive success criteria
