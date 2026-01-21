/**
 * Test Utilities
 *
 * Shared utilities for test case definitions including validation helpers
 * and assertion functions.
 */

import { ApiClient, ApiClientError } from '../api-client/client';

export interface TestCase {
  id: string;
  name: string;
  endpoint: string;
  scenario: string;
  setup: (client: ApiClient) => Promise<any>;
  execute: (client: ApiClient, context?: any) => Promise<unknown>;
  assert: (actual: unknown, expected: unknown) => ValidationResult;
  cleanup?: (client: ApiClient, context?: any) => Promise<void>;
}

export interface ValidationResult {
  passed: boolean;
  message?: string;
  diff?: {
    path: string;
    expected: unknown;
    actual: unknown;
  }[];
}

/**
 * Compare objects while ignoring specified dynamic fields
 */
export function compareWithIgnoredFields(
  actual: any,
  expected: any,
  ignoredFields: string[] = ['timestamp', 'uptime_secs', 'modifiedAt', 'createdAt']
): ValidationResult {
  const diffs: Array<{ path: string; expected: unknown; actual: unknown }> = [];

  const compare = (a: any, e: any, path: string = ''): void => {
    if (ignoredFields.some(field => path.endsWith(field))) {
      return; // Skip ignored fields
    }

    if (typeof e !== typeof a) {
      diffs.push({ path, expected: e, actual: a });
      return;
    }

    if (e === null || a === null) {
      if (e !== a) {
        diffs.push({ path, expected: e, actual: a });
      }
      return;
    }

    if (typeof e === 'object' && !Array.isArray(e)) {
      for (const key in e) {
        const newPath = path ? `${path}.${key}` : key;
        if (!(key in a)) {
          diffs.push({ path: newPath, expected: e[key], actual: undefined });
        } else {
          compare(a[key], e[key], newPath);
        }
      }
      return;
    }

    if (Array.isArray(e)) {
      if (!Array.isArray(a) || e.length !== a.length) {
        diffs.push({ path, expected: e, actual: a });
        return;
      }
      e.forEach((item, idx) => {
        compare(a[idx], item, `${path}[${idx}]`);
      });
      return;
    }

    if (e !== a) {
      diffs.push({ path, expected: e, actual: a });
    }
  };

  compare(actual, expected);

  return {
    passed: diffs.length === 0,
    message: diffs.length > 0 ? `Found ${diffs.length} difference(s)` : 'All fields match',
    diff: diffs,
  };
}

/**
 * Assert that an error response matches expectations
 */
export function assertError(
  error: unknown,
  expectedStatus: number,
  expectedMessage?: string
): ValidationResult {
  if (!(error instanceof ApiClientError)) {
    return {
      passed: false,
      message: `Expected ApiClientError but got ${error?.constructor.name || 'unknown'}`,
    };
  }

  if (error.statusCode !== expectedStatus) {
    return {
      passed: false,
      message: `Expected status ${expectedStatus} but got ${error.statusCode}`,
      diff: [
        {
          path: 'statusCode',
          expected: expectedStatus,
          actual: error.statusCode,
        },
      ],
    };
  }

  if (expectedMessage && error.apiError?.message !== expectedMessage) {
    return {
      passed: false,
      message: `Error message mismatch`,
      diff: [
        {
          path: 'error.message',
          expected: expectedMessage,
          actual: error.apiError?.message,
        },
      ],
    };
  }

  return { passed: true, message: 'Error response matches expected' };
}

/**
 * Validate that a response has required fields
 */
export function assertRequiredFields(
  obj: any,
  fields: string[],
  objectName = 'object'
): ValidationResult {
  const missing = fields.filter(field => !(field in obj));
  if (missing.length > 0) {
    return {
      passed: false,
      message: `${objectName} missing required fields: ${missing.join(', ')}`,
    };
  }
  return { passed: true, message: `${objectName} has all required fields` };
}

/**
 * Validate that a field has the expected type
 */
export function assertFieldType(
  obj: any,
  field: string,
  expectedType: string
): ValidationResult {
  const actualType = typeof obj[field];
  if (actualType !== expectedType) {
    return {
      passed: false,
      message: `Field '${field}' should be ${expectedType} but is ${actualType}`,
    };
  }
  return { passed: true, message: `Field '${field}' has correct type` };
}
