/**
 * API Test Case Definitions
 *
 * Comprehensive test cases for all REST API endpoints following AAA pattern:
 * - Arrange: Set up test environment and preconditions
 * - Act: Execute the API call
 * - Assert: Validate response against expected results
 */

import { ApiClient } from '../api-client/client';
import {
  TestCase,
  ValidationResult,
  assertError,
  assertRequiredFields,
  assertFieldType,
} from './test-utils';

/**
 * Status endpoint test cases
 */
const statusTests: TestCase[] = [
  {
    id: 'status-001',
    name: 'GET /api/status - daemon running',
    endpoint: '/api/status',
    scenario: 'running',
    setup: async () => undefined,
    execute: async (client) => await client.getStatus(),
    assert: (actual) => {
      const response = actual as any;
      if (response.status !== 'running' && response.status !== 'starting') {
        return {
          passed: false,
          message: `Expected status 'running' or 'starting' but got '${response.status}'`,
        };
      }
      const typeCheck = assertFieldType(response, 'version', 'string');
      if (!typeCheck.passed) return typeCheck;
      return { passed: true, message: 'Status response valid' };
    },
  },
];

/**
 * Device endpoint test cases
 */
const deviceTests: TestCase[] = [
  {
    id: 'devices-001',
    name: 'GET /api/devices - list devices',
    endpoint: '/api/devices',
    scenario: 'empty',
    setup: async () => undefined,
    execute: async (client) => await client.getDevices(),
    assert: (actual) => {
      const response = actual as any;
      if (!Array.isArray(response.devices)) {
        return {
          passed: false,
          message: `Expected devices array but got ${typeof response.devices}`,
        };
      }
      return { passed: true, message: `Found ${response.devices.length} device(s)` };
    },
  },
  {
    id: 'devices-002',
    name: 'GET /api/devices - validates device structure',
    endpoint: '/api/devices',
    scenario: 'multiple_devices',
    setup: async () => undefined,
    execute: async (client) => await client.getDevices(),
    assert: (actual) => {
      const response = actual as any;
      if (!Array.isArray(response.devices)) {
        return { passed: false, message: 'Expected devices array' };
      }
      if (response.devices.length > 0) {
        const result = assertRequiredFields(
          response.devices[0],
          ['id', 'name', 'path', 'active', 'scope'],
          'device'
        );
        if (!result.passed) return result;
      }
      return { passed: true, message: 'Device structure valid' };
    },
  },
  {
    id: 'devices-003',
    name: 'PATCH /api/devices/:id - update device name',
    endpoint: '/api/devices/:id',
    scenario: 'update_success',
    setup: async () => undefined,
    execute: async (client) => {
      const devices = await client.getDevices();
      if (devices.devices.length === 0) {
        return { skipped: true, reason: 'No devices available' };
      }
      return await client.patchDevice(devices.devices[0].id, { name: 'Updated Test Device' });
    },
    assert: (actual) => {
      const response = actual as any;
      if (response.skipped) {
        return { passed: true, message: `Skipped: ${response.reason}` };
      }
      if (!response.success) {
        return { passed: false, message: 'Expected success: true' };
      }
      return { passed: true, message: 'Device updated successfully' };
    },
  },
  {
    id: 'devices-004',
    name: 'PATCH /api/devices/:id - nonexistent device fails',
    endpoint: '/api/devices/:id',
    scenario: 'device_not_found',
    setup: async () => undefined,
    execute: async (client) => {
      try {
        await client.patchDevice('nonexistent-device-12345', { name: 'Test' });
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected nonexistent device update to fail' };
      }
      return assertError(result.error, 404);
    },
  },
];

/**
 * Profile endpoint test cases
 */
const profileTests: TestCase[] = [
  {
    id: 'profiles-001',
    name: 'GET /api/profiles - list profiles',
    endpoint: '/api/profiles',
    scenario: 'default_only',
    setup: async () => undefined,
    execute: async (client) => await client.getProfiles(),
    assert: (actual) => {
      const response = actual as any;
      if (!Array.isArray(response.profiles)) {
        return { passed: false, message: 'Expected profiles array' };
      }
      if (response.profiles.length === 0) {
        return { passed: false, message: 'Expected at least default profile' };
      }
      const result = assertRequiredFields(
        response.profiles[0],
        ['name', 'rhaiPath', 'layerCount', 'isActive'],
        'profile'
      );
      if (!result.passed) return result;
      return { passed: true, message: `Found ${response.profiles.length} profile(s)` };
    },
  },
  {
    id: 'profiles-002',
    name: 'POST /api/profiles - create new profile',
    endpoint: '/api/profiles',
    scenario: 'create_success',
    setup: async () => undefined,
    execute: async (client) => {
      const testName = `test-profile-${Date.now()}`;
      return await client.createProfile(testName);
    },
    assert: (actual) => {
      const response = actual as any;
      if (!response.success) {
        return { passed: false, message: 'Expected success: true' };
      }
      const typeCheck = assertFieldType(response, 'name', 'string');
      if (!typeCheck.passed) return typeCheck;
      return { passed: true, message: `Created profile: ${response.name}` };
    },
    cleanup: async (client) => {
      try {
        const profiles = await client.getProfiles();
        const testProfiles = profiles.profiles.filter(p => p.name.startsWith('test-profile-'));
        for (const profile of testProfiles) {
          if (!profile.isActive) {
            await client.deleteProfile(profile.name);
          }
        }
      } catch (err) {
        // Ignore cleanup errors
      }
    },
  },
  {
    id: 'profiles-003',
    name: 'POST /api/profiles - duplicate name fails',
    endpoint: '/api/profiles',
    scenario: 'duplicate_name',
    setup: async (client) => {
      const profiles = await client.getProfiles();
      if (!profiles.profiles.some(p => p.name === 'default')) {
        await client.createProfile('default');
      }
    },
    execute: async (client) => {
      try {
        await client.createProfile('default');
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected duplicate profile creation to fail' };
      }
      return assertError(result.error, 409, 'Profile already exists: default');
    },
  },
  {
    id: 'profiles-004',
    name: 'GET /api/profiles/:name/config - get default config',
    endpoint: '/api/profiles/:name/config',
    scenario: 'default_profile',
    setup: async () => undefined,
    execute: async (client) => await client.getProfileConfig('default'),
    assert: (actual) => {
      const response = actual as any;
      if (response.name !== 'default') {
        return { passed: false, message: `Expected name 'default' but got '${response.name}'` };
      }
      const typeCheck = assertFieldType(response, 'source', 'string');
      if (!typeCheck.passed) return typeCheck;
      return { passed: true, message: 'Config response valid' };
    },
  },
  {
    id: 'profiles-005',
    name: 'GET /api/profiles/:name/config - nonexistent profile fails',
    endpoint: '/api/profiles/:name/config',
    scenario: 'not_found',
    setup: async () => undefined,
    execute: async (client) => {
      try {
        await client.getProfileConfig('nonexistent-profile-12345');
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected nonexistent profile to fail' };
      }
      return assertError(result.error, 404);
    },
  },
  {
    id: 'profiles-006',
    name: 'POST /api/profiles/:name/activate - activate default',
    endpoint: '/api/profiles/:name/activate',
    scenario: 'activate_success',
    setup: async () => undefined,
    execute: async (client) => await client.activateProfile('default'),
    assert: (actual) => {
      const response = actual as any;
      if (!response.success) {
        return { passed: false, message: `Activation failed: ${response.error || 'unknown'}` };
      }
      return { passed: true, message: 'Profile activated successfully' };
    },
  },
  {
    id: 'profiles-007',
    name: 'POST /api/profiles/:name/activate - nonexistent fails',
    endpoint: '/api/profiles/:name/activate',
    scenario: 'not_found',
    setup: async () => undefined,
    execute: async (client) => {
      try {
        await client.activateProfile('nonexistent-profile-12345');
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected nonexistent profile activation to fail' };
      }
      return assertError(result.error, 404);
    },
  },
  {
    id: 'profiles-008',
    name: 'DELETE /api/profiles/:name - delete existing profile',
    endpoint: '/api/profiles/:name',
    scenario: 'delete_success',
    setup: async (client) => {
      const testName = `delete-test-${Date.now()}`;
      await client.createProfile(testName);
      return { testName };
    },
    execute: async (client, context) => {
      return await client.deleteProfile((context as any).testName);
    },
    assert: (actual) => {
      const response = actual as any;
      if (!response.success) {
        return { passed: false, message: 'Expected success: true' };
      }
      return { passed: true, message: 'Profile deleted successfully' };
    },
  },
  {
    id: 'profiles-009',
    name: 'DELETE /api/profiles/:name - nonexistent fails',
    endpoint: '/api/profiles/:name',
    scenario: 'not_found',
    setup: async () => undefined,
    execute: async (client) => {
      try {
        await client.deleteProfile('nonexistent-profile-12345');
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected nonexistent profile deletion to fail' };
      }
      return assertError(result.error, 404);
    },
  },
  {
    id: 'profiles-010',
    name: 'DELETE /api/profiles/:name - cannot delete active profile',
    endpoint: '/api/profiles/:name',
    scenario: 'cannot_delete_active',
    setup: async (client) => {
      await client.activateProfile('default');
    },
    execute: async (client) => {
      try {
        await client.deleteProfile('default');
        return { error: null };
      } catch (err) {
        return { error: err };
      }
    },
    assert: (actual) => {
      const result = actual as any;
      if (!result.error) {
        return { passed: false, message: 'Expected active profile deletion to fail' };
      }
      return assertError(result.error, 409);
    },
  },
  {
    id: 'profiles-011',
    name: 'POST /api/profiles/:name/config - set valid config',
    endpoint: '/api/profiles/:name/config',
    scenario: 'set_config_success',
    setup: async (client) => {
      const testName = `config-test-${Date.now()}`;
      await client.createProfile(testName);
      return { testName };
    },
    execute: async (client, context) => {
      const source = '// Test configuration\nlet base = layer("base");';
      return await client.setProfileConfig((context as any).testName, source);
    },
    assert: (actual) => {
      const response = actual as any;
      if (!response.success) {
        return { passed: false, message: 'Expected success: true' };
      }
      return { passed: true, message: 'Config set successfully' };
    },
    cleanup: async (client, context) => {
      try {
        await client.deleteProfile((context as any).testName);
      } catch (err) {
        // Ignore cleanup errors
      }
    },
  },
];

/**
 * Metrics endpoint test cases
 */
const metricTests: TestCase[] = [
  {
    id: 'metrics-001',
    name: 'GET /api/metrics/latency - get latency stats',
    endpoint: '/api/metrics/latency',
    scenario: 'no_data',
    setup: async () => undefined,
    execute: async (client) => await client.getMetrics(),
    assert: (actual) => {
      const response = actual as any;
      const result = assertRequiredFields(
        response,
        ['min_us', 'avg_us', 'max_us', 'p50_us', 'p95_us', 'p99_us', 'count'],
        'metrics'
      );
      if (!result.passed) return result;

      // All values should be numbers
      const fields = ['min_us', 'avg_us', 'max_us', 'p50_us', 'p95_us', 'p99_us', 'count'];
      for (const field of fields) {
        const typeCheck = assertFieldType(response, field, 'number');
        if (!typeCheck.passed) return typeCheck;
      }

      return { passed: true, message: `Metrics valid (count: ${response.count})` };
    },
  },
];

/**
 * Layout endpoint test cases
 */
const layoutTests: TestCase[] = [
  {
    id: 'layouts-001',
    name: 'GET /api/layouts - list keyboard layouts',
    endpoint: '/api/layouts',
    scenario: 'standard_layouts',
    setup: async () => undefined,
    execute: async (client) => await client.getLayouts(),
    assert: (actual) => {
      const response = actual as any;
      if (!Array.isArray(response.layouts)) {
        return { passed: false, message: 'Expected layouts array' };
      }
      if (response.layouts.length === 0) {
        return { passed: false, message: 'Expected at least one layout' };
      }
      const result = assertRequiredFields(response.layouts[0], ['id', 'name'], 'layout');
      if (!result.passed) return result;
      return { passed: true, message: `Found ${response.layouts.length} layout(s)` };
    },
  },
];

/**
 * All API test cases combined
 */
export const testCases: TestCase[] = [
  ...statusTests,
  ...deviceTests,
  ...profileTests,
  ...metricTests,
  ...layoutTests,
];

/**
 * Export count for validation
 */
export const TEST_COUNT = testCases.length;
