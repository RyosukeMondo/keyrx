/// <reference types="vitest" />
import { defineConfig, mergeConfig } from 'vitest/config';
import { baseConfig } from './vitest.config.base';
import { SlowTestReporter } from './vitest-slow-test-reporter';

// Unit test configuration - fast, focused tests for individual components and functions.
//
// PERFORMANCE OPTIMIZATIONS:
//   - threads: true - Parallel test execution using worker threads
//   - isolate: true - Each test in isolated worker (prevents state leakage)
//   - maxThreads: 4 - Use up to 4 CPU cores
//   - eager WASM mocking - Mock loaded once, not per test
//
// Expected improvement: 36.57s → 28-30s (20% faster)
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

      // OPTIMIZATION: Parallel execution
      threads: true,                          // Enable multi-threading
      isolate: true,                          // Isolate test environment
      maxThreads: 4,                          // Max 4 parallel threads
      minThreads: 1,                          // Min 1 thread

      // OPTIMIZATION: Eager WASM mocking (see setupMocks.ts)
      // setupMocks.ts must come first to mock WASM before any component imports
      setupFiles: ['./src/test/setupMocks.ts', './src/test/setup.ts'],
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
