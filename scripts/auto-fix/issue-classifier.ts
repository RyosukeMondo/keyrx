/**
 * Issue Classifier
 *
 * Classifies test failures into actionable categories and identifies fixable issues.
 * Analyzes error patterns to determine appropriate remediation strategies.
 */

import { TestResult } from '../test-executor/executor';
import { Diff } from '../comparator/response-comparator';

export type IssueType = 'network' | 'validation' | 'logic' | 'data';

export interface Issue {
  type: IssueType;
  fixable: boolean;
  priority: number;
  description: string;
  suggestedFix?: string;
  testId: string;
  testName: string;
  errorMessage?: string;
  diffs?: Diff[];
}

/**
 * Issue classifier for test failures
 */
export class IssueClassifier {
  /**
   * Classify a test result and extract issues
   */
  classify(testResult: TestResult): Issue[] {
    if (testResult.status === 'pass') {
      return [];
    }

    const issues: Issue[] = [];
    const baseIssue = {
      testId: testResult.id,
      testName: testResult.name,
      errorMessage: testResult.error,
    };

    // Analyze error type
    if (testResult.status === 'error' || testResult.status === 'timeout') {
      issues.push(...this.classifyError(testResult, baseIssue));
    }

    // Analyze validation failures
    if (testResult.status === 'fail' && testResult.diff && testResult.diff.length > 0) {
      issues.push(...this.classifyValidationFailure(testResult, baseIssue));
    }

    return issues;
  }

  /**
   * Classify error-type failures (network, timeout, etc.)
   */
  private classifyError(
    testResult: TestResult,
    baseIssue: Pick<Issue, 'testId' | 'testName' | 'errorMessage'>
  ): Issue[] {
    const error = testResult.error?.toLowerCase() || '';

    // Network errors - highly fixable
    if (
      error.includes('econnrefused') ||
      error.includes('econnreset') ||
      error.includes('fetch failed') ||
      error.includes('network error')
    ) {
      return [
        {
          ...baseIssue,
          type: 'network',
          fixable: true,
          priority: 1,
          description: 'Connection refused - daemon not running or not ready',
          suggestedFix: 'Restart daemon and wait for ready state',
        },
      ];
    }

    // Timeout errors - fixable by waiting or restarting
    if (
      testResult.status === 'timeout' ||
      error.includes('timeout') ||
      error.includes('timed out')
    ) {
      return [
        {
          ...baseIssue,
          type: 'network',
          fixable: true,
          priority: 2,
          description: 'Request timeout - daemon may be slow or unresponsive',
          suggestedFix: 'Increase timeout or restart daemon',
        },
      ];
    }

    // HTTP error codes
    if (error.includes('500') || error.includes('internal server error')) {
      return [
        {
          ...baseIssue,
          type: 'logic',
          fixable: false,
          priority: 3,
          description: 'Internal server error - daemon logic issue',
        },
      ];
    }

    if (error.includes('404') || error.includes('not found')) {
      return [
        {
          ...baseIssue,
          type: 'data',
          fixable: true,
          priority: 2,
          description: 'Resource not found - may need to reseed fixtures',
          suggestedFix: 'Check test setup - ensure resources are created',
        },
      ];
    }

    // Parse errors - schema issues
    if (error.includes('json parse') || error.includes('invalid json')) {
      return [
        {
          ...baseIssue,
          type: 'validation',
          fixable: false,
          priority: 3,
          description: 'Invalid JSON response - daemon API issue',
        },
      ];
    }

    // Unknown errors - not fixable
    return [
      {
        ...baseIssue,
        type: 'logic',
        fixable: false,
        priority: 3,
        description: `Unknown error: ${testResult.error}`,
      },
    ];
  }

  /**
   * Classify validation failures (diff analysis)
   */
  private classifyValidationFailure(
    testResult: TestResult,
    baseIssue: Pick<Issue, 'testId' | 'testName' | 'errorMessage'>
  ): Issue[] {
    const diffs = testResult.diff || [];
    const issues: Issue[] = [];

    // Categorize diffs by type
    const typeMismatches = diffs.filter(d => d.type === 'type-mismatch');
    const missingFields = diffs.filter(d => d.type === 'missing');
    const extraFields = diffs.filter(d => d.type === 'extra');
    const valueMismatches = diffs.filter(d => d.type === 'value-mismatch');

    // Type mismatches indicate schema issues
    if (typeMismatches.length > 0) {
      issues.push({
        ...baseIssue,
        type: 'validation',
        fixable: true,
        priority: 1,
        description: `Type mismatch in ${typeMismatches.length} field(s)`,
        suggestedFix: 'Update expected results to match API schema',
        diffs: typeMismatches,
      });
    }

    // Missing fields indicate schema changes or data issues
    if (missingFields.length > 0) {
      const hasCriticalFields = missingFields.some(d =>
        d.path.includes('status') || d.path.includes('name') || d.path.includes('id')
      );

      issues.push({
        ...baseIssue,
        type: hasCriticalFields ? 'logic' : 'validation',
        fixable: !hasCriticalFields,
        priority: hasCriticalFields ? 3 : 1,
        description: `Missing ${missingFields.length} expected field(s)`,
        suggestedFix: hasCriticalFields
          ? undefined
          : 'Update expected results or check test setup',
        diffs: missingFields,
      });
    }

    // Extra fields are usually benign (API added new fields)
    if (extraFields.length > 0) {
      issues.push({
        ...baseIssue,
        type: 'validation',
        fixable: true,
        priority: 1,
        description: `${extraFields.length} unexpected field(s) in response`,
        suggestedFix: 'Update expected results to include new fields',
        diffs: extraFields,
      });
    }

    // Value mismatches could be logic or data issues
    if (valueMismatches.length > 0) {
      // Check if values are just different strings/numbers vs structural issues
      const isDataMismatch = valueMismatches.every(d => {
        const actual = d.actual;
        const expected = d.expected;
        // If both are primitives (not objects), likely data issue
        return (
          typeof actual !== 'object' &&
          typeof expected !== 'object' &&
          actual !== null &&
          expected !== null
        );
      });

      issues.push({
        ...baseIssue,
        type: isDataMismatch ? 'data' : 'logic',
        fixable: isDataMismatch,
        priority: isDataMismatch ? 2 : 3,
        description: `Value mismatch in ${valueMismatches.length} field(s)`,
        suggestedFix: isDataMismatch
          ? 'Check test data or update expected results'
          : undefined,
        diffs: valueMismatches,
      });
    }

    // If no specific issues identified but we have diffs, generic validation issue
    if (issues.length === 0 && diffs.length > 0) {
      issues.push({
        ...baseIssue,
        type: 'validation',
        fixable: false,
        priority: 3,
        description: `Response mismatch - ${diffs.length} differences found`,
        diffs,
      });
    }

    return issues;
  }

  /**
   * Classify multiple test results
   */
  classifyAll(testResults: TestResult[]): Issue[] {
    return testResults.flatMap(result => this.classify(result));
  }

  /**
   * Get fixable issues sorted by priority
   */
  getFixableIssues(issues: Issue[]): Issue[] {
    return issues
      .filter(issue => issue.fixable)
      .sort((a, b) => a.priority - b.priority);
  }

  /**
   * Group issues by type
   */
  groupByType(issues: Issue[]): Map<IssueType, Issue[]> {
    const groups = new Map<IssueType, Issue[]>();
    for (const issue of issues) {
      const existing = groups.get(issue.type) || [];
      existing.push(issue);
      groups.set(issue.type, existing);
    }
    return groups;
  }
}

/**
 * Create an issue classifier
 */
export function createClassifier(): IssueClassifier {
  return new IssueClassifier();
}
