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
 * Key definition schema for KLE JSON
 */
const KeySchema = z.object({
  id: z.string(),
  label: z.string(),
  position: z.object({
    x: z.number(),
    y: z.number(),
  }),
  size: z.object({
    width: z.number(),
    height: z.number(),
  }),
});

/**
 * Layout response schema
 */
const LayoutSchema = z.object({
  id: z.string(),
  name: z.string(),
  layout_type: z.string(),
  keys: z.array(KeySchema),
});

type Layout = z.infer<typeof LayoutSchema>;

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

      // Get the specific layout
      const response = await client.customRequest(
        'GET',
        `/api/layouts/${layoutName}`,
        LayoutSchema
      );

      return {
        status: response.status,
        data: response.data,
        layoutName, // Store for assertion
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as Layout & { layoutName: string };

      // Validate structure
      const hasRequiredFields =
        typeof actualData.id === 'string' &&
        typeof actualData.name === 'string' &&
        typeof actualData.layout_type === 'string' &&
        Array.isArray(actualData.keys);

      if (!hasRequiredFields) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Missing required layout fields (id, name, layout_type, keys)',
        };
      }

      // Validate that keys array has at least one key
      if (actualData.keys.length === 0) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Layout must have at least one key',
        };
      }

      // Validate key structure (check first key)
      const firstKey = actualData.keys[0];
      const hasValidKeyStructure =
        typeof firstKey.id === 'string' &&
        typeof firstKey.label === 'string' &&
        typeof firstKey.position.x === 'number' &&
        typeof firstKey.position.y === 'number' &&
        typeof firstKey.size.width === 'number' &&
        typeof firstKey.size.height === 'number';

      if (!hasValidKeyStructure) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Invalid key structure in layout',
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
