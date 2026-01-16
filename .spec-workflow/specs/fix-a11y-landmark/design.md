# Design: Fix Accessibility Landmark Violation

## Issue
Multiple `<div role="region">` elements in ConfigPage lack unique identifying labels.

**WCAG Requirement**: Landmarks with same role must have unique labels (aria-label or aria-labelledby)

## Fix Strategy
1. Identify all `role="region"` elements in ConfigPage and related components
2. Add descriptive `aria-label` to each region
3. Ensure labels are unique and descriptive

## Example Fix
```tsx
// Before
<div className="bg-slate-800 border..." role="region">

// After
<div className="bg-slate-800 border..." role="region" aria-label="Keyboard Configuration Panel">
```

## Regions to Label
- Keyboard visualizer region
- Configuration panel region
- Code editor region
- Key palette region
