# Performance Testing

Comprehensive performance testing suite for KeyRx UI v2, measuring Core Web Vitals and bundle sizes.

## Overview

This directory contains performance tests that validate the application meets defined performance budgets:

- **Core Web Vitals**: LCP, FCP, TTI, CLS, FID
- **Bundle Sizes**: JavaScript, CSS, total assets
- **Asset Loading**: Resource timing, caching, compression

## Performance Budgets

### Core Web Vitals
- **LCP** (Largest Contentful Paint): < 2.5s
- **FCP** (First Contentful Paint): < 1.5s
- **TTI** (Time to Interactive): < 3.0s
- **CLS** (Cumulative Layout Shift): < 0.1
- **FID** (First Input Delay): < 100ms

### Bundle Sizes
- **Total JavaScript**: ≤ 250KB (gzipped)
- **Total CSS**: ≤ 50KB (gzipped)
- **Largest JS chunk**: ≤ 150KB
- **Total Assets** (JS + CSS + fonts): ≤ 500KB

## Test Files

### `core-web-vitals.spec.ts`
Measures all five Core Web Vitals metrics for each page route:
- LCP: Time to render largest content element
- FCP: Time to first paint of any content
- TTI: Time until page is fully interactive
- CLS: Visual stability score (layout shifts)
- FID: Input delay on first user interaction

Generates detailed performance reports with per-route metrics.

### `bundle-analysis.spec.ts`
Analyzes bundle sizes and asset loading:
- Total JavaScript and CSS sizes
- Individual chunk sizes
- Lazy loading verification (code splitting)
- Compression detection (gzip/brotli)
- Resource loading times
- Cache header analysis

## Running Tests

### Prerequisites
1. Start the dev server:
   ```bash
   npm run dev
   ```

2. Ensure dev server is running on `http://localhost:5173`

### Run All Performance Tests
```bash
npm run test:performance
```

### Run Specific Test Suites
```bash
# Core Web Vitals only
npx playwright test tests/performance/core-web-vitals.spec.ts

# Bundle analysis only
npx playwright test tests/performance/bundle-analysis.spec.ts
```

### Run with UI (Debug Mode)
```bash
npx playwright test tests/performance --ui
```

### Generate HTML Report
```bash
npx playwright test tests/performance --reporter=html
npx playwright show-report
```

## Understanding Results

### Passing Tests
When all tests pass, you'll see:
```
✓ HomePage should meet all Core Web Vitals budgets
✓ DevicesPage should meet all Core Web Vitals budgets
✓ Should meet JavaScript bundle size budget
```

### Failing Tests
If tests fail, you'll see detailed error messages:
```
Error: LCP should be < 2500ms, got 3245.67ms
```

Check the console output for detailed metrics and identify optimization opportunities.

### Performance Reports

Tests output detailed console logs:

**Core Web Vitals Report:**
```
HomePage Performance Metrics:
  LCP: 1245.67ms (budget: 2500ms) ✓
  FCP: 892.34ms (budget: 1500ms) ✓
  TTI: 1567.89ms (budget: 3000ms) ✓
  CLS: 0.0234 (budget: 0.1) ✓
  FID: 12.45ms (budget: 100ms) ✓
```

**Bundle Size Report:**
```
=== Bundle Size Analysis ===

JavaScript:
  - index-abc123.js: 142.34 KB (compressed)
  - vendor-def456.js: 78.90 KB (compressed)
  Total JS: 221.24 KB ✓

CSS:
  - index-ghi789.css: 34.56 KB (compressed)
  Total CSS: 34.56 KB ✓

Total Assets: 255.80 KB ✓
```

## Troubleshooting

### Dev Server Not Running
If tests fail with connection errors:
```bash
# Start dev server in another terminal
npm run dev
```

### Metrics Exceed Budgets
If metrics exceed budgets, consider:

**For LCP/FCP/TTI:**
- Reduce bundle sizes (code splitting)
- Optimize images (use WebP, lazy loading)
- Minimize render-blocking resources
- Use React.lazy() for route components

**For CLS:**
- Reserve space for dynamic content (skeleton loaders)
- Specify image dimensions
- Avoid inserting content above existing content

**For Bundle Size:**
- Analyze bundle with `vite-plugin-visualizer`
- Remove unused dependencies
- Use dynamic imports for large libraries
- Enable tree-shaking and minification

### Flaky Tests
Performance tests can be flaky due to:
- CPU load (close other applications)
- Network conditions (use fast, stable connection)
- Browser caching (tests run with fresh cache)

Rerun failed tests to confirm real issues vs. transient slowdowns.

## CI/CD Integration

These tests run in CI to ensure performance regressions are caught early:

```yaml
# .github/workflows/ui-tests.yml
- name: Run performance tests
  run: npm run test:performance
```

Performance budgets are enforced - failing tests block merges.

## Maintenance

### Updating Budgets
If requirements change, update budgets in test files:

```typescript
// tests/performance/core-web-vitals.spec.ts
const BUDGETS = {
  LCP: 2500,  // Update here
  FCP: 1500,
  TTI: 3000,
  CLS: 0.1,
  FID: 100,
};
```

### Adding New Routes
When adding new pages, add them to the `routes` array:

```typescript
const routes = [
  { path: '/', name: 'HomePage' },
  { path: '/devices', name: 'DevicesPage' },
  { path: '/your-new-page', name: 'YourNewPage' }, // Add here
];
```

## References

- [Web Vitals](https://web.dev/vitals/)
- [Lighthouse Performance Scoring](https://web.dev/performance-scoring/)
- [Playwright Testing](https://playwright.dev/)
- [Vite Performance](https://vitejs.dev/guide/performance.html)
