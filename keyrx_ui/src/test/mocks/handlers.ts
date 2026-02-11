/**
 * MSW (Mock Service Worker) request handlers
 * Defines mock API endpoints for integration testing
 */

import { http, HttpResponse } from 'msw';
import type { DeviceEntry } from '../../types';

interface MockProfile {
  name: string;
  rhaiPath: string;
  krxPath: string;
  modifiedAt: string;
  createdAt: string;
  layerCount: number;
  deviceCount: number;
  keyCount: number;
  isActive: boolean;
}

// Mock data
const initialDevices: DeviceEntry[] = [
  {
    id: 'device-1',
    name: 'Test Keyboard 1',
    path: '/dev/input/event0',
    vendorId: 0x1234,
    productId: 0x5678,
    serial: null,
    active: true,
    layout: 'ANSI_104',
    enabled: true,
    scope: 'Global',
    lastSeen: Date.now(),
    isVirtual: false,
  },
  {
    id: 'device-2',
    name: 'Test Keyboard 2',
    path: '/dev/input/event1',
    vendorId: 0x1234,
    productId: 0x5679,
    serial: null,
    active: true,
    layout: 'ANSI_104',
    enabled: true,
    scope: 'Global',
    lastSeen: Date.now(),
    isVirtual: false,
  },
];

const mockDevices: DeviceEntry[] = JSON.parse(JSON.stringify(initialDevices));

const initialProfiles: MockProfile[] = [
  {
    name: 'default',
    rhaiPath: '/home/user/.config/keyrx/profiles/default.rhai',
    krxPath: '/home/user/.config/keyrx/profiles/default.krx',
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    modifiedAt: '2024-01-01T00:00:00Z',
    layerCount: 1,
    deviceCount: 0,
    keyCount: 0,
  },
  {
    name: 'gaming',
    rhaiPath: '/home/user/.config/keyrx/profiles/gaming.rhai',
    krxPath: '/home/user/.config/keyrx/profiles/gaming.krx',
    isActive: false,
    createdAt: '2024-01-02T00:00:00Z',
    modifiedAt: '2024-01-02T00:00:00Z',
    layerCount: 2,
    deviceCount: 0,
    keyCount: 15,
  },
];

const mockProfiles: MockProfile[] = JSON.parse(JSON.stringify(initialProfiles));

export const handlers = [
  // Device endpoints
  http.get('/api/devices', () => {
    return HttpResponse.json({ devices: mockDevices });
  }),

  // Global settings endpoints
  http.get('/api/settings/global-layout', () => {
    return HttpResponse.json({ layout: 'ANSI_104' });
  }),

  http.put('/api/settings/global-layout', async ({ request }) => {
    const body = (await request.json()) as { layout: string };
    // In a real implementation, this would persist the layout
    return HttpResponse.json({ success: true });
  }),

  http.put('/api/devices/:id/name', async ({ request, params }) => {
    const { id } = params;
    const body = (await request.json()) as { name: string };

    const device = mockDevices.find((d) => d.id === id);
    if (!device) {
      return HttpResponse.json(
        { error: 'Device not found', errorCode: 'DEVICE_NOT_FOUND' },
        { status: 404 }
      );
    }

    device.name = body.name;
    // Return the updated device in DeviceEntry format
    const response: any = {
      ...device,
      last_seen: device.lastSeen || Date.now(),
    };
    // Remove null serial field to pass validation
    if (response.serial === null) {
      delete response.serial;
    }
    return HttpResponse.json(response);
  }),

  http.patch('/api/devices/:id', async ({ request, params }) => {
    const { id } = params;
    const body = (await request.json()) as {
      name?: string;
      layout?: string;
      enabled?: boolean;
    };

    const device = mockDevices.find((d) => d.id === id);
    if (!device) {
      return HttpResponse.json(
        { error: 'Device not found', errorCode: 'DEVICE_NOT_FOUND' },
        { status: 404 }
      );
    }

    if (body.name !== undefined) {
      device.name = body.name;
    }
    if (body.layout !== undefined) {
      device.layout = body.layout;
    }
    if (body.enabled !== undefined) {
      device.enabled = body.enabled;
    }

    return HttpResponse.json({ success: true });
  }),

  http.put('/api/devices/:id/enabled', async ({ request, params }) => {
    const { id } = params;
    const body = (await request.json()) as { enabled: boolean };

    const device = mockDevices.find((d) => d.id === id);
    if (!device) {
      return HttpResponse.json(
        { error: 'Device not found', errorCode: 'DEVICE_NOT_FOUND' },
        { status: 404 }
      );
    }

    device.enabled = body.enabled;
    return HttpResponse.json({ success: true });
  }),

  http.delete('/api/devices/:id', ({ params }) => {
    const { id } = params;
    const index = mockDevices.findIndex((d) => d.id === id);

    if (index === -1) {
      return HttpResponse.json(
        { error: 'Device not found', errorCode: 'DEVICE_NOT_FOUND' },
        { status: 404 }
      );
    }

    // Save the device before removing it
    const deletedDevice = mockDevices[index];
    mockDevices.splice(index, 1);

    // Return device in DeviceEntry format (with last_seen as snake_case)
    // Remove null serial field to pass validation (optional means omit, not null)
    const response: any = {
      ...deletedDevice,
      last_seen: deletedDevice.lastSeen || Date.now(),
    };
    if (response.serial === null) {
      delete response.serial;
    }
    return HttpResponse.json(response);
  }),

  // Profile endpoints
  http.get('/api/profiles', () => {
    return HttpResponse.json({ profiles: mockProfiles });
  }),

  http.get('/api/profiles/active', () => {
    const activeProfile = mockProfiles.find((p) => p.isActive);
    return HttpResponse.json({ active_profile: activeProfile?.name ?? null });
  }),

  http.post('/api/profiles', async ({ request }) => {
    const body = (await request.json()) as {
      name: string;
      template: string;
    };

    // Check for duplicate
    if (mockProfiles.find((p) => p.name === body.name)) {
      return HttpResponse.json(
        { error: 'Profile already exists', errorCode: 'PROFILE_EXISTS' },
        { status: 409 }
      );
    }

    const newProfile: MockProfile = {
      name: body.name,
      rhaiPath: `/home/user/.config/keyrx/profiles/${body.name}.rhai`,
      krxPath: `/home/user/.config/keyrx/profiles/${body.name}.krx`,
      isActive: false,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
      layerCount: 1,
      deviceCount: 0,
      keyCount: 0,
    };

    mockProfiles.push(newProfile);
    return HttpResponse.json({
      name: newProfile.name,
      rhaiPath: newProfile.rhaiPath,
      krxPath: newProfile.krxPath,
      modifiedAt: newProfile.modifiedAt,
      createdAt: newProfile.createdAt,
      layerCount: newProfile.layerCount,
      deviceCount: newProfile.deviceCount,
      keyCount: newProfile.keyCount,
      isActive: newProfile.isActive,
    });
  }),

  http.post('/api/profiles/:name/activate', ({ params }) => {
    const { name } = params;
    const profile = mockProfiles.find((p) => p.name === name);

    if (!profile) {
      return HttpResponse.json(
        { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
        { status: 404 }
      );
    }

    // Deactivate all profiles
    mockProfiles.forEach((p) => {
      p.isActive = false;
    });

    // Activate the target profile
    profile.isActive = true;
    return HttpResponse.json({
      success: true,
      compile_time_ms: 42,
      reload_time_ms: 10,
    });
  }),

  http.delete('/api/profiles/:name', ({ params }) => {
    const { name } = params;

    // Cannot delete active profile
    const profile = mockProfiles.find((p) => p.name === name);
    if (profile?.isActive) {
      return HttpResponse.json(
        {
          error: 'Cannot delete active profile',
          errorCode: 'PROFILE_ACTIVE',
        },
        { status: 400 }
      );
    }

    const index = mockProfiles.findIndex((p) => p.name === name);
    if (index === -1) {
      return HttpResponse.json(
        { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
        { status: 404 }
      );
    }

    mockProfiles.splice(index, 1);
    return HttpResponse.json({ success: true });
  }),

  // Profile validation endpoint
  http.post('/api/profiles/:name/validate', ({ params }) => {
    const { name } = params;
    const profile = mockProfiles.find((p) => p.name === name);

    if (!profile) {
      return HttpResponse.json(
        { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
        { status: 404 }
      );
    }

    // Mock validation - always return valid for test profiles
    return HttpResponse.json({
      valid: true,
      errors: [],
    });
  }),

  // Config endpoints
  http.get('/api/config/:profile', ({ params }) => {
    const { profile } = params;

    if (!mockProfiles.find((p) => p.name === profile)) {
      return HttpResponse.json(
        { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
        { status: 404 }
      );
    }

    return HttpResponse.json({
      layers: [
        {
          id: 'base',
          name: 'Base Layer',
          mappings: {},
        },
      ],
    });
  }),

  http.put('/api/config/:profile/key', async ({ request, params }) => {
    const { profile } = params;
    const body = await request.json();

    if (!mockProfiles.find((p) => p.name === profile)) {
      return HttpResponse.json(
        { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
        { status: 404 }
      );
    }

    return HttpResponse.json({ success: true });
  }),

  // Metrics endpoints
  http.get('/api/metrics/latency', () => {
    return HttpResponse.json({
      average: 1.2,
      min: 0.5,
      max: 3.8,
      p50: 1.1,
      p95: 2.5,
      p99: 3.2,
    });
  }),

  http.get('/api/metrics/events', () => {
    return HttpResponse.json({
      events: [
        {
          timestamp: Date.now() - 5000,
          type: 'key_press',
          keyCode: 'KEY_A',
          outputCode: 'KEY_B',
          layer: 'base',
          latency: 1.2,
        },
        {
          timestamp: Date.now() - 4000,
          type: 'key_release',
          keyCode: 'KEY_A',
          outputCode: 'KEY_B',
          layer: 'base',
          latency: 1.1,
        },
      ],
      total: 2,
    });
  }),
];

/**
 * Reset mock data to initial state
 * Call this in afterEach to ensure test isolation
 */
export function resetMockData() {
  mockProfiles.length = 0;
  mockProfiles.push(...JSON.parse(JSON.stringify(initialProfiles)));

  mockDevices.length = 0;
  mockDevices.push(...JSON.parse(JSON.stringify(initialDevices)));
}
