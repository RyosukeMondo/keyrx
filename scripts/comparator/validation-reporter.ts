/**
 * Validation reporter for API test results
 *
 * Formats test comparison results for both human consumption (console output)
 * and machine processing (JSON reports).
 *
 * Features:
 * - Color-coded console output (with NO_COLOR support)
 * - Structured JSON reports
 * - Diff visualization
 * - Test duration and timing
 * - Summary statistics
 */

import { ComparisonResult, Diff } from './response-comparator.js';

/**
 * Test result from test executor
 */
export interface TestResult {
  /** Unique test identifier */
  id: string;
  /** Human-readable test name */
  name: string;
  /** Test status */
  status: 'pass' | 'fail' | 'skip' | 'error';
  /** Test duration in milliseconds */
  duration: number;
  /** Comparison result (if status is 'fail') */
  comparison?: ComparisonResult;
  /** Error message (if status is 'error') */
  error?: string;
  /** Actual response received */
  actual?: unknown;
  /** Expected response */
  expected?: unknown;
}

/**
 * Test suite result
 */
export interface TestSuiteResult {
  /** Suite name/description */
  name?: string;
  /** Total test count */
  total: number;
  /** Number of passed tests */
  passed: number;
  /** Number of failed tests */
  failed: number;
  /** Number of skipped tests */
  skipped: number;
  /** Number of error tests */
  errors: number;
  /** Total suite duration in milliseconds */
  duration: number;
  /** Individual test results */
  results: TestResult[];
  /** Timestamp when suite was run */
  timestamp?: string;
}

/**
 * JSON report format
 */
export interface JsonReport {
  /** Report version */
  version: string;
  /** Timestamp when report was generated */
  timestamp: string;
  /** Test suite name */
  suite?: string;
  /** Summary statistics */
  summary: {
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    errors: number;
    duration: number;
    passRate: number;
  };
  /** Individual test results */
  results: {
    id: string;
    name: string;
    status: string;
    duration: number;
    error?: string;
    diffs?: Array<{
      path: string;
      type: string;
      expected?: unknown;
      actual?: unknown;
      description: string;
    }>;
  }[];
}

/**
 * ANSI color codes for terminal output
 */
const colors = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',

  // Foreground colors
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  gray: '\x1b[90m',

  // Background colors
  bgRed: '\x1b[41m',
  bgGreen: '\x1b[42m',
  bgYellow: '\x1b[43m',
};

/**
 * Validation reporter for test results
 */
export class ValidationReporter {
  private useColors: boolean;

  constructor() {
    // Check for NO_COLOR or CI environment variables
    this.useColors = this.shouldUseColors();
  }

  /**
   * Determine if colors should be used
   */
  private shouldUseColors(): boolean {
    // Respect NO_COLOR environment variable
    if (process.env.NO_COLOR !== undefined) {
      return false;
    }

    // Check if running in CI (some CI environments support colors)
    if (process.env.CI !== undefined) {
      // GitHub Actions, GitLab CI support colors
      if (process.env.GITHUB_ACTIONS || process.env.GITLAB_CI) {
        return true;
      }
      // Default to no colors in CI
      return false;
    }

    // Check if stdout is a TTY
    return process.stdout.isTTY ?? false;
  }

  /**
   * Apply color to text if colors are enabled
   */
  private colorize(text: string, color: keyof typeof colors): string {
    if (!this.useColors) {
      return text;
    }
    return `${colors[color]}${text}${colors.reset}`;
  }

  /**
   * Format test suite results for human consumption
   *
   * @param suiteResult - Test suite result
   * @returns Formatted string for console output
   */
  formatHuman(suiteResult: TestSuiteResult): string {
    const lines: string[] = [];

    // Header
    lines.push('');
    lines.push(this.colorize('═'.repeat(80), 'cyan'));
    lines.push(this.colorize(`  Test Suite Results${suiteResult.name ? `: ${suiteResult.name}` : ''}`, 'bold'));
    lines.push(this.colorize('═'.repeat(80), 'cyan'));
    lines.push('');

    // Summary
    const passRate = suiteResult.total > 0
      ? ((suiteResult.passed / suiteResult.total) * 100).toFixed(1)
      : '0.0';

    lines.push(this.colorize('Summary:', 'bold'));
    lines.push(`  Total:    ${suiteResult.total} tests`);
    lines.push(`  ${this.colorize('✓ Passed:', 'green')}  ${suiteResult.passed}`);

    if (suiteResult.failed > 0) {
      lines.push(`  ${this.colorize('✗ Failed:', 'red')}  ${suiteResult.failed}`);
    }

    if (suiteResult.skipped > 0) {
      lines.push(`  ${this.colorize('○ Skipped:', 'yellow')} ${suiteResult.skipped}`);
    }

    if (suiteResult.errors > 0) {
      lines.push(`  ${this.colorize('⚠ Errors:', 'magenta')}  ${suiteResult.errors}`);
    }

    lines.push(`  Duration: ${this.formatDuration(suiteResult.duration)}`);

    const passRateColor = suiteResult.failed === 0 ? 'green' :
                         parseFloat(passRate) >= 80 ? 'yellow' : 'red';
    lines.push(`  Pass Rate: ${this.colorize(`${passRate}%`, passRateColor)}`);
    lines.push('');

    // Individual test results
    if (suiteResult.results.length > 0) {
      lines.push(this.colorize('Test Results:', 'bold'));
      lines.push('');

      for (const result of suiteResult.results) {
        lines.push(...this.formatTestResult(result));
      }
    }

    // Footer
    lines.push(this.colorize('═'.repeat(80), 'cyan'));
    lines.push('');

    return lines.join('\n');
  }

  /**
   * Format individual test result
   */
  private formatTestResult(result: TestResult): string[] {
    const lines: string[] = [];

    // Status icon and name
    let statusIcon: string;
    let statusColor: keyof typeof colors;

    switch (result.status) {
      case 'pass':
        statusIcon = '✓';
        statusColor = 'green';
        break;
      case 'fail':
        statusIcon = '✗';
        statusColor = 'red';
        break;
      case 'skip':
        statusIcon = '○';
        statusColor = 'yellow';
        break;
      case 'error':
        statusIcon = '⚠';
        statusColor = 'magenta';
        break;
    }

    const status = this.colorize(`${statusIcon} ${result.status.toUpperCase()}`, statusColor);
    const duration = this.colorize(`(${this.formatDuration(result.duration)})`, 'gray');

    lines.push(`${status} ${result.name} ${duration}`);

    // Show error message if present
    if (result.error) {
      lines.push(this.colorize(`  Error: ${result.error}`, 'red'));
    }

    // Show comparison diffs if present
    if (result.comparison && !result.comparison.matches && result.comparison.diffs.length > 0) {
      lines.push(this.colorize('  Differences:', 'yellow'));

      // Limit diff output to 100 lines
      const maxDiffs = 20;
      const diffsToShow = result.comparison.diffs.slice(0, maxDiffs);

      for (const diff of diffsToShow) {
        lines.push(...this.formatDiff(diff));
      }

      if (result.comparison.diffs.length > maxDiffs) {
        const remaining = result.comparison.diffs.length - maxDiffs;
        lines.push(this.colorize(`  ... and ${remaining} more differences`, 'gray'));
      }

      // Show full diff if available
      if (result.expected !== undefined && result.actual !== undefined) {
        lines.push('');
        lines.push(this.colorize('  Full comparison:', 'cyan'));
        lines.push(...this.formatFullDiff(result.expected, result.actual));
      }
    }

    lines.push('');
    return lines;
  }

  /**
   * Format a single diff
   */
  private formatDiff(diff: Diff): string[] {
    const lines: string[] = [];
    const indent = '    ';

    // Path
    lines.push(`${indent}${this.colorize(`Path: ${diff.path}`, 'cyan')}`);

    // Type
    const typeColor: keyof typeof colors =
      diff.type === 'missing' ? 'red' :
      diff.type === 'extra' ? 'yellow' :
      diff.type === 'type-mismatch' ? 'magenta' : 'red';

    lines.push(`${indent}Type: ${this.colorize(diff.type, typeColor)}`);

    // Expected vs Actual
    if (diff.expected !== undefined) {
      lines.push(`${indent}${this.colorize('-', 'red')} Expected: ${this.formatValue(diff.expected)}`);
    }
    if (diff.actual !== undefined) {
      lines.push(`${indent}${this.colorize('+', 'green')} Actual:   ${this.formatValue(diff.actual)}`);
    }

    return lines;
  }

  /**
   * Format full diff comparison
   */
  private formatFullDiff(expected: unknown, actual: unknown): string[] {
    const lines: string[] = [];
    const indent = '    ';

    try {
      const expectedJson = JSON.stringify(expected, null, 2);
      const actualJson = JSON.stringify(actual, null, 2);

      const expectedLines = expectedJson.split('\n');
      const actualLines = actualJson.split('\n');

      const maxLines = Math.max(expectedLines.length, actualLines.length);
      const displayLimit = 50; // Limit full diff to 50 lines

      for (let i = 0; i < Math.min(maxLines, displayLimit); i++) {
        const expectedLine = i < expectedLines.length ? expectedLines[i] : '';
        const actualLine = i < actualLines.length ? actualLines[i] : '';

        if (expectedLine === actualLine) {
          lines.push(`${indent}  ${this.colorize(expectedLine, 'gray')}`);
        } else {
          if (expectedLine) {
            lines.push(`${indent}${this.colorize('-', 'red')} ${expectedLine}`);
          }
          if (actualLine) {
            lines.push(`${indent}${this.colorize('+', 'green')} ${actualLine}`);
          }
        }
      }

      if (maxLines > displayLimit) {
        lines.push(`${indent}${this.colorize(`... (${maxLines - displayLimit} more lines)`, 'gray')}`);
      }
    } catch (error) {
      lines.push(`${indent}${this.colorize('Unable to format diff', 'red')}`);
    }

    return lines;
  }

  /**
   * Format a value for display
   */
  private formatValue(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `"${value}"`;
    if (typeof value === 'number' || typeof value === 'boolean') return String(value);
    if (Array.isArray(value)) return `[${value.length} items]`;
    if (typeof value === 'object') return '{...}';
    return String(value);
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
   * Format test suite results as JSON
   *
   * @param suiteResult - Test suite result
   * @returns JSON report object
   */
  formatJson(suiteResult: TestSuiteResult): JsonReport {
    const passRate = suiteResult.total > 0
      ? (suiteResult.passed / suiteResult.total) * 100
      : 0;

    return {
      version: '1.0',
      timestamp: suiteResult.timestamp ?? new Date().toISOString(),
      suite: suiteResult.name,
      summary: {
        total: suiteResult.total,
        passed: suiteResult.passed,
        failed: suiteResult.failed,
        skipped: suiteResult.skipped,
        errors: suiteResult.errors,
        duration: suiteResult.duration,
        passRate: parseFloat(passRate.toFixed(2)),
      },
      results: suiteResult.results.map((result) => ({
        id: result.id,
        name: result.name,
        status: result.status,
        duration: result.duration,
        error: result.error,
        diffs: result.comparison?.diffs.map((diff) => ({
          path: diff.path,
          type: diff.type,
          expected: diff.expected,
          actual: diff.actual,
          description: diff.description,
        })),
      })),
    };
  }

  /**
   * Write JSON report to file
   *
   * @param suiteResult - Test suite result
   * @param filepath - Output file path
   */
  async writeJsonReport(suiteResult: TestSuiteResult, filepath: string): Promise<void> {
    const report = this.formatJson(suiteResult);
    const json = JSON.stringify(report, null, 2);

    const fs = await import('fs/promises');
    await fs.writeFile(filepath, json, 'utf-8');
  }

  /**
   * Create a reporter instance
   */
  static create(): ValidationReporter {
    return new ValidationReporter();
  }
}

/**
 * Create a validation reporter instance
 */
export function createReporter(): ValidationReporter {
  return new ValidationReporter();
}
