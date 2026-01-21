/**
 * Response Comparator
 *
 * Deep comparison of API responses with configurable options for ignoring
 * dynamic fields, semantic array comparison, and detailed diff output.
 */

export interface ComparisonOptions {
  ignoreFields?: string[];
  ignoreArrayOrder?: boolean;
  ignoreWhitespace?: boolean;
  maxDepth?: number;
}

export interface Diff {
  path: string;
  type: 'missing' | 'extra' | 'type-mismatch' | 'value-mismatch';
  expected?: unknown;
  actual?: unknown;
}

export interface ComparisonResult {
  matches: boolean;
  diff: Diff[];
  ignoredFields: string[];
}

/**
 * Response comparator with configurable comparison rules
 */
export class ResponseComparator {
  private defaultOptions: Required<ComparisonOptions> = {
    ignoreFields: ['timestamp', 'uptime_secs', 'modifiedAt', 'createdAt'],
    ignoreArrayOrder: false,
    ignoreWhitespace: false,
    maxDepth: 50,
  };

  /**
   * Compare two objects with optional configuration
   */
  compare(
    actual: unknown,
    expected: unknown,
    options?: ComparisonOptions
  ): ComparisonResult {
    const opts = { ...this.defaultOptions, ...options };
    const diffs: Diff[] = [];
    const ignoredFields: string[] = [];

    this.compareValues(actual, expected, '', opts, diffs, ignoredFields, 0);

    return {
      matches: diffs.length === 0,
      diff: diffs,
      ignoredFields,
    };
  }

  /**
   * Recursively compare values
   */
  private compareValues(
    actual: unknown,
    expected: unknown,
    path: string,
    options: Required<ComparisonOptions>,
    diffs: Diff[],
    ignoredFields: string[],
    depth: number
  ): void {
    // Check depth limit to prevent infinite recursion
    if (depth > options.maxDepth) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected: '<max depth exceeded>',
        actual: '<max depth exceeded>',
      });
      return;
    }

    // Check if this field should be ignored
    if (this.shouldIgnoreField(path, options.ignoreFields)) {
      ignoredFields.push(path);
      return;
    }

    // Handle null/undefined
    if (actual === null && expected === null) return;
    if (actual === undefined && expected === undefined) return;

    if (actual === null || actual === undefined) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
      });
      return;
    }

    if (expected === null || expected === undefined) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
      });
      return;
    }

    // Type mismatch
    const actualType = this.getType(actual);
    const expectedType = this.getType(expected);

    if (actualType !== expectedType) {
      diffs.push({
        path,
        type: 'type-mismatch',
        expected: `${expectedType}: ${JSON.stringify(expected)}`,
        actual: `${actualType}: ${JSON.stringify(actual)}`,
      });
      return;
    }

    // Handle arrays
    if (Array.isArray(actual) && Array.isArray(expected)) {
      this.compareArrays(actual, expected, path, options, diffs, ignoredFields, depth);
      return;
    }

    // Handle objects
    if (actualType === 'object') {
      this.compareObjects(
        actual as Record<string, unknown>,
        expected as Record<string, unknown>,
        path,
        options,
        diffs,
        ignoredFields,
        depth
      );
      return;
    }

    // Handle strings with whitespace normalization
    if (actualType === 'string' && options.ignoreWhitespace) {
      const normalizedActual = (actual as string).trim().replace(/\s+/g, ' ');
      const normalizedExpected = (expected as string).trim().replace(/\s+/g, ' ');
      if (normalizedActual !== normalizedExpected) {
        diffs.push({
          path,
          type: 'value-mismatch',
          expected,
          actual,
        });
      }
      return;
    }

    // Primitive value comparison
    if (actual !== expected) {
      diffs.push({
        path,
        type: 'value-mismatch',
        expected,
        actual,
      });
    }
  }

  /**
   * Compare two arrays
   */
  private compareArrays(
    actual: unknown[],
    expected: unknown[],
    path: string,
    options: Required<ComparisonOptions>,
    diffs: Diff[],
    ignoredFields: string[],
    depth: number
  ): void {
    if (options.ignoreArrayOrder) {
      // Semantic comparison: check if all elements exist (ignoring order)
      if (actual.length !== expected.length) {
        diffs.push({
          path: `${path}.length`,
          type: 'value-mismatch',
          expected: expected.length,
          actual: actual.length,
        });
        return;
      }

      // Try to match each expected element with an actual element
      const unmatchedActual = [...actual];
      for (let i = 0; i < expected.length; i++) {
        const expectedItem = expected[i];
        const matchIndex = unmatchedActual.findIndex(actualItem => {
          const tempDiffs: Diff[] = [];
          this.compareValues(actualItem, expectedItem, '', options, tempDiffs, [], depth + 1);
          return tempDiffs.length === 0;
        });

        if (matchIndex === -1) {
          diffs.push({
            path: `${path}[${i}]`,
            type: 'missing',
            expected: expectedItem,
            actual: undefined,
          });
        } else {
          unmatchedActual.splice(matchIndex, 1);
        }
      }

      unmatchedActual.forEach((item, idx) => {
        diffs.push({
          path: `${path}[extra-${idx}]`,
          type: 'extra',
          expected: undefined,
          actual: item,
        });
      });
    } else {
      // Strict order comparison
      if (actual.length !== expected.length) {
        diffs.push({
          path: `${path}.length`,
          type: 'value-mismatch',
          expected: expected.length,
          actual: actual.length,
        });
      }

      const minLength = Math.min(actual.length, expected.length);
      for (let i = 0; i < minLength; i++) {
        this.compareValues(
          actual[i],
          expected[i],
          `${path}[${i}]`,
          options,
          diffs,
          ignoredFields,
          depth + 1
        );
      }
    }
  }

  /**
   * Compare two objects
   */
  private compareObjects(
    actual: Record<string, unknown>,
    expected: Record<string, unknown>,
    path: string,
    options: Required<ComparisonOptions>,
    diffs: Diff[],
    ignoredFields: string[],
    depth: number
  ): void {
    // Check for missing keys
    for (const key in expected) {
      if (!(key in actual)) {
        const fieldPath = path ? `${path}.${key}` : key;
        if (!this.shouldIgnoreField(fieldPath, options.ignoreFields)) {
          diffs.push({
            path: fieldPath,
            type: 'missing',
            expected: expected[key],
            actual: undefined,
          });
        } else {
          ignoredFields.push(fieldPath);
        }
      }
    }

    // Check for extra keys
    for (const key in actual) {
      if (!(key in expected)) {
        const fieldPath = path ? `${path}.${key}` : key;
        if (!this.shouldIgnoreField(fieldPath, options.ignoreFields)) {
          diffs.push({
            path: fieldPath,
            type: 'extra',
            expected: undefined,
            actual: actual[key],
          });
        } else {
          ignoredFields.push(fieldPath);
        }
      }
    }

    // Compare common keys
    for (const key in expected) {
      if (key in actual) {
        const fieldPath = path ? `${path}.${key}` : key;
        this.compareValues(
          actual[key],
          expected[key],
          fieldPath,
          options,
          diffs,
          ignoredFields,
          depth + 1
        );
      }
    }
  }

  /**
   * Check if a field path should be ignored
   */
  private shouldIgnoreField(path: string, ignoreFields: string[]): boolean {
    return ignoreFields.some(field => {
      // Exact match
      if (path === field) return true;
      // Field name at end of path (e.g., "uptime_secs" matches "status.uptime_secs")
      if (path.endsWith(`.${field}`)) return true;
      return false;
    });
  }

  /**
   * Get type of value (handling arrays separately from objects)
   */
  private getType(value: unknown): string {
    if (value === null) return 'null';
    if (Array.isArray(value)) return 'array';
    return typeof value;
  }
}

/**
 * Create a response comparator with default options
 */
export function createComparator(options?: ComparisonOptions): ResponseComparator {
  const comparator = new ResponseComparator();
  return comparator;
}
