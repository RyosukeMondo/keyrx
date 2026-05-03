/**
 * Device management API client
 */

import { apiClient } from './client';
import {
  validateApiResponse,
  DeviceListResponseSchema,
  DeviceRpcInfoSchema,
} from './schemas';
import type { DeviceEntry, DeviceScope } from '../types';
import { setDeviceEnabled as persistDeviceEnabled } from '../utils/deviceStorage';

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
    enabled: true, // Default enabled; actual enabled/disabled state is managed at component layer via localStorage
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
    DeviceRpcInfoSchema,
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
    DeviceRpcInfoSchema,
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
  validateApiResponse(
    DeviceRpcInfoSchema,
    response,
    `DELETE /api/devices/${id}`
  );
  return { success: true };
}

/**
 * Set device enabled/disabled state (persisted to localStorage)
 *
 * Note: Backend doesn't support enabled state, so this is client-side only.
 * Persistence delegates to deviceStorage utility.
 */
export async function setDeviceEnabled(
  id: string,
  enabled: boolean
): Promise<DeviceResponse> {
  persistDeviceEnabled(id, enabled);
  return { success: true };
}
