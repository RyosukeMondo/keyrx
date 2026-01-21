/**
 * Auto-Fix Strategies
 *
 * Implementation of fix strategies for different types of test failures.
 * Each strategy is idempotent and safe to apply multiple times.
 */

import * as fs from 'fs';
import * as path from 'path';
import { Issue } from './issue-classifier';
import { DaemonFixture } from '../fixtures/daemon-fixture';
import { ApiClient } from '../api-client/client';
import { TestCase } from '../test-cases/test-utils';

export interface FixResult {
  success: boolean;
  message: string;
  retry: boolean;
}

export interface FixContext {
  daemon: DaemonFixture;
  apiClient: ApiClient;
  testCase: TestCase;
  expectedResultsPath?: string;
}

/**
 * Base interface for fix strategies
 */
export interface FixStrategy {
  canFix(issue: Issue): boolean;
  apply(issue: Issue, context: FixContext): Promise<FixResult>;
}

/**
 * Strategy: Restart daemon to fix network errors
 */
export class RestartDaemonStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'network' && issue.fixable;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    try {
      console.log('  → Restarting daemon...');

      // Stop existing daemon
      await context.daemon.stop();
      await this.sleep(1000); // Wait for cleanup

      // Start daemon again
      await context.daemon.start();
      await context.daemon.waitUntilReady(30000);

      return {
        success: true,
        message: 'Daemon restarted successfully',
        retry: true,
      };
    } catch (error) {
      return {
        success: false,
        message: `Failed to restart daemon: ${error}`,
        retry: false,
      };
    }
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Strategy: Retry test with longer timeout
 */
export class RetryWithTimeoutStrategy implements FixStrategy {
  private attemptCounts = new Map<string, number>();

  canFix(issue: Issue): boolean {
    return (
      issue.type === 'network' &&
      issue.errorMessage?.toLowerCase().includes('timeout') === true
    );
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const attempts = this.attemptCounts.get(issue.testId) || 0;

    // Don't retry more than twice
    if (attempts >= 2) {
      return {
        success: false,
        message: 'Max retry attempts reached',
        retry: false,
      };
    }

    this.attemptCounts.set(issue.testId, attempts + 1);

    // Just wait a bit and retry
    await this.sleep(2000);

    return {
      success: true,
      message: `Retrying test (attempt ${attempts + 1})`,
      retry: true,
    };
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Strategy: Update expected results for schema changes
 */
export class UpdateExpectedResultsStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return (
      issue.type === 'validation' &&
      issue.fixable &&
      issue.diffs !== undefined &&
      issue.diffs.length > 0
    );
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    // This is conservative - we only update for specific patterns
    const diffs = issue.diffs || [];

    // Check if all diffs are "extra" fields (new fields added to API)
    const allExtraFields = diffs.every(d => d.type === 'extra');
    const allTypeMismatches = diffs.every(d => d.type === 'type-mismatch');

    if (!allExtraFields && !allTypeMismatches) {
      return {
        success: false,
        message: 'Cannot auto-update: diffs are not all extra fields or type mismatches',
        retry: false,
      };
    }

    console.log('  → Would update expected results (not implemented for safety)');
    console.log(`    Test: ${issue.testName}`);
    console.log(`    Diffs: ${diffs.length} field(s)`);

    // Note: For safety, we don't actually modify expected-results.json
    // In a real implementation, you would:
    // 1. Load expected-results.json
    // 2. Find the test case's expected result
    // 3. Apply the diff to update it
    // 4. Save back to file
    // 5. Return { success: true, retry: true }

    return {
      success: false,
      message: 'Expected results update not implemented (requires manual review)',
      retry: false,
    };
  }
}

/**
 * Strategy: Reseed test data fixtures
 */
export class ReseedFixtureStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'data' && issue.fixable;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    try {
      console.log('  → Attempting to reseed fixtures...');

      // Check if error is about missing resource (404)
      if (issue.errorMessage?.includes('404') || issue.errorMessage?.includes('not found')) {
        // Try to re-run the test setup
        console.log('  → Re-running test setup...');

        // This assumes the test's setup() creates necessary fixtures
        // We don't actually re-run it here to avoid side effects
        // but the retry will trigger setup again

        return {
          success: true,
          message: 'Fixture issue detected, will retry with fresh setup',
          retry: true,
        };
      }

      return {
        success: false,
        message: 'Cannot automatically reseed - unknown data issue',
        retry: false,
      };
    } catch (error) {
      return {
        success: false,
        message: `Failed to reseed fixtures: ${error}`,
        retry: false,
      };
    }
  }
}

/**
 * Strategy: Wait and retry for transient failures
 */
export class RetryTransientFailureStrategy implements FixStrategy {
  private attemptCounts = new Map<string, number>();

  canFix(issue: Issue): boolean {
    // Only retry fixable issues that aren't logic bugs
    return issue.fixable && issue.type !== 'logic' && issue.priority <= 2;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const attempts = this.attemptCounts.get(issue.testId) || 0;

    if (attempts >= 1) {
      return {
        success: false,
        message: 'Already retried once',
        retry: false,
      };
    }

    this.attemptCounts.set(issue.testId, attempts + 1);

    // Wait briefly for transient issues to resolve
    await this.sleep(1000);

    return {
      success: true,
      message: 'Retrying after brief wait',
      retry: true,
    };
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Strategy registry with priority ordering
 */
export class FixStrategyRegistry {
  private strategies: FixStrategy[] = [];

  constructor() {
    // Register strategies in priority order (most specific first)
    this.register(new RestartDaemonStrategy());
    this.register(new RetryWithTimeoutStrategy());
    this.register(new UpdateExpectedResultsStrategy());
    this.register(new ReseedFixtureStrategy());
    this.register(new RetryTransientFailureStrategy());
  }

  /**
   * Register a fix strategy
   */
  register(strategy: FixStrategy): void {
    this.strategies.push(strategy);
  }

  /**
   * Find appropriate strategy for an issue
   */
  findStrategy(issue: Issue): FixStrategy | null {
    return this.strategies.find(s => s.canFix(issue)) || null;
  }

  /**
   * Apply the best strategy for an issue
   */
  async applyFix(issue: Issue, context: FixContext): Promise<FixResult> {
    const strategy = this.findStrategy(issue);

    if (!strategy) {
      return {
        success: false,
        message: 'No applicable fix strategy found',
        retry: false,
      };
    }

    console.log(`  Applying fix: ${strategy.constructor.name}`);
    return strategy.apply(issue, context);
  }

  /**
   * Get all registered strategies
   */
  getStrategies(): FixStrategy[] {
    return [...this.strategies];
  }
}

/**
 * Create a fix strategy registry with default strategies
 */
export function createFixRegistry(): FixStrategyRegistry {
  return new FixStrategyRegistry();
}
