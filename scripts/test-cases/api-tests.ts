/**
 * API Test Case Definitions for Automated E2E Testing
 *
 * Comprehensive test suite covering all REST API endpoints with:
 * - Arrange-Act-Assert (AAA) pattern
 * - Parametrized tests for multiple scenarios
 * - Isolated test execution with setup/cleanup
 * - Expected results validation
 */

import { ApiClient } from '../api-client/client.js';
import type { ExpectedResults, ScenarioDefinition } from './types.js';
import { healthMetricsTestCases } from './health-metrics.tests.js';
import { deviceManagementTestCases } from './device-management.tests.js';
import { profileManagementTestCases } from './profile-management.tests.js';
import { configLayersTestCases } from './config-layers.tests.js';
import { layoutsTestCases } from './layouts.tests.js';
import { macrosTestCases } from './macros.tests.js';

/**
 * Test case execution result
 */
export interface TestResult {
  passed: boolean;
  actual: unknown;
  expected: unknown;
  error?: string;
  diff?: unknown;
}

/**
 * Test case definition
 */
export interface TestCase {
  /** Unique test identifier */
  id: string;
  /** Human-readable test name */
  name: string;
  /** API endpoint being tested */
  endpoint: string;
  /** Test scenario name (matches expected-results.json) */
  scenario: string;
  /** Test category for organization */
  category: 'health' | 'profiles' | 'devices' | 'metrics' | 'layouts' | 'status' | 'config' | 'macros';
  /** Test priority (1 = critical, 2 = important, 3 = nice-to-have) */
  priority: 1 | 2 | 3;
  /** Setup function - prepare test environment */
  setup: (client: ApiClient) => Promise<void>;
  /** Execute function - perform API call and return result */
  execute: (client: ApiClient) => Promise<{
    status: number;
    data: unknown;
    headers?: Record<string, string>;
  }>;
  /** Assert function - validate response against expected results */
  assert: (actual: unknown, expected: ScenarioDefinition) => TestResult;
  /** Cleanup function - restore environment state */
  cleanup: (client: ApiClient) => Promise<void>;
}

/**
 * Default assertion function using deep equality
 */
function defaultAssert(actual: unknown, expected: ScenarioDefinition): TestResult {
  // Simple deep equality check - will be enhanced by comparator in Phase 3
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected.body);
  const passed = actualJson === expectedJson;

  return {
    passed,
    actual,
    expected: expected.body,
    error: passed ? undefined : 'Response does not match expected result',
  };
}

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
 * Helper function to wait for daemon readiness
 */
async function waitForDaemonReady(client: ApiClient, timeoutMs = 5000): Promise<void> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeoutMs) {
    try {
      const response = await client.getHealth();
      if (response.data.status === 'ok') {
        return;
      }
    } catch {
      // Daemon not ready yet, continue polling
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error('Daemon did not become ready within timeout');
}

/**
 * Helper function to create a test profile
 */
async function createTestProfile(client: ApiClient, name: string): Promise<void> {
  try {
    await client.createProfile(name);
  } catch (error) {
    // Ignore if profile already exists
    if (error instanceof Error && !error.message.includes('409')) {
      throw error;
    }
  }
}

/**
 * Helper function to delete a test profile (cleanup)
 */
async function deleteTestProfile(client: ApiClient, name: string): Promise<void> {
  try {
    await client.deleteProfile(name);
  } catch {
    // Ignore errors during cleanup
  }
}

/**
 * Comprehensive API test suite
 */
export const apiTestCases: TestCase[] = [
  // =================================================================
  // Health & Version Tests
  // =================================================================
  {
    id: 'health-001',
    name: 'GET /api/health - Daemon healthy',
    endpoint: '/api/health',
    scenario: 'healthy',
    category: 'health',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getHealth();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { status: string; version: string };
      return {
        passed: actualData.status === 'ok',
        actual,
        expected: expected.body,
        error: actualData.status !== 'ok' ? 'Health check failed' : undefined,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'version-001',
    name: 'GET /api/version - Get daemon version',
    endpoint: '/api/version',
    scenario: 'default',
    category: 'health',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getVersion();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { version: string; build_time: string; platform: string };
      const hasRequiredFields =
        typeof actualData.version === 'string' &&
        typeof actualData.build_time === 'string' &&
        typeof actualData.platform === 'string';

      return {
        passed: hasRequiredFields,
        actual,
        expected: expected.body,
        error: hasRequiredFields ? undefined : 'Missing required version fields',
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Status Tests
  // =================================================================
  {
    id: 'status-001',
    name: 'GET /api/status - Daemon running without active profile',
    endpoint: '/api/status',
    scenario: 'running',
    category: 'status',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getStatus();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { status: string; daemon_running: boolean };
      const passed = actualData.status === 'running' && actualData.daemon_running === true;

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Daemon not in expected running state',
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Profile Tests
  // =================================================================
  {
    id: 'profiles-001',
    name: 'GET /api/profiles - List profiles',
    endpoint: '/api/profiles',
    scenario: 'with_default',
    category: 'profiles',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getProfiles();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { profiles: unknown[] };
      const passed = Array.isArray(actualData.profiles);

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'profiles is not an array',
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-002',
    name: 'GET /api/profiles/active - No active profile',
    endpoint: '/api/profiles/active',
    scenario: 'no_active',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getActiveProfile();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: defaultAssert,
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-003',
    name: 'POST /api/profiles - Create new profile',
    endpoint: '/api/profiles',
    scenario: 'create_success',
    category: 'profiles',
    priority: 1,
    setup: async (client) => {
      // Ensure test profile doesn't exist
      await deleteTestProfile(client, 'test-profile-create');
    },
    execute: async (client) => {
      const response = await client.createProfile('test-profile-create', 'empty');
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { message: string; profile: { name: string } };
      const passed =
        actualData.profile?.name === 'test-profile-create' &&
        typeof actualData.message === 'string';

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile creation response invalid',
      };
    },
    cleanup: async (client) => {
      await deleteTestProfile(client, 'test-profile-create');
    },
  },

  {
    id: 'profiles-004',
    name: 'POST /api/profiles - Reject duplicate profile',
    endpoint: '/api/profiles',
    scenario: 'create_duplicate',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Ensure test profile exists
      await createTestProfile(client, 'test-profile-duplicate');
    },
    execute: async (client) => {
      try {
        const response = await client.createProfile('test-profile-duplicate');
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 409 Conflict error
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
      const actualData = actual as { error?: string; code?: string };
      const passed = actualData.code === 'PROFILE_EXISTS' || actualData.error?.includes('exists');

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Duplicate profile not rejected correctly',
      };
    },
    cleanup: async (client) => {
      await deleteTestProfile(client, 'test-profile-duplicate');
    },
  },

  {
    id: 'profiles-005',
    name: 'POST /api/profiles/:name/activate - Activate existing profile',
    endpoint: '/api/profiles/:name/activate',
    scenario: 'activate_success',
    category: 'profiles',
    priority: 1,
    setup: async (client) => {
      // Ensure test profile exists
      await createTestProfile(client, 'test-profile-activate');
    },
    execute: async (client) => {
      const response = await client.activateProfile('test-profile-activate');
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { profile?: string; message?: string };
      const passed =
        actualData.profile === 'test-profile-activate' ||
        typeof actualData.message === 'string';

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile activation failed',
      };
    },
    cleanup: async (client) => {
      await deleteTestProfile(client, 'test-profile-activate');
    },
  },

  {
    id: 'profiles-006',
    name: 'POST /api/profiles/:name/activate - Reject nonexistent profile',
    endpoint: '/api/profiles/:name/activate',
    scenario: 'activate_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.activateProfile('nonexistent-profile-xyz');
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
      const actualData = actual as { error?: string; code?: string };
      const passed = actualData.code === 'PROFILE_NOT_FOUND' || actualData.error?.includes('not found');

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Nonexistent profile activation not rejected correctly',
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-007',
    name: 'DELETE /api/profiles/:name - Delete existing profile',
    endpoint: '/api/profiles/:name',
    scenario: 'delete_success',
    category: 'profiles',
    priority: 1,
    setup: async (client) => {
      // Ensure test profile exists
      await createTestProfile(client, 'test-profile-delete');
    },
    execute: async (client) => {
      const response = await client.deleteProfile('test-profile-delete');
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { success?: boolean; message?: string };
      const passed = actualData.success === true || typeof actualData.message === 'string';

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile deletion failed',
      };
    },
    cleanup: noOpCleanup, // Profile already deleted
  },

  {
    id: 'profiles-008',
    name: 'DELETE /api/profiles/:name - Reject nonexistent profile',
    endpoint: '/api/profiles/:name',
    scenario: 'delete_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.deleteProfile('nonexistent-profile-xyz');
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
      const actualData = actual as { error?: string; code?: string };
      const passed = actualData.code === 'PROFILE_NOT_FOUND' || actualData.error?.includes('not found');

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Nonexistent profile deletion not rejected correctly',
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-009',
    name: 'GET /api/profiles/:name - Get profile configuration',
    endpoint: '/api/profiles/:name',
    scenario: 'default',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Ensure test profile exists
      await createTestProfile(client, 'test-profile-get-config');
    },
    execute: async (client) => {
      const response = await client.getProfileConfig('test-profile-get-config');
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { name?: string; source?: string };
      const passed =
        actualData.name === 'test-profile-get-config' ||
        typeof actualData.source === 'string';

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile config retrieval failed',
      };
    },
    cleanup: async (client) => {
      await deleteTestProfile(client, 'test-profile-get-config');
    },
  },

  {
    id: 'profiles-010',
    name: 'PUT /api/profiles/:name - Update profile configuration',
    endpoint: '/api/profiles/:name',
    scenario: 'update_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Ensure test profile exists
      await createTestProfile(client, 'test-profile-update');
    },
    execute: async (client) => {
      const response = await client.setProfileConfig('test-profile-update', {
        source: '// Test config\nmap("a", "b");',
      });
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { success?: boolean };
      const passed = actualData.success === true;

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile config update failed',
      };
    },
    cleanup: async (client) => {
      await deleteTestProfile(client, 'test-profile-update');
    },
  },

  // =================================================================
  // Device Tests
  // =================================================================
  {
    id: 'devices-001',
    name: 'GET /api/devices - List all devices',
    endpoint: '/api/devices',
    scenario: 'empty',
    category: 'devices',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getDevices();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { devices: unknown[] };
      const passed = Array.isArray(actualData.devices);

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'devices is not an array',
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'devices-002',
    name: 'PATCH /api/devices/:id - Update device configuration (enabled)',
    endpoint: '/api/devices/:id',
    scenario: 'update_success',
    category: 'devices',
    priority: 2,
    setup: async (client) => {
      // First get device list to find a device ID
      const devices = await client.getDevices();
      if (devices.data.devices.length === 0) {
        throw new Error('No devices available for testing - skipping device update test');
      }
    },
    execute: async (client) => {
      // Get first available device
      const devices = await client.getDevices();
      const deviceId = devices.data.devices[0]?.id as string;

      if (!deviceId) {
        throw new Error('No device ID available');
      }

      const response = await client.patchDevice(deviceId, { enabled: false });
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { message?: string; device?: unknown };
      const passed = typeof actualData.message === 'string' || actualData.device !== undefined;

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Device update failed',
      };
    },
    cleanup: async (client) => {
      // Restore device to enabled state
      try {
        const devices = await client.getDevices();
        const deviceId = devices.data.devices[0]?.id as string;
        if (deviceId) {
          await client.patchDevice(deviceId, { enabled: true });
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'devices-003',
    name: 'PATCH /api/devices/:id - Reject nonexistent device',
    endpoint: '/api/devices/:id',
    scenario: 'update_not_found',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.patchDevice('nonexistent-device-xyz', { enabled: false });
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
      const actualData = actual as { error?: string; code?: string };
      const passed = actualData.code === 'DEVICE_NOT_FOUND' || actualData.error?.includes('not found');

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Nonexistent device update not rejected correctly',
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Metrics Tests
  // =================================================================
  {
    id: 'metrics-001',
    name: 'GET /api/metrics/latency - Get latency metrics',
    endpoint: '/api/metrics/latency',
    scenario: 'default',
    category: 'metrics',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getLatencyMetrics();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        min_us?: number;
        avg_us?: number;
        max_us?: number;
        p95_us?: number;
        p99_us?: number;
      };
      const passed =
        typeof actualData.min_us === 'number' &&
        typeof actualData.avg_us === 'number' &&
        typeof actualData.max_us === 'number';

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Latency metrics missing required fields',
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Layouts Tests
  // =================================================================
  {
    id: 'layouts-001',
    name: 'GET /api/layouts - Get available keyboard layouts',
    endpoint: '/api/layouts',
    scenario: 'list',
    category: 'layouts',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const response = await client.getLayouts();
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as { layouts: unknown[] };
      const passed = Array.isArray(actualData.layouts);

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'layouts is not an array',
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Integration Tests (Multi-step workflows)
  // =================================================================
  {
    id: 'integration-001',
    name: 'Profile lifecycle - Create, Activate, Deactivate, Delete',
    endpoint: 'multiple',
    scenario: 'profile_lifecycle',
    category: 'profiles',
    priority: 1,
    setup: async (client) => {
      // Ensure clean state
      await deleteTestProfile(client, 'test-lifecycle-profile');
    },
    execute: async (client) => {
      // Step 1: Create profile
      const createResponse = await client.createProfile('test-lifecycle-profile', 'empty');

      // Step 2: Activate profile
      const activateResponse = await client.activateProfile('test-lifecycle-profile');

      // Step 3: Verify active
      const activeResponse = await client.getActiveProfile();

      // Step 4: Delete profile (will also deactivate)
      const deleteResponse = await client.deleteProfile('test-lifecycle-profile');

      return {
        status: 200,
        data: {
          create: createResponse.data,
          activate: activateResponse.data,
          active: activeResponse.data,
          delete: deleteResponse.data,
        },
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        create: { profile: { name: string } };
        activate: { profile: string };
        active: { active_profile: string };
        delete: { success?: boolean; message?: string };
      };

      const passed =
        actualData.create.profile.name === 'test-lifecycle-profile' &&
        (actualData.activate.profile === 'test-lifecycle-profile') &&
        (actualData.active.active_profile === 'test-lifecycle-profile') &&
        (actualData.delete.success === true || typeof actualData.delete.message === 'string');

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Profile lifecycle workflow failed',
      };
    },
    cleanup: noOpCleanup, // Profile already deleted
  },

  {
    id: 'integration-002',
    name: 'Device configuration - List, Update, Verify',
    endpoint: 'multiple',
    scenario: 'device_config_workflow',
    category: 'devices',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      // Step 1: List devices
      const listResponse = await client.getDevices();

      if (listResponse.data.devices.length === 0) {
        // No devices available, return mock success
        return {
          status: 200,
          data: {
            skipped: true,
            reason: 'No devices available',
          },
        };
      }

      const deviceId = listResponse.data.devices[0].id;

      // Step 2: Update device
      const updateResponse = await client.patchDevice(deviceId, { enabled: false });

      // Step 3: Verify update
      const verifyResponse = await client.getDevices();
      const updatedDevice = verifyResponse.data.devices.find((d: { id: string }) => d.id === deviceId);

      // Step 4: Restore original state
      await client.patchDevice(deviceId, { enabled: true });

      return {
        status: 200,
        data: {
          list: listResponse.data,
          update: updateResponse.data,
          verify: updatedDevice,
        },
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        skipped?: boolean;
        list?: { devices: unknown[] };
        update?: unknown;
        verify?: { enabled: boolean };
      };

      // Allow skipped tests when no devices available
      if (actualData.skipped) {
        return {
          passed: true,
          actual,
          expected: expected.body,
        };
      }

      const passed =
        Array.isArray(actualData.list?.devices) &&
        actualData.verify?.enabled === false;

      return {
        passed,
        actual,
        expected: expected.body,
        error: passed ? undefined : 'Device configuration workflow failed',
      };
    },
    cleanup: noOpCleanup, // Cleanup already done in execute
  },
];

/**
 * Get test cases by category
 */
export function getTestCasesByCategory(category: TestCase['category']): TestCase[] {
  return apiTestCases.filter((tc) => tc.category === category);
}

/**
 * Get test cases by priority
 */
export function getTestCasesByPriority(priority: TestCase['priority']): TestCase[] {
  return apiTestCases.filter((tc) => tc.priority === priority);
}

/**
 * Get critical test cases (priority 1)
 */
export function getCriticalTestCases(): TestCase[] {
  return getTestCasesByPriority(1);
}

/**
 * Get test case by ID
 */
export function getTestCaseById(id: string): TestCase | undefined {
  return apiTestCases.find((tc) => tc.id === id);
}

/**
 * Get all test cases
 */
export function getAllTestCases(): TestCase[] {
  return [
    ...apiTestCases,
    ...healthMetricsTestCases,
    ...deviceManagementTestCases,
    ...profileManagementTestCases,
    ...configLayersTestCases,
    ...layoutsTestCases,
    ...macrosTestCases,
  ];
}

/**
 * Get test statistics
 */
export function getTestStatistics(): {
  total: number;
  byCategory: Record<TestCase['category'], number>;
  byPriority: Record<TestCase['priority'], number>;
} {
  const byCategory = apiTestCases.reduce(
    (acc, tc) => {
      acc[tc.category] = (acc[tc.category] || 0) + 1;
      return acc;
    },
    {} as Record<TestCase['category'], number>
  );

  const byPriority = apiTestCases.reduce(
    (acc, tc) => {
      acc[tc.priority] = (acc[tc.priority] || 0) + 1;
      return acc;
    },
    {} as Record<TestCase['priority'], number>
  );

  return {
    total: apiTestCases.length,
    byCategory,
    byPriority,
  };
}
