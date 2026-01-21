/**
 * Layouts API Test Cases
 *
 * Tests for keyboard layout endpoints:
 * - GET /api/layouts/:name - Get specific layout KLE JSON
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase, TestResult } from './api-tests.js';
import type { ScenarioDefinition } from './types.js';
import { z } from 'zod';

/**
 * No-op setup function for tests that don't need preparation
 */
const noOpSetup = async (): Promise<void> => {
  // No setup needed
};

/**
 * No-op cleanup function for tests that don't modify state
 */
const noOpCleanup = async (): Promise<void> => {
  // No cleanup needed
};

/**
 * KLE JSON schema - raw array format
 * KLE JSON is an array of arrays/objects representing keyboard rows
 */
const KleJsonSchema = z.array(z.any());

type KleJson = z.infer<typeof KleJsonSchema>;

/**
 * Error response schema for not found
 */
const ErrorSchema = z.object({
  error: z.string(),
  code: z.string().optional(),
});

/**
 * Layouts test cases
 */
export const layoutsTestCases: TestCase[] = [
  // =================================================================
  // GET /api/layouts/:name - Get specific layout
  // =================================================================
  {
    id: 'layouts-002',
    name: 'GET /api/layouts/:name - Get layout details',
    endpoint: '/api/layouts/:name',
    scenario: 'get_layout',
    category: 'layouts',
    priority: 1,
    setup: async () => {
      // No setup needed - we assume at least one layout exists from layouts-001 test
    },
    execute: async (client) => {
      // First, get the list of layouts to find a valid layout name
      const listResponse = await client.customRequest(
        'GET',
        '/api/layouts',
        z.object({
          layouts: z.array(z.string()),
        })
      );

      if (!listResponse.data.layouts || listResponse.data.layouts.length === 0) {
        throw new Error('No layouts available for testing');
      }

      // Use the first available layout
      const layoutName = listResponse.data.layouts[0];

      // Get the specific layout - returns raw KLE JSON array
      const response = await client.customRequest(
        'GET',
        `/api/layouts/${layoutName}`,
        KleJsonSchema
      );

      return {
        status: response.status,
        data: response.data,
        layoutName, // Store for assertion
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { status: number; data: any; layoutName: string };

      // Validate that data is an array (KLE JSON format)
      if (!Array.isArray(actualData.data)) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected KLE JSON array format',
        };
      }

      // Validate that array is not empty
      if (actualData.data.length === 0) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'KLE JSON array must have at least one row',
        };
      }

      // Validate that first element is an array or object (typical KLE format)
      const firstRow = actualData.data[0];
      if (!Array.isArray(firstRow) && typeof firstRow !== 'object') {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Invalid KLE JSON format - rows must be arrays or objects',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'layouts-002b',
    name: 'GET /api/layouts/:name - Not found layout',
    endpoint: '/api/layouts/:name',
    scenario: 'not_found',
    category: 'layouts',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const layoutName = 'nonexistent-layout-12345';

      try {
        const response = await client.customRequest(
          'GET',
          `/api/layouts/${layoutName}`,
          ErrorSchema
        );

        return {
          status: response.status,
          data: response.data,
        };
      } catch (error: any) {
        // Expected to fail with 404
        if (error.response?.status === 404) {
          return {
            status: 404,
            data: error.response.data,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const result = actual as { status: number; data: any };

      // Should return 404 status
      if (result.status !== 404) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 404 status, got ${result.status}`,
        };
      }

      // Should have error message
      if (!result.data.error || typeof result.data.error !== 'string') {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected error message in response',
        };
      }

      // Error message should mention the layout not found
      if (!result.data.error.toLowerCase().includes('not found')) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Error message should indicate layout not found',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },
];
