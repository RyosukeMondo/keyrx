# Accessibility Testing Guide

This guide documents accessibility testing procedures for the KeyRx Web UI v2.

## Requirements

- WCAG 2.1 Level AA compliance
- 0 axe-core violations in automated scan
- Lighthouse accessibility score ≥95
- Manual testing with screen readers (NVDA/JAWS)
- Keyboard-only navigation support

## Automated Testing

### 1. axe-core Development Mode

axe-core runs automatically in development mode and logs violations to the browser console.

**To enable:**
```bash
npm run dev
```

Open browser console (F12) and check for axe-core violation messages.

### 2. Playwright Accessibility Tests

Run automated accessibility tests with Playwright:

```bash
npm run test:a11y
```

This runs:
- axe-core scans on all pages
- WCAG 2.1 Level A and AA rule checks
- Color contrast validation
- ARIA attribute validation
- Keyboard navigation tests

**To run specific test:**
```bash
npx playwright test tests/accessibility.spec.ts
```

### 3. Lighthouse Audits

Run Lighthouse performance and accessibility audits:

```bash
npm run test:lighthouse
```

This generates reports in `./lighthouse-reports/` directory.

**Requirements:**
- Chromium browser
- Dev server running on http://localhost:5173
- Chrome debugging port 9222

**To view Lighthouse reports:**
```bash
open lighthouse-reports/HomePage-*.html
```

## Manual Testing

### Keyboard Navigation Testing

**Requirements:**
- All features accessible via keyboard only
- Focus visible (2px outline)
- Tab order logical (top to bottom, left to right)
- Escape closes modals/dropdowns

**Test procedure:**
1. Navigate to each page using only keyboard
2. Press Tab to move through interactive elements
3. Verify focus indicator is visible on all elements
4. Press Enter/Space to activate buttons and links
5. Use arrow keys to navigate dropdowns and lists
6. Press Escape to close modals and dropdowns
7. Verify focus returns to trigger element after modal closes

**Pages to test:**
- HomePage (/)
- DevicesPage (/devices)
- ProfilesPage (/profiles)
- ConfigPage (/config)
- MetricsPage (/metrics)
- SimulatorPage (/simulator)

### Screen Reader Testing

**Tools:**
- NVDA (Windows) - https://www.nvaccess.org/download/
- JAWS (Windows) - https://www.freedomscientific.com/products/software/jaws/
- VoiceOver (macOS) - Built-in (Cmd+F5)

**Test procedure:**

#### NVDA (Windows)
1. Install NVDA and restart computer
2. Launch NVDA (Ctrl+Alt+N)
3. Navigate to http://localhost:5173
4. Use arrow keys to navigate through content
5. Use Tab to navigate interactive elements
6. Verify all elements are announced correctly
7. Verify landmarks are announced (navigation, main, etc.)

**Key NVDA commands:**
- Insert+F7: List of elements
- Insert+Down: Read all
- H: Next heading
- Tab: Next interactive element
- Enter: Activate element

#### VoiceOver (macOS)
1. Enable VoiceOver (Cmd+F5)
2. Navigate to http://localhost:5173
3. Use VO+arrow keys to navigate
4. Verify all elements are announced correctly

**Key VoiceOver commands:**
- VO+Right/Left: Navigate
- VO+Space: Activate
- VO+U: Rotor (lists)
- VO+A: Start reading

### Color Contrast Testing

**Tools:**
- WebAIM Contrast Checker - https://webaim.org/resources/contrastchecker/
- axe DevTools (browser extension)

**Test procedure:**
1. Install axe DevTools browser extension
2. Navigate to each page
3. Click axe DevTools icon
4. Run "Full Page Scan"
5. Check "Color Contrast" results
6. Verify all text meets WCAG AA (4.5:1 for normal text, 3:1 for large text)

**Manual verification:**
1. Use browser DevTools to inspect text elements
2. Check computed styles for color and background-color
3. Use WebAIM Contrast Checker to verify ratios:
   - Normal text: 4.5:1 minimum
   - Large text (18pt or 14pt bold): 3:1 minimum
   - UI components: 3:1 minimum

## Common Issues and Fixes

### Missing ARIA Labels

**Issue:** Buttons or inputs without accessible names

**Fix:**
```tsx
// Bad
<button onClick={handleClick}>X</button>

// Good
<button onClick={handleClick} aria-label="Close modal">X</button>
```

### Incorrect ARIA Roles

**Issue:** Custom elements without proper roles

**Fix:**
```tsx
// Bad
<div onClick={handleClick}>Click me</div>

// Good
<button onClick={handleClick} aria-label="Action">Click me</button>
```

### Missing Focus Indicators

**Issue:** Focus not visible on interactive elements

**Fix:**
```css
/* Add to component */
.button:focus {
  outline: 2px solid var(--color-primary-500);
  outline-offset: 2px;
}
```

### Color Contrast Failures

**Issue:** Text doesn't have sufficient contrast with background

**Fix:**
1. Check design tokens in `src/styles/tokens.css`
2. Verify colors meet WCAG AA ratios
3. Adjust colors if needed (lighter text or darker backgrounds)

### Form Input Without Labels

**Issue:** Input fields without associated labels

**Fix:**
```tsx
// Bad
<input type="text" placeholder="Name" />

// Good
<label htmlFor="name-input">Name</label>
<input id="name-input" type="text" aria-label="Device name" />
```

## Test Coverage

### Automated Tests Coverage

- ✅ All pages scanned for WCAG 2.1 Level A and AA violations
- ✅ Color contrast checked on all pages
- ✅ ARIA attributes validated
- ✅ Keyboard navigation verified
- ✅ Focus management tested (modals, dropdowns)
- ✅ Form inputs checked for labels
- ✅ Images checked for alt text
- ✅ Heading hierarchy validated
- ✅ Link text verified

### Manual Tests Coverage

- □ Screen reader testing with NVDA
- □ Screen reader testing with JAWS
- □ Screen reader testing with VoiceOver
- □ Keyboard-only navigation on all pages
- □ Touch target size verification (≥44px)
- □ Zoom testing (up to 200%)
- □ Reduced motion testing

## Continuous Integration

Accessibility tests run automatically in CI/CD pipeline:

```yaml
# .github/workflows/ui-tests.yml
- name: Run accessibility tests
  run: npm run test:a11y

- name: Run Lighthouse audits
  run: npm run test:lighthouse

- name: Upload Lighthouse reports
  uses: actions/upload-artifact@v3
  with:
    name: lighthouse-reports
    path: lighthouse-reports/
```

## Resources

- WCAG 2.1 Guidelines: https://www.w3.org/WAI/WCAG21/quickref/
- axe-core Rules: https://github.com/dequelabs/axe-core/blob/develop/doc/rule-descriptions.md
- ARIA Practices: https://www.w3.org/WAI/ARIA/apg/
- WebAIM: https://webaim.org/
- Lighthouse Documentation: https://developer.chrome.com/docs/lighthouse/

## Success Criteria

- [x] axe-core integrated in development mode
- [x] Playwright accessibility tests created
- [x] Lighthouse audit tests created
- [ ] 0 axe-core violations on all pages
- [ ] Lighthouse accessibility score ≥95 on all pages
- [ ] Manual screen reader testing passed
- [ ] Keyboard navigation testing passed
- [ ] Color contrast validation passed
