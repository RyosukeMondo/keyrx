# Accessibility Checklist

Quick reference checklist for ensuring WCAG 2.1 Level AA compliance.

## Perceivable

### Text Alternatives

- [ ] All images have alt text
- [ ] Decorative images have empty alt (`alt=""`)
- [ ] Form inputs have labels or aria-label
- [ ] Icons have accessible names
- [ ] SVGs have title and desc elements

### Time-based Media

- [ ] Videos have captions (if applicable)
- [ ] Audio has transcripts (if applicable)

### Adaptable

- [ ] Proper heading hierarchy (h1 → h2 → h3, no skips)
- [ ] Semantic HTML used (nav, main, article, section, aside, footer)
- [ ] Reading order matches visual order
- [ ] Forms use fieldset and legend for groups

### Distinguishable

- [ ] Text color contrast ≥4.5:1 (normal text)
- [ ] Text color contrast ≥3:1 (large text, 18pt or 14pt bold)
- [ ] UI component contrast ≥3:1
- [ ] No information conveyed by color alone
- [ ] Text can be resized to 200% without loss of functionality
- [ ] No horizontal scrolling at 320px width (mobile)

## Operable

### Keyboard Accessible

- [ ] All functionality available via keyboard
- [ ] Tab order is logical
- [ ] Focus visible on all interactive elements (2px outline)
- [ ] No keyboard trap (can navigate away from all elements)
- [ ] Skip navigation link provided

### Enough Time

- [ ] No time limits (or time limits can be adjusted/extended)
- [ ] Animations can be paused/stopped

### Seizures and Physical Reactions

- [ ] No flashing content (≥3 flashes per second)
- [ ] Animations respect prefers-reduced-motion

### Navigable

- [ ] Page has descriptive title
- [ ] Link text describes destination
- [ ] Multiple ways to find pages (navigation, sitemap, search)
- [ ] Headings describe page structure
- [ ] Current location is visible (breadcrumbs, active state)

### Input Modalities

- [ ] Gestures have keyboard alternatives
- [ ] Touch targets ≥44x44px
- [ ] Click actions don't require precision

## Understandable

### Readable

- [ ] Page language specified (`<html lang="en">`)
- [ ] Language changes marked (`<span lang="ja">`)
- [ ] Abbreviations explained (title or aria-label)

### Predictable

- [ ] Navigation consistent across pages
- [ ] Interactive elements behave consistently
- [ ] No unexpected context changes on focus
- [ ] No unexpected context changes on input

### Input Assistance

- [ ] Error messages are clear and specific
- [ ] Labels or instructions provided for inputs
- [ ] Error prevention for critical actions (confirm dialogs)
- [ ] Suggestions provided for errors

## Robust

### Compatible

- [ ] Valid HTML (no parsing errors)
- [ ] ARIA attributes used correctly
- [ ] Status messages use aria-live
- [ ] Name, role, and value available for all components

## Component-Specific Checklist

### Button

- [ ] Has accessible name (aria-label or text content)
- [ ] Has role="button" if not using `<button>` element
- [ ] Shows focus indicator
- [ ] Responds to Enter and Space keys
- [ ] aria-disabled when disabled
- [ ] aria-busy when loading

### Input

- [ ] Has associated label or aria-label
- [ ] aria-invalid when error
- [ ] aria-required when required
- [ ] Error message has id and aria-describedby
- [ ] Placeholder not used as label

### Dropdown/Select

- [ ] Has label or aria-label
- [ ] Opens with Enter/Space
- [ ] Navigates with arrow keys
- [ ] Selects with Enter
- [ ] Closes with Escape
- [ ] Focus returns to trigger on close
- [ ] Selected value announced

### Modal/Dialog

- [ ] role="dialog"
- [ ] aria-modal="true"
- [ ] aria-labelledby points to title
- [ ] Focus trapped within modal
- [ ] Closes with Escape
- [ ] Focus returns to trigger on close
- [ ] First focusable element receives focus

### Tooltip

- [ ] role="tooltip"
- [ ] Trigger has aria-describedby
- [ ] Shows on keyboard focus
- [ ] Hides on blur/Escape
- [ ] Not essential information (also in label)

### Card

- [ ] Has semantic structure (article or section)
- [ ] Heading describes card content
- [ ] Links have descriptive text

### Navigation

- [ ] Wrapped in `<nav>` element
- [ ] Has aria-label or aria-labelledby
- [ ] Current page marked with aria-current="page"
- [ ] Links have descriptive text

### Form

- [ ] Has submit button
- [ ] Validation errors announced (aria-live)
- [ ] Required fields marked (aria-required)
- [ ] Error messages associated (aria-describedby)
- [ ] Success messages announced (aria-live)

## Testing Tools

### Automated

- [ ] axe DevTools (browser extension) - 0 violations
- [ ] Lighthouse accessibility audit - score ≥95
- [ ] WAVE (browser extension) - 0 errors

### Manual

- [ ] Keyboard navigation - all features work
- [ ] Screen reader (NVDA/JAWS/VoiceOver) - all content announced
- [ ] Zoom to 200% - no loss of functionality
- [ ] Color contrast checker - all ratios pass

## Quick Fixes

### Missing alt text
```tsx
// Before
<img src="logo.png" />

// After
<img src="logo.png" alt="KeyRx Logo" />
```

### Button without label
```tsx
// Before
<button onClick={close}>×</button>

// After
<button onClick={close} aria-label="Close">×</button>
```

### Form input without label
```tsx
// Before
<input type="text" placeholder="Name" />

// After
<label htmlFor="name">Name</label>
<input id="name" type="text" aria-label="Device name" />
```

### Low color contrast
```css
/* Before */
.text {
  color: #888; /* Contrast ratio: 2.9:1 - FAIL */
  background: #fff;
}

/* After */
.text {
  color: #666; /* Contrast ratio: 5.7:1 - PASS */
  background: #fff;
}
```

### Missing focus indicator
```css
/* Before */
button:focus {
  outline: none; /* DON'T DO THIS */
}

/* After */
button:focus {
  outline: 2px solid #3B82F6;
  outline-offset: 2px;
}
```

### Keyboard trap
```tsx
// Before
const Modal = () => {
  return <div>{children}</div>;
};

// After
const Modal = () => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return <div role="dialog" aria-modal="true">{children}</div>;
};
```

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [axe-core Rules](https://github.com/dequelabs/axe-core/blob/develop/doc/rule-descriptions.md)
