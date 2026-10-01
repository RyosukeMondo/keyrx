/**
 * Device management API client
 */

import { apiClient } from './client';
import {
  validateApiResponse,
  DeviceListResponseSchema,
  DeviceEntrySchema,
  SuccessResponseSchema,
} from './schemas';
import type { DeviceEntry } from '../types';
import type { GlobalLayout } from '../types/generated';

interface RenameDeviceRequest {
  name: string;
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
 * Forget a device (remove from device list)
 */
export async function forgetDevice(id: string): Promise<DeviceResponse> {
  const response = await apiClient.delete<{ success: boolean }>(
    `/api/devices/${id}`
  );
  validateApiResponse(
    SuccessResponseSchema,
    response,
    `DELETE /api/devices/${id}`
  );
  return { success: true };
}

/**
 * Fetch the global default keyboard layout (`ANSI_104` when unset)
 */
export async function fetchGlobalLayout(): Promise<string> {
  return (await fetchGlobalLayoutSetting()) ?? 'ANSI_104';
}

/**
 * Fetch the stored global layout, or null when the user never chose one
 * (lets callers tell "unset" apart from an explicit ANSI_104).
 */
export async function fetchGlobalLayoutSetting(): Promise<string | null> {
  const response = await apiClient.get<GlobalLayout>(
    '/api/settings/global-layout'
  );
  return response.layout ?? null;
}

/**
 * Set the global default keyboard layout
 */
export async function setGlobalLayout(layout: string): Promise<void> {
  await apiClient.put('/api/settings/global-layout', { layout });
}
