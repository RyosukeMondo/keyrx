/**
 * Macro Recorder API Test Cases
 *
 * Tests for macro recording endpoints:
 * - POST /api/macros/start-recording - Start recording macro
 * - POST /api/macros/stop-recording - Stop recording macro and get event count
 * - GET /api/macros/recorded-events - Get recorded events with metadata
 * - POST /api/macros/clear - Clear all recorded events
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase, TestResult } from './api-tests.js';
import { extractData } from './api-tests.js';
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
 * Start recording response schema
 */
const StartRecordingSchema = z.object({
  success: z.boolean(),
  message: z.string(),
});

type StartRecordingResponse = z.infer<typeof StartRecordingSchema>;

/**
 * Stop recording response schema
 */
const StopRecordingSchema = z.object({
  success: z.boolean(),
  message: z.string(),
  event_count: z.number(),
});

type StopRecordingResponse = z.infer<typeof StopRecordingSchema>;

/**
 * Recorded events response schema
 */
const RecordedEventsSchema = z.object({
  success: z.boolean(),
  recording: z.boolean(),
  event_count: z.number(),
  events: z.array(z.any()), // Event structure varies
});

type RecordedEventsResponse = z.infer<typeof RecordedEventsSchema>;

/**
 * Clear events response schema
 */
const ClearEventsSchema = z.object({
  success: z.boolean(),
  message: z.string(),
});

type ClearEventsResponse = z.infer<typeof ClearEventsSchema>;

/**
 * Macro Recorder test cases
 */
export const macrosTestCases: TestCase[] = [
  // =================================================================
  // Start Recording Tests
  // =================================================================
  {
    id: 'macros-001',
    name: 'POST /api/macros/start-recording - Start recording successfully',
    endpoint: '/api/macros/start-recording',
    scenario: 'success',
    category: 'macros',
    priority: 1,
    setup: async (client) => {
      // Ensure we're not already recording and events are cleared
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if not recording
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/macros/start-recording',
        StartRecordingSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as StartRecordingResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Recording did not start successfully',
        };
      }

      if (actualData.message !== 'Recording started') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected message "Recording started", got "${actualData.message}"`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      // Stop recording and clear events
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    expectedStatus: 200,
    expectedBody: {
      success: true,
      message: 'Recording started',
    },
  },

  {
    id: 'macros-001b',
    name: 'POST /api/macros/start-recording - Fail when already recording',
    endpoint: '/api/macros/start-recording',
    scenario: 'already_recording',
    category: 'macros',
    priority: 2,
    setup: async (client) => {
      // Ensure recording is active
      await client.customRequest('POST', '/api/macros/clear', z.any());
      await client.customRequest('POST', '/api/macros/start-recording', z.any());
    },
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/macros/start-recording',
          z.any()
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
      const actualData = extractData(actual);
      const result = actual as { success: boolean; error?: { code: string; message: string } };

      if (result.success !== false || result.error?.code !== 'BAD_REQUEST') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error with BAD_REQUEST code, got ${JSON.stringify(result)}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      // Stop recording and clear events
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    expectedStatus: 400,
    expectedBody: {
      error: 'Already recording',
    },
  },

  // =================================================================
  // Stop Recording Tests
  // =================================================================
  {
    id: 'macros-002',
    name: 'POST /api/macros/stop-recording - Stop recording successfully',
    endpoint: '/api/macros/stop-recording',
    scenario: 'success',
    category: 'macros',
    priority: 1,
    setup: async (client) => {
      // Start recording first
      await client.customRequest('POST', '/api/macros/clear', z.any());
      await client.customRequest('POST', '/api/macros/start-recording', z.any());
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/macros/stop-recording',
        StopRecordingSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as StopRecordingResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Recording did not stop successfully',
        };
      }

      if (actualData.message !== 'Recording stopped') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected message "Recording stopped", got "${actualData.message}"`,
        };
      }

      if (typeof actualData.event_count !== 'number') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'event_count must be a number',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    expectedStatus: 200,
    expectedBody: {
      success: true,
      message: 'Recording stopped',
      event_count: 0,
    },
  },

  {
    id: 'macros-002b',
    name: 'POST /api/macros/stop-recording - Fail when not recording',
    endpoint: '/api/macros/stop-recording',
    scenario: 'not_recording',
    category: 'macros',
    priority: 2,
    setup: async (client) => {
      // Ensure we're not recording
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/macros/stop-recording',
          z.any()
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
      const actualData = extractData(actual);
      const result = actual as { success: boolean; error?: { code: string; message: string } };

      if (result.success !== false || result.error?.code !== 'BAD_REQUEST') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error with BAD_REQUEST code, got ${JSON.stringify(result)}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 400,
    expectedBody: {
      error: 'Not recording',
    },
  },

  // =================================================================
  // Get Recorded Events Tests
  // =================================================================
  {
    id: 'macros-003',
    name: 'GET /api/macros/recorded-events - Get events while not recording',
    endpoint: '/api/macros/recorded-events',
    scenario: 'not_recording',
    category: 'macros',
    priority: 1,
    setup: async (client) => {
      // Ensure we're not recording and events are cleared
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/macros/recorded-events',
        RecordedEventsSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as RecordedEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Request did not succeed',
        };
      }

      if (actualData.recording !== false) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected recording to be false',
        };
      }

      if (actualData.event_count !== 0) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 0 events, got ${actualData.event_count}`,
        };
      }

      if (!Array.isArray(actualData.events)) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'events must be an array',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      recording: false,
      event_count: 0,
      events: [],
    },
  },

  {
    id: 'macros-003b',
    name: 'GET /api/macros/recorded-events - Get events while recording',
    endpoint: '/api/macros/recorded-events',
    scenario: 'while_recording',
    category: 'macros',
    priority: 2,
    setup: async (client) => {
      // Start recording
      await client.customRequest('POST', '/api/macros/clear', z.any());
      await client.customRequest('POST', '/api/macros/start-recording', z.any());
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/macros/recorded-events',
        RecordedEventsSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as RecordedEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Request did not succeed',
        };
      }

      if (actualData.recording !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected recording to be true',
        };
      }

      if (typeof actualData.event_count !== 'number') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'event_count must be a number',
        };
      }

      if (!Array.isArray(actualData.events)) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'events must be an array',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    expectedStatus: 200,
    expectedBody: {
      success: true,
      recording: true,
      event_count: 0,
      events: [],
    },
  },

  // =================================================================
  // Clear Events Tests
  // =================================================================
  {
    id: 'macros-004',
    name: 'POST /api/macros/clear - Clear events successfully',
    endpoint: '/api/macros/clear',
    scenario: 'success',
    category: 'macros',
    priority: 1,
    setup: async (client) => {
      // Ensure we're not recording
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/macros/clear',
        ClearEventsSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as ClearEventsResponse;

      if (!actualData.success) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Clear operation did not succeed',
        };
      }

      if (actualData.message !== 'Events cleared') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected message "Events cleared", got "${actualData.message}"`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      message: 'Events cleared',
    },
  },

  {
    id: 'macros-004b',
    name: 'POST /api/macros/clear - Verify events are cleared',
    endpoint: '/api/macros/clear',
    scenario: 'verify_cleared',
    category: 'macros',
    priority: 2,
    setup: async (client) => {
      // Clear, then verify
      try {
        await client.customRequest('POST', '/api/macros/stop-recording', z.any());
      } catch {
        // Ignore if already stopped
      }
      await client.customRequest('POST', '/api/macros/clear', z.any());
    },
    execute: async (client) => {
      // Clear again
      await client.customRequest('POST', '/api/macros/clear', ClearEventsSchema);

      // Verify events are empty
      const eventsResponse = await client.customRequest(
        'GET',
        '/api/macros/recorded-events',
        RecordedEventsSchema
      );

      return {
        status: eventsResponse.status,
        data: eventsResponse.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as RecordedEventsResponse;

      if (actualData.event_count !== 0) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 0 events after clear, got ${actualData.event_count}`,
        };
      }

      if (actualData.events.length !== 0) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected empty events array, got ${actualData.events.length} events`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
    expectedStatus: 200,
    expectedBody: {
      success: true,
      recording: false,
      event_count: 0,
      events: [],
    },
  },
];
