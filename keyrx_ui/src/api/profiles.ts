/**
 * Profile management API client
 */

import { apiClient } from './client';
import {
  validateApiResponse,
  ProfileListResponseSchema,
  ProfileRpcInfoSchema,
  ActivationRpcResultSchema,
  type ProfileRpcInfo,
  type ActivationRpcResult,
  type ValidationResult,
} from './schemas';
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
  const response = await apiClient.post<ProfileRpcInfo>(
    '/api/profiles',
    request
  );
  // Validate the returned profile info
  validateApiResponse(ProfileRpcInfoSchema, response, 'POST /api/profiles');
  return { success: true };
}

/**
 * Activate a profile
 */
export async function activateProfile(name: string): Promise<ActivationResult> {
  const response = await apiClient.post<ActivationRpcResult>(
    `/api/profiles/${name}/activate`
  );
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
  await apiClient.put<EmptyResponse>(`/api/profiles/${originalName}`, updates);
  return { success: true };
}

/**
 * Delete a profile
 */
export async function deleteProfile(name: string): Promise<ProfileResponse> {
  await apiClient.delete<EmptyResponse>(`/api/profiles/${name}`);
  return { success: true };
}

/**
 * Duplicate a profile
 */
export async function duplicateProfile(
  sourceName: string,
  newName: string
): Promise<ProfileResponse> {
  const response = await apiClient.post<ProfileRpcInfo>(
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
 * Validate profile configuration
 */
export async function validateConfig(
  config: string
): Promise<ValidationResult> {
  return apiClient.post<ValidationResult>('/api/profiles/validate', { config });
}
