/**
 * Profile management API client
 */

import { apiClient } from './client';
import {
  validateApiResponse,
  ProfileListResponseSchema,
  ProfileRpcInfoSchema,
  ActivationRpcResultSchema,
} from './schemas';
import type { z } from 'zod';
import type { ProfileMetadata, Template, ActivationResult } from '../types';

interface CreateProfileRequest {
  name: string;
  template: Template;
}

interface ProfileResponse {
  success: boolean;
}

interface EmptyResponse {
  [key: string]: unknown;
}

/**
 * Fetch active profile name
 * Returns the currently active profile name or null if none
 */
export async function fetchActiveProfile(): Promise<string | null> {
  const response = await apiClient.get<{ active_profile: string | null }>(
    '/api/profiles/active'
  );
  return response.active_profile;
}

/**
 * Fetch all profiles
 */
export async function fetchProfiles(): Promise<ProfileMetadata[]> {
  const response = await apiClient.get<{ profiles: ProfileMetadata[] }>(
    '/api/profiles'
  );
  const validated = validateApiResponse(
    ProfileListResponseSchema,
    response,
    'GET /api/profiles'
  );

  // Map response to ProfileMetadata format
  return validated.profiles.map((p) => ({
    name: p.name,
    rhaiPath: p.rhaiPath,
    krxPath: p.krxPath,
    createdAt: p.createdAt,
    modifiedAt: p.modifiedAt,
    deviceCount: p.deviceCount,
    keyCount: p.keyCount,
    isActive: p.isActive,
  }));
}

/**
 * Create a new profile
 */
export async function createProfile(
  name: string,
  template: Template
): Promise<ProfileResponse> {
  const request: CreateProfileRequest = { name, template };
  const response = await apiClient.post<z.infer<typeof ProfileRpcInfoSchema>>('/api/profiles', request);
  // Validate the returned profile info
  validateApiResponse(ProfileRpcInfoSchema, response, 'POST /api/profiles');
  return { success: true };
}

/**
 * Activate a profile
 */
export async function activateProfile(name: string): Promise<ActivationResult> {
  const response = await apiClient.post<z.infer<typeof ActivationRpcResultSchema>>(`/api/profiles/${name}/activate`);
  const validated = validateApiResponse(
    ActivationRpcResultSchema,
    response,
    `POST /api/profiles/${name}/activate`
  );

  // Map RPC activation result to ActivationResult format
  return {
    success: validated.success,
    profile: name,
    compiledSize: 0, // RPC doesn't provide this, use placeholder
    compileTimeMs: validated.compile_time_ms,
    errors: validated.error ? [validated.error] : [],
  };
}

/**
 * Update profile metadata (name, description)
 */
export async function updateProfile(
  originalName: string,
  updates: { name?: string; description?: string }
): Promise<ProfileResponse> {
  const response = await apiClient.put<EmptyResponse>(
    `/api/profiles/${originalName}`,
    updates
  );
  // Validate the response
  if (response && typeof response === 'object') {
    console.debug(
      JSON.stringify({
        timestamp: new Date().toISOString(),
        level: 'debug',
        service: 'API Validation',
        event: 'update_profile_success',
        context: { originalName, updates },
      })
    );
  }
  return { success: true };
}

/**
 * Delete a profile
 */
export async function deleteProfile(name: string): Promise<ProfileResponse> {
  const response = await apiClient.delete<EmptyResponse>(`/api/profiles/${name}`);
  // Validate the response - for delete, we expect either empty or success indicator
  // Since there's no specific schema for delete response, we'll just check it doesn't error
  if (response && typeof response === 'object') {
    console.debug(
      JSON.stringify({
        timestamp: new Date().toISOString(),
        level: 'debug',
        service: 'API Validation',
        event: 'delete_profile_success',
        context: { profileName: name },
      })
    );
  }
  return { success: true };
}

/**
 * Duplicate a profile
 */
export async function duplicateProfile(
  sourceName: string,
  newName: string
): Promise<ProfileResponse> {
  const response = await apiClient.post<z.infer<typeof ProfileRpcInfoSchema>>(
    `/api/profiles/${sourceName}/duplicate`,
    { newName }
  );
  // Validate the returned profile info
  validateApiResponse(
    ProfileRpcInfoSchema,
    response,
    `POST /api/profiles/${sourceName}/duplicate`
  );
  return { success: true };
}

/**
 * Validation error structure
 */
export interface ValidationError {
  line: number;
  column: number;
  length: number;
  message: string;
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

/**
 * Validate profile configuration
 */
export async function validateConfig(
  config: string
): Promise<ValidationResult> {
  return apiClient.post<ValidationResult>('/api/profiles/validate', { config });
}
