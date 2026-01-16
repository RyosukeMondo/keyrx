# Requirements: Fix Accessibility Landmark Violation

## Overview
Fix 1 failing accessibility test: landmark-unique violation in ConfigPage where regions need unique aria-label attributes.

## User Stories

### 1. Add unique aria-labels to landmark regions
**EARS**: WHEN using screen readers, THEN all landmark regions are distinguishable, SO THAT navigation is accessible.

**Acceptance**: All regions have unique aria-label or aria-labelledby attributes

## Technical Requirements
- Fix landmark-unique violation in ConfigPage
- All accessibility tests pass (35/35)
- WCAG 2.1 AA compliance maintained

## Success Metrics
- `npm run test:a11y` → 35/35 pass (100%)
- axe-core reports 0 violations
