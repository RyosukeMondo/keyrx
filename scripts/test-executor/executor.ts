/**
 * Test Executor for Automated E2E API Testing
 *
 * Orchestrates test suite execution with:
 * - Sequential test execution for isolation
 * - Accurate timing measurements
 * - Graceful error handling with guaranteed cleanup
 * - Progress logging to console
 * - Detailed result collection
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase, TestResult } from '../test-cases/api-tests.js';
import type { TestSuiteResult, TestExecutionResult, ExpectedResults } from '../test-cases/types.js';

/**
 * Test executor configuration
 */
export interface TestExecutorConfig {
  /** Timeout per test in milliseconds */
  testTimeout?: number;
  /** Enable verbose logging */
  verbose?: boolean;
  /** Expected results database */
  expectedResults?: ExpectedResults;
}

/**
 * Test executor - orchestrates test suite execution
 */
export class TestExecutor {
  private readonly testTimeout: number;
  private readonly verbose: boolean;
  private readonly expectedResults?: ExpectedResults;

  constructor(config: TestExecutorConfig = {}) {
    this.testTimeout = config.testTimeout ?? 30000; // 30 second default
    this.verbose = config.verbose ?? false;
    this.expectedResults = config.expectedResults;
  }

  /**
   * Compute category statistics from test results
   */
  private computeCategoryStats(results: TestExecutionResult[]): Map<string, {
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    duration: number;
  }> {
    const categories = new Map<string, {
      total: number;
      passed: number;
      failed: number;
      skipped: number;
      duration: number;
    }>();

    for (const result of results) {
      const category = result.category || 'uncategorized';

      if (!categories.has(category)) {
        categories.set(category, {
          total: 0,
          passed: 0,
          failed: 0,
          skipped: 0,
          duration: 0,
        });
      }

      const stats = categories.get(category)!;
      stats.total++;
      stats.duration += result.duration;

      switch (result.status) {
        case 'passed':
          stats.passed++;
          break;
        case 'failed':
        case 'error':
          stats.failed++;
          break;
        case 'skipped':
          stats.skipped++;
          break;
      }
    }

    return categories;
  }

  /**
   * Format duration in human-readable form
   */
  private formatDuration(ms: number): string {
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    } else if (ms < 60000) {
      return `${(ms / 1000).toFixed(2)}s`;
    } else {
      const minutes = Math.floor(ms / 60000);
      const seconds = ((ms % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  }

  /**
   * Run all test cases in the suite
   */
  async runAll(client: ApiClient, cases: TestCase[]): Promise<TestSuiteResult> {
    const startTime = new Date().toISOString();
    const suiteStartMs = Date.now();
    const results: TestExecutionResult[] = [];

    this.log(`\n${'='.repeat(80)}`);
    this.log(`Starting test suite: ${cases.length} tests`);
    this.log(`${'='.repeat(80)}\n`);

    for (let i = 0; i < cases.length; i++) {
      const testCase = cases[i];
      const testNumber = i + 1;
      const testName = testCase.name || testCase.description || testCase.id;

      this.log(`[${testNumber}/${cases.length}] Running: ${testName}`);

      const result = await this.runSingle(client, testCase);
      results.push(result);

      // Log result with color coding (if supported)
      const statusIcon = result.status === 'passed' ? '✓' : result.status === 'skipped' ? '○' : '✗';
      const statusText = result.status.toUpperCase();
      this.log(`  ${statusIcon} ${statusText} (${result.duration}ms)`);

      if (result.error && this.verbose) {
        this.log(`  Error: ${result.error}`);
      }
    }

    const suiteDuration = Date.now() - suiteStartMs;
    const endTime = new Date().toISOString();

    const passed = results.filter((r) => r.status === 'passed').length;
    const failed = results.filter((r) => r.status === 'failed' || r.status === 'error').length;
    const skipped = results.filter((r) => r.status === 'skipped').length;

    // Compute category statistics
    const categoryStats = this.computeCategoryStats(results);

    this.log(`\n${'='.repeat(80)}`);
    this.log(`Test suite complete:`);
    this.log(`  Total:   ${results.length}`);
    this.log(`  Passed:  ${passed}`);
    this.log(`  Failed:  ${failed}`);
    this.log(`  Skipped: ${skipped}`);
    this.log(`  Duration: ${suiteDuration}ms`);

    // Log category breakdown
    if (categoryStats.size > 1 || (categoryStats.size === 1 && !categoryStats.has('uncategorized'))) {
      this.log(`\nCategory Performance:`);
      const sortedCategories = Array.from(categoryStats.entries()).sort((a, b) => a[0].localeCompare(b[0]));

      for (const [category, stats] of sortedCategories) {
        if (category === 'uncategorized') continue;

        const categoryName = category.charAt(0).toUpperCase() + category.slice(1);
        const statusIcon = stats.failed === 0 ? '✓' : '✗';
        const passRate = stats.total > 0 ? ((stats.passed / stats.total) * 100).toFixed(1) : '0.0';

        this.log(`  ${statusIcon} ${categoryName}: ${stats.passed}/${stats.total} passed (${passRate}%) - ${this.formatDuration(stats.duration)}`);
      }
    }

    this.log(`${'='.repeat(80)}\n`);

    return {
      total: results.length,
      passed,
      failed,
      skipped,
      duration: suiteDuration,
      results,
      startTime,
      endTime,
    };
  }

  /**
   * Verify daemon state before test execution
   * Throws if daemon is in an unexpected state
   */
  private async verifyDaemonState(client: ApiClient): Promise<void> {
    try {
      const statusResponse = await client.getStatus();

      // Verify daemon is responsive
      if (!statusResponse.data) {
        throw new Error('Daemon status response is empty');
      }

      // Log state for debugging if verbose
      if (this.verbose) {
        this.log(`  [Isolation] Daemon state verified: running=${statusResponse.data.running}`);
      }
    } catch (error) {
      throw new Error(`Daemon state verification failed: ${error instanceof Error ? error.message : 'unknown error'}`);
    }
  }

  /**
   * Run a single test case with timeout and error handling
   */
  async runSingle(client: ApiClient, testCase: TestCase): Promise<TestExecutionResult> {
    const startMs = Date.now();

    try {
      // Verify daemon state before test (isolation guard)
      if (this.verbose) {
        this.log(`  [Isolation] Verifying daemon state`);
      }
      await this.verifyDaemonState(client);

      // Execute test with timeout
      const result = await this.executeWithTimeout(client, testCase);
      const duration = Date.now() - startMs;

      return {
        id: testCase.id,
        name: testCase.name || testCase.description || testCase.id,
        endpoint: testCase.endpoint,
        scenario: testCase.scenario,
        category: testCase.category,
        priority: testCase.priority,
        status: result.status,
        duration,
        error: result.error,
        actual: result.actual,
        expected: result.expected,
        diff: result.diff,
      };
    } catch (error) {
      const duration = Date.now() - startMs;

      // Capture error details
      let errorMessage = 'Unknown error';
      let stackTrace: string | undefined;

      if (error instanceof Error) {
        errorMessage = error.message;
        stackTrace = error.stack;
      } else if (typeof error === 'string') {
        errorMessage = error;
      } else {
        errorMessage = JSON.stringify(error);
      }

      return {
        id: testCase.id,
        name: testCase.name || testCase.description || testCase.id,
        endpoint: testCase.endpoint,
        scenario: testCase.scenario,
        category: testCase.category,
        priority: testCase.priority,
        status: 'error',
        duration,
        error: errorMessage,
        stackTrace,
      };
    }
  }

  /**
   * Execute test with timeout wrapper
   */
  private async executeWithTimeout(client: ApiClient, testCase: TestCase): Promise<{
    status: 'passed' | 'failed' | 'skipped';
    error?: string;
    actual?: unknown;
    expected?: unknown;
    diff?: unknown;
  }> {
    return new Promise(async (resolve, reject) => {
      // Setup timeout
      const timeoutId = setTimeout(() => {
        reject(new Error(`Test timeout after ${this.testTimeout}ms`));
      }, this.testTimeout);

      try {
        // Step 1: Setup
        if (this.verbose) {
          this.log(`  [Setup] Running setup function`);
        }
        await testCase.setup(client);

        // Step 2: Execute
        if (this.verbose) {
          this.log(`  [Execute] Running test execution`);
        }
        const response = await testCase.execute(client);

        // Step 3: Handle workflow tests (which return success/failure directly)
        let testResult: TestResult;
        if (testCase.category === 'workflows' || !testCase.assert) {
          // Workflow test - check for success field or expectedStatus
          const isSuccess = (response as any).success !== false;
          testResult = {
            passed: isSuccess,
            actual: response,
            expected: testCase.expectedResponse || { success: true },
          };
        } else {
          // Step 3: Get expected result
          const expectedResult = this.getExpectedResult(testCase);

          // Step 4: Assert
          if (this.verbose) {
            this.log(`  [Assert] Validating response`);
          }
          testResult = testCase.assert(response, expectedResult);
        }

        // Step 5: Cleanup (always runs)
        try {
          if (this.verbose) {
            this.log(`  [Cleanup] Running cleanup function`);
          }
          await testCase.cleanup(client);
        } catch (cleanupError) {
          // Log cleanup errors but don't fail the test
          if (this.verbose) {
            console.error(`  [Cleanup] Warning: cleanup failed:`, cleanupError);
          }
        }

        clearTimeout(timeoutId);

        // Resolve based on assertion result
        resolve({
          status: testResult.passed ? 'passed' : 'failed',
          error: testResult.error,
          actual: testResult.actual,
          expected: testResult.expected,
          diff: testResult.diff,
        });
      } catch (error) {
        // Ensure cleanup runs even on error
        try {
          if (this.verbose) {
            this.log(`  [Cleanup] Running cleanup after error`);
          }
          await testCase.cleanup(client);
        } catch (cleanupError) {
          // Log cleanup errors but don't mask original error
          if (this.verbose) {
            console.error(`  [Cleanup] Warning: cleanup failed:`, cleanupError);
          }
        }

        clearTimeout(timeoutId);
        reject(error);
      }
    });
  }

  /**
   * Get expected result from database
   */
  private getExpectedResult(testCase: TestCase): {
    method: string;
    status: number;
    body: unknown;
  } {
    // If no expected results database, return placeholder
    if (!this.expectedResults) {
      return {
        method: 'GET',
        status: 200,
        body: {},
      };
    }

    // Lookup expected result from database
    const endpointResults = this.expectedResults.endpoints[testCase.endpoint];
    if (!endpointResults) {
      // Try common scenarios
      const commonScenario = this.expectedResults.commonScenarios?.[testCase.scenario];
      if (commonScenario) {
        return commonScenario;
      }

      // Return placeholder if not found
      return {
        method: 'GET',
        status: 200,
        body: {},
      };
    }

    const scenario = endpointResults.scenarios[testCase.scenario];
    if (!scenario) {
      return {
        method: 'GET',
        status: 200,
        body: {},
      };
    }

    return scenario;
  }

  /**
   * Log message to console
   */
  private log(message: string): void {
    console.log(message);
  }
}

/**
 * Create test executor instance
 *
 * @example
 * const executor = createTestExecutor({ testTimeout: 30000, verbose: true });
 * const results = await executor.runAll(client, testCases);
 */
export function createTestExecutor(config?: TestExecutorConfig): TestExecutor {
  return new TestExecutor(config);
}

/**
 * Run test suite with default configuration
 *
 * @example
 * const results = await runTestSuite(client, testCases);
 */
export async function runTestSuite(
  client: ApiClient,
  cases: TestCase[],
  config?: TestExecutorConfig
): Promise<TestSuiteResult> {
  const executor = createTestExecutor(config);
  return executor.runAll(client, cases);
}
