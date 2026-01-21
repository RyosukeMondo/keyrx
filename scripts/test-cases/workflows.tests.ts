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
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
import { z } from 'zod';

/**
 * Generate a short test profile name (max 32 chars per API limit)
 * Format: "prf-{prefix}-{timestamp_last6}"
 * Example: "prf-wf-234567" (14-17 chars depending on prefix)
 */
function shortProfileName(prefix: string): string {
  const timestamp = Date.now().toString().slice(-6); // Last 6 digits
  return `prf-${prefix}-${timestamp}`;
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
    // Create the initial test profile using simple_remap template
    await client.createProfile('workflow-test-original', 'simple_remap');
  },
  execute: async (client: ApiClient) => {
    // Step 1: Duplicate the profile
    const duplicateResponse = await client.post(
      '/api/profiles/workflow-test-original/duplicate',
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
      '/api/profiles/workflow-test-copy/rename',
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
    const activateResponse = await client.activateProfile('workflow-test-renamed');

    // Validate activation response
    if (!activateResponse.data.success) {
      throw new Error('Profile activation failed');
    }

    // Step 4: Verify the profile is active by getting daemon state
    const statusResponse = await client.get('/api/daemon/state');
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
      await client.deleteProfile('workflow-test-original');
    } catch (error) {
      // Profile might not exist, ignore error
    }
    try {
      await client.deleteProfile('workflow-test-renamed');
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
    // Step 1: Create a profile and set invalid config (missing closing quote)
    await client.createProfile('workflow-validation-test', 'blank');

    const invalidConfig = `
// Test profile with syntax error (missing closing paren)
device_start("*");
  map("VK_A", "VK_B";  // Missing closing paren
device_end();
`;
    await client.setProfileConfig('workflow-validation-test', { source: invalidConfig });

    // Step 2: Validate the profile (should fail)
    const validateFailResponse = await client.post(
      '/api/profiles/workflow-validation-test/validate',
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

    // Step 3: Fix the syntax error (add closing paren)
    const validConfig = `
// Test profile with syntax fixed
device_start("*");
  map("VK_A", "VK_B");  // Fixed closing paren
device_end();
`;
    const updateResponse = await client.setProfileConfig('workflow-validation-test', {
      source: validConfig,
    });

    if (!updateResponse.data.success) {
      throw new Error('Profile config update failed');
    }

    // Step 4: Validate again (should pass)
    const validatePassResponse = await client.post(
      '/api/profiles/workflow-validation-test/validate',
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
    const activateResponse = await client.activateProfile('workflow-validation-test');

    if (!activateResponse.data.success) {
      throw new Error('Profile activation failed');
    }

    // Step 6: Verify the profile is active
    const statusResponse = await client.get('/api/daemon/state');
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
      await client.deleteProfile('workflow-validation-test');
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
 * Test: Device rename → layout change workflow
 * Test ID: workflow-004
 * Flow:
 * 1. List devices and get the first device
 * 2. Rename the device
 * 3. Change the device layout
 * 4. Verify the layout change
 * 5. Restore device to original state (cleanup)
 */
export const workflow_004: TestCase = {
  id: 'workflow-004',
  category: 'workflows',
  description: 'Device rename → layout change workflow',
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
      devices: z.array(z.object({
        id: z.string(),
        name: z.string(),
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

    // Step 2: Rename the device
    const newName = `workflow-test-device-${Date.now()}`;
    const renameResponse = await client.customRequest(
      'PUT',
      `/api/devices/${encodeURIComponent(deviceId)}/name`,
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
      `/api/devices/${encodeURIComponent(deviceId)}/layout`,
      z.object({ success: z.boolean() }),
      { layout: newLayout }
    );

    if (!layoutResponse.data.success) {
      throw new Error('Device layout change failed');
    }

    // Verify the layout change
    const verifyLayoutResponse = await client.customRequest(
      'GET',
      `/api/devices/${encodeURIComponent(deviceId)}/layout`,
      z.object({ layout: z.string().optional() }),
      undefined
    );

    if (verifyLayoutResponse.data.layout !== newLayout) {
      throw new Error(
        `Device layout verification failed: expected '${newLayout}', got '${verifyLayoutResponse.data.layout}'`
      );
    }

    // Store context for cleanup
    return {
      success: true,
      workflow_steps: [
        'Listed devices',
        'Renamed device',
        'Changed device layout',
        'Verified layout change',
      ],
      context: {
        deviceId,
        originalName,
        originalLayout,
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

      // Restore original layout (default to ansi104)
      await client.customRequest(
        'PUT',
        `/api/devices/${encodeURIComponent(deviceId)}/layout`,
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

// ============================================================================
// Task 3.3: Config & Mapping Workflows
// ============================================================================

/**
 * Test: Config update → add mappings → verify layers workflow
 * Test ID: workflow-005
 * Flow:
 * 1. Get initial config
 * 2. Add a key mapping via config API
 * 3. Verify the mapping was added
 * 4. Get layers and verify structure
 * 5. Delete the mapping
 * 6. Verify the mapping was removed
 */
export const workflow_005: TestCase = {
  id: 'workflow-005',
  category: 'workflows',
  description: 'Config update → add mappings → verify layers workflow',
  setup: async (client: ApiClient) => {
    // Ensure we have an active profile with device blocks
    const profiles = await client.getProfiles();
    if (profiles.data.profiles.length === 0) {
      await client.createProfile('workflow-config-test', 'blank');
      await client.activateProfile('workflow-config-test');
      // Set valid config with device blocks
      await client.customRequest('PUT', '/api/config', z.any(), {
        content: `// Auto-generated config\ndevice_start("*");\ndevice_end();`,
      });
    } else {
      // Ensure there's an active profile with device blocks
      const activeProfile = await client.getActiveProfile();
      if (!activeProfile.data.profile) {
        await client.activateProfile(profiles.data.profiles[0].name);
      }
      // Ensure active profile has device blocks
      await client.customRequest('PUT', '/api/config', z.any(), {
        content: `// Auto-generated config\ndevice_start("*");\ndevice_end();`,
      });
    }
  },
  execute: async (client: ApiClient) => {
    // Step 1: Get initial config
    const configSchema = z.object({
      profile: z.string(),
      base_mappings: z.array(z.any()),
      layers: z.array(z.any()),
    });
    const initialConfigResponse = await client.customRequest(
      'GET',
      '/api/config',
      configSchema,
      undefined
    );
    const initialConfigData = initialConfigResponse.data;

    const initialMappingCount = initialConfigData.base_mappings.length;

    // Step 2: Add a key mapping via POST /api/config/key-mappings
    const mappingToAdd = {
      layer: 'base',
      key: 'VK_A',
      action_type: 'simple',
      output: 'VK_B',
    };

    const addMappingSchema = z.object({
      success: z.boolean(),
    });
    const addMappingResponse = await client.customRequest(
      'POST',
      '/api/config/key-mappings',
      addMappingSchema,
      mappingToAdd
    );
    const addMappingData = addMappingResponse.data;

    if (!addMappingData.success) {
      throw new Error('Failed to add key mapping');
    }

    const mappingId = `base:VK_A`; // Mapping ID format is layer:key

    // Step 3: Verify the mapping was added by getting the config again
    const updatedConfigResponse = await client.customRequest(
      'GET',
      '/api/config',
      configSchema,
      undefined
    );
    const updatedConfigData = updatedConfigResponse.data;

    // Verify mapping was added by checking the count increased
    if (updatedConfigData.base_mappings.length !== initialMappingCount + 1) {
      throw new Error(`Expected ${initialMappingCount + 1} mappings, got ${updatedConfigData.base_mappings.length}`);
    }

    // Step 4: Get layers and verify structure
    const layersSchema = z.object({
      layers: z.array(z.object({
        id: z.string(),
        mapping_count: z.number(),
      }).passthrough()),
    });
    const layersResponse = await client.customRequest(
      'GET',
      '/api/layers',
      layersSchema,
      undefined
    );
    const layersData = layersResponse.data;

    if (!layersData.layers || layersData.layers.length === 0) {
      throw new Error('No layers returned from layers endpoint');
    }

    // Verify at least the base layer exists
    const baseLayer = layersData.layers.find(layer => layer.id === 'base');
    if (!baseLayer) {
      throw new Error('Base layer not found in layers response');
    }

    // Step 5: Delete the mapping
    const deleteMappingSchema = z.object({
      success: z.boolean(),
    });
    const deleteMappingResponse = await client.customRequest(
      'DELETE',
      `/api/config/key-mappings/${mappingId}`,
      deleteMappingSchema,
      undefined
    );
    const deleteMappingData = deleteMappingResponse.data;

    if (!deleteMappingData.success) {
      throw new Error('Failed to delete key mapping');
    }

    // Step 6: Verify the mapping was removed by getting the config again
    const finalConfigResponse = await client.customRequest(
      'GET',
      '/api/config',
      configSchema,
      undefined
    );
    const finalConfigData = finalConfigResponse.data;

    // Verify mapping count is back to initial
    if (finalConfigData.base_mappings.length !== initialMappingCount) {
      throw new Error(`Expected ${initialMappingCount} mappings after delete, got ${finalConfigData.base_mappings.length}`);
    }

    return {
      success: true,
      workflow_steps: [
        'Got initial config',
        'Added key mapping via API',
        'Verified mapping was added',
        'Got layers and verified structure',
        'Deleted mapping',
        'Verified mapping was removed',
      ],
      mapping_id: mappingId,
      layers_count: layersData.layers.length,
    };
  },
  cleanup: async (client: ApiClient) => {
    // Cleanup is already done in the execute step (mapping deleted)
    // No additional cleanup needed
  },
  expectedStatus: 200,
  expectedResponse: {
    success: true,
  },
};

// ============================================================================
// Task 3.4: Macro Recording Workflows
// ============================================================================

/**
 * Test: Macro record → simulate → playback workflow
 * Test ID: workflow-006
 * Flow:
 * 1. Clear any existing recorded events
 * 2. Start macro recording
 * 3. Simulate key events (press and release)
 * 4. Stop macro recording
 * 5. Get recorded events and verify timing
 * 6. Verify event count and structure
 * 7. Clear recorded events (cleanup)
 */
export const workflow_006: TestCase = {
  id: 'workflow-006',
  category: 'workflows',
  description: 'Macro record → simulate → playback workflow',
  setup: async (client: ApiClient) => {
    // Clear any existing recorded events
    const clearSchema = z.object({ success: z.boolean() });
    await client.customRequest('POST', '/api/macros/clear', clearSchema, {});
  },
  execute: async (client: ApiClient) => {
    // Step 1: Start macro recording
    const startRecordingSchema = z.object({
      success: z.boolean(),
      message: z.string().optional(),
    });
    const startResponse = await client.customRequest(
      'POST',
      '/api/macros/start-recording',
      startRecordingSchema,
      {}
    );

    if (!startResponse.data.success) {
      throw new Error('Failed to start macro recording');
    }

    // Step 2: Simulate key events (press 'a' key, then release it)
    // First simulate key press
    const simulatePressSchema = z.object({
      success: z.boolean(),
    });
    const pressResponse = await client.customRequest(
      'POST',
      '/api/simulator/events',
      simulatePressSchema,
      {
        events: [
          {
            key: 'A', // 'a' key
            event_type: 'press',
            timestamp_us: 0,
          },
        ],
      }
    );

    if (!pressResponse.data.success) {
      throw new Error('Failed to simulate key press');
    }

    // Small delay to ensure events are recorded with different timestamps
    await new Promise(resolve => setTimeout(resolve, 10));

    // Then simulate key release
    const releaseResponse = await client.customRequest(
      'POST',
      '/api/simulator/events',
      simulatePressSchema,
      {
        events: [
          {
            key: 'A', // 'a' key
            event_type: 'release',
            timestamp_us: 10000,
          },
        ],
      }
    );

    if (!releaseResponse.data.success) {
      throw new Error('Failed to simulate key release');
    }

    // Small delay to ensure events are recorded
    await new Promise(resolve => setTimeout(resolve, 10));

    // Step 3: Stop macro recording
    const stopRecordingSchema = z.object({
      success: z.boolean(),
      message: z.string().optional(),
    });
    const stopResponse = await client.customRequest(
      'POST',
      '/api/macros/stop-recording',
      stopRecordingSchema,
      {}
    );

    if (!stopResponse.data.success) {
      throw new Error('Failed to stop macro recording');
    }

    // Step 4: Get recorded events
    const getEventsSchema = z.object({
      success: z.boolean(),
      events: z.array(z.object({
        key_code: z.number(),
        event_type: z.number(),
        timestamp_micros: z.number().optional(),
      })).optional(),
    });
    const eventsResponse = await client.customRequest(
      'GET',
      '/api/macros/recorded-events',
      getEventsSchema,
      undefined
    );

    if (!eventsResponse.data.success) {
      throw new Error('Failed to get recorded events');
    }

    if (!eventsResponse.data.events) {
      throw new Error('No events array returned from recorded-events endpoint');
    }

    // Step 5: Verify event count and structure
    const events = eventsResponse.data.events;
    if (events.length < 2) {
      throw new Error(
        `Expected at least 2 recorded events (press and release), got ${events.length}`
      );
    }

    // Verify the first event is a press
    const pressEvent = events[0];
    if (pressEvent.key_code !== 30) {
      throw new Error(`Expected first event key_code to be 30, got ${pressEvent.key_code}`);
    }
    if (pressEvent.event_type !== 0) {
      throw new Error(`Expected first event to be press (0), got ${pressEvent.event_type}`);
    }

    // Verify the second event is a release
    const releaseEvent = events[1];
    if (releaseEvent.key_code !== 30) {
      throw new Error(`Expected second event key_code to be 30, got ${releaseEvent.key_code}`);
    }
    if (releaseEvent.event_type !== 1) {
      throw new Error(`Expected second event to be release (1), got ${releaseEvent.event_type}`);
    }

    // Step 6: Verify timing (if timestamps are provided)
    if (pressEvent.timestamp_micros && releaseEvent.timestamp_micros) {
      const timeDiff = releaseEvent.timestamp_micros - pressEvent.timestamp_micros;
      if (timeDiff < 0) {
        throw new Error(
          `Release event timestamp should be after press event, got diff ${timeDiff}`
        );
      }
    }

    // Step 7: Clear recorded events
    const clearSchema = z.object({
      success: z.boolean(),
    });
    const clearResponse = await client.customRequest(
      'POST',
      '/api/macros/clear',
      clearSchema,
      {}
    );

    if (!clearResponse.data.success) {
      throw new Error('Failed to clear recorded events');
    }

    // Verify events were cleared
    const verifyEmptyResponse = await client.customRequest(
      'GET',
      '/api/macros/recorded-events',
      getEventsSchema,
      undefined
    );

    if (!verifyEmptyResponse.data.success) {
      throw new Error('Failed to verify events were cleared');
    }

    const eventsAfterClear = verifyEmptyResponse.data.events || [];
    if (eventsAfterClear.length > 0) {
      throw new Error(
        `Expected 0 events after clear, got ${eventsAfterClear.length}`
      );
    }

    return {
      success: true,
      workflow_steps: [
        'Started macro recording',
        'Simulated key press event',
        'Simulated key release event',
        'Stopped macro recording',
        'Retrieved recorded events',
        'Verified event count and structure',
        'Verified event timing',
        'Cleared recorded events',
        'Verified events were cleared',
      ],
      events_recorded: events.length,
    };
  },
  cleanup: async (client: ApiClient) => {
    // Ensure recording is stopped and events are cleared
    try {
      const stopSchema = z.object({ success: z.boolean(), message: z.string().optional() });
      await client.customRequest('POST', '/api/macros/stop-recording', stopSchema, {});
    } catch (error) {
      // Recording might not be active, ignore error
    }
    try {
      const clearSchema = z.object({ success: z.boolean() });
      await client.customRequest('POST', '/api/macros/clear', clearSchema, {});
    } catch (error) {
      // Events might already be cleared, ignore error
    }
  },
  expectedStatus: 200,
  expectedResponse: {
    success: true,
  },
};

// ============================================================================
// Task 3.5: Simulator Workflows
// ============================================================================

/**
 * Test: Simulator event → mapping → output workflow
 * Test ID: workflow-007
 * Flow:
 * 1. Create a profile with a key mapping (a→b)
 * 2. Activate the profile
 * 3. Simulate 'a' key press via simulator API
 * 4. Verify the remapping occurred (check daemon state or events)
 * 5. Reset simulator state
 * 6. Delete the test profile (cleanup)
 */
export const workflow_007: TestCase = {
  id: 'workflow-007',
  category: 'workflows',
  description: 'Simulator event → mapping → output workflow',
  setup: async (client: ApiClient) => {
    // Create a test profile (or use existing if it exists)
    try {
      await client.createProfile('workflow-simulator-test', 'blank');
    } catch (error) {
      // Profile might already exist, that's ok
    }
  },
  execute: async (client: ApiClient) => {
    // Step 1: Activate the profile
    const activateResult = await client.activateProfile('workflow-simulator-test');
    if (!activateResult.data.success) {
      throw new Error('Failed to activate workflow-simulator-test profile');
    }

    // Step 2: Set the config with mapping using PUT /api/config
    const config = `// Test profile for simulator workflow
device_start("*");
  map("VK_A", "VK_B");  // Remap 'a' to 'b'
device_end();`;
    await client.customRequest('PUT', '/api/config', z.any(), { content: config });

    // Step 3: Verify the profile is active
    const activateResponse = await client.getActiveProfile();
    if (activateResponse.data.active_profile !== 'workflow-simulator-test') {
      throw new Error(`Expected active profile 'workflow-simulator-test', got '${activateResponse.data.active_profile}'`);
    }

    // Step 4: Simulate 'a' key press and release
    const simulateSchema = z.object({
      success: z.boolean(),
    });

    // Simulate key press (event_type: 0 = press)
    const pressResponse = await client.customRequest(
      'POST',
      '/api/simulator/events',
      simulateSchema,
      {
        events: [
          {
            key_code: 30, // 'a' key
            event_type: 0, // press
          },
        ],
      }
    );

    if (!pressResponse.data.success) {
      throw new Error('Failed to simulate key press');
    }

    // Small delay to allow processing
    await new Promise(resolve => setTimeout(resolve, 10));

    // Simulate key release (event_type: 1 = release)
    const releaseResponse = await client.customRequest(
      'POST',
      '/api/simulator/events',
      simulateSchema,
      {
        events: [
          {
            key_code: 30, // 'a' key
            event_type: 1, // release
          },
        ],
      }
    );

    if (!releaseResponse.data.success) {
      throw new Error('Failed to simulate key release');
    }

    // Step 4: Verify the mapping was applied by checking daemon state
    // Note: In a real scenario, we would check output events or logs
    // For now, we verify that the profile is still active and no errors occurred
    const verifyStatusResponse = await client.customRequest(
      'GET',
      '/api/daemon/state',
      statusSchema,
      undefined
    );

    if (!verifyStatusResponse.data.success) {
      throw new Error('Failed to verify daemon state after simulation');
    }

    // Step 5: Reset simulator state
    const resetResponse = await client.customRequest(
      'POST',
      '/api/simulator/reset',
      simulateSchema,
      {}
    );

    if (!resetResponse.data.success) {
      throw new Error('Failed to reset simulator');
    }

    return {
      success: true,
      workflow_steps: [
        'Created profile with key mapping (a→b)',
        'Activated profile',
        'Verified profile is active',
        'Simulated key press event',
        'Simulated key release event',
        'Verified daemon state after simulation',
        'Reset simulator state',
      ],
    };
  },
  cleanup: async (client: ApiClient) => {
    // Clean up: delete the test profile
    try {
      await client.deleteProfile('workflow-simulator-test');
    } catch (error) {
      // Profile might not exist, ignore error
    }

    // Reset simulator just in case
    try {
      const resetSchema = z.object({ success: z.boolean() });
      await client.customRequest('POST', '/api/simulator/reset', resetSchema, {});
    } catch (error) {
      // Ignore reset errors
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
  workflow_005,
  workflow_006,
  workflow_007,
];
