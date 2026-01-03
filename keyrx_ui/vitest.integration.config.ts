/// <reference types="vitest" />
import { defineConfig, mergeConfig } from 'vitest/config';
import { baseConfig } from './vitest.config.base';

// Integration test configuration - slower tests for component interactions and full page testing.
//
// Includes:
//   - Integration tests (src/**/*.integration.test.{ts,tsx}, tests/integration/**)
//   - Accessibility tests (src/**/*.a11y.test.{ts,tsx}, tests/a11y/**)
//
// Excludes:
//   - Unit tests (regular *.test.{ts,tsx} files without integration/a11y suffix)
//   - E2E tests (e2e/**, tests/e2e/**)
//   - Performance tests (tests/performance/**)
//
// Timeouts:
//   - Test timeout: 30000ms (adequate for async operations)
//   - Hook timeout: 10000ms
export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      name: 'integration',
      include: [
        // Integration tests
        'src/**/*.integration.test.{ts,tsx}',
        'tests/integration/**/*.test.{ts,tsx}',
        // Accessibility tests
        'src/**/*.a11y.test.{ts,tsx}',
        'tests/a11y/**/*.test.{ts,tsx}',
      ],
      exclude: [
        'node_modules/**',
        'dist/**',
        // E2E tests
        'e2e/**',
        'tests/e2e/**',
        // Performance tests
        'tests/performance/**',
      ],
      testTimeout: 30000,
      hookTimeout: 10000,
    },
  })
);
