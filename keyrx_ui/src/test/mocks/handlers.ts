/**
 * MSW (Mock Service Worker) request handlers
 * Defines mock API endpoints for integration testing
 */

import { http, HttpResponse } from 'msw';
import type { DeviceEntry, DeviceScope, ProfileEntry } from '../../types';

// Mock data
const mockDevices: DeviceEntry[] = [
  {
    id: 'device-1',
    name: 'Test Keyboard 1',
    path: '/dev/input/event0',
    scope: 'global' as DeviceScope,
    vendorId: 0x1234,
    productId: 0x5678,
  },
  {
    id: 'device-2',
    name: 'Test Keyboard 2',
    path: '/dev/input/event1',
    scope: 'local' as DeviceScope,
    vendorId: 0x1234,
    productId: 0x5679,
  },
];

const mockProfiles: ProfileEntry[] = [
  {
    name: 'default',
    displayName: 'Default Profile',
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    modifiedAt: '2024-01-01T00:00:00Z',
  },
  {
    name: 'gaming',
    displayName: 'Gaming Profile',
    isActive: false,
    createdAt: '2024-01-02T00:00:00Z',
    modifiedAt: '2024-01-02T00:00:00Z',
  },
];

export const handlers = [
  // Device endpoints
  http.get('/api/devices', () => {
    return HttpResponse.json(mockDevices);
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
    return HttpResponse.json({ success: true });
  }),

  http.put('/api/devices/:id/scope', async ({ request, params }) => {
    const { id } = params;
    const body = (await request.json()) as { scope: DeviceScope };

    const device = mockDevices.find((d) => d.id === id);
    if (!device) {
      return HttpResponse.json(
        { error: 'Device not found', errorCode: 'DEVICE_NOT_FOUND' },
        { status: 404 }
      );
    }

    device.scope = body.scope;
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

    mockDevices.splice(index, 1);
    return HttpResponse.json({ success: true });
  }),

  // Profile endpoints
  http.get('/api/profiles', () => {
    return HttpResponse.json(mockProfiles);
  }),

  http.post('/api/profiles', async ({ request }) => {
    const body = (await request.json()) as {
      name: string;
      displayName: string;
    };

    // Check for duplicate
    if (mockProfiles.find((p) => p.name === body.name)) {
      return HttpResponse.json(
        { error: 'Profile already exists', errorCode: 'PROFILE_EXISTS' },
        { status: 409 }
      );
    }

    const newProfile: ProfileEntry = {
      name: body.name,
      displayName: body.displayName,
      isActive: false,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
    };

    mockProfiles.push(newProfile);
    return HttpResponse.json({ success: true });
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
    return HttpResponse.json({ success: true });
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
