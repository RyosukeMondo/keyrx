# Keyboard Navigation Guide

This document describes the keyboard navigation features implemented in the KeyRx2 web UI.

## Table of Contents

- [Global Keyboard Shortcuts](#global-keyboard-shortcuts)
- [Navigation](#navigation)
- [Components](#components)
- [Accessibility Features](#accessibility-features)
- [Testing](#testing)

## Global Keyboard Shortcuts

The application supports the following global keyboard shortcuts:

| Shortcut | Action | Available Where |
|----------|--------|-----------------|
| `Ctrl+B` / `⌘B` | Toggle sidebar | All pages |
| `Escape` | Close modal/dialog | When modal is open |
| `Tab` | Move focus forward | All pages |
| `Shift+Tab` | Move focus backward | All pages |

## Navigation

### Skip to Content

Press `Tab` when the page loads to reveal a "Skip to main content" link. Press `Enter` to skip navigation and jump directly to the main content area.

### Sidebar Navigation

- Use `Tab` to navigate through sidebar items
- Press `Enter` or `Space` to activate a navigation link
- Press `Escape` to close sidebar on mobile/tablet

### Keyboard Visualizer

On the Configuration page the keyboard is a single Tab stop (roving
tabindex). `Tab` enters it; inside it:

- `Arrow keys` - Move to the nearest key in that direction (spatial, not DOM order)
- `Home` / `End` - First / last key of the current row
- `Enter` or `Space` - Select the focused key: focus moves to the Key editor
  (scrolled into view) and the choice is announced through a polite live region
- `Escape` (in the editor) - Deselect and return focus to the key

Keys are named with their DSL names (`CapsLock`, `LCtrl`), never QMK codes
(`KC_CAPS`). In the read-only Test (simulator) tab the keys are not focusable.

## Components

### Modal

- `Escape` - Close modal
- `Tab` - Navigate through focusable elements (focus trapped within modal)
- `Shift+Tab` - Navigate backward (focus trapped within modal)
- Focus automatically returns to the element that opened the modal when closed

### Dropdown

The dropdown component (using Headless UI) supports:

- `Enter` or `Space` - Open dropdown
- `Arrow Up/Down` - Navigate options
- `Enter` - Select option
- `Escape` - Close dropdown
- `Home` - Jump to first option
- `End` - Jump to last option
- When searchable: Type to filter options

### Button

- `Enter` or `Space` - Activate button
- Button shows 2px blue focus outline when focused

### Input Fields

- Standard browser keyboard navigation
- `Enter` - Submit form or save inline edits (where applicable)
- `Escape` - Cancel inline edits (where applicable)

### Device Management (DevicesPage)

**Inline Rename:**
- Click "Rename" button or tab to it and press `Enter`
- Type new name
- Press `Enter` to save
- Press `Escape` to cancel

**Scope Selection:**
- Tab to scope buttons
- Press `Enter` or `Space` to select scope (Global or Device-Specific)
- Radio button semantics with `role="radio"` and `aria-checked`

### Layer Selection (ConfigPage)

- The layer list is a listbox and a single Tab stop; it shows Base plus the
  layers in use. "All layers" reveals MD_00..MD_FF.
- `Up` / `Down` / `Home` / `End` move; `Enter` or `Space` activates a layer
- Current layer indicated with `aria-selected="true"`

### Profile Cards

- Tab to navigate between cards
- Tab to "Activate", "Edit", or "Delete" buttons
- Press `Enter` or `Space` to trigger action

## Accessibility Features

### Focus Indicators

All interactive elements show a visible focus indicator:
- 2px blue outline (`--color-primary-500`)
- 2px offset from element
- High contrast for visibility

### ARIA Attributes

Components use appropriate ARIA attributes:
- `aria-label` - Descriptive labels for all interactive elements
- `aria-pressed` - Toggle button states
- `aria-checked` - Radio button states
- `aria-invalid` - Form validation errors
- `aria-live="assertive"` - Error message announcements
- `aria-modal` - Modal dialogs
- `role="dialog"` - Dialog semantics
- `role="radio"` - Radio button semantics
- `role="group"` - Grouped controls

### Tab Order

Tab order follows the visual layout:
1. Skip to content link (hidden until focused)
2. Navigation (sidebar or bottom nav)
3. Main content area (top to bottom, left to right)

### Focus Management

- Focus trapped in modal dialogs
- Focus returns to trigger element when modal closes
- Logical tab order maintained on all pages
- No keyboard traps (users can always navigate away)

## Testing

### Manual Testing

1. **Tab Order Test:**
   - Load any page
   - Press `Tab` repeatedly
   - Verify focus moves in logical order
   - Verify all interactive elements are reachable

2. **Focus Visibility Test:**
   - Tab through all interactive elements
   - Verify focus indicator is visible (2px blue outline)
   - Verify focus indicator is not obscured

3. **Keyboard-Only Navigation Test:**
   - Unplug mouse
   - Navigate entire application using only keyboard
   - Verify all features are accessible

4. **Modal Focus Trap Test:**
   - Open a modal
   - Tab through all elements
   - Verify focus wraps from last element to first
   - Press `Escape` and verify modal closes
   - Verify focus returns to trigger element

5. **Screen Reader Test:**
   - Use NVDA (Windows) or VoiceOver (Mac)
   - Navigate application
   - Verify all elements are announced correctly
   - Verify labels are descriptive

### Automated Testing

Run keyboard navigation tests:

```bash
npm run test:a11y
```

Tests verify:
- Focus management hooks work correctly
- Keyboard utilities function as expected
- Components trap focus when required
- Focus returns correctly

## Browser Support

Keyboard navigation is tested and supported on:

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Troubleshooting

### Focus Indicator Not Visible

If focus indicators are not visible:
1. Check browser settings - some browsers allow disabling focus outlines
2. Verify CSS is loading correctly (check for `focus:outline` classes)
3. Check for conflicting CSS that sets `outline: none`

### Focus Trapped in Component

If you can't tab out of a component:
1. This is intentional for modals (press `Escape` to close)
2. For other components, this is a bug - please report

### Skip to Content Link Not Appearing

The skip link is visually hidden until focused:
1. Press `Tab` once when page loads
2. Link should appear at top of page
3. If not, check z-index and positioning

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Keyboard Accessibility](https://webaim.org/articles/keyboard/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
