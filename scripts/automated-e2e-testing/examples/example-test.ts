/**
 * Example Test Case: Profile Creation and Activation
 *
 * This file demonstrates best practices for writing E2E API tests.
 * It shows the complete lifecycle: setup, execute, assert, and cleanup.
 *
 * Key concepts covered:
 * - Test isolation (cleanup before and after)
 * - Type safety with ApiClient
 * - Error handling
 * - Response validation with comparators
 * - Deterministic test data
 */

import type { TestCase } from '../../test-cases/types.js';
import type { ApiClient } from '../../api-client/client.js';
import { ResponseComparator } from '../../comparator/response-comparator.js';

/**
 * Example: Simple profile creation test
 *
 * This test demonstrates the minimal structure of a test case.
 */
export const simpleProfileTest: TestCase = {
  // Unique identifier for this test
  id: 'example-simple-001',

  // Descriptive name shown in reports
  name: 'Example: Create profile with default settings',

  // API endpoint being tested
  endpoint: '/api/profiles',

  // Specific scenario within this endpoint
  scenario: 'create_default',

  // Category for organizing tests
  category: 'profiles',

  // Priority: 1 (high) = critical path, 3 (low) = nice to have
  priority: 2,

  // Setup: Ensure clean state before test
  // This runs BEFORE execute()
  setup: async () => {
    // BEST PRACTICE: Always clean up potential leftovers from previous runs
    const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

    try {
      await client.deleteProfile('example-profile');
    } catch (error) {
      // It's OK if profile doesn't exist
      // We just want to ensure it's not there
    }
  },

  // Execute: Make the API call we're testing
  // This is the main test action
  execute: async (client: ApiClient) => {
    // BEST PRACTICE: Use the typed API client for type safety
    return await client.createProfile('example-profile', {
      description: 'Example profile for testing',
      mappings: [],
    });
  },

  // Assert: Validate the response
  // Compare actual response with expected results
  assert: (response, expected) => {
    // BEST PRACTICE: Use ResponseComparator for robust comparison
    const comparator = new ResponseComparator();

    // Ignore dynamic fields that change on each run
    return comparator.compare(response, expected, {
      ignoreFields: ['createdAt', 'id', 'timestamp'],
    });
  },

  // Cleanup: Remove test data
  // This runs AFTER execute(), even if the test fails
  cleanup: async () => {
    const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

    try {
      await client.deleteProfile('example-profile');
    } catch (error) {
      // Log cleanup failures but don't fail the test
      console.warn(`Cleanup failed for example-profile: ${error.message}`);
    }
  },
};

/**
 * Example: Complex multi-step workflow test
 *
 * This test demonstrates:
 * - Multiple API calls in sequence
 * - State verification
 * - Error handling
 * - Complex assertions
 */
export const complexWorkflowTest: TestCase = {
  id: 'example-complex-001',
  name: 'Example: Complete profile workflow (create → activate → verify)',
  endpoint: '/api/profiles',
  scenario: 'workflow_complete',
  category: 'profiles',
  priority: 1,

  setup: async () => {
    const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

    // Clean up test profile if it exists
    try {
      await client.deleteProfile('workflow-test-profile');
    } catch {
      // Profile doesn't exist, that's fine
    }

    // BEST PRACTICE: Verify preconditions
    const status = await client.getStatus();
    if (status.activeProfile === 'workflow-test-profile') {
      throw new Error('Test profile should not be active before test');
    }
  },

  execute: async (client: ApiClient) => {
    // Step 1: Create a new profile
    const createResult = await client.createProfile('workflow-test-profile', {
      description: 'Workflow test profile',
      mappings: [
        {
          from: 'CapsLock',
          to: 'Escape',
        },
      ],
    });

    // BEST PRACTICE: Validate intermediate results
    if (!createResult || !createResult.name) {
      throw new Error('Profile creation failed');
    }

    // Step 2: Activate the profile
    await client.activateProfile('workflow-test-profile');

    // Step 3: Verify the profile is active
    const status = await client.getStatus();

    // Return all relevant data for assertion
    return {
      created: createResult,
      activeProfile: status.activeProfile,
      isActive: status.activeProfile === 'workflow-test-profile',
    };
  },

  assert: (response: any, expected: any) => {
    const comparator = new ResponseComparator();

    // Complex assertion: verify multiple conditions
    const result = comparator.compare(response, expected, {
      ignoreFields: ['created.id', 'created.createdAt'],
    });

    // Additional custom assertions
    if (!result.matches) {
      return result;
    }

    // BEST PRACTICE: Add semantic checks beyond simple equality
    if (!response.isActive) {
      return {
        matches: false,
        diff: [
          {
            path: 'isActive',
            expected: true,
            actual: response.isActive,
          },
        ],
        ignoredFields: [],
      };
    }

    return { matches: true, diff: [], ignoredFields: [] };
  },

  cleanup: async () => {
    const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

    // BEST PRACTICE: Clean up in reverse order of creation
    try {
      // First, deactivate if active
      const status = await client.getStatus();
      if (status.activeProfile === 'workflow-test-profile') {
        await client.deactivateProfile();
      }
    } catch (error) {
      console.warn('Failed to deactivate profile:', error.message);
    }

    try {
      // Then delete the profile
      await client.deleteProfile('workflow-test-profile');
    } catch (error) {
      console.warn('Failed to delete profile:', error.message);
    }
  },
};

/**
 * Example: Error handling test
 *
 * This test demonstrates how to test error conditions.
 */
export const errorHandlingTest: TestCase = {
  id: 'example-error-001',
  name: 'Example: Profile not found returns 404',
  endpoint: '/api/profiles/:name',
  scenario: 'not_found',
  category: 'profiles',
  priority: 2,

  execute: async (client: ApiClient) => {
    // BEST PRACTICE: Use try-catch for expected errors
    try {
      await client.getProfile('nonexistent-profile-12345');

      // If we get here, the test should fail
      throw new Error('Expected 404 error but request succeeded');
    } catch (error: any) {
      // Return error details for assertion
      return {
        status: error.status || error.statusCode || 500,
        message: error.message,
        type: error.name || 'Error',
      };
    }
  },

  assert: (response: any, expected: any) => {
    // BEST PRACTICE: Verify error status codes and messages
    if (response.status !== expected.status) {
      return {
        matches: false,
        diff: [
          {
            path: 'status',
            expected: expected.status,
            actual: response.status,
          },
        ],
        ignoredFields: [],
      };
    }

    return { matches: true, diff: [], ignoredFields: [] };
  },

  // No cleanup needed for error tests
};

/**
 * Export all example tests
 */
export const exampleTests: TestCase[] = [
  simpleProfileTest,
  complexWorkflowTest,
  errorHandlingTest,
];
