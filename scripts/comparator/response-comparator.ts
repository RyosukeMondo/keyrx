/**
 * Response comparator for API test validation
 *
 * Provides deep equality checking with detailed diff output for test validation.
 * Handles complex scenarios including:
 * - Nested objects and arrays
 * - Configurable field ignoring (timestamps, IDs, etc.)
 * - Semantic array comparison (order-independent)
 * - Whitespace normalization
 * - Circular reference detection
 */

/**
 * Comparison options
 */
export interface ComparisonOptions {
  /** Fields to ignore during comparison (e.g., ['timestamp', 'id']) */
  ignoreFields?: string[];
  /** Whether to ignore array order (semantic comparison) */
  ignoreArrayOrder?: boolean;
  /** Whether to ignore whitespace differences in strings */
  ignoreWhitespace?: boolean;
  /** Maximum depth for nested object comparison (prevents infinite recursion) */
  maxDepth?: number;
}

/**
 * Detailed difference information
 */
export interface Diff {
  /** Path to the differing field (e.g., 'data.profiles[0].name') */
  path: string;
  /** Type of difference */
  type: 'missing' | 'extra' | 'type-mismatch' | 'value-mismatch';
  /** Expected value */
  expected?: unknown;
  /** Actual value */
  actual?: unknown;
  /** Human-readable description */
  description: string;
}

/**
 * Comparison result
 */
export interface ComparisonResult {
  /** Whether the values match */
  matches: boolean;
  /** List of differences found (empty if matches = true) */
  diffs: Diff[];
  /** Fields that were ignored during comparison */
  ignoredFields: string[];
}

/**
 * Response comparator for API test validation
 */
export class ResponseComparator {
  private readonly defaultOptions: Required<ComparisonOptions> = {
    ignoreFields: [],
    ignoreArrayOrder: false,
    ignoreWhitespace: false,
    maxDepth: 50,
  };

  /**
   * Compare two values and return detailed diff
   *
   * @param actual - Actual value from API response
   * @param expected - Expected value from test database
   * @param options - Comparison options
   * @returns Comparison result with diff details
   */
  compare(
    actual: unknown,
    expected: unknown,
    options?: ComparisonOptions
  ): ComparisonResult {
    const opts = { ...this.defaultOptions, ...options };
    const diffs: Diff[] = [];
    const seen = new WeakSet();

    this.compareValues(actual, expected, '', diffs, opts, seen, 0);

    return {
      matches: diffs.length === 0,
      diffs,
      ignoredFields: opts.ignoreFields,
    };
  }

  /**
   * Recursively compare two values
   */
  private compareValues(
    actual: unknown,
    expected: unknown,
    path: string,
    diffs: Diff[],
    options: Required<ComparisonOptions>,
    seen: WeakSet<object>,
    depth: number
  ): void {
    // Check depth limit to prevent stack overflow
    if (depth > options.maxDepth) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
        description: `Maximum comparison depth (${options.maxDepth}) exceeded at ${path}`,
      });
      return;
    }

    // Handle null and undefined
    if (actual === null && expected === null) return;
    if (actual === undefined && expected === undefined) return;

    if (actual === null || actual === undefined) {
      diffs.push({
        path,
        type: 'missing',
        expected,
        actual,
        description: `Expected ${this.formatValue(expected)} but got ${this.formatValue(actual)}`,
      });
      return;
    }

    if (expected === null || expected === undefined) {
      diffs.push({
        path,
        type: 'extra',
        expected,
        actual,
        description: `Expected ${this.formatValue(expected)} but got ${this.formatValue(actual)}`,
      });
      return;
    }

    // Get types
    const actualType = this.getType(actual);
    const expectedType = this.getType(expected);

    // Type mismatch
    if (actualType !== expectedType) {
      diffs.push({
        path,
        type: 'type-mismatch',
        expected,
        actual,
        description: `Type mismatch: expected ${expectedType} but got ${actualType}`,
      });
      return;
    }

    // Compare based on type
    switch (actualType) {
      case 'string':
        this.compareStrings(actual as string, expected as string, path, diffs, options);
        break;
      case 'number':
      case 'boolean':
        this.comparePrimitives(actual, expected, path, diffs);
        break;
      case 'array':
        this.compareArrays(
          actual as unknown[],
          expected as unknown[],
          path,
          diffs,
          options,
          seen,
          depth
        );
        break;
      case 'object':
        this.compareObjects(
          actual as Record<string, unknown>,
          expected as Record<string, unknown>,
          path,
          diffs,
          options,
          seen,
          depth
        );
        break;
      case 'date':
        this.compareDates(actual as Date, expected as Date, path, diffs);
        break;
      default:
        // For other types (functions, symbols, etc.), use strict equality
        if (actual !== expected) {
          diffs.push({
            path,
            type: 'value-mismatch',
            expected,
            actual,
            description: `Values differ: expected ${this.formatValue(expected)} but got ${this.formatValue(actual)}`,
          });
        }
    }
  }

  /**
   * Compare primitive values (number, boolean)
   */
  private comparePrimitives(
    actual: unknown,
    expected: unknown,
    path: string,
    diffs: Diff[]
  ): void {
    if (actual !== expected) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
        description: `Expected ${this.formatValue(expected)} but got ${this.formatValue(actual)}`,
      });
    }
  }

  /**
   * Compare string values with optional whitespace normalization
   */
  private compareStrings(
    actual: string,
    expected: string,
    path: string,
    diffs: Diff[],
    options: Required<ComparisonOptions>
  ): void {
    const actualValue = options.ignoreWhitespace ? actual.trim() : actual;
    const expectedValue = options.ignoreWhitespace ? expected.trim() : expected;

    if (actualValue !== expectedValue) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected: expectedValue,
        actual: actualValue,
        description: `String mismatch at ${path}`,
      });
    }
  }

  /**
   * Compare Date objects
   */
  private compareDates(actual: Date, expected: Date, path: string, diffs: Diff[]): void {
    if (actual.getTime() !== expected.getTime()) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected: expected.toISOString(),
        actual: actual.toISOString(),
        description: `Date mismatch: expected ${expected.toISOString()} but got ${actual.toISOString()}`,
      });
    }
  }

  /**
   * Compare arrays
   */
  private compareArrays(
    actual: unknown[],
    expected: unknown[],
    path: string,
    diffs: Diff[],
    options: Required<ComparisonOptions>,
    seen: WeakSet<object>,
    depth: number
  ): void {
    // Check for circular references
    if (seen.has(actual)) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
        description: `Circular reference detected at ${path}`,
      });
      return;
    }
    seen.add(actual);

    if (options.ignoreArrayOrder) {
      // Semantic comparison: check if all expected items exist in actual (order-independent)
      if (actual.length !== expected.length) {
        diffs.push({
          path,
          type: 'value-mismatch',
          expected: expected.length,
          actual: actual.length,
          description: `Array length mismatch: expected ${expected.length} items but got ${actual.length}`,
        });
        return;
      }

      // For each expected item, find a matching actual item
      const usedIndices = new Set<number>();
      for (let i = 0; i < expected.length; i++) {
        let found = false;
        for (let j = 0; j < actual.length; j++) {
          if (usedIndices.has(j)) continue;

          const tempDiffs: Diff[] = [];
          this.compareValues(
            actual[j],
            expected[i],
            `${path}[${i}]`,
            tempDiffs,
            options,
            new WeakSet(seen),
            depth + 1
          );

          if (tempDiffs.length === 0) {
            usedIndices.add(j);
            found = true;
            break;
          }
        }

        if (!found) {
          diffs.push({
            path: `${path}[${i}]`,
            type: 'missing',
            expected: expected[i],
            actual: undefined,
            description: `Expected item not found in array (order-independent comparison)`,
          });
        }
      }
    } else {
      // Ordered comparison
      if (actual.length !== expected.length) {
        diffs.push({
          path,
          type: 'value-mismatch',
          expected: expected.length,
          actual: actual.length,
          description: `Array length mismatch: expected ${expected.length} items but got ${actual.length}`,
        });
        // Continue comparing elements up to min length
        const minLength = Math.min(actual.length, expected.length);
        for (let i = 0; i < minLength; i++) {
          this.compareValues(
            actual[i],
            expected[i],
            `${path}[${i}]`,
            diffs,
            options,
            seen,
            depth + 1
          );
        }
        return;
      }

      // Compare each element in order
      for (let i = 0; i < expected.length; i++) {
        this.compareValues(
          actual[i],
          expected[i],
          `${path}[${i}]`,
          diffs,
          options,
          seen,
          depth + 1
        );
      }
    }
  }

  /**
   * Compare objects
   */
  private compareObjects(
    actual: Record<string, unknown>,
    expected: Record<string, unknown>,
    path: string,
    diffs: Diff[],
    options: Required<ComparisonOptions>,
    seen: WeakSet<object>,
    depth: number
  ): void {
    // Check for circular references
    if (seen.has(actual)) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
        description: `Circular reference detected at ${path}`,
      });
      return;
    }
    seen.add(actual);

    // Get all keys
    const actualKeys = new Set(Object.keys(actual));
    const expectedKeys = new Set(Object.keys(expected));
    const allKeys = new Set([...actualKeys, ...expectedKeys]);

    for (const key of allKeys) {
      const fieldPath = path ? `${path}.${key}` : key;

      // Check if field should be ignored
      if (this.shouldIgnoreField(fieldPath, options.ignoreFields)) {
        continue;
      }

      // Check if key exists in both
      if (!actualKeys.has(key)) {
        diffs.push({
          path: fieldPath,
          type: 'missing',
          expected: expected[key],
          actual: undefined,
          description: `Missing field: ${key}`,
        });
        continue;
      }

      if (!expectedKeys.has(key)) {
        diffs.push({
          path: fieldPath,
          type: 'extra',
          expected: undefined,
          actual: actual[key],
          description: `Extra field: ${key}`,
        });
        continue;
      }

      // Compare values
      this.compareValues(
        actual[key],
        expected[key],
        fieldPath,
        diffs,
        options,
        seen,
        depth + 1
      );
    }
  }

  /**
   * Check if a field should be ignored based on ignore patterns
   */
  private shouldIgnoreField(fieldPath: string, ignoreFields: string[]): boolean {
    for (const pattern of ignoreFields) {
      // Support both exact match and suffix match
      // e.g., 'timestamp' matches 'data.timestamp' and 'timestamp'
      // e.g., '*.id' matches 'user.id', 'profile.id', etc.
      if (fieldPath === pattern) return true;
      if (fieldPath.endsWith(`.${pattern}`)) return true;
      if (pattern.startsWith('*.') && fieldPath.endsWith(pattern.slice(1))) return true;
    }
    return false;
  }

  /**
   * Get the type of a value
   */
  private getType(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (Array.isArray(value)) return 'array';
    if (value instanceof Date) return 'date';
    return typeof value;
  }

  /**
   * Format a value for display in error messages
   */
  private formatValue(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `"${value}"`;
    if (typeof value === 'number' || typeof value === 'boolean') return String(value);
    if (Array.isArray(value)) return `array[${value.length}]`;
    if (value instanceof Date) return value.toISOString();
    if (typeof value === 'object') return 'object';
    return String(value);
  }

  /**
   * Generate a text diff for two JSON objects (for display purposes)
   * Simple line-by-line comparison without external dependencies
   */
  static generateTextDiff(expected: unknown, actual: unknown): string {
    const expectedJson = JSON.stringify(expected, null, 2);
    const actualJson = JSON.stringify(actual, null, 2);

    const expectedLines = expectedJson.split('\n');
    const actualLines = actualJson.split('\n');

    const maxLines = Math.max(expectedLines.length, actualLines.length);
    const diffLines: string[] = ['--- Expected', '+++ Actual', ''];

    for (let i = 0; i < maxLines; i++) {
      const expectedLine = i < expectedLines.length ? expectedLines[i] : undefined;
      const actualLine = i < actualLines.length ? actualLines[i] : undefined;

      if (expectedLine === actualLine) {
        diffLines.push(`  ${expectedLine ?? ''}`);
      } else {
        if (expectedLine !== undefined) {
          diffLines.push(`- ${expectedLine}`);
        }
        if (actualLine !== undefined) {
          diffLines.push(`+ ${actualLine}`);
        }
      }
    }

    return diffLines.join('\n');
  }
}

/**
 * Create a response comparator instance
 */
export function createComparator(): ResponseComparator {
  return new ResponseComparator();
}
