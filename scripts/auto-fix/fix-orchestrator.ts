/**
 * Fix Orchestrator for Auto-Fix System
 *
 * Coordinates automated fix attempts across multiple failed tests:
 * - Classifies issues using IssueClassifier
 * - Applies fix strategies in priority order
 * - Retries tests after successful fixes
 * - Tracks fix history to prevent infinite loops
 * - Respects max iterations and timeout limits
 *
 * The orchestrator is the entry point for the auto-fix system,
 * coordinating all components to iteratively fix test failures.
 */

import type { Issue, IssueClassifier } from './issue-classifier.js';
import type { FixStrategy, FixContext, FixResult } from './fix-strategies.js';
import type { TestExecutionResult } from '../test-cases/types.js';
import type { TestCase } from '../test-cases/api-tests.js';
import type { TestExecutor } from '../test-executor/executor.js';
import type { ApiClient } from '../api-client/client.js';
import { createIssueClassifier } from './issue-classifier.js';
import { getAllStrategies, selectStrategies } from './fix-strategies.js';

/**
 * Configuration for fix orchestrator
 */
export interface FixOrchestratorConfig {
  /** Maximum iterations per test (default: 3) */
  maxIterations?: number;
  /** Global timeout for entire fix process in ms (default: 5 minutes) */
  globalTimeout?: number;
  /** Enable verbose logging */
  verbose?: boolean;
}

/**
 * Result of a single fix attempt
 */
export interface FixAttempt {
  /** Iteration number (1-based) */
  iteration: number;
  /** Issue that was classified */
  issue: Issue;
  /** Strategy that was applied */
  strategyName: string;
  /** Result of applying the strategy */
  success: boolean;
  /** Message from the fix strategy */
  message: string;
  /** Error details if fix failed */
  error?: string;
  /** Whether test should be retried after this fix */
  retry: boolean;
}

/**
 * Result of fix orchestration for a single test
 */
export interface TestFixResult {
  /** Test ID */
  testId: string;
  /** Test name */
  testName: string;
  /** Initial test status */
  initialStatus: 'failed' | 'error';
  /** Final test status after fix attempts */
  finalStatus: 'fixed' | 'failed';
  /** All fix attempts made */
  fixAttempts: FixAttempt[];
  /** Number of iterations performed */
  iterations: number;
  /** Final test result (if test was retried) */
  finalTestResult?: TestExecutionResult;
}

/**
 * Result of fix orchestration across all tests
 */
export interface OrchestrationResult {
  /** Total tests that needed fixing */
  totalTests: number;
  /** Tests that were successfully fixed */
  fixedTests: number;
  /** Tests that could not be fixed */
  failedTests: number;
  /** Tests skipped (no fixable issues) */
  skippedTests: number;
  /** Total fix attempts across all tests */
  totalFixAttempts: number;
  /** Successful fix attempts */
  successfulFixAttempts: number;
  /** Total duration in ms */
  duration: number;
  /** Individual test fix results */
  testResults: TestFixResult[];
}

/**
 * Fix orchestrator - coordinates auto-fix process
 */
export class FixOrchestrator {
  private readonly maxIterations: number;
  private readonly globalTimeout: number;
  private readonly verbose: boolean;
  private readonly classifier: IssueClassifier;
  private readonly strategies: FixStrategy[];
  private readonly startTime: number;

  constructor(config: FixOrchestratorConfig = {}) {
    this.maxIterations = config.maxIterations ?? 3;
    this.globalTimeout = config.globalTimeout ?? 5 * 60 * 1000; // 5 minutes
    this.verbose = config.verbose ?? false;
    this.classifier = createIssueClassifier();
    this.strategies = getAllStrategies();
    this.startTime = Date.now();
  }

  /**
   * Fix and retry failed tests
   *
   * Main entry point for fix orchestration. Processes each failed test:
   * 1. Classify issues
   * 2. Apply fixes in priority order
   * 3. Retry test after each fix
   * 4. Repeat until fixed or max iterations reached
   */
  async fixAndRetry(
    testResults: TestExecutionResult[],
    testCases: TestCase[],
    testExecutor: TestExecutor,
    apiClient: ApiClient,
    fixContext: Omit<FixContext, 'testResult'>
  ): Promise<OrchestrationResult> {
    const startMs = Date.now();
    const testFixResults: TestFixResult[] = [];

    // Filter to only failed/error tests
    const failedTests = testResults.filter(
      (r) => r.status === 'failed' || r.status === 'error'
    );

    this.log(`\n${'='.repeat(80)}`);
    this.log(`Starting auto-fix for ${failedTests.length} failed tests`);
    this.log(`Max iterations per test: ${this.maxIterations}`);
    this.log(`Global timeout: ${this.globalTimeout}ms`);
    this.log(`${'='.repeat(80)}\n`);

    for (const testResult of failedTests) {
      // Check global timeout
      if (this.isGlobalTimeoutExceeded()) {
        this.log(`⏱️  Global timeout exceeded, stopping auto-fix`);
        break;
      }

      // Find corresponding test case
      const testCase = testCases.find((tc) => tc.id === testResult.id);
      if (!testCase) {
        this.log(`⚠️  Test case not found for ${testResult.id}, skipping`);
        continue;
      }

      // Fix and retry this test
      const fixResult = await this.fixSingleTest(
        testResult,
        testCase,
        testExecutor,
        apiClient,
        fixContext
      );
      testFixResults.push(fixResult);
    }

    const duration = Date.now() - startMs;

    // Calculate summary statistics
    const fixedTests = testFixResults.filter((r) => r.finalStatus === 'fixed').length;
    const failedTestCount = testFixResults.filter((r) => r.finalStatus === 'failed').length;
    const skippedTests = testFixResults.filter((r) => r.fixAttempts.length === 0).length;
    const totalFixAttempts = testFixResults.reduce((sum, r) => sum + r.fixAttempts.length, 0);
    const successfulFixAttempts = testFixResults.reduce(
      (sum, r) => sum + r.fixAttempts.filter((a) => a.success).length,
      0
    );

    this.log(`\n${'='.repeat(80)}`);
    this.log(`Auto-fix complete:`);
    this.log(`  Total tests: ${testFixResults.length}`);
    this.log(`  Fixed:       ${fixedTests}`);
    this.log(`  Failed:      ${failedTestCount}`);
    this.log(`  Skipped:     ${skippedTests}`);
    this.log(`  Fix success rate: ${totalFixAttempts > 0 ? Math.round((successfulFixAttempts / totalFixAttempts) * 100) : 0}%`);
    this.log(`  Duration:    ${duration}ms`);
    this.log(`${'='.repeat(80)}\n`);

    return {
      totalTests: testFixResults.length,
      fixedTests,
      failedTests: failedTestCount,
      skippedTests,
      totalFixAttempts,
      successfulFixAttempts,
      duration,
      testResults: testFixResults,
    };
  }

  /**
   * Fix and retry a single test
   */
  private async fixSingleTest(
    testResult: TestExecutionResult,
    testCase: TestCase,
    testExecutor: TestExecutor,
    apiClient: ApiClient,
    baseFixContext: Omit<FixContext, 'testResult'>
  ): Promise<TestFixResult> {
    this.log(`\n🔧 Fixing test: ${testResult.name}`);

    const fixAttempts: FixAttempt[] = [];
    const appliedFixes = new Set<string>(); // Track applied fixes to prevent loops
    let currentTestResult = testResult;
    let iteration = 0;

    // Track initial status
    const initialStatus = testResult.status as 'failed' | 'error';

    while (iteration < this.maxIterations) {
      iteration++;

      // Check global timeout
      if (this.isGlobalTimeoutExceeded()) {
        this.log(`  ⏱️  Global timeout exceeded`);
        break;
      }

      // Classify issues
      const issues = this.classifier.classifyTest(currentTestResult);

      if (issues.length === 0) {
        this.log(`  ℹ️  No fixable issues identified`);
        break;
      }

      // Sort issues by priority (lower number = higher priority)
      issues.sort((a, b) => a.priority - b.priority);

      // Find first fixable issue that we haven't tried yet
      let fixApplied = false;

      for (const issue of issues) {
        // Create fix ID to track if we've already tried this
        const fixId = this.createFixId(issue);

        if (appliedFixes.has(fixId)) {
          if (this.verbose) {
            this.log(`  ⏭️  Skipping already attempted fix: ${fixId}`);
          }
          continue;
        }

        // Find applicable strategies
        const applicableStrategies = selectStrategies(issue);

        if (applicableStrategies.length === 0) {
          if (this.verbose) {
            this.log(`  ⚠️  No strategies available for issue: ${issue.description}`);
          }
          continue;
        }

        // Try first strategy
        const strategy = applicableStrategies[0];
        this.log(`  🔨 Iteration ${iteration}: Applying ${strategy.name} for ${issue.type} issue`);

        const fixContext: FixContext = {
          ...baseFixContext,
          testResult: currentTestResult,
        };

        try {
          const fixResult = await strategy.apply(issue, fixContext);

          // Record fix attempt
          const attempt: FixAttempt = {
            iteration,
            issue,
            strategyName: strategy.name,
            success: fixResult.success,
            message: fixResult.message,
            error: fixResult.error,
            retry: fixResult.retry,
          };
          fixAttempts.push(attempt);

          this.log(`  ${fixResult.success ? '✓' : '✗'} ${fixResult.message}`);

          // Mark this fix as tried
          appliedFixes.add(fixId);
          fixApplied = true;

          // If fix was successful and recommends retry, retry the test
          if (fixResult.success && fixResult.retry) {
            this.log(`  🔄 Retrying test...`);

            const retryResult = await testExecutor.runSingle(apiClient, testCase);

            if (retryResult.status === 'passed') {
              this.log(`  ✅ Test passed after fix!`);
              return {
                testId: testResult.id,
                testName: testResult.name,
                initialStatus,
                finalStatus: 'fixed',
                fixAttempts,
                iterations: iteration,
                finalTestResult: retryResult,
              };
            } else {
              this.log(`  ❌ Test still failing after fix`);
              // Update current test result for next iteration
              currentTestResult = retryResult;
            }
          }

          // Break after applying one fix per iteration
          break;
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error);
          this.log(`  ✗ Strategy threw error: ${errorMsg}`);

          // Record failed attempt
          fixAttempts.push({
            iteration,
            issue,
            strategyName: strategy.name,
            success: false,
            message: 'Strategy threw exception',
            error: errorMsg,
            retry: false,
          });

          appliedFixes.add(fixId);
          fixApplied = true;
          break;
        }
      }

      // If no fix was applied this iteration, we can't make progress
      if (!fixApplied) {
        this.log(`  ⚠️  No applicable fixes remaining`);
        break;
      }
    }

    // Test was not fixed
    this.log(`  ❌ Test could not be fixed after ${iteration} iteration(s)`);
    return {
      testId: testResult.id,
      testName: testResult.name,
      initialStatus,
      finalStatus: 'failed',
      fixAttempts,
      iterations: iteration,
      finalTestResult: currentTestResult,
    };
  }

  /**
   * Create a unique ID for a fix to prevent duplicate attempts
   */
  private createFixId(issue: Issue): string {
    return `${issue.type}-${issue.testId}-${issue.description.substring(0, 50)}`;
  }

  /**
   * Check if global timeout has been exceeded
   */
  private isGlobalTimeoutExceeded(): boolean {
    return Date.now() - this.startTime > this.globalTimeout;
  }

  /**
   * Log message to console
   */
  private log(message: string): void {
    console.log(message);
  }
}

/**
 * Create fix orchestrator instance
 *
 * @example
 * const orchestrator = createFixOrchestrator({ maxIterations: 3, verbose: true });
 * const results = await orchestrator.fixAndRetry(failedTests, testCases, executor, client, context);
 */
export function createFixOrchestrator(config?: FixOrchestratorConfig): FixOrchestrator {
  return new FixOrchestrator(config);
}

/**
 * Convenience function to fix and retry failed tests
 *
 * @example
 * const results = await fixAndRetryTests(failedTests, testCases, executor, client, context, { maxIterations: 3 });
 * console.log(`Fixed ${results.fixedTests}/${results.totalTests} tests`);
 */
export async function fixAndRetryTests(
  testResults: TestExecutionResult[],
  testCases: TestCase[],
  testExecutor: TestExecutor,
  apiClient: ApiClient,
  fixContext: Omit<FixContext, 'testResult'>,
  config?: FixOrchestratorConfig
): Promise<OrchestrationResult> {
  const orchestrator = createFixOrchestrator(config);
  return orchestrator.fixAndRetry(testResults, testCases, testExecutor, apiClient, fixContext);
}
