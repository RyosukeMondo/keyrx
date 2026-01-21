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
 * Event log response schema
 */
const EventLogSchema = z.object({
  count: z.number(),
  events: z.array(z.any()), // Event structure varies
});

type EventLog = z.infer<typeof EventLogSchema>;

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

  // =================================================================
  // Event Log Tests
  // =================================================================
  {
    id: 'metrics-002',
    name: 'GET /api/metrics/events - Get event log with default limit',
    endpoint: '/api/metrics/events',
    scenario: 'default',
    category: 'metrics',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/metrics/events',
        EventLogSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as EventLog;

      // Validate structure
      const hasRequiredFields =
        typeof actualData.count === 'number' &&
        Array.isArray(actualData.events);

      if (!hasRequiredFields) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Missing required event log fields (count, events)',
        };
      }

      // Validate count matches array length
      if (actualData.count !== actualData.events.length) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Event count mismatch: count=${actualData.count}, array length=${actualData.events.length}`,
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
    id: 'metrics-002b',
    name: 'GET /api/metrics/events?count=10 - Get event log with custom limit',
    endpoint: '/api/metrics/events',
    scenario: 'with_limit',
    category: 'metrics',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/metrics/events?count=10',
        EventLogSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as EventLog;

      // Validate structure
      const hasRequiredFields =
        typeof actualData.count === 'number' &&
        Array.isArray(actualData.events);

      if (!hasRequiredFields) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Missing required event log fields',
        };
      }

      // Validate count matches array length
      if (actualData.count !== actualData.events.length) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Event count mismatch: count=${actualData.count}, array length=${actualData.events.length}`,
        };
      }

      // Validate returned count is <= requested limit (10)
      if (actualData.count > 10) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected at most 10 events, got ${actualData.count}`,
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

  // =================================================================
  // Event Log Clear Tests
  // =================================================================
  {
    id: 'metrics-003',
    name: 'DELETE /api/metrics/events - Clear event log (not implemented)',
    endpoint: '/api/metrics/events',
    scenario: 'not_implemented',
    category: 'metrics',
    priority: 3,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'DELETE',
        '/api/metrics/events',
        z.object({
          success: z.boolean(),
          error: z.string().optional(),
        })
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { success: boolean; error?: string };

      // This endpoint is not yet implemented in the daemon
      // It should return success=false with an error message
      if (actualData.success === true) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected success=false for unimplemented endpoint',
        };
      }

      if (!actualData.error || !actualData.error.includes('not yet implemented')) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected error message indicating not implemented',
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
