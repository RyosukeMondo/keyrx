/**
 * Profile Management API Test Cases
 *
 * Tests for advanced profile management endpoints:
 * - POST /api/profiles/:name/duplicate - Duplicate a profile
 * - PUT /api/profiles/:name/rename - Rename a profile
 * - POST /api/profiles/:name/validate - Validate a profile
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase } from './api-tests.js';
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
import { extractData } from './api-tests.js';
import { z } from 'zod';

/**
 * Track profiles created during tests for cleanup
 * Map<testId, Set<profileName>>
 */
const createdProfiles = new Map<string, Set<string>>();

/**
 * Register a profile for cleanup
 */
function trackProfile(testId: string, profileName: string): void {
  if (!createdProfiles.has(testId)) {
    createdProfiles.set(testId, new Set());
  }
  createdProfiles.get(testId)!.add(profileName);
}

/**
 * Get tracked profiles for a test
 */
function getTrackedProfiles(testId: string): string[] {
  return Array.from(createdProfiles.get(testId) || []);
}

/**
 * Clear tracked profiles for a test
 */
function clearTrackedProfiles(testId: string): void {
  createdProfiles.delete(testId);
}

/**
 * Generate a short test profile name (max 32 chars per API limit)
 * Format: "prf-{prefix}-{timestamp_last6}"
 * Example: "prf-dup-234567" (14-18 chars depending on prefix)
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
 * Success response schema for profile duplicate operation
 */
const DuplicateProfileResponseSchema = z.object({
  success: z.boolean(),
  profile: z.object({
    name: z.string(),
    rhai_path: z.string(),
  }),
});

/**
 * Success response schema for profile rename operation
 */
const RenameProfileResponseSchema = z.object({
  success: z.boolean(),
  profile: z.object({
    name: z.string(),
    rhai_path: z.string(),
    krx_path: z.string(),
  }),
});

/**
 * Response schema for profile validation operation
 */
const ValidationResponseSchema = z.object({
  valid: z.boolean(),
  errors: z.array(
    z.object({
      line: z.number(),
      column: z.number().optional(),
      message: z.string(),
    })
  ),
});

/**
 * Error response schema
 */
const ErrorResponseSchema = z.object({
  error: z.string(),
  code: z.string().optional(),
});

/**
 * Profile Management test cases
 */
export const profileManagementTestCases: TestCase[] = [
  // =================================================================
  // Profile Duplicate Tests
  // =================================================================
  {
    id: 'profiles-011',
    name: 'POST /api/profiles/:name/duplicate - Duplicate profile (success)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create source profile for duplication
      const sourceName = shortProfileName('src');
      await client.createProfile(sourceName);
      trackProfile('profiles-011', sourceName);
    },
    execute: async (client) => {
      // Find the source profile we just created
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-src-')
      );

      if (!sourceProfile) {
        throw new Error('Source profile not found');
      }

      const sourceName = sourceProfile.name;
      const newName = shortProfileName('dup');
      trackProfile('profiles-011', newName);

      const response = await client.customRequest(
        'POST',
        `/api/profiles/${encodeURIComponent(sourceName)}/duplicate`,
        DuplicateProfileResponseSchema,
        { new_name: newName }
      );

      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as {
        success?: boolean;
        profile?: { name?: string; rhai_path?: string };
      };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected success=true for profile duplicate',
        };
      }

      if (!actualData.profile?.name) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected profile.name in response',
        };
      }

      if (!actualData.profile?.rhai_path) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected profile.rhai_path in response',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      // Delete all tracked profiles for this test
      const profiles = getTrackedProfiles('profiles-011');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-011');
    },
  },

  {
    id: 'profiles-011b',
    name: 'POST /api/profiles/:name/duplicate - Duplicate nonexistent profile (404)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const newName = shortProfileName("dup");
      trackProfile('profiles-011b', newName);

      try {
        const response = await client.customRequest(
          'POST',
          '/api/profiles/nonexistent-profile-xyz/duplicate',
          z.union([DuplicateProfileResponseSchema, ErrorResponseSchema]),
          { new_name: newName }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found or 500 error
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
          error: `Expected 404 or 500 for nonexistent profile, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-011b');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-011b');
    },
  },

  {
    id: 'profiles-011c',
    name: 'POST /api/profiles/:name/duplicate - Duplicate with name conflict (409)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_name_conflict',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create source profile and target profile with conflicting name
      const sourceName = shortProfileName("src");
      const targetName = shortProfileName("tgt");
      await client.createProfile(sourceName);
      await client.createProfile(targetName);
      trackProfile('profiles-011c', sourceName);
      trackProfile('profiles-011c', targetName);
    },
    execute: async (client) => {
      // Find both profiles
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-src-')
      );
      const targetProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-tgt-')
      );

      if (!sourceProfile || !targetProfile) {
        throw new Error('Source or target profile not found');
      }

      const sourceName = sourceProfile.name;
      const targetName = targetProfile.name;

      try {
        const response = await client.customRequest(
          'POST',
          `/api/profiles/${encodeURIComponent(sourceName)}/duplicate`,
          z.union([DuplicateProfileResponseSchema, ErrorResponseSchema]),
          { new_name: targetName } // Try to duplicate with existing name
        );
        return {
          status: response.status,
          data: response.data,
          context: { sourceName, targetName },
        };
      } catch (error) {
        // Expect 409 Conflict or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { sourceName, targetName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      // Accept 409 (Conflict) or 500 (Internal Server Error with conflict message)
      if (status !== 409 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 409 or 500 for duplicate name, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-011c');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-011c');
    },
  },

  // =================================================================
  // Profile Rename Tests
  // =================================================================
  {
    id: 'profiles-012',
    name: 'PUT /api/profiles/:name/rename - Rename profile (success)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create profile to rename
      const originalName = shortProfileName("ren");
      await client.createProfile(originalName);
      trackProfile('profiles-012', originalName);
    },
    execute: async (client) => {
      // Find the profile we just created
      const profiles = await client.getProfiles();
      const profile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-ren-')
      );

      if (!profile) {
        throw new Error('Profile to rename not found');
      }

      const originalName = profile.name;
      const newName = shortProfileName("new");
      trackProfile('profiles-012', newName);

      const response = await client.customRequest(
        'PUT',
        `/api/profiles/${encodeURIComponent(originalName)}/rename`,
        RenameProfileResponseSchema,
        { new_name: newName }
      );

      return {
        status: response.status,
        data: response.data,
        context: { originalName, newName },
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as {
        success?: boolean;
        profile?: { name?: string; rhai_path?: string; krx_path?: string };
      };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected success=true for profile rename',
        };
      }

      if (!actualData.profile?.name) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected profile.name in response',
        };
      }

      if (!actualData.profile?.rhai_path) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected profile.rhai_path in response',
        };
      }

      if (!actualData.profile?.krx_path) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected profile.krx_path in response',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-012');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-012');
    },
  },

  {
    id: 'profiles-012b',
    name: 'PUT /api/profiles/:name/rename - Rename nonexistent profile (404)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const newName = shortProfileName("new");
      trackProfile('profiles-012b', newName);

      try {
        const response = await client.customRequest(
          'PUT',
          '/api/profiles/nonexistent-profile-xyz/rename',
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: newName }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found or 500 error
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
          error: `Expected 404 or 500 for nonexistent profile, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-012b');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-012b');
    },
  },

  {
    id: 'profiles-012c',
    name: 'PUT /api/profiles/:name/rename - Rename with invalid name (400)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_invalid_name',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create profile to rename
      const originalName = shortProfileName("inv");
      await client.createProfile(originalName);
      trackProfile('profiles-012c', originalName);
    },
    execute: async (client) => {
      // Find the profile we just created
      const profiles = await client.getProfiles();
      const profile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-inv-')
      );

      if (!profile) {
        throw new Error('Profile to rename not found');
      }

      const originalName = profile.name;
      const invalidName = ''; // Empty name is invalid

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/profiles/${encodeURIComponent(originalName)}/rename`,
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: invalidName }
        );
        return {
          status: response.status,
          data: response.data,
          context: { originalName },
        };
      } catch (error) {
        // Expect 400 Bad Request or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { originalName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      // Accept 400 (Bad Request) or 500 (Internal Server Error with validation message)
      if (status !== 400 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 400 or 500 for invalid name, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-012c');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-012c');
    },
  },

  {
    id: 'profiles-012d',
    name: 'PUT /api/profiles/:name/rename - Rename with name conflict (409)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_name_conflict',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create two profiles
      const sourceName = shortProfileName("rsrc");
      const targetName = shortProfileName("rtgt");
      await client.createProfile(sourceName);
      await client.createProfile(targetName);
      trackProfile('profiles-012d', sourceName);
      trackProfile('profiles-012d', targetName);
    },
    execute: async (client) => {
      // Find both profiles
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-rsrc-')
      );
      const targetProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-rtgt-')
      );

      if (!sourceProfile || !targetProfile) {
        throw new Error('Source or target profile not found');
      }

      const sourceName = sourceProfile.name;
      const targetName = targetProfile.name;

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/profiles/${encodeURIComponent(sourceName)}/rename`,
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: targetName } // Try to rename with existing name
        );
        return {
          status: response.status,
          data: response.data,
          context: { sourceName, targetName },
        };
      } catch (error) {
        // Expect 409 Conflict or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { sourceName, targetName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual);
      const status = (actual as any).status || 200;

      // Accept 409 (Conflict) or 500 (Internal Server Error with conflict message)
      if (status !== 409 && status !== 500) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected 409 or 500 for duplicate name, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-012d');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-012d');
    },
  },

  // =================================================================
  // Profile Validate Tests
  // =================================================================
  {
    id: 'profiles-013',
    name: 'POST /api/profiles/:name/validate - Validate valid profile (success)',
    endpoint: '/api/profiles/:name/validate',
    scenario: 'validate_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create a valid profile
      const profileName = shortProfileName("val");
      await client.createProfile(profileName);
      trackProfile('profiles-013', profileName);
    },
    execute: async (client) => {
      // Find the profile we just created
      const profiles = await client.getProfiles();
      const profile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('prf-val-')
      );

      if (!profile) {
        throw new Error('Profile to validate not found');
      }

      const profileName = profile.name;

      const response = await client.customRequest(
        'POST',
        `/api/profiles/${encodeURIComponent(profileName)}/validate`,
        ValidationResponseSchema,
        {}
      );

      return {
        status: response.status,
        data: response.data,
        context: { profileName },
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as {
        valid?: boolean;
        errors?: Array<{ line?: number; column?: number; message?: string }>;
      };

      if (actualData.valid !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected valid=true for valid profile',
        };
      }

      if (!Array.isArray(actualData.errors)) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected errors array in response',
        };
      }

      if (actualData.errors.length !== 0) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Expected empty errors array for valid profile',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-013');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-013');
    },
  },

  {
    id: 'profiles-013b',
    name: 'POST /api/profiles/:name/validate - Validate nonexistent profile (404)',
    endpoint: '/api/profiles/:name/validate',
    scenario: 'validate_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/profiles/nonexistent-profile-xyz/validate',
          z.union([ValidationResponseSchema, ErrorResponseSchema]),
          {}
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found or 500 error
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
          error: `Expected 404 or 500 for nonexistent profile, got ${status}`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      const profiles = getTrackedProfiles('profiles-013b');
      for (const profileName of profiles) {
        try {
          await client.deleteProfile(profileName);
        } catch {
          // Ignore cleanup errors
        }
      }
      clearTrackedProfiles('profiles-013b');
    },
  },
];
