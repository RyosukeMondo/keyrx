/**
 * Feature Workflow Test Cases
 *
 * Tests for complex multi-step workflows that exercise multiple endpoints:
 * - Profile lifecycle workflows (duplicate → rename → activate)
 * - Device management workflows (rename → layout → disable)
 * - Config & mapping workflows (update → add mappings → verify layers)
 * - Macro recording workflows (record → simulate → playback)
 * - Simulator workflows (event → mapping → output)
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase } from './api-tests.js';
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
 * Success response schema for profile operations
 */
const ProfileResponseSchema = z.object({
  success: z.boolean(),
  profile: z.object({
    name: z.string(),
    rhai_path: z.string(),
  }).passthrough(),
});

/**
 * Success response schema for profile activation
 */
const ActivateProfileResponseSchema = z.object({
  success: z.boolean(),
  profile_name: z.string(),
});

/**
 * Success response schema for delete operations
 */
const DeleteResponseSchema = z.object({
  success: z.boolean(),
});

// ============================================================================
// Phase 3: Feature Workflow Tests
// ============================================================================

// ============================================================================
// Task 3.1: Profile Lifecycle Workflows
// ============================================================================

/**
 * Test: Profile duplicate → rename → activate workflow
 * Test ID: workflow-002
 * Flow:
 * 1. Create a test profile
 * 2. Duplicate the profile with a new name
 * 3. Rename the duplicated profile
 * 4. Activate the renamed profile
 * 5. Verify the profile is active
 * 6. Delete both profiles (cleanup)
 */
export const workflow_002: TestCase = {
  id: 'workflow-002',
  category: 'workflows',
  description: 'Profile duplicate → rename → activate workflow',
  setup: async (client: ApiClient) => {
    // Create the initial test profile
    const config = `
// Test profile for workflow
base_layer = "base";

[keymap.base]
"a" = "b"  // Simple remapping for testing
`;
    await client.post('/profiles/workflow-test-original', { config });
  },
  execute: async (client: ApiClient) => {
    // Step 1: Duplicate the profile
    const duplicateResponse = await client.post(
      '/profiles/workflow-test-original/duplicate',
      { new_name: 'workflow-test-copy' }
    );

    // Validate duplicate response
    const duplicateData = ProfileResponseSchema.parse(duplicateResponse);
    if (!duplicateData.success) {
      throw new Error('Profile duplication failed');
    }
    if (duplicateData.profile.name !== 'workflow-test-copy') {
      throw new Error(
        `Expected duplicated profile name 'workflow-test-copy', got '${duplicateData.profile.name}'`
      );
    }

    // Step 2: Rename the duplicated profile
    const renameResponse = await client.put(
      '/profiles/workflow-test-copy/rename',
      { new_name: 'workflow-test-renamed' }
    );

    // Validate rename response
    const renameData = ProfileResponseSchema.parse(renameResponse);
    if (!renameData.success) {
      throw new Error('Profile rename failed');
    }
    if (renameData.profile.name !== 'workflow-test-renamed') {
      throw new Error(
        `Expected renamed profile name 'workflow-test-renamed', got '${renameData.profile.name}'`
      );
    }

    // Step 3: Activate the renamed profile
    const activateResponse = await client.post('/active-profile', {
      profile_name: 'workflow-test-renamed',
    });

    // Validate activation response
    const activateData = ActivateProfileResponseSchema.parse(activateResponse);
    if (!activateData.success) {
      throw new Error('Profile activation failed');
    }
    if (activateData.profile_name !== 'workflow-test-renamed') {
      throw new Error(
        `Expected active profile 'workflow-test-renamed', got '${activateData.profile_name}'`
      );
    }

    // Step 4: Verify the profile is active by getting daemon state
    const statusResponse = await client.get('/daemon/state');
    const statusData = z.object({
      success: z.boolean(),
      active_profile: z.string().nullable(),
    }).parse(statusResponse);

    if (statusData.active_profile !== 'workflow-test-renamed') {
      throw new Error(
        `Expected active profile to be 'workflow-test-renamed', got '${statusData.active_profile}'`
      );
    }

    return {
      success: true,
      workflow_steps: [
        'Created original profile',
        'Duplicated profile',
        'Renamed duplicate',
        'Activated renamed profile',
        'Verified active profile',
      ],
    };
  },
  cleanup: async (client: ApiClient) => {
    // Clean up both profiles
    try {
      await client.delete('/profiles/workflow-test-original');
    } catch (error) {
      // Profile might not exist, ignore error
    }
    try {
      await client.delete('/profiles/workflow-test-renamed');
    } catch (error) {
      // Profile might not exist, ignore error
    }
  },
  expectedStatus: 200,
  expectedResponse: {
    success: true,
  },
};

/**
 * Test: Profile validation → fix → activate workflow
 * Test ID: workflow-003
 * Flow:
 * 1. Create a profile with invalid syntax
 * 2. Validate the profile (should fail)
 * 3. Fix the syntax error
 * 4. Validate again (should pass)
 * 5. Activate the fixed profile
 * 6. Verify the profile is active
 * 7. Delete the profile (cleanup)
 */
export const workflow_003: TestCase = {
  id: 'workflow-003',
  category: 'workflows',
  description: 'Profile validation → fix → activate workflow',
  setup: noOpSetup,
  execute: async (client: ApiClient) => {
    // Step 1: Create a profile with invalid syntax (missing closing quote)
    const invalidConfig = `
// Test profile with syntax error
base_layer = "base;

[keymap.base]
"a" = "b"
`;
    await client.post('/profiles/workflow-validation-test', { config: invalidConfig });

    // Step 2: Validate the profile (should fail)
    const validateFailResponse = await client.post(
      '/profiles/workflow-validation-test/validate',
      {}
    );

    const validateFailData = z.object({
      success: z.boolean(),
      valid: z.boolean().optional(),
      errors: z.array(z.string()).optional(),
    }).parse(validateFailResponse);

    if (validateFailData.success && validateFailData.valid) {
      throw new Error('Expected validation to fail for invalid syntax, but it passed');
    }

    if (!validateFailData.errors || validateFailData.errors.length === 0) {
      throw new Error('Expected validation errors for invalid syntax');
    }

    // Step 3: Fix the syntax error (add closing quote)
    const validConfig = `
// Test profile with syntax fixed
base_layer = "base";

[keymap.base]
"a" = "b"
`;
    const updateResponse = await client.put('/profiles/workflow-validation-test/config', {
      config: validConfig,
    });

    const updateData = z.object({
      success: z.boolean(),
    }).parse(updateResponse);

    if (!updateData.success) {
      throw new Error('Profile config update failed');
    }

    // Step 4: Validate again (should pass)
    const validatePassResponse = await client.post(
      '/profiles/workflow-validation-test/validate',
      {}
    );

    const validatePassData = z.object({
      success: z.boolean(),
      valid: z.boolean(),
      errors: z.array(z.string()).optional(),
    }).parse(validatePassResponse);

    if (!validatePassData.success || !validatePassData.valid) {
      throw new Error(
        `Expected validation to pass for valid syntax, errors: ${validatePassData.errors?.join(', ')}`
      );
    }

    // Step 5: Activate the fixed profile
    const activateResponse = await client.post('/active-profile', {
      profile_name: 'workflow-validation-test',
    });

    const activateData = ActivateProfileResponseSchema.parse(activateResponse);
    if (!activateData.success) {
      throw new Error('Profile activation failed');
    }
    if (activateData.profile_name !== 'workflow-validation-test') {
      throw new Error(
        `Expected active profile 'workflow-validation-test', got '${activateData.profile_name}'`
      );
    }

    // Step 6: Verify the profile is active
    const statusResponse = await client.get('/daemon/state');
    const statusData = z.object({
      success: z.boolean(),
      active_profile: z.string().nullable(),
    }).parse(statusResponse);

    if (statusData.active_profile !== 'workflow-validation-test') {
      throw new Error(
        `Expected active profile to be 'workflow-validation-test', got '${statusData.active_profile}'`
      );
    }

    return {
      success: true,
      workflow_steps: [
        'Created profile with invalid syntax',
        'Validated profile (failed as expected)',
        'Fixed syntax error',
        'Validated again (passed)',
        'Activated fixed profile',
        'Verified active profile',
      ],
    };
  },
  cleanup: async (client: ApiClient) => {
    // Clean up the test profile
    try {
      await client.delete('/profiles/workflow-validation-test');
    } catch (error) {
      // Profile might not exist, ignore error
    }
  },
  expectedStatus: 200,
  expectedResponse: {
    success: true,
  },
};

// ============================================================================
// Task 3.2: Device Management Workflows
// ============================================================================

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
 * Test: Device rename → layout change → disable workflow
 * Test ID: workflow-004
 * Flow:
 * 1. List devices and get the first device
 * 2. Rename the device
 * 3. Change the device layout
 * 4. Disable the device
 * 5. Verify the device is disabled
 * 6. Restore device to original state (cleanup)
 */
export const workflow_004: TestCase = {
  id: 'workflow-004',
  category: 'workflows',
  description: 'Device rename → layout change → disable workflow',
  setup: async (client: ApiClient) => {
    // Verify at least one device exists
    const deviceId = await getFirstDeviceId(client);
    if (!deviceId) {
      throw new Error('No devices available for testing - test will be skipped');
    }
  },
  execute: async (client: ApiClient) => {
    // Step 1: List devices and get the first device
    const devicesResponse = await client.getDevices();
    const devicesData = z.object({
      success: z.boolean(),
      devices: z.array(z.object({
        id: z.string(),
        name: z.string(),
        enabled: z.boolean().optional(),
        layout: z.string().optional(),
      }).passthrough()),
    }).parse(devicesResponse.data);

    if (devicesData.devices.length === 0) {
      throw new Error('No devices available for testing');
    }

    const device = devicesData.devices[0];
    const deviceId = device.id;
    const originalName = device.name;
    const originalLayout = device.layout || 'ansi104';
    const originalEnabled = device.enabled ?? true;

    // Step 2: Rename the device
    const newName = `workflow-test-device-${Date.now()}`;
    const renameResponse = await client.customRequest(
      'PUT',
      `/api/devices/${deviceId}/name`,
      z.object({ success: z.boolean() }),
      { name: newName }
    );

    if (!renameResponse.data.success) {
      throw new Error('Device rename failed');
    }

    // Verify the rename by listing devices again
    const verifyRenameResponse = await client.getDevices();
    const verifyRenameData = z.object({
      devices: z.array(z.object({
        id: z.string(),
        name: z.string(),
      }).passthrough()),
    }).parse(verifyRenameResponse.data);

    const renamedDevice = verifyRenameData.devices.find(d => d.id === deviceId);
    if (!renamedDevice || renamedDevice.name !== newName) {
      throw new Error(
        `Device rename verification failed: expected name '${newName}', got '${renamedDevice?.name}'`
      );
    }

    // Step 3: Change the device layout
    const newLayout = originalLayout === 'ansi104' ? 'iso105' : 'ansi104';
    const layoutResponse = await client.customRequest(
      'PUT',
      `/api/devices/${deviceId}/layout`,
      z.object({ success: z.boolean() }),
      { layout: newLayout }
    );

    if (!layoutResponse.data.success) {
      throw new Error('Device layout change failed');
    }

    // Verify the layout change
    const verifyLayoutResponse = await client.customRequest(
      'GET',
      `/api/devices/${deviceId}/layout`,
      z.object({ success: z.boolean(), layout: z.string() }),
      undefined
    );

    if (!verifyLayoutResponse.data.success || verifyLayoutResponse.data.layout !== newLayout) {
      throw new Error(
        `Device layout verification failed: expected '${newLayout}', got '${verifyLayoutResponse.data.layout}'`
      );
    }

    // Step 4: Disable the device
    const disableResponse = await client.patchDevice(deviceId, { enabled: false });

    const disableData = z.object({
      success: z.boolean(),
    }).parse(disableResponse.data);

    if (!disableData.success) {
      throw new Error('Device disable failed');
    }

    // Step 5: Verify the device is disabled
    const verifyDisableResponse = await client.getDevices();
    const verifyDisableData = z.object({
      devices: z.array(z.object({
        id: z.string(),
        enabled: z.boolean().optional(),
      }).passthrough()),
    }).parse(verifyDisableResponse.data);

    const disabledDevice = verifyDisableData.devices.find(d => d.id === deviceId);
    if (!disabledDevice || disabledDevice.enabled !== false) {
      throw new Error(
        `Device disable verification failed: expected enabled=false, got enabled=${disabledDevice?.enabled}`
      );
    }

    // Store context for cleanup
    return {
      success: true,
      workflow_steps: [
        'Listed devices',
        'Renamed device',
        'Changed device layout',
        'Disabled device',
        'Verified device is disabled',
      ],
      context: {
        deviceId,
        originalName,
        originalLayout,
        originalEnabled,
      },
    };
  },
  cleanup: async (client: ApiClient) => {
    // Restore the device to its original state
    try {
      const deviceId = await getFirstDeviceId(client);
      if (!deviceId) {
        return;
      }

      // Get the devices to find the test device
      const devicesResponse = await client.getDevices();
      const device = devicesResponse.data.devices.find((d: any) => d.id === deviceId);

      if (!device) {
        return;
      }

      // Restore enabled state
      await client.patchDevice(deviceId, { enabled: true });

      // Restore original layout (default to ansi104)
      await client.customRequest(
        'PUT',
        `/api/devices/${deviceId}/layout`,
        z.object({ success: z.boolean() }),
        { layout: 'ansi104' }
      );

      // Note: We can't easily restore the original name without storing it,
      // but the device will still function correctly with a test name
    } catch (error) {
      // Ignore cleanup errors
    }
  },
  expectedStatus: 200,
  expectedResponse: {
    success: true,
  },
};

/**
 * All workflow test cases
 */
export const workflowTestCases: TestCase[] = [
  workflow_002,
  workflow_003,
  workflow_004,
];
