/**
 * Example: Multi-Step Workflow Test
 *
 * This example demonstrates how to test complex workflows that involve
 * multiple API calls and state changes.
 *
 * Use this pattern for:
 * - Feature workflows (user journeys)
 * - Multi-step operations
 * - State transition tests
 * - Integration scenarios
 */

import { TestCase } from '../test-executor/types.js';
import { ApiClient } from '../api-client/client.js';
import { ResponseComparator } from '../comparator/response-comparator.js';

/**
 * Example 1: Profile Lifecycle Workflow
 *
 * Tests a complete profile workflow:
 * Create → Duplicate → Rename → Activate → Verify → Delete
 */
export function getProfileLifecycleWorkflow(): TestCase {
  return {
    id: 'example-workflow-001',
    name: 'Profile Lifecycle Workflow - create, duplicate, rename, activate',
    endpoint: 'multiple',
    scenario: 'profile_lifecycle',
    category: 'workflow',
    priority: 2,

    // Setup: ensure clean state
    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Clean up any leftover test profiles
      const testProfiles = [
        'workflow-original',
        'workflow-copy',
        'workflow-renamed',
      ];

      for (const profileName of testProfiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Profile doesn't exist, that's fine
        }
      }
    },

    // Execute: run the workflow
    execute: async (client: ApiClient) => {
      const results = [];

      // Step 1: Create original profile
      console.log('Step 1: Creating original profile...');
      const createResult = await client.createProfile('workflow-original', {
        template: 'basic',
      });
      results.push({
        step: 'create',
        success: createResult.success,
        profile: createResult.profile?.name,
      });

      // Step 2: Duplicate the profile
      console.log('Step 2: Duplicating profile...');
      const duplicateResult = await client.duplicateProfile(
        'workflow-original',
        'workflow-copy'
      );
      results.push({
        step: 'duplicate',
        success: duplicateResult.success,
        newProfile: duplicateResult.profile?.name,
      });

      // Step 3: Rename the copy
      console.log('Step 3: Renaming copy...');
      const renameResult = await client.renameProfile(
        'workflow-copy',
        'workflow-renamed'
      );
      results.push({
        step: 'rename',
        success: renameResult.success,
        newName: 'workflow-renamed',
      });

      // Step 4: Activate the renamed profile
      console.log('Step 4: Activating renamed profile...');
      const activateResult = await client.activateProfile('workflow-renamed');
      results.push({
        step: 'activate',
        success: activateResult.success,
      });

      // Step 5: Verify active profile
      console.log('Step 5: Verifying active profile...');
      const activeProfile = await client.getActiveProfile();
      results.push({
        step: 'verify',
        activeProfile: activeProfile.name,
        isCorrect: activeProfile.name === 'workflow-renamed',
      });

      // Step 6: List all profiles
      console.log('Step 6: Listing profiles...');
      const profiles = await client.getProfiles();
      results.push({
        step: 'list',
        profileCount: profiles.length,
        hasOriginal: profiles.some((p) => p.name === 'workflow-original'),
        hasRenamed: profiles.some((p) => p.name === 'workflow-renamed'),
      });

      return {
        success: true,
        steps: results,
      };
    },

    // Assert: validate all steps succeeded
    assert: (response, expected) => {
      const steps = response.steps;

      // Validate each step
      const allStepsSucceeded = steps.every((step) => {
        if (step.step === 'verify') {
          return step.isCorrect === true;
        }
        if (step.step === 'list') {
          return step.hasOriginal && step.hasRenamed;
        }
        return step.success === true;
      });

      return allStepsSucceeded;
    },

    // Cleanup: remove all test profiles
    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      const testProfiles = [
        'workflow-original',
        'workflow-copy',
        'workflow-renamed',
      ];

      for (const profileName of testProfiles) {
        try {
          await client.deleteProfile(profileName);
        } catch (error) {
          // Log but don't fail cleanup
          if (error.status !== 404) {
            console.warn(`Failed to delete ${profileName}:`, error.message);
          }
        }
      }
    },
  };
}

/**
 * Example 2: Error Recovery Workflow
 *
 * Tests error detection and recovery:
 * Create invalid → Validate (fail) → Fix → Validate (pass) → Use
 */
export function getErrorRecoveryWorkflow(): TestCase {
  return {
    id: 'example-workflow-002',
    name: 'Error Recovery Workflow - detect, fix, retry',
    endpoint: 'multiple',
    scenario: 'error_recovery',
    category: 'workflow',
    priority: 2,

    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('workflow-error-recovery');
      } catch {
        // Doesn't exist, good
      }
    },

    execute: async (client: ApiClient) => {
      const results = [];

      // Step 1: Create profile with invalid syntax
      console.log('Step 1: Creating profile with invalid syntax...');
      const invalidConfig = `
        // Invalid: syntax error
        map_key("a" tap("b"));  // Missing comma
      `;

      await client.createProfile('workflow-error-recovery', {
        template: 'blank',
      });
      await client.updateProfile('workflow-error-recovery', {
        source: invalidConfig,
      });

      // Step 2: Validate (should fail)
      console.log('Step 2: Validating invalid profile...');
      const validation1 = await client.validateProfile('workflow-error-recovery');
      results.push({
        step: 'validate_invalid',
        valid: validation1.valid,
        errorCount: validation1.errors?.length || 0,
      });

      // Step 3: Fix the syntax error
      console.log('Step 3: Fixing syntax error...');
      const validConfig = `
        // Fixed: correct syntax
        map_key("a", tap("b"));
      `;

      await client.updateProfile('workflow-error-recovery', {
        source: validConfig,
      });
      results.push({
        step: 'fix',
        success: true,
      });

      // Step 4: Validate again (should pass)
      console.log('Step 4: Validating fixed profile...');
      const validation2 = await client.validateProfile('workflow-error-recovery');
      results.push({
        step: 'validate_fixed',
        valid: validation2.valid,
        errorCount: validation2.errors?.length || 0,
      });

      // Step 5: Activate the fixed profile
      console.log('Step 5: Activating fixed profile...');
      const activateResult = await client.activateProfile(
        'workflow-error-recovery'
      );
      results.push({
        step: 'activate',
        success: activateResult.success,
      });

      return {
        success: true,
        steps: results,
      };
    },

    assert: (response, expected) => {
      const steps = response.steps;

      // Step 1: validation should fail
      const invalidStep = steps.find((s) => s.step === 'validate_invalid');
      if (invalidStep?.valid !== false) {
        return false;
      }

      // Step 2: fix should succeed
      const fixStep = steps.find((s) => s.step === 'fix');
      if (fixStep?.success !== true) {
        return false;
      }

      // Step 3: validation should pass
      const validStep = steps.find((s) => s.step === 'validate_fixed');
      if (validStep?.valid !== true) {
        return false;
      }

      // Step 4: activation should succeed
      const activateStep = steps.find((s) => s.step === 'activate');
      if (activateStep?.success !== true) {
        return false;
      }

      return true;
    },

    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('workflow-error-recovery');
      } catch (error) {
        if (error.status !== 404) {
          console.warn('Cleanup warning:', error.message);
        }
      }
    },
  };
}

/**
 * Example 3: State Transition Workflow
 *
 * Tests device state changes:
 * List → Rename → Change Layout → Disable → Verify → Re-enable
 */
export function getStateTransitionWorkflow(): TestCase {
  return {
    id: 'example-workflow-003',
    name: 'Device State Transition Workflow',
    endpoint: 'multiple',
    scenario: 'device_state',
    category: 'workflow',
    priority: 2,

    execute: async (client: ApiClient) => {
      const results = [];

      // Step 1: List devices
      console.log('Step 1: Listing devices...');
      const devices = await client.getDevices();
      results.push({
        step: 'list',
        deviceCount: devices.length,
      });

      if (devices.length === 0) {
        return {
          success: false,
          error: 'No devices available',
          steps: results,
        };
      }

      const deviceId = devices[0].id;
      const originalName = devices[0].name;
      const originalEnabled = devices[0].enabled;

      // Step 2: Rename device
      console.log('Step 2: Renaming device...');
      await client.renameDevice(deviceId, 'workflow-test-device');
      const renamed = await client.getDevices();
      const renamedDevice = renamed.find((d) => d.id === deviceId);
      results.push({
        step: 'rename',
        newName: renamedDevice?.name,
        success: renamedDevice?.name === 'workflow-test-device',
      });

      // Step 3: Change layout
      console.log('Step 3: Changing layout...');
      await client.setDeviceLayout(deviceId, 'ansi-104');
      const layout = await client.getDeviceLayout(deviceId);
      results.push({
        step: 'layout',
        layout: layout,
        success: layout === 'ansi-104',
      });

      // Step 4: Disable device
      console.log('Step 4: Disabling device...');
      await client.patchDevice(deviceId, { enabled: false });
      const disabled = await client.getDevices();
      const disabledDevice = disabled.find((d) => d.id === deviceId);
      results.push({
        step: 'disable',
        enabled: disabledDevice?.enabled,
        success: disabledDevice?.enabled === false,
      });

      // Step 5: Restore original state
      console.log('Step 5: Restoring original state...');
      await client.renameDevice(deviceId, originalName);
      await client.patchDevice(deviceId, { enabled: originalEnabled });
      results.push({
        step: 'restore',
        success: true,
      });

      return {
        success: true,
        steps: results,
      };
    },

    assert: (response, expected) => {
      if (!response.success) {
        return false;
      }

      // All steps should succeed
      return response.steps.every((step) => {
        return step.success !== false;
      });
    },
  };
}

/**
 * Key Points for Workflow Tests:
 *
 * 1. Structure:
 *    - Break workflow into clear steps
 *    - Log each step for debugging
 *    - Return intermediate results
 *    - Handle errors gracefully
 *
 * 2. Validation:
 *    - Validate each step's result
 *    - Don't just check final state
 *    - Test both happy path and error cases
 *
 * 3. Cleanup:
 *    - Restore original state when possible
 *    - Clean up all created resources
 *    - Handle cleanup failures gracefully
 *
 * 4. Debugging:
 *    - Include step names in results
 *    - Log progress with console.log
 *    - Return detailed results for failures
 *
 * 5. Best Practices:
 *    - Keep workflows focused (5-7 steps max)
 *    - Test real user scenarios
 *    - Make workflows independent
 *    - Don't rely on other tests
 */

/**
 * Common Workflow Patterns:
 *
 * 1. Create → Configure → Use → Verify → Cleanup
 * 2. Create → Validate (fail) → Fix → Validate (pass) → Use
 * 3. Get State → Modify → Verify → Restore
 * 4. Setup → Execute → Check Events → Verify → Cleanup
 * 5. Create Multiple → Link → Verify Links → Delete All
 */
