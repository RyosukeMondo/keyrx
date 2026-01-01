/**
 * Configuration management API client
 */

import { apiClient } from './client';
import type { KeyMapping } from '../types';

interface ConfigData {
  profileName: string;
  activeLayer: string;
  keyMappings: Record<string, KeyMapping>;
  layers: string[];
}

interface SetKeyMappingRequest {
  key: string;
  mapping: KeyMapping;
}

interface DeleteKeyMappingRequest {
  key: string;
}

interface ConfigResponse {
  success: boolean;
}

/**
 * Fetch configuration for a profile
 */
export async function fetchConfig(profile: string): Promise<ConfigData> {
  return apiClient.get<ConfigData>(`/api/config/${profile}`);
}

/**
 * Set or update a key mapping
 */
export async function setKeyMapping(
  profile: string,
  key: string,
  mapping: KeyMapping
): Promise<ConfigResponse> {
  const request: SetKeyMappingRequest = { key, mapping };
  return apiClient.put<ConfigResponse>(`/api/config/${profile}/key`, request);
}

/**
 * Delete a key mapping (restore to default)
 */
export async function deleteKeyMapping(
  profile: string,
  key: string
): Promise<ConfigResponse> {
  const request: DeleteKeyMappingRequest = { key };
  return apiClient.delete<ConfigResponse>(
    `/api/config/${profile}/key`,
    request
  );
}

/**
 * Export configuration as JSON
 */
export async function exportConfig(profile: string): Promise<string> {
  return apiClient.get<string>(`/api/config/${profile}/export`);
}

/**
 * Import configuration from JSON
 */
export async function importConfig(
  profile: string,
  configData: string
): Promise<ConfigResponse> {
  return apiClient.post<ConfigResponse>(`/api/config/${profile}/import`, {
    data: configData,
  });
}
