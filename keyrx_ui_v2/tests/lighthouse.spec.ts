import { test, expect } from '@playwright/test';
import { playAudit } from 'playwright-lighthouse';
import type { Page } from '@playwright/test';

/**
 * Lighthouse Performance and Accessibility Audit
 *
 * Tests Core Web Vitals and accessibility metrics.
 * Requirements: Req 4 (Performance Budget), Task 31 (Lighthouse accessibility ≥95)
 *
 * Note: This test requires Chromium browser and a running dev server.
 */

const routes = [
  { path: '/', name: 'HomePage' },
  { path: '/devices', name: 'DevicesPage' },
  { path: '/profiles', name: 'ProfilesPage' },
  { path: '/config', name: 'ConfigPage' },
  { path: '/metrics', name: 'MetricsPage' },
  { path: '/simulator', name: 'SimulatorPage' },
];

// Force Chromium browser for Lighthouse tests
test.use({ browserName: 'chromium' });

test.describe('Lighthouse Audits', () => {

  for (const route of routes) {
    test(`${route.name} should meet accessibility score threshold (≥95)`, async ({
      page,
      context,
    }) => {
      await page.goto(`http://localhost:5173${route.path}`);
      await page.waitForLoadState('networkidle');

      // Run Lighthouse audit
      await playAudit({
        page,
        port: 9222, // Chrome debugging port
        thresholds: {
          accessibility: 95,
        },
        reports: {
          formats: {
            html: true,
            json: true,
          },
          directory: './lighthouse-reports',
          name: `${route.name}-${Date.now()}`,
        },
      });
    });

    test(`${route.name} should meet performance budgets`, async ({ page, context }) => {
      await page.goto(`http://localhost:5173${route.path}`);
      await page.waitForLoadState('networkidle');

      // Run Lighthouse audit with performance thresholds
      await playAudit({
        page,
        port: 9222,
        thresholds: {
          performance: 90,
          accessibility: 95,
          'best-practices': 90,
          seo: 80,
        },
        reports: {
          formats: {
            json: true,
          },
          directory: './lighthouse-reports',
          name: `${route.name}-performance-${Date.now()}`,
        },
      });
    });
  }

  test('All pages should meet Core Web Vitals', async ({ page }) => {
    const metrics = {
      LCP: 2500, // Largest Contentful Paint < 2.5s
      FID: 100, // First Input Delay < 100ms
      CLS: 0.1, // Cumulative Layout Shift < 0.1
    };

    for (const route of routes) {
      await page.goto(`http://localhost:5173${route.path}`);
      await page.waitForLoadState('networkidle');

      // Measure Core Web Vitals
      const webVitals = await page.evaluate(() => {
        return new Promise((resolve) => {
          const vitals: Record<string, number> = {};

          // Largest Contentful Paint
          new PerformanceObserver((list) => {
            const entries = list.getEntries();
            const lastEntry = entries[entries.length - 1] as PerformanceEntry;
            vitals.LCP = lastEntry.startTime;
          }).observe({ type: 'largest-contentful-paint', buffered: true });

          // First Input Delay (simulated with interaction observer)
          new PerformanceObserver((list) => {
            const entries = list.getEntries();
            entries.forEach((entry) => {
              if ('processingStart' in entry && 'startTime' in entry) {
                vitals.FID =
                  (entry as PerformanceEventTiming).processingStart - entry.startTime;
              }
            });
          }).observe({ type: 'first-input', buffered: true });

          // Cumulative Layout Shift
          let clsValue = 0;
          new PerformanceObserver((list) => {
            list.getEntries().forEach((entry) => {
              if ('value' in entry && !entry.hadRecentInput) {
                clsValue += (entry as LayoutShift).value;
              }
            });
            vitals.CLS = clsValue;
          }).observe({ type: 'layout-shift', buffered: true });

          // Wait a bit for metrics to be collected
          setTimeout(() => resolve(vitals), 2000);
        });
      });

      // Check LCP
      if ('LCP' in webVitals) {
        expect(webVitals.LCP).toBeLessThan(metrics.LCP);
      }

      // Check CLS
      if ('CLS' in webVitals) {
        expect(webVitals.CLS).toBeLessThan(metrics.CLS);
      }

      console.log(`${route.name} Web Vitals:`, webVitals);
    }
  });
});

// Type definition for Layout Shift entry
interface LayoutShift extends PerformanceEntry {
  value: number;
  hadRecentInput: boolean;
}
