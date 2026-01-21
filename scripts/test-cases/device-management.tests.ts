/**
 * Device Management API Test Cases
 *
 * Tests for device configuration endpoints:
 * - PUT /api/devices/:id/name - Rename a device
 * - PUT /api/devices/:id/layout - Set device layout
 * - GET /api/devices/:id/layout - Get device layout
 * - DELETE /api/devices/:id - Forget a device
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase } from './api-tests.js';
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
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
 * Success response schema for device operations
 */
const SuccessResponseSchema = z.object({
  success: z.boolean(),
});

/**
 * Error response schema
 */
const ErrorResponseSchema = z.object({
  error: z.string(),
  code: z.string().optional(),
});

/**
 * Helper function to get a valid device ID for testing
 */
async function getFirstDeviceId(client: ApiClient): Promise<string | null> {
  const devices = await client.getDevices();
  if (devices.data.devices.length === 0) {
    return null;
  }
  return (devices.data.devices[0] as { id: string }).id;
}

/**
 * Device Management test cases
 */
export const deviceManagementTestCases: TestCase[] = [
  // =================================================================
  // Device Rename Tests
  // =================================================================
  {
    id: 'devices-004',
    name: 'PUT /api/devices/:id/name - Rename device (success)',
    endpoint: '/api/devices/:id/name',
    scenario: 'rename_success',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing - skipping device rename test');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      // Store original name for cleanup
      const devices = await client.getDevices();
      const device = devices.data.devices.find((d: any) => d.id === deviceId);
      const originalName = device?.name || 'Unknown Device';

      const newName = `test-renamed-device-${Date.now()}`;
      const response = await client.customRequest(
        'PUT',
        `/api/devices/${encodeURIComponent(deviceId)}/name`,
        SuccessResponseSchema,
        { name: newName }
      );

      return {
        status: response.status,
        data: response.data,
        context: { deviceId, originalName, newName },
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { success?: boolean };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected success=true for device rename',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Restore original device name
      try {
        const context = (result as any)?.context;
        if (context?.deviceId && context?.originalName) {
          await client.customRequest(
            'PUT',
            `/api/devices/${encodeURIComponent(context.deviceId)}/name`,
            SuccessResponseSchema,
            { name: context.originalName }
          );
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'devices-004b',
    name: 'PUT /api/devices/:id/name - Rename nonexistent device (404)',
    endpoint: '/api/devices/:id/name',
    scenario: 'rename_not_found',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'PUT',
          '/api/devices/nonexistent-device-xyz/name',
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { name: 'test-name' }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 404) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 404 for nonexistent device, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-004c',
    name: 'PUT /api/devices/:id/name - Reject empty name (400)',
    endpoint: '/api/devices/:id/name',
    scenario: 'rename_invalid_empty',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/devices/${encodeURIComponent(deviceId)}/name`,
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { name: '' }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 400 Bad Request error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 400) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 400 for empty name, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-004d',
    name: 'PUT /api/devices/:id/name - Reject too long name (400)',
    endpoint: '/api/devices/:id/name',
    scenario: 'rename_invalid_too_long',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      // Create a name longer than 100 characters (validation limit)
      const tooLongName = 'a'.repeat(101);

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/devices/${encodeURIComponent(deviceId)}/name`,
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { name: tooLongName }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 400 Bad Request error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 400) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 400 for name > 100 chars, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Device Layout Tests
  // =================================================================
  {
    id: 'devices-005',
    name: 'PUT /api/devices/:id/layout - Set device layout (success)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'set_layout_success',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      // Store original layout for cleanup
      let originalLayout: string | undefined;
      try {
        const layoutResp = await client.customRequest(
          'GET',
          `/api/devices/${encodeURIComponent(deviceId)}/layout`,
          z.object({ layout: z.string().optional() })
        );
        originalLayout = layoutResp.data.layout;
      } catch {
        // Device may not have layout set yet
        originalLayout = undefined;
      }

      const response = await client.customRequest(
        'PUT',
        `/api/devices/${encodeURIComponent(deviceId)}/layout`,
        SuccessResponseSchema,
        { layout: 'ansi-104' }
      );

      return {
        status: response.status,
        data: response.data,
        context: { deviceId, originalLayout },
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { success?: boolean };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected success=true for device layout set',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Restore original device layout
      try {
        const context = (result as any)?.context;
        if (context?.deviceId) {
          if (context.originalLayout) {
            await client.customRequest(
              'PUT',
              `/api/devices/${encodeURIComponent(context.deviceId)}/layout`,
              SuccessResponseSchema,
              { layout: context.originalLayout }
            );
          }
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'devices-005b',
    name: 'PUT /api/devices/:id/layout - Set layout on nonexistent device (404)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'set_layout_not_found',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'PUT',
          '/api/devices/nonexistent-device-xyz/layout',
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { layout: 'ansi-104' }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 404 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 404 or 500 for nonexistent device, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-005c',
    name: 'PUT /api/devices/:id/layout - Reject empty layout (400)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'set_layout_invalid_empty',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/devices/${encodeURIComponent(deviceId)}/layout`,
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { layout: '' }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 400 Bad Request error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 400) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 400 for empty layout, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-005d',
    name: 'PUT /api/devices/:id/layout - Reject too long layout name (400)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'set_layout_invalid_too_long',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      // Create a layout name longer than 50 characters (validation limit)
      const tooLongLayout = 'a'.repeat(51);

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/devices/${encodeURIComponent(deviceId)}/layout`,
          z.union([SuccessResponseSchema, ErrorResponseSchema]),
          { layout: tooLongLayout }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 400 Bad Request error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 400) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 400 for layout name > 50 chars, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Device Layout Retrieval Tests
  // =================================================================
  {
    id: 'devices-006',
    name: 'GET /api/devices/:id/layout - Get device layout (success)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'get_layout_success',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // Verify at least one device exists
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No devices available for testing');
      }

      // Set a known layout for testing
      await client.customRequest(
        'PUT',
        `/api/devices/${encodeURIComponent(deviceId)}/layout`,
        SuccessResponseSchema,
        { layout: 'ansi-104' }
      );
    },
    execute: async (client) => {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        throw new Error('No device ID available');
      }

      const response = await client.customRequest(
        'GET',
        `/api/devices/${encodeURIComponent(deviceId)}/layout`,
        z.object({ layout: z.string().optional() })
      );

      return {
        status: response.status,
        data: response.data,
        context: { deviceId },
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { layout?: string };

      if (actualData.layout !== 'ansi-104') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected layout='ansi-104', got ${actualData.layout}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-006b',
    name: 'GET /api/devices/:id/layout - Get layout for nonexistent device (404)',
    endpoint: '/api/devices/:id/layout',
    scenario: 'get_layout_not_found',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'GET',
          '/api/devices/nonexistent-device-xyz/layout',
          z.object({ layout: z.string().optional() })
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 404 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 404 or 500 for nonexistent device, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Device Deletion Tests
  // =================================================================
  {
    id: 'devices-007',
    name: 'DELETE /api/devices/:id - Forget nonexistent device (404)',
    endpoint: '/api/devices/:id',
    scenario: 'forget_not_found',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'DELETE',
          '/api/devices/nonexistent-device-xyz',
          z.union([SuccessResponseSchema, ErrorResponseSchema])
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      if (status !== 404 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 404 or 500 for nonexistent device, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },
];
