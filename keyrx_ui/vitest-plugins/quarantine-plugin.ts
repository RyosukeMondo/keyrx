/**
 * Vitest Plugin: Quarantine Filter
 *
 * This plugin integrates with Vitest to exclude quarantined tests from main test runs.
 * Quarantined tests are tracked in tests/quarantine.json and can be run separately.
 */

import type { Plugin } from 'vite';
import { getQuarantinedTestPatterns, checkQuarantineHealth } from '../tests/quarantine-manager';

export interface QuarantinePluginOptions {
  /** Whether to run quarantined tests instead of excluding them (default: false) */
  runQuarantined?: boolean;
  /** Whether to print quarantine health status (default: true) */
  printStatus?: boolean;
}

/**
 * Vitest plugin to filter quarantined tests
 */
export function quarantinePlugin(options: QuarantinePluginOptions = {}): Plugin {
  const { runQuarantined = false, printStatus = true } = options;

  return {
    name: 'vitest-quarantine-plugin',
    config(config, { command }) {
      // Only apply during test runs
      if (command !== 'serve' || !config.test) {
        return;
      }

      // Check quarantine health and print status
      if (printStatus) {
        const health = checkQuarantineHealth();
        if (health.message) {
          console.log(`\n${health.message}\n`);
        }
      }

      const quarantinedTests = getQuarantinedTestPatterns();

      if (quarantinedTests.length === 0) {
        if (printStatus && !runQuarantined) {
          console.log('✓ No tests in quarantine\n');
        }
        return;
      }

      // Modify test config based on mode
      return {
        test: {
          ...config.test,
          // In quarantine mode, only run quarantined tests
          // In normal mode, exclude quarantined tests
          ...(runQuarantined
            ? {
                include: quarantinedTests.map(testPath => {
                  // Extract file path from full test path
                  // Format: "src/pages/ConfigPage.test.tsx > ConfigPage > test name"
                  const filePath = testPath.split(' > ')[0];
                  return filePath;
                }),
              }
            : {
                exclude: [
                  ...(config.test?.exclude || []),
                  // Vitest doesn't support excluding by test name directly,
                  // so we'll handle this via test name matching in setup
                ],
              }),
        },
      };
    },
  };
}

/**
 * Helper function to check if a test should be quarantined
 * Called from test setup to skip quarantined tests
 */
export function isTestQuarantined(testFullPath: string): boolean {
  const quarantinedTests = getQuarantinedTestPatterns();
  return quarantinedTests.includes(testFullPath);
}
