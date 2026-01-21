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

      this.log(`[${testNumber}/${cases.length}] Running: ${testCase.name}`);

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

    this.log(`\n${'='.repeat(80)}`);
    this.log(`Test suite complete:`);
    this.log(`  Total:   ${results.length}`);
    this.log(`  Passed:  ${passed}`);
    this.log(`  Failed:  ${failed}`);
    this.log(`  Skipped: ${skipped}`);
    this.log(`  Duration: ${suiteDuration}ms`);
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
   * Run a single test case with timeout and error handling
   */
  async runSingle(client: ApiClient, testCase: TestCase): Promise<TestExecutionResult> {
    const startMs = Date.now();

    try {
      // Execute test with timeout
      const result = await this.executeWithTimeout(client, testCase);
      const duration = Date.now() - startMs;

      return {
        id: testCase.id,
        name: testCase.name,
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
        name: testCase.name,
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

        // Step 3: Get expected result
        const expectedResult = this.getExpectedResult(testCase);

        // Step 4: Assert
        if (this.verbose) {
          this.log(`  [Assert] Validating response`);
        }
        const testResult: TestResult = testCase.assert(response.data, expectedResult);

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
