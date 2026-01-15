/**
 * Quarantine Manager - Manages flaky test quarantine list
 *
 * This utility provides functions to load, validate, and query the quarantine list.
 * Used by Vitest config to exclude quarantined tests from main runs.
 */

import fs from 'fs';
import path from 'path';

export interface QuarantineEntry {
  testPath: string;
  reason: string;
  quarantinedAt: string;
  issueUrl: string;
  failureRate?: number;
  lastFailure?: string;
  assignee?: string;
}

export interface ResolvedEntry {
  testPath: string;
  resolvedAt: string;
  resolution?: string;
}

export interface QuarantineMetadata {
  maxQuarantineSize: number;
  alertThreshold: number;
  autoRemoveAfterDays: number;
}

export interface QuarantineConfig {
  version: string;
  lastUpdated: string;
  description?: string;
  quarantine: QuarantineEntry[];
  resolved: ResolvedEntry[];
  metadata: QuarantineMetadata;
}

/**
 * Load quarantine configuration from JSON file
 */
export function loadQuarantineConfig(): QuarantineConfig | null {
  const configPath = path.resolve(__dirname, 'quarantine.json');

  try {
    if (!fs.existsSync(configPath)) {
      console.warn(`Quarantine config not found at: ${configPath}`);
      return null;
    }

    const content = fs.readFileSync(configPath, 'utf-8');
    const config: QuarantineConfig = JSON.parse(content);

    // Validate basic structure
    if (!config.quarantine || !Array.isArray(config.quarantine)) {
      console.error('Invalid quarantine config: missing quarantine array');
      return null;
    }

    return config;
  } catch (error) {
    console.error(`Failed to load quarantine config: ${error}`);
    return null;
  }
}

/**
 * Get list of quarantined test paths for Vitest exclusion
 */
export function getQuarantinedTestPatterns(): string[] {
  const config = loadQuarantineConfig();
  if (!config || config.quarantine.length === 0) {
    return [];
  }

  // Convert test paths to Vitest exclude patterns
  // Format: "src/pages/ConfigPage.test.tsx > ConfigPage > should handle WebSocket reconnection"
  return config.quarantine.map(entry => entry.testPath);
}

/**
 * Check if quarantine size exceeds alert threshold
 */
export function checkQuarantineHealth(): {
  healthy: boolean;
  size: number;
  threshold: number;
  maxSize: number;
  message?: string;
} {
  const config = loadQuarantineConfig();
  if (!config) {
    return {
      healthy: true,
      size: 0,
      threshold: 0,
      maxSize: 0,
    };
  }

  const size = config.quarantine.length;
  const { alertThreshold, maxQuarantineSize } = config.metadata;

  if (size >= maxQuarantineSize) {
    return {
      healthy: false,
      size,
      threshold: alertThreshold,
      maxSize: maxQuarantineSize,
      message: `⚠️  CRITICAL: Quarantine size (${size}) has reached maximum (${maxQuarantineSize})!`,
    };
  }

  if (size >= alertThreshold) {
    return {
      healthy: false,
      size,
      threshold: alertThreshold,
      maxSize: maxQuarantineSize,
      message: `⚠️  WARNING: Quarantine size (${size}) exceeds alert threshold (${alertThreshold})`,
    };
  }

  return {
    healthy: true,
    size,
    threshold: alertThreshold,
    maxSize: maxQuarantineSize,
    message: size > 0 ? `ℹ️  ${size} test(s) currently in quarantine` : undefined,
  };
}

/**
 * Find tests that should be auto-removed from quarantine (older than autoRemoveAfterDays)
 */
export function findStaleQuarantineEntries(): QuarantineEntry[] {
  const config = loadQuarantineConfig();
  if (!config) {
    return [];
  }

  const { autoRemoveAfterDays } = config.metadata;
  const now = new Date();
  const staleThreshold = new Date(now.getTime() - autoRemoveAfterDays * 24 * 60 * 60 * 1000);

  return config.quarantine.filter(entry => {
    const quarantinedDate = new Date(entry.quarantinedAt);
    return quarantinedDate < staleThreshold;
  });
}

/**
 * Print quarantine status report (for CLI usage)
 */
export function printQuarantineStatus(): void {
  const config = loadQuarantineConfig();
  if (!config) {
    console.log('No quarantine configuration found.');
    return;
  }

  const health = checkQuarantineHealth();
  const stale = findStaleQuarantineEntries();

  console.log('\n=== Test Quarantine Status ===\n');
  console.log(`Total in quarantine: ${health.size}`);
  console.log(`Alert threshold: ${health.threshold}`);
  console.log(`Maximum allowed: ${health.maxSize}`);

  if (health.message) {
    console.log(`\n${health.message}`);
  }

  if (config.quarantine.length > 0) {
    console.log('\nQuarantined tests:');
    config.quarantine.forEach((entry, idx) => {
      console.log(`\n${idx + 1}. ${entry.testPath}`);
      console.log(`   Reason: ${entry.reason}`);
      console.log(`   Quarantined: ${new Date(entry.quarantinedAt).toLocaleDateString()}`);
      console.log(`   Issue: ${entry.issueUrl}`);
      if (entry.failureRate !== undefined) {
        console.log(`   Failure rate: ${(entry.failureRate * 100).toFixed(1)}%`);
      }
      if (entry.assignee) {
        console.log(`   Assignee: ${entry.assignee}`);
      }
    });
  }

  if (stale.length > 0) {
    console.log(`\n⚠️  ${stale.length} stale test(s) should be removed or fixed:`);
    stale.forEach(entry => {
      console.log(`   - ${entry.testPath}`);
    });
  }

  if (config.resolved.length > 0) {
    console.log(`\n✓ ${config.resolved.length} test(s) previously resolved`);
  }

  console.log('\n==============================\n');
}

// CLI usage - this file can be run directly with: npx tsx tests/quarantine-manager.ts
// Note: ESM modules can't use require.main === module, so we check process.argv
const isDirectRun = import.meta.url === `file://${process.argv[1]}`;
if (isDirectRun) {
  printQuarantineStatus();
}
