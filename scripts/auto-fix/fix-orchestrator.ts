/**
 * Fix Orchestrator
 *
 * Coordinates automatic fixing of test failures through iterative
 * application of fix strategies with retry logic and history tracking.
 */

import { TestResult, TestExecutor } from '../test-executor/executor';
import { IssueClassifier, Issue } from './issue-classifier';
import { FixStrategyRegistry, FixContext, FixResult } from './fix-strategies';
import { TestCase } from '../test-cases/test-utils';
import { ApiClient } from '../api-client/client';
import { DaemonFixture } from '../fixtures/daemon-fixture';

export interface FixAttempt {
  strategy: string;
  success: boolean;
  message: string;
  timestamp: number;
}

export interface FixedTestResult {
  testId: string;
  testName: string;
  initialStatus: 'pass' | 'fail' | 'error' | 'timeout';
  finalStatus: 'pass' | 'fail' | 'error' | 'timeout' | 'fixed' | 'unfixed';
  fixAttempts: FixAttempt[];
  iterations: number;
}

export interface FixOrchestratorResult {
  totalTests: number;
  fixedTests: number;
  unfixedTests: number;
  totalIterations: number;
  results: FixedTestResult[];
  duration: number;
}

export interface OrchestratorConfig {
  maxIterations: number;
  maxTotalTime?: number;
  daemon: DaemonFixture;
  apiClient: ApiClient;
  testCases: TestCase[];
  executor: TestExecutor;
}

/**
 * Orchestrates automatic fixing and retrying of failed tests
 */
export class FixOrchestrator {
  private classifier: IssueClassifier;
  private registry: FixStrategyRegistry;
  private fixHistory: Set<string>;
  private config: OrchestratorConfig;

  constructor(
    config: OrchestratorConfig,
    classifier?: IssueClassifier,
    registry?: FixStrategyRegistry
  ) {
    this.config = config;
    this.classifier = classifier || new IssueClassifier();
    this.registry = registry || new FixStrategyRegistry();
    this.fixHistory = new Set<string>();
  }

  /**
   * Fix and retry failed tests iteratively
   */
  async fixAndRetry(testResults: TestResult[]): Promise<FixOrchestratorResult> {
    const startTime = Date.now();
    const maxTime = this.config.maxTotalTime || 5 * 60 * 1000; // 5 minutes default

    const results: FixedTestResult[] = [];
    let totalIterations = 0;

    // Initialize results for all tests
    for (const testResult of testResults) {
      results.push({
        testId: testResult.id,
        testName: testResult.name,
        initialStatus: testResult.status,
        finalStatus: testResult.status === 'pass' ? 'pass' : 'unfixed',
        fixAttempts: [],
        iterations: 0,
      });
    }

    // Get failed tests that need fixing
    let failedTests = testResults.filter(r => r.status !== 'pass');

    console.log('');
    console.log(`Auto-fix: ${failedTests.length} failed test(s) to fix`);
    console.log(`Max iterations: ${this.config.maxIterations}`);
    console.log('');

    // Iterative fix loop
    for (let iteration = 0; iteration < this.config.maxIterations; iteration++) {
      if (failedTests.length === 0) {
        console.log('✓ All tests passing!');
        break;
      }

      // Check timeout
      if (Date.now() - startTime > maxTime) {
        console.log('⏱ Fix timeout reached, stopping iteration');
        break;
      }

      totalIterations++;
      console.log(`\n═══ Iteration ${iteration + 1}/${this.config.maxIterations} ═══`);
      console.log(`Failed tests: ${failedTests.length}`);

      // Classify issues
      const issues = this.classifier.classifyAll(failedTests);
      const fixableIssues = this.classifier.getFixableIssues(issues);

      if (fixableIssues.length === 0) {
        console.log('✗ No fixable issues found');
        break;
      }

      console.log(`Found ${fixableIssues.length} fixable issue(s)`);

      // Group issues by test
      const issuesByTest = this.groupIssuesByTest(fixableIssues);

      // Apply fixes for each test
      const testsToRetry: string[] = [];

      for (const [testId, testIssues] of issuesByTest) {
        const testCase = this.findTestCase(testId);
        if (!testCase) {
          console.log(`  ✗ Test case not found: ${testId}`);
          continue;
        }

        const resultIndex = results.findIndex(r => r.testId === testId);
        const result = results[resultIndex];

        console.log(`\n  Test: ${result.testName}`);

        // Try to fix each issue
        let shouldRetry = false;

        for (const issue of testIssues) {
          // Check if we've already tried to fix this exact issue
          const issueKey = this.getIssueKey(issue);
          if (this.fixHistory.has(issueKey)) {
            console.log(`    Skipping duplicate fix attempt: ${issue.description}`);
            continue;
          }

          console.log(`    Issue: ${issue.description}`);

          const fixContext: FixContext = {
            daemon: this.config.daemon,
            apiClient: this.config.apiClient,
            testCase,
          };

          const fixResult = await this.registry.applyFix(issue, fixContext);

          // Record attempt
          const attempt: FixAttempt = {
            strategy: issue.suggestedFix || 'unknown',
            success: fixResult.success,
            message: fixResult.message,
            timestamp: Date.now(),
          };

          result.fixAttempts.push(attempt);

          if (fixResult.success) {
            console.log(`    ✓ ${fixResult.message}`);
            this.fixHistory.add(issueKey);

            if (fixResult.retry) {
              shouldRetry = true;
            }
          } else {
            console.log(`    ✗ ${fixResult.message}`);
          }
        }

        if (shouldRetry) {
          testsToRetry.push(testId);
          result.iterations++;
        }
      }

      // Retry tests that had successful fixes
      if (testsToRetry.length > 0) {
        console.log(`\n  Retrying ${testsToRetry.length} test(s)...`);

        const retryResults = await this.retryTests(testsToRetry);

        // Update results
        for (const retryResult of retryResults) {
          const resultIndex = results.findIndex(r => r.testId === retryResult.id);
          if (resultIndex !== -1) {
            const result = results[resultIndex];
            result.finalStatus = retryResult.status === 'pass' ? 'fixed' : 'unfixed';

            if (retryResult.status === 'pass') {
              console.log(`    ✓ ${retryResult.name} - FIXED`);
            } else {
              console.log(`    ✗ ${retryResult.name} - Still failing`);
            }
          }
        }

        // Update failed tests list
        failedTests = retryResults.filter(r => r.status !== 'pass');
      } else {
        console.log('\n  No tests to retry');
        break;
      }
    }

    const duration = Date.now() - startTime;
    const fixedTests = results.filter(r => r.finalStatus === 'fixed').length;
    const unfixedTests = results.filter(
      r => r.finalStatus === 'unfixed' || r.finalStatus === 'fail'
    ).length;

    console.log('\n═══ Auto-fix Complete ═══');
    console.log(`Fixed: ${fixedTests}`);
    console.log(`Unfixed: ${unfixedTests}`);
    console.log(`Iterations: ${totalIterations}`);
    console.log(`Duration: ${(duration / 1000).toFixed(2)}s`);

    return {
      totalTests: results.length,
      fixedTests,
      unfixedTests,
      totalIterations,
      results,
      duration,
    };
  }

  /**
   * Retry specific tests by ID
   */
  private async retryTests(testIds: string[]): Promise<TestResult[]> {
    const testCases = testIds
      .map(id => this.findTestCase(id))
      .filter((tc): tc is TestCase => tc !== undefined);

    if (testCases.length === 0) {
      return [];
    }

    // Run only these specific tests
    const executor = this.config.executor;
    const client = this.config.apiClient;

    const results: TestResult[] = [];

    for (const testCase of testCases) {
      const result = await executor.runSingle(client, testCase);
      results.push(result);
    }

    return results;
  }

  /**
   * Find test case by ID
   */
  private findTestCase(testId: string): TestCase | undefined {
    return this.config.testCases.find(tc => tc.id === testId);
  }

  /**
   * Group issues by test ID
   */
  private groupIssuesByTest(issues: Issue[]): Map<string, Issue[]> {
    const grouped = new Map<string, Issue[]>();

    for (const issue of issues) {
      const existing = grouped.get(issue.testId) || [];
      existing.push(issue);
      grouped.set(issue.testId, existing);
    }

    return grouped;
  }

  /**
   * Generate unique key for issue to prevent duplicate fixes
   */
  private getIssueKey(issue: Issue): string {
    return `${issue.testId}:${issue.type}:${issue.description}`;
  }

  /**
   * Get fix history for debugging
   */
  getFixHistory(): Set<string> {
    return new Set(this.fixHistory);
  }

  /**
   * Clear fix history (useful for testing)
   */
  clearHistory(): void {
    this.fixHistory.clear();
  }
}

/**
 * Create a fix orchestrator with default configuration
 */
export function createOrchestrator(config: OrchestratorConfig): FixOrchestrator {
  return new FixOrchestrator(config);
}
