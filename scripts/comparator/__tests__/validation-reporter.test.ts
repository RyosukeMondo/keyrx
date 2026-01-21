/**
 * Tests for ValidationReporter
 */

import { ValidationReporter, TestResult, TestSuiteResult } from '../validation-reporter.js';
import { ComparisonResult } from '../response-comparator.js';

describe('ValidationReporter', () => {
  let reporter: ValidationReporter;

  beforeEach(() => {
    // Disable colors for testing
    process.env.NO_COLOR = '1';
    reporter = new ValidationReporter();
  });

  afterEach(() => {
    delete process.env.NO_COLOR;
  });

  describe('formatHuman', () => {
    test('should format passing test suite', () => {
      const suiteResult: TestSuiteResult = {
        name: 'API Tests',
        total: 3,
        passed: 3,
        failed: 0,
        skipped: 0,
        errors: 0,
        duration: 1500,
        results: [
          {
            id: 'test-1',
            name: 'GET /api/status',
            status: 'pass',
            duration: 500,
          },
          {
            id: 'test-2',
            name: 'GET /api/devices',
            status: 'pass',
            duration: 500,
          },
          {
            id: 'test-3',
            name: 'GET /api/profiles',
            status: 'pass',
            duration: 500,
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      expect(output).toContain('Test Suite Results: API Tests');
      expect(output).toContain('Total:    3 tests');
      expect(output).toContain('✓ Passed:  3');
      expect(output).toContain('Pass Rate: 100.0%');
    });

    test('should format failing test suite', () => {
      const comparison: ComparisonResult = {
        matches: false,
        diffs: [
          {
            path: 'data.status',
            type: 'value-mismatch',
            expected: 'active',
            actual: 'inactive',
            description: 'Expected "active" but got "inactive"',
          },
        ],
        ignoredFields: [],
      };

      const suiteResult: TestSuiteResult = {
        name: 'API Tests',
        total: 2,
        passed: 1,
        failed: 1,
        skipped: 0,
        errors: 0,
        duration: 1000,
        results: [
          {
            id: 'test-1',
            name: 'GET /api/status',
            status: 'pass',
            duration: 500,
          },
          {
            id: 'test-2',
            name: 'GET /api/devices',
            status: 'fail',
            duration: 500,
            comparison,
            expected: { data: { status: 'active' } },
            actual: { data: { status: 'inactive' } },
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      expect(output).toContain('Test Suite Results: API Tests');
      expect(output).toContain('Total:    2 tests');
      expect(output).toContain('✓ Passed:  1');
      expect(output).toContain('✗ Failed:  1');
      expect(output).toContain('Pass Rate: 50.0%');
      expect(output).toContain('GET /api/devices');
      expect(output).toContain('Differences:');
      expect(output).toContain('data.status');
    });

    test('should format test with error', () => {
      const suiteResult: TestSuiteResult = {
        total: 1,
        passed: 0,
        failed: 0,
        skipped: 0,
        errors: 1,
        duration: 500,
        results: [
          {
            id: 'test-1',
            name: 'GET /api/status',
            status: 'error',
            duration: 500,
            error: 'Network timeout',
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      expect(output).toContain('⚠ Errors:  1');
      expect(output).toContain('GET /api/status');
      expect(output).toContain('Error: Network timeout');
    });

    test('should format test with skipped status', () => {
      const suiteResult: TestSuiteResult = {
        total: 1,
        passed: 0,
        failed: 0,
        skipped: 1,
        errors: 0,
        duration: 0,
        results: [
          {
            id: 'test-1',
            name: 'GET /api/status',
            status: 'skip',
            duration: 0,
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      expect(output).toContain('○ Skipped: 1');
      expect(output).toContain('GET /api/status');
    });

    test('should handle empty test suite', () => {
      const suiteResult: TestSuiteResult = {
        total: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        errors: 0,
        duration: 0,
        results: [],
      };

      const output = reporter.formatHuman(suiteResult);

      expect(output).toContain('Test Suite Results');
      expect(output).toContain('Total:    0 tests');
      expect(output).toContain('Pass Rate: 0.0%');
    });
  });

  describe('formatJson', () => {
    test('should format test suite as JSON', () => {
      const suiteResult: TestSuiteResult = {
        name: 'API Tests',
        total: 2,
        passed: 1,
        failed: 1,
        skipped: 0,
        errors: 0,
        duration: 1000,
        timestamp: '2024-01-01T00:00:00.000Z',
        results: [
          {
            id: 'test-1',
            name: 'GET /api/status',
            status: 'pass',
            duration: 500,
          },
          {
            id: 'test-2',
            name: 'GET /api/devices',
            status: 'fail',
            duration: 500,
            comparison: {
              matches: false,
              diffs: [
                {
                  path: 'data.count',
                  type: 'value-mismatch',
                  expected: 5,
                  actual: 3,
                  description: 'Expected 5 but got 3',
                },
              ],
              ignoredFields: [],
            },
          },
        ],
      };

      const json = reporter.formatJson(suiteResult);

      expect(json.version).toBe('1.0');
      expect(json.timestamp).toBe('2024-01-01T00:00:00.000Z');
      expect(json.suite).toBe('API Tests');
      expect(json.summary.total).toBe(2);
      expect(json.summary.passed).toBe(1);
      expect(json.summary.failed).toBe(1);
      expect(json.summary.passRate).toBe(50);
      expect(json.results).toHaveLength(2);
      expect(json.results[0].id).toBe('test-1');
      expect(json.results[1].diffs).toBeDefined();
      expect(json.results[1].diffs).toHaveLength(1);
    });

    test('should include diff details in JSON', () => {
      const suiteResult: TestSuiteResult = {
        total: 1,
        passed: 0,
        failed: 1,
        skipped: 0,
        errors: 0,
        duration: 500,
        results: [
          {
            id: 'test-1',
            name: 'Test',
            status: 'fail',
            duration: 500,
            comparison: {
              matches: false,
              diffs: [
                {
                  path: 'user.name',
                  type: 'value-mismatch',
                  expected: 'Alice',
                  actual: 'Bob',
                  description: 'Expected "Alice" but got "Bob"',
                },
                {
                  path: 'user.age',
                  type: 'missing',
                  expected: 30,
                  actual: undefined,
                  description: 'Missing field: age',
                },
              ],
              ignoredFields: [],
            },
          },
        ],
      };

      const json = reporter.formatJson(suiteResult);

      expect(json.results[0].diffs).toHaveLength(2);
      expect(json.results[0].diffs?.[0].path).toBe('user.name');
      expect(json.results[0].diffs?.[0].type).toBe('value-mismatch');
      expect(json.results[0].diffs?.[1].path).toBe('user.age');
      expect(json.results[0].diffs?.[1].type).toBe('missing');
    });
  });

  describe('Duration formatting', () => {
    test('should format durations correctly', () => {
      const cases = [
        { duration: 50, expected: '50ms' },
        { duration: 1500, expected: '1.50s' },
        { duration: 65000, expected: '1m 5s' },
      ];

      for (const { duration, expected } of cases) {
        const suiteResult: TestSuiteResult = {
          total: 1,
          passed: 1,
          failed: 0,
          skipped: 0,
          errors: 0,
          duration,
          results: [
            {
              id: 'test-1',
              name: 'Test',
              status: 'pass',
              duration,
            },
          ],
        };

        const output = reporter.formatHuman(suiteResult);
        expect(output).toContain(expected);
      }
    });
  });

  describe('Color handling', () => {
    test('should respect NO_COLOR environment variable', () => {
      process.env.NO_COLOR = '1';
      const reporter = new ValidationReporter();

      const suiteResult: TestSuiteResult = {
        total: 1,
        passed: 1,
        failed: 0,
        skipped: 0,
        errors: 0,
        duration: 500,
        results: [
          {
            id: 'test-1',
            name: 'Test',
            status: 'pass',
            duration: 500,
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      // Output should not contain ANSI color codes
      expect(output).not.toContain('\x1b[');
    });

    test('should disable colors in CI by default', () => {
      delete process.env.NO_COLOR;
      process.env.CI = 'true';

      const reporter = new ValidationReporter();

      const suiteResult: TestSuiteResult = {
        total: 1,
        passed: 1,
        failed: 0,
        skipped: 0,
        errors: 0,
        duration: 500,
        results: [
          {
            id: 'test-1',
            name: 'Test',
            status: 'pass',
            duration: 500,
          },
        ],
      };

      const output = reporter.formatHuman(suiteResult);

      // Output should not contain ANSI color codes in CI
      expect(output).not.toContain('\x1b[');

      delete process.env.CI;
    });
  });
});
