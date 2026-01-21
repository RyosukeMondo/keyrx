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
 * All workflow test cases
 */
export const workflowTestCases: TestCase[] = [
  workflow_002,
];
