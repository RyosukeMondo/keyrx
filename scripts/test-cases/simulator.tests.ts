/**
 * Simulator API Test Cases
 *
 * Tests for event simulation endpoints:
 * - POST /api/simulator/events - Simulate keyboard events (scenarios or custom)
 * - POST /api/simulator/reset - Reset simulator state (ephemeral, no-op)
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
 * Simulate events response schema
 */
const SimulateEventsSchema = z.object({
  success: z.boolean(),
  event_count: z.number(),
  outputs: z.array(z.string()),
  duration_us: z.number(),
});

type SimulateEventsResponse = z.infer<typeof SimulateEventsSchema>;

/**
 * Reset simulator response schema
 */
const ResetSimulatorSchema = z.object({
  success: z.boolean(),
  message: z.string(),
});

type ResetSimulatorResponse = z.infer<typeof ResetSimulatorSchema>;

/**
 * Simulator test cases
 */
export const simulatorTestCases: TestCase[] = [
  // =================================================================
  // Simulate Events Tests
  // =================================================================
  {
    id: 'simulator-001',
    name: 'POST /api/simulator/events - Simulate single key press/release',
    endpoint: '/api/simulator/events',
    scenario: 'single_key',
    category: 'simulator',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/simulator/events',
        SimulateEventsSchema,
        {
          events: [
            {
              device_id: null,
              timestamp_us: 0,
              key: 'A',
              event_type: 'press',
            },
            {
              device_id: null,
              timestamp_us: 50000,
              key: 'A',
              event_type: 'release',
            },
          ],
        }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as SimulateEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Simulation did not succeed',
        };
      }

      if (actualData.event_count !== 2) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 2 events, got ${actualData.event_count}`,
        };
      }

      if (!Array.isArray(actualData.outputs) || actualData.outputs.length !== 2) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 2 outputs, got ${actualData.outputs?.length || 0}`,
        };
      }

      if (typeof actualData.duration_us !== 'number') {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'duration_us must be a number',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      event_count: 2,
      outputs: ['Press A at 0μs', 'Release A at 50000μs'],
      duration_us: 50000,
    },
  },

  {
    id: 'simulator-001b',
    name: 'POST /api/simulator/events - Simulate key sequence',
    endpoint: '/api/simulator/events',
    scenario: 'key_sequence',
    category: 'simulator',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/simulator/events',
        SimulateEventsSchema,
        {
          events: [
            { device_id: null, timestamp_us: 0, key: 'Shift', event_type: 'press' },
            { device_id: null, timestamp_us: 10000, key: 'A', event_type: 'press' },
            { device_id: null, timestamp_us: 60000, key: 'A', event_type: 'release' },
            { device_id: null, timestamp_us: 70000, key: 'Shift', event_type: 'release' },
          ],
        }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as SimulateEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Simulation did not succeed',
        };
      }

      if (actualData.event_count !== 4) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 4 events, got ${actualData.event_count}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      event_count: 4,
      outputs: [
        'Press Shift at 0μs',
        'Press A at 10000μs',
        'Release A at 60000μs',
        'Release Shift at 70000μs',
      ],
      duration_us: 70000,
    },
  },

  {
    id: 'simulator-001c',
    name: 'POST /api/simulator/events - Use built-in scenario',
    endpoint: '/api/simulator/events',
    scenario: 'builtin_scenario',
    category: 'simulator',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/simulator/events',
        SimulateEventsSchema,
        {
          scenario: 'tap-hold-under-threshold',
        }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as SimulateEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Simulation did not succeed',
        };
      }

      if (actualData.event_count < 1) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected at least 1 event, got ${actualData.event_count}`,
        };
      }

      if (!Array.isArray(actualData.outputs)) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'outputs must be an array',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      event_count: 2,
      outputs: [],
      duration_us: 0,
    },
  },

  {
    id: 'simulator-001d',
    name: 'POST /api/simulator/events - Fail with no events or scenario',
    endpoint: '/api/simulator/events',
    scenario: 'missing_params',
    category: 'simulator',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/simulator/events',
          z.any(),
          {}
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error: any) {
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        return {
          status: 500,
          data: { error: error.message },
        };
      }
    },
    assert: (actual, expected) => {
      const result = actual as { success: boolean; error?: { code: string; message: string } };

      if (result.success !== false) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected error response, got ${JSON.stringify(result)}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 400,
    expectedBody: {
      error: "Must provide either 'scenario' or 'events'",
    },
  },

  {
    id: 'simulator-001e',
    name: 'POST /api/simulator/events - Fail with unknown scenario',
    endpoint: '/api/simulator/events',
    scenario: 'unknown_scenario',
    category: 'simulator',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/simulator/events',
          z.any(),
          {
            scenario: 'invalid-scenario-name',
          }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error: any) {
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        return {
          status: 500,
          data: { error: error.message },
        };
      }
    },
    assert: (actual, expected) => {
      const result = actual as { success: boolean; error?: { code: string; message: string } };

      if (result.success !== false) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected error response, got ${JSON.stringify(result)}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 400,
    expectedBody: {
      error: 'Unknown scenario: invalid-scenario-name',
    },
  },

  // =================================================================
  // Reset Simulator Tests
  // =================================================================
  {
    id: 'simulator-002',
    name: 'POST /api/simulator/reset - Reset simulator state',
    endpoint: '/api/simulator/reset',
    scenario: 'reset',
    category: 'simulator',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/simulator/reset',
        ResetSimulatorSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as ResetSimulatorResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Reset operation did not succeed',
        };
      }

      if (typeof actualData.message !== 'string') {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'message must be a string',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      message: 'Simulator state is ephemeral (no persistent state to reset)',
    },
  },

  {
    id: 'simulator-002b',
    name: 'POST /api/simulator/reset - Verify idempotency',
    endpoint: '/api/simulator/reset',
    scenario: 'idempotent',
    category: 'simulator',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      // Call reset twice
      await client.customRequest('POST', '/api/simulator/reset', ResetSimulatorSchema);
      const response = await client.customRequest(
        'POST',
        '/api/simulator/reset',
        ResetSimulatorSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as ResetSimulatorResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Second reset did not succeed',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      message: 'Simulator state is ephemeral (no persistent state to reset)',
    },
  },
];
