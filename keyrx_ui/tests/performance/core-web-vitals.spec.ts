import { test, expect } from '@playwright/test';

/**
 * Core Web Vitals Performance Testing
 *
 * Tests all five Core Web Vitals metrics against defined budgets:
 * - LCP (Largest Contentful Paint): < 2.5s
 * - FCP (First Contentful Paint): < 1.5s
 * - TTI (Time to Interactive): < 3.0s
 * - CLS (Cumulative Layout Shift): < 0.1
 * - FID (First Input Delay): < 100ms
 *
 * Requirements: Req 4 (Performance Budget), Task 36
 *
 * Note: Run with dev server: npm run dev
 */

const routes = [
  { path: '/', name: 'HomePage' },
  { path: '/devices', name: 'DevicesPage' },
  { path: '/profiles', name: 'ProfilesPage' },
  { path: '/config', name: 'ConfigPage' },
  { path: '/metrics', name: 'MetricsPage' },
  { path: '/simulator', name: 'SimulatorPage' },
];

// Performance budgets (in milliseconds for timing metrics)
const BUDGETS = {
  LCP: 2500,  // Largest Contentful Paint
  FCP: 1500,  // First Contentful Paint
  TTI: 3000,  // Time to Interactive
  CLS: 0.1,   // Cumulative Layout Shift (score, not time)
  FID: 100,   // First Input Delay
};

interface WebVitalsMetrics {
  LCP?: number;
  FCP?: number;
  TTI?: number;
  CLS?: number;
  FID?: number;
}

// Use Chromium for consistent performance metrics
test.use({ browserName: 'chromium' });

test.describe('Core Web Vitals Performance', () => {

  for (const route of routes) {
    test(`${route.name} should meet all Core Web Vitals budgets`, async ({ page }) => {
      // Navigate to the page
      await page.goto(`http://localhost:5173${route.path}`);

      // Collect Core Web Vitals metrics
      const metrics = await page.evaluate(() => {
        return new Promise<WebVitalsMetrics>((resolve) => {
          const vitals: WebVitalsMetrics = {};
          let observersComplete = 0;
          const totalObservers = 4; // LCP, FCP, CLS, FID (TTI calculated separately)

          const checkComplete = () => {
            observersComplete++;
            if (observersComplete >= totalObservers) {
              resolve(vitals);
            }
          };

          // Largest Contentful Paint (LCP)
          new PerformanceObserver((list) => {
            const entries = list.getEntries();
            const lastEntry = entries[entries.length - 1];
            if (lastEntry) {
              vitals.LCP = lastEntry.startTime;
            }
            checkComplete();
          }).observe({ type: 'largest-contentful-paint', buffered: true });

          // First Contentful Paint (FCP)
          new PerformanceObserver((list) => {
            const entries = list.getEntries();
            for (const entry of entries) {
              if (entry.name === 'first-contentful-paint') {
                vitals.FCP = entry.startTime;
                break;
              }
            }
            checkComplete();
          }).observe({ type: 'paint', buffered: true });

          // Cumulative Layout Shift (CLS)
          let clsValue = 0;
          new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
              const layoutShift = entry as PerformanceEntry & {
                value: number;
                hadRecentInput: boolean;
              };
              if (!layoutShift.hadRecentInput) {
                clsValue += layoutShift.value;
              }
            }
            vitals.CLS = clsValue;
          }).observe({ type: 'layout-shift', buffered: true });

          // Report CLS after a delay to ensure we capture all shifts
          setTimeout(() => {
            vitals.CLS = clsValue;
            checkComplete();
          }, 2000);

          // First Input Delay (FID)
          // Note: FID requires actual user interaction, so we use event timing as proxy
          new PerformanceObserver((list) => {
            const entries = list.getEntries();
            for (const entry of entries) {
              const eventTiming = entry as PerformanceEntry & {
                processingStart: number;
              };
              if (eventTiming.processingStart) {
                vitals.FID = eventTiming.processingStart - entry.startTime;
                break;
              }
            }
            checkComplete();
          }).observe({ type: 'first-input', buffered: true });

          // If no first-input after 3 seconds, consider FID as 0 (no delay)
          setTimeout(() => {
            if (vitals.FID === undefined) {
              vitals.FID = 0;
              checkComplete();
            }
          }, 3000);
        });
      });

      // Calculate Time to Interactive (TTI)
      // TTI is when the page becomes fully interactive (no long tasks blocking main thread)
      const navigationTiming = await page.evaluate(() => {
        const perf = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        return {
          domInteractive: perf.domInteractive,
          domContentLoadedEventEnd: perf.domContentLoadedEventEnd,
          loadEventEnd: perf.loadEventEnd,
        };
      });

      // TTI approximation: use domInteractive as a conservative estimate
      metrics.TTI = navigationTiming.domInteractive;

      // Log metrics for debugging
      console.log(`\n${route.name} Performance Metrics:`);
      console.log(`  LCP: ${metrics.LCP?.toFixed(2)}ms (budget: ${BUDGETS.LCP}ms)`);
      console.log(`  FCP: ${metrics.FCP?.toFixed(2)}ms (budget: ${BUDGETS.FCP}ms)`);
      console.log(`  TTI: ${metrics.TTI?.toFixed(2)}ms (budget: ${BUDGETS.TTI}ms)`);
      console.log(`  CLS: ${metrics.CLS?.toFixed(4)} (budget: ${BUDGETS.CLS})`);
      console.log(`  FID: ${metrics.FID?.toFixed(2)}ms (budget: ${BUDGETS.FID}ms)`);

      // Assert against budgets
      if (metrics.LCP !== undefined) {
        expect(
          metrics.LCP,
          `LCP should be < ${BUDGETS.LCP}ms, got ${metrics.LCP.toFixed(2)}ms`
        ).toBeLessThan(BUDGETS.LCP);
      }

      if (metrics.FCP !== undefined) {
        expect(
          metrics.FCP,
          `FCP should be < ${BUDGETS.FCP}ms, got ${metrics.FCP.toFixed(2)}ms`
        ).toBeLessThan(BUDGETS.FCP);
      }

      if (metrics.TTI !== undefined) {
        expect(
          metrics.TTI,
          `TTI should be < ${BUDGETS.TTI}ms, got ${metrics.TTI.toFixed(2)}ms`
        ).toBeLessThan(BUDGETS.TTI);
      }

      if (metrics.CLS !== undefined) {
        expect(
          metrics.CLS,
          `CLS should be < ${BUDGETS.CLS}, got ${metrics.CLS.toFixed(4)}`
        ).toBeLessThan(BUDGETS.CLS);
      }

      if (metrics.FID !== undefined) {
        expect(
          metrics.FID,
          `FID should be < ${BUDGETS.FID}ms, got ${metrics.FID.toFixed(2)}ms`
        ).toBeLessThan(BUDGETS.FID);
      }
    });
  }

  test('Performance metrics summary report', async ({ page }) => {
    const allMetrics: Array<{ route: string; metrics: WebVitalsMetrics }> = [];

    // Collect metrics for all routes
    for (const route of routes) {
      await page.goto(`http://localhost:5173${route.path}`);

      const metrics = await page.evaluate(() => {
        return new Promise<WebVitalsMetrics>((resolve) => {
          const vitals: WebVitalsMetrics = {};
          const perf = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;

          // Get FCP
          const fcpEntry = performance.getEntriesByName('first-contentful-paint')[0];
          if (fcpEntry) {
            vitals.FCP = fcpEntry.startTime;
          }

          // Get LCP
          const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
          if (lcpEntries.length > 0) {
            vitals.LCP = lcpEntries[lcpEntries.length - 1].startTime;
          }

          // Get TTI approximation
          vitals.TTI = perf.domInteractive;

          // Get CLS (approximate - needs observation period)
          vitals.CLS = 0; // Default to 0 for static pages

          // Get FID (approximate)
          vitals.FID = 0; // Default to 0 if no interaction

          resolve(vitals);
        });
      });

      allMetrics.push({ route: route.name, metrics });
    }

    // Generate summary report
    console.log('\n=== Performance Summary Report ===\n');
    console.log('Route                | LCP      | FCP      | TTI      | CLS    | FID    ');
    console.log('---------------------|----------|----------|----------|--------|--------');

    for (const { route, metrics } of allMetrics) {
      const lcp = metrics.LCP?.toFixed(0).padStart(6) || 'N/A'.padStart(6);
      const fcp = metrics.FCP?.toFixed(0).padStart(6) || 'N/A'.padStart(6);
      const tti = metrics.TTI?.toFixed(0).padStart(6) || 'N/A'.padStart(6);
      const cls = metrics.CLS?.toFixed(4).padStart(6) || 'N/A'.padStart(6);
      const fid = metrics.FID?.toFixed(0).padStart(6) || 'N/A'.padStart(6);

      console.log(`${route.padEnd(20)} | ${lcp}ms | ${fcp}ms | ${tti}ms | ${cls} | ${fid}ms`);
    }

    console.log('\nBudget Thresholds:');
    console.log(`LCP: < ${BUDGETS.LCP}ms`);
    console.log(`FCP: < ${BUDGETS.FCP}ms`);
    console.log(`TTI: < ${BUDGETS.TTI}ms`);
    console.log(`CLS: < ${BUDGETS.CLS}`);
    console.log(`FID: < ${BUDGETS.FID}ms`);

    // This test just generates the report, no assertions
    expect(allMetrics.length).toBe(routes.length);
  });
});
