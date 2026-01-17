/* eslint-disable local-rules/test-naming-convention */
/// <reference types="vitest" />
import type { UserConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';
import os from 'os';
import { quarantinePlugin } from './vitest-plugins/quarantine-plugin';

/**
 * Base Vitest configuration for shared test settings.
 * This config is imported by vitest.unit.config.ts and vitest.integration.config.ts
 * to avoid duplication of test settings.
 *
 * Note: Exported as plain object (not defineConfig) to avoid type conflicts
 * between vite and vitest's bundled vite version.
 */
export const baseConfig: UserConfig = {
  plugins: [
    wasm(),
    topLevelAwait(),
    react(),
    quarantinePlugin({
      runQuarantined: process.env.RUN_QUARANTINE === 'true',
      printStatus: true,
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // Fix wasm-pack 'env' import issue
      'env': path.resolve(__dirname, './src/wasm/env-shim.js'),
    },
  },
  optimizeDeps: {
    exclude: ['@/wasm/pkg/keyrx_core'],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
    // Custom reporters for test quality monitoring
    reporters: [
      'default',
      './vitest-reporters/test-balance-reporter.ts',
    ],
    // Parallel execution configuration
    pool: 'threads',
    poolOptions: {
      threads: {
        // Optimize thread count based on CPU cores
        // Use 75% of available cores to leave headroom for system
        maxThreads: Math.max(1, Math.floor((os.cpus().length || 4) * 0.75)),
        minThreads: 1,
      },
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'json-summary', 'html', 'lcov'],
      reportsDirectory: './coverage',
      // Include all source files in coverage, even if not imported by tests
      all: true,
      exclude: [
        'node_modules/**',
        'dist/**',
        'src/test/**',
        'tests/**',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/*.integration.test.{ts,tsx}',
        '**/*.e2e.{ts,tsx}',
        'src/wasm/pkg/**',
        'src/test/mocks/**',
        '**/*.d.ts',
        '**/types/**',
        'vitest-reporters/**',
        'vitest*.config.ts',
        'vite.config.ts',
        'playwright*.config.ts',
        // Setup and test utility files
        'src/test/setup.ts',
        'tests/testUtils.tsx',
        'tests/helpers/**',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
        // Per-file thresholds for critical paths
        perFile: true,
      },
      // Note: Per-directory thresholds for critical paths (hooks/, api/) are enforced at 90%
      // This is validated in CI via the coverage report. See the test:coverage script.
      // Critical paths: src/hooks/*, src/api/*, src/services/*
    },
  },
};
