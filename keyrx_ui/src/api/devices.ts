/**
 * Device management API client
 */

import { apiClient } from './client';
import {
  validateApiResponse,
  DeviceListResponseSchema,
  DeviceEntrySchema,
} from './schemas';
import type { DeviceEntry, DeviceScope } from '../types';
import * as deviceStorage from '../utils/deviceStorage';

interface RenameDeviceRequest {
  name: string;
}

interface SetScopeRequest {
  scope: DeviceScope;
}

interface DeviceResponse {
  success: boolean;
}

interface DevicesListResponse {
  devices: DeviceEntry[];
}

/**
 * Fetch all connected devices
 */
export async function fetchDevices(): Promise<DeviceEntry[]> {
  const response = await apiClient.get<DevicesListResponse>('/api/devices');
  const validated = validateApiResponse(
    DeviceListResponseSchema,
    response,
    'GET /api/devices'
  );

  // Map validated response to DeviceEntry format
  // The REST API returns DeviceRpcInfo format (id, name, path, serial, active, scope?, layout?)
  return validated.devices.map((device) => ({
    id: device.id,
    name: device.name,
    path: device.path,
    serial: device.serial || null,
    active: device.active,
    scope:
      device.scope === 'DeviceSpecific'
        ? 'device-specific'
        : device.scope === 'Global'
          ? 'global'
          : 'global', // Default to global if unset
    layout: device.layout || null,
    isVirtual: device.name.toLowerCase().startsWith('keyrx'), // Virtual if name starts with "keyrx" (daemon's uinput device)
    enabled: deviceStorage.isDeviceEnabled(device.id), // Load enabled state from localStorage
  }));
}

/**
 * Rename a device
 */
export async function renameDevice(
  id: string,
  name: string
): Promise<DeviceResponse> {
  const request: RenameDeviceRequest = { name };
  const response = await apiClient.put<DeviceEntry>(
    `/api/devices/${id}/name`,
    request
  );
  // Validate the returned device entry
  validateApiResponse(
    DeviceEntrySchema,
    response,
    `PUT /api/devices/${id}/name`
  );
  return { success: true };
}

/**
 * Set device scope (global or local)
 */
export async function setDeviceScope(
  id: string,
  scope: DeviceScope
): Promise<DeviceResponse> {
  const request: SetScopeRequest = { scope };
  const response = await apiClient.put<DeviceEntry>(
    `/api/devices/${id}/scope`,
    request
  );
  // Validate the returned device entry
  validateApiResponse(
    DeviceEntrySchema,
    response,
    `PUT /api/devices/${id}/scope`
  );
  return { success: true };
}

/**
 * Forget a device (remove from device list)
 */
export async function forgetDevice(id: string): Promise<DeviceResponse> {
  const response = await apiClient.delete<DeviceEntry>(`/api/devices/${id}`);
  // Validate the returned device entry
  validateApiResponse(DeviceEntrySchema, response, `DELETE /api/devices/${id}`);
  // Clean up enabled state from localStorage
  deviceStorage.removeDeviceEnabledState(id);
  return { success: true };
}

/**
 * Set device enabled/disabled state
 *
 * Note: Backend doesn't currently support enabled state, so we persist
 * this client-side using localStorage. This allows users to hide devices
 * without permanently forgetting them.
 */
export async function setDeviceEnabled(
  id: string,
  enabled: boolean
): Promise<DeviceResponse> {
  // Persist to localStorage
  deviceStorage.setDeviceEnabled(id, enabled);

  // Return success immediately since this is client-side only
  return { success: true };
}
