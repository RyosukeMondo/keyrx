/**
 * Example Test Case
 *
 * This file demonstrates how to write a complete E2E test case with all
 * best practices applied. Use this as a template for new tests.
 *
 * Test Scenario:
 *   1. Create a new profile with specific configuration
 *   2. Activate the profile
 *   3. Verify the profile is active via /api/status
 *   4. Clean up by deleting the profile
 *
 * This example demonstrates:
 *   - Proper setup/execute/assert/cleanup structure
 *   - Type-safe API interactions
 *   - Error handling
 *   - Test isolation
 *   - Detailed assertions with helpful error messages
 */

import { TestCase, ValidationResult } from '../../test-cases/test-utils';
import { ApiClient } from '../../api-client/client';

/**
 * Context type for this test
 * Using explicit types helps catch errors early
 */
interface ExampleTestContext {
  profileName: string;
  createdAt: number;
}

/**
 * Example Test: Profile Creation and Activation
 *
 * This test verifies the complete workflow of creating and activating a profile.
 */
export const exampleProfileTest: TestCase = {
  // Unique identifier - use descriptive kebab-case
  id: 'example-profile-create-activate',

  // Human-readable name - should clearly describe what's being tested
  name: 'Example: Create and activate a profile',

  // API endpoint being tested (primary endpoint)
  endpoint: '/api/profiles',

  // Scenario description - helps identify test category
  scenario: 'create-and-activate',

  /**
   * Setup Phase
   *
   * Prepares the test environment. This should:
   * 1. Clean up any existing test data (idempotent)
   * 2. Create necessary preconditions
   * 3. Return context object for later phases
   *
   * @param client - Type-safe API client
   * @returns Context object with test data
   */
  async setup(client: ApiClient): Promise<ExampleTestContext> {
    // Generate unique profile name to avoid conflicts
    // Using timestamp ensures uniqueness across test runs
    const profileName = `example-test-${Date.now()}`;

    console.log(`[Setup] Creating test profile: ${profileName}`);

    // Best Practice: Clean up any existing profile first (idempotent)
    // This ensures the test works even if previous run failed to clean up
    try {
      await client.deleteProfile(profileName);
      console.log(`[Setup] Cleaned up existing profile`);
    } catch (error) {
      // Profile doesn't exist - that's fine, we'll create it
      console.log(`[Setup] No existing profile to clean up`);
    }

    // Return context for use in execute and cleanup phases
    return {
      profileName,
      createdAt: Date.now(),
    };
  },

  /**
   * Execute Phase
   *
   * Performs the actual operation being tested. This should:
   * 1. Call the API endpoint(s)
   * 2. Capture the response
   * 3. Handle both success and error cases
   * 4. Return the actual result for assertion
   *
   * @param client - API client
   * @param context - Context from setup phase
   * @returns Actual result to be validated
   */
  async execute(
    client: ApiClient,
    context: ExampleTestContext
  ): Promise<any> {
    console.log(`[Execute] Creating profile: ${context.profileName}`);

    // Step 1: Create the profile
    const createResponse = await client.createProfile(context.profileName);

    console.log(`[Execute] Profile created with status: ${createResponse.status}`);

    // Step 2: Activate the profile
    console.log(`[Execute] Activating profile: ${context.profileName}`);
    const activateResponse = await client.activateProfile(context.profileName);

    console.log(`[Execute] Profile activated with status: ${activateResponse.status}`);

    // Step 3: Verify via status endpoint
    console.log(`[Execute] Fetching current status`);
    const statusResponse = await client.getStatus();

    // Return complete result for assertion phase
    return {
      createResponse,
      activateResponse,
      statusResponse,
      profileName: context.profileName,
    };
  },

  /**
   * Assert Phase
   *
   * Validates the actual result against expected behavior. This should:
   * 1. Check status codes
   * 2. Validate response structure
   * 3. Verify data correctness
   * 4. Return detailed validation result with helpful messages
   *
   * @param actual - Result from execute phase
   * @param expected - Expected values (can be null if using hardcoded expectations)
   * @returns Validation result with pass/fail and details
   */
  assert(actual: any, expected: any): ValidationResult {
    // Validate Step 1: Profile creation
    if (actual.createResponse.status !== 200) {
      return {
        passed: false,
        message: `Profile creation failed. Expected status 200, got ${actual.createResponse.status}`,
        diff: [
          {
            path: 'createResponse.status',
            expected: 200,
            actual: actual.createResponse.status,
          },
        ],
      };
    }

    // Validate response structure
    if (!actual.createResponse.data || !actual.createResponse.data.name) {
      return {
        passed: false,
        message: 'Profile creation response missing required fields (data.name)',
      };
    }

    // Validate profile name matches
    if (actual.createResponse.data.name !== actual.profileName) {
      return {
        passed: false,
        message: `Profile name mismatch. Expected '${actual.profileName}', got '${actual.createResponse.data.name}'`,
        diff: [
          {
            path: 'createResponse.data.name',
            expected: actual.profileName,
            actual: actual.createResponse.data.name,
          },
        ],
      };
    }

    // Validate Step 2: Profile activation
    if (actual.activateResponse.status !== 200) {
      return {
        passed: false,
        message: `Profile activation failed. Expected status 200, got ${actual.activateResponse.status}`,
        diff: [
          {
            path: 'activateResponse.status',
            expected: 200,
            actual: actual.activateResponse.status,
          },
        ],
      };
    }

    // Validate Step 3: Status reflects active profile
    if (actual.statusResponse.status !== 200) {
      return {
        passed: false,
        message: `Status check failed. Expected status 200, got ${actual.statusResponse.status}`,
      };
    }

    const status = actual.statusResponse.data;

    // Check if our profile is marked as active
    if (!status.activeProfile || status.activeProfile !== actual.profileName) {
      return {
        passed: false,
        message: `Profile not active in status. Expected '${actual.profileName}', got '${status.activeProfile}'`,
        diff: [
          {
            path: 'statusResponse.data.activeProfile',
            expected: actual.profileName,
            actual: status.activeProfile,
          },
        ],
      };
    }

    // All validations passed!
    return {
      passed: true,
      message: `Profile '${actual.profileName}' successfully created and activated`,
    };
  },

  /**
   * Cleanup Phase (Optional but Recommended)
   *
   * Removes test data and restores state. This should:
   * 1. Delete created resources
   * 2. Handle cleanup errors gracefully (best effort)
   * 3. Not throw errors (cleanup failures shouldn't fail the test)
   *
   * IMPORTANT: Cleanup always runs, even if test failed!
   *
   * @param client - API client
   * @param context - Context from setup phase
   */
  async cleanup(
    client: ApiClient,
    context: ExampleTestContext
  ): Promise<void> {
    console.log(`[Cleanup] Removing test profile: ${context.profileName}`);

    try {
      // Delete the test profile
      await client.deleteProfile(context.profileName);
      console.log(`[Cleanup] Profile deleted successfully`);
    } catch (error) {
      // Best Practice: Log but don't throw
      // Cleanup is best-effort; failures shouldn't break the test
      console.warn(
        `[Cleanup] Failed to delete profile: ${error instanceof Error ? error.message : String(error)}`
      );
    }

    // If we had activated the profile, we might want to restore default
    try {
      // Check if default profile exists and activate it
      const profiles = await client.getProfiles();
      const hasDefault = profiles.some((p: any) => p.name === 'default');

      if (hasDefault) {
        await client.activateProfile('default');
        console.log(`[Cleanup] Restored default profile`);
      }
    } catch (error) {
      console.warn(`[Cleanup] Failed to restore default profile: ${error}`);
    }
  },
};

/**
 * Example Test: Error Handling
 *
 * This test demonstrates how to test error cases (e.g., 404, 409, 400).
 * Some tests intentionally trigger errors to verify error handling.
 */
export const exampleErrorTest: TestCase = {
  id: 'example-profile-not-found',
  name: 'Example: Handle profile not found (404)',
  endpoint: '/api/profiles/:name',
  scenario: 'not-found',

  async setup(client: ApiClient): Promise<any> {
    // Generate a profile name that definitely doesn't exist
    const nonexistentProfile = `nonexistent-${Date.now()}`;

    // Ensure it really doesn't exist
    try {
      await client.deleteProfile(nonexistentProfile);
    } catch {
      // Expected - profile doesn't exist
    }

    return { profileName: nonexistentProfile };
  },

  async execute(client: ApiClient, context: any): Promise<any> {
    // Try to get a profile that doesn't exist
    try {
      const response = await client.getProfile(context.profileName);
      // If we get here, something's wrong (should have thrown 404)
      return {
        success: true,
        status: response.status,
        error: null,
      };
    } catch (error: any) {
      // Expected path - profile not found
      return {
        success: false,
        status: error.response?.status || 500,
        error: error.message,
      };
    }
  },

  assert(actual: any, expected: any): ValidationResult {
    // We expect the request to fail with 404
    if (actual.success) {
      return {
        passed: false,
        message: 'Expected 404 Not Found, but request succeeded',
      };
    }

    if (actual.status !== 404) {
      return {
        passed: false,
        message: `Expected 404 Not Found, got ${actual.status}`,
        diff: [
          {
            path: 'status',
            expected: 404,
            actual: actual.status,
          },
        ],
      };
    }

    return {
      passed: true,
      message: 'Correctly returned 404 for nonexistent profile',
    };
  },

  async cleanup(client: ApiClient, context: any): Promise<void> {
    // No cleanup needed - we didn't create anything
    console.log('[Cleanup] No cleanup needed for error test');
  },
};

/**
 * Example Test: Simple Case
 *
 * For simple tests that don't need complex setup/cleanup.
 */
export const exampleSimpleTest: TestCase = {
  id: 'example-status-check',
  name: 'Example: Simple status check',
  endpoint: '/api/status',
  scenario: 'healthy',

  // Minimal setup
  async setup(client: ApiClient): Promise<any> {
    return {}; // No setup needed
  },

  // Simple execution
  async execute(client: ApiClient, context: any): Promise<any> {
    return await client.getStatus();
  },

  // Simple assertion
  assert(actual: any, expected: any): ValidationResult {
    if (actual.status !== 200) {
      return {
        passed: false,
        message: `Expected 200 OK, got ${actual.status}`,
      };
    }

    if (!actual.data || !actual.data.status) {
      return {
        passed: false,
        message: 'Response missing status field',
      };
    }

    return { passed: true };
  },

  // No cleanup needed
  cleanup: undefined,
};

/**
 * Export all example tests
 *
 * You would typically add these to scripts/test-cases/api-tests.ts:
 *
 * import { exampleProfileTest, exampleErrorTest, exampleSimpleTest } from './examples/example-test';
 *
 * export function getAllTestCases(): TestCase[] {
 *   return [
 *     exampleProfileTest,
 *     exampleErrorTest,
 *     exampleSimpleTest,
 *     // ... other tests
 *   ];
 * }
 */
export const exampleTests = [
  exampleProfileTest,
  exampleErrorTest,
  exampleSimpleTest,
];

/**
 * Key Takeaways:
 *
 * 1. Structure: setup → execute → assert → cleanup
 * 2. Isolation: Each test creates its own data
 * 3. Cleanup: Always clean up, handle errors gracefully
 * 4. Type Safety: Use explicit types for context and results
 * 5. Error Messages: Provide helpful, actionable messages
 * 6. Logging: Log each phase for debugging
 * 7. Comments: Explain the "why", not just the "what"
 * 8. Best Effort: Cleanup shouldn't fail the test
 *
 * Anti-Patterns to Avoid:
 *
 * ❌ Hard-coded test data (use timestamps or UUIDs)
 * ❌ No cleanup (leaves test environment dirty)
 * ❌ Vague assertions ("test failed")
 * ❌ Tests that depend on other tests
 * ❌ Timing-dependent tests (use polling, not delays)
 * ❌ Silent error swallowing
 * ❌ Global state mutation
 *
 * For more details, see:
 * - DEV_GUIDE.md - Complete developer guide
 * - README.md - System overview
 * - scripts/test-cases/api-tests.ts - Real test examples
 */
