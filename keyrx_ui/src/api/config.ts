/**
 * Configuration management API client
 *
 * Uses dependency injection to avoid coupling to concrete HTTP client.
 * Import this module and pass an ApiClient instance to the functions.
 */

// Transport primitive types (defined inline to maintain API layer boundary)
interface MacroStep {
  type: 'press' | 'release' | 'delay';
  key?: string;
  delayMs?: number;
}

interface KeyMapping {
  type:
    | 'simple'
    | 'modifier'
    | 'lock'
    | 'tap_hold'
    | 'layer_active'
    | 'macro'
    | 'layer_switch';
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  modifierKey?: string;
  lockKey?: string;
  macroSteps?: MacroStep[];
  targetLayer?: string;
}

/**
 * Minimal API client interface for config operations.
 * Allows injection of mock clients in tests.
 */
export interface ConfigApiClient {
  get<T>(endpoint: string): Promise<T>;
  post<T>(endpoint: string, body?: unknown): Promise<T>;
  put<T>(endpoint: string, body?: unknown): Promise<T>;
  delete<T>(endpoint: string, body?: unknown): Promise<T>;
}

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
export async function fetchConfig(
  profile: string,
  client: ConfigApiClient
): Promise<ConfigData> {
  return client.get<ConfigData>(`/api/config/${profile}`);
}

/**
 * Set or update a key mapping
 */
export async function setKeyMapping(
  profile: string,
  key: string,
  mapping: KeyMapping,
  client: ConfigApiClient
): Promise<ConfigResponse> {
  const request: SetKeyMappingRequest = { key, mapping };
  return client.put<ConfigResponse>(`/api/config/${profile}/key`, request);
}

/**
 * Delete a key mapping (restore to default)
 */
export async function deleteKeyMapping(
  profile: string,
  key: string,
  client: ConfigApiClient
): Promise<ConfigResponse> {
  const request: DeleteKeyMappingRequest = { key };
  return client.delete<ConfigResponse>(`/api/config/${profile}/key`, request);
}

/**
 * Export configuration as JSON
 */
export async function exportConfig(
  profile: string,
  client: ConfigApiClient
): Promise<string> {
  return client.get<string>(`/api/config/${profile}/export`);
}

/**
 * Import configuration from JSON
 */
export async function importConfig(
  profile: string,
  configData: string,
  client: ConfigApiClient
): Promise<ConfigResponse> {
  return client.post<ConfigResponse>(`/api/config/${profile}/import`, {
    data: configData,
  });
}
