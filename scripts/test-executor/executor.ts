/**
 * Test Executor
 *
 * Orchestrates test suite execution with timing, error handling, and result collection.
 * Runs tests sequentially to avoid race conditions and ensure deterministic behavior.
 */

import { ApiClient } from '../api-client/client';
import { TestCase } from '../test-cases/test-utils';

export interface TestResult {
  id: string;
  name: string;
  status: 'pass' | 'fail' | 'error' | 'timeout';
  duration: number;
  error?: string;
  actual?: unknown;
  expected?: unknown;
  diff?: Array<{
    path: string;
    expected: unknown;
    actual: unknown;
  }>;
}

export interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  errors: number;
  duration: number;
  results: TestResult[];
}

export interface ExecutorConfig {
  timeoutMs?: number;
  continueOnFailure?: boolean;
  verbose?: boolean;
}

/**
 * Test executor that runs test cases and collects results
 */
export class TestExecutor {
  private config: Required<ExecutorConfig>;

  constructor(config: ExecutorConfig = {}) {
    this.config = {
      timeoutMs: config.timeoutMs ?? 30000,
      continueOnFailure: config.continueOnFailure ?? true,
      verbose: config.verbose ?? false,
    };
  }

  /**
   * Run all test cases in the suite
   */
  async runAll(client: ApiClient, cases: TestCase[]): Promise<TestSuiteResult> {
    const startTime = Date.now();
    const results: TestResult[] = [];

    this.log(`Running ${cases.length} test cases...`);

    for (let i = 0; i < cases.length; i++) {
      const testCase = cases[i];
      this.log(`[${i + 1}/${cases.length}] ${testCase.name}`);

      const result = await this.runSingle(client, testCase);
      results.push(result);

      if (result.status === 'pass') {
        this.log(`  ✓ PASS (${result.duration}ms)`);
      } else if (result.status === 'fail') {
        this.log(`  ✗ FAIL (${result.duration}ms): ${result.error}`);
        if (result.diff && result.diff.length > 0) {
          this.log(`    Differences:`);
          for (const diff of result.diff.slice(0, 3)) {
            this.log(`      ${diff.path}: expected ${JSON.stringify(diff.expected)}, got ${JSON.stringify(diff.actual)}`);
          }
          if (result.diff.length > 3) {
            this.log(`      ... and ${result.diff.length - 3} more`);
          }
        }
      } else if (result.status === 'error') {
        this.log(`  ✗ ERROR (${result.duration}ms): ${result.error}`);
      } else if (result.status === 'timeout') {
        this.log(`  ✗ TIMEOUT (${result.duration}ms)`);
      }

      // Stop execution if configured to not continue on failure
      if (!this.config.continueOnFailure && result.status !== 'pass') {
        this.log('Stopping execution due to failure');
        break;
      }
    }

    const duration = Date.now() - startTime;
    const passed = results.filter(r => r.status === 'pass').length;
    const failed = results.filter(r => r.status === 'fail').length;
    const errors = results.filter(r => r.status === 'error' || r.status === 'timeout').length;

    const summary: TestSuiteResult = {
      total: results.length,
      passed,
      failed,
      errors,
      duration,
      results,
    };

    this.log('');
    this.log(`Test Suite Complete: ${passed} passed, ${failed} failed, ${errors} errors (${duration}ms)`);

    return summary;
  }

  /**
   * Run a single test case
   */
  async runSingle(client: ApiClient, testCase: TestCase): Promise<TestResult> {
    const startTime = Date.now();
    let context: any;

    try {
      // Setup phase
      context = await this.withTimeout(
        testCase.setup(client),
        this.config.timeoutMs,
        'Setup timeout'
      );

      // Execute phase
      const actual = await this.withTimeout(
        testCase.execute(client, context),
        this.config.timeoutMs,
        'Execute timeout'
      );

      // Assert phase
      const validationResult = testCase.assert(actual, undefined);
      const duration = Date.now() - startTime;

      // Cleanup phase (always run, even if test failed)
      if (testCase.cleanup) {
        try {
          await this.withTimeout(
            testCase.cleanup(client, context),
            this.config.timeoutMs,
            'Cleanup timeout'
          );
        } catch (cleanupError) {
          // Log cleanup errors but don't fail the test
          this.log(`  Warning: Cleanup failed: ${cleanupError}`);
        }
      }

      if (validationResult.passed) {
        return {
          id: testCase.id,
          name: testCase.name,
          status: 'pass',
          duration,
        };
      } else {
        return {
          id: testCase.id,
          name: testCase.name,
          status: 'fail',
          duration,
          error: validationResult.message,
          diff: validationResult.diff,
          actual: undefined, // Could extract from validation result if needed
          expected: undefined,
        };
      }
    } catch (error) {
      const duration = Date.now() - startTime;

      // Attempt cleanup even on error
      if (testCase.cleanup) {
        try {
          await testCase.cleanup(client, context);
        } catch (cleanupError) {
          // Ignore cleanup errors after test error
        }
      }

      const isTimeout = error instanceof Error && error.message.includes('timeout');

      return {
        id: testCase.id,
        name: testCase.name,
        status: isTimeout ? 'timeout' : 'error',
        duration,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Wrap a promise with timeout
   */
  private async withTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number,
    errorMessage: string
  ): Promise<T> {
    return Promise.race([
      promise,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error(errorMessage)), timeoutMs)
      ),
    ]);
  }

  /**
   * Log message if verbose mode is enabled
   */
  private log(message: string): void {
    if (this.config.verbose) {
      console.log(message);
    }
  }
}

/**
 * Create a test executor with default configuration
 */
export function createExecutor(config?: ExecutorConfig): TestExecutor {
  return new TestExecutor(config);
}
