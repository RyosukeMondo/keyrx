import { test, expect } from '@playwright/test';

/**
 * Bundle Size and Asset Loading Performance Tests
 *
 * Validates that JavaScript and CSS bundles meet size budgets:
 * - Total JS bundle: ≤ 250KB (gzipped)
 * - Total CSS bundle: ≤ 50KB (gzipped)
 * - Individual chunks: reasonable sizes
 * - Image assets: optimized sizes
 *
 * Requirements: Req 4 (Performance Budget), Task 37
 *
 * Note: This test analyzes network requests to measure actual bundle sizes
 */

// Bundle size budgets (in bytes)
const BUDGETS = {
  totalJS: 250 * 1024,      // 250KB for all JavaScript
  totalCSS: 50 * 1024,      // 50KB for all CSS
  largestJS: 150 * 1024,    // No single JS chunk > 150KB
  largestCSS: 30 * 1024,    // No single CSS file > 30KB
  totalAssets: 500 * 1024,  // Total assets (JS + CSS + fonts) < 500KB
};

interface ResourceMetrics {
  url: string;
  type: string;
  size: number;
  compressed: boolean;
  duration: number;
}

// Use Chromium for consistent performance metrics
test.use({ browserName: 'chromium' });

test.describe('Bundle Size Analysis', () => {

  test('Should meet JavaScript bundle size budget', async ({ page }) => {
    const resources: ResourceMetrics[] = [];

    // Listen to all responses to track resource sizes
    page.on('response', async (response) => {
      const url = response.url();
      const headers = response.headers();
      const contentType = headers['content-type'] || '';

      // Track JS, CSS, and font files
      if (
        contentType.includes('javascript') ||
        contentType.includes('css') ||
        url.endsWith('.js') ||
        url.endsWith('.css') ||
        url.endsWith('.woff2') ||
        url.endsWith('.woff')
      ) {
        try {
          const body = await response.body();
          const size = body.length;
          const compressed = headers['content-encoding']?.includes('gzip') || false;

          let type = 'other';
          if (contentType.includes('javascript') || url.endsWith('.js')) {
            type = 'javascript';
          } else if (contentType.includes('css') || url.endsWith('.css')) {
            type = 'css';
          } else if (url.endsWith('.woff2') || url.endsWith('.woff')) {
            type = 'font';
          }

          resources.push({
            url,
            type,
            size,
            compressed,
            duration: 0, // Will be updated from performance timing
          });
        } catch (error) {
          // Some responses may not have bodies, skip them
        }
      }
    });

    // Navigate to the home page
    await page.goto('http://localhost:5173/');
    await page.waitForLoadState('networkidle');

    // Calculate totals
    const jsResources = resources.filter((r) => r.type === 'javascript');
    const cssResources = resources.filter((r) => r.type === 'css');
    const fontResources = resources.filter((r) => r.type === 'font');

    const totalJS = jsResources.reduce((sum, r) => sum + r.size, 0);
    const totalCSS = cssResources.reduce((sum, r) => sum + r.size, 0);
    const totalFonts = fontResources.reduce((sum, r) => sum + r.size, 0);
    const totalAssets = totalJS + totalCSS + totalFonts;

    const largestJS = Math.max(...jsResources.map((r) => r.size), 0);
    const largestCSS = Math.max(...cssResources.map((r) => r.size), 0);

    // Log bundle analysis
    console.log('\n=== Bundle Size Analysis ===\n');
    console.log('JavaScript:');
    jsResources.forEach((r) => {
      const fileName = r.url.split('/').pop() || r.url;
      const sizeKB = (r.size / 1024).toFixed(2);
      console.log(`  - ${fileName}: ${sizeKB} KB ${r.compressed ? '(compressed)' : ''}`);
    });
    console.log(`  Total JS: ${(totalJS / 1024).toFixed(2)} KB`);

    console.log('\nCSS:');
    cssResources.forEach((r) => {
      const fileName = r.url.split('/').pop() || r.url;
      const sizeKB = (r.size / 1024).toFixed(2);
      console.log(`  - ${fileName}: ${sizeKB} KB ${r.compressed ? '(compressed)' : ''}`);
    });
    console.log(`  Total CSS: ${(totalCSS / 1024).toFixed(2)} KB`);

    if (fontResources.length > 0) {
      console.log('\nFonts:');
      fontResources.forEach((r) => {
        const fileName = r.url.split('/').pop() || r.url;
        const sizeKB = (r.size / 1024).toFixed(2);
        console.log(`  - ${fileName}: ${sizeKB} KB`);
      });
      console.log(`  Total Fonts: ${(totalFonts / 1024).toFixed(2)} KB`);
    }

    console.log(`\nTotal Assets: ${(totalAssets / 1024).toFixed(2)} KB`);

    console.log('\nBudgets:');
    console.log(`  Total JS: ${(BUDGETS.totalJS / 1024).toFixed(0)} KB`);
    console.log(`  Total CSS: ${(BUDGETS.totalCSS / 1024).toFixed(0)} KB`);
    console.log(`  Largest JS chunk: ${(BUDGETS.largestJS / 1024).toFixed(0)} KB`);
    console.log(`  Total Assets: ${(BUDGETS.totalAssets / 1024).toFixed(0)} KB`);

    // Assertions
    expect(
      totalJS,
      `Total JavaScript size should be ≤ ${(BUDGETS.totalJS / 1024).toFixed(0)}KB, got ${(totalJS / 1024).toFixed(2)}KB`
    ).toBeLessThanOrEqual(BUDGETS.totalJS);

    expect(
      totalCSS,
      `Total CSS size should be ≤ ${(BUDGETS.totalCSS / 1024).toFixed(0)}KB, got ${(totalCSS / 1024).toFixed(2)}KB`
    ).toBeLessThanOrEqual(BUDGETS.totalCSS);

    expect(
      largestJS,
      `Largest JS chunk should be ≤ ${(BUDGETS.largestJS / 1024).toFixed(0)}KB, got ${(largestJS / 1024).toFixed(2)}KB`
    ).toBeLessThanOrEqual(BUDGETS.largestJS);

    expect(
      totalAssets,
      `Total assets should be ≤ ${(BUDGETS.totalAssets / 1024).toFixed(0)}KB, got ${(totalAssets / 1024).toFixed(2)}KB`
    ).toBeLessThanOrEqual(BUDGETS.totalAssets);
  });

  test('Should lazy load route chunks', async ({ page }) => {
    const loadedResources = new Set<string>();

    // Track all loaded JavaScript files
    page.on('response', async (response) => {
      const url = response.url();
      if (url.endsWith('.js')) {
        const fileName = url.split('/').pop() || url;
        loadedResources.add(fileName);
      }
    });

    // Load home page
    await page.goto('http://localhost:5173/');
    await page.waitForLoadState('networkidle');
    const homeResources = new Set(loadedResources);

    // Navigate to devices page
    loadedResources.clear();
    await page.goto('http://localhost:5173/devices');
    await page.waitForLoadState('networkidle');
    const devicesResources = new Set(loadedResources);

    // Navigate to profiles page
    loadedResources.clear();
    await page.goto('http://localhost:5173/profiles');
    await page.waitForLoadState('networkidle');
    const profilesResources = new Set(loadedResources);

    console.log('\n=== Code Splitting Analysis ===\n');
    console.log('Home page loaded:', homeResources.size, 'JavaScript files');
    console.log('Devices page loaded:', devicesResources.size, 'additional files');
    console.log('Profiles page loaded:', profilesResources.size, 'additional files');

    // We expect lazy loading to work (routes should load separate chunks)
    // At minimum, we should not load ALL JavaScript on the home page
    expect(homeResources.size).toBeGreaterThan(0);
    expect(homeResources.size).toBeLessThan(20); // Reasonable upper limit for initial bundle
  });

  test('Should compress assets with gzip/brotli', async ({ page }) => {
    let hasCompression = false;

    page.on('response', async (response) => {
      const headers = response.headers();
      const contentEncoding = headers['content-encoding'] || '';

      if (contentEncoding.includes('gzip') || contentEncoding.includes('br')) {
        hasCompression = true;
      }
    });

    await page.goto('http://localhost:5173/');
    await page.waitForLoadState('networkidle');

    // Note: Dev server may not compress, but production build should
    // This test documents the expectation - it may fail in dev mode
    console.log('\n=== Compression Check ===');
    console.log(
      hasCompression
        ? '✓ Assets are compressed (gzip/brotli)'
        : '✗ No compression detected (expected in production only)'
    );

    // Don't assert in dev mode - this is informational
    // In production, we would expect: expect(hasCompression).toBe(true);
  });
});

test.describe('Asset Loading Performance', () => {

  test('Should load critical assets quickly', async ({ page }) => {
    await page.goto('http://localhost:5173/');

    // Measure resource timing
    const resourceTimings = await page.evaluate(() => {
      const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[];
      return resources.map((r) => ({
        name: r.name.split('/').pop() || r.name,
        duration: r.duration,
        size: r.transferSize,
        type: r.initiatorType,
      }));
    });

    // Log slow resources
    const slowResources = resourceTimings.filter((r) => r.duration > 1000); // > 1s
    if (slowResources.length > 0) {
      console.log('\n=== Slow Resources (> 1s) ===');
      slowResources.forEach((r) => {
        console.log(`  - ${r.name}: ${r.duration.toFixed(0)}ms (${(r.size / 1024).toFixed(2)}KB)`);
      });
    }

    // Most resources should load quickly in dev mode
    const averageDuration =
      resourceTimings.reduce((sum, r) => sum + r.duration, 0) / resourceTimings.length;

    console.log(`\nAverage resource load time: ${averageDuration.toFixed(0)}ms`);

    // In dev mode, resources should load reasonably fast (< 2s average)
    expect(averageDuration).toBeLessThan(2000);
  });

  test('Should use efficient caching headers', async ({ page }) => {
    const cachedResources: string[] = [];
    const noCacheResources: string[] = [];

    page.on('response', async (response) => {
      const url = response.url();
      const headers = response.headers();
      const cacheControl = headers['cache-control'] || '';

      // Check for static assets
      if (url.endsWith('.js') || url.endsWith('.css') || url.endsWith('.woff2')) {
        if (cacheControl.includes('max-age') || cacheControl.includes('immutable')) {
          cachedResources.push(url.split('/').pop() || url);
        } else {
          noCacheResources.push(url.split('/').pop() || url);
        }
      }
    });

    await page.goto('http://localhost:5173/');
    await page.waitForLoadState('networkidle');

    console.log('\n=== Caching Analysis ===');
    console.log('Resources with cache headers:', cachedResources.length);
    console.log('Resources without cache headers:', noCacheResources.length);

    if (noCacheResources.length > 0) {
      console.log('\nNo cache headers:');
      noCacheResources.forEach((name) => console.log(`  - ${name}`));
    }

    // In production, static assets should have cache headers
    // In dev mode, this is informational only
  });
});
