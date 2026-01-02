/**
 * Profile management API client
 */

import { apiClient } from './client';
import type { ProfileMetadata, Template, ActivationResult } from '../types';

interface CreateProfileRequest {
  name: string;
  template: Template;
}

interface ProfileResponse {
  success: boolean;
}

/**
 * Fetch all profiles
 */
export async function fetchProfiles(): Promise<ProfileMetadata[]> {
  const response = await apiClient.get<{ profiles: ProfileMetadata[] }>('/api/profiles');
  return response.profiles;
}

/**
 * Create a new profile
 */
export async function createProfile(
  name: string,
  template: Template
): Promise<ProfileResponse> {
  const request: CreateProfileRequest = { name, template };
  return apiClient.post<ProfileResponse>('/api/profiles', request);
}

/**
 * Activate a profile
 */
export async function activateProfile(
  name: string
): Promise<ActivationResult> {
  return apiClient.post<ActivationResult>(`/api/profiles/${name}/activate`);
}

/**
 * Delete a profile
 */
export async function deleteProfile(name: string): Promise<ProfileResponse> {
  return apiClient.delete<ProfileResponse>(`/api/profiles/${name}`);
}

/**
 * Duplicate a profile
 */
export async function duplicateProfile(
  sourceName: string,
  newName: string
): Promise<ProfileResponse> {
  return apiClient.post<ProfileResponse>(
    `/api/profiles/${sourceName}/duplicate`,
    { newName }
  );
}
