/// <reference types="vitest" />
import { defineConfig, mergeConfig } from 'vitest/config';
import { baseConfig } from './vitest.config.base';
import { SlowTestReporter } from './vitest-slow-test-reporter';

// Unit test configuration - fast, focused tests for individual components and functions.
//
// Includes:
//   - src/**/*.test.{ts,tsx} (all unit tests)
//
// Excludes:
//   - Integration tests (__integration__/**, tests/integration/**)
//   - Accessibility tests (tests/a11y/**)
//   - E2E tests (e2e/**, tests/e2e/**)
//   - Performance tests (tests/performance/**)
//
// Timeouts:
//   - Test timeout: 3000ms (fast feedback, reduced from 5000ms)
//   - Hook timeout: 2000ms (reduced from 3000ms)
//   - Slow test threshold: 1000ms (warns if test exceeds this)
export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      name: 'unit',
      include: ['src/**/*.test.{ts,tsx}'],
      exclude: [
        'node_modules/**',
        'dist/**',
        // Integration tests
        'src/**/__integration__/**',
        'tests/integration/**',
        '**/*.integration.test.{ts,tsx}',
        // Accessibility tests
        'tests/a11y/**',
        '**/*.a11y.test.{ts,tsx}',
        // E2E tests
        'e2e/**',
        'tests/e2e/**',
        // Performance tests
        'tests/performance/**',
      ],
      testTimeout: 3000,
      hookTimeout: 2000,
      slowTestThreshold: 1000,
      reporters: [
        'default',
        new SlowTestReporter(1000),
      ],
    },
  })
);
