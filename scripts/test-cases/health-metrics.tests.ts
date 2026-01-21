/**
 * Health & Metrics API Test Cases
 *
 * Tests for daemon health, metrics, and state endpoints:
 * - GET /api/daemon/state - Full daemon state with modifiers/locks/layers
 * - GET /api/metrics/events - Event log retrieval with pagination
 * - DELETE /api/metrics/events - Event log clearing
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
 * Daemon state response schema
 */
const DaemonStateSchema = z.object({
  active_layer: z.string().nullable(),
  modifiers: z.array(z.string()),
  locks: z.array(z.string()),
  raw_state: z.array(z.boolean()),
  active_modifier_count: z.number(),
  active_lock_count: z.number(),
});

type DaemonState = z.infer<typeof DaemonStateSchema>;

/**
 * Health & Metrics test cases
 */
export const healthMetricsTestCases: TestCase[] = [
  // =================================================================
  // Daemon State Tests
  // =================================================================
  {
    id: 'health-007',
    name: 'GET /api/daemon/state - Get full daemon state',
    endpoint: '/api/daemon/state',
    scenario: 'default',
    category: 'health',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/daemon/state',
        DaemonStateSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as DaemonState;

      // Validate structure
      const hasRequiredFields =
        typeof actualData.active_modifier_count === 'number' &&
        typeof actualData.active_lock_count === 'number' &&
        Array.isArray(actualData.modifiers) &&
        Array.isArray(actualData.locks) &&
        Array.isArray(actualData.raw_state);

      if (!hasRequiredFields) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Missing required daemon state fields',
        };
      }

      // Validate raw_state has 255 bits
      if (actualData.raw_state.length !== 255) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected raw_state to have 255 bits, got ${actualData.raw_state.length}`,
        };
      }

      // Validate counts match array lengths
      if (actualData.active_modifier_count !== actualData.modifiers.length) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Modifier count mismatch: count=${actualData.active_modifier_count}, array length=${actualData.modifiers.length}`,
        };
      }

      if (actualData.active_lock_count !== actualData.locks.length) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Lock count mismatch: count=${actualData.active_lock_count}, array length=${actualData.locks.length}`,
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
