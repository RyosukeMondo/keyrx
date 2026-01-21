/**
 * Validation Reporter
 *
 * Formats test results for human-readable output and machine-parseable JSON.
 * Provides color-coded terminal output and detailed diff visualization.
 */

import { TestSuiteResult, TestResult } from '../test-executor/executor';

export interface ReportOptions {
  noColor?: boolean;
  maxDiffLines?: number;
}

/**
 * Color codes for terminal output
 */
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  gray: '\x1b[90m',
  bold: '\x1b[1m',
};

/**
 * Validation reporter for formatting test results
 */
export class ValidationReporter {
  private options: Required<ReportOptions>;

  constructor(options: ReportOptions = {}) {
    const noColor = options.noColor ?? (process.env.CI === 'true' || process.env.NO_COLOR === '1');
    this.options = {
      noColor,
      maxDiffLines: options.maxDiffLines ?? 100,
    };
  }

  /**
   * Format results for human-readable output
   */
  formatHuman(result: TestSuiteResult): string {
    const lines: string[] = [];

    // Header
    lines.push('');
    lines.push(this.color('bold', '═'.repeat(80)));
    lines.push(this.color('bold', '  TEST SUITE RESULTS'));
    lines.push(this.color('bold', '═'.repeat(80)));
    lines.push('');

    // Summary
    const passRate = result.total > 0 ? ((result.passed / result.total) * 100).toFixed(1) : '0.0';
    lines.push(`  Total:   ${result.total} tests`);
    lines.push(`  ${this.color('green', '✓ Passed:')} ${result.passed}`);
    lines.push(`  ${this.color('red', '✗ Failed:')} ${result.failed}`);
    lines.push(`  ${this.color('yellow', '⚠ Errors:')}  ${result.errors}`);
    lines.push(`  Duration: ${this.formatDuration(result.duration)}`);
    lines.push(`  Pass Rate: ${passRate}%`);
    lines.push('');

    // Individual test results
    if (result.results.length > 0) {
      lines.push(this.color('bold', '  TEST DETAILS'));
      lines.push(this.color('gray', '  ' + '─'.repeat(78)));
      lines.push('');

      for (let i = 0; i < result.results.length; i++) {
        const testResult = result.results[i];
        lines.push(this.formatTestResult(testResult, i + 1));
        lines.push('');
      }
    }

    // Footer
    lines.push(this.color('bold', '═'.repeat(80)));
    const overallStatus = result.failed === 0 && result.errors === 0 ? 'PASSED' : 'FAILED';
    const statusColor = overallStatus === 'PASSED' ? 'green' : 'red';
    lines.push(this.color(statusColor, `  ${overallStatus}: ${result.passed}/${result.total} tests passed`));
    lines.push(this.color('bold', '═'.repeat(80)));
    lines.push('');

    return lines.join('\n');
  }

  /**
   * Format a single test result
   */
  private formatTestResult(result: TestResult, index: number): string {
    const lines: string[] = [];
    const prefix = `  [${index}]`;

    // Test name and status
    if (result.status === 'pass') {
      lines.push(`${prefix} ${this.color('green', '✓')} ${result.name}`);
      lines.push(`       ${this.color('gray', `Duration: ${result.duration}ms`)}`);
    } else if (result.status === 'fail') {
      lines.push(`${prefix} ${this.color('red', '✗')} ${result.name}`);
      lines.push(`       ${this.color('gray', `Duration: ${result.duration}ms`)}`);
      lines.push(`       ${this.color('red', `Error: ${result.error || 'Unknown failure'}`)}`);

      // Show diff if available
      if (result.diff && result.diff.length > 0) {
        lines.push(`       ${this.color('yellow', 'Differences:')}`);
        const maxDiffs = Math.min(result.diff.length, this.options.maxDiffLines);
        for (let i = 0; i < maxDiffs; i++) {
          const diff = result.diff[i];
          lines.push(`         ${this.color('gray', diff.path)}`);
          lines.push(`           ${this.color('red', '- Expected:')} ${this.formatValue(diff.expected)}`);
          lines.push(`           ${this.color('green', '+ Actual:  ')} ${this.formatValue(diff.actual)}`);
        }
        if (result.diff.length > maxDiffs) {
          lines.push(`         ${this.color('gray', `... and ${result.diff.length - maxDiffs} more differences`)}`);
        }
      }
    } else if (result.status === 'error') {
      lines.push(`${prefix} ${this.color('yellow', '⚠')} ${result.name}`);
      lines.push(`       ${this.color('gray', `Duration: ${result.duration}ms`)}`);
      lines.push(`       ${this.color('yellow', `Error: ${result.error || 'Unknown error'}`)}`);
    } else if (result.status === 'timeout') {
      lines.push(`${prefix} ${this.color('yellow', '⏱')} ${result.name}`);
      lines.push(`       ${this.color('gray', `Duration: ${result.duration}ms`)}`);
      lines.push(`       ${this.color('yellow', 'Test timeout exceeded')}`);
    }

    return lines.join('\n');
  }

  /**
   * Format results as JSON
   */
  formatJson(result: TestSuiteResult): string {
    const report = {
      version: '1.0',
      timestamp: new Date().toISOString(),
      summary: {
        total: result.total,
        passed: result.passed,
        failed: result.failed,
        errors: result.errors,
        duration: result.duration,
        passRate: result.total > 0 ? (result.passed / result.total) * 100 : 0,
      },
      results: result.results.map(r => ({
        id: r.id,
        name: r.name,
        status: r.status,
        duration: r.duration,
        error: r.error,
        diff: r.diff,
      })),
    };

    return JSON.stringify(report, null, 2);
  }

  /**
   * Apply color to text if colors are enabled
   */
  private color(colorName: keyof typeof colors, text: string): string {
    if (this.options.noColor) {
      return text;
    }
    return `${colors[colorName]}${text}${colors.reset}`;
  }

  /**
   * Format duration in a human-readable way
   */
  private formatDuration(ms: number): string {
    if (ms < 1000) {
      return `${ms}ms`;
    }
    const seconds = (ms / 1000).toFixed(2);
    return `${seconds}s`;
  }

  /**
   * Format a value for display in diff
   */
  private formatValue(value: unknown): string {
    if (value === undefined) {
      return this.color('gray', 'undefined');
    }
    if (value === null) {
      return this.color('gray', 'null');
    }
    if (typeof value === 'string') {
      // Truncate long strings
      if (value.length > 80) {
        return `"${value.substring(0, 77)}..."`;
      }
      return `"${value}"`;
    }
    if (typeof value === 'object') {
      const json = JSON.stringify(value);
      if (json.length > 80) {
        return json.substring(0, 77) + '...';
      }
      return json;
    }
    return String(value);
  }
}

/**
 * Create a validation reporter
 */
export function createReporter(options?: ReportOptions): ValidationReporter {
  return new ValidationReporter(options);
}

/**
 * Quick helper to print results to console
 */
export function printResults(result: TestSuiteResult, options?: ReportOptions): void {
  const reporter = new ValidationReporter(options);
  console.log(reporter.formatHuman(result));
}

/**
 * Quick helper to get JSON results
 */
export function toJson(result: TestSuiteResult): string {
  const reporter = new ValidationReporter();
  return reporter.formatJson(result);
}
