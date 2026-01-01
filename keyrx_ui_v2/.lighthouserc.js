/**
 * Lighthouse CI Configuration
 *
 * Defines performance budgets and quality thresholds for automated testing.
 * Lighthouse scores must meet minimum thresholds for CI to pass.
 */

module.exports = {
  ci: {
    collect: {
      // Start a local server to test the production build
      startServerCommand: 'npm run preview',
      startServerReadyPattern: 'Local:',
      startServerReadyTimeout: 30000,
      url: [
        'http://localhost:4173/',
        'http://localhost:4173/profiles',
        'http://localhost:4173/config/Default',
        'http://localhost:4173/dashboard',
        'http://localhost:4173/devices',
      ],
      numberOfRuns: 3, // Run 3 times and take median
      settings: {
        // Disable throttling for faster tests (we're testing bundle size, not network)
        throttlingMethod: 'provided',
        // Use desktop preset
        preset: 'desktop',
      },
    },
    assert: {
      // Performance thresholds - all must score >= 90
      assertions: {
        // Core Web Vitals
        'categories:performance': ['error', { minScore: 0.9 }],
        'categories:accessibility': ['error', { minScore: 0.9 }],
        'categories:best-practices': ['error', { minScore: 0.9 }],
        'categories:seo': ['warn', { minScore: 0.8 }], // SEO is less critical for a local app

        // Specific metrics
        'first-contentful-paint': ['warn', { maxNumericValue: 2000 }], // 2s
        'largest-contentful-paint': ['warn', { maxNumericValue: 3000 }], // 3s
        'cumulative-layout-shift': ['error', { maxNumericValue: 0.1 }],
        'total-blocking-time': ['warn', { maxNumericValue: 300 }], // 300ms

        // Resource budgets
        'resource-summary:script:size': ['error', { maxNumericValue: 1024000 }], // 1MB JS
        'resource-summary:stylesheet:size': ['error', { maxNumericValue: 102400 }], // 100KB CSS
        'resource-summary:document:size': ['warn', { maxNumericValue: 51200 }], // 50KB HTML
        'resource-summary:total:size': ['warn', { maxNumericValue: 3145728 }], // 3MB total

        // Bundle optimization checks
        'uses-long-cache-ttl': 'off', // Not applicable for local development
        'unused-javascript': ['warn', { maxLength: 1 }], // Warn about unused JS
        'modern-image-formats': 'off', // We don't have many images
        'offscreen-images': 'off',
      },
    },
    upload: {
      // Store results in .lighthouseci directory for later analysis
      target: 'filesystem',
      outputDir: '.lighthouseci',
    },
  },
};
