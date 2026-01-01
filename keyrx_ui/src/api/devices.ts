/**
 * Device management API client
 */

import { apiClient } from './client';
import type { DeviceEntry, DeviceScope } from '../types';

interface RenameDeviceRequest {
  name: string;
}

interface SetScopeRequest {
  scope: DeviceScope;
}

interface DeviceResponse {
  success: boolean;
}

/**
 * Fetch all connected devices
 */
export async function fetchDevices(): Promise<DeviceEntry[]> {
  return apiClient.get<DeviceEntry[]>('/api/devices');
}

/**
 * Rename a device
 */
export async function renameDevice(
  id: string,
  name: string
): Promise<DeviceResponse> {
  const request: RenameDeviceRequest = { name };
  return apiClient.put<DeviceResponse>(`/api/devices/${id}/name`, request);
}

/**
 * Set device scope (global or local)
 */
export async function setDeviceScope(
  id: string,
  scope: DeviceScope
): Promise<DeviceResponse> {
  const request: SetScopeRequest = { scope };
  return apiClient.put<DeviceResponse>(`/api/devices/${id}/scope`, request);
}

/**
 * Forget a device (remove from device list)
 */
export async function forgetDevice(id: string): Promise<DeviceResponse> {
  return apiClient.delete<DeviceResponse>(`/api/devices/${id}`);
}
